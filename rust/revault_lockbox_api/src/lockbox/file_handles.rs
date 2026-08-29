use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Read, Seek, SeekFrom, Write};

use zeroize::Zeroizing;

use super::file_import_pipeline::CompressionFrameWrite;
use super::files::{
    ranges_overlap, seek_position, to_io_error, trim_zeroes, FilePageWriter,
    FILE_COMPRESSION_FRAME_BYTES,
};
use super::{Lockbox, Writable};
use crate::constants::DEFAULT_FILE_PERMISSIONS;
use crate::lockbox_path::LockboxPath;
use crate::node_kind::NodeKind;
use crate::security::validate_permissions;
use crate::toc_entry::TocEntry;
use crate::{Error, Result};

/// Options used when opening a writable file handle inside a lockbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenFileOptions {
    /// Create the file if it does not already exist.
    pub create: bool,
    /// Truncate the file to zero bytes when it is opened.
    pub truncate: bool,
    /// Unix-style permission bits to use for newly-created files.
    pub permissions: Option<u32>,
}

impl OpenFileOptions {
    /// Open an existing file without creating or truncating it.
    pub const fn existing() -> Self {
        Self {
            create: false,
            truncate: false,
            permissions: None,
        }
    }

    /// Open a file, creating it when it does not already exist.
    pub const fn create() -> Self {
        Self {
            create: true,
            truncate: false,
            permissions: None,
        }
    }

    /// Open a file, creating it when needed and truncating it to zero bytes.
    pub const fn create_truncate() -> Self {
        Self {
            create: true,
            truncate: true,
            permissions: None,
        }
    }
}

impl Default for OpenFileOptions {
    fn default() -> Self {
        Self::existing()
    }
}

/// Seekable read handle over a file inside a lockbox.
pub struct LockboxFileReader<'a, State = Writable> {
    lockbox: &'a Lockbox<State>,
    path: LockboxPath,
    position: u64,
    len: u64,
    cache_page_index: Option<u64>,
    cache_page: Zeroizing<Vec<u8>>,
}

/// Seekable read/write handle over a file inside a writable lockbox.
pub struct LockboxFileMut<'a> {
    lockbox: &'a mut Lockbox<Writable>,
    path: LockboxPath,
    position: u64,
    len: u64,
    permissions: u32,
    exists_on_open: bool,
    truncate_existing: bool,
    dirty_pages: BTreeMap<u64, Zeroizing<Vec<u8>>>,
    closed: bool,
}

impl Lockbox<Writable> {
    /// Open a seekable read/write handle over a file inside the lockbox.
    pub fn open_file_for_write(
        &mut self,
        path: &LockboxPath,
        options: OpenFileOptions,
    ) -> Result<LockboxFileMut<'_>> {
        let path = path.file_path()?;
        self.ensure_mirror_path_mutable(&path)?;
        let permissions = validate_permissions(
            options.permissions.unwrap_or(
                self.toc_entries
                    .get(path.as_str())
                    .filter(|entry| !entry.deleted)
                    .map(|entry| entry.permissions)
                    .unwrap_or(DEFAULT_FILE_PERMISSIONS),
            ),
        )?;
        let existing = self
            .toc_entries
            .get(path.as_str())
            .filter(|entry| !entry.deleted)
            .cloned();
        if let Some(entry) = existing.as_ref() {
            if entry.node_kind != NodeKind::File {
                return Err(Error::InvalidOperation(format!(
                    "{} is not a file",
                    entry.path.as_str()
                )));
            }
        } else if !options.create {
            return Err(Error::NotFound(path.to_string()));
        } else {
            self.ensure_parent_directory(&path)?;
        }
        let len = if options.truncate {
            0
        } else {
            existing.as_ref().map(|entry| entry.len).unwrap_or(0)
        };
        Ok(LockboxFileMut {
            lockbox: self,
            path,
            position: 0,
            len,
            permissions,
            exists_on_open: existing.is_some(),
            truncate_existing: options.truncate,
            dirty_pages: BTreeMap::new(),
            closed: false,
        })
    }
}

impl<'a, State> LockboxFileReader<'a, State> {
    pub(super) fn new(lockbox: &'a Lockbox<State>, path: LockboxPath, len: u64) -> Self {
        Self {
            lockbox,
            path,
            position: 0,
            len,
            cache_page_index: None,
            cache_page: Zeroizing::new(Vec::new()),
        }
    }

    /// Current logical file length.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Whether this file is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn read_internal(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() || self.position >= self.len {
            return Ok(0);
        }
        let mut total = 0usize;
        while total < buf.len() && self.position < self.len {
            let page_index = self.position / FILE_COMPRESSION_FRAME_BYTES as u64;
            let page_start = page_index * FILE_COMPRESSION_FRAME_BYTES as u64;
            if self.cache_page_index != Some(page_index) {
                let page_len = (FILE_COMPRESSION_FRAME_BYTES as u64).min(self.len - page_start);
                self.cache_page = Zeroizing::new(
                    self.lockbox
                        .read_file_range(&self.path, page_start, page_len)?,
                );
                self.cache_page_index = Some(page_index);
            }
            let page_offset = (self.position - page_start) as usize;
            let available = self.cache_page.len().saturating_sub(page_offset);
            if available == 0 {
                break;
            }
            let take = (buf.len() - total)
                .min(available)
                .min((self.len - self.position) as usize);
            buf[total..total + take]
                .copy_from_slice(&self.cache_page[page_offset..page_offset + take]);
            self.position += take as u64;
            total += take;
        }
        Ok(total)
    }
}

impl<'a, State> Read for LockboxFileReader<'a, State> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read_internal(buf).map_err(to_io_error)
    }
}

impl<'a, State> Seek for LockboxFileReader<'a, State> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.position = seek_position(self.position, self.len, pos)?;
        Ok(self.position)
    }
}

impl<'a> LockboxFileMut<'a> {
    /// Current logical file length.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Whether this file is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Flush dirty logical pages into the lockbox state.
    ///
    /// This does not call `Lockbox::commit`; callers retain the existing
    /// lockbox-level transaction boundary.
    pub fn flush(&mut self) -> Result<()> {
        let rollback = crate::lockbox::commit::CommitRollback::capture(self.lockbox);
        match self.flush_inner() {
            Ok(()) => Ok(()),
            Err(err) => {
                rollback.restore(self.lockbox);
                Err(err)
            }
        }
    }

    fn flush_inner(&mut self) -> Result<()> {
        if self.dirty_pages.is_empty() && !self.truncate_existing && self.exists_on_open {
            return Ok(());
        }

        let old = self
            .lockbox
            .toc_entries
            .get(self.path.as_str())
            .filter(|entry| !entry.deleted && entry.node_kind == NodeKind::File)
            .cloned();
        let mut kept_chunks = Vec::new();
        let mut removed_chunks = Vec::new();
        let dirty_ranges = self
            .dirty_pages
            .keys()
            .map(|page_index| {
                let start = page_index.saturating_mul(FILE_COMPRESSION_FRAME_BYTES as u64);
                (
                    start,
                    start.saturating_add(FILE_COMPRESSION_FRAME_BYTES as u64),
                )
            })
            .collect::<Vec<_>>();

        if let Some(entry) = old.as_ref() {
            for chunk in &entry.chunks {
                let chunk_end = chunk.file_offset.saturating_add(chunk.len);
                let dirty = dirty_ranges
                    .iter()
                    .any(|(start, end)| ranges_overlap(chunk.file_offset, chunk_end, *start, *end));
                if self.truncate_existing || chunk_end > self.len || dirty {
                    removed_chunks.push(chunk.clone());
                } else {
                    kept_chunks.push(chunk.clone());
                }
            }
        }

        if !removed_chunks.is_empty() {
            let kept_frame_ids = kept_chunks
                .iter()
                .map(|chunk| chunk.compression_frame_id)
                .collect::<BTreeSet<_>>();
            let removed_entry = TocEntry {
                path: self.path.clone(),
                len: old.as_ref().map(|entry| entry.len).unwrap_or(0),
                record_offset: old.as_ref().map(|entry| entry.record_offset).unwrap_or(0),
                record_len: old.as_ref().map(|entry| entry.record_len).unwrap_or(0),
                record_object_id: old
                    .as_ref()
                    .map(|entry| entry.record_object_id)
                    .unwrap_or(0),
                deleted: false,
                node_kind: NodeKind::File,
                permissions: self.permissions,
                chunks: removed_chunks.clone(),
            };
            self.lockbox
                .rewrite_shared_compression_frames_before_removal(&removed_entry)?;
            for chunk in &removed_chunks {
                if kept_frame_ids.contains(&chunk.compression_frame_id) {
                    continue;
                }
                for segment in &chunk.segments {
                    self.lockbox.schedule_page_object_redaction(
                        segment.page_offset,
                        segment.page_len,
                        segment.object_id,
                    )?;
                }
            }
        }

        self.lockbox.remove_pending_small_file(&self.path);

        let mut new_chunks = Vec::new();
        let mut dirty_writes = Vec::new();
        for (page_index, page) in &self.dirty_pages {
            let page_start = page_index.saturating_mul(FILE_COMPRESSION_FRAME_BYTES as u64);
            if page_start >= self.len {
                continue;
            }
            let actual_len =
                (FILE_COMPRESSION_FRAME_BYTES as u64).min(self.len - page_start) as usize;
            if let Some((start, end)) = trim_zeroes(&page[..actual_len]) {
                dirty_writes.push((
                    page_start + start as u64,
                    Zeroizing::new(page[start..end].to_vec()),
                ));
            }
        }
        {
            let mut writer = FilePageWriter::new(&mut *self.lockbox);
            for (file_offset, data) in &dirty_writes {
                writer.write_compression_frame(
                    CompressionFrameWrite {
                        path: &self.path,
                        permissions: self.permissions,
                        total_len: self.len,
                        file_offset: *file_offset,
                        data,
                    },
                    &mut new_chunks,
                )?;
            }
            writer.finish(&mut new_chunks)?;
        }

        kept_chunks.extend(new_chunks.clone());
        kept_chunks.sort_by_key(|chunk| chunk.file_offset);
        let record_offset = kept_chunks
            .first()
            .and_then(|chunk| chunk.segments.first())
            .map(|segment| segment.page_offset)
            .unwrap_or(0);
        let record_len = kept_chunks
            .first()
            .and_then(|chunk| chunk.segments.first())
            .map(|segment| segment.page_len)
            .unwrap_or(0);
        let record_object_id = kept_chunks
            .first()
            .and_then(|chunk| chunk.segments.first())
            .map(|segment| segment.object_id)
            .unwrap_or(0);

        let entry = TocEntry {
            path: self.path.clone(),
            len: self.len,
            record_offset,
            record_len,
            record_object_id,
            deleted: false,
            node_kind: NodeKind::File,
            permissions: self.permissions,
            chunks: kept_chunks,
        };
        if !new_chunks.is_empty() {
            let new_ref_entry = TocEntry {
                chunks: new_chunks,
                ..entry.clone()
            };
            self.lockbox.add_entry_record_refs(&new_ref_entry);
        }
        self.lockbox.toc_entries.insert(self.path.clone(), entry);
        self.lockbox.mark_toc_dirty(&self.path);
        self.lockbox.needs_packing = true;
        self.dirty_pages.clear();
        self.exists_on_open = true;
        self.truncate_existing = false;
        Ok(())
    }

    /// Flush and close the handle.
    pub fn close(mut self) -> Result<()> {
        self.flush()?;
        self.closed = true;
        Ok(())
    }

    fn read_internal(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() || self.position >= self.len {
            return Ok(0);
        }
        let mut total = 0usize;
        while total < buf.len() && self.position < self.len {
            let page_index = self.position / FILE_COMPRESSION_FRAME_BYTES as u64;
            let page_start = page_index * FILE_COMPRESSION_FRAME_BYTES as u64;
            let page_offset = (self.position - page_start) as usize;
            let take = (buf.len() - total)
                .min(FILE_COMPRESSION_FRAME_BYTES - page_offset)
                .min((self.len - self.position) as usize);
            if let Some(page) = self.dirty_pages.get(&page_index) {
                buf[total..total + take].copy_from_slice(&page[page_offset..page_offset + take]);
            } else if self.exists_on_open && !self.truncate_existing {
                let data = self
                    .lockbox
                    .read_file_range(&self.path, self.position, take as u64)?;
                let read = data.len().min(take);
                buf[total..total + read].copy_from_slice(&data[..read]);
                if read < take {
                    buf[total + read..total + take].fill(0);
                }
            } else {
                buf[total..total + take].fill(0);
            }
            self.position += take as u64;
            total += take;
        }
        Ok(total)
    }

    fn write_internal(&mut self, buf: &[u8]) -> Result<usize> {
        let mut total = 0usize;
        while total < buf.len() {
            let page_index = self.position / FILE_COMPRESSION_FRAME_BYTES as u64;
            let page_start = page_index * FILE_COMPRESSION_FRAME_BYTES as u64;
            let page_offset = (self.position - page_start) as usize;
            let take = (buf.len() - total).min(FILE_COMPRESSION_FRAME_BYTES - page_offset);
            if !self.dirty_pages.contains_key(&page_index) {
                let mut page = if self.exists_on_open && !self.truncate_existing {
                    self.lockbox.read_file_range(
                        &self.path,
                        page_start,
                        FILE_COMPRESSION_FRAME_BYTES as u64,
                    )?
                } else {
                    Vec::new()
                };
                page.resize(FILE_COMPRESSION_FRAME_BYTES, 0);
                self.dirty_pages.insert(page_index, Zeroizing::new(page));
            }
            let page = self
                .dirty_pages
                .get_mut(&page_index)
                .ok_or_else(|| Error::InvalidOperation("dirty page missing".to_string()))?;
            page[page_offset..page_offset + take].copy_from_slice(&buf[total..total + take]);
            self.position = self.position.saturating_add(take as u64);
            self.len = self.len.max(self.position);
            total += take;
        }
        Ok(total)
    }
}

impl<'a> Read for LockboxFileMut<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read_internal(buf).map_err(to_io_error)
    }
}

impl<'a> Write for LockboxFileMut<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_internal(buf).map_err(to_io_error)
    }

    fn flush(&mut self) -> io::Result<()> {
        LockboxFileMut::flush(self).map_err(to_io_error)
    }
}

impl<'a> Seek for LockboxFileMut<'a> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.position = seek_position(self.position, self.len, pos)?;
        Ok(self.position)
    }
}

impl<'a> Drop for LockboxFileMut<'a> {
    fn drop(&mut self) {
        if !self.closed {
            if let Err(err) = self.flush() {
                self.lockbox.poisoned = Some(err.to_string());
            }
        }
    }
}
