use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Cursor, Read, SeekFrom, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::file_handles::LockboxFileReader;
use super::file_import_pipeline::{
    CompressionFrameWrite, FileImportPipeline, ParallelCompressionJob, ParallelCompressionResult,
    PreparedCompressionFrame,
};
use super::Lockbox;
use crate::compression::{
    decode_compression_frame, validate_compression_frame_lengths, COMPRESSION_NONE,
    ZSTD_BULK_IMPORT_LEVEL, ZSTD_DEFAULT_LEVEL,
};
use crate::compression_frame_manifest::{CompressionFrameManifest, CompressionFrameSlice};
use crate::constants::{
    DEFAULT_FILE_PERMISSIONS, DEFAULT_MAX_PAGE_BODY_BYTES, DEFAULT_MAX_PAGE_LOGICAL_BYTES,
};
use crate::crypto::strong_checksum;
use crate::file_chunk::{CompressionFrameSegment, FileChunk, PendingFileChunk};
use crate::file_format::decode_compression_frame_segment_payload_view;
use crate::lockbox_path::LockboxPath;
use crate::node_kind::NodeKind;
use crate::page::{PageObject, PageObjectKind, DEFAULT_PAGE_BYTES};
use crate::page_buffer::ZeroizingBytes;
use crate::page_object_packer::PageObjectPacker;
use crate::security::validate_permissions;
use crate::storage::atomic_file_replacement::AtomicFileReplacement;
use crate::toc_entry::TocEntry;
use crate::{Error, Result, WorkloadProfile};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

// A verified native slice can be written directly while its zeroizing cache
// allocation stays alive. Vec-returning APIs copy shared slices only when needed.
#[cfg(any(test, feature = "native-block-layout"))]
pub(super) enum NativeReadBytes {
    Owned(Vec<u8>),
    Shared {
        frame: Arc<super::CachedCompressionFrame>,
        range: std::ops::Range<usize>,
        source_guard: Option<(u64, u64)>,
    },
}

#[cfg(any(test, feature = "native-block-layout"))]
impl NativeReadBytes {
    fn ensure_current(&self, storage: &crate::storage::StorageBackend) -> Result<()> {
        if let Self::Shared {
            source_guard: Some((required_end, revision)),
            ..
        } = self
        {
            if *required_end > storage.current_len()? {
                return Err(Error::Truncated);
            }
            if *revision == u64::MAX || *revision != storage.write_revision() {
                return Err(Error::CorruptRecord);
            }
        }
        Ok(())
    }

    fn into_vec(self, storage: &crate::storage::StorageBackend) -> Result<Vec<u8>> {
        match self {
            Self::Owned(bytes) => Ok(bytes),
            shared @ Self::Shared { .. } => {
                let mut bytes = ZeroizingBytes::new(shared.to_vec());
                // Owned results keep the original cache-hit check after copying.
                shared.ensure_current(storage)?;
                Ok(std::mem::take(&mut *bytes))
            }
        }
    }
}

#[cfg(any(test, feature = "native-block-layout"))]
impl std::ops::Deref for NativeReadBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self {
            Self::Owned(bytes) => bytes,
            Self::Shared { frame, range, .. } => &frame.data[range.clone()],
        }
    }
}

const SMALL_FILE_PACKING_LIMIT: usize = 1024 * 1024;
const SMALL_FILE_COMPRESSION_FRAME_BYTES: usize = 4 * 1024;
const BULK_IMPORT_SMALL_FILE_COMPRESSION_FRAME_BYTES: usize = 2 * 1024 * 1024;
pub(super) const FILE_COMPRESSION_FRAME_BYTES: usize = 2 * 1024 * 1024;
const MAX_SEGMENT_BYTES: usize = DEFAULT_MAX_PAGE_BODY_BYTES - 64 * 1024;
const DECODED_COMPRESSION_FRAME_CACHE_BYTES: usize = 64 * 1024 * 1024;

// An immutable, fully verified slice of a raw frame. Keeping the page alive
// shares its existing zeroizing payload rather than duplicating plaintext.
pub(super) struct VerifiedFileRange {
    page: Arc<crate::page::DecodedPage>,
    object_index: usize,
    range: std::ops::Range<usize>,
}

impl VerifiedFileRange {
    pub(super) fn with_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Result<R> {
        self.page.objects[self.object_index].with_payload(|payload| f(&payload[self.range.clone()]))
    }
}

fn validate_frame_segment(
    decoded: &crate::file_format::payload::DecodedFileChunkView<'_>,
    chunk: &FileChunk,
    segment: &CompressionFrameSegment,
    expected_total_len: u64,
) -> Result<()> {
    let manifest_slice_missing = decoded.manifest.as_ref().is_some_and(|manifest| {
        manifest
            .slice_for(
                &chunk.stored_path,
                chunk.file_offset,
                chunk.compression_frame_offset,
                chunk.len,
            )
            .filter(|slice| slice.total_len == 0 || slice.total_len == expected_total_len)
            .is_none()
    });
    if decoded.compression_frame_id != chunk.compression_frame_id
        || decoded.compression_frame_len != chunk.compression_frame_len
        || decoded.compressed_len != chunk.compressed_len
        || decoded.compression != chunk.compression
        || decoded.compression_frame_digest != chunk.compression_frame_digest
        || manifest_slice_missing
        || decoded.segment_offset != segment.segment_offset
        || decoded.data.len() as u64 != segment.segment_len
    {
        return Err(Error::CorruptRecord);
    }
    Ok(())
}

#[cfg(test)]
mod range_integrity_tests {
    use super::*;

    #[test]
    fn legacy_writer_rejects_native_preparation_before_mutating_storage() {
        let mut archive = Lockbox::create(b"prepared layout mismatch test");
        let path = LockboxPath::new("/native").unwrap();
        let prepared = FileImportPipeline::new(3, 1)
            .with_native_blocks(true)
            .prepare(&[CompressionFrameWrite {
                path: &path,
                permissions: 0o600,
                total_len: 11,
                file_offset: 0,
                data: b"test bytes!",
            }]);
        let before = archive.to_bytes();
        let sequence = archive.sequence;
        let mut chunks = Vec::new();
        {
            let mut writer = FilePageWriter::new(&mut archive);
            writer.native_blocks = false;
            assert!(writer
                .write_prepared_compression_frame(prepared, &mut chunks)
                .is_err());
        }
        assert!(chunks.is_empty());
        assert_eq!(archive.sequence, sequence);
        assert_eq!(archive.to_bytes(), before);
    }

    #[test]
    fn partial_frame_read_checks_bytes_outside_the_requested_range() {
        let lb = Lockbox::create(b"range integrity test");
        let mut stored = vec![42; 4096];
        let chunk = FileChunk {
            #[cfg(test)]
            block_frame: None,
            stored_path: LockboxPath::new("/test").unwrap(),
            file_offset: 0,
            len: stored.len() as u64,
            compression_frame_offset: 0,
            compression_frame_len: stored.len() as u64,
            compressed_len: stored.len() as u64,
            compression: COMPRESSION_NONE,
            compression_frame_id: 1,
            compression_frame_digest: strong_checksum(&stored),
            segments: Vec::new(),
        };
        assert_eq!(
            lb.read_checked_frame_slice(&chunk, &(0..8), &stored, Vec::new())
                .unwrap(),
            vec![42; 8]
        );
        stored[4095] ^= 1;
        assert!(matches!(
            lb.read_checked_frame_slice(&chunk, &(0..8), &stored, Vec::new()),
            Err(Error::CorruptRecord)
        ));
    }
}

/// Ordering used by `Lockbox::stream_content`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ContentStreamOrder {
    /// Stream by lockbox path and logical file offset.
    #[default]
    Logical,
    /// Stream stored content chunks by physical page offset where possible.
    Physical,
}
/// Options for streaming lockbox file content without extracting to disk.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ContentStreamOptions {
    /// Logical or physical order in which chunks are yielded.
    pub order: ContentStreamOrder,
}

/// Metadata for one streamed content range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentChunk {
    /// Logical lockbox path containing this range.
    pub path: LockboxPath,
    /// Byte offset within the logical file.
    pub file_offset: u64,
    /// Number of logical bytes in the range.
    pub len: u64,
    /// Backing file offset when the chunk has physical storage.
    pub physical_offset: Option<u64>,
    /// Whether the range is a sparse zero-filled hole.
    pub sparse: bool,
}

impl<State> Lockbox<State> {
    pub(crate) fn rewrite_shared_compression_frames_before_removal(
        &mut self,
        entry: &TocEntry,
    ) -> Result<()> {
        if entry.deleted || entry.node_kind != NodeKind::File || entry.chunks.is_empty() {
            return Ok(());
        }
        let removed_compression_frames = entry
            .chunks
            .iter()
            .map(|chunk| chunk.compression_frame_id)
            .collect::<BTreeSet<_>>();
        let mut shared = BTreeSet::new();
        for other in self.toc_entries.values() {
            if other.deleted || other.path == entry.path || other.node_kind != NodeKind::File {
                continue;
            }
            if other
                .chunks
                .iter()
                .any(|chunk| removed_compression_frames.contains(&chunk.compression_frame_id))
            {
                for chunk in &other.chunks {
                    if removed_compression_frames.contains(&chunk.compression_frame_id) {
                        shared.insert(chunk.compression_frame_id);
                    }
                }
            }
        }
        if shared.is_empty() {
            return Ok(());
        }

        let mut groups: BTreeMap<u64, Vec<SharedCompressionFrameSurvivor>> = BTreeMap::new();
        for other in self.toc_entries.values() {
            if other.deleted || other.path == entry.path || other.node_kind != NodeKind::File {
                continue;
            }
            for chunk in &other.chunks {
                if shared.contains(&chunk.compression_frame_id) {
                    let data = self.read_file_chunk_compression_frame(other.len, chunk)?;
                    groups.entry(chunk.compression_frame_id).or_default().push(
                        SharedCompressionFrameSurvivor {
                            path: other.path.clone(),
                            permissions: other.permissions,
                            total_len: other.len,
                            file_offset: chunk.file_offset,
                            data,
                        },
                    );
                }
            }
        }

        let mut replacement_indices: Vec<(LockboxPath, u64, usize)> = Vec::new();
        let mut replacements: Vec<(LockboxPath, u64, FileChunk)> = Vec::new();
        {
            let mut writer = FilePageWriter::new(self);
            let mut written = Vec::new();
            for (old_compression_frame_id, survivors) in &groups {
                let writes = survivors
                    .iter()
                    .map(|survivor| CompressionFrameWrite {
                        path: &survivor.path,
                        permissions: survivor.permissions,
                        total_len: survivor.total_len,
                        file_offset: survivor.file_offset,
                        data: &survivor.data,
                    })
                    .collect::<Vec<_>>();
                let indices = writer.write_compression_frame_bundle(&writes, &mut written)?;
                for (survivor, chunk_index) in survivors.iter().zip(indices) {
                    replacement_indices.push((
                        survivor.path.clone(),
                        *old_compression_frame_id,
                        chunk_index,
                    ));
                }
            }
            writer.finish(&mut written)?;
            replacements.extend(replacement_indices.into_iter().map(
                |(path, old_compression_frame_id, chunk_index)| {
                    (path, old_compression_frame_id, written[chunk_index].clone())
                },
            ));
        }

        let mut by_path: BTreeMap<LockboxPath, Vec<(u64, FileChunk)>> = BTreeMap::new();
        for (path, old_compression_frame_id, chunk) in replacements {
            by_path
                .entry(path)
                .or_default()
                .push((old_compression_frame_id, chunk));
        }
        let mut dirty = Vec::new();
        for (path, chunks) in by_path {
            if let Some(live) = self.toc_entries.get_mut(path.as_str()) {
                let replaced_compression_frames = chunks
                    .iter()
                    .map(|(old_compression_frame_id, _)| *old_compression_frame_id)
                    .collect::<BTreeSet<_>>();
                live.chunks.retain(|chunk| {
                    !replaced_compression_frames.contains(&chunk.compression_frame_id)
                });
                live.chunks
                    .extend(chunks.into_iter().map(|(_, chunk)| chunk));
                live.chunks.sort_by_key(|chunk| chunk.file_offset);
                if let Some(first) = live.chunks.first().and_then(|chunk| chunk.segments.first()) {
                    live.record_offset = first.page_offset;
                    live.record_len = first.page_len;
                    live.record_object_id = first.object_id;
                }
                dirty.push(live.path.clone());
            }
        }
        self.mark_toc_dirty_paths(dirty.iter());
        self.rebuild_record_ref_counts();
        Ok(())
    }

    pub(crate) fn validate_replace_intent(&self, path: &LockboxPath, replace: bool) -> Result<()> {
        let exists = self.exists(path);
        match (replace, exists) {
            (false, true) => Err(Error::AlreadyExists(path.to_string())),
            (true, false) => Err(Error::NotFound(path.to_string())),
            _ => Ok(()),
        }
    }

    /// Open a seekable read handle over a file inside the lockbox.
    pub fn open_file(&self, path: &LockboxPath) -> Result<LockboxFileReader<'_, State>> {
        let path = path.as_file_path()?;
        let entry = self
            .toc_entries
            .get(path)
            .filter(|entry| !entry.deleted && entry.node_kind == NodeKind::File)
            .ok_or_else(|| Error::NotFound(path.to_string()))?;
        Ok(LockboxFileReader::new(self, entry.path.clone(), entry.len))
    }

    /// Add or replace a file from an in-memory byte slice.
    ///
    /// Missing parent directories are created with the default directory
    /// permissions when adding a new file.
    ///
    /// When `replace` is `false`, returns `Error::AlreadyExists` if `path`
    /// already names an existing file or symlink. When `replace` is `true`,
    /// returns `Error::NotFound` if there is no existing entry to replace. Returns
    /// `Error::InvalidPath` for directory-only or unsafe lockbox paths and
    /// propagates storage or encoding errors from the write.
    pub fn add_file(&mut self, path: &LockboxPath, data: &[u8], replace: bool) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        self.add_file_with_permissions(path, data, DEFAULT_FILE_PERMISSIONS, replace)
    }

    /// Add or replace a file with explicit Unix-style permissions.
    ///
    /// Missing parent directories are created with the default directory
    /// permissions when adding a new file.
    ///
    /// `permissions` is a Unix mode value containing only the low permission
    /// bits, written in Rust as octal literals such as `0o600`, `0o640`, or
    /// `0o755`. File type bits, sticky/setuid/setgid bits, and platform ACLs
    /// are not supported.
    ///
    /// When `replace` is `false`, returns `Error::AlreadyExists` if `path`
    /// already names an existing file or symlink. When `replace` is `true`,
    /// returns `Error::NotFound` if there is no existing entry to replace. Returns
    /// `Error::InvalidPath` for directory-only or unsafe lockbox paths,
    /// `Error::InvalidPath` for unsupported permission bits, and propagates
    /// storage or encoding errors from the write.
    pub fn add_file_with_permissions(
        &mut self,
        path: &LockboxPath,
        data: &[u8],
        permissions: u32,
        replace: bool,
    ) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        self.ensure_mirror_path_mutable(path)?;
        if data.len() <= SMALL_FILE_PACKING_LIMIT {
            return self.stage_small_file(path, data, permissions, replace);
        }
        if self.worker_jobs() > 1 {
            return self.add_file_from_reader_with_permissions(
                path,
                Cursor::new(data),
                permissions,
                replace,
            );
        }
        self.write_file_from_source_with_permissions(
            path,
            permissions,
            replace,
            |archive, path, permissions| {
                archive.write_file_data_from_slice(path, data, permissions)
            },
        )
    }

    /// Add or replace a file by streaming bytes from a reader.
    ///
    /// Missing parent directories are created with the default directory
    /// permissions when adding a new file.
    ///
    /// When `replace` is `false`, returns `Error::AlreadyExists` if `path`
    /// already names an existing file or symlink. When `replace` is `true`,
    /// returns `Error::NotFound` if there is no existing entry to replace. Returns
    /// `Error::InvalidPath` for directory-only or unsafe lockbox paths and
    /// propagates reader, storage, or encoding errors from the write.
    pub fn add_file_from_reader(
        &mut self,
        path: &LockboxPath,
        reader: impl Read,
        replace: bool,
    ) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        self.add_file_from_reader_with_permissions(path, reader, DEFAULT_FILE_PERMISSIONS, replace)
    }

    /// Add or replace a file by reading from a host filesystem path.
    ///
    /// Missing parent directories are created with the default directory
    /// permissions when adding a new file.
    ///
    /// When `replace` is `false`, returns `Error::AlreadyExists` if
    /// `destination` already names an existing file or symlink. When `replace`
    /// is `true`, returns `Error::NotFound` if there is no existing entry to replace.
    /// Returns `Error::InvalidPath` for directory-only or unsafe destination
    /// paths and `Error::Io` if the host file cannot be read.
    pub fn add_file_from_path(
        &mut self,
        source: &Path,
        destination: &LockboxPath,
        replace: bool,
    ) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        let stat_start = Instant::now();
        let metadata = std::fs::metadata(source)
            .map_err(|err| Error::Io(format!("stat {}: {err}", source.display())))?;
        self.add_host_stat_nanos(stat_start.elapsed().as_nanos());
        if metadata.len() <= SMALL_FILE_PACKING_LIMIT as u64 {
            let read_start = Instant::now();
            let data = std::fs::read(source)
                .map_err(|err| Error::Io(format!("read {}: {err}", source.display())))?;
            self.add_host_read_nanos(read_start.elapsed().as_nanos());
            return self.add_file(destination, &data, replace);
        }
        let file = std::fs::File::open(source)
            .map_err(|err| Error::Io(format!("open {}: {err}", source.display())))?;
        self.add_file_from_reader(destination, file, replace)
    }

    /// Add or replace a host file and require the exact bytes consumed by the
    /// import to match `expected_sha256`.
    ///
    /// This preserves bulk small-file packing while allowing callers that
    /// scanned a changing filesystem to prove that the imported bytes match
    /// their scan. A mismatch leaves the change uncommitted and returns an
    /// error.
    pub fn add_file_from_path_verified(
        &mut self,
        source: &Path,
        destination: &LockboxPath,
        replace: bool,
        expected_sha256: &[u8; 32],
    ) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        struct HashingReader<R> {
            inner: R,
            hasher: Sha256,
        }
        impl<R: Read> Read for HashingReader<R> {
            fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
                let count = self.inner.read(buffer)?;
                self.hasher.update(&buffer[..count]);
                Ok(count)
            }
        }

        let metadata = std::fs::metadata(source)
            .map_err(|err| Error::Io(format!("stat {}: {err}", source.display())))?;
        let actual: [u8; 32] = if metadata.len() <= SMALL_FILE_PACKING_LIMIT as u64 {
            let data = std::fs::read(source)
                .map_err(|err| Error::Io(format!("read {}: {err}", source.display())))?;
            let digest = Sha256::digest(&data).into();
            self.add_file(destination, &data, replace)?;
            digest
        } else {
            let file = std::fs::File::open(source)
                .map_err(|err| Error::Io(format!("open {}: {err}", source.display())))?;
            let mut reader = HashingReader {
                inner: file,
                hasher: Sha256::new(),
            };
            self.add_file_from_reader(destination, &mut reader, replace)?;
            reader.hasher.finalize().into()
        };
        if &actual != expected_sha256 {
            return Err(Error::InvalidOperation(format!(
                "source file changed while it was imported: {}",
                source.display()
            )));
        }
        Ok(())
    }

    /// Add or replace a streamed file with explicit Unix-style permissions.
    ///
    /// Missing parent directories are created with the default directory
    /// permissions when adding a new file.
    ///
    /// `permissions` is a Unix mode value containing only the low permission
    /// bits, written in Rust as octal literals such as `0o600`, `0o640`, or
    /// `0o755`. File type bits, sticky/setuid/setgid bits, and platform ACLs
    /// are not supported.
    ///
    /// When `replace` is `false`, returns `Error::AlreadyExists` if `path`
    /// already names an existing file or symlink. When `replace` is `true`,
    /// returns `Error::NotFound` if there is no existing entry to replace. Returns
    /// `Error::InvalidPath` for directory-only or unsafe lockbox paths,
    /// `Error::InvalidPath` for unsupported permission bits, and propagates
    /// reader, storage, or encoding errors from the write.
    pub fn add_file_from_reader_with_permissions(
        &mut self,
        path: &LockboxPath,
        reader: impl Read,
        permissions: u32,
        replace: bool,
    ) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        self.ensure_mirror_path_mutable(path)?;
        self.write_file_from_reader_with_permissions(path, reader, permissions, replace)
    }

    fn write_file_from_reader_with_permissions(
        &mut self,
        path: &LockboxPath,
        reader: impl Read,
        permissions: u32,
        replace: bool,
    ) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        self.write_file_from_source_with_permissions(
            path,
            permissions,
            replace,
            |archive, path, permissions| {
                let jobs = archive.worker_jobs();
                if jobs > 1 {
                    archive.write_file_data_parallel(path, reader, permissions, jobs)
                } else {
                    archive.write_file_data_sequential(path, reader, permissions)
                }
            },
        )
    }

    fn write_file_from_source_with_permissions(
        &mut self,
        path: &LockboxPath,
        permissions: u32,
        replace: bool,
        write_data: impl FnOnce(&mut Self, &LockboxPath, u32) -> Result<(u64, Vec<FileChunk>)>,
    ) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        let path = path.file_path()?;
        let permissions = validate_permissions(permissions)?;
        self.validate_replace_intent(&path, replace)?;
        self.create_parent_dirs_for(&path)?;
        if self.should_discard_file_pages_after_flush()
            && self.pending_small_files.contains_key(path.as_str())
        {
            self.flush_bulk_small_file_packer()?;
        }
        self.remove_pending_small_file(&path);
        if let Some(old) = self.toc_entries.get(path.as_str()).cloned() {
            self.free_entry_slots(old)?;
        }

        let (file_offset, chunks) = write_data(self, &path, permissions)?;

        let entry = TocEntry {
            path: path.clone(),
            len: file_offset,
            record_offset: chunks
                .first()
                .and_then(|chunk| chunk.segments.first())
                .map(|segment| segment.page_offset)
                .unwrap_or(0),
            record_len: chunks
                .first()
                .and_then(|chunk| chunk.segments.first())
                .map(|segment| segment.page_len)
                .unwrap_or(0),
            record_object_id: chunks
                .first()
                .and_then(|chunk| chunk.segments.first())
                .map(|segment| segment.object_id)
                .unwrap_or(0),
            deleted: false,
            node_kind: NodeKind::File,
            permissions,
            chunks,
        };
        self.add_entry_record_refs(&entry);
        self.toc_entries.insert(path.clone(), entry);
        self.mark_toc_dirty(&path);
        self.needs_packing = true;
        Ok(())
    }

    fn write_file_data_from_slice(
        &mut self,
        path: &LockboxPath,
        data: &[u8],
        permissions: u32,
    ) -> Result<(u64, Vec<FileChunk>)> {
        // Only large in-memory inputs reach this path. Borrow each frame from
        // the caller instead of copying it through the streaming read buffer.
        debug_assert!(data.len() > SMALL_FILE_PACKING_LIMIT);
        let mut writer = FilePageWriter::new(self);
        let pipeline = writer.import_pipeline(1);
        pipeline.with_encoder(
            &data[..data.len().min(FILE_COMPRESSION_FRAME_BYTES)],
            |encode| {
                let mut chunks = Vec::new();
                let mut file_offset = 0u64;
                for frame in data.chunks(FILE_COMPRESSION_FRAME_BYTES) {
                    let prepared = pipeline.prepare_with_encoder(
                        &[CompressionFrameWrite {
                            path,
                            permissions,
                            total_len: 0,
                            file_offset,
                            data: frame,
                        }],
                        encode,
                    );
                    writer.write_prepared_compression_frame(prepared, &mut chunks)?;
                    file_offset += frame.len() as u64;
                }
                writer.finish(&mut chunks)?;
                Ok((file_offset, chunks))
            },
        )
    }

    fn write_file_data_sequential(
        &mut self,
        path: &LockboxPath,
        mut reader: impl Read,
        permissions: u32,
    ) -> Result<(u64, Vec<FileChunk>)> {
        let mut chunks = Vec::new();
        let mut file_offset = 0u64;
        let mut writer = FilePageWriter::new(self);
        let mut buffer = vec![0; FILE_COMPRESSION_FRAME_BYTES];
        loop {
            let read_start = Instant::now();
            let read = read_next_chunk(&mut reader, &mut buffer)?;
            writer
                .lockbox
                .add_host_read_nanos(read_start.elapsed().as_nanos());
            if read == 0 {
                if file_offset == 0 {
                    writer.write_compression_frame(
                        CompressionFrameWrite {
                            path,
                            permissions,
                            total_len: 0,
                            file_offset: 0,
                            data: &[],
                        },
                        &mut chunks,
                    )?;
                }
                break;
            }
            writer.write_compression_frame(
                CompressionFrameWrite {
                    path,
                    permissions,
                    total_len: 0,
                    file_offset,
                    data: &buffer[..read],
                },
                &mut chunks,
            )?;
            file_offset += read as u64;
        }
        writer.finish(&mut chunks)?;
        Ok((file_offset, chunks))
    }

    fn write_file_data_parallel(
        &mut self,
        path: &LockboxPath,
        mut reader: impl Read,
        permissions: u32,
        jobs: usize,
    ) -> Result<(u64, Vec<FileChunk>)> {
        let jobs = jobs.max(1);
        let level = self.compression_frame_zstd_level();
        let pipeline = FileImportPipeline::new(level, jobs)
            .with_compression(self.configured_compression())
            .with_native_blocks(cfg!(feature = "native-block-layout") && self.format_mode.0 != 0);
        let queue_bound = jobs.saturating_mul(2).max(1);
        let (job_tx, job_rx) = std::sync::mpsc::sync_channel::<ParallelCompressionJob>(queue_bound);
        let (result_tx, result_rx) = std::sync::mpsc::channel::<ParallelCompressionResult>();
        let job_rx = Arc::new(Mutex::new(job_rx));

        std::thread::scope(|scope| -> Result<(u64, Vec<FileChunk>)> {
            for _ in 0..jobs {
                let job_rx = Arc::clone(&job_rx);
                let result_tx = result_tx.clone();
                scope.spawn(move || loop {
                    let job = match job_rx.lock() {
                        Ok(rx) => rx.recv(),
                        Err(_) => return,
                    };
                    let Ok(job) = job else {
                        return;
                    };
                    let result = pipeline.prepare_parallel_job(job);
                    if result_tx.send(result).is_err() {
                        return;
                    }
                });
            }
            drop(result_tx);

            let mut writer = FilePageWriter::new(self);
            let mut chunks = Vec::new();
            let mut frame_order = ParallelFrameOrder::new();
            let mut file_offset = 0u64;
            let mut job_count = 0usize;
            let mut buffer = vec![0; FILE_COMPRESSION_FRAME_BYTES];
            loop {
                let read_start = Instant::now();
                let read = read_next_chunk(&mut reader, &mut buffer)?;
                writer
                    .lockbox
                    .add_host_read_nanos(read_start.elapsed().as_nanos());
                if read == 0 {
                    if file_offset == 0 {
                        job_tx
                            .send(ParallelCompressionJob {
                                index: job_count,
                                path: path.clone(),
                                permissions,
                                total_len: 0,
                                file_offset: 0,
                                data: Vec::new(),
                            })
                            .map_err(|_| {
                                Error::Io("compression worker stopped unexpectedly".to_string())
                            })?;
                        job_count += 1;
                        frame_order.drain_ready(&result_rx, &mut writer, &mut chunks)?;
                    }
                    break;
                }

                job_tx
                    .send(ParallelCompressionJob {
                        index: job_count,
                        path: path.clone(),
                        permissions,
                        total_len: 0,
                        file_offset,
                        data: buffer[..read].to_vec(),
                    })
                    .map_err(|_| {
                        Error::Io("compression worker stopped unexpectedly".to_string())
                    })?;
                file_offset += read as u64;
                job_count += 1;
                frame_order.drain_ready(&result_rx, &mut writer, &mut chunks)?;
            }
            drop(job_tx);

            while frame_order.received_count < job_count {
                let result = result_rx.recv().map_err(|_| {
                    Error::Io("compression worker stopped unexpectedly".to_string())
                })?;
                frame_order.accept(result, &mut writer, &mut chunks)?;
            }
            writer.finish(&mut chunks)?;
            Ok((file_offset, chunks))
        })
    }

    /// Return the complete contents of a file.
    ///
    /// Returns `Error::InvalidPath` for directory-only paths, `Error::NotFound`
    /// if `path` is absent or not a file, `Error::CorruptRecord` if stored file
    /// metadata is inconsistent, and `Error::Io` if an internal write into the
    /// output buffer fails. Returns `Error::SecurityLimitExceeded` if the
    /// multi-frame or sparse output reservation cannot be satisfied.
    pub fn get_file(&self, path: &LockboxPath) -> Result<Vec<u8>> {
        let path_key = path.as_file_path()?;
        let entry = self
            .toc_entries
            .get(path_key)
            .filter(|entry| !entry.deleted && entry.node_kind == NodeKind::File)
            .ok_or_else(|| Error::NotFound(path_key.to_string()))?;
        if let Some(pending) = self.pending_small_files.get(path_key) {
            return Ok(pending.data.to_vec());
        }
        // The decoder already owns the complete output for a single covering
        // chunk. Return it directly instead of cloning/sorting the chunk list
        // and copying the decoded bytes through another Vec writer.
        if let [chunk] = entry.chunks.as_slice() {
            if chunk.file_offset == 0 && chunk.len == entry.len {
                let mut decoded =
                    ZeroizingBytes::new(self.read_file_chunk_compression_frame(entry.len, chunk)?);
                if decoded.len() as u64 != entry.len {
                    return Err(Error::CorruptRecord);
                }
                return Ok(std::mem::take(&mut *decoded));
            }
        }
        let mut out = Vec::new();
        // Reserve raw multi-frame output once. Compressed frames retain the
        // existing allocation order: reserving the final output before decoder
        // scratch allocation regressed compressible large-file reads.
        if entry
            .chunks
            .iter()
            .all(|chunk| chunk.compression == COMPRESSION_NONE)
        {
            let capacity = usize::try_from(entry.len).map_err(|_| {
                Error::SecurityLimitExceeded("file contents exceed addressable memory".to_string())
            })?;
            out.try_reserve_exact(capacity).map_err(|_| {
                Error::SecurityLimitExceeded("cannot allocate complete file contents".to_string())
            })?;
        }
        self.extract_file_to_writer(path, &mut out)?;
        Ok(out)
    }

    /// Extract a file's contents to a writer.
    ///
    /// Returns `Error::InvalidPath` for directory-only paths, `Error::NotFound`
    /// if `path` is absent or not a file, `Error::CorruptRecord` if stored file
    /// metadata is inconsistent, and `Error::Io` if the writer fails.
    pub fn extract_file_to_writer(&self, path: &LockboxPath, mut writer: impl Write) -> Result<()> {
        let path = path.as_file_path()?;
        let entry = self
            .toc_entries
            .get(path)
            .filter(|entry| !entry.deleted && entry.node_kind == NodeKind::File)
            .ok_or_else(|| Error::NotFound(path.to_string()))?;

        if let Some(pending) = self.pending_small_files.get(path) {
            writer
                .write_all(&pending.data)
                .map_err(|err| Error::Io(err.to_string()))?;
            return Ok(());
        }

        if entry.chunks.is_empty() {
            write_zeroes(&mut writer, entry.len)?;
            return Ok(());
        }

        let mut chunks = entry.chunks.clone();
        chunks.sort_by_key(|chunk| chunk.file_offset);
        let mut written = 0u64;
        for chunk in chunks {
            if chunk.file_offset < written || chunk.file_offset > entry.len {
                return Err(Error::CorruptRecord);
            }
            if chunk.file_offset > written {
                write_zeroes(&mut writer, chunk.file_offset - written)?;
                written = chunk.file_offset;
            }
            if chunk.file_offset.saturating_add(chunk.len) > entry.len {
                return Err(Error::CorruptRecord);
            }
            #[cfg(any(test, feature = "native-block-layout"))]
            let decoded_chunk = if chunk.block_frame.is_some() {
                self.read_native_block_chunk_view(entry.len, &chunk, 0..chunk.len, true)?
            } else {
                NativeReadBytes::Owned(self.read_file_chunk_compression_frame(entry.len, &chunk)?)
            };
            #[cfg(not(any(test, feature = "native-block-layout")))]
            let decoded_chunk = self.read_file_chunk_compression_frame(entry.len, &chunk)?;
            #[cfg(any(test, feature = "native-block-layout"))]
            decoded_chunk.ensure_current(&self.storage)?;
            writer
                .write_all(&decoded_chunk)
                .map_err(|err| Error::Io(err.to_string()))?;
            written = written.saturating_add(decoded_chunk.len() as u64);
            if written > entry.len {
                return Err(Error::CorruptRecord);
            }
        }
        if written < entry.len {
            write_zeroes(&mut writer, entry.len - written)?;
        } else if written != entry.len {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }

    /// Extract a file's contents to a host filesystem path.
    ///
    /// When `replace` is `false`, returns `Error::AlreadyExists` if the
    /// destination path already exists. When `replace` is `true`, returns
    /// `Error::NotFound` if the destination path does not already exist.
    /// Returns `Error::Io` if the destination file cannot be created. Returns
    /// the same errors as `extract_file_to_writer` for lockbox read failures.
    pub fn extract_file_to(
        &self,
        source: &LockboxPath,
        destination: &Path,
        replace: bool,
    ) -> Result<()> {
        if !replace {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(destination)
                .map_err(|err| {
                    if err.kind() == std::io::ErrorKind::AlreadyExists {
                        Error::AlreadyExists(destination.display().to_string())
                    } else {
                        Error::Io(format!("create {}: {err}", destination.display()))
                    }
                })?;
            return self.extract_file_to_writer(source, &mut file);
        }

        if !destination.exists() {
            return Err(Error::NotFound(destination.display().to_string()));
        }
        let (replacement, mut temp_file) =
            AtomicFileReplacement::create_unique(destination, ".lockbox-extract-file")?;
        let result = self.extract_file_to_writer(source, &mut temp_file);
        if let Err(err) = result {
            replacement.discard();
            return Err(err);
        }
        drop(temp_file);
        replacement.install()?;
        Ok(())
    }

    /// Return stored Unix-style permissions for a file or symlink.
    ///
    /// The returned value uses the low Unix permission bits only, for example
    /// `0o600`, `0o640`, or `0o755`.
    pub fn permissions(&self, path: &LockboxPath) -> Option<u32> {
        let path = path.as_file_path().ok()?;
        self.toc_entries
            .get(path)
            .filter(|entry| !entry.deleted)
            .map(|entry| entry.permissions)
    }

    /// Read a bounded byte range from a file.
    ///
    /// Returns `Error::InvalidPath` for directory-only paths, `Error::NotFound`
    /// if `path` is absent or not a file, and `Error::CorruptRecord` if stored
    /// file metadata is inconsistent. A range outside the file returns an empty
    /// vector rather than an error.
    pub fn read_file_range(&self, path: &LockboxPath, offset: u64, len: u64) -> Result<Vec<u8>> {
        let path = path.as_file_path()?;
        let entry = self
            .toc_entries
            .get(path)
            .filter(|entry| !entry.deleted && entry.node_kind == NodeKind::File)
            .ok_or_else(|| Error::NotFound(path.to_string()))?;
        if len == 0 || offset >= entry.len {
            return Ok(Vec::new());
        }
        let wanted_end = offset.saturating_add(len).min(entry.len);

        if let Some(pending) = self.pending_small_files.get(path) {
            let start = offset.min(pending.data.len() as u64) as usize;
            let end = wanted_end.min(pending.data.len() as u64) as usize;
            return Ok(pending.data[start..end].to_vec());
        }

        let capacity = usize::try_from(wanted_end - offset).map_err(|_| {
            Error::SecurityLimitExceeded("requested range exceeds addressable memory".to_string())
        })?;
        // Exact chunk reads transfer their decoded allocation below. Do not
        // allocate a second, unused output buffer before knowing it is needed.
        let mut out = Vec::new();
        let mut chunks = entry.chunks.clone();
        chunks.sort_by_key(|chunk| chunk.file_offset);
        let mut cursor = offset;
        for chunk in chunks {
            let chunk_start = chunk.file_offset;
            let chunk_end = chunk.file_offset.saturating_add(chunk.len);
            if chunk_start > chunk_end || chunk_end > entry.len {
                return Err(Error::CorruptRecord);
            }
            if chunk_end <= offset || chunk_start >= wanted_end {
                continue;
            }
            if chunk_start > cursor {
                let zeroes = chunk_start.min(wanted_end) - cursor;
                if out.capacity() == 0 {
                    out.reserve(capacity);
                }
                out.resize(out.len() + zeroes as usize, 0);
                cursor = chunk_start.min(wanted_end);
            }

            let copy_start = cursor.max(chunk_start) - chunk_start;
            let copy_end = wanted_end.min(chunk_end) - chunk_start;
            let mut decoded_chunk = ZeroizingBytes::new(self.read_file_chunk_range(
                entry.len,
                &chunk,
                copy_start..copy_end,
            )?);
            // An aligned frame read already owns exactly the requested bytes.
            // Transfer it to the caller instead of making another full copy.
            if out.is_empty() && chunk_start == offset && chunk_end == wanted_end {
                return Ok(std::mem::take(&mut *decoded_chunk));
            }
            if out.capacity() == 0 {
                out.reserve(capacity);
            }
            out.extend_from_slice(&decoded_chunk);
            cursor = chunk_start + copy_end;
            if cursor >= wanted_end {
                break;
            }
        }
        if cursor < wanted_end {
            if out.capacity() == 0 {
                out.reserve(capacity);
            }
            out.resize(out.len() + (wanted_end - cursor) as usize, 0);
        }
        Ok(out)
    }

    #[cfg(any(test, feature = "native-block-layout"))]
    pub(super) fn read_native_file_window(
        &self,
        expected_total_len: u64,
        chunk: &FileChunk,
    ) -> Result<NativeReadBytes> {
        // Preserve the format policy checked by read_file_chunk_range.
        if self.format_mode.0 != 0
            && self.format_mode.options().compression == crate::Compression::None
            && chunk.compression != COMPRESSION_NONE
        {
            return Err(Error::CorruptRecord);
        }
        self.read_native_block_chunk_view(expected_total_len, chunk, 0..chunk.len, true)
    }

    pub(crate) fn read_file_chunk_compression_frame(
        &self,
        expected_total_len: u64,
        chunk: &FileChunk,
    ) -> Result<Vec<u8>> {
        self.read_file_chunk_range(expected_total_len, chunk, 0..chunk.len)
    }

    pub(super) fn verified_raw_file_range(
        &self,
        path: &LockboxPath,
        offset: u64,
        len: u64,
    ) -> Result<Option<VerifiedFileRange>> {
        let path = path.as_file_path()?;
        if self.pending_small_files.contains_key(path) {
            return Ok(None);
        }
        let entry = self.toc_entries.get(path).ok_or(Error::CorruptRecord)?;
        let wanted_end = offset.checked_add(len).ok_or(Error::CorruptRecord)?;
        let Some(chunk) = entry.chunks.iter().find(|chunk| {
            chunk.file_offset <= offset
                && chunk
                    .file_offset
                    .checked_add(chunk.len)
                    .is_some_and(|end| end >= wanted_end)
        }) else {
            return Ok(None);
        };
        #[cfg(any(test, feature = "native-block-layout"))]
        if chunk.block_frame.is_some() {
            // This shortcut authenticates a whole-frame hash, not a block index.
            return Ok(None);
        }
        if chunk.compression != COMPRESSION_NONE || chunk.segments.len() != 1 {
            return Ok(None);
        }
        let segment = &chunk.segments[0];
        if segment.segment_offset != 0 || segment.segment_len != chunk.compressed_len {
            return Ok(None);
        }
        validate_compression_frame_lengths(chunk.compression_frame_len, chunk.compressed_len)?;
        if chunk.compressed_len != chunk.compression_frame_len
            || chunk
                .file_offset
                .checked_add(chunk.len)
                .is_none_or(|end| end > entry.len)
        {
            return Err(Error::CorruptRecord);
        }
        let page = self.read_shared_page(segment.page_offset)?;
        let object_index = page
            .objects
            .iter()
            .position(|object| object.id == segment.object_id)
            .ok_or(Error::CorruptRecord)?;
        let range = page.objects[object_index].with_payload(|payload| {
            let decoded = decode_compression_frame_segment_payload_view(payload)?;
            validate_frame_segment(&decoded, chunk, segment, entry.len)?;
            if strong_checksum(decoded.data) != chunk.compression_frame_digest {
                return Err(Error::CorruptRecord);
            }
            let chunk_end = chunk
                .compression_frame_offset
                .checked_add(chunk.len)
                .ok_or(Error::CorruptRecord)?;
            if chunk_end > decoded.data.len() as u64 {
                return Err(Error::CorruptRecord);
            }
            // Segment data is the final slice of its object payload.
            let data_start = payload.len() - decoded.data.len();
            let start = data_start
                + usize::try_from(chunk.compression_frame_offset + (offset - chunk.file_offset))
                    .map_err(|_| Error::CorruptRecord)?;
            let end = start
                .checked_add(usize::try_from(len).map_err(|_| Error::CorruptRecord)?)
                .ok_or(Error::CorruptRecord)?;
            if end > payload.len() {
                return Err(Error::CorruptRecord);
            }
            Ok(start..end)
        })??;
        Ok(Some(VerifiedFileRange {
            page,
            object_index,
            range,
        }))
    }

    fn read_file_chunk_range(
        &self,
        expected_total_len: u64,
        chunk: &FileChunk,
        range: std::ops::Range<u64>,
    ) -> Result<Vec<u8>> {
        if range.start > range.end || range.end > chunk.len {
            return Err(Error::CorruptRecord);
        }
        if self.format_mode.0 != 0
            && self.format_mode.options().compression == crate::Compression::None
            && chunk.compression != COMPRESSION_NONE
        {
            return Err(Error::CorruptRecord);
        }
        #[cfg(any(test, feature = "native-block-layout"))]
        if chunk.block_frame.is_some() {
            return self.read_native_block_chunk(expected_total_len, chunk, range);
        }
        if let Some(cached) =
            self.read_cached_compression_frame_slice(expected_total_len, chunk, &range)?
        {
            return Ok(cached);
        }
        validate_compression_frame_lengths(chunk.compression_frame_len, chunk.compressed_len)?;
        if chunk.compressed_len > DEFAULT_MAX_PAGE_LOGICAL_BYTES as u64 {
            return Err(Error::SecurityLimitExceeded(
                "compressed compression-frame exceeds safety limit".to_string(),
            ));
        }
        let compressed_len =
            usize::try_from(chunk.compressed_len).map_err(|_| Error::CorruptRecord)?;
        // Most frames occupy a single page object. Borrow its already checked
        // payload instead of assembling and later wiping another frame-sized buffer.
        if let [segment] = chunk.segments.as_slice() {
            if segment.segment_offset == 0 && segment.segment_len == chunk.compressed_len {
                return self.with_page_object(segment.page_offset, segment.object_id, |object| {
                    object.with_payload(|payload| {
                        let decoded = decode_compression_frame_segment_payload_view(payload)?;
                        validate_frame_segment(&decoded, chunk, segment, expected_total_len)?;
                        self.read_checked_frame_slice(
                            chunk,
                            &range,
                            decoded.data,
                            decoded
                                .manifest
                                .map(|manifest| manifest.slices)
                                .unwrap_or_default(),
                        )
                    })?
                });
            }
        }
        let mut stored = ZeroizingBytes::new(vec![0u8; compressed_len]);
        let mut cache_slices = None;
        for segment in &chunk.segments {
            self.with_page_object(segment.page_offset, segment.object_id, |object| {
                object.with_payload(|payload| {
                    let decoded = decode_compression_frame_segment_payload_view(payload)?;
                    if let Some(manifest) = decoded.manifest.as_ref() {
                        if cache_slices.is_none() {
                            cache_slices = Some(manifest.slices.clone());
                        }
                    }
                    validate_frame_segment(&decoded, chunk, segment, expected_total_len)?;
                    let start = usize::try_from(segment.segment_offset)
                        .map_err(|_| Error::CorruptRecord)?;
                    let end = start
                        .checked_add(decoded.data.len())
                        .ok_or(Error::CorruptRecord)?;
                    if end > stored.len() {
                        return Err(Error::CorruptRecord);
                    }
                    stored[start..end].copy_from_slice(decoded.data);
                    Ok(())
                })?
            })?;
        }
        self.read_checked_frame_slice(chunk, &range, &stored, cache_slices.unwrap_or_default())
    }

    #[cfg(any(test, feature = "native-block-layout"))]
    fn read_native_block_chunk(
        &self,
        expected_total_len: u64,
        chunk: &FileChunk,
        range: std::ops::Range<u64>,
    ) -> Result<Vec<u8>> {
        self.read_native_block_chunk_with_cache(expected_total_len, chunk, range, true)
    }

    #[cfg(any(test, feature = "native-block-layout"))]
    fn read_native_block_chunk_with_cache(
        &self,
        expected_total_len: u64,
        chunk: &FileChunk,
        range: std::ops::Range<u64>,
        cache_single_frames: bool,
    ) -> Result<Vec<u8>> {
        self.read_native_block_chunk_view(expected_total_len, chunk, range, cache_single_frames)?
            .into_vec(&self.storage)
    }

    #[cfg(any(test, feature = "native-block-layout"))]
    fn read_native_block_chunk_view(
        &self,
        expected_total_len: u64,
        chunk: &FileChunk,
        range: std::ops::Range<u64>,
        cache_single_frames: bool,
    ) -> Result<NativeReadBytes> {
        // A decoded frame was admitted only after verifying its complete body.
        // Reuse that proof only for the exact committed reference and allocation.
        crate::file_format::indexed_frame::block_page::validate_chunk_reference(
            self.lockbox_id,
            self.format_mode,
            expected_total_len,
            chunk,
        )?;
        let [segment] = chunk.segments.as_slice() else {
            return Err(Error::CorruptRecord);
        };
        let required_end = segment
            .page_offset
            .checked_add(segment.page_len)
            .ok_or(Error::CorruptRecord)?;
        let revision = self.storage.write_revision();
        if let Some(cached) = self.native_cached_frame(chunk)? {
            // Consumers validate before exposing cached bytes: into_vec after
            // copying, extraction before the writer, and native file handles
            // through their retained extent/revision before every window copy.
            let reference = chunk.block_frame.as_ref().ok_or(Error::CorruptRecord)?;
            if cached
                .native_reference
                .as_ref()
                .is_none_or(|(verified, allocation)| {
                    verified != reference.as_ref() || allocation != segment
                })
            {
                return Err(Error::CorruptRecord);
            }
            let bounds = Self::cached_frame_bounds(&cached, expected_total_len, chunk, &range)?;
            return Ok(NativeReadBytes::Shared {
                frame: cached,
                range: bounds,
                source_guard: Some((required_end, revision)),
            });
        }
        // Whole small frames can be captured in one bounded source read. Keep
        // the same metadata/index/body validators and revalidate the live source
        // before returning bytes; partial ranges never take this eager path.
        const SMALL_FRAME_SNAPSHOT_BYTES: u64 = 16 * 1024;
        if !matches!(&self.storage, crate::storage::StorageBackend::Memory(_))
            && range.start == 0
            && range.end == chunk.len
            && chunk.compression_frame_offset == 0
            && chunk.len == chunk.compression_frame_len
            && chunk.compression_frame_len <= SMALL_FRAME_SNAPSHOT_BYTES
            && segment.page_len <= SMALL_FRAME_SNAPSHOT_BYTES
        {
            use crate::file_format::indexed_frame::block_page::{validate_chunk_reference, Reader};
            use crate::storage::{Storage, StorageBackend};
            // The bounded exact read detects truncation. Validate live source
            // identity/extent and revision after decoding, before exposing bytes.
            let mut page = ZeroizingBytes::new(vec![0; segment.page_len as usize]);
            self.storage.read_at_into(segment.page_offset, &mut page)?;
            let identity = validate_chunk_reference(
                self.lockbox_id,
                self.format_mode,
                expected_total_len,
                chunk,
            )?;
            let reference = chunk.block_frame.as_ref().ok_or(Error::CorruptRecord)?;
            let (mut decoded, slices) = StorageBackend::with_read_snapshot(page, |snapshot| {
                let read = |key: &[u8]| {
                    let reader = Reader::open(
                        snapshot,
                        0,
                        identity,
                        &reference.descriptor,
                        segment.page_len as usize,
                        key,
                    )?;
                    reader.validate_slice(chunk, expected_total_len)?;
                    Ok((
                        ZeroizingBytes::new(reader.read(0..chunk.len)?),
                        reader.slices().to_vec(),
                    ))
                };
                if self.format_mode.plaintext() {
                    read(&[0; 32])
                } else {
                    self.key.with_bytes(read)?
                }
            })?;
            if required_end > self.storage.current_len()? {
                return Err(Error::Truncated);
            }
            if revision == u64::MAX || revision != self.storage.write_revision() {
                return Err(Error::CorruptRecord);
            }
            if cache_single_frames && self.should_cache_native_decoded_frame(decoded.len()) {
                let len = decoded.len();
                let frame = self
                    .cache_decoded_compression_frame_at_revision(
                        chunk,
                        slices,
                        std::mem::take(&mut *decoded),
                        revision,
                    )
                    .ok_or(Error::CorruptRecord)?;
                return Ok(NativeReadBytes::Shared {
                    frame,
                    range: 0..len,
                    source_guard: None,
                });
            }
            return Ok(NativeReadBytes::Owned(std::mem::take(&mut *decoded)));
        }
        let reader = self.open_native_block_chunk(expected_total_len, chunk)?;
        let start = chunk
            .compression_frame_offset
            .checked_add(range.start)
            .ok_or(Error::CorruptRecord)?;
        let end = chunk
            .compression_frame_offset
            .checked_add(range.end)
            .ok_or(Error::CorruptRecord)?;
        let frame_len =
            usize::try_from(chunk.compression_frame_len).map_err(|_| Error::CorruptRecord)?;
        // Cache complete file slices (including packed neighbors), but do not
        // expand a partial raw range just to populate the cache. Compressed
        // reads already decode the complete frame.
        if self.should_cache_native_decoded_frame(frame_len)
            && (cache_single_frames || reader.slices().len() > 1)
            && (chunk.compression != COMPRESSION_NONE
                || (range.start == 0 && range.end == chunk.len))
        {
            let mut decoded = ZeroizingBytes::new(reader.read(0..chunk.compression_frame_len)?);
            let start = usize::try_from(start).map_err(|_| Error::CorruptRecord)?;
            let end = usize::try_from(end).map_err(|_| Error::CorruptRecord)?;
            decoded.get(start..end).ok_or(Error::CorruptRecord)?;
            let frame = self
                .cache_decoded_compression_frame_at_revision(
                    chunk,
                    reader.slices().to_vec(),
                    std::mem::take(&mut *decoded),
                    reader.write_revision(),
                )
                .ok_or(Error::CorruptRecord)?;
            return Ok(NativeReadBytes::Shared {
                frame,
                range: start..end,
                source_guard: None,
            });
        }
        reader
            .read_with_cache(start..end, &self.page_manager)
            .map(NativeReadBytes::Owned)
    }

    #[cfg(any(test, feature = "native-block-layout"))]
    pub(super) fn open_native_block_chunk(
        &self,
        expected_total_len: u64,
        chunk: &FileChunk,
    ) -> Result<crate::file_format::indexed_frame::block_page::Reader<'_>> {
        use crate::file_format::indexed_frame::block_page::{validate_chunk_reference, Reader};
        let identity =
            validate_chunk_reference(self.lockbox_id, self.format_mode, expected_total_len, chunk)?;
        let reference = chunk.block_frame.as_ref().ok_or(Error::CorruptRecord)?;
        let descriptor = &reference.descriptor;
        let [segment] = chunk.segments.as_slice() else {
            return Err(Error::CorruptRecord);
        };
        let page_len = usize::try_from(segment.page_len).map_err(|_| Error::CorruptRecord)?;
        let cached = self
            .page_manager
            .borrow_mut()
            .native_index(segment.page_offset, self.storage.write_revision());
        let open = |key: &[u8]| {
            if let Some(cached) = &cached {
                return cached.bind(&self.storage, chunk, expected_total_len, key);
            }
            let reader = Reader::open(
                &self.storage,
                segment.page_offset,
                identity,
                descriptor,
                page_len,
                key,
            )?;
            reader.validate_slice(chunk, expected_total_len)?;
            self.page_manager.borrow_mut().insert_native_index(
                segment.page_offset,
                segment.page_len,
                reader.cache_entry(chunk)?,
                reader.write_revision(),
            );
            Ok(reader)
        };
        if self.format_mode.plaintext() {
            // Plaintext metadata and block indexes are checked against their
            // committed hashes, not a content key. Owner authorization still
            // belongs to signed archive opening. Avoid exposing protected key
            // memory for an operation that cannot use it.
            open(&[0; 32])
        } else {
            self.key.with_bytes(open)?
        }
    }

    fn read_checked_frame_slice(
        &self,
        chunk: &FileChunk,
        range: &std::ops::Range<u64>,
        stored: &[u8],
        cache_slices: Vec<CompressionFrameSlice>,
    ) -> Result<Vec<u8>> {
        if strong_checksum(stored) != chunk.compression_frame_digest {
            return Err(Error::CorruptRecord);
        }
        let start =
            usize::try_from(chunk.compression_frame_offset).map_err(|_| Error::CorruptRecord)?;
        let len = usize::try_from(chunk.len).map_err(|_| Error::CorruptRecord)?;
        let end = start.checked_add(len).ok_or(Error::CorruptRecord)?;
        if end > usize::try_from(chunk.compression_frame_len).map_err(|_| Error::CorruptRecord)? {
            return Err(Error::CorruptRecord);
        }
        let end = start + usize::try_from(range.end).map_err(|_| Error::CorruptRecord)?;
        let start = start + usize::try_from(range.start).map_err(|_| Error::CorruptRecord)?;
        if chunk.compression == COMPRESSION_NONE {
            if stored.len() as u64 != chunk.compression_frame_len || end > stored.len() {
                return Err(Error::CorruptRecord);
            }
            let out = stored[start..end].to_vec();
            if self.should_cache_decoded_compression_frame(stored.len()) {
                self.cache_decoded_compression_frame_owned(chunk, cache_slices, stored.to_vec());
            }
            return Ok(out);
        }

        let mut decoded = ZeroizingBytes::new(decode_compression_frame(
            chunk.compression,
            stored,
            chunk.compression_frame_len,
        )?);
        if start == 0
            && end == decoded.len()
            && !self.should_cache_decoded_compression_frame(decoded.len())
        {
            return Ok(std::mem::take(&mut *decoded));
        }
        let out = decoded[start..end].to_vec();
        self.cache_decoded_compression_frame_owned(
            chunk,
            cache_slices,
            std::mem::take(&mut *decoded),
        );
        Ok(out)
    }

    fn stage_small_file(
        &mut self,
        path: &LockboxPath,
        data: &[u8],
        permissions: u32,
        replace: bool,
    ) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        let path = path.file_path()?;
        let permissions = validate_permissions(permissions)?;
        self.validate_replace_intent(&path, replace)?;
        self.create_parent_dirs_for(&path)?;
        if let Some(old) = self.toc_entries.get(path.as_str()).cloned() {
            self.free_entry_slots(old)?;
        }

        self.insert_pending_small_file(
            path.clone(),
            PendingFileChunk {
                path: path.clone(),
                permissions,
                total_len: data.len() as u64,
                data: Arc::from(data),
            },
        );
        self.toc_entries.insert(
            path.clone(),
            TocEntry {
                path: path.clone(),
                len: data.len() as u64,
                record_offset: 0,
                record_len: 0,
                record_object_id: 0,
                deleted: false,
                node_kind: NodeKind::File,
                permissions,
                chunks: Vec::new(),
            },
        );
        self.mark_toc_dirty(&path);
        if self.should_discard_file_pages_after_flush()
            && self.pending_small_file_bytes >= MAX_SEGMENT_BYTES
        {
            self.flush_pending_small_files()?;
        }
        Ok(())
    }

    pub(crate) fn remove_pending_small_file(
        &mut self,
        path: &LockboxPath,
    ) -> Option<PendingFileChunk> {
        let removed = self.pending_small_files.remove(path);
        if let Some(pending) = removed.as_ref() {
            self.pending_small_file_bytes = self
                .pending_small_file_bytes
                .saturating_sub(pending.data.len());
        }
        removed
    }

    pub(crate) fn insert_pending_small_file(
        &mut self,
        path: LockboxPath,
        pending: PendingFileChunk,
    ) {
        let pending_len = pending.data.len();
        if let Some(old) = self.pending_small_files.insert(path, pending) {
            self.pending_small_file_bytes =
                self.pending_small_file_bytes.saturating_sub(old.data.len());
        }
        self.pending_small_file_bytes = self.pending_small_file_bytes.saturating_add(pending_len);
    }

    pub(crate) fn flush_pending_small_files(&mut self) -> Result<()> {
        if self.pending_small_files.is_empty() {
            return Ok(());
        }

        let pending = std::mem::take(&mut self.pending_small_files);
        self.pending_small_file_bytes = 0;
        let compression_frame_target = self.small_file_compression_frame_target();
        let mut writer = FilePageWriter::new(self);
        let mut all_chunks = Vec::new();
        let mut updates = Vec::new();
        let mut dirty_paths = Vec::new();
        let pending = pending.into_values().collect::<Vec<_>>();
        let mut batches = Vec::new();
        let mut batch = Vec::new();
        let mut batch_bytes = 0usize;
        for chunk in &pending {
            if !batch.is_empty()
                && batch_bytes.saturating_add(chunk.data.len()) > compression_frame_target
            {
                batches.push(batch);
                batch = Vec::new();
                batch_bytes = 0;
            }
            batch_bytes = batch_bytes.saturating_add(chunk.data.len());
            batch.push(CompressionFrameWrite {
                path: &chunk.path,
                permissions: chunk.permissions,
                total_len: chunk.total_len,
                file_offset: 0,
                data: &chunk.data,
            });
        }
        if !batch.is_empty() {
            batches.push(batch);
        }
        let batch_indices = writer.write_compression_frame_batches(&batches, &mut all_chunks)?;
        for (batch, indices) in batches.iter().zip(batch_indices) {
            for (frame, chunk_index) in batch.iter().zip(indices) {
                updates.push((
                    (*frame.path).clone(),
                    frame.permissions,
                    frame.total_len,
                    chunk_index,
                ));
            }
        }
        writer.finish(&mut all_chunks)?;
        for (path, permissions, total_len, chunk_index) in updates {
            let chunks = vec![all_chunks[chunk_index].clone()];
            if let Some(entry) = writer.lockbox.toc_entries.get_mut(path.as_str()) {
                entry.record_offset = chunks
                    .first()
                    .and_then(|chunk| chunk.segments.first())
                    .map(|segment| segment.page_offset)
                    .unwrap_or(0);
                entry.record_len = chunks
                    .first()
                    .and_then(|chunk| chunk.segments.first())
                    .map(|segment| segment.page_len)
                    .unwrap_or(0);
                entry.record_object_id = chunks
                    .first()
                    .and_then(|chunk| chunk.segments.first())
                    .map(|segment| segment.object_id)
                    .unwrap_or(0);
                entry.len = total_len;
                entry.permissions = permissions;
                entry.chunks = chunks;
                dirty_paths.push(entry.path.clone());
                let entry = entry.clone();
                writer.lockbox.add_entry_record_refs(&entry);
            }
        }
        writer.lockbox.mark_toc_dirty_paths(dirty_paths.iter());
        Ok(())
    }

    pub(crate) fn flush_bulk_small_file_packer(&mut self) -> Result<()> {
        self.flush_pending_small_files()
    }

    pub(crate) fn pack_small_file_pages(&mut self) -> Result<()> {
        let mut candidates = Vec::new();
        for entry in self.toc_entries.values() {
            if entry.deleted
                || entry.node_kind != NodeKind::File
                || entry.len > 1024 * 1024
                || entry_has_sparse_ranges(entry)
            {
                continue;
            }
            let data = self.get_file(&entry.path)?;
            candidates.push((entry.path.clone(), entry.permissions, data, entry.clone()));
        }

        if candidates.len() < 10 {
            return Ok(());
        }

        for (_, _, _, old) in &candidates {
            self.free_entry_slots(old.clone())?;
        }

        let compression_frame_target = self.small_file_compression_frame_target();
        let mut writer = FilePageWriter::new(self);
        let mut all_chunks = Vec::new();
        let mut updates = Vec::new();
        let mut dirty_paths = Vec::new();
        let mut batches = Vec::new();
        let mut batch = Vec::new();
        let mut batch_bytes = 0usize;
        for (path, permissions, data, _) in &candidates {
            if !batch.is_empty()
                && batch_bytes.saturating_add(data.len()) > compression_frame_target
            {
                batches.push(batch);
                batch = Vec::new();
                batch_bytes = 0;
            }
            let len = data.len() as u64;
            batch_bytes = batch_bytes.saturating_add(data.len());
            batch.push(CompressionFrameWrite {
                path,
                permissions: *permissions,
                total_len: len,
                file_offset: 0,
                data,
            });
        }
        if !batch.is_empty() {
            batches.push(batch);
        }
        let batch_indices = writer.write_compression_frame_batches(&batches, &mut all_chunks)?;
        for (batch, indices) in batches.iter().zip(batch_indices) {
            for (frame, chunk_index) in batch.iter().zip(indices) {
                updates.push((
                    (*frame.path).clone(),
                    frame.permissions,
                    frame.total_len,
                    chunk_index,
                ));
            }
        }
        writer.finish(&mut all_chunks)?;
        for (path, permissions, len, chunk_index) in updates {
            let chunks = vec![all_chunks[chunk_index].clone()];
            if let Some(entry) = writer.lockbox.toc_entries.get_mut(path.as_str()) {
                entry.record_offset = chunks
                    .first()
                    .and_then(|chunk| chunk.segments.first())
                    .map(|segment| segment.page_offset)
                    .unwrap_or(0);
                entry.record_len = chunks
                    .first()
                    .and_then(|chunk| chunk.segments.first())
                    .map(|segment| segment.page_len)
                    .unwrap_or(0);
                entry.record_object_id = chunks
                    .first()
                    .and_then(|chunk| chunk.segments.first())
                    .map(|segment| segment.object_id)
                    .unwrap_or(0);
                entry.len = len;
                entry.permissions = permissions;
                entry.chunks = chunks;
                dirty_paths.push(entry.path.clone());
                let entry = entry.clone();
                writer.lockbox.add_entry_record_refs(&entry);
            }
        }
        writer.lockbox.mark_toc_dirty_paths(dirty_paths.iter());
        Ok(())
    }

    fn small_file_compression_frame_target(&self) -> usize {
        match self.workload_profile {
            WorkloadProfile::BulkImport => BULK_IMPORT_SMALL_FILE_COMPRESSION_FRAME_BYTES,
            _ => SMALL_FILE_COMPRESSION_FRAME_BYTES,
        }
    }

    fn compression_frame_zstd_level(&self) -> i32 {
        match self.workload_profile {
            WorkloadProfile::BulkImport => ZSTD_BULK_IMPORT_LEVEL,
            _ => ZSTD_DEFAULT_LEVEL,
        }
    }

    fn configured_compression(&self) -> Option<crate::Compression> {
        (self.format_mode.0 != 0).then(|| self.format_mode.options().compression)
    }

    fn decoded_compression_frame_cache_limit(&self) -> usize {
        match self.workload_profile {
            WorkloadProfile::ReadMostly | WorkloadProfile::ExtractMany => {
                DECODED_COMPRESSION_FRAME_CACHE_BYTES
            }
            _ => 0,
        }
    }

    fn read_cached_compression_frame_slice(
        &self,
        expected_total_len: u64,
        chunk: &FileChunk,
        range: &std::ops::Range<u64>,
    ) -> Result<Option<Vec<u8>>> {
        #[cfg(any(test, feature = "native-block-layout"))]
        if chunk.block_frame.is_some() {
            return self
                .native_cached_frame(chunk)?
                .map(|entry| Self::cached_frame_slice(&entry, expected_total_len, chunk, range))
                .transpose();
        }
        let cache = self.compression_frame_cache.borrow();
        let Some(entry) = cache.entries.get(&chunk.compression_frame_id) else {
            return Ok(None);
        };
        Self::cached_frame_slice(entry, expected_total_len, chunk, range).map(Some)
    }

    #[cfg(any(test, feature = "native-block-layout"))]
    fn native_cached_frame(
        &self,
        chunk: &FileChunk,
    ) -> Result<Option<Arc<super::CachedCompressionFrame>>> {
        let [segment] = chunk.segments.as_slice() else {
            return Err(Error::CorruptRecord);
        };
        Ok(self
            .page_manager
            .borrow_mut()
            .native_decoded_page(segment.page_offset, self.storage.write_revision()))
    }

    #[cfg(any(test, feature = "native-block-layout"))]
    fn should_cache_native_decoded_frame(&self, len: usize) -> bool {
        let limit = self.page_manager.borrow().stats().limit_bytes;
        limit > 0 && len as u64 <= limit
    }

    fn cached_frame_slice(
        entry: &super::CachedCompressionFrame,
        expected_total_len: u64,
        chunk: &FileChunk,
        range: &std::ops::Range<u64>,
    ) -> Result<Vec<u8>> {
        let bounds = Self::cached_frame_bounds(entry, expected_total_len, chunk, range)?;
        Ok(entry.data[bounds].to_vec())
    }

    fn cached_frame_bounds(
        entry: &super::CachedCompressionFrame,
        expected_total_len: u64,
        chunk: &FileChunk,
        range: &std::ops::Range<u64>,
    ) -> Result<std::ops::Range<usize>> {
        if range.start > range.end || range.end > chunk.len {
            return Err(Error::CorruptRecord);
        }
        if entry.compression != chunk.compression
            || entry.compression_frame_len != chunk.compression_frame_len
            || entry.compressed_len != chunk.compressed_len
            || entry.compression_frame_digest != chunk.compression_frame_digest
        {
            return Err(Error::CorruptRecord);
        }
        let has_slice = entry.slices.iter().any(|slice| {
            slice.compression_frame_offset == chunk.compression_frame_offset
                && slice.len == chunk.len
                && slice.file_offset == chunk.file_offset
                && (slice.total_len == 0 || slice.total_len == expected_total_len)
                && slice.path == chunk.stored_path
        });
        if !has_slice {
            return Err(Error::CorruptRecord);
        }
        let start =
            usize::try_from(chunk.compression_frame_offset).map_err(|_| Error::CorruptRecord)?;
        let len = usize::try_from(chunk.len).map_err(|_| Error::CorruptRecord)?;
        let end = start.checked_add(len).ok_or(Error::CorruptRecord)?;
        if end > entry.data.len() {
            return Err(Error::CorruptRecord);
        }
        let end = start + usize::try_from(range.end).map_err(|_| Error::CorruptRecord)?;
        let start = start + usize::try_from(range.start).map_err(|_| Error::CorruptRecord)?;
        Ok(start..end)
    }

    fn cache_decoded_compression_frame_owned(
        &self,
        chunk: &FileChunk,
        slices: Vec<CompressionFrameSlice>,
        decoded: Vec<u8>,
    ) {
        drop(self.cache_decoded_compression_frame_at_revision(chunk, slices, decoded, 0));
    }

    // Some supplies native-frame ownership, not a promise of cache admission.
    // The caller may retain it through a write even if the cache evicts it.
    fn cache_decoded_compression_frame_at_revision(
        &self,
        chunk: &FileChunk,
        slices: Vec<CompressionFrameSlice>,
        mut decoded: Vec<u8>,
        _revision: u64,
    ) -> Option<Arc<super::CachedCompressionFrame>> {
        #[cfg(any(test, feature = "native-block-layout"))]
        if chunk.block_frame.is_some() {
            let [segment] = chunk.segments.as_slice() else {
                crate::page_buffer::zeroize_bytes(&mut decoded);
                return None;
            };
            return Some(
                self.page_manager.borrow_mut().insert_native_decoded_page(
                    segment.page_offset,
                    segment.page_len,
                    super::CachedCompressionFrame {
                        native_reference: chunk
                            .block_frame
                            .as_ref()
                            .map(|reference| ((**reference).clone(), segment.clone())),
                        compression: chunk.compression,
                        compression_frame_len: chunk.compression_frame_len,
                        compressed_len: chunk.compressed_len,
                        compression_frame_digest: chunk.compression_frame_digest,
                        slices,
                        data: decoded,
                    },
                    _revision,
                ),
            );
        }
        if !self.should_cache_decoded_compression_frame(decoded.len()) {
            crate::page_buffer::zeroize_bytes(&mut decoded);
            return None;
        }
        self.insert_decoded_compression_frame(chunk, slices, decoded);
        None
    }

    fn should_cache_decoded_compression_frame(&self, decoded_len: usize) -> bool {
        let limit = self.decoded_compression_frame_cache_limit();
        limit > 0 && decoded_len <= limit
    }

    fn insert_decoded_compression_frame(
        &self,
        chunk: &FileChunk,
        slices: Vec<CompressionFrameSlice>,
        decoded: Vec<u8>,
    ) {
        let decoded_len = decoded.len();
        let limit = self.decoded_compression_frame_cache_limit();
        let mut cache = self.compression_frame_cache.borrow_mut();
        if let Some(old) = cache.entries.remove(&chunk.compression_frame_id) {
            cache.used_bytes = cache.used_bytes.saturating_sub(old.data.len());
        }
        while cache.used_bytes.saturating_add(decoded_len) > limit {
            let Some(key) = cache.entries.keys().next().copied() else {
                break;
            };
            if let Some(old) = cache.entries.remove(&key) {
                cache.used_bytes = cache.used_bytes.saturating_sub(old.data.len());
            }
        }
        cache.entries.insert(
            chunk.compression_frame_id,
            super::CachedCompressionFrame {
                #[cfg(any(test, feature = "native-block-layout"))]
                native_reference: None,
                compression: chunk.compression,
                compression_frame_len: chunk.compression_frame_len,
                compressed_len: chunk.compressed_len,
                compression_frame_digest: chunk.compression_frame_digest,
                slices,
                data: decoded,
            },
        );
        cache.used_bytes = cache.used_bytes.saturating_add(decoded_len);
    }

    #[cfg(test)]
    pub(crate) fn decoded_compression_frame_cache_entries_for_tests(&self) -> usize {
        self.compression_frame_cache.borrow().entries.len()
            + self.page_manager.borrow().native_decoded_entries()
    }

    /// Stream file content ranges without extracting files to the host filesystem.
    pub fn stream_content<F>(&self, options: ContentStreamOptions, visitor: F) -> Result<()>
    where
        F: FnMut(ContentChunk, &mut dyn Read) -> Result<()>,
    {
        // Workspace setup and final wiping must amortize across enough data.
        // Count file slices, not shared frame lengths, so many tiny files in
        // one cached frame do not spuriously select the larger allocation.
        let mut compressed_bytes = 0u64;
        let reuse = self
            .toc_entries
            .values()
            .filter(|entry| !entry.deleted && entry.node_kind == NodeKind::File)
            .flat_map(|entry| &entry.chunks)
            .any(|chunk| {
                if chunk.compression != COMPRESSION_NONE && chunk.compression_frame_len >= 64 * 1024
                {
                    compressed_bytes = compressed_bytes.saturating_add(chunk.len);
                }
                compressed_bytes >= 8 * 1024 * 1024
            });
        if reuse {
            crate::compression::with_decode_workspace(|| {
                self.stream_content_inner(options, visitor)
            })
        } else {
            self.stream_content_inner(options, visitor)
        }
    }

    fn stream_content_inner<F>(&self, options: ContentStreamOptions, mut visitor: F) -> Result<()>
    where
        F: FnMut(ContentChunk, &mut dyn Read) -> Result<()>,
    {
        // A sequential pass larger than the cache evicts single-file frames
        // before a later pass can reuse them. Packed frames still benefit
        // neighboring slices within this pass.
        #[cfg(any(test, feature = "native-block-layout"))]
        let stream_len = self
            .toc_entries
            .values()
            .filter(|entry| !entry.deleted && entry.node_kind == NodeKind::File)
            .fold(0u64, |total, entry| total.saturating_add(entry.len));
        #[cfg(any(test, feature = "native-block-layout"))]
        let cache_single_frames = stream_len <= self.decoded_compression_frame_cache_limit() as u64;
        #[cfg(not(any(test, feature = "native-block-layout")))]
        let cache_single_frames = false;
        let mut stream_state = ContentStreamState::default();
        match options.order {
            ContentStreamOrder::Logical => {
                for entry in self.toc_entries.values() {
                    if entry.deleted || entry.node_kind != NodeKind::File {
                        continue;
                    }
                    self.stream_file_content(
                        entry,
                        &mut visitor,
                        cache_single_frames,
                        &mut stream_state,
                    )?;
                }
            }
            ContentStreamOrder::Physical => {
                let mut items = Vec::new();
                for entry in self.toc_entries.values() {
                    if entry.deleted || entry.node_kind != NodeKind::File {
                        continue;
                    }
                    if self.pending_small_files.contains_key(&entry.path) {
                        if entry.len > 0 {
                            items.push(ContentStreamItem {
                                path: entry.path.clone(),
                                file_offset: 0,
                                len: entry.len,
                                total_len: entry.len,
                                physical_offset: None,
                                sparse: false,
                                chunk: None,
                            });
                        }
                        continue;
                    }
                    collect_content_stream_items(entry, &mut items)?;
                }
                items.sort_by(|left, right| {
                    left.physical_offset
                        .unwrap_or(u64::MAX)
                        .cmp(&right.physical_offset.unwrap_or(u64::MAX))
                        .then_with(|| left.path.cmp(&right.path))
                        .then_with(|| left.file_offset.cmp(&right.file_offset))
                });
                for item in items {
                    self.visit_content_stream_item(
                        item,
                        &mut visitor,
                        cache_single_frames,
                        &mut stream_state,
                    )?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn stream_file_content<'a, F>(
        &'a self,
        entry: &TocEntry,
        visitor: &mut F,
        cache_single_frames: bool,
        stream_state: &mut ContentStreamState<'a>,
    ) -> Result<()>
    where
        F: FnMut(ContentChunk, &mut dyn Read) -> Result<()>,
    {
        if self.pending_small_files.contains_key(&entry.path) {
            if entry.len > 0 {
                self.visit_content_stream_item(
                    ContentStreamItem {
                        path: entry.path.clone(),
                        file_offset: 0,
                        len: entry.len,
                        total_len: entry.len,
                        physical_offset: None,
                        sparse: false,
                        chunk: None,
                    },
                    visitor,
                    cache_single_frames,
                    stream_state,
                )?;
            }
            return Ok(());
        }
        let mut items = Vec::new();
        collect_content_stream_items(entry, &mut items)?;
        for item in items {
            self.visit_content_stream_item(item, visitor, cache_single_frames, stream_state)?;
        }
        Ok(())
    }

    fn visit_content_stream_item<'a, F>(
        &'a self,
        item: ContentStreamItem,
        visitor: &mut F,
        _cache_single_frames: bool,
        _state: &mut ContentStreamState<'a>,
    ) -> Result<()>
    where
        F: FnMut(ContentChunk, &mut dyn Read) -> Result<()>,
    {
        let chunk = ContentChunk {
            path: item.path.clone(),
            file_offset: item.file_offset,
            len: item.len,
            physical_offset: item.physical_offset,
            sparse: item.sparse,
        };
        #[cfg(any(test, feature = "native-block-layout"))]
        if let Some(reference) = item.chunk.as_ref().filter(|reference| {
            !item.sparse
                && reference.block_frame.is_some()
                && reference.len < reference.compression_frame_len
        }) {
            use crate::file_format::indexed_frame::block_page::validate_chunk_reference;
            validate_chunk_reference(self.lockbox_id, self.format_mode, item.total_len, reference)?;
            if !_state.native.as_ref().is_some_and(|retained| {
                retained.reference.block_frame == reference.block_frame
                    && retained.reference.segments == reference.segments
            }) {
                // Drop and wipe the previous frame before allocating another.
                _state.native = None;
                let reader = self.open_native_block_chunk(item.total_len, reference)?;
                let cached = {
                    if let Some(entry) = self.native_cached_frame(reference)? {
                        if entry.compression != reference.compression
                            || entry.compression_frame_len != reference.compression_frame_len
                            || entry.compressed_len != reference.compressed_len
                            || entry.compression_frame_digest != reference.compression_frame_digest
                            || entry.slices != reader.slices()
                        {
                            return Err(Error::CorruptRecord);
                        }
                        Some(ZeroizingBytes::new(entry.data.clone()))
                    } else {
                        None
                    }
                };
                let was_cached = cached.is_some();
                let data = match cached {
                    Some(data) => data,
                    None => ZeroizingBytes::new(reader.read(0..reference.compression_frame_len)?),
                };
                if !was_cached
                    && self.should_cache_decoded_compression_frame(data.len())
                    && self.should_cache_native_decoded_frame(data.len())
                {
                    let _ = self.cache_decoded_compression_frame_at_revision(
                        reference,
                        reader.slices().to_vec(),
                        data.to_vec(),
                        reader.write_revision(),
                    );
                }
                _state.native = Some(NativeStreamFrame {
                    reference: reference.clone(),
                    reader,
                    data,
                });
            }
            let retained = _state.native.as_ref().ok_or(Error::CorruptRecord)?;
            retained.reader.validate_slice(reference, item.total_len)?;
            retained.reader.ensure_current()?;
            let start = usize::try_from(reference.compression_frame_offset)
                .map_err(|_| Error::CorruptRecord)?;
            let end = start
                .checked_add(usize::try_from(reference.len).map_err(|_| Error::CorruptRecord)?)
                .ok_or(Error::CorruptRecord)?;
            let bytes = retained.data.get(start..end).ok_or(Error::CorruptRecord)?;
            return visitor(chunk, &mut Cursor::new(bytes));
        }
        #[cfg(any(test, feature = "native-block-layout"))]
        {
            _state.native = None;
        }
        if item.sparse {
            let mut reader = ZeroReader {
                remaining: item.len,
            };
            visitor(chunk, &mut reader)?;
        } else {
            let data = match item.chunk.as_ref() {
                #[cfg(any(test, feature = "native-block-layout"))]
                Some(chunk) if chunk.block_frame.is_some() => self
                    .read_native_block_chunk_with_cache(
                        item.total_len,
                        chunk,
                        0..chunk.len,
                        _cache_single_frames,
                    )?,
                Some(chunk) => self.read_file_chunk_compression_frame(item.total_len, chunk)?,
                None => self.read_file_range(&item.path, item.file_offset, item.len)?,
            };
            let mut reader = Cursor::new(data);
            visitor(chunk, &mut reader)?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct ContentStreamState<'a> {
    #[cfg(any(test, feature = "native-block-layout"))]
    native: Option<NativeStreamFrame<'a>>,
    #[cfg(not(any(test, feature = "native-block-layout")))]
    _lifetime: std::marker::PhantomData<&'a ()>,
}

#[cfg(any(test, feature = "native-block-layout"))]
// One verified packed frame retained within a stream call. The reader checks
// the source before each slice; dropping this state wipes data, including on
// visitor error or unwind. It never retains a second frame during replacement.
struct NativeStreamFrame<'a> {
    reference: FileChunk,
    reader: crate::file_format::indexed_frame::block_page::Reader<'a>,
    data: ZeroizingBytes,
}

#[derive(Debug)]
struct ContentStreamItem {
    path: LockboxPath,
    file_offset: u64,
    len: u64,
    total_len: u64,
    physical_offset: Option<u64>,
    sparse: bool,
    chunk: Option<FileChunk>,
}

struct ZeroReader {
    remaining: u64,
}

impl Read for ZeroReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.remaining == 0 || buf.is_empty() {
            return Ok(0);
        }
        let len = buf.len().min(self.remaining as usize);
        buf[..len].fill(0);
        self.remaining -= len as u64;
        Ok(len)
    }
}

fn collect_content_stream_items(
    entry: &TocEntry,
    items: &mut Vec<ContentStreamItem>,
) -> Result<()> {
    if entry.len == 0 {
        return Ok(());
    }
    let mut chunks = entry.chunks.clone();
    chunks.sort_by_key(|chunk| chunk.file_offset);
    let mut cursor = 0u64;
    for chunk in chunks {
        let chunk_end = chunk.file_offset.saturating_add(chunk.len);
        if chunk.file_offset < cursor || chunk_end > entry.len {
            return Err(Error::CorruptRecord);
        }
        if chunk.file_offset > cursor {
            items.push(ContentStreamItem {
                path: entry.path.clone(),
                file_offset: cursor,
                len: chunk.file_offset - cursor,
                total_len: entry.len,
                physical_offset: None,
                sparse: true,
                chunk: None,
            });
        }
        if chunk.len > 0 {
            items.push(ContentStreamItem {
                path: entry.path.clone(),
                file_offset: chunk.file_offset,
                len: chunk.len,
                total_len: entry.len,
                physical_offset: first_physical_offset(&chunk),
                sparse: false,
                chunk: Some(chunk.clone()),
            });
        }
        cursor = chunk_end;
    }
    if cursor < entry.len {
        items.push(ContentStreamItem {
            path: entry.path.clone(),
            file_offset: cursor,
            len: entry.len - cursor,
            total_len: entry.len,
            physical_offset: None,
            sparse: true,
            chunk: None,
        });
    }
    Ok(())
}

fn first_physical_offset(chunk: &FileChunk) -> Option<u64> {
    chunk
        .segments
        .iter()
        .map(|segment| segment.page_offset)
        .min()
}

fn entry_has_sparse_ranges(entry: &TocEntry) -> bool {
    if entry.len == 0 {
        return false;
    }
    let mut chunks = entry.chunks.clone();
    chunks.sort_by_key(|chunk| chunk.file_offset);
    let mut cursor = 0u64;
    for chunk in chunks {
        if chunk.file_offset != cursor {
            return true;
        }
        cursor = cursor.saturating_add(chunk.len);
    }
    cursor != entry.len
}

pub(super) fn ranges_overlap(
    left_start: u64,
    left_end: u64,
    right_start: u64,
    right_end: u64,
) -> bool {
    left_start < right_end && right_start < left_end
}

pub(super) fn trim_zeroes(bytes: &[u8]) -> Option<(usize, usize)> {
    let start = bytes.iter().position(|byte| *byte != 0)?;
    let end = bytes
        .iter()
        .rposition(|byte| *byte != 0)
        .map(|index| index + 1)?;
    Some((start, end))
}

fn write_zeroes(writer: &mut impl Write, mut len: u64) -> Result<()> {
    const ZERO_BUF: [u8; 8192] = [0; 8192];
    while len > 0 {
        let write_len = ZERO_BUF.len().min(len as usize);
        writer
            .write_all(&ZERO_BUF[..write_len])
            .map_err(|err| Error::Io(err.to_string()))?;
        len -= write_len as u64;
    }
    Ok(())
}

pub(super) fn seek_position(current: u64, len: u64, pos: SeekFrom) -> io::Result<u64> {
    let absolute = match pos {
        SeekFrom::Start(offset) => offset as i128,
        SeekFrom::End(offset) => len as i128 + offset as i128,
        SeekFrom::Current(offset) => current as i128 + offset as i128,
    };
    if absolute < 0 || absolute > u64::MAX as i128 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid seek before start of file",
        ));
    }
    Ok(absolute as u64)
}

pub(super) fn to_io_error(err: Error) -> io::Error {
    io::Error::other(err)
}

fn read_next_chunk(reader: &mut impl Read, buffer: &mut [u8]) -> Result<usize> {
    let mut read_total = 0usize;
    while read_total < buffer.len() {
        let read = reader
            .read(&mut buffer[read_total..])
            .map_err(|err| Error::Io(err.to_string()))?;
        if read == 0 {
            break;
        }
        read_total += read;
    }
    Ok(read_total)
}

struct ParallelFrameOrder {
    pending: BTreeMap<usize, PreparedCompressionFrame>,
    next_index: usize,
    received_count: usize,
}

impl ParallelFrameOrder {
    fn new() -> Self {
        Self {
            pending: BTreeMap::new(),
            next_index: 0,
            received_count: 0,
        }
    }

    fn drain_ready<State>(
        &mut self,
        result_rx: &std::sync::mpsc::Receiver<ParallelCompressionResult>,
        writer: &mut FilePageWriter<'_, State>,
        chunks: &mut Vec<FileChunk>,
    ) -> Result<()> {
        loop {
            match result_rx.try_recv() {
                Ok(result) => self.accept(result, writer, chunks)?,
                Err(std::sync::mpsc::TryRecvError::Empty) => return Ok(()),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    return Err(Error::Io(
                        "compression worker stopped unexpectedly".to_string(),
                    ));
                }
            }
        }
    }

    fn accept<State>(
        &mut self,
        result: ParallelCompressionResult,
        writer: &mut FilePageWriter<'_, State>,
        chunks: &mut Vec<FileChunk>,
    ) -> Result<()> {
        self.received_count += 1;
        self.pending.insert(result.index, result.frame);
        while let Some(frame) = self.pending.remove(&self.next_index) {
            writer.write_prepared_compression_frame(frame, chunks)?;
            self.next_index += 1;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct PendingSegment {
    chunk_indices: Vec<usize>,
    segment_offset: u64,
    segment_len: u64,
}

pub(super) struct FilePageWriter<'a, State> {
    lockbox: &'a mut Lockbox<State>,
    packer: PageObjectPacker<PendingSegment>,
    #[cfg(any(test, feature = "native-block-layout"))]
    native_blocks: bool,
}

struct SharedCompressionFrameSurvivor {
    path: LockboxPath,
    permissions: u32,
    total_len: u64,
    file_offset: u64,
    data: Vec<u8>,
}

impl Drop for SharedCompressionFrameSurvivor {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

impl<'a, State> FilePageWriter<'a, State> {
    pub(super) fn new(lockbox: &'a mut Lockbox<State>) -> Self {
        #[cfg(any(test, feature = "native-block-layout"))]
        let native_blocks = cfg!(feature = "native-block-layout") && lockbox.format_mode.0 != 0;
        Self {
            lockbox,
            packer: PageObjectPacker::new(DEFAULT_PAGE_BYTES),
            #[cfg(any(test, feature = "native-block-layout"))]
            native_blocks,
        }
    }

    #[cfg(test)]
    pub(super) fn native(lockbox: &'a mut Lockbox<State>) -> Self {
        Self {
            native_blocks: true,
            ..Self::new(lockbox)
        }
    }

    pub(super) fn write_compression_frame(
        &mut self,
        frame: CompressionFrameWrite<'_>,
        chunks: &mut Vec<FileChunk>,
    ) -> Result<()> {
        self.write_compression_frame_bundle(&[frame], chunks)
            .map(|_| ())
    }

    pub(super) fn write_compression_frame_bundle(
        &mut self,
        frames: &[CompressionFrameWrite<'_>],
        chunks: &mut Vec<FileChunk>,
    ) -> Result<Vec<usize>> {
        let prepared = self.import_pipeline(1).prepare(frames);
        self.write_prepared_compression_frame(prepared, chunks)
    }

    fn write_compression_frame_batches(
        &mut self,
        batches: &[Vec<CompressionFrameWrite<'_>>],
        chunks: &mut Vec<FileChunk>,
    ) -> Result<Vec<Vec<usize>>> {
        let prepared = self
            .import_pipeline(self.lockbox.worker_jobs())
            .prepare_batches(batches);
        let mut indices = Vec::with_capacity(prepared.len());
        for frame in prepared {
            indices.push(self.write_prepared_compression_frame(frame, chunks)?);
        }
        Ok(indices)
    }

    fn import_pipeline(&self, jobs: usize) -> FileImportPipeline {
        let pipeline = FileImportPipeline::new(self.lockbox.compression_frame_zstd_level(), jobs)
            .with_compression(self.lockbox.configured_compression());
        #[cfg(any(test, feature = "native-block-layout"))]
        let pipeline = pipeline.with_native_blocks(self.native_blocks);
        pipeline
    }

    fn write_prepared_compression_frame(
        &mut self,
        prepared: PreparedCompressionFrame,
        chunks: &mut Vec<FileChunk>,
    ) -> Result<Vec<usize>> {
        #[cfg(any(test, feature = "native-block-layout"))]
        if self.native_blocks {
            return self.write_prepared_native_frame(prepared, chunks);
        }
        let compression_frame_digest = prepared.integrity.legacy_checksum()?;
        self.lockbox.add_frame_prepare_nanos(prepared.prepare_nanos);
        self.lockbox.sequence += 1;
        let compression_frame_id = self.lockbox.sequence;
        let mut chunk_indices = Vec::with_capacity(prepared.slices.len());
        let manifest = CompressionFrameManifest {
            compression_frame_id,
            compression: prepared.compression,
            compression_frame_len: prepared.compression_frame_len,
            compressed_len: prepared.compressed_len,
            compression_frame_digest,
            slices: prepared.slices,
        };
        for slice in &manifest.slices {
            let chunk_index = chunks.len();
            chunks.push(FileChunk {
                #[cfg(any(test, feature = "native-block-layout"))]
                block_frame: None,
                stored_path: slice.path.clone(),
                file_offset: slice.file_offset,
                len: slice.len,
                compression_frame_offset: slice.compression_frame_offset,
                compression_frame_len: manifest.compression_frame_len,
                compressed_len: manifest.compressed_len,
                compression: manifest.compression,
                compression_frame_id,
                compression_frame_digest: manifest.compression_frame_digest,
                segments: Vec::new(),
            });
            chunk_indices.push(chunk_index);
        }

        if prepared.stored.is_empty() {
            self.add_segment(&manifest, 0, &chunk_indices, &[], chunks)?;
            return Ok(chunk_indices);
        }

        let mut offset = 0usize;
        while offset < prepared.stored.len() {
            let end = (offset + MAX_SEGMENT_BYTES).min(prepared.stored.len());
            self.add_segment(
                &manifest,
                offset as u64,
                &chunk_indices,
                &prepared.stored[offset..end],
                chunks,
            )?;
            offset = end;
        }
        Ok(chunk_indices)
    }

    #[cfg(any(test, feature = "native-block-layout"))]
    fn write_prepared_native_frame(
        &mut self,
        prepared: PreparedCompressionFrame,
        chunks: &mut Vec<FileChunk>,
    ) -> Result<Vec<usize>> {
        use crate::file_chunk::BlockFrameReference;
        use crate::file_format::indexed_frame::block_page::PageIdentity;
        use crate::page_cache::PageWritePolicy;
        self.flush(chunks)?;
        self.lockbox.add_frame_prepare_nanos(prepared.prepare_nanos);
        let started = Instant::now();
        let frame_id = self
            .lockbox
            .sequence
            .checked_add(1)
            .ok_or(Error::CorruptRecord)?;
        let page_id = frame_id.checked_add(1).ok_or(Error::CorruptRecord)?;
        let identity = PageIdentity {
            archive: self.lockbox.lockbox_id,
            mode: self.lockbox.format_mode,
            page_id,
            sequence: page_id,
        };
        let page = std::sync::Arc::new(
            self.lockbox
                .key
                .with_bytes(|key| prepared.encode_native_page(identity, frame_id, key))??,
        );
        self.lockbox.sequence = page_id;
        let page_len = page.bytes().len() as u64;
        let offset = self.lockbox.allocate_page_offset(page_len)?;
        self.lockbox.page_manager.borrow_mut().stage_native_page(
            offset,
            self.lockbox.lockbox_id,
            page.clone(),
            PageWritePolicy::DiscardAfterFlush,
        )?;
        // The normal preparation protocol owns both reused and appended extents.
        // Publish no chunk reference until those bytes have actually been written.
        self.lockbox.flush_discardable_pages()?;
        let reference = std::sync::Arc::new(BlockFrameReference {
            descriptor: page.descriptor().clone(),
            sequence: identity.sequence,
        });
        let mut indices = Vec::with_capacity(prepared.slices.len());
        for slice in prepared.slices {
            indices.push(chunks.len());
            chunks.push(FileChunk {
                block_frame: Some(reference.clone()),
                stored_path: slice.path,
                file_offset: slice.file_offset,
                len: slice.len,
                compression_frame_offset: slice.compression_frame_offset,
                compression_frame_len: reference.descriptor.logical_len,
                compressed_len: reference.descriptor.stored_len,
                compression: reference.descriptor.compression,
                compression_frame_id: frame_id,
                compression_frame_digest: reference.descriptor.index_commitment,
                segments: vec![CompressionFrameSegment {
                    page_offset: offset,
                    page_len,
                    object_id: page_id,
                    segment_offset: 0,
                    segment_len: reference.descriptor.stored_len,
                }],
            });
        }
        self.lockbox
            .add_page_write_nanos(started.elapsed().as_nanos());
        Ok(indices)
    }

    fn add_segment(
        &mut self,
        manifest: &CompressionFrameManifest,
        segment_offset: u64,
        chunk_indices: &[usize],
        segment: &[u8],
        chunks: &mut [FileChunk],
    ) -> Result<()> {
        self.lockbox.sequence += 1;
        let object_id = self.lockbox.sequence;
        let payload = crate::payload::encode_compression_frame_segment_payload_with_compression(
            manifest,
            segment_offset,
            segment,
            self.lockbox.configured_compression(),
        )?;
        let object = PageObject::new(PageObjectKind::FileData, object_id, payload);
        let context = PendingSegment {
            chunk_indices: chunk_indices.to_vec(),
            segment_offset,
            segment_len: segment.len() as u64,
        };

        let encoded_len = self.packer.encoded_object_len(&object)?;
        if !self.packer.is_empty() && !self.fits_with(encoded_len)? {
            self.flush(chunks)?;
        }
        if !self.fits_with(encoded_len)? {
            return Err(Error::SecurityLimitExceeded(
                "file segment does not fit in a page".to_string(),
            ));
        }
        self.packer.push_encoded(object, context, encoded_len)?;
        Ok(())
    }

    pub(super) fn finish(&mut self, chunks: &mut [FileChunk]) -> Result<()> {
        self.flush(chunks)
    }

    fn fits_with(&self, encoded_len: usize) -> Result<bool> {
        self.packer.fits_encoded_len(encoded_len)
    }

    fn flush(&mut self, chunks: &mut [FileChunk]) -> Result<()> {
        if self.packer.is_empty() {
            return Ok(());
        }
        let write_start = Instant::now();
        let pending = self.packer.pending().to_vec();
        let objects = pending
            .iter()
            .map(|pending| pending.object.clone())
            .collect::<Vec<_>>();
        let page_size = crate::page::page_size_for_encoded_objects_with_format(
            &objects,
            self.lockbox.format_mode,
        )?;
        let page_offset = self.lockbox.allocate_page_offset(page_size as u64)?;
        if self.lockbox.should_discard_file_pages_after_flush() {
            self.lockbox
                .write_insert_only_page_at(page_offset, self.lockbox.sequence, objects)?;
            self.lockbox.flush_discardable_pages()?;
        } else {
            self.lockbox
                .write_decoded_page_at(page_offset, self.lockbox.sequence, objects)?;
        }
        for pending in pending {
            for chunk_index in pending.context.chunk_indices {
                if let Some(chunk) = chunks.get_mut(chunk_index) {
                    chunk.segments.push(CompressionFrameSegment {
                        page_offset,
                        page_len: page_size as u64,
                        object_id: pending.object.id,
                        segment_offset: pending.context.segment_offset,
                        segment_len: pending.context.segment_len,
                    });
                }
            }
        }
        self.packer.clear();
        self.lockbox
            .add_page_write_nanos(write_start.elapsed().as_nanos());
        Ok(())
    }
}
