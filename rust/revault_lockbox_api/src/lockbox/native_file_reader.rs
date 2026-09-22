//! Handle-local staged block reader. Its storage borrow retains the archive's
//! lock; its small authenticated index and data window live only with the handle.
use super::Lockbox;
use crate::file_format::indexed_frame::block_page::Reader;
use crate::page_buffer::ZeroizingBytes;
use crate::storage::StorageBackend;
use crate::{Error, LockboxPath, Result};

pub(super) struct NativeFileReader<'a> {
    reader: Option<Reader<'a>>,
    storage: &'a StorageBackend,
    required_end: u64,
    revision: u64,
    file_start: u64,
    file_end: u64,
    frame_start: u64,
    window_start: u64,
    window: ZeroizingBytes,
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
        // Compressed reads already decode the entire frame. Share its verified
        // decoded cache across handles, retaining only this file's slice locally.
        let (reader, window) = if compressed {
            (
                None,
                ZeroizingBytes::new(archive.read_file_chunk_compression_frame(entry.len, chunk)?),
            )
        } else {
            (
                Some(archive.open_native_block_chunk(entry.len, chunk)?),
                ZeroizingBytes::new(Vec::new()),
            )
        };
        Ok(Some(Self {
            reader,
            storage: &archive.storage,
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
            self.window =
                ZeroizingBytes::new(reader.read(self.frame_start + start..self.frame_start + end)?);
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
