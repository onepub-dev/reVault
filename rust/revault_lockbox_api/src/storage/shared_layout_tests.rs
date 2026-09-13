//! Integration probe using the real storage backend, not a public CLI E2E test.
//! The experimental packet is not yet a native page/TOC representation.
#[path = "../../benches/shared_layout.rs"]
pub(crate) mod codec;

use super::{Storage, StorageBackend};
use codec::{ByteSource, Reader};
use std::cell::RefCell;
use std::ops::Range;
use zeroize::Zeroizing;

/// Borrow the actual backend: retain its file lock, revision checks, external
/// failure latch and byte budget. Never open a second independent descriptor.
pub(crate) struct StorageSource<'a> {
    storage: &'a StorageBackend,
    start: u64,
    length: usize,
    reads: RefCell<Vec<Range<usize>>>,
}

impl<'a> StorageSource<'a> {
    pub(crate) fn new(storage: &'a StorageBackend, start: u64, length: usize) -> Self {
        Self {
            storage,
            start,
            length,
            reads: RefCell::new(Vec::new()),
        }
    }
}

impl ByteSource for StorageSource<'_> {
    fn len(&self) -> Result<usize, &'static str> {
        self.storage
            .ensure_current()
            .map_err(|_| "source invalidated")?;
        let end = self
            .start
            .checked_add(self.length as u64)
            .ok_or("extent overflow")?;
        if end > self.storage.len().map_err(|_| "source length")? {
            return Err("truncated extent");
        }
        Ok(self.length)
    }

    fn read(&self, range: Range<usize>) -> Result<Vec<u8>, &'static str> {
        if range.start > range.end || range.end > self.len()? {
            return Err("source range");
        }
        let offset = self
            .start
            .checked_add(range.start as u64)
            .ok_or("offset overflow")?;
        // read_at_into permits wiping partial successful transport bytes on error.
        let mut out = Zeroizing::new(vec![0; range.len()]);
        self.storage
            .read_at_into(offset, &mut out)
            .map_err(|_| "source read")?;
        self.storage
            .ensure_current()
            .map_err(|_| "source invalidated")?;
        self.reads.borrow_mut().push(range);
        Ok(std::mem::take(&mut *out))
    }
}

#[test]
fn common_reader_uses_real_memory_and_locked_file_storage_in_all_modes() {
    let signer = ed25519_dalek::SigningKey::from_bytes(&[19; 32]);
    let key = [29; 32];
    let content: Vec<_> = (0..65539).map(|n| (n % 251) as u8).collect();
    let root = fixture_directory();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                let (descriptor, packet) = codec::encode(
                    &content,
                    16384,
                    compressed,
                    encrypted.then_some(&key),
                    signed.then_some(&signer),
                )
                .unwrap();
                let mut bytes = vec![71; 317];
                bytes.extend_from_slice(&packet);
                bytes.extend_from_slice(&[79; 211]);
                let path = root.join(format!("{encrypted}-{signed}-{compressed}"));
                let file = StorageBackend::create_file(&path, &bytes).unwrap();
                for storage in [StorageBackend::memory(bytes.clone()), file] {
                    let source = StorageSource::new(&storage, 317, packet.len());
                    let verifier = signer.verifying_key();
                    let reader = Reader::open(
                        &descriptor,
                        &source,
                        encrypted.then_some(&key),
                        signed.then_some(&verifier),
                    )
                    .unwrap();
                    let open_reads = source.reads.borrow().len();
                    assert_eq!(open_reads, if signed { 2 } else { 1 });
                    assert_eq!(reader.read(17000..17013).unwrap(), content[17000..17013]);
                    if !compressed {
                        assert_eq!(
                            source.reads.borrow().last().unwrap().len(),
                            16384 + if encrypted { 16 } else { 0 }
                        );
                    }
                    assert_eq!(reader.read(0..content.len()).unwrap(), content);
                    assert_eq!(reader.read(content.len()..content.len()).unwrap(), b"");
                    assert!(source
                        .reads
                        .borrow()
                        .iter()
                        .all(|range| range.end <= packet.len()));
                    assert_eq!(storage.read_at(0, 317).unwrap(), vec![71; 317]);
                    assert_eq!(
                        storage.read_at(317 + packet.len() as u64, 211).unwrap(),
                        vec![79; 211]
                    );
                    if storage.path().is_some() {
                        // Same-inode damage does not change the file identity. The
                        // block commitment/AEAD must reject it after index caching.
                        let mut damaged = storage.clone();
                        damaged
                            .write_at(
                                317 + packet.len() as u64 - 1,
                                &[packet[packet.len() - 1] ^ 1],
                            )
                            .unwrap();
                        assert!(reader.read(0..content.len()).is_err());
                    }
                }
            }
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn real_storage_rejects_invalid_extents_and_truncation_after_index_load() {
    let root = fixture_directory();
    let path = root.join("truncated");
    let (descriptor, packet) = codec::encode(&[43; 32768], 16384, false, None, None).unwrap();
    let storage = StorageBackend::create_file(&path, &packet).unwrap();
    let source = StorageSource::new(&storage, 0, packet.len());
    let reader = Reader::open(&descriptor, &source, None, None).unwrap();
    assert!(StorageSource::new(&storage, u64::MAX, 2).len().is_err());
    assert!(source.read(0..packet.len() + 1).is_err());
    assert!(source.read(Range { start: 2, end: 1 }).is_err());
    // Out-of-band damage is intentional here; no public CLI produces truncation.
    std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(1)
        .unwrap();
    assert!(reader.read(0..1).is_err());
    assert!(reader.read(0..0).is_err());
    drop(storage);
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn borrowed_storage_retains_lock_and_rejects_replaced_file() {
    let root = fixture_directory();
    let path = root.join("held");
    let (descriptor, packet) = codec::encode(&[43; 32768], 16384, false, None, None).unwrap();
    let storage = StorageBackend::create_file(&path, &packet).unwrap();
    let competing = std::fs::File::open(&path).unwrap();
    let expired = std::time::Instant::now() - super::file_lock::lock_timeout();
    let source = StorageSource::new(&storage, 0, packet.len());
    let reader = Reader::open(&descriptor, &source, None, None).unwrap();
    assert!(super::archive_lock::acquire(&competing, &path, true, expired).is_err());
    let replacement = root.join("replacement");
    let replacement_store = StorageBackend::create_file(&replacement, &packet).unwrap();
    // Deliberate bypass of cooperative locking to model hostile replacement.
    std::fs::rename(&replacement, &path).unwrap();
    assert!(reader.read(0..1).is_err());
    assert!(reader.read(0..0).is_err());
    drop(storage);
    super::archive_lock::acquire(&competing, &path, true, expired).unwrap();
    drop(competing);
    drop(replacement_store);
    std::fs::remove_dir_all(root).unwrap();
}

fn fixture_directory() -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "revault-real-shared-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    root
}
