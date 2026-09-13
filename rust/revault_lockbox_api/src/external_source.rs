//! Read-only archives supplied by an immutable external byte source.
//!
//! Enabled by `external-source`. HTTP transport belongs to the embedding: bind
//! every range to one immutable revision, validate exact lengths and ranges, and
//! reject replacement. Use [`ExternalReader::try_read`] for every operation;
//! missing bytes cannot be mistaken for successful fallback or absent content.
//! Closures must be repeatable and must not publish results or perform external
//! side effects until `try_read` returns `Ok`.
//!
//! A borrowed handle cannot escape the checked operation:
//! ```compile_fail
//! use revault_lockbox_api::external_source::ExternalReader;
//! fn escape(reader: &mut ExternalReader) {
//!     let leaked = reader.try_read(|archive| Ok(archive));
//! }
//! ```
//! Writes are unavailable through the read-only handle:
//! ```compile_fail
//! use revault_lockbox_api::{LockboxPath, external_source::ExternalReader};
//! fn write(reader: &mut ExternalReader) {
//!     reader.try_read(|archive| archive.add_file(&LockboxPath::new("/x")?, b"x", false));
//! }
//! ```
//!
//! ```
//! use std::sync::Arc;
//! use revault_lockbox_api::{Encryption, Lockbox, LockboxCreateOptions, LockboxPath, Signing};
//! use revault_lockbox_api::external_source::{ExternalReader, ExternalReaderOptions,
//!     ExternalReadError, SourceError, SparseSource};
//! // Generate a public archive solely to demonstrate the retry contract.
//! let mut writer = Lockbox::create_in_memory_with_options(
//!     LockboxCreateOptions::new(Encryption::None, Signing::None))?;
//! let path = LockboxPath::new("/hello.txt")?;
//! writer.add_file(&path, b"hello", false)?;
//! writer.commit()?;
//! let bytes = writer.try_to_bytes()?;
//! let source = Arc::new(SparseSource::new(bytes.len() as u64,
//!     "immutable-revision-1".into(), 4096, bytes.len() + 4096)?);
//! let mut reader = ExternalReader::new(source.clone(), ExternalReaderOptions::default())?;
//! let contents = loop {
//!     match reader.try_read(|archive| archive.get_file(&path)) {
//!         Ok(contents) => break contents,
//!         Err(ExternalReadError::Source(SourceError::MissingRange {offset, length})) => {
//!             // In a browser, await an exact, revision-checked HTTP range here.
//!             let block = bytes[offset as usize..offset as usize + length].to_vec();
//!             source.supply("immutable-revision-1", offset, block)?;
//!         }
//!         Err(error) => return Err(error.into()),
//!     }
//! };
//! assert_eq!(contents, b"hello");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
use crate::storage::{Storage, StorageBackend};
use crate::{Error, Lockbox, LockboxOptions, ReadOnly, SecretVec};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// A failure to obtain bytes from an immutable external source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceError {
    /// Fetch this bounded range from the same revision, then retry the operation.
    MissingRange {
        /// Absolute byte offset in the archive.
        offset: u64,
        /// Exact number of bytes required (may include block alignment).
        length: usize,
    },
    /// The bytes no longer belong to the original revision. Create a new reader.
    VersionChanged,
    /// The reader or source has been cancelled and cannot be reused.
    Cancelled,
    /// Invalid transport data, a short read, or another terminal source failure.
    Unavailable(String),
}
impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for SourceError {}

/// An immutable random-access source. Implementations may fetch synchronously or
/// return `MissingRange` for an asynchronous caller to satisfy between attempts.
///
/// The length and byte contents MUST remain fixed for the lifetime of a reader.
/// A transport must verify source identity on every fetch, including refetches
/// after eviction. This contract does not authenticate unsigned archive content.
pub trait ReadAtSource: Send + Sync + 'static {
    /// Fixed logical archive length, without materializing its bytes.
    fn len(&self) -> u64;
    /// Whether the logical source is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Read into `output`, returning the number of bytes actually supplied.
    /// Short successful reads are rejected by reVault as terminal failures.
    fn read_at(&self, offset: u64, output: &mut [u8]) -> Result<usize, SourceError>;
    /// Check cancellation or known revision failure, even on a decoded-cache hit.
    fn validate(&self) -> Result<(), SourceError> {
        Ok(())
    }
}

/// A source failure, a normal archive error, or an exhausted retry budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalReadError {
    /// A structured source failure; only `MissingRange` is retryable.
    Source(SourceError),
    /// A normal archive validation, lookup, or decoding failure.
    Archive(Error),
    /// Too many consecutive incomplete attempts; create a new reader with an
    /// appropriate cache/operation size instead of retrying indefinitely.
    RetryLimitExceeded,
}
impl fmt::Display for ExternalReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ExternalReadError {}

/// Resource limits for one external reader. Sparse block memory is separately
/// bounded by [`SparseSource`]; decoded-page memory uses `lockbox.cache_limit`.
#[derive(Debug, Clone, Copy)]
pub struct ExternalReaderOptions {
    /// Maximum size of one physical source read, before allocation (default 64 MiB).
    pub max_read_bytes: usize,
    /// Maximum consecutive attempts that need more data (default 4096).
    pub max_missing_attempts: usize,
    /// Maximum cumulative physical read bytes across an operation and its retries
    /// (default 256 MiB), including reads satisfied from the sparse cache.
    pub max_source_bytes_per_operation: u64,
    /// Existing decoded-page cache and workload options.
    pub lockbox: LockboxOptions,
}
impl Default for ExternalReaderOptions {
    fn default() -> Self {
        Self {
            max_read_bytes: 64 * 1024 * 1024,
            max_missing_attempts: 4096,
            max_source_bytes_per_operation: 256 * 1024 * 1024,
            lockbox: LockboxOptions::default(),
        }
    }
}

/// A serialized, read-only external archive session. No raw lockbox or file
/// handle can escape the operation's borrow. Every operation checks source
/// failures independently of internal fallback results.
pub struct ExternalReader {
    storage: ExternalStorage,
    archive: Option<Lockbox<ReadOnly>>,
    key: Option<SecretVec>,
    options: ExternalReaderOptions,
    missing_attempts: usize,
    terminal: Option<ExternalReadError>,
}
impl ExternalReader {
    /// Create a lazy reader for an unencrypted archive (signed or unsigned).
    pub fn new(
        source: Arc<dyn ReadAtSource>,
        options: ExternalReaderOptions,
    ) -> Result<Self, ExternalReadError> {
        if source.is_empty()
            || options.max_read_bytes == 0
            || options.max_missing_attempts == 0
            || options.max_source_bytes_per_operation == 0
        {
            return Err(ExternalReadError::Archive(Error::InvalidInput(
                "external source length and limits must be nonzero".into(),
            )));
        }
        Ok(Self {
            storage: ExternalStorage {
                length: source.len(),
                source,
                failure: Arc::new(Mutex::new(None)),
                max_read_bytes: options.max_read_bytes,
                max_operation_bytes: options.max_source_bytes_per_operation,
                operation_bytes: Arc::new(Mutex::new(0)),
            },
            archive: None,
            key: None,
            options,
            missing_attempts: 0,
            terminal: None,
        })
    }
    /// Create a reader with an already obtained content key. Password/contact
    /// discovery is deliberately not performed by downloading the whole archive.
    pub fn with_content_key(
        source: Arc<dyn ReadAtSource>,
        key: SecretVec,
        options: ExternalReaderOptions,
    ) -> Result<Self, ExternalReadError> {
        let mut reader = Self::new(source, options)?;
        reader.key = Some(key);
        Ok(reader)
    }
    /// Cancel this session and release its decoded cache and retained content key.
    pub fn cancel(&mut self) {
        self.archive = None;
        self.key = None;
        self.terminal = Some(ExternalReadError::Source(SourceError::Cancelled));
    }
    /// Attempt one repeatable read operation. Fetch `MissingRange` externally and
    /// call again; all other source failures terminate this reader. A failed
    /// source read invalidates the parsed archive, so fallback-created partial
    /// metadata is never reused. Closures must defer side effects until success.
    pub fn try_read<T>(
        &mut self,
        operation: impl FnOnce(&Lockbox<ReadOnly>) -> crate::Result<T>,
    ) -> Result<T, ExternalReadError> {
        if let Some(error) = &self.terminal {
            return Err(error.clone());
        }
        self.storage
            .clear_failure()
            .map_err(ExternalReadError::Source)?;
        let validation = self.storage.source.validate().and_then(|()| {
            if self.storage.source.len() == self.storage.length {
                Ok(())
            } else {
                Err(SourceError::VersionChanged)
            }
        });
        if let Err(error) = validation {
            return self.fail_source(error);
        }
        // Keep candidate state local until the complete operation is validated.
        // Unwinding a source or caller callback therefore also discards it.
        let mut candidate = self.archive.take();
        let result = (|| {
            if candidate.is_none() {
                let key = match &self.key {
                    Some(key) => key.try_clone()?,
                    None => {
                        let header = self.storage.read_at(0, crate::constants::HEADER_LEN)?;
                        let header = crate::file_format::current_header::read_header(&header)?;
                        if !header.format_mode.plaintext() {
                            return Err(Error::InvalidInput(
                                "this external archive requires a content key".into(),
                            ));
                        }
                        SecretVec::try_from_slice(&[0; 32])?
                    }
                };
                let archive = Lockbox::open_storage_with_secret_key_mode(
                    StorageBackend::External(self.storage.clone()),
                    key,
                    self.options.lockbox,
                    false,
                )?;
                // Never invoke a caller's closure on an incompletely opened archive.
                if self.storage.failure().map_err(source_io)?.is_some() {
                    return Err(Error::Io("incomplete external open".into()));
                }
                candidate = Some(archive.into_state());
            }
            operation(
                candidate
                    .as_ref()
                    .ok_or_else(|| Error::InvalidOperation("external archive not open".into()))?,
            )
        })();
        if let Err(error) = self.storage.source.validate() {
            return self.fail_source(error);
        }
        if self.storage.source.len() != self.storage.length {
            return self.fail_source(SourceError::VersionChanged);
        }
        if let Some(error) = self.storage.failure().map_err(ExternalReadError::Source)? {
            return self.fail_source(error);
        }
        self.missing_attempts = 0;
        *self
            .storage
            .operation_bytes
            .lock()
            .map_err(|_| ExternalReadError::Source(poisoned()))? = 0;
        if result.is_ok() {
            self.archive = candidate;
        }
        result.map_err(ExternalReadError::Archive)
    }
    fn fail_source<T>(&mut self, error: SourceError) -> Result<T, ExternalReadError> {
        self.archive = None;
        let error = match error {
            SourceError::MissingRange { offset, length }
                if length == 0
                    || length > self.storage.max_read_bytes
                    || offset
                        .checked_add(length as u64)
                        .is_none_or(|end| end > self.storage.length) =>
            {
                SourceError::Unavailable(
                    "external source requested an invalid missing range".into(),
                )
            }
            other => other,
        };
        let failure = if matches!(error, SourceError::MissingRange { .. }) {
            self.missing_attempts += 1;
            if self.missing_attempts >= self.options.max_missing_attempts {
                ExternalReadError::RetryLimitExceeded
            } else {
                ExternalReadError::Source(error)
            }
        } else {
            ExternalReadError::Source(error)
        };
        if !matches!(
            failure,
            ExternalReadError::Source(SourceError::MissingRange { .. })
        ) {
            self.key = None;
            self.terminal = Some(failure.clone());
        }
        Err(failure)
    }
}

fn source_io(error: SourceError) -> Error {
    Error::Io(error.to_string())
}
fn poisoned() -> SourceError {
    SourceError::Unavailable("external source state poisoned".into())
}

#[derive(Clone)]
pub(crate) struct ExternalStorage {
    source: Arc<dyn ReadAtSource>,
    length: u64,
    failure: Arc<Mutex<Option<SourceError>>>,
    max_read_bytes: usize,
    max_operation_bytes: u64,
    operation_bytes: Arc<Mutex<u64>>,
}
impl fmt::Debug for ExternalStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExternalStorage")
            .field("length", &self.length)
            .finish_non_exhaustive()
    }
}
impl ExternalStorage {
    pub(crate) fn ensure_current(&self) -> crate::Result<()> {
        if let Some(error) = self.failure().map_err(source_io)? {
            return Err(source_io(error));
        }
        self.source.validate().map_err(|error| self.latch(error))?;
        if self.source.len() != self.length {
            return Err(self.latch(SourceError::VersionChanged));
        }
        Ok(())
    }

    fn failure(&self) -> Result<Option<SourceError>, SourceError> {
        Ok(self.failure.lock().map_err(|_| poisoned())?.clone())
    }
    fn clear_failure(&self) -> Result<(), SourceError> {
        *self.failure.lock().map_err(|_| poisoned())? = None;
        Ok(())
    }
    fn latch(&self, error: SourceError) -> Error {
        if let Ok(mut failure) = self.failure.lock() {
            if failure.is_none() {
                *failure = Some(error.clone());
            }
        }
        source_io(error)
    }
    fn bounds(&self, offset: u64, len: usize) -> crate::Result<()> {
        if len > self.max_read_bytes {
            return Err(self.latch(SourceError::Unavailable(
                "external read exceeds configured byte limit".into(),
            )));
        }
        if offset
            .checked_add(len as u64)
            .is_none_or(|end| end > self.length)
        {
            return Err(self.latch(SourceError::Unavailable(
                "external read exceeds source length".into(),
            )));
        }
        Ok(())
    }
}
impl Storage for ExternalStorage {
    fn len(&self) -> crate::Result<u64> {
        Ok(self.length)
    }
    fn read_at(&self, offset: u64, len: usize) -> crate::Result<Vec<u8>> {
        self.bounds(offset, len)?;
        let mut bytes = vec![0; len];
        self.read_at_into(offset, &mut bytes)?;
        Ok(bytes)
    }
    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> crate::Result<()> {
        if let Some(error) = self.failure().map_err(source_io)? {
            return Err(source_io(error));
        }
        self.bounds(offset, out.len())?;
        if out.is_empty() {
            return Ok(());
        }
        {
            let mut spent = self
                .operation_bytes
                .lock()
                .map_err(|_| self.latch(poisoned()))?;
            let next = spent
                .checked_add(out.len() as u64)
                .filter(|value| *value <= self.max_operation_bytes);
            let Some(next) = next else {
                return Err(self.latch(SourceError::Unavailable(
                    "external operation exceeds cumulative read budget".into(),
                )));
            };
            *spent = next;
        }
        let result = self.source.read_at(offset, out);
        match result {
            Ok(count) if count == out.len() => Ok(()),
            Ok(_) => Err(self.latch(SourceError::Unavailable(
                "external source returned a short or oversized read".into(),
            ))),
            Err(SourceError::MissingRange { offset, length })
                if length == 0
                    || length > self.max_read_bytes
                    || offset
                        .checked_add(length as u64)
                        .is_none_or(|end| end > self.length) =>
            {
                Err(self.latch(SourceError::Unavailable(
                    "external source requested an invalid missing range".into(),
                )))
            }
            Err(error) => Err(self.latch(error)),
        }
    }
    fn read_at_secure(
        &self,
        offset: u64,
        len: usize,
    ) -> crate::Result<crate::secret_vec::SecureVec> {
        self.bounds(offset, len)?;
        let mut out = crate::secret_vec::SecureVec::new();
        out.resize_zeroed(len)?;
        out.with_mut_bytes(|bytes| self.read_at_into(offset, bytes))??;
        Ok(out)
    }
    fn append(&mut self, _: &[u8]) -> crate::Result<u64> {
        Err(Error::InvalidOperation(
            "external archive is read-only".into(),
        ))
    }
    fn write_at(&mut self, _: u64, _: &[u8]) -> crate::Result<()> {
        Err(Error::InvalidOperation(
            "external archive is read-only".into(),
        ))
    }
    fn truncate(&mut self, _: u64) -> crate::Result<()> {
        Err(Error::InvalidOperation(
            "external archive is read-only".into(),
        ))
    }
    fn sync(&self) -> crate::Result<()> {
        Ok(())
    }
}

/// Shared, bounded LRU block cache for asynchronous range delivery. All supplied
/// blocks must have the fixed revision identifier and exact aligned length.
/// Eviction relies on the transport validating that revision again on refetch.
/// The budget covers stored payload bytes; metadata is bounded by block count.
#[derive(Clone)]
pub struct SparseSource {
    length: u64,
    revision: Arc<str>,
    block_bytes: usize,
    capacity_bytes: usize,
    state: Arc<Mutex<SparseState>>,
}
#[derive(Default)]
struct SparseState {
    blocks: BTreeMap<u64, (Vec<u8>, u64)>,
    bytes: usize,
    clock: u64,
    terminal: Option<SourceError>,
}
impl SparseState {
    fn tick(&mut self) -> u64 {
        if self.clock == u64::MAX {
            for (_, age) in self.blocks.values_mut() {
                *age = 0;
            }
            self.clock = 0;
        }
        self.clock += 1;
        self.clock
    }
}
impl SparseSource {
    /// Create an empty cache. Blocks must be at least 512 bytes; the budget must
    /// hold at least one block. No allocation proportional to archive size occurs.
    pub fn new(
        length: u64,
        revision: String,
        block_bytes: usize,
        capacity_bytes: usize,
    ) -> Result<Self, SourceError> {
        if length == 0 || revision.is_empty() || block_bytes < 512 || capacity_bytes < block_bytes {
            return Err(SourceError::Unavailable(
                "invalid sparse length, revision, block size or budget".into(),
            ));
        }
        Ok(Self {
            length,
            revision: revision.into(),
            block_bytes,
            capacity_bytes,
            state: Arc::new(Mutex::new(SparseState::default())),
        })
    }
    /// Insert one aligned complete block (the final block may be shorter).
    /// Conflicting bytes or revisions invalidate the source and discard its cache.
    pub fn supply(&self, revision: &str, offset: u64, bytes: Vec<u8>) -> Result<(), SourceError> {
        let mut state = self.state.lock().map_err(|_| poisoned())?;
        if let Some(error) = &state.terminal {
            return Err(error.clone());
        }
        if revision != self.revision.as_ref() {
            state.blocks.clear();
            state.bytes = 0;
            state.terminal = Some(SourceError::VersionChanged);
            return Err(SourceError::VersionChanged);
        }
        if offset >= self.length
            || offset % self.block_bytes as u64 != 0
            || bytes.len() != (self.length - offset).min(self.block_bytes as u64) as usize
        {
            return Err(SourceError::Unavailable(
                "expected one exact aligned sparse block".into(),
            ));
        }
        if let Some((previous, _)) = state.blocks.get(&offset) {
            if previous != &bytes {
                state.blocks.clear();
                state.bytes = 0;
                state.terminal = Some(SourceError::VersionChanged);
                return Err(SourceError::VersionChanged);
            }
            let age = state.tick();
            if let Some((_, previous_age)) = state.blocks.get_mut(&offset) {
                *previous_age = age;
            }
            return Ok(());
        }
        while state.bytes > self.capacity_bytes - bytes.len() {
            let oldest = state
                .blocks
                .iter()
                .min_by_key(|(_, (_, age))| *age)
                .map(|(&offset, _)| offset)
                .ok_or_else(|| {
                    SourceError::Unavailable("invalid sparse cache accounting".into())
                })?;
            if let Some((removed, _)) = state.blocks.remove(&oldest) {
                state.bytes -= removed.len();
            }
        }
        let age = state.tick();
        state.bytes += bytes.len();
        state.blocks.insert(offset, (bytes, age));
        Ok(())
    }
    /// Payload bytes currently retained, excluding decoded pages and in-flight I/O.
    pub fn cached_bytes(&self) -> Result<usize, SourceError> {
        Ok(self.state.lock().map_err(|_| poisoned())?.bytes)
    }
    /// Cancel every reader sharing this source and discard its cached blocks.
    pub fn cancel(&self) -> Result<(), SourceError> {
        let mut state = self.state.lock().map_err(|_| poisoned())?;
        state.blocks.clear();
        state.bytes = 0;
        state.terminal = Some(SourceError::Cancelled);
        Ok(())
    }
}
impl ReadAtSource for SparseSource {
    fn len(&self) -> u64 {
        self.length
    }
    fn validate(&self) -> Result<(), SourceError> {
        match &self.state.lock().map_err(|_| poisoned())?.terminal {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }
    fn read_at(&self, offset: u64, output: &mut [u8]) -> Result<usize, SourceError> {
        let end = offset
            .checked_add(output.len() as u64)
            .filter(|end| *end <= self.length)
            .ok_or_else(|| SourceError::Unavailable("sparse read outside source".into()))?;
        let mut state = self.state.lock().map_err(|_| poisoned())?;
        if let Some(error) = &state.terminal {
            return Err(error.clone());
        }
        let mut position = offset;
        while position < end {
            let start = position / self.block_bytes as u64 * self.block_bytes as u64;
            let age = state.tick();
            let (bytes, last_used) =
                state
                    .blocks
                    .get_mut(&start)
                    .ok_or(SourceError::MissingRange {
                        offset: start,
                        length: (self.length - start).min(self.block_bytes as u64) as usize,
                    })?;
            let finish = end.min(start + bytes.len() as u64);
            output[(position - offset) as usize..(finish - offset) as usize]
                .copy_from_slice(&bytes[(position - start) as usize..(finish - start) as usize]);
            *last_used = age;
            position = finish;
        }
        Ok(output.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_file_handle_retains_index_and_bounds_raw_reads_without_bypassing_failure_latch() {
        use crate::{
            Compression, Encryption, LockboxCreateOptions, LockboxProtection, OwnerSigningKeyPair,
            Signing, SizePadding,
        };
        use std::io::{Read, Seek, SeekFrom};
        struct Probe {
            bytes: Vec<u8>,
            cancelled: Mutex<bool>,
            reads: Mutex<Vec<(u64, usize)>>,
        }
        impl ReadAtSource for Probe {
            fn len(&self) -> u64 {
                self.bytes.len() as u64
            }
            fn validate(&self) -> Result<(), SourceError> {
                if *self.cancelled.lock().unwrap() {
                    Err(SourceError::Cancelled)
                } else {
                    Ok(())
                }
            }
            fn read_at(&self, offset: u64, out: &mut [u8]) -> Result<usize, SourceError> {
                self.reads.lock().unwrap().push((offset, out.len()));
                out.copy_from_slice(&self.bytes[offset as usize..offset as usize + out.len()]);
                Ok(out.len())
            }
        }
        let signer = OwnerSigningKeyPair::generate().unwrap();
        let input = (0..2 * 1024 * 1024 + 13)
            .map(|n| (n % 251) as u8)
            .collect::<Vec<_>>();
        for encrypted in [false, true] {
            for signed in [false, true] {
                for compression in [Compression::None, Compression::default()] {
                    for size_padding in [SizePadding::Default, SizePadding::None] {
                        let mut archive =
                            Lockbox::create_in_memory_with_options(LockboxCreateOptions {
                                compression,
                                size_padding,
                                ..LockboxCreateOptions::new(
                                    if encrypted {
                                        Encryption::Encrypted(LockboxProtection::ContentKey(
                                            SecretVec::try_from_slice(&[67; 32]).unwrap(),
                                        ))
                                    } else {
                                        Encryption::None
                                    },
                                    if signed {
                                        Signing::Owner(&signer)
                                    } else {
                                        Signing::None
                                    },
                                )
                            })
                            .unwrap();
                        archive.commit().unwrap();
                        // Unit-level injection: there is not yet a public writer for
                        // native block pages. Test the real public Read/Seek handle.
                        let paths =
                            crate::lockbox::block_frame_tests::install(&mut archive, &input);
                        let expected = &input[input.len() / 2..];
                        let source = Arc::new(Probe {
                            bytes: archive.to_bytes(),
                            cancelled: Mutex::new(false),
                            reads: Mutex::new(Vec::new()),
                        });
                        let session =
                            ExternalReader::new(source.clone(), ExternalReaderOptions::default())
                                .unwrap();
                        crate::lockbox::block_frame_tests::replace_storage(
                            &mut archive,
                            StorageBackend::External(session.storage.clone()),
                        );
                        let mut reader = archive.open_file(&paths[1]).unwrap();
                        reader.seek(SeekFrom::Start(7)).unwrap();
                        let mut small = [0; 13];
                        reader.read_exact(&mut small).unwrap();
                        assert_eq!(small, expected[7..20]);
                        let initial = source.reads.lock().unwrap().clone();
                        assert_eq!(initial.len(), 4, "header, metadata, index, data");
                        if compression == Compression::None {
                            assert_eq!(initial[3].1, 16384 + if encrypted { 16 } else { 0 });
                        }
                        // A repeated read within the window needs no metadata,
                        // index, or data fetch, in either codec mode.
                        reader.seek(SeekFrom::Start(7)).unwrap();
                        reader.read_exact(&mut small).unwrap();
                        assert_eq!(small, expected[7..20]);
                        assert_eq!(*source.reads.lock().unwrap(), initial);
                        reader.seek(SeekFrom::Start(32770)).unwrap();
                        reader.read_exact(&mut small).unwrap();
                        assert_eq!(small, expected[32770..32783]);
                        assert_eq!(
                            source.reads.lock().unwrap().len(),
                            initial.len() + usize::from(compression == Compression::None)
                        );
                        // Sequential consumption reuses the index and preserves bytes
                        // across windows and the unaligned packed-file boundary.
                        reader.rewind().unwrap();
                        let mut actual = Vec::new();
                        let mut buffer = [0; 65536];
                        loop {
                            let count = reader.read(&mut buffer).unwrap();
                            if count == 0 {
                                break;
                            }
                            actual.extend_from_slice(&buffer[..count]);
                        }
                        assert_eq!(actual, expected);
                        assert!(
                            source.reads.lock().unwrap()[4..]
                                .iter()
                                .all(|(offset, _)| *offset >= initial[3].0),
                            "sequential reads must not reload metadata or the index"
                        );
                        if compression != Compression::None {
                            assert_eq!(
                                source.reads.lock().unwrap().len(),
                                4,
                                "decode once per retained chunk"
                            );
                        }
                        reader.seek(SeekFrom::End(-13)).unwrap();
                        reader.read_exact(&mut small).unwrap();
                        let count = source.reads.lock().unwrap().len();
                        *source.cancelled.lock().unwrap() = true;
                        reader.seek(SeekFrom::End(-13)).unwrap();
                        assert!(reader.read_exact(&mut small).is_err());
                        *source.cancelled.lock().unwrap() = false;
                        assert!(
                            reader.read_exact(&mut small).is_err(),
                            "cache cannot clear terminal failure"
                        );
                        assert_eq!(source.reads.lock().unwrap().len(), count);
                    }
                }
            }
        }
    }

    #[test]
    fn native_block_reader_preserves_external_failure_latching_in_every_mode() {
        use crate::file_format::indexed_frame::{encode_block_frame, BlockFrameReader};
        use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
        struct Probe {
            bytes: Vec<u8>,
            state: Mutex<u8>,
            reads: Mutex<Vec<(u64, usize)>>,
        }
        impl ReadAtSource for Probe {
            fn len(&self) -> u64 {
                self.bytes.len() as u64 + u64::from(*self.state.lock().unwrap() == 5)
            }
            fn validate(&self) -> Result<(), SourceError> {
                match *self.state.lock().unwrap() {
                    1 => Err(SourceError::Cancelled),
                    2 => Err(SourceError::VersionChanged),
                    _ => Ok(()),
                }
            }
            fn read_at(&self, offset: u64, out: &mut [u8]) -> Result<usize, SourceError> {
                self.reads.lock().unwrap().push((offset, out.len()));
                let state = *self.state.lock().unwrap();
                if state == 4 {
                    return Err(SourceError::MissingRange {
                        offset,
                        length: out.len(),
                    });
                }
                out.copy_from_slice(&self.bytes[offset as usize..offset as usize + out.len()]);
                Ok(out.len() - usize::from(state == 3 && !out.is_empty()))
            }
        }
        let key = [29; 32];
        for encrypted in [false, true] {
            for signed in [false, true] {
                for compressed in [false, true] {
                    let mode = crate::creation_options::FormatMode::new(LockboxFormatOptions {
                        encryption: if encrypted {
                            EncryptionMode::ChaCha20Poly1305
                        } else {
                            EncryptionMode::None
                        },
                        signing: if signed {
                            SigningMode::Owner
                        } else {
                            SigningMode::None
                        },
                        compression: if compressed {
                            Compression::default()
                        } else {
                            Compression::None
                        },
                        size_padding: SizePadding::Default,
                    });
                    let (descriptor, packet) = encode_block_frame(
                        crate::LockboxId::from_bytes([71; 16]),
                        31,
                        mode,
                        &[43; 32768],
                        &key,
                    )
                    .unwrap();
                    for failure in 1..=5 {
                        let source = Arc::new(Probe {
                            bytes: packet.clone(),
                            state: Mutex::new(0),
                            reads: Mutex::new(Vec::new()),
                        });
                        let session =
                            ExternalReader::new(source.clone(), ExternalReaderOptions::default())
                                .unwrap();
                        let backend = StorageBackend::External(session.storage.clone());
                        let reader =
                            BlockFrameReader::open(&descriptor, &backend, 0, packet.len(), &key)
                                .unwrap();
                        assert_eq!(
                            source.reads.lock().unwrap().len(),
                            1,
                            "open must read only the committed index"
                        );
                        assert_eq!(reader.read(7..19).unwrap(), [43; 12]);
                        assert_eq!(source.reads.lock().unwrap().len(), 2);
                        if !compressed {
                            assert_eq!(
                                source.reads.lock().unwrap()[1].1,
                                16384 + if encrypted { 16 } else { 0 }
                            );
                        }
                        *source.state.lock().unwrap() = failure;
                        assert!(reader.read(7..19).is_err());
                        let latched = session.storage.failure().unwrap().unwrap();
                        match failure {
                            1 => assert_eq!(latched, SourceError::Cancelled),
                            2 | 5 => assert_eq!(latched, SourceError::VersionChanged),
                            3 => assert!(matches!(latched, SourceError::Unavailable(_))),
                            4 => assert!(matches!(latched, SourceError::MissingRange { .. })),
                            _ => unreachable!(),
                        }
                        *source.state.lock().unwrap() = 0;
                        let reads_before_retry = source.reads.lock().unwrap().len();
                        // Even empty reads after a cached index must reject the latch.
                        assert!(reader.read(0..0).is_err());
                        assert!(reader.read(7..19).is_err());
                        assert_eq!(source.reads.lock().unwrap().len(), reads_before_retry);
                        if failure == 4 {
                            // Only the checked external operation may reset a retryable
                            // MissingRange between attempts; the block reader cannot.
                            session.storage.clear_failure().unwrap();
                            assert_eq!(reader.read(7..19).unwrap(), [43; 12]);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn backend_refuses_every_mutation_and_checks_limits_before_secure_allocation() {
        let source = Arc::new(SparseSource::new(4096, "v1".into(), 512, 512).unwrap());
        let mut storage = ExternalStorage {
            source,
            length: 4096,
            max_read_bytes: 1024,
            max_operation_bytes: 8192,
            operation_bytes: Arc::new(Mutex::new(0)),
            failure: Arc::new(Mutex::new(None)),
        };
        assert!(storage.append(b"x").is_err());
        assert!(storage.write_at(0, b"x").is_err());
        assert!(storage.truncate(0).is_err());
        assert!(StorageBackend::External(storage.clone())
            .read_at_secure(0, usize::MAX)
            .is_err());
        assert!(matches!(
            storage.failure().unwrap(),
            Some(SourceError::Unavailable(_))
        ));
        assert!(StorageBackend::External(storage).is_read_only());
    }
}
