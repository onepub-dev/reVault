//! Native file reader with owned or shared verified windows. Its storage borrow
//! retains the archive lock while complete decoded windows share cache ownership.
use super::Lockbox;
use crate::file_format::indexed_frame::block_page::Reader;
use crate::page_buffer::ZeroizingBytes;
use crate::storage::StorageBackend;
use crate::{Error, LockboxPath, Result};

enum NativeWindow {
    Owned(ZeroizingBytes),
    Shared {
        frame: std::sync::Arc<super::CachedCompressionFrame>,
        range: std::ops::Range<usize>,
    },
}

impl From<super::files::NativeReadBytes> for NativeWindow {
    fn from(bytes: super::files::NativeReadBytes) -> Self {
        match bytes {
            super::files::NativeReadBytes::Owned(bytes) => Self::Owned(ZeroizingBytes::new(bytes)),
            super::files::NativeReadBytes::Shared { frame, range, .. } => {
                // The handle retains its storage borrow and independently checks
                // the same full extent and revision before every window copy.
                // A small packed slice must not pin its larger neighbors after
                // cache eviction. Whole-frame handles retain no more decoded
                // bytes than their previous owned window.
                if range.len() == frame.data.len() {
                    Self::Shared { frame, range }
                } else {
                    Self::Owned(ZeroizingBytes::new(frame.data[range].to_vec()))
                }
            }
        }
    }
}

impl std::ops::Deref for NativeWindow {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self {
            Self::Owned(bytes) => bytes,
            Self::Shared { frame, range } => &frame.data[range.clone()],
        }
    }
}

pub(super) struct NativeFileReader<'a> {
    reader: Option<Reader<'a>>,
    storage: &'a StorageBackend,
    page_cache: &'a std::cell::RefCell<crate::page_cache::PageCache>,
    required_end: u64,
    revision: u64,
    file_start: u64,
    file_end: u64,
    frame_start: u64,
    window_start: u64,
    window: NativeWindow,
}

impl<'a> NativeFileReader<'a> {
    #[cfg(all(test, feature = "native-block-layout"))]
    pub(super) fn retained_window_for_tests(&self) -> &[u8] {
        &self.window
    }

    pub(super) fn open<State>(
        archive: &'a Lockbox<State>,
        path: &LockboxPath,
        position: u64,
    ) -> Result<Option<Self>> {
        let Some(entry) = archive.toc_entries.get(path) else {
            return Ok(None);
        };
        if archive.pending_small_files.contains_key(path) {
            return Ok(None);
        }
        let Some(chunk) = entry.chunks.iter().find(|chunk| {
            chunk.block_frame.is_some()
                && position >= chunk.file_offset
                && position - chunk.file_offset < chunk.len
        }) else {
            return Ok(None);
        };
        let [segment] = chunk.segments.as_slice() else {
            return Err(Error::CorruptRecord);
        };
        let required_end = segment
            .page_offset
            .checked_add(segment.page_len)
            .ok_or(Error::CorruptRecord)?;
        let revision = archive.storage.write_revision();
        let compressed = chunk.compression != crate::compression::COMPRESSION_NONE;
        // Compressed reads already decode the entire frame. Share complete
        // verified windows; packed slices keep bounded owned copies.
        // A complete small frame occupies at most one raw data block. Capturing
        // its bounded allocation also avoids separate metadata/index reads,
        // without expanding a raw range to additional data blocks.
        let small_frame = chunk.compression_frame_offset == 0
            && chunk.len == chunk.compression_frame_len
            && chunk.len <= crate::file_format::indexed_frame::BLOCK_BYTES as u64
            && segment.page_len <= crate::file_format::indexed_frame::BLOCK_BYTES as u64;
        let (reader, window) = if compressed || small_frame {
            (
                None,
                NativeWindow::from(archive.read_native_file_window(entry.len, chunk)?),
            )
        } else {
            (
                Some(archive.open_native_block_chunk(entry.len, chunk)?),
                NativeWindow::Owned(ZeroizingBytes::new(Vec::new())),
            )
        };
        Ok(Some(Self {
            reader,
            storage: &archive.storage,
            page_cache: &archive.page_manager,
            required_end,
            revision,
            file_start: chunk.file_offset,
            file_end: chunk.file_offset + chunk.len,
            frame_start: chunk.compression_frame_offset,
            window_start: 0,
            window,
        }))
    }

    pub(super) fn contains(&self, position: u64) -> bool {
        (self.file_start..self.file_end).contains(&position)
    }

    pub(super) fn read(&mut self, position: u64, out: &mut [u8]) -> Result<usize> {
        let relative = position - self.file_start;
        let len = self.file_end - self.file_start;
        if relative < self.window_start || relative - self.window_start >= self.window.len() as u64
        {
            let reader = self.reader.as_ref().ok_or(Error::CorruptRecord)?;
            let count = out.len().min((len - relative) as usize);
            let frame_position = self.frame_start + relative;
            if count >= crate::file_format::indexed_frame::BLOCK_BYTES
                && reader.read_aligned_raw_into(
                    frame_position..frame_position + count as u64,
                    &mut out[..count],
                )?
            {
                return Ok(count);
            }
            let (start, end) = {
                const BLOCK: u64 = crate::file_format::indexed_frame::BLOCK_BYTES as u64;
                // Align to the shared frame, not this packed file's start.
                let frame_position = self.frame_start + relative;
                let start =
                    (frame_position / BLOCK * BLOCK).max(self.frame_start) - self.frame_start;
                let requested = (out.len() as u64).min(len - relative);
                let end = (self.frame_start + relative + requested).div_ceil(BLOCK) * BLOCK;
                (start, (end - self.frame_start).min(len))
            };
            self.window = NativeWindow::Owned(ZeroizingBytes::new(reader.read_with_cache(
                self.frame_start + start..self.frame_start + end,
                self.page_cache,
            )?));
            self.window_start = start;
        } else {
            // Misses validate in the frame reader. Hits still check identity,
            // full padded extent and latched external failures before copying.
            if self.required_end > self.storage.current_len()? {
                return Err(Error::Truncated);
            }
            if self.revision == u64::MAX || self.revision != self.storage.write_revision() {
                return Err(Error::CorruptRecord);
            }
        }
        let offset = (relative - self.window_start) as usize;
        let count = out.len().min(self.window.len() - offset);
        out[..count].copy_from_slice(&self.window[offset..offset + count]);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_windows_release_owners_and_do_not_pin_packed_neighbors() {
        for packed in [false, true] {
            let frame = std::sync::Arc::new(super::super::CachedCompressionFrame {
                native_reference: None,
                compression: 1,
                compression_frame_len: 8192,
                compressed_len: 64,
                compression_frame_digest: [3; 32],
                slices: Vec::new(),
                data: vec![7; 8192],
            });
            let weak = std::sync::Arc::downgrade(&frame);
            let pointer = frame.data.as_ptr();
            let range = if packed { 7..20 } else { 0..8192 };
            let window = NativeWindow::from(super::super::files::NativeReadBytes::Shared {
                frame,
                range,
                source_guard: None,
            });
            assert_eq!(window.len(), if packed { 13 } else { 8192 });
            assert!(window.iter().all(|byte| *byte == 7));
            if packed {
                assert!(
                    weak.upgrade().is_none(),
                    "a small handle cannot keep its packed neighbors alive"
                );
            } else {
                assert!(weak.upgrade().is_some());
                assert_eq!(
                    window.as_ptr(),
                    pointer,
                    "complete windows reuse the verified allocation"
                );
            }
            drop(window);
            assert!(
                weak.upgrade().is_none(),
                "the last handle must release its zeroizing frame owner"
            );
        }
    }
}
