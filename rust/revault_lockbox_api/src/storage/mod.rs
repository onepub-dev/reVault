pub(crate) mod archive_lock;
pub(crate) mod atomic_file_replacement;
pub(crate) mod cache_options;
pub(crate) mod file_lock;
pub(crate) mod free_index;
pub(crate) mod free_slot;
pub(crate) mod page_cache;
#[cfg(test)]
pub(crate) mod shared_layout_tests;

use crate::secret_vec::SecureVec;
use crate::{Error, Result};
#[cfg(test)]
use std::cell::Cell;
use std::fs;
#[cfg(not(unix))]
use std::io::Read;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub(crate) trait Storage: Clone + std::fmt::Debug {
    fn len(&self) -> Result<u64>;
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>>;
    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()>;
    fn append(&mut self, bytes: &[u8]) -> Result<u64>;
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()>;
    fn truncate(&mut self, len: u64) -> Result<()>;
    fn sync(&self) -> Result<()>;

    fn read_at_secure(&self, offset: u64, len: usize) -> Result<SecureVec> {
        let mut out = SecureVec::new();
        out.resize_zeroed(len)?;
        out.with_mut_bytes(|bytes| self.read_at_into(offset, bytes))??;
        Ok(out)
    }

    fn read_all(&self) -> Result<Vec<u8>> {
        let len = self.len()?;
        if len > usize::MAX as u64 {
            return Err(Error::SecurityLimitExceeded(
                "vault is too large to materialize in memory".to_string(),
            ));
        }
        self.read_at(0, len as usize)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum StorageBackend {
    Memory(MemoryStore),
    File(FileStore),
    #[cfg(feature = "external-source")]
    External(crate::external_source::ExternalStorage),
}

impl StorageBackend {
    pub(crate) fn ensure_current(&self) -> Result<()> {
        #[cfg(feature = "external-source")]
        if let Self::External(store) = self {
            return store.ensure_current();
        }
        if let Self::File(store) = self {
            let file = store.lock_file()?;
            store.ensure_current(&file)?;
        }
        Ok(())
    }

    pub(crate) fn memory(bytes: Vec<u8>) -> Self {
        Self::Memory(MemoryStore::new(bytes))
    }

    pub(crate) fn file(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self::File(FileStore::open(path, false)?))
    }

    pub(crate) fn file_for_write(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self::File(FileStore::open(path, true)?))
    }

    pub(crate) fn file_for_recovery(path: impl AsRef<Path>) -> Result<Self> {
        Self::file_for_write(path).map_err(|err| match err {
            Error::Io(message) | Error::LockUnavailable(message) => Error::RecoveryBlocked(message),
            other => other,
        })
    }

    pub(crate) fn create_file(path: impl AsRef<Path>, initial_bytes: &[u8]) -> Result<Self> {
        Ok(Self::File(FileStore::create(path, initial_bytes)?))
    }

    #[cfg(any(test, feature = "migration"))]
    pub(crate) fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
        if !path.exists() {
            Self::create_file(path, bytes)?;
            return Ok(());
        }
        let _original = Self::file_for_write(path)?;
        let (replacement, mut file) =
            atomic_file_replacement::AtomicFileReplacement::create_unique(path, ".lockbox-write")?;
        let result = (|| {
            archive_lock::acquire(
                &file,
                replacement.temp_path(),
                true,
                std::time::Instant::now(),
            )?;
            file.write_all(bytes)
                .map_err(|err| Error::Io(err.to_string()))?;
            file.sync_all().map_err(|err| Error::Io(err.to_string()))?;
            replacement.install()
        })();
        replacement.discard();
        result
    }

    pub(crate) fn is_read_only(&self) -> bool {
        #[cfg(feature = "external-source")]
        if matches!(self, Self::External(_)) {
            return true;
        }
        matches!(self, Self::File(store) if !store.writable)
    }

    #[cfg(feature = "bindings")]
    pub(crate) fn publish(path: &Path, bytes: &[u8], overwrite: bool) -> Result<Self> {
        if !overwrite
            || !path
                .try_exists()
                .map_err(|err| Error::Io(err.to_string()))?
        {
            return Self::create_file(path, bytes);
        }
        let _original = Self::file_for_write(path)?;
        let (replacement, mut file) =
            atomic_file_replacement::AtomicFileReplacement::create_unique(
                path,
                ".lockbox-replace",
            )?;
        let result = (|| {
            archive_lock::acquire(
                &file,
                replacement.temp_path(),
                true,
                std::time::Instant::now(),
            )?;
            file.write_all(bytes)
                .map_err(|err| Error::Io(err.to_string()))?;
            file.sync_all().map_err(|err| Error::Io(err.to_string()))?;
            replacement.install()?;
            Ok(Self::File(FileStore {
                path: path.to_path_buf(),
                file: Arc::new(RwLock::new(file)),
                writable: true,
            }))
        })();
        replacement.discard();
        result
    }

    pub(crate) fn relocate(&mut self, path: &Path) {
        if let Self::File(store) = self {
            store.path = path.to_path_buf();
        }
    }

    pub(crate) fn path(&self) -> Option<&Path> {
        match self {
            Self::Memory(_) => None,
            #[cfg(feature = "external-source")]
            Self::External(_) => None,
            Self::File(store) => Some(store.path()),
        }
    }
}

impl Storage for StorageBackend {
    fn read_at_secure(&self, offset: u64, len: usize) -> Result<SecureVec> {
        #[cfg(feature = "external-source")]
        if let Self::External(store) = self {
            return store.read_at_secure(offset, len);
        }
        let mut out = SecureVec::new();
        out.resize_zeroed(len)?;
        out.with_mut_bytes(|bytes| self.read_at_into(offset, bytes))??;
        Ok(out)
    }

    fn len(&self) -> Result<u64> {
        match self {
            Self::Memory(store) => store.len(),
            Self::File(store) => store.len(),
            #[cfg(feature = "external-source")]
            Self::External(store) => store.len(),
        }
    }

    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        match self {
            Self::Memory(store) => store.read_at(offset, len),
            Self::File(store) => store.read_at(offset, len),
            #[cfg(feature = "external-source")]
            Self::External(store) => store.read_at(offset, len),
        }
    }

    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        match self {
            Self::Memory(store) => store.read_at_into(offset, out),
            Self::File(store) => store.read_at_into(offset, out),
            #[cfg(feature = "external-source")]
            Self::External(store) => store.read_at_into(offset, out),
        }
    }

    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        match self {
            Self::Memory(store) => store.append(bytes),
            Self::File(store) => store.append(bytes),
            #[cfg(feature = "external-source")]
            Self::External(store) => store.append(bytes),
        }
    }

    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        match self {
            Self::Memory(store) => store.write_at(offset, bytes),
            Self::File(store) => store.write_at(offset, bytes),
            #[cfg(feature = "external-source")]
            Self::External(store) => store.write_at(offset, bytes),
        }
    }

    fn truncate(&mut self, len: u64) -> Result<()> {
        match self {
            StorageBackend::Memory(store) => store.truncate(len),
            StorageBackend::File(store) => store.truncate(len),
            #[cfg(feature = "external-source")]
            StorageBackend::External(store) => store.truncate(len),
        }
    }

    fn sync(&self) -> Result<()> {
        match self {
            Self::Memory(store) => store.sync(),
            Self::File(store) => store.sync(),
            #[cfg(feature = "external-source")]
            Self::External(store) => store.sync(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MemoryStore {
    bytes: Vec<u8>,
    #[cfg(test)]
    fail_append_after_successes: Option<usize>,
    #[cfg(any(test, feature = "test-support"))]
    fail_next_write_at: Option<u64>,
    #[cfg(test)]
    fail_sync_after_successes: Cell<Option<usize>>,
    #[cfg(test)]
    fail_operation_after_successes: Cell<Option<usize>>,
    #[cfg(test)]
    operation_count: Cell<usize>,
}

impl MemoryStore {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            #[cfg(test)]
            fail_append_after_successes: None,
            #[cfg(any(test, feature = "test-support"))]
            fail_next_write_at: None,
            #[cfg(test)]
            fail_sync_after_successes: Cell::new(None),
            #[cfg(test)]
            fail_operation_after_successes: Cell::new(None),
            #[cfg(test)]
            operation_count: Cell::new(0),
        }
    }

    #[cfg(test)]
    fn fail_append_after_successes(&mut self, successes: usize) {
        self.fail_append_after_successes = Some(successes);
    }

    #[cfg(any(test, feature = "test-support"))]
    fn fail_next_write_at(&mut self, offset: u64) {
        self.fail_next_write_at = Some(offset);
    }

    #[cfg(test)]
    fn fail_sync_after_successes(&mut self, successes: usize) {
        self.fail_sync_after_successes.set(Some(successes));
    }

    #[cfg(test)]
    fn fail_operation_after_successes(&mut self, successes: usize) {
        self.operation_count.set(0);
        self.fail_operation_after_successes.set(Some(successes));
    }

    #[cfg(test)]
    fn operation_count(&self) -> usize {
        self.operation_count.get()
    }

    #[cfg(test)]
    fn reset_operation_count(&self) {
        self.operation_count.set(0);
        self.fail_operation_after_successes.set(None);
    }

    #[cfg(test)]
    fn should_fail_operation(&self) -> bool {
        self.operation_count.set(self.operation_count.get() + 1);
        let Some(remaining) = self.fail_operation_after_successes.get() else {
            return false;
        };
        if remaining == 0 {
            self.fail_operation_after_successes.set(None);
            true
        } else {
            self.fail_operation_after_successes.set(Some(remaining - 1));
            false
        }
    }

    #[cfg(test)]
    fn should_fail_append(&mut self) -> bool {
        let Some(remaining) = self.fail_append_after_successes.as_mut() else {
            return false;
        };
        if *remaining == 0 {
            self.fail_append_after_successes = None;
            true
        } else {
            *remaining -= 1;
            false
        }
    }

    #[cfg(any(test, feature = "test-support"))]
    fn should_fail_write_at(&mut self, offset: u64) -> bool {
        if self.fail_next_write_at == Some(offset) {
            self.fail_next_write_at = None;
            true
        } else {
            false
        }
    }

    #[cfg(test)]
    fn should_fail_sync(&self) -> bool {
        let Some(remaining) = self.fail_sync_after_successes.get() else {
            return false;
        };
        if remaining == 0 {
            self.fail_sync_after_successes.set(None);
            true
        } else {
            self.fail_sync_after_successes.set(Some(remaining - 1));
            false
        }
    }
}

impl Storage for MemoryStore {
    fn len(&self) -> Result<u64> {
        Ok(self.bytes.len() as u64)
    }

    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        let start = offset as usize;
        let end = start
            .checked_add(len)
            .ok_or_else(|| Error::Io("storage read offset overflow".to_string()))?;
        if end > self.bytes.len() {
            return Err(Error::Truncated);
        }
        Ok(self.bytes[start..end].to_vec())
    }

    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        let start = offset as usize;
        let end = start
            .checked_add(out.len())
            .ok_or_else(|| Error::Io("storage read offset overflow".to_string()))?;
        if end > self.bytes.len() {
            return Err(Error::Truncated);
        }
        out.copy_from_slice(&self.bytes[start..end]);
        Ok(())
    }

    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        #[cfg(test)]
        if self.should_fail_operation() {
            return Err(Error::Io("injected storage operation failure".to_string()));
        }
        #[cfg(test)]
        if self.should_fail_append() {
            return Err(Error::Io("injected storage append failure".to_string()));
        }
        let offset = self.bytes.len() as u64;
        self.bytes.extend_from_slice(bytes);
        Ok(offset)
    }

    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        #[cfg(test)]
        if self.should_fail_operation() {
            return Err(Error::Io("injected storage operation failure".to_string()));
        }
        #[cfg(any(test, feature = "test-support"))]
        if self.should_fail_write_at(offset) {
            return Err(Error::Io("injected storage write failure".to_string()));
        }
        let start = offset as usize;
        let end = start
            .checked_add(bytes.len())
            .ok_or_else(|| Error::Io("storage write offset overflow".to_string()))?;
        if end > self.bytes.len() {
            return Err(Error::Io("storage write beyond end".to_string()));
        }
        self.bytes[start..end].copy_from_slice(bytes);
        Ok(())
    }

    fn truncate(&mut self, len: u64) -> Result<()> {
        #[cfg(test)]
        if self.should_fail_operation() {
            return Err(Error::Io("injected storage operation failure".into()));
        }
        let len = usize::try_from(len)
            .map_err(|_| Error::SecurityLimitExceeded("storage is too large".to_string()))?;
        if len > self.bytes.len() {
            return Err(Error::InvalidOperation(
                "storage cannot be extended by truncate".to_string(),
            ));
        }
        self.bytes.truncate(len);
        Ok(())
    }

    fn sync(&self) -> Result<()> {
        #[cfg(test)]
        if self.should_fail_operation() {
            return Err(Error::Io("injected storage operation failure".to_string()));
        }
        #[cfg(test)]
        if self.should_fail_sync() {
            return Err(Error::Io("injected storage sync failure".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
impl StorageBackend {
    pub(crate) fn fail_memory_operation_after_successes(&mut self, successes: usize) {
        match self {
            Self::Memory(store) => store.fail_operation_after_successes(successes),
            _ => panic!("failure injection is only available for memory storage"),
        }
    }

    pub(crate) fn memory_operation_count(&self) -> usize {
        match self {
            Self::Memory(store) => store.operation_count(),
            _ => panic!("operation counting is only available for memory storage"),
        }
    }

    pub(crate) fn reset_memory_operation_count(&self) {
        match self {
            Self::Memory(store) => store.reset_operation_count(),
            _ => panic!("operation counting is only available for memory storage"),
        }
    }

    pub(crate) fn fail_memory_append_after_successes(&mut self, successes: usize) {
        match self {
            Self::Memory(store) => store.fail_append_after_successes(successes),
            _ => panic!("failure injection is only available for memory storage"),
        }
    }

    pub(crate) fn fail_memory_next_write_at(&mut self, offset: u64) {
        match self {
            Self::Memory(store) => store.fail_next_write_at(offset),
            _ => panic!("failure injection is only available for memory storage"),
        }
    }

    pub(crate) fn fail_memory_sync_after_successes(&mut self, successes: usize) {
        match self {
            Self::Memory(store) => store.fail_sync_after_successes(successes),
            _ => panic!("failure injection is only available for memory storage"),
        }
    }
}

#[cfg(feature = "test-support")]
impl StorageBackend {
    pub(crate) fn inject_test_write_failure_at(&mut self, offset: u64) {
        match self {
            Self::Memory(store) => store.fail_next_write_at(offset),
            _ => panic!("test write failure is only available for memory storage"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FileStore {
    path: PathBuf,
    // A single descriptor retains the archive lock across clones. Positional
    // readers may share it, but mutations must still exclude all reads: a
    // read_exact_at call can involve multiple kernel reads on a short read.
    file: Arc<RwLock<std::fs::File>>,
    writable: bool,
}

impl FileStore {
    fn open(path: impl AsRef<Path>, writable: bool) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = archive_lock::open(&path, writable)?;
        Ok(Self {
            path,
            file: Arc::new(RwLock::new(file)),
            writable,
        })
    }

    fn create(path: impl AsRef<Path>, initial_bytes: &[u8]) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| Error::Io(err.to_string()))?;
        }
        // Initialize and lock privately before publishing the inode. hard_link
        // atomically publishes without replacing an existing destination.
        let (replacement, mut file) =
            atomic_file_replacement::AtomicFileReplacement::create_unique(
                &path,
                ".lockbox-create",
            )?;
        let result = (|| {
            archive_lock::acquire(&file, &path, true, std::time::Instant::now())?;
            file.write_all(initial_bytes)
                .map_err(|err| Error::Io(err.to_string()))?;
            file.sync_all().map_err(|err| Error::Io(err.to_string()))?;
            fs::hard_link(replacement.temp_path(), &path).map_err(|err| {
                if err.kind() == std::io::ErrorKind::AlreadyExists {
                    Error::AlreadyExists(path.display().to_string())
                } else {
                    Error::Io(format!("publish {}: {err}", path.display()))
                }
            })?;
            replacement.sync_parent()?;
            Ok(Self {
                path,
                file: Arc::new(RwLock::new(file)),
                writable: true,
            })
        })();
        replacement.discard();
        result
    }

    fn lock_file(&self) -> Result<std::sync::RwLockWriteGuard<'_, std::fs::File>> {
        self.file
            .write()
            .map_err(|_| Error::Io("storage file lock poisoned".to_string()))
    }

    fn ensure_current(&self, file: &std::fs::File) -> Result<()> {
        if archive_lock::is_current(file, &self.path)? {
            Ok(())
        } else {
            Err(Error::LockUnavailable(format!(
                "archive was replaced; reopen {}",
                self.path.display()
            )))
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Storage for FileStore {
    fn len(&self) -> Result<u64> {
        Ok(self
            .lock_file()?
            .metadata()
            .map_err(|err| Error::Io(format!("metadata {}: {err}", self.path.display())))?
            .len())
    }

    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        let mut out = vec![0; len];
        self.read_at_into(offset, &mut out)?;
        Ok(out)
    }

    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        #[cfg(unix)]
        let result = {
            use std::os::unix::fs::FileExt;
            let file = self
                .file
                .read()
                .map_err(|_| Error::Io("storage file lock poisoned".to_string()))?;
            file.read_exact_at(out, offset)
        };
        // Retain serialized cursor I/O where we do not have a cursor-independent
        // exact read. In particular, Windows seek_read changes the file cursor.
        #[cfg(not(unix))]
        let result = {
            let mut file = self.lock_file()?;
            file.seek(SeekFrom::Start(offset))
                .map_err(|err| Error::Io(format!("seek {}: {err}", self.path.display())))?;
            file.read_exact(out)
        };
        result.map_err(|err| {
            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                Error::Truncated
            } else {
                Error::Io(format!("read {}: {err}", self.path.display()))
            }
        })
    }

    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        let mut file = self.lock_file()?;
        self.ensure_current(&file)?;
        let offset = file
            .seek(SeekFrom::End(0))
            .map_err(|err| Error::Io(format!("seek {}: {err}", self.path.display())))?;
        file.write_all(bytes)
            .map_err(|err| Error::Io(format!("append {}: {err}", self.path.display())))?;
        Ok(offset)
    }

    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        let mut file = self.lock_file()?;
        self.ensure_current(&file)?;
        file.seek(SeekFrom::Start(offset))
            .map_err(|err| Error::Io(format!("seek {}: {err}", self.path.display())))?;
        file.write_all(bytes)
            .map_err(|err| Error::Io(format!("write {}: {err}", self.path.display())))
    }

    fn truncate(&mut self, len: u64) -> Result<()> {
        let file = self.lock_file()?;
        self.ensure_current(&file)?;
        file.set_len(len)
            .map_err(|err| Error::Io(format!("truncate {}: {err}", self.path.display())))
    }

    fn sync(&self) -> Result<()> {
        self.lock_file()?
            .sync_data()
            .map_err(|err| Error::Io(format!("sync {}: {err}", self.path.display())))
    }
}

#[cfg(all(test, unix))]
mod archive_lock_tests {
    use super::*;
    use std::time::Instant;

    fn path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "revault-{label}-{}",
            crate::LockboxId::new_random().unwrap()
        ))
    }

    #[test]
    fn positional_reads_share_the_handle_without_moving_its_cursor() {
        let path = path("positional-reads");
        let bytes: Vec<u8> = (0..131_072).map(|i| (i % 251) as u8).collect();
        let store = FileStore::create(&path, &bytes).unwrap();
        store
            .lock_file()
            .unwrap()
            .seek(SeekFrom::Start(37))
            .unwrap();
        let read_guard = store.file.read().unwrap();
        let clone = store.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            sender.send(clone.read_at(12_345, 65_537)).unwrap();
        });
        // A held read guard must not serialize another positional reader. Drop
        // it before joining even on timeout so a regression cannot hang tests.
        let result = receiver.recv_timeout(std::time::Duration::from_secs(5));
        drop(read_guard);
        worker.join().unwrap();
        assert_eq!(result.unwrap().unwrap(), bytes[12_345..77_882]);
        assert_eq!(store.lock_file().unwrap().stream_position().unwrap(), 37);
        assert!(matches!(
            store.read_at(bytes.len() as u64 - 1, 2),
            Err(Error::Truncated)
        ));
        assert!(store.read_at(bytes.len() as u64, 0).unwrap().is_empty());
        drop(store);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn cloned_positional_readers_observe_complete_writes_and_serialized_appends() {
        let path = path("positional-mutations");
        let size = 131_072;
        let store = FileStore::create(&path, &vec![0; size]).unwrap();
        std::thread::scope(|scope| {
            for id in 1..=4u8 {
                let mut clone = store.clone();
                scope.spawn(move || {
                    for _ in 0..32 {
                        clone.write_at(0, &vec![id; size]).unwrap();
                        let bytes = clone.read_at(0, size).unwrap();
                        assert!(bytes.iter().all(|byte| *byte == bytes[0]));
                        let offset = clone.append(&[id; 64]).unwrap();
                        assert!(offset >= size as u64);
                        assert_eq!(clone.read_at(offset, 64).unwrap(), [id; 64]);
                    }
                });
            }
        });
        assert_eq!(store.len().unwrap(), (size + 4 * 32 * 64) as u64);
        let mut clone = store.clone();
        clone.truncate(1).unwrap();
        assert!(matches!(store.read_at(0, 2), Err(Error::Truncated)));
        drop(clone);
        drop(store);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn cloned_storage_retains_lock_and_replaced_writer_is_rejected() {
        let path = path("clone-lock");
        let store = StorageBackend::create_file(&path, b"original").unwrap();
        let mut clone = store.clone();
        drop(store);
        let other = std::fs::File::open(&path).unwrap();
        let deadline = Instant::now() - file_lock::lock_timeout();
        assert!(matches!(
            archive_lock::acquire(&other, &path, false, deadline),
            Err(Error::LockUnavailable(_))
        ));
        let replacement_path = path.with_extension("replacement");
        let replacement = StorageBackend::create_file(&replacement_path, b"replacement").unwrap();
        // Simulate publication by a rewrite owning the original shared handle.
        fs::rename(&replacement_path, &path).unwrap();
        assert!(matches!(
            clone.write_at(0, b"bad"),
            Err(Error::LockUnavailable(_))
        ));
        assert_eq!(fs::read(&path).unwrap(), b"replacement");
        assert_eq!(
            clone.len().unwrap(),
            8,
            "length follows the original handle"
        );
        drop(clone);
        drop(replacement);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn nested_vault_guard_retains_lock_after_outer_guard_drops() {
        use super::file_lock::{FileLockScope, ScopedFileLock};
        let path = path("nested-vault-lock");
        let outer = ScopedFileLock::acquire(&path, FileLockScope::Vault).unwrap();
        let inner = ScopedFileLock::acquire(&path, FileLockScope::Vault).unwrap();
        drop(outer);
        let sidecar = file_lock::lock_path_for(&path);
        let other = std::fs::File::open(&sidecar).unwrap();
        let deadline = Instant::now() - file_lock::lock_timeout();
        assert!(matches!(
            archive_lock::acquire(&other, &sidecar, false, deadline),
            Err(Error::LockUnavailable(_))
        ));
        drop(inner);
        archive_lock::acquire(&other, &sidecar, false, Instant::now()).unwrap();
        drop(other);
        fs::remove_file(sidecar).unwrap();
    }
}
