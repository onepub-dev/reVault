//! Handle-local staged block reader. Its storage borrow retains the archive's
//! lock; its small authenticated index and data window live only with the handle.
use super::Lockbox;
use crate::file_format::indexed_frame::block_page::Reader;
use crate::page_buffer::ZeroizingBytes;
use crate::{LockboxPath, Result};

pub(super) struct NativeFileReader<'a> {
    reader: Reader<'a>,
    file_start: u64,
    file_end: u64,
    frame_start: u64,
    compressed: bool,
    window_start: u64,
    window: ZeroizingBytes,
}

impl<'a> NativeFileReader<'a> {
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
        let reader = archive.open_native_block_chunk(entry.len, chunk)?;
        Ok(Some(Self {
            reader,
            file_start: chunk.file_offset,
            file_end: chunk.file_offset + chunk.len,
            frame_start: chunk.compression_frame_offset,
            compressed: chunk.compression != crate::compression::COMPRESSION_NONE,
            window_start: 0,
            window: ZeroizingBytes::new(Vec::new()),
        }))
    }

    pub(super) fn contains(&self, position: u64) -> bool {
        (self.file_start..self.file_end).contains(&position)
    }

    pub(super) fn read(&mut self, position: u64, out: &mut [u8]) -> Result<usize> {
        // A cache hit must not bypass replacement/truncation or a latched
        // external-source failure, even though no source bytes are fetched.
        self.reader.ensure_current()?;
        let relative = position - self.file_start;
        let len = self.file_end - self.file_start;
        if relative < self.window_start || relative - self.window_start >= self.window.len() as u64
        {
            let (start, end) = if self.compressed {
                (0, len)
            } else {
                const BLOCK: u64 = crate::file_format::indexed_frame::BLOCK_BYTES as u64;
                // Align to the shared frame, not this packed file's start.
                let frame_position = self.frame_start + relative;
                let start =
                    (frame_position / BLOCK * BLOCK).max(self.frame_start) - self.frame_start;
                let requested = (out.len() as u64).min(len - relative);
                let end = (self.frame_start + relative + requested).div_ceil(BLOCK) * BLOCK;
                (start, (end - self.frame_start).min(len))
            };
            self.window = ZeroizingBytes::new(
                self.reader
                    .read(self.frame_start + start..self.frame_start + end)?,
            );
            self.window_start = start;
        }
        let offset = (relative - self.window_start) as usize;
        let count = out.len().min(self.window.len() - offset);
        out[..count].copy_from_slice(&self.window[offset..offset + count]);
        Ok(count)
    }
}
