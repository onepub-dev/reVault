use super::*;
mod tree_tests;
use crate::storage::StorageBackend;
use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
use std::io::Cursor;
const KEY: &[u8; 32] = &[79; 32];
fn archive() -> LockboxId {
    LockboxId::from_bytes([18; 16])
}
fn mode(encrypted: bool, signed: bool, compressed: bool, padded: bool) -> FormatMode {
    FormatMode::new(LockboxFormatOptions {
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
        size_padding: if padded {
            SizePadding::Default
        } else {
            SizePadding::None
        },
    })
}
fn key(mode: FormatMode) -> Option<&'static [u8]> {
    (!mode.plaintext()).then_some(KEY.as_slice())
}
fn authority<'a>(mode: FormatMode, public: &'a crate::OwnerSigningPublicKey) -> Authority<'a> {
    if mode.signed() {
        Authority::Owner(public)
    } else if mode.plaintext() {
        Authority::Checksum
    } else {
        Authority::Symmetric(KEY)
    }
}
fn pattern(offset: u64) -> u8 {
    ((offset / 101) % 29) as u8
}
struct Pattern {
    position: u64,
    len: u64,
    fail: Option<u64>,
}
impl Read for Pattern {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if self.fail.is_some_and(|at| self.position >= at) {
            return Err(std::io::Error::other("synthetic source failure"));
        }
        let n = out.len().min(8191).min((self.len - self.position) as usize);
        for (i, byte) in out[..n].iter_mut().enumerate() {
            *byte = pattern(self.position + i as u64);
        }
        self.position += n as u64;
        Ok(n)
    }
}
#[test]
fn streamed_files_and_ranges_round_trip_all_modes_and_units() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                for padded in [false, true] {
                    for unit in [65536, MAX_LOGICAL] {
                        let mode = mode(encrypted, signed, compressed, padded);
                        let authority = authority(mode, &public);
                        let large: Vec<_> = (0..300_071).map(pattern).collect();
                        let inputs = vec![
                            Input {
                                path: b"empty".to_vec(),
                                reader: Cursor::new(Vec::new()),
                            },
                            Input {
                                path: b"small".to_vec(),
                                reader: Cursor::new(b"small file bytes".to_vec()),
                            },
                            Input {
                                path: b"large".to_vec(),
                                reader: Cursor::new(large.clone()),
                            },
                        ];
                        let storage = Files::create(
                            StorageBackend::memory(Vec::new()),
                            archive(),
                            mode,
                            &authority,
                            signed.then_some(&owner),
                            key(mode),
                            unit,
                            inputs,
                        )
                        .unwrap();
                        let mut files = Files::open(
                            StorageBackend::memory(storage.read_all().unwrap()),
                            archive(),
                            mode,
                            &authority,
                            key(mode),
                        )
                        .unwrap();
                        assert_eq!(files.info(b"empty").unwrap().unwrap().len, 0);
                        assert!(files.info(b"absent").unwrap().is_none());
                        files
                            .read_range(b"empty", 0, 0, |_| panic!("empty range callback"))
                            .unwrap();
                        for (offset, len) in [
                            (0, large.len() as u64),
                            (0, 4096),
                            (65000, 4096),
                            (260000, 8192),
                            (300000, 71),
                            (300071, 0),
                        ] {
                            let mut position = offset;
                            files
                                .read_range(b"large", offset, len, |bytes| {
                                    assert!(bytes.len() <= unit);
                                    assert_eq!(
                                        bytes,
                                        &large[position as usize..position as usize + bytes.len()]
                                    );
                                    position += bytes.len() as u64;
                                    Ok(())
                                })
                                .unwrap();
                            assert_eq!(position, offset + len);
                        }
                        if !compressed {
                            for offset in (0..large.len() - 4096).step_by(4096) {
                                let mut calls = 0;
                                files
                                    .read_range(b"large", offset as u64, 4096, |bytes| {
                                        calls += 1;
                                        assert_eq!(bytes, &large[offset..offset + 4096]);
                                        Ok(())
                                    })
                                    .unwrap();
                                assert_eq!(
                                    calls, 1,
                                    "aligned raw range crossed allocation boundary"
                                );
                            }
                        }
                        assert!(files.read_range(b"large", u64::MAX, 1, |_| Ok(())).is_err());
                        assert!(files.read_range(b"large", 300071, 1, |_| Ok(())).is_err());
                        assert!(files.read_range(b"absent", 0, 0, |_| Ok(())).is_err());
                        assert!(files
                            .read_range(b"large", 0, 1, |_| Err(Error::Io(
                                "visitor failure".into()
                            )))
                            .is_err());
                        let mut small = Vec::new();
                        files
                            .read_range(b"small", 0, 16, |bytes| {
                                small.extend_from_slice(bytes);
                                Ok(())
                            })
                            .unwrap();
                        assert_eq!(small, b"small file bytes");
                        let storage = files.into_storage();
                        assert_eq!(
                            storage.len().unwrap(),
                            publication::select(&storage, archive(), mode, &authority)
                                .unwrap()
                                .anchor
                                .sealed_len
                        );
                    }
                }
            }
        }
    }
}

// Count physical reads across a complete multi-page extent traversal. This
// candidate has no public CLI path; the counter does not change stored bytes.
#[derive(Clone, Debug)]
struct ReadCounts {
    storage: StorageBackend,
    reads: std::rc::Rc<std::cell::RefCell<BTreeMap<u64, usize>>>,
}
impl Storage for ReadCounts {
    fn len(&self) -> Result<u64> {
        self.storage.len()
    }
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        *self.reads.borrow_mut().entry(offset).or_default() += 1;
        self.storage.read_at(offset, len)
    }
    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        *self.reads.borrow_mut().entry(offset).or_default() += 1;
        self.storage.read_at_into(offset, out)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        self.storage.append(bytes)
    }
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        self.storage.write_at(offset, bytes)
    }
    fn truncate(&mut self, len: u64) -> Result<()> {
        self.storage.truncate(len)
    }
    fn sync(&self) -> Result<()> {
        self.storage.sync()
    }
}

#[test]
fn one_file_can_exceed_the_ownership_envelopes_512_extent_limit() {
    let mode = mode(false, false, false, true);
    let authority = Authority::Checksum;
    let len = 33 * 1024 * 1024 + 17;
    let storage = Files::create(
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        &authority,
        None,
        None,
        65536,
        [Input {
            path: b"large".to_vec(),
            reader: Pattern {
                position: 0,
                len,
                fail: None,
            },
        }],
    )
    .unwrap();
    let reads = Default::default();
    let counted = ReadCounts { storage, reads };
    let reads = counted.reads.clone();
    let mut files = Files::open(counted, archive(), mode, &authority, None).unwrap();
    assert!(files.info(b"large").unwrap().unwrap().count() > 512);
    reads.borrow_mut().clear();
    let mut position = 0;
    files
        .read_range(b"large", 0, len, |bytes| {
            for byte in bytes {
                assert_eq!(*byte, pattern(position));
                position += 1;
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(position, len);
    // The file-info lookup and range walk may share pages, but no page may
    // be read once per extent. This covers a file spanning several index leaves.
    assert!(reads.borrow().values().all(|count| *count <= 2));
}
#[test]
fn signed_plaintext_open_remains_eager_and_other_modes_verify_before_yield() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            let mode = mode(encrypted, signed, false, true);
            let authority = authority(mode, &public);
            let mut storage = Files::create(
                StorageBackend::memory(Vec::new()),
                archive(),
                mode,
                &authority,
                signed.then_some(&owner),
                key(mode),
                65536,
                [
                    Input {
                        path: b"first".to_vec(),
                        reader: Cursor::new(vec![31; 80000]),
                    },
                    Input {
                        path: b"other".to_vec(),
                        reader: Cursor::new(b"surviving neighbour".to_vec()),
                    },
                ],
            )
            .unwrap();
            let files =
                Files::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
            let info = files.info(b"first").unwrap().unwrap();
            let chunk = files
                .index
                .get(
                    &storage,
                    files.anchor.index,
                    files.anchor.sealed_len,
                    CHUNK,
                    &chunk_key(info.id, 0),
                )
                .unwrap()
                .unwrap();
            let extent = OwnedRecord::decode(&chunk.value).unwrap().extents[0];
            let byte = storage.read_at(extent.start, 1).unwrap()[0];
            storage.write_at(extent.start, &[byte ^ 0x80]).unwrap();
            let opened = Files::open(storage, archive(), mode, &authority, key(mode));
            if signed && !encrypted {
                assert!(opened.is_err());
                continue;
            }
            let mut files = opened.unwrap();
            let mut yielded = false;
            assert!(files
                .read_range(b"first", 0, 1, |_| {
                    yielded = true;
                    Ok(())
                })
                .is_err());
            assert!(!yielded);
            let mut other = Vec::new();
            files
                .read_range(b"other", 0, 19, |bytes| {
                    other.extend_from_slice(bytes);
                    Ok(())
                })
                .unwrap();
            assert_eq!(other, b"surviving neighbour");
        }
    }
}
#[test]
fn semantic_audit_rejects_missing_and_orphaned_chunk_membership() {
    let mode = mode(false, false, false, false);
    let authority = Authority::Checksum;
    let original = Files::create(
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        &authority,
        None,
        None,
        65536,
        [Input {
            path: b"file".to_vec(),
            reader: Cursor::new(vec![7; 90000]),
        }],
    )
    .unwrap();
    let files = Files::open(original.clone(), archive(), mode, &authority, None).unwrap();
    let info = files.info(b"file").unwrap().unwrap();
    for (namespace, name) in [
        (FILE, b"file".to_vec()),
        (CHUNK, chunk_key(info.id, 0).to_vec()),
        (CHUNK, chunk_key(info.id, 1).to_vec()),
    ] {
        // Construct writer bugs using authenticated transactions. No public CLI
        // activates this candidate or permits orphan chunk records directly.
        let mut tx =
            Transaction::begin(original.clone(), archive(), mode, &authority, None).unwrap();
        tx.remove(namespace, &name).unwrap();
        let (storage, _) = tx.commit(&authority, None).unwrap();
        assert!(Files::open(storage.clone(), archive(), mode, &authority, None).is_err());
        if namespace == CHUNK {
            // Deliberately bypass the full open audit to test the range walk's
            // own missing-first/missing-last ordinal check against an otherwise
            // authenticated writer-bug state. No production entry point does this.
            let anchor = publication::select(&storage, archive(), mode, &authority)
                .unwrap()
                .anchor;
            let mut files = Files {
                storage,
                anchor,
                index: Index::new(archive(), mode, None).unwrap(),
                codec: Codec::packed(archive(), mode, None).unwrap(),
            };
            assert!(matches!(
                files.read_range(b"file", 0, info.len, |_| Ok(())),
                Err(Error::CorruptRecord)
            ));
        }
    }
}
#[test]
fn source_failure_aborts_real_file_allocations_and_reopens_empty() {
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let mut random = [0; 8];
    getrandom::fill(&mut random).unwrap();
    let cleanup = Cleanup(std::env::temp_dir().join(format!(
        "revault-candidate-source-failure-{}-{}",
        std::process::id(),
        u64::from_le_bytes(random)
    )));
    let mode = mode(false, false, false, true);
    let authority = Authority::Checksum;
    let mut storage = StorageBackend::create_file(&cleanup.0, &[]).unwrap();
    assert!(Files::create(
        storage.clone(),
        archive(),
        mode,
        &authority,
        None,
        None,
        65536,
        [Input {
            path: b"file".to_vec(),
            reader: Pattern {
                position: 0,
                len: 500000,
                fail: Some(180000)
            }
        }]
    )
    .is_err());
    let anchor = allocation::recover(&mut storage, archive(), mode, &authority, None).unwrap();
    let index = Index::new(archive(), mode, None).unwrap();
    Snapshot::inspect(&storage, &anchor, &index)
        .unwrap()
        .verify_reclaimed(&storage)
        .unwrap();
    drop(storage);
    let files = Files::open(
        StorageBackend::file(&cleanup.0).unwrap(),
        archive(),
        mode,
        &authority,
        None,
    )
    .unwrap();
    assert!(files.info(b"file").unwrap().is_none());
}

fn packed_pair(
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
) -> StorageBackend {
    Files::create(
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        authority,
        signer,
        key(mode),
        65536,
        [
            Input {
                path: b"erase".to_vec(),
                reader: Cursor::new(vec![0xa7; 4096]),
            },
            Input {
                path: b"keep".to_vec(),
                reader: Cursor::new(vec![0x39; 8192]),
            },
        ],
    )
    .unwrap()
}
fn first_record<S: Storage>(files: &Files<S>, path: &[u8]) -> OwnedRecord {
    let info = files.info(path).unwrap().unwrap();
    let entry = files
        .index
        .get(
            &files.storage,
            files.anchor.index,
            files.anchor.sealed_len,
            CHUNK,
            &chunk_key(info.id, 0),
        )
        .unwrap()
        .unwrap();
    OwnedRecord::decode(&entry.value).unwrap()
}
#[test]
fn shared_pack_deletion_rewrites_survivor_and_erases_whole_old_allocation_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                for padded in [false, true] {
                    let mode = mode(encrypted, signed, compressed, padded);
                    let authority = authority(mode, &public);
                    let storage = packed_pair(mode, &authority, signed.then_some(&owner));
                    let files =
                        Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
                    let erased = first_record(&files, b"erase");
                    let kept = first_record(&files, b"keep");
                    assert_eq!(erased.extents, kept.extents);
                    let old = erased.extents[0];
                    let mut storage = Files::remove(
                        files.into_storage(),
                        archive(),
                        mode,
                        &authority,
                        signed.then_some(&owner),
                        key(mode),
                        [b"erase".to_vec()],
                    )
                    .unwrap();
                    let anchor =
                        allocation::recover(&mut storage, archive(), mode, &authority, key(mode))
                            .unwrap();
                    Snapshot::inspect(
                        &storage,
                        &anchor,
                        &Index::new(archive(), mode, key(mode)).unwrap(),
                    )
                    .unwrap()
                    .verify_reclaimed(&storage)
                    .unwrap();
                    assert!(storage
                        .read_at(old.start, old.len as usize)
                        .unwrap()
                        .iter()
                        .all(|byte| *byte == 0));
                    let mut files = Files::open(
                        StorageBackend::memory(storage.read_all().unwrap()),
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap();
                    assert!(files.info(b"erase").unwrap().is_none());
                    let kept = first_record(&files, b"keep");
                    assert_ne!(old.start, kept.extents[0].start);
                    let slice = Slice::decode(&kept.metadata).unwrap();
                    assert_eq!(slice.start, 0);
                    assert_eq!(slice.descriptor.logical_len, 8192);
                    let mut got = Vec::new();
                    files
                        .read_range(b"keep", 0, 8192, |bytes| {
                            got.extend_from_slice(bytes);
                            Ok(())
                        })
                        .unwrap();
                    assert_eq!(got, vec![0x39; 8192]);
                    let storage = files.into_storage();
                    let before = storage.read_all().unwrap();
                    let storage = Files::remove(
                        storage,
                        archive(),
                        mode,
                        &authority,
                        signed.then_some(&owner),
                        key(mode),
                        [b"erase".to_vec()],
                    )
                    .unwrap();
                    assert_eq!(
                        storage.read_all().unwrap(),
                        before,
                        "no-change deletion wrote bytes"
                    );
                    let storage = Files::remove(
                        storage,
                        archive(),
                        mode,
                        &authority,
                        signed.then_some(&owner),
                        key(mode),
                        [b"keep".to_vec()],
                    )
                    .unwrap();
                    let files =
                        Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
                    assert!(files.info(b"keep").unwrap().is_none());
                }
            }
        }
    }
}
#[test]
fn shared_pack_audit_rejects_hidden_deleted_bytes_overlaps_and_conflicting_descriptors() {
    let mode = mode(false, false, false, true);
    let authority = Authority::Checksum;
    let original = packed_pair(mode, &authority, None);
    for fault in 0..4 {
        let files = Files::open(original.clone(), archive(), mode, &authority, None).unwrap();
        let erase_id = files.info(b"erase").unwrap().unwrap().id;
        let keep_id = files.info(b"keep").unwrap().unwrap().id;
        let kept = first_record(&files, b"keep");
        let mut slice = Slice::decode(&kept.metadata).unwrap();
        let mut tx =
            Transaction::begin(files.into_storage(), archive(), mode, &authority, None).unwrap();
        // Internal authenticated writer-bug fixtures: the public CLI cannot emit
        // this candidate or create an orphaned logical slice directly.
        if fault == 0 {
            tx.remove(FILE, b"erase").unwrap();
            tx.remove(CHUNK, &chunk_key(erase_id, 0)).unwrap();
        } else if fault == 3 {
            tx.remove(FILE, b"keep").unwrap();
            tx.remove(CHUNK, &chunk_key(keep_id, 0)).unwrap();
        } else {
            if fault == 1 {
                slice.start -= 1;
            } else {
                slice.descriptor.object[0] ^= 1;
            }
            tx.put(
                CHUNK,
                &chunk_key(keep_id, 0),
                &slice.encode(),
                &kept.extents,
            )
            .unwrap();
        }
        let (storage, _) = tx.commit(&authority, None).unwrap();
        assert!(Files::open(storage, archive(), mode, &authority, None).is_err());
    }
}

#[derive(Clone, Debug)]
struct SharedMemory(std::sync::Arc<std::sync::Mutex<StorageBackend>>);
impl SharedMemory {
    fn new(bytes: Vec<u8>) -> Self {
        Self(std::sync::Arc::new(std::sync::Mutex::new(
            StorageBackend::memory(bytes),
        )))
    }
    fn operations(&self) -> usize {
        self.0.lock().unwrap().memory_operation_count()
    }
    fn fail(&self, at: usize) {
        self.0
            .lock()
            .unwrap()
            .fail_memory_operation_after_successes(at);
    }
}
impl Storage for SharedMemory {
    fn len(&self) -> Result<u64> {
        self.0.lock().unwrap().len()
    }
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        self.0.lock().unwrap().read_at(offset, len)
    }
    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        self.0.lock().unwrap().read_at_into(offset, out)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        self.0.lock().unwrap().append(bytes)
    }
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        self.0.lock().unwrap().write_at(offset, bytes)
    }
    fn truncate(&mut self, len: u64) -> Result<()> {
        self.0.lock().unwrap().truncate(len)
    }
    fn sync(&self) -> Result<()> {
        self.0.lock().unwrap().sync()
    }
}
#[test]
fn shared_pack_removal_recovers_complete_old_or_new_files_at_every_storage_failure() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for mode in [
        mode(false, false, false, false),
        mode(true, true, true, true),
        mode(false, true, true, true),
        mode(true, false, false, true),
    ] {
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let seed = packed_pair(mode, &authority, signer).read_all().unwrap();
        let files = Files::open(
            StorageBackend::memory(seed.clone()),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let base = files.anchor.generation;
        let old = first_record(&files, b"erase").extents[0];
        drop(files);
        let observed = SharedMemory::new(seed.clone());
        Files::remove(
            observed.clone(),
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            [b"erase".to_vec()],
        )
        .unwrap();
        let count = observed.operations();
        assert!(count > 10);
        for at in 0..count {
            let mut damaged = SharedMemory::new(seed.clone());
            damaged.fail(at);
            let _result = Files::remove(
                damaged.clone(),
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                [b"erase".to_vec()],
            );
            let selected =
                allocation::recover(&mut damaged, archive(), mode, &authority, key(mode)).unwrap();
            Snapshot::inspect(
                &damaged,
                &selected,
                &Index::new(archive(), mode, key(mode)).unwrap(),
            )
            .unwrap()
            .verify_reclaimed(&damaged)
            .unwrap();
            let mut files = Files::open(
                StorageBackend::memory(damaged.read_all().unwrap()),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap();
            let mut kept = Vec::new();
            files
                .read_range(b"keep", 0, 8192, |bytes| {
                    kept.extend_from_slice(bytes);
                    Ok(())
                })
                .unwrap();
            assert_eq!(kept, vec![0x39; 8192]);
            if selected.generation == base {
                let mut erased = Vec::new();
                files
                    .read_range(b"erase", 0, 4096, |bytes| {
                        erased.extend_from_slice(bytes);
                        Ok(())
                    })
                    .unwrap();
                assert_eq!(erased, vec![0xa7; 4096]);
            } else {
                assert_eq!(selected.generation, base + 1);
                assert!(files.info(b"erase").unwrap().is_none());
                assert!(damaged
                    .read_at(old.start, old.len as usize)
                    .unwrap()
                    .iter()
                    .all(|byte| *byte == 0));
            }
            cases += 1;
        }
    }
    println!("PACKED_FILE_REMOVAL_FAILURE_CASES {cases}");
}
#[test]
fn packed_slice_version_and_bounds_fail_closed() {
    let mode = mode(false, false, false, true);
    let codec = Codec::new(archive(), mode, None).unwrap();
    let files = Files::open(
        packed_pair(mode, &Authority::Checksum, None),
        archive(),
        mode,
        &Authority::Checksum,
        None,
    )
    .unwrap();
    let info = files.info(b"keep").unwrap().unwrap();
    let mut old = info.encode();
    old[..8].copy_from_slice(b"RV4FIL01");
    assert!(FileInfo::decode(&old, &codec).is_err());
    let record = first_record(&files, b"keep");
    let slice = Slice::decode(&record.metadata).unwrap();
    assert!(Slice::decode(&slice.descriptor.encode()).is_err());
    let mut invalid = slice.clone();
    invalid.start = u32::MAX;
    assert!(Slice::decode(&invalid.encode()).is_err());
    let mut invalid = slice.encode();
    invalid[76] = 1;
    assert!(Slice::decode(&invalid).is_err());
}

#[test]
fn shared_pack_keeps_intact_neighbor_readable_after_other_fragment_damage() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                let mode = mode(encrypted, signed, compressed, true);
                let authority = authority(mode, &public);
                let mut files = Files::open(
                    packed_pair(mode, &authority, signed.then_some(&owner)),
                    archive(),
                    mode,
                    &authority,
                    key(mode),
                )
                .unwrap();
                let first = first_record(&files, b"erase");
                let second = first_record(&files, b"keep");
                assert_eq!(first.extents, second.extents);
                // Out-of-band payload damage has no public CLI operation. Keep the
                // selected authenticated membership and damage only the first fragment.
                let start = first.extents[0].start;
                let byte = files.storage.read_at(start, 1).unwrap()[0];
                files.storage.write_at(start, &[byte ^ 0x80]).unwrap();
                assert!(files.read_range(b"erase", 0, 4096, |_| Ok(())).is_err());
                let mut got = Vec::new();
                files
                    .read_range(b"keep", 0, 8192, |bytes| {
                        got.extend_from_slice(bytes);
                        Ok(())
                    })
                    .unwrap();
                assert_eq!(got, vec![0x39; 8192]);
            }
        }
    }
}

#[test]
fn encrypted_pack_padding_is_authenticated_ciphertext_and_cannot_hide_retired_bytes() {
    let mode = mode(true, true, true, true);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let storage = packed_pair(mode, &authority, Some(&owner));
    let files = Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
    let first = first_record(&files, b"erase");
    let second = first_record(&files, b"keep");
    let slice = Slice::decode(&second.metadata).unwrap();
    let end = slice.start as u64 + slice.descriptor.stored_len() as u64;
    let pack = first.extents[0];
    let padding = files
        .storage
        .read_at(pack.start + end, (pack.len - end) as usize)
        .unwrap();
    assert!(padding.len() >= 28);
    assert!(padding.iter().any(|b| *b != 0));
    files.codec.verify_pack_padding(&padding).unwrap();
    let mut storage = files.into_storage();
    let byte = storage.read_at(pack.start + end, 1).unwrap()[0];
    storage.write_at(pack.start + end, &[byte ^ 1]).unwrap();
    assert!(Files::open(storage, archive(), mode, &authority, key(mode)).is_err());
}

fn update_inputs(values: &[(&[u8], &[u8])]) -> Vec<Input<Cursor<Vec<u8>>>> {
    values
        .iter()
        .map(|(path, bytes)| Input {
            path: path.to_vec(),
            reader: Cursor::new(bytes.to_vec()),
        })
        .collect()
}
fn assert_file<S: Storage>(files: &mut Files<S>, path: &[u8], expected: Option<&[u8]>) {
    let info = files.info(path).unwrap();
    if let Some(expected) = expected {
        assert_eq!(info.unwrap().len, expected.len() as u64);
        let mut got = Vec::new();
        files
            .read_range(path, 0, expected.len() as u64, |bytes| {
                got.extend_from_slice(bytes);
                Ok(())
            })
            .unwrap();
        assert_eq!(got, expected);
    } else {
        assert!(info.is_none());
    }
}
#[test]
fn packed_updates_cover_add_replace_delete_empty_and_no_change_in_every_mode() {
    // Internal protocol tests: no public CLI writes candidate C yet.
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                for padded in [false, true] {
                    let mode = mode(encrypted, signed, compressed, padded);
                    let authority = authority(mode, &public);
                    let signer = signed.then_some(&owner);
                    let mut storage = packed_pair(mode, &authority, signer);
                    let files = Files::open(
                        StorageBackend::memory(storage.read_all().unwrap()),
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap();
                    let old = first_record(&files, b"erase").extents[0];
                    let base = files.anchor.generation;
                    let large: Vec<_> = (0..300_017).map(pattern).collect();
                    storage = Files::update(
                        storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        MAX_LOGICAL,
                        update_inputs(&[
                            (b"erase", &large),
                            (b"added", b"new content"),
                            (b"empty", b""),
                        ]),
                        [],
                    )
                    .unwrap();
                    assert!(storage
                        .read_at(old.start, old.len as usize)
                        .unwrap()
                        .iter()
                        .all(|b| *b == 0));
                    let mut files = Files::open(
                        StorageBackend::memory(storage.read_all().unwrap()),
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap();
                    assert_eq!(files.anchor.generation, base + 1);
                    assert_file(&mut files, b"erase", Some(&large));
                    assert_file(&mut files, b"keep", Some(&vec![0x39; 8192]));
                    assert_file(&mut files, b"added", Some(b"new content"));
                    assert_file(&mut files, b"empty", Some(b""));
                    let before = storage.read_all().unwrap();
                    storage = Files::update(
                        storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        65536,
                        update_inputs(&[
                            (b"erase", &large),
                            (b"added", b"new content"),
                            (b"empty", b""),
                        ]),
                        [b"absent".to_vec()],
                    )
                    .unwrap();
                    assert_eq!(
                        storage.read_all().unwrap(),
                        before,
                        "no-change includes identical journal bytes"
                    );
                    storage = Files::update(
                        storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        MAX_LOGICAL,
                        update_inputs(&[(b"erase", b"smaller"), (b"added", b"")]),
                        [b"empty".to_vec()],
                    )
                    .unwrap();
                    let mut files = Files::open(
                        StorageBackend::memory(storage.read_all().unwrap()),
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap();
                    assert_eq!(files.anchor.generation, base + 2);
                    assert_file(&mut files, b"erase", Some(b"smaller"));
                    assert_file(&mut files, b"added", Some(b""));
                    assert_file(&mut files, b"empty", None);
                    assert_file(&mut files, b"keep", Some(&vec![0x39; 8192]));
                    Snapshot::inspect(&storage, &files.anchor, &files.index)
                        .unwrap()
                        .verify_reclaimed(&storage)
                        .unwrap();
                }
            }
        }
    }
}

struct ChangingSource {
    cursor: Cursor<Vec<u8>>,
    second: bool,
    behavior: u8,
}
impl Read for ChangingSource {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if self.second && self.behavior == 3 && self.cursor.position() >= 65536 {
            return Err(std::io::Error::other("staging source failed"));
        }
        self.cursor.read(out)
    }
}
impl std::io::Seek for ChangingSource {
    fn seek(&mut self, position: std::io::SeekFrom) -> std::io::Result<u64> {
        if matches!(position, std::io::SeekFrom::Start(0)) {
            self.second = true;
            match self.behavior {
                0 => self.cursor.get_mut()[100_000] ^= 1,
                1 => self.cursor.get_mut().push(7),
                2 => self.cursor.get_mut().truncate(70_000),
                _ => (),
            }
        }
        std::io::Seek::seek(&mut self.cursor, position)
    }
}
#[test]
fn update_source_changes_and_failures_abort_staged_bytes_without_losing_old_files() {
    for behavior in 0..4 {
        let mode = mode(false, false, false, true);
        let seed = packed_pair(mode, &Authority::Checksum, None)
            .read_all()
            .unwrap();
        let mut storage = SharedMemory::new(seed.clone());
        let before = publication::select(&storage, archive(), mode, &Authority::Checksum)
            .unwrap()
            .anchor;
        let result = Files::update(
            storage.clone(),
            archive(),
            mode,
            &Authority::Checksum,
            None,
            None,
            MAX_LOGICAL,
            vec![Input {
                path: b"erase".to_vec(),
                reader: ChangingSource {
                    cursor: Cursor::new(vec![0x61; 150_000]),
                    second: false,
                    behavior,
                },
            }],
            [],
        );
        assert!(result.is_err());
        assert!(
            storage.operations() > 10,
            "failure follows physical preparation"
        );
        let selected =
            allocation::recover(&mut storage, archive(), mode, &Authority::Checksum, None).unwrap();
        assert_eq!(selected.generation, before.generation);
        Snapshot::inspect(
            &storage,
            &selected,
            &Index::new(archive(), mode, None).unwrap(),
        )
        .unwrap()
        .verify_reclaimed(&storage)
        .unwrap();
        let mut files = Files::open(
            StorageBackend::memory(storage.read_all().unwrap()),
            archive(),
            mode,
            &Authority::Checksum,
            None,
        )
        .unwrap();
        assert_file(&mut files, b"erase", Some(&vec![0xa7; 4096]));
        assert_file(&mut files, b"keep", Some(&vec![0x39; 8192]));
    }
}

#[test]
fn mixed_update_is_atomic_at_every_storage_failure() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for mode in [
        mode(false, false, false, false),
        mode(true, true, true, true),
        mode(false, true, true, true),
        mode(true, false, false, true),
    ] {
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let seed = packed_pair(mode, &authority, signer).read_all().unwrap();
        let base = publication::select(
            &StorageBackend::memory(seed.clone()),
            archive(),
            mode,
            &authority,
        )
        .unwrap()
        .anchor
        .generation;
        let values: &[(&[u8], &[u8])] = &[(b"erase", b"replacement"), (b"new", b"addition")];
        let observed = SharedMemory::new(seed.clone());
        Files::update(
            observed.clone(),
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            MAX_LOGICAL,
            update_inputs(values),
            [b"keep".to_vec()],
        )
        .unwrap();
        for at in 0..observed.operations() {
            let mut damaged = SharedMemory::new(seed.clone());
            damaged.fail(at);
            let _ = Files::update(
                damaged.clone(),
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                MAX_LOGICAL,
                update_inputs(values),
                [b"keep".to_vec()],
            );
            let selected =
                allocation::recover(&mut damaged, archive(), mode, &authority, key(mode)).unwrap();
            Snapshot::inspect(
                &damaged,
                &selected,
                &Index::new(archive(), mode, key(mode)).unwrap(),
            )
            .unwrap()
            .verify_reclaimed(&damaged)
            .unwrap();
            let mut files = Files::open(
                StorageBackend::memory(damaged.read_all().unwrap()),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap();
            if selected.generation == base {
                assert_file(&mut files, b"erase", Some(&vec![0xa7; 4096]));
                assert_file(&mut files, b"keep", Some(&vec![0x39; 8192]));
                assert_file(&mut files, b"new", None);
            } else {
                assert_eq!(selected.generation, base + 1);
                assert_file(&mut files, b"erase", Some(b"replacement"));
                assert_file(&mut files, b"keep", None);
                assert_file(&mut files, b"new", Some(b"addition"));
            }
            cases += 1;
        }
    }
    println!("PACKED_FILE_UPDATE_FAILURE_CASES {cases}");
}

#[derive(Default)]
struct Salvaged {
    active: Option<(Vec<u8>, Vec<u8>, u64)>,
    files: BTreeMap<Vec<u8>, Vec<u8>>,
    incomplete: Vec<Vec<u8>>,
}
impl recovery::Sink for Salvaged {
    fn begin(&mut self, path: &[u8], len: u64) -> Result<()> {
        assert!(self.active.is_none());
        self.active = Some((path.to_vec(), Vec::new(), len));
        Ok(())
    }
    fn data(&mut self, bytes: &[u8]) -> Result<()> {
        self.active.as_mut().unwrap().1.extend_from_slice(bytes);
        Ok(())
    }
    fn finish(&mut self, complete: bool) -> Result<()> {
        let (path, bytes, len) = self.active.take().unwrap();
        if complete {
            assert_eq!(bytes.len() as u64, len);
            assert!(self.files.insert(path, bytes).is_none());
        } else {
            self.incomplete.push(path);
        }
        Ok(())
    }
}
#[test]
fn fresh_read_only_salvage_isolates_shared_fragment_damage_in_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                let mode = mode(encrypted, signed, compressed, true);
                let authority = authority(mode, &public);
                let files = Files::open(
                    packed_pair(mode, &authority, signed.then_some(&owner)),
                    archive(),
                    mode,
                    &authority,
                    key(mode),
                )
                .unwrap();
                let bad = first_record(&files, b"erase");
                let slice = Slice::decode(&bad.metadata).unwrap();
                let start = slice.physical(bad.extents[0]).unwrap().start;
                let generation = files.anchor.generation;
                let mut storage = files.into_storage();
                let byte = storage.read_at(start, 1).unwrap()[0];
                storage.write_at(start, &[byte ^ 1]).unwrap();
                let before = storage.read_all().unwrap();
                let mut sink = Salvaged::default();
                let report =
                    Files::salvage(&storage, archive(), mode, &authority, key(mode), &mut sink)
                        .unwrap();
                assert_eq!(report.generation, generation);
                assert_eq!(
                    (report.complete, report.incomplete, report.orphan_chunks),
                    (1, 1, 0)
                );
                assert_eq!(report.membership.unavailable, 0);
                assert_eq!(sink.files[b"keep".as_slice()], vec![0x39; 8192]);
                assert_eq!(sink.incomplete, vec![b"erase".to_vec()]);
                assert_eq!(storage.read_all().unwrap(), before);
            }
        }
    }
}
#[test]
fn salvage_ignores_damaged_padding_and_maps_but_requires_selected_membership_root() {
    let mode = mode(false, true, false, true);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let files = Files::open(
        packed_pair(mode, &authority, Some(&owner)),
        archive(),
        mode,
        &authority,
        None,
    )
    .unwrap();
    let anchor = files.anchor.clone();
    let record = first_record(&files, b"keep");
    let slice = Slice::decode(&record.metadata).unwrap();
    let padding_start =
        record.extents[0].start + slice.start as u64 + slice.descriptor.stored_len() as u64;
    let seed = files.into_storage().read_all().unwrap();
    // Out-of-band corruption has no public CLI operation; candidate C is test-only.
    for damage in [
        padding_start,
        anchor.allocation.primary,
        anchor.allocation.mirror,
    ] {
        let mut storage = StorageBackend::memory(seed.clone());
        let b = storage.read_at(damage, 1).unwrap()[0];
        storage.write_at(damage, &[b ^ 1]).unwrap();
        let mut sink = Salvaged::default();
        let report =
            Files::salvage(&storage, archive(), mode, &authority, None, &mut sink).unwrap();
        assert_eq!((report.complete, report.incomplete), (2, 0));
        assert_eq!(sink.files[b"erase".as_slice()], vec![0xa7; 4096]);
        assert_eq!(sink.files[b"keep".as_slice()], vec![0x39; 8192]);
    }
    let mut storage = StorageBackend::memory(seed);
    for offset in [anchor.index.primary, anchor.index.mirror] {
        let b = storage.read_at(offset, 1).unwrap()[0];
        storage.write_at(offset, &[b ^ 1]).unwrap();
    }
    let mut sink = Salvaged::default();
    assert!(Files::salvage(&storage, archive(), mode, &authority, None, &mut sink).is_err());
    assert!(sink.files.is_empty());
}

#[test]
fn salvage_reports_missing_authenticated_subtrees_and_uses_intact_mirrors() {
    use crate::file_format::authenticated_index::Visit;
    let mode = mode(true, true, true, false);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let storage = Files::create(
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        &authority,
        Some(&owner),
        key(mode),
        MAX_LOGICAL,
        (0..700).map(|i| Input {
            path: format!("file-{i:04}").into_bytes(),
            reader: Cursor::new(vec![(i % 251) as u8; 128]),
        }),
    )
    .unwrap();
    let files = Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
    let mut current = files.anchor.index;
    let mut pages = BTreeMap::new();
    files
        .index
        .visit_owned(
            &files.storage,
            files.anchor.index,
            files.anchor.sealed_len,
            |visit| {
                match visit {
                    Visit::Page(page) => current = page,
                    Visit::Entry(entry) => {
                        pages.entry(entry.namespace).or_insert(current);
                    }
                }
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(pages.len(), 2);
    let seed = files.into_storage().read_all().unwrap();
    for page in pages.into_values() {
        for both in [false, true] {
            let mut storage = StorageBackend::memory(seed.clone());
            for offset in [Some(page.primary), both.then_some(page.mirror)]
                .into_iter()
                .flatten()
            {
                let b = storage.read_at(offset, 1).unwrap()[0];
                storage.write_at(offset, &[b ^ 1]).unwrap();
            }
            let before = storage.read_all().unwrap();
            let mut sink = Salvaged::default();
            let report =
                Files::salvage(&storage, archive(), mode, &authority, key(mode), &mut sink)
                    .unwrap();
            if both {
                assert!(report.membership.unavailable > 0);
                assert_eq!(report.membership.damaged_pages.len(), 1);
                assert!(report.complete < 700 && report.complete > 0);
            } else {
                assert_eq!(report.membership.unavailable, 0);
                assert_eq!((report.complete, report.incomplete), (700, 0));
            }
            for (path, bytes) in sink.files {
                let i: usize = std::str::from_utf8(&path).unwrap()[5..].parse().unwrap();
                assert_eq!(bytes, vec![(i % 251) as u8; 128]);
            }
            assert_eq!(storage.read_all().unwrap(), before);
        }
    }
}
#[test]
fn salvage_recovers_prior_intact_files_after_tail_truncation() {
    let mode = mode(false, true, false, true);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let files = Files::open(
        packed_pair(mode, &authority, Some(&owner)),
        archive(),
        mode,
        &authority,
        None,
    )
    .unwrap();
    let cut = files.anchor.index.mirror;
    assert!(files.anchor.index.primary + files.anchor.index.len <= cut);
    for path in [b"erase".as_slice(), b"keep"] {
        let extent = first_record(&files, path).extents[0];
        assert!(extent.start + extent.len <= cut);
    }
    let mut storage = files.into_storage();
    storage.truncate(cut).unwrap();
    let before = storage.read_all().unwrap();
    assert!(Files::open(
        StorageBackend::memory(before.clone()),
        archive(),
        mode,
        &authority,
        None
    )
    .is_err());
    let mut sink = Salvaged::default();
    let report = Files::salvage(&storage, archive(), mode, &authority, None, &mut sink).unwrap();
    assert_eq!((report.complete, report.incomplete), (2, 0));
    assert_eq!(sink.files[b"erase".as_slice()], vec![0xa7; 4096]);
    assert_eq!(sink.files[b"keep".as_slice()], vec![0x39; 8192]);
    assert_eq!(storage.read_all().unwrap(), before);
}
#[test]
fn salvage_never_resurrects_deleted_or_unpublished_membership() {
    let mode = mode(false, true, false, true);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let seed = packed_pair(mode, &authority, Some(&owner));
    let mut storage = Files::remove(
        seed,
        archive(),
        mode,
        &authority,
        Some(&owner),
        None,
        [b"erase".to_vec()],
    )
    .unwrap();
    let selected = publication::select(&storage, archive(), mode, &authority)
        .unwrap()
        .anchor;
    // Authentic-looking but unreferenced bytes appended after publication do not
    // authorize membership. Payload scanning cannot bring the deleted file back.
    storage.append(&vec![0xa7; 4096]).unwrap();
    let before = storage.read_all().unwrap();
    let mut sink = Salvaged::default();
    let report = Files::salvage(&storage, archive(), mode, &authority, None, &mut sink).unwrap();
    assert_eq!(report.generation, selected.generation);
    assert_eq!((report.complete, report.incomplete), (1, 0));
    assert!(!sink.files.contains_key(b"erase".as_slice()));
    assert_eq!(sink.files[b"keep".as_slice()], vec![0x39; 8192]);
    assert_eq!(storage.read_all().unwrap(), before);
}
#[test]
fn salvage_propagates_sink_and_key_errors_without_finishing_active_file() {
    struct Failing {
        active: bool,
    }
    impl recovery::Sink for Failing {
        fn begin(&mut self, _: &[u8], _: u64) -> Result<()> {
            self.active = true;
            Ok(())
        }
        fn data(&mut self, _: &[u8]) -> Result<()> {
            Err(Error::Io("sink full".into()))
        }
        fn finish(&mut self, _: bool) -> Result<()> {
            panic!("failed sink must not finish")
        }
    }
    let mode = mode(true, true, false, true);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let storage = packed_pair(mode, &authority, Some(&owner));
    let mut sink = Failing { active: false };
    assert!(matches!(
        Files::salvage(&storage, archive(), mode, &authority, key(mode), &mut sink),
        Err(Error::Io(_))
    ));
    assert!(sink.active);
    let mut sink = Salvaged::default();
    assert!(Files::salvage(
        &storage,
        archive(),
        mode,
        &authority,
        Some(&[0; 32]),
        &mut sink
    )
    .is_err());
    assert!(sink.files.is_empty());
    let other = OwnerSigningKeyPair::generate().unwrap().public_key();
    assert!(Files::salvage(
        &storage,
        archive(),
        mode,
        &Authority::Owner(&other),
        key(mode),
        &mut sink
    )
    .is_err());
}

#[test]
fn packed_update_aging_reclaims_bytes_and_no_change_never_accumulates() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for mode in [
        mode(false, false, false, true),
        mode(true, true, true, true),
        mode(false, true, true, true),
        mode(true, false, false, false),
    ] {
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = packed_pair(mode, &authority, signer);
        let mut high_water = 0;
        for cycle in 0..100 {
            let bytes = vec![(cycle % 2) as u8; 4096];
            storage = Files::update(
                storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                MAX_LOGICAL,
                update_inputs(&[(b"erase", &bytes)]),
                [],
            )
            .unwrap();
            let selected = publication::select(&storage, archive(), mode, &authority)
                .unwrap()
                .anchor;
            Snapshot::inspect(
                &storage,
                &selected,
                &Index::new(archive(), mode, key(mode)).unwrap(),
            )
            .unwrap()
            .verify_reclaimed(&storage)
            .unwrap();
            let before = storage.read_all().unwrap();
            if cycle < 10 {
                high_water = high_water.max(before.len());
            } else {
                assert!(
                    before.len() <= high_water,
                    "physical growth after warmup, cycle {cycle}"
                );
            }
            storage = Files::update(
                storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                MAX_LOGICAL,
                update_inputs(&[(b"erase", &bytes)]),
                [],
            )
            .unwrap();
            assert_eq!(storage.read_all().unwrap(), before);
            let mut files = Files::open(
                StorageBackend::memory(before),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap();
            assert_file(&mut files, b"erase", Some(&bytes));
            assert_file(&mut files, b"keep", Some(&vec![0x39; 8192]));
        }
    }
}
#[test]
fn conflicting_update_paths_fail_before_archive_mutation() {
    let mode = mode(false, false, false, true);
    for duplicate in [false, true] {
        let storage = SharedMemory::new(
            packed_pair(mode, &Authority::Checksum, None)
                .read_all()
                .unwrap(),
        );
        let before = storage.read_all().unwrap();
        let inputs = if duplicate {
            update_inputs(&[(b"new", b"one"), (b"new", b"two")])
        } else {
            update_inputs(&[(b"erase", b"replacement")])
        };
        let removals = if duplicate {
            vec![]
        } else {
            vec![b"erase".to_vec()]
        };
        assert!(Files::update(
            storage.clone(),
            archive(),
            mode,
            &Authority::Checksum,
            None,
            None,
            MAX_LOGICAL,
            inputs,
            removals
        )
        .is_err());
        assert_eq!(storage.operations(), 0);
        assert_eq!(storage.read_all().unwrap(), before);
    }
}

#[test]
fn salvage_discards_partial_files_and_preserves_authenticated_empty_files() {
    let mode = mode(false, true, false, true);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let storage = Files::create(
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        &authority,
        Some(&owner),
        None,
        65536,
        update_inputs(&[
            (b"partial", &vec![0x51; 150_000]),
            (b"empty", b""),
            (b"intact", b"surviving bytes"),
        ]),
    )
    .unwrap();
    let files = Files::open(storage, archive(), mode, &authority, None).unwrap();
    let id = files.info(b"partial").unwrap().unwrap().id;
    let entry = files
        .index
        .get(
            &files.storage,
            files.anchor.index,
            files.anchor.sealed_len,
            CHUNK,
            &chunk_key(id, 1),
        )
        .unwrap()
        .unwrap();
    let record = OwnedRecord::decode(&entry.value).unwrap();
    let fragment = Slice::decode(&record.metadata)
        .unwrap()
        .physical(record.extents[0])
        .unwrap();
    let mut storage = files.into_storage();
    storage.write_at(fragment.start, &[0x99]).unwrap();
    let mut sink = Salvaged::default();
    let report = Files::salvage(&storage, archive(), mode, &authority, None, &mut sink).unwrap();
    assert_eq!((report.complete, report.incomplete), (2, 1));
    assert_eq!(sink.incomplete, vec![b"partial".to_vec()]);
    assert_eq!(sink.files[b"empty".as_slice()], b"");
    assert_eq!(sink.files[b"intact".as_slice()], b"surviving bytes");
}

#[test]
fn compaction_preserves_identity_content_and_generation_in_every_mode() {
    // Internal test-only format: no public CLI can construct candidate C.
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                for padded in [false, true] {
                    let mode = mode(encrypted, signed, compressed, padded);
                    let authority = authority(mode, &public);
                    let signer = signed.then_some(&owner);
                    let mut storage = packed_pair(mode, &authority, signer);
                    storage = Files::update(
                        storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        MAX_LOGICAL,
                        update_inputs(&[(b"erase", &vec![0xa1; 700_000]), (b"empty", b"")]),
                        [],
                    )
                    .unwrap();
                    storage = Files::update(
                        storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        MAX_LOGICAL,
                        update_inputs(&[(b"erase", b"current small replacement")]),
                        [],
                    )
                    .unwrap();
                    let original = storage.read_all().unwrap();
                    let mut files =
                        Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
                    let generation = files.anchor.generation;
                    let commitment = files.anchor.commitment().unwrap();
                    let identities: Vec<_> = [b"erase".as_slice(), b"keep", b"empty"]
                        .iter()
                        .map(|path| files.info(path).unwrap().unwrap().encode())
                        .collect();
                    let compacted = files
                        .compact_into(
                            StorageBackend::memory(Vec::new()),
                            &authority,
                            signer,
                            key(mode),
                        )
                        .unwrap();
                    assert_eq!(files.storage.read_all().unwrap(), original);
                    assert!(compacted.len().unwrap() < original.len() as u64);
                    let mut reopened = Files::open(
                        StorageBackend::memory(compacted.read_all().unwrap()),
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap();
                    assert_eq!(reopened.anchor.generation, generation + 1);
                    assert_eq!(reopened.anchor.previous, commitment);
                    for (path, identity) in [b"erase".as_slice(), b"keep", b"empty"]
                        .iter()
                        .zip(identities)
                    {
                        assert_eq!(reopened.info(path).unwrap().unwrap().encode(), identity);
                    }
                    assert_file(&mut reopened, b"erase", Some(b"current small replacement"));
                    assert_file(&mut reopened, b"keep", Some(&vec![0x39; 8192]));
                    assert_file(&mut reopened, b"empty", Some(b""));
                    Snapshot::inspect(&reopened.storage, &reopened.anchor, &reopened.index)
                        .unwrap()
                        .verify_reclaimed(&reopened.storage)
                        .unwrap();
                    // The replacement is writable through the existing journal, not merely
                    // a readable copy with an uninitialized preparation region.
                    let updated = Files::update(
                        reopened.into_storage(),
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        MAX_LOGICAL,
                        update_inputs(&[(b"later", b"after compaction")]),
                        [b"erase".to_vec()],
                    )
                    .unwrap();
                    let mut reopened =
                        Files::open(updated, archive(), mode, &authority, key(mode)).unwrap();
                    assert_eq!(reopened.anchor.generation, generation + 2);
                    assert_file(&mut reopened, b"later", Some(b"after compaction"));
                    assert_file(&mut reopened, b"erase", None);
                    assert_file(&mut reopened, b"keep", Some(&vec![0x39; 8192]));
                }
            }
        }
    }
}
#[test]
fn failed_compaction_clears_replacement_and_never_modifies_source() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for mode in [
        mode(false, false, false, false),
        mode(true, true, true, true),
        mode(false, true, true, true),
        mode(true, false, false, true),
    ] {
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let storage = packed_pair(mode, &authority, signer);
        let before = storage.read_all().unwrap();
        let mut files = Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
        let observed = SharedMemory::new(Vec::new());
        files
            .compact_into(observed.clone(), &authority, signer, key(mode))
            .unwrap();
        for at in 0..observed.operations() {
            let replacement = SharedMemory::new(Vec::new());
            replacement.fail(at);
            assert!(
                files
                    .compact_into(replacement.clone(), &authority, signer, key(mode))
                    .is_err(),
                "failure case {at}"
            );
            assert_eq!(
                replacement.len().unwrap(),
                0,
                "failed replacement must be cleared"
            );
            assert_eq!(files.storage.read_all().unwrap(), before);
            let mut original = Files::open(
                StorageBackend::memory(before.clone()),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap();
            assert_file(&mut original, b"erase", Some(&vec![0xa7; 4096]));
            assert_file(&mut original, b"keep", Some(&vec![0x39; 8192]));
            cases += 1;
        }
    }
    println!("CANDIDATE_COMPACTION_FAILURE_CASES {cases}");
}
#[test]
fn compaction_refuses_nonempty_output_and_damaged_source_in_lazy_modes() {
    for encrypted in [false, true] {
        let mode = mode(encrypted, false, true, true);
        let authority = if encrypted {
            Authority::Symmetric(KEY)
        } else {
            Authority::Checksum
        };
        let mut files = Files::open(
            packed_pair(mode, &authority, None),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let replacement = SharedMemory::new(b"must preserve destination".to_vec());
        assert!(files
            .compact_into(replacement.clone(), &authority, None, key(mode))
            .is_err());
        assert_eq!(
            replacement.read_all().unwrap(),
            b"must preserve destination"
        );
        let record = first_record(&files, b"erase");
        let fragment = Slice::decode(&record.metadata)
            .unwrap()
            .physical(record.extents[0])
            .unwrap();
        let byte = files.storage.read_at(fragment.start, 1).unwrap()[0];
        files.storage.write_at(fragment.start, &[byte ^ 1]).unwrap();
        let damaged = files.storage.read_all().unwrap();
        let replacement = SharedMemory::new(Vec::new());
        assert!(files
            .compact_into(replacement.clone(), &authority, None, key(mode))
            .is_err());
        assert_eq!(replacement.operations(), 0);
        assert_eq!(files.storage.read_all().unwrap(), damaged);
    }
}

mod compaction_tests;

mod aging;

#[test]
fn whole_layout_cost_model_preserves_source_and_descriptor_bindings() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                for padded in [false, true] {
                    let mode = mode(encrypted, signed, compressed, padded);
                    let authority = authority(mode, &public);
                    let signer = signed.then_some(&owner);
                    let source = Files::create(
                        StorageBackend::memory(Vec::new()),
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        MAX_LOGICAL,
                        update_inputs(&[
                            (b"empty", b""),
                            (b"small", b"small contents"),
                            (b"multiple", &vec![0x93; 400_013]),
                        ]),
                    )
                    .unwrap();
                    let original = source.read_all().unwrap();
                    let mut files =
                        Files::open(source, archive(), mode, &authority, key(mode)).unwrap();
                    let projected = super::cost_model::project(&mut files).unwrap();
                    assert_eq!(projected["files"], 3);
                    assert_eq!(projected["metadata_roundtrip_verified"], true);
                    assert_eq!(projected["descriptor_binding_preserved"], true);
                    assert_eq!(projected["inline_catalogue_fits"], true);
                    assert!(projected["fragments"].as_u64().unwrap() >= 3);
                    assert!(
                        projected["largest_fragment_decoded_bytes"]
                            .as_u64()
                            .unwrap()
                            <= MAX_LOGICAL as u64
                    );
                    assert_eq!(files.storage.read_all().unwrap(), original);
                }
            }
        }
    }
}

#[test]
fn dense_catalogue_reconstructs_and_reads_relocated_fragments_in_all_modes() {
    use super::dense_catalogue::Catalogue;
    use crate::file_format::allocation_map::Extent;
    use crate::file_format::publication_anchor::REGION_LEN;
    use crate::page_buffer::ZeroizingBytes;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                for padded in [false, true] {
                    let mode = mode(encrypted, signed, compressed, padded);
                    let authority = authority(mode, &public);
                    let large = vec![0x92; 400_013];
                    let source = Files::create(
                        StorageBackend::memory(Vec::new()),
                        archive(),
                        mode,
                        &authority,
                        signed.then_some(&owner),
                        key(mode),
                        MAX_LOGICAL,
                        update_inputs(&[
                            (b"empty", b""),
                            (b"multiple", &large),
                            (b"small", b"small contents"),
                        ]),
                    )
                    .unwrap();
                    let original = source.read_all().unwrap();
                    let mut source =
                        Files::open(source, archive(), mode, &authority, key(mode)).unwrap();
                    let mut payload = StorageBackend::memory(vec![0; REGION_LEN]);
                    let mut body = None;
                    let projection = super::cost_model::project_into(
                        &mut source,
                        |extent, bytes| {
                            assert_eq!(payload.append(bytes)?, extent.start);
                            Ok(())
                        },
                        |bytes| {
                            body = Some(ZeroizingBytes::new(bytes.to_vec()));
                            Ok(())
                        },
                    )
                    .unwrap();
                    let body = body.unwrap();
                    assert_eq!(
                        payload.len().unwrap(),
                        projection["projected_compacted_bytes"].as_u64().unwrap()
                    );
                    let mut codec = Codec::shared_packed(archive(), mode, key(mode)).unwrap();
                    let catalogue =
                        Catalogue::decode(&body, &codec, payload.len().unwrap()).unwrap();
                    catalogue.verify_padding(&payload, &codec).unwrap();
                    assert_eq!(catalogue.files.len(), 3);
                    for file in &catalogue.files {
                        assert_eq!(file.permissions, 0o644); // Explicit synthetic cost-model value.
                        let mut decoded =
                            ZeroizingBytes::new(Vec::with_capacity(file.info.len as usize));
                        for fragment in &file.fragments {
                            let pack = &catalogue.packs[fragment.pack];
                            let extent = Extent {
                                start: pack.extent.start + fragment.relative as u64,
                                len: fragment.descriptor.stored_len() as u64,
                                digest: fragment.digest,
                            };
                            decoded.extend_from_slice(
                                &codec
                                    .load(
                                        &payload,
                                        extent,
                                        payload.len().unwrap(),
                                        &fragment.descriptor,
                                    )
                                    .unwrap(),
                            );
                            if extent.start < crate::file_format::preparation_journal::DATA_START {
                                let old = Codec::packed(archive(), mode, key(mode)).unwrap();
                                assert!(old
                                    .validate_extent(
                                        extent,
                                        payload.len().unwrap(),
                                        &fragment.descriptor
                                    )
                                    .is_err());
                            }
                        }
                        let expected: &[u8] = match file.path.as_slice() {
                            b"empty" => b"",
                            b"multiple" => &large,
                            b"small" => b"small contents",
                            _ => panic!("unexpected path"),
                        };
                        assert_eq!(decoded.as_slice(), expected);
                        assert_eq!(
                            <[u8; 32]>::from(Sha256::digest(decoded.as_slice())),
                            file.info.digest
                        );
                    }
                    assert_eq!(source.storage.read_all().unwrap(), original);
                    // The body is a complete fresh catalogue. Prefixes and trailing bytes
                    // cannot be accepted as a partial or different allocation graph.
                    for end in 0..body.len() {
                        assert!(
                            Catalogue::decode(&body[..end], &codec, payload.len().unwrap())
                                .is_err()
                        );
                    }
                    let mut malformed = body.to_vec();
                    malformed.push(0);
                    assert!(Catalogue::decode(&malformed, &codec, payload.len().unwrap()).is_err());
                    let mut malformed = body.to_vec();
                    malformed[8] = 0x83;
                    malformed.insert(9, 0);
                    assert!(Catalogue::decode(&malformed, &codec, payload.len().unwrap()).is_err());
                    let mut malformed = body.to_vec();
                    malformed[9] = 2;
                    assert!(Catalogue::decode(&malformed, &codec, payload.len().unwrap()).is_err());
                    let mut malformed = body.to_vec();
                    malformed[10..14].copy_from_slice(&u32::MAX.to_le_bytes());
                    assert!(Catalogue::decode(&malformed, &codec, payload.len().unwrap()).is_err());
                }
            }
        }
    }
}

#[test]
fn dense_catalogue_sink_failure_preserves_source_and_propagates() {
    let mode = mode(false, false, true, true);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let storage = Files::create(
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        &authority,
        None,
        None,
        MAX_LOGICAL,
        update_inputs(&[(b"file", b"synthetic content")]),
    )
    .unwrap();
    let before = storage.read_all().unwrap();
    let mut files = Files::open(storage, archive(), mode, &authority, None).unwrap();
    for fail_body in [false, true] {
        let result = super::cost_model::project_into(
            &mut files,
            |_, _| {
                if !fail_body {
                    Err(Error::InvalidInput("synthetic pack sink failure".into()))
                } else {
                    Ok(())
                }
            },
            |_| {
                Err(Error::InvalidInput(
                    "synthetic catalogue sink failure".into(),
                ))
            },
        );
        assert!(matches!(result, Err(Error::InvalidInput(_))));
        assert_eq!(files.storage.read_all().unwrap(), before);
    }
}

#[test]
fn dense_image_reopens_files_ranges_and_separated_controls_in_all_modes() {
    use super::dense_image::{from_candidate, Image};
    use crate::file_format::publication_anchor::FAILURE_REGION;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let large = vec![0x57; 400_013];
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                for padded in [false, true] {
                    let mode = mode(encrypted, signed, compressed, padded);
                    let authority = authority(mode, &public);
                    let source = Files::create(
                        StorageBackend::memory(Vec::new()),
                        archive(),
                        mode,
                        &authority,
                        signed.then_some(&owner),
                        key(mode),
                        MAX_LOGICAL,
                        update_inputs(&[
                            (b"empty", b""),
                            (b"multiple", &large),
                            (b"small", b"small contents"),
                        ]),
                    )
                    .unwrap();
                    let before = source.read_all().unwrap();
                    let mut source =
                        Files::open(source, archive(), mode, &authority, key(mode)).unwrap();
                    let output = from_candidate(
                        &mut source,
                        StorageBackend::memory(Vec::new()),
                        &authority,
                        signed.then_some(&owner),
                        key(mode),
                        &[],
                    )
                    .unwrap();
                    assert_eq!(source.storage.read_all().unwrap(), before);
                    for lost in [None, Some(0), Some(FAILURE_REGION)] {
                        let mut fresh = StorageBackend::memory(output.read_all().unwrap());
                        if let Some(offset) = lost {
                            fresh
                                .write_at(offset, &vec![0; FAILURE_REGION as usize])
                                .unwrap();
                        }
                        let mut image =
                            Image::open(fresh, archive(), mode, &authority, key(mode)).unwrap();
                        for (name, expected) in [
                            (b"empty".as_slice(), b"".as_slice()),
                            (b"multiple", large.as_slice()),
                            (b"small", b"small contents"),
                        ] {
                            let mut found = Vec::new();
                            image
                                .read_range(name, 0, expected.len() as u64, |bytes| {
                                    found.extend_from_slice(bytes);
                                    Ok(())
                                })
                                .unwrap();
                            assert_eq!(found, expected);
                        }
                        let mut range = Vec::new();
                        image
                            .read_range(b"multiple", 61000, 9000, |bytes| {
                                range.extend_from_slice(bytes);
                                Ok(())
                            })
                            .unwrap();
                        assert_eq!(range, large[61000..70000]);
                        assert!(image
                            .read_range(b"multiple", u64::MAX, 1, |_| Ok(()))
                            .is_err());
                        assert!(image.read_range(b"absent", 0, 0, |_| Ok(())).is_err());
                        assert!(image.read_range(b"empty", 1, 0, |_| Ok(())).is_err());
                    }
                    let mut damaged = StorageBackend::memory(output.read_all().unwrap());
                    let offset = crate::file_format::publication_anchor::REGION_LEN as u64;
                    let byte = damaged.read_at(offset, 1).unwrap()[0];
                    damaged.write_at(offset, &[byte ^ 1]).unwrap();
                    let opened = Image::open(damaged, archive(), mode, &authority, key(mode));
                    if mode.plaintext() && mode.signed() {
                        assert!(opened.is_err());
                    } else {
                        let mut image = opened.unwrap();
                        let mut exposed = 0;
                        assert!(image
                            .read_range(b"multiple", 0, 1, |bytes| {
                                exposed += bytes.len();
                                Ok(())
                            })
                            .is_err());
                        assert_eq!(exposed, 0);
                    }
                }
            }
        }
    }
}
#[test]
fn dense_image_failures_clear_fresh_output_and_preserve_source() {
    use super::dense_image::from_candidate;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for mode in [
        mode(false, false, true, true),
        mode(true, true, true, true),
        mode(false, true, false, false),
        mode(true, false, false, true),
    ] {
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let storage = packed_pair(mode, &authority, signer);
        let before = storage.read_all().unwrap();
        let mut source = Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
        let observed = SharedMemory::new(Vec::new());
        from_candidate(
            &mut source,
            observed.clone(),
            &authority,
            signer,
            key(mode),
            &[],
        )
        .unwrap();
        for at in 0..observed.operations() {
            let output = SharedMemory::new(Vec::new());
            output.fail(at);
            assert!(
                from_candidate(
                    &mut source,
                    output.clone(),
                    &authority,
                    signer,
                    key(mode),
                    &[]
                )
                .is_err(),
                "failure {at}"
            );
            assert_eq!(output.len().unwrap(), 0);
            assert_eq!(source.storage.read_all().unwrap(), before);
            cases += 1;
        }
        let output = SharedMemory::new(b"existing destination".to_vec());
        assert!(from_candidate(
            &mut source,
            output.clone(),
            &authority,
            signer,
            key(mode),
            &[]
        )
        .is_err());
        assert_eq!(output.read_all().unwrap(), b"existing destination");
    }
    println!("DENSE_IMAGE_MUTATION_FAILURES {cases}");
}
#[test]
fn dense_image_rejects_owner_substitution_and_reads_password_directory() {
    use super::dense_image::{from_candidate, Image};
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(true, true, true, true);
    let authority = authority(mode, &public);
    let mut source = Files::open(
        packed_pair(mode, &authority, Some(&owner)),
        archive(),
        mode,
        &authority,
        key(mode),
    )
    .unwrap();
    let wrong_owner = OwnerSigningKeyPair::generate().unwrap();
    let wrong_public = wrong_owner.public_key();
    let output = SharedMemory::new(Vec::new());
    assert!(from_candidate(
        &mut source,
        output.clone(),
        &Authority::Owner(&wrong_public),
        Some(&wrong_owner),
        key(mode),
        &[]
    )
    .is_err());
    assert_eq!(output.operations(), 0);
    let original_keys = source.anchor.keys;
    source.anchor.keys = source.anchor.index; // No public candidate access-tree writer exists.
    assert!(from_candidate(
        &mut source,
        output.clone(),
        &authority,
        Some(&owner),
        key(mode),
        &[]
    )
    .is_err());
    assert_eq!(output.operations(), 0);
    source.anchor.keys = original_keys;
    let slots = [crate::key_slot::KeySlot::password_bytes(
        1,
        b"synthetic dense password",
        vec![77; 16],
        KEY,
    )
    .unwrap()];
    let mut storage = from_candidate(
        &mut source,
        StorageBackend::memory(Vec::new()),
        &authority,
        Some(&owner),
        key(mode),
        &slots,
    )
    .unwrap();
    let mut reopened = Image::open(
        StorageBackend::memory(storage.read_all().unwrap()),
        archive(),
        mode,
        &authority,
        key(mode),
    )
    .unwrap();
    let mut bytes = Vec::new();
    reopened
        .read_range(b"keep", 0, 8192, |part| {
            bytes.extend_from_slice(part);
            Ok(())
        })
        .unwrap();
    assert_eq!(bytes, vec![0x39; 8192]);
    assert_eq!(reopened.storage.len().unwrap(), storage.len().unwrap());
    let password = crate::SecretString::try_from_slice(b"synthetic dense password").unwrap();
    let mut credential_open = Image::open_credential(
        StorageBackend::memory(storage.read_all().unwrap()),
        archive(),
        mode,
        Some(&public),
        publication::bootstrap::Credential::Password(&password),
        Some(1),
    )
    .unwrap();
    let mut bytes = Vec::new();
    credential_open
        .read_range(b"keep", 0, 8192, |part| {
            bytes.extend_from_slice(part);
            Ok(())
        })
        .unwrap();
    assert_eq!(bytes, vec![0x39; 8192]);
    let original_len = storage.len().unwrap();
    super::dense_update::edit(
        &mut storage,
        archive(),
        mode,
        &authority,
        Some(&owner),
        key(mode),
        b"erase",
        b"renamed",
        Some(0o600),
    )
    .unwrap();
    let before = storage.read_all().unwrap();
    for signer in [None, Some(&wrong_owner)] {
        assert!(super::dense_update::return_inline(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode)
        )
        .is_err());
        assert_eq!(storage.read_all().unwrap(), before);
    }
    super::dense_update::return_inline(
        &mut storage,
        archive(),
        mode,
        &authority,
        Some(&owner),
        key(mode),
    )
    .unwrap();
    assert_eq!(storage.len().unwrap(), original_len);
    let mut after_trim = Image::open_credential(
        StorageBackend::memory(storage.read_all().unwrap()),
        archive(),
        mode,
        Some(&public),
        publication::bootstrap::Credential::Password(&password),
        Some(1),
    )
    .unwrap();
    let mut bytes = Vec::new();
    after_trim
        .read_range(b"renamed", 0, 4096, |part| {
            bytes.extend_from_slice(part);
            Ok(())
        })
        .unwrap();
    assert_eq!(bytes, vec![0xa7; 4096]);
    let wrong = crate::SecretString::try_from_slice(b"wrong synthetic password").unwrap();
    assert!(Image::open_credential(
        storage,
        archive(),
        mode,
        Some(&public),
        publication::bootstrap::Credential::Password(&wrong),
        None
    )
    .is_err());
}

#[test]
fn dense_salvage_preserves_intact_neighbours_and_ignores_unrelated_controls() {
    use super::dense_image::{from_candidate, salvage, Image};
    use crate::file_format::publication_anchor::{FAILURE_REGION, REGION_LEN};
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                let mode = mode(encrypted, signed, compressed, true);
                let authority = authority(mode, &public);
                let mut source = Files::open(
                    packed_pair(mode, &authority, signed.then_some(&owner)),
                    archive(),
                    mode,
                    &authority,
                    key(mode),
                )
                .unwrap();
                let output = from_candidate(
                    &mut source,
                    StorageBackend::memory(Vec::new()),
                    &authority,
                    signed.then_some(&owner),
                    key(mode),
                    &[],
                )
                .unwrap();
                let seed = output.read_all().unwrap();
                for kind in ["fragment", "padding", "journal", "metadata"] {
                    let mut damaged = StorageBackend::memory(seed.clone());
                    match kind {
                        "fragment" => {
                            let old = damaged.read_at(REGION_LEN as u64, 1).unwrap()[0];
                            damaged.write_at(REGION_LEN as u64, &[old ^ 1]).unwrap();
                        }
                        "padding" => {
                            let at = damaged.len().unwrap() - 1;
                            let old = damaged.read_at(at, 1).unwrap()[0];
                            damaged.write_at(at, &[old ^ 1]).unwrap();
                        }
                        "journal" => {
                            for bank in [0, FAILURE_REGION] {
                                damaged.write_at(bank + 8192, &vec![0; 4096]).unwrap();
                            }
                        }
                        "metadata" => {
                            for bank in [0, FAILURE_REGION] {
                                damaged.write_at(bank + 16384, &vec![0; 49152]).unwrap();
                            }
                        }
                        _ => unreachable!(),
                    }
                    let before = damaged.read_all().unwrap();
                    let mut sink = Salvaged::default();
                    let report =
                        salvage(&damaged, archive(), mode, &authority, key(mode), &mut sink);
                    if kind == "metadata" {
                        assert!(report.is_err());
                        assert!(sink.files.is_empty() && sink.active.is_none());
                    } else {
                        let report = report.unwrap();
                        assert_eq!(report.generation, 1);
                        if kind == "fragment" {
                            assert_eq!((report.complete, report.incomplete), (1, 1));
                            assert_eq!(sink.incomplete, vec![b"erase".to_vec()]);
                        } else {
                            assert_eq!((report.complete, report.incomplete), (2, 0));
                        }
                        assert_eq!(sink.files[b"keep".as_slice()], vec![0x39; 8192]);
                    }
                    if kind != "fragment" || mode.plaintext() && mode.signed() {
                        assert!(Image::open(
                            StorageBackend::memory(before.clone()),
                            archive(),
                            mode,
                            &authority,
                            key(mode)
                        )
                        .is_err());
                    }
                    assert_eq!(damaged.read_all().unwrap(), before);
                }
                // Truncate inside the last file's stored fragment, leaving all selected
                // front metadata intact. Earlier authenticated contents remain salvageable.
                let (_, body) = publication::shared::open_private(
                    &output,
                    archive(),
                    mode,
                    &authority,
                    key(mode),
                )
                .unwrap();
                let codec = Codec::shared_packed(archive(), mode, key(mode)).unwrap();
                let catalogue =
                    super::dense_catalogue::Catalogue::decode(&body, &codec, output.len().unwrap())
                        .unwrap();
                let file = catalogue
                    .files
                    .iter()
                    .find(|file| file.path.as_slice() == b"keep")
                    .unwrap();
                let fragment = file.fragments.last().unwrap();
                let end = catalogue.packs[fragment.pack].extent.start
                    + fragment.relative as u64
                    + fragment.descriptor.stored_len() as u64;
                let mut truncated = StorageBackend::memory(seed);
                truncated.truncate(end - 1).unwrap();
                let mut sink = Salvaged::default();
                let report = salvage(
                    &truncated,
                    archive(),
                    mode,
                    &authority,
                    key(mode),
                    &mut sink,
                )
                .unwrap();
                assert_eq!((report.complete, report.incomplete), (1, 1));
                assert_eq!(sink.files[b"erase".as_slice()], vec![0xa7; 4096]);
                assert_eq!(sink.incomplete, vec![b"keep".to_vec()]);
            }
        }
    }
}
#[test]
fn dense_packing_exposes_the_larger_logical_loss_from_a_physical_region_failure() {
    use super::dense_image::{from_candidate, salvage};
    use crate::file_format::publication_anchor::{FAILURE_REGION, REGION_LEN};
    let mode = mode(false, true, true, true);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let inputs = (0..512).map(|index| Input {
        path: format!("file-{index:04}").into_bytes(),
        reader: Cursor::new(vec![index as u8; 4096]),
    });
    let source = Files::create(
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        &authority,
        Some(&owner),
        None,
        MAX_LOGICAL,
        inputs,
    )
    .unwrap();
    let mut source = Files::open(source, archive(), mode, &authority, None).unwrap();
    let old_pack = first_record(&source, b"file-0000").extents[0];
    let image = from_candidate(
        &mut source,
        StorageBackend::memory(Vec::new()),
        &authority,
        Some(&owner),
        None,
        &[],
    )
    .unwrap();
    let mut old_damaged = StorageBackend::memory(source.storage.read_all().unwrap());
    old_damaged
        .write_at(old_pack.start, &vec![0; FAILURE_REGION as usize])
        .unwrap();
    let mut sink = Salvaged::default();
    let old = Files::salvage(&old_damaged, archive(), mode, &authority, None, &mut sink).unwrap();
    assert_eq!((old.complete, old.incomplete), (448, 64));
    let mut damaged = StorageBackend::memory(image.read_all().unwrap());
    damaged
        .write_at(REGION_LEN as u64, &vec![0; FAILURE_REGION as usize])
        .unwrap();
    let mut sink = Salvaged::default();
    let dense = salvage(&damaged, archive(), mode, &authority, None, &mut sink).unwrap();
    assert_eq!((dense.complete, dense.incomplete), (0, 512));
    let mut one_byte = StorageBackend::memory(image.read_all().unwrap());
    let first = one_byte.read_at(REGION_LEN as u64, 1).unwrap()[0];
    one_byte.write_at(REGION_LEN as u64, &[first ^ 1]).unwrap();
    let mut sink = Salvaged::default();
    let isolated = salvage(&one_byte, archive(), mode, &authority, None, &mut sink).unwrap();
    assert_eq!((isolated.complete, isolated.incomplete), (511, 1));
    for (path, bytes) in &sink.files {
        let index: usize = std::str::from_utf8(path)
            .unwrap()
            .trim_start_matches("file-")
            .parse()
            .unwrap();
        assert_eq!(bytes, &vec![index as u8; 4096]);
    }
    println!("DENSE_REGION_LOSS old_incomplete={} old_logical_bytes={} dense_incomplete={} dense_logical_bytes={} single_byte_incomplete={}", old.incomplete, old.incomplete * 4096, dense.incomplete, dense.incomplete * 4096, isolated.incomplete);
}

#[derive(Clone, Debug)]
struct DenseReadFailure {
    storage: StorageBackend,
    fail_at: u64,
}
impl Storage for DenseReadFailure {
    fn len(&self) -> Result<u64> {
        self.storage.len()
    }
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        if offset == self.fail_at {
            return Err(Error::InvalidInput(
                "synthetic dense recovery I/O failure".into(),
            ));
        }
        self.storage.read_at(offset, len)
    }
    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        out.copy_from_slice(&self.read_at(offset, out.len())?);
        Ok(())
    }
    fn append(&mut self, _: &[u8]) -> Result<u64> {
        panic!("salvage wrote storage")
    }
    fn write_at(&mut self, _: u64, _: &[u8]) -> Result<()> {
        panic!("salvage wrote storage")
    }
    fn truncate(&mut self, _: u64) -> Result<()> {
        panic!("salvage truncated storage")
    }
    fn sync(&self) -> Result<()> {
        panic!("salvage synced storage")
    }
}
#[test]
fn dense_salvage_discards_partial_files_and_propagates_fatal_errors() {
    use super::dense_image::{from_candidate, salvage, Image};
    let mode = mode(true, true, true, true);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let source = Files::create(
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        &authority,
        Some(&owner),
        key(mode),
        65536,
        update_inputs(&[
            (b"empty", b""),
            (b"intact", b"intact contents"),
            (b"partial", &vec![0x91; 150_000]),
        ]),
    )
    .unwrap();
    let mut source = Files::open(source, archive(), mode, &authority, key(mode)).unwrap();
    let image = from_candidate(
        &mut source,
        StorageBackend::memory(Vec::new()),
        &authority,
        Some(&owner),
        key(mode),
        &[],
    )
    .unwrap();
    let (_, body) =
        publication::shared::open_private(&image, archive(), mode, &authority, key(mode)).unwrap();
    let codec = Codec::shared_packed(archive(), mode, key(mode)).unwrap();
    let catalogue =
        super::dense_catalogue::Catalogue::decode(&body, &codec, image.len().unwrap()).unwrap();
    let file = catalogue
        .files
        .iter()
        .find(|file| file.path.as_slice() == b"partial")
        .unwrap();
    let fragment = &file.fragments[1];
    let at = catalogue.packs[fragment.pack].extent.start + fragment.relative as u64;
    let mut damaged = StorageBackend::memory(image.read_all().unwrap());
    let byte = damaged.read_at(at, 1).unwrap()[0];
    damaged.write_at(at, &[byte ^ 1]).unwrap();
    let mut sink = Salvaged::default();
    let report = salvage(&damaged, archive(), mode, &authority, key(mode), &mut sink).unwrap();
    assert_eq!((report.complete, report.incomplete), (2, 1));
    assert_eq!(sink.files[b"empty".as_slice()], b"");
    assert_eq!(sink.files[b"intact".as_slice()], b"intact contents");
    assert!(!sink.files.contains_key(b"partial".as_slice()));
    let mut sink = Salvaged::default();
    assert!(salvage(
        &image,
        archive(),
        mode,
        &authority,
        Some(&[99; 32]),
        &mut sink
    )
    .is_err());
    assert!(sink.files.is_empty() && sink.active.is_none());
    let wrong = OwnerSigningKeyPair::generate().unwrap().public_key();
    assert!(salvage(
        &image,
        archive(),
        mode,
        &Authority::Owner(&wrong),
        key(mode),
        &mut sink
    )
    .is_err());
    assert!(sink.files.is_empty() && sink.active.is_none());
    let guarded = DenseReadFailure {
        storage: StorageBackend::memory(image.read_all().unwrap()),
        fail_at: at,
    };
    assert!(
        matches!(salvage(&guarded, archive(), mode, &authority, key(mode), &mut sink), Err(Error::InvalidInput(ref reason)) if reason == "synthetic dense recovery I/O failure")
    );
    assert_eq!(sink.files.len(), 2); // Previously finished files are still only staged.
    sink.files.clear(); // Caller discards the entire failed batch.
    struct FailingSink;
    impl recovery::Sink for FailingSink {
        fn begin(&mut self, _: &[u8], _: u64) -> Result<()> {
            Ok(())
        }
        fn data(&mut self, _: &[u8]) -> Result<()> {
            Err(Error::InvalidInput("synthetic sink failure".into()))
        }
        fn finish(&mut self, _: bool) -> Result<()> {
            Ok(())
        }
    }
    assert!(
        matches!(salvage(&image, archive(), mode, &authority, key(mode), &mut FailingSink), Err(Error::InvalidInput(ref reason)) if reason == "synthetic sink failure")
    );
    let password = crate::SecretString::try_from_slice(b"no password slot exists").unwrap();
    assert!(matches!(
        Image::open_credential(
            image,
            archive(),
            mode,
            Some(&public),
            publication::bootstrap::Credential::Password(&password),
            None
        ),
        Err(Error::InvalidKey)
    ));
}

#[test]
fn dense_normal_open_checks_unallocated_public_slots_without_blocking_salvage() {
    use super::dense_image::{from_candidate, Image};
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        let mode = mode(encrypted, true, true, true);
        let authority = authority(mode, &public);
        let mut source = Files::open(
            packed_pair(mode, &authority, Some(&owner)),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let mut storage = from_candidate(
            &mut source,
            StorageBackend::memory(Vec::new()),
            &authority,
            Some(&owner),
            key(mode),
            &[],
        )
        .unwrap();
        storage
            .write_at(12288, b"unowned retired key wrapper")
            .unwrap();
        assert!(Image::open(storage.clone(), archive(), mode, &authority, key(mode)).is_err());
        let mut sink = Salvaged::default();
        let report = super::dense_image::salvage(
            &storage,
            archive(),
            mode,
            &authority,
            key(mode),
            &mut sink,
        )
        .unwrap();
        assert_eq!((report.complete, report.incomplete), (2, 0));
        assert_eq!(
            sink.files.get(b"keep".as_slice()).unwrap(),
            &vec![0x39; 8192]
        );
    }
}

#[test]
fn dense_ownership_catalogue_persists_canonical_states_and_preserves_file_bindings() {
    use super::dense_catalogue::Catalogue;
    use super::dense_image::{from_candidate, Image};
    use crate::file_format::publication_anchor::{shared, REGION_LEN};
    use shared::ownership::{Span, Vacant, VacantKind};
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for compressed in [false, true] {
            let mode = mode(encrypted, true, compressed, true);
            let authority = authority(mode, &public);
            let mut source = Files::open(
                packed_pair(mode, &authority, Some(&owner)),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap();
            let storage = from_candidate(
                &mut source,
                StorageBackend::memory(Vec::new()),
                &authority,
                Some(&owner),
                key(mode),
                &[],
            )
            .unwrap();
            let (anchor, body) =
                shared::open_private(&storage, archive(), mode, &authority, key(mode)).unwrap();
            let codec = Codec::shared_packed(archive(), mode, key(mode)).unwrap();
            let mut catalogue = Catalogue::decode(&body, &codec, anchor.sealed_len).unwrap();
            assert!(catalogue.encode(&codec, anchor.sealed_len).is_err());
            catalogue.upgrade(&anchor).unwrap();
            let encoded = catalogue.encode(&codec, anchor.sealed_len).unwrap();
            assert_eq!(&encoded[..8], b"RV4DENS2");
            let decoded = Catalogue::decode(&encoded, &codec, anchor.sealed_len).unwrap();
            assert_eq!(
                decoded
                    .encode(&codec, anchor.sealed_len)
                    .unwrap()
                    .as_slice(),
                encoded.as_slice()
            );
            decoded
                .graph(&anchor)
                .unwrap()
                .verify_free(&storage)
                .unwrap();
            let mut replacement = StorageBackend::memory(storage.read_all().unwrap());
            replacement.write_at(0, &vec![0; REGION_LEN]).unwrap();
            shared::initialize(
                &mut replacement,
                archive(),
                mode,
                &authority,
                Some(&owner),
                key(mode),
                &encoded,
                &[],
            )
            .unwrap();
            let mut image =
                Image::open(replacement, archive(), mode, &authority, key(mode)).unwrap();
            let mut keep = Vec::new();
            image
                .read_range(b"keep", 0, 8192, |bytes| {
                    keep.extend_from_slice(bytes);
                    Ok(())
                })
                .unwrap();
            assert_eq!(keep, vec![0x39; 8192]);
            let descriptor = catalogue.files[0].fragments[0].descriptor.clone();
            catalogue.files[0].fragments[0].descriptor.object[0] ^= 1;
            assert!(catalogue.encode(&codec, anchor.sealed_len).is_err());
            catalogue.files[0].fragments[0].descriptor = descriptor;
            // Persist a pending tail; structural decoding alone does not grant
            // ownership. The selected sealed length and complete graph must agree.
            catalogue.vacant.push(Vacant {
                span: Span {
                    start: anchor.sealed_len,
                    len: 65536,
                },
                kind: VacantKind::Pending,
            });
            let body = catalogue.encode(&codec, anchor.sealed_len + 65536).unwrap();
            let decoded = Catalogue::decode(&body, &codec, anchor.sealed_len + 65536).unwrap();
            assert!(decoded.graph(&anchor).is_err());
            let mut grown = anchor.clone();
            grown.sealed_len += 65536;
            decoded.graph(&grown).unwrap();
            assert!(Catalogue::decode(&body[..body.len() - 1], &codec, grown.sealed_len).is_err());
            let mut extra = body.to_vec();
            extra.push(0);
            assert!(Catalogue::decode(&extra, &codec, grown.sealed_len).is_err());
        }
    }
}

fn dense_seed(
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
) -> StorageBackend {
    let mut source = Files::open(
        packed_pair(mode, authority, signer),
        archive(),
        mode,
        authority,
        key(mode),
    )
    .unwrap();
    super::dense_image::from_candidate(
        &mut source,
        StorageBackend::memory(Vec::new()),
        authority,
        signer,
        key(mode),
        &[],
    )
    .unwrap()
}
fn check_dense_edit(
    storage: &StorageBackend,
    mode: FormatMode,
    authority: &Authority<'_>,
    old: &[u8],
    new: &[u8],
) -> bool {
    use crate::file_format::publication_anchor::shared;
    let (anchor, body) =
        shared::open_private(storage, archive(), mode, authority, key(mode)).unwrap();
    let codec = Codec::shared_packed(archive(), mode, key(mode)).unwrap();
    let catalogue =
        super::dense_catalogue::Catalogue::decode(&body, &codec, anchor.sealed_len).unwrap();
    catalogue
        .graph(&anchor)
        .unwrap()
        .verify_reclaimed(storage)
        .unwrap();
    let changed = catalogue
        .files
        .iter()
        .any(|file| file.path.as_slice() == new);
    assert_ne!(
        changed,
        catalogue
            .files
            .iter()
            .any(|file| file.path.as_slice() == old)
    );
    let name = if changed { new } else { old };
    let file = catalogue
        .files
        .iter()
        .find(|file| file.path.as_slice() == name)
        .unwrap();
    assert_eq!(file.permissions, if changed { 0o600 } else { 0o644 });
    let mut image =
        super::dense_image::Image::open(storage.clone(), archive(), mode, authority, key(mode))
            .unwrap();
    let mut bytes = Vec::new();
    image
        .read_range(name, 0, 4096, |part| {
            bytes.extend_from_slice(part);
            Ok(())
        })
        .unwrap();
    assert_eq!(bytes, vec![0xa7; 4096]);
    let mut bytes = Vec::new();
    image
        .read_range(b"keep", 0, 8192, |part| {
            bytes.extend_from_slice(part);
            Ok(())
        })
        .unwrap();
    assert_eq!(bytes, vec![0x39; 8192]);
    changed
}
#[test]
fn dense_metadata_edits_reopen_with_bounded_control_growth_in_all_modes() {
    use crate::file_format::publication_anchor::{shared, REGION_LEN};
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                for padded in [false, true] {
                    let mode = mode(encrypted, signed, compressed, padded);
                    let authority = authority(mode, &public);
                    let signer = signed.then_some(&owner);
                    let mut storage = dense_seed(mode, &authority, signer);
                    let (anchor, _) =
                        shared::open_private(&storage, archive(), mode, &authority, key(mode))
                            .unwrap();
                    let payload = storage
                        .read_at(
                            REGION_LEN as u64,
                            (anchor.sealed_len - REGION_LEN as u64) as usize,
                        )
                        .unwrap();
                    let mut high_water = 0;
                    for cycle in 0..6 {
                        let (from, to, permission) = if cycle % 2 == 0 {
                            (b"erase".as_slice(), b"renamed".as_slice(), 0o600)
                        } else {
                            (b"renamed".as_slice(), b"erase".as_slice(), 0o644)
                        };
                        assert!(super::dense_update::edit(
                            &mut storage,
                            archive(),
                            mode,
                            &authority,
                            signer,
                            key(mode),
                            from,
                            to,
                            Some(permission)
                        )
                        .unwrap());
                        if cycle % 2 == 0 {
                            assert!(check_dense_edit(
                                &storage, mode, &authority, b"erase", b"renamed"
                            ));
                        } else {
                            assert!(!check_dense_edit(
                                &storage, mode, &authority, b"erase", b"renamed"
                            ));
                        }
                        let (next, _) =
                            shared::open_private(&storage, archive(), mode, &authority, key(mode))
                                .unwrap();
                        assert_eq!(next.generation, cycle + 2);
                        assert_eq!(next.index.primary < REGION_LEN as u64, cycle % 2 == 1);
                        if cycle == 0 {
                            high_water = next.sealed_len;
                        } else {
                            assert_eq!(next.sealed_len, high_water);
                        }
                        assert_eq!(
                            storage.read_at(REGION_LEN as u64, payload.len()).unwrap(),
                            payload
                        );
                        let before = storage.read_all().unwrap();
                        assert!(!super::dense_update::edit(
                            &mut storage,
                            archive(),
                            mode,
                            &authority,
                            signer,
                            key(mode),
                            to,
                            to,
                            Some(permission)
                        )
                        .unwrap());
                        assert_eq!(storage.read_all().unwrap(), before);
                    }
                    let before = storage.read_all().unwrap();
                    for (from, to, bits) in [
                        (b"erase".as_slice(), b"keep".as_slice(), None),
                        (b"missing", b"new", None),
                        (b"erase", b"", None),
                        (b"erase", b"new", Some(0xffff)),
                    ] {
                        assert!(super::dense_update::edit(
                            &mut storage,
                            archive(),
                            mode,
                            &authority,
                            signer,
                            key(mode),
                            from,
                            to,
                            bits
                        )
                        .is_err());
                        assert_eq!(storage.read_all().unwrap(), before);
                    }
                }
            }
        }
    }
}
#[test]
fn dense_metadata_edits_recover_every_returned_storage_failure() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for mode in [
        mode(false, false, false, false),
        mode(false, true, true, true),
        mode(true, false, false, true),
        mode(true, true, true, true),
    ] {
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut seed = dense_seed(mode, &authority, signer);
        for reverse in [false, true] {
            let (from, to, bits) = if reverse {
                (b"renamed".as_slice(), b"erase".as_slice(), 0o644)
            } else {
                (b"erase".as_slice(), b"renamed".as_slice(), 0o600)
            };
            let observed = SharedMemory::new(seed.read_all().unwrap());
            let mut store = observed.clone();
            super::dense_update::edit(
                &mut store,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                from,
                to,
                Some(bits),
            )
            .unwrap();
            for at in 0..observed.operations() {
                let mut failed = SharedMemory::new(seed.read_all().unwrap());
                failed.fail(at);
                let _ = super::dense_update::edit(
                    &mut failed,
                    archive(),
                    mode,
                    &authority,
                    signer,
                    key(mode),
                    from,
                    to,
                    Some(bits),
                );
                super::dense_update::recover(&mut failed, archive(), mode, &authority, key(mode))
                    .unwrap_or_else(|error| panic!("reverse={reverse} at={at}: {error}"));
                check_dense_edit(
                    &StorageBackend::memory(failed.read_all().unwrap()),
                    mode,
                    &authority,
                    b"erase",
                    b"renamed",
                );
                cases += 1;
            }
            seed = StorageBackend::memory(observed.read_all().unwrap());
        }
    }
    println!("DENSE_METADATA_RETURNED_FAILURES {cases}");
}
#[test]
fn dense_metadata_edits_survive_volatile_writes_torn_records_and_failed_sync() {
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for mode in [
        mode(false, false, false, false),
        mode(false, true, true, true),
        mode(true, false, false, true),
        mode(true, true, true, true),
    ] {
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut seed = dense_seed(mode, &authority, signer).read_all().unwrap();
        for reverse in [false, true] {
            let (from, to, bits) = if reverse {
                (b"renamed".as_slice(), b"erase".as_slice(), 0o644)
            } else {
                (b"erase".as_slice(), b"renamed".as_slice(), 0o600)
            };
            let mut observed = CrashStore::new(seed.clone(), None, 0, false);
            super::dense_update::edit(
                &mut observed,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                from,
                to,
                Some(bits),
            )
            .unwrap();
            for at in 0..observed.operations() {
                for prefix in [0, 97, usize::MAX] {
                    for persist_sync in [false, true] {
                        let mut failed =
                            CrashStore::new(seed.clone(), Some(at), prefix, persist_sync);
                        let _ = super::dense_update::edit(
                            &mut failed,
                            archive(),
                            mode,
                            &authority,
                            signer,
                            key(mode),
                            from,
                            to,
                            Some(bits),
                        );
                        let mut recovered = StorageBackend::memory(failed.durable());
                        super::dense_update::recover(&mut recovered,archive(),mode,&authority,key(mode)).unwrap_or_else(|error|panic!("reverse={reverse} at={at} prefix={prefix} sync={persist_sync}: {error}"));
                        check_dense_edit(&recovered, mode, &authority, b"erase", b"renamed");
                        cases += 1;
                    }
                }
            }
            seed = observed.durable();
        }
    }
    println!("DENSE_METADATA_POWER_LOSS_CASES {cases}");
}

#[test]
fn dense_recovery_refuses_forged_cleanup_of_live_or_fixed_bytes() {
    use crate::file_format::preparation_journal::compact::session::InlineSession;
    use crate::file_format::preparation_journal::{Reservation, FREE};
    use crate::file_format::publication_anchor::shared;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        let mode = mode(encrypted, true, true, true);
        let authority = authority(mode, &public);
        let seed = dense_seed(mode, &authority, Some(&owner));
        let (anchor, _) =
            shared::open_private(&seed, archive(), mode, &authority, key(mode)).unwrap();
        for start in [0, 8192, 16384, 131072, anchor.sealed_len] {
            let mut storage = StorageBackend::memory(seed.read_all().unwrap());
            let mut journal = InlineSession::open(&storage, archive(), mode, key(mode)).unwrap();
            journal
                .begin(
                    &mut storage,
                    shared::commitment(&anchor).unwrap(),
                    vec![Reservation {
                        namespace: FREE,
                        base: start,
                        start,
                        len: 16,
                    }],
                )
                .unwrap();
            let before = storage.read_all().unwrap();
            assert!(super::dense_update::recover(
                &mut storage,
                archive(),
                mode,
                &authority,
                key(mode)
            )
            .is_err());
            assert_eq!(storage.read_all().unwrap(), before);
        }
        let wrong = OwnerSigningKeyPair::generate().unwrap();
        let mut storage = StorageBackend::memory(seed.read_all().unwrap());
        let before = storage.read_all().unwrap();
        assert!(super::dense_update::edit(
            &mut storage,
            archive(),
            mode,
            &authority,
            Some(&wrong),
            key(mode),
            b"erase",
            b"renamed",
            None
        )
        .is_err());
        assert_eq!(storage.read_all().unwrap(), before);
        assert!(super::dense_update::edit(
            &mut storage,
            archive(),
            mode,
            &authority,
            None,
            key(mode),
            b"erase",
            b"renamed",
            None
        )
        .is_err());
        assert_eq!(storage.read_all().unwrap(), before);
    }
}

#[test]
fn dense_recovery_itself_survives_interruption_in_abort_and_commit_cleanup() {
    use crate::file_format::preparation_journal::compact::session::InlineSession;
    use crate::file_format::preparation_journal::tests::CrashStore;
    use crate::file_format::publication_anchor::shared;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for mode in [
        mode(false, false, false, false),
        mode(false, true, true, true),
        mode(true, false, false, true),
        mode(true, true, true, true),
    ] {
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut seed = dense_seed(mode, &authority, signer).read_all().unwrap();
        for reverse in [false, true] {
            let (from, to, bits) = if reverse {
                (b"renamed".as_slice(), b"erase".as_slice(), 0o644)
            } else {
                (b"erase".as_slice(), b"renamed".as_slice(), 0o600)
            };
            let mut observed = CrashStore::new(seed.clone(), None, 0, false);
            super::dense_update::edit(
                &mut observed,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                from,
                to,
                Some(bits),
            )
            .unwrap();
            let mut checkpoints = [None, None];
            for at in 0..observed.operations() {
                let mut failed = CrashStore::new(seed.clone(), Some(at), usize::MAX, true);
                let _ = super::dense_update::edit(
                    &mut failed,
                    archive(),
                    mode,
                    &authority,
                    signer,
                    key(mode),
                    from,
                    to,
                    Some(bits),
                );
                let bytes = failed.durable();
                let state = StorageBackend::memory(bytes.clone());
                let journal = InlineSession::open(&state, archive(), mode, key(mode)).unwrap();
                if !journal.active() {
                    continue;
                }
                let (anchor, _) =
                    shared::snapshot(&state, archive(), mode, &authority, key(mode)).unwrap();
                let committed = journal.base() != shared::commitment(&anchor).unwrap();
                checkpoints[usize::from(committed)] = Some(bytes);
            }
            for (committed, checkpoint) in checkpoints.into_iter().enumerate() {
                let checkpoint = checkpoint.expect("both abort and commit cleanup checkpoints");
                let mut observed = CrashStore::new(checkpoint.clone(), None, 0, false);
                super::dense_update::recover(&mut observed, archive(), mode, &authority, key(mode))
                    .unwrap();
                for at in 0..observed.operations() {
                    for prefix in [0, 97, usize::MAX] {
                        for persist_sync in [false, true] {
                            let mut failed =
                                CrashStore::new(checkpoint.clone(), Some(at), prefix, persist_sync);
                            let _ = super::dense_update::recover(
                                &mut failed,
                                archive(),
                                mode,
                                &authority,
                                key(mode),
                            );
                            let mut recovered = StorageBackend::memory(failed.durable());
                            super::dense_update::recover(&mut recovered,archive(),mode,&authority,key(mode)).unwrap_or_else(|error|panic!("reverse={reverse} committed={committed} recovery_at={at}: {error}"));
                            let changed = check_dense_edit(
                                &recovered, mode, &authority, b"erase", b"renamed",
                            );
                            assert_eq!(changed, (committed == 1) != reverse);
                            cases += 1;
                        }
                    }
                }
            }
            seed = observed.durable();
        }
    }
    println!("DENSE_METADATA_RECOVERY_INTERRUPTION_CASES {cases}");
}
#[test]
fn dense_metadata_updates_keep_authority_after_either_control_or_external_root_region_is_lost() {
    use crate::file_format::publication_anchor::{shared, FAILURE_REGION};
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for mode in [
        mode(false, false, false, false),
        mode(false, true, true, true),
        mode(true, false, false, true),
        mode(true, true, true, true),
    ] {
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = dense_seed(mode, &authority, signer);
        let mut regions = vec![0, FAILURE_REGION];
        for reverse in [false, true] {
            let (from, to, bits) = if reverse {
                (b"renamed".as_slice(), b"erase".as_slice(), 0o644)
            } else {
                (b"erase".as_slice(), b"renamed".as_slice(), 0o600)
            };
            super::dense_update::edit(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                from,
                to,
                Some(bits),
            )
            .unwrap();
            let (anchor, _) =
                shared::snapshot(&storage, archive(), mode, &authority, key(mode)).unwrap();
            if !reverse {
                regions.extend([anchor.index.primary, anchor.index.mirror]);
            }
            for start in &regions {
                let mut damaged = StorageBackend::memory(storage.read_all().unwrap());
                damaged
                    .write_at(*start, &vec![0; FAILURE_REGION as usize])
                    .unwrap();
                // Ordinary read and writable recovery both retain this generation.
                assert_eq!(
                    check_dense_edit(&damaged, mode, &authority, b"erase", b"renamed"),
                    !reverse
                );
                let reopened = super::dense_update::recover(
                    &mut damaged,
                    archive(),
                    mode,
                    &authority,
                    key(mode),
                )
                .unwrap();
                assert_eq!(reopened, anchor);
                assert_eq!(
                    check_dense_edit(&damaged, mode, &authority, b"erase", b"renamed"),
                    !reverse
                );
                cases += 1;
            }
        }
    }
    println!("DENSE_METADATA_SEPARATE_REGION_LOSS_CASES {cases}");
}

#[test]
fn dense_metadata_file_updates_close_and_reopen_with_persisted_permissions_and_contents() {
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let mut random = [0; 8];
    getrandom::fill(&mut random).unwrap();
    let cleanup = Cleanup(std::env::temp_dir().join(format!(
        "revault-dense-metadata-{}-{}",
        std::process::id(),
        u64::from_le_bytes(random)
    )));
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(true, true, true, true);
    let authority = authority(mode, &public);
    let seed = dense_seed(mode, &authority, Some(&owner));
    let storage = StorageBackend::create_file(&cleanup.0, &seed.read_all().unwrap()).unwrap();
    storage.sync().unwrap();
    drop(storage);
    for reverse in [false, true] {
        let (from, to, bits) = if reverse {
            (b"renamed".as_slice(), b"erase".as_slice(), 0o644)
        } else {
            (b"erase".as_slice(), b"renamed".as_slice(), 0o600)
        };
        let mut writable = StorageBackend::file_for_write(&cleanup.0).unwrap();
        super::dense_update::edit(
            &mut writable,
            archive(),
            mode,
            &authority,
            Some(&owner),
            key(mode),
            from,
            to,
            Some(bits),
        )
        .unwrap();
        drop(writable);
        let reader = StorageBackend::file(&cleanup.0).unwrap();
        assert_eq!(
            check_dense_edit(&reader, mode, &authority, b"erase", b"renamed"),
            !reverse
        );
        drop(reader);
    }
}

#[test]
fn paged_cost_model_preserves_source_and_budgets_actual_hybrid_publication_bytes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                for padded in [false, true] {
                    let mode = mode(encrypted, signed, compressed, padded);
                    let authority = authority(mode, &public);
                    let mut files = Files::open(
                        packed_pair(mode, &authority, signed.then_some(&owner)),
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap();
                    let before = files.storage.read_all().unwrap();
                    let result =
                        super::paged_cost::project(&mut files, &authority, key(mode)).unwrap();
                    assert_eq!(result["inline_geometry_fits"], true);
                    assert_eq!(result["frame_roundtrips_verified"], true);
                    assert_eq!(result["file_leaf_pages"], 1);
                    assert_eq!(result["pack_leaf_pages"], 1);
                    assert!(result["peak_leaf_bytes_per_bank"].as_u64().unwrap() <= 49152);
                    assert!(
                        result["largest_edited_root_required_bytes"]
                            .as_u64()
                            .unwrap()
                            <= 2016
                    );
                    let auth = result["authenticated_owner_or_mac_bytes"].as_u64().unwrap();
                    if signed {
                        assert!(auth > 4096 && auth + 320 <= 6144);
                    } else {
                        assert_eq!(auth, if encrypted { 32 } else { 0 });
                    }
                    let variants = result["page_granularity_comparison"].as_array().unwrap();
                    assert_eq!(variants.len(), 4);
                    for (variant, count) in variants.iter().zip([16, 32, 64, 128]) {
                        assert_eq!(variant["max_records_per_leaf"], count);
                        assert_eq!(variant["frame_roundtrips_verified"], true);
                        assert!(variant["largest_leaf_decoded_bytes"].as_u64().unwrap() <= 16384);
                        assert!(
                            variant["leaf_unrounded_frame_bytes_per_bank"]
                                .as_u64()
                                .unwrap()
                                <= variant["live_leaf_bytes_per_bank"].as_u64().unwrap()
                        );
                    }
                    assert_eq!(files.storage.read_all().unwrap(), before);
                }
            }
        }
    }
}

#[test]
fn paged_cost_model_keeps_large_pack_tables_out_of_the_embedded_root() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for mode in [
        mode(false, false, false, true),
        mode(true, true, false, true),
    ] {
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let storage = Files::create(
            StorageBackend::memory(Vec::new()),
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            65536,
            [Input {
                path: b"large.bin".to_vec(),
                reader: Pattern {
                    position: 0,
                    len: 8 * 1024 * 1024,
                    fail: None,
                },
            }],
        )
        .unwrap();
        let mut files = Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
        let before = Sha256::digest(files.storage.read_all().unwrap());
        let result = super::paged_cost::project(&mut files, &authority, key(mode)).unwrap();
        assert_eq!(result["inline_geometry_fits"], true);
        assert_eq!(result["file_leaf_pages"], 1);
        assert!(result["pack_leaf_pages"].as_u64().unwrap() >= 4);
        assert!(
            result["largest_edited_root_required_bytes"]
                .as_u64()
                .unwrap()
                <= 2016
        );
        assert_eq!(Sha256::digest(files.storage.read_all().unwrap()), before);
    }
}

#[test]
fn paged_cost_model_handles_two_byte_leaf_counts() {
    let mode = mode(true, false, true, true);
    let authority = Authority::Symmetric(KEY);
    let storage = Files::create(
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        &authority,
        None,
        key(mode),
        65536,
        (0..128).map(|index| Input {
            path: format!("empty-{index:03}").into_bytes(),
            reader: Cursor::new(Vec::<u8>::new()),
        }),
    )
    .unwrap();
    let mut files = Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
    let before = files.storage.read_all().unwrap();
    let result = super::paged_cost::project(&mut files, &authority, key(mode)).unwrap();
    for (variant, pages) in result["page_granularity_comparison"]
        .as_array()
        .unwrap()
        .iter()
        .zip([8, 4, 2, 1])
    {
        assert_eq!(variant["file_leaf_pages"], pages);
        assert_eq!(variant["pack_leaf_pages"], 0);
        assert_eq!(variant["frame_roundtrips_verified"], true);
    }
    assert_eq!(files.storage.read_all().unwrap(), before);
}
// Internal prototype tests: no public CLI writer supports this control layout.
#[test]
fn dense_metadata_tail_returns_to_original_size_and_preserves_bytes_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                for padded in [false, true] {
                    let mode = mode(encrypted, signed, compressed, padded);
                    let authority = authority(mode, &public);
                    let signer = signed.then_some(&owner);
                    let mut storage = dense_seed(mode, &authority, signer);
                    let original = storage.len().unwrap();
                    for cycle in 0..4 {
                        let (from, to, bits) = if cycle % 2 == 0 {
                            (b"erase".as_slice(), b"renamed".as_slice(), 0o600)
                        } else {
                            (b"renamed".as_slice(), b"erase".as_slice(), 0o644)
                        };
                        super::dense_update::edit(
                            &mut storage,
                            archive(),
                            mode,
                            &authority,
                            signer,
                            key(mode),
                            from,
                            to,
                            Some(bits),
                        )
                        .unwrap();
                        assert!(storage.len().unwrap() > original);
                        assert!(super::dense_update::return_inline(
                            &mut storage,
                            archive(),
                            mode,
                            &authority,
                            signer,
                            key(mode)
                        )
                        .unwrap());
                        assert_eq!(storage.len().unwrap(), original);
                        assert_eq!(
                            check_dense_edit(&storage, mode, &authority, b"erase", b"renamed"),
                            cycle % 2 == 0
                        );
                        let before = storage.read_all().unwrap();
                        assert!(!super::dense_update::return_inline(
                            &mut storage,
                            archive(),
                            mode,
                            &authority,
                            signer,
                            key(mode)
                        )
                        .unwrap());
                        assert_eq!(storage.read_all().unwrap(), before);
                    }
                }
            }
        }
    }
}

#[test]
fn dense_metadata_tail_recovers_returned_failures_and_power_loss() {
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut returned = 0;
    let mut crashes = 0;
    for mode in [
        mode(false, false, false, false),
        mode(false, true, true, true),
        mode(true, false, false, true),
        mode(true, true, true, true),
    ] {
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = dense_seed(mode, &authority, signer);
        let original = storage.len().unwrap();
        super::dense_update::edit(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            b"erase",
            b"renamed",
            Some(0o600),
        )
        .unwrap();
        let seed = storage.read_all().unwrap();
        let mut observed = SharedMemory::new(seed.clone());
        super::dense_update::return_inline(
            &mut observed,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
        )
        .unwrap();
        for at in 0..observed.operations() {
            let mut failed = SharedMemory::new(seed.clone());
            failed.fail(at);
            let _ = super::dense_update::return_inline(
                &mut failed,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
            );
            super::dense_update::recover(&mut failed, archive(), mode, &authority, key(mode))
                .unwrap_or_else(|e| panic!("returned at={at}: {e}"));
            let mut reopened = StorageBackend::memory(failed.read_all().unwrap());
            assert!(check_dense_edit(
                &reopened, mode, &authority, b"erase", b"renamed"
            ));
            super::dense_update::return_inline(
                &mut reopened,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
            )
            .unwrap();
            assert_eq!(reopened.len().unwrap(), original);
            returned += 1;
        }
        let mut observed = CrashStore::new(seed.clone(), None, 0, false);
        super::dense_update::return_inline(
            &mut observed,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
        )
        .unwrap();
        for at in 0..observed.operations() {
            for prefix in [0, 97, usize::MAX] {
                for persist_sync in [false, true] {
                    let mut failed = CrashStore::new(seed.clone(), Some(at), prefix, persist_sync);
                    let _ = super::dense_update::return_inline(
                        &mut failed,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                    );
                    let mut reopened = StorageBackend::memory(failed.durable());
                    super::dense_update::recover(
                        &mut reopened,
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap_or_else(|e| {
                        panic!("crash at={at} prefix={prefix} sync={persist_sync}: {e}")
                    });
                    assert!(check_dense_edit(
                        &reopened, mode, &authority, b"erase", b"renamed"
                    ));
                    super::dense_update::return_inline(
                        &mut reopened,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                    )
                    .unwrap();
                    assert_eq!(reopened.len().unwrap(), original);
                    crashes += 1;
                }
            }
        }
    }
    println!("DENSE_METADATA_TAIL_FAILURES returned={returned} power_loss={crashes}");
}

#[test]
fn dense_metadata_tail_cleanup_resumes_after_interruption_and_bank_loss() {
    use crate::file_format::preparation_journal::compact::session::InlineSession;
    use crate::file_format::preparation_journal::tests::CrashStore;
    use crate::file_format::publication_anchor::{shared, FAILURE_REGION};
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(true, true, true, true);
    let authority = authority(mode, &public);
    let mut seed = dense_seed(mode, &authority, Some(&owner));
    let original = seed.len().unwrap();
    super::dense_update::edit(
        &mut seed,
        archive(),
        mode,
        &authority,
        Some(&owner),
        key(mode),
        b"erase",
        b"renamed",
        Some(0o600),
    )
    .unwrap();
    let seed = seed.read_all().unwrap();
    let mut observed = CrashStore::new(seed.clone(), None, 0, false);
    super::dense_update::return_inline(
        &mut observed,
        archive(),
        mode,
        &authority,
        Some(&owner),
        key(mode),
    )
    .unwrap();
    let mut checkpoints = [None, None, None];
    for at in 0..observed.operations() {
        let mut failed = CrashStore::new(seed.clone(), Some(at), usize::MAX, true);
        let _ = super::dense_update::return_inline(
            &mut failed,
            archive(),
            mode,
            &authority,
            Some(&owner),
            key(mode),
        );
        let bytes = failed.durable();
        let state = StorageBackend::memory(bytes.clone());
        let journal = InlineSession::open(&state, archive(), mode, key(mode)).unwrap();
        if !journal.active() {
            continue;
        }
        let (anchor, _) = shared::snapshot(&state, archive(), mode, &authority, key(mode)).unwrap();
        let committed = journal.base() != shared::commitment(&anchor).unwrap();
        let stage = if !committed {
            0
        } else if bytes.len() as u64 > anchor.sealed_len {
            1
        } else {
            2
        };
        if checkpoints[stage].is_none() {
            checkpoints[stage] = Some(bytes);
        }
    }
    let mut cases = 0;
    for (stage, checkpoint) in checkpoints.into_iter().enumerate() {
        let checkpoint =
            checkpoint.expect("abort, committed tail and truncated active checkpoints");
        let mut observed = CrashStore::new(checkpoint.clone(), None, 0, false);
        super::dense_update::recover(&mut observed, archive(), mode, &authority, key(mode))
            .unwrap();
        for at in 0..observed.operations() {
            for prefix in [0, 97, usize::MAX] {
                for persist_sync in [false, true] {
                    let mut failed =
                        CrashStore::new(checkpoint.clone(), Some(at), prefix, persist_sync);
                    let _ = super::dense_update::recover(
                        &mut failed,
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    );
                    let mut reopened = StorageBackend::memory(failed.durable());
                    super::dense_update::recover(
                        &mut reopened,
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap_or_else(|e| {
                        panic!("stage={stage} at={at} prefix={prefix} sync={persist_sync}: {e}")
                    });
                    assert!(check_dense_edit(
                        &reopened, mode, &authority, b"erase", b"renamed"
                    ));
                    if stage > 0 {
                        assert_eq!(reopened.len().unwrap(), original);
                    }
                    cases += 1;
                }
            }
        }
        for bank in [0, FAILURE_REGION] {
            let mut damaged = StorageBackend::memory(checkpoint.clone());
            damaged
                .write_at(bank, &vec![0; FAILURE_REGION as usize])
                .unwrap();
            super::dense_update::recover(&mut damaged, archive(), mode, &authority, key(mode))
                .unwrap();
            assert!(check_dense_edit(
                &damaged, mode, &authority, b"erase", b"renamed"
            ));
        }
    }
    println!("DENSE_METADATA_TAIL_RECOVERY_INTERRUPTION_CASES {cases}; bank_loss=6");
}

// Public APIs construct the logical source. Internal candidate storage is needed
// because this experimental image still has no public CLI constructor.
fn public_filesystem_metadata(
    owner: &OwnerSigningKeyPair,
) -> Vec<super::dense_catalogue::Metadata> {
    use crate::{
        ListOptions, Lockbox, LockboxEntryKind, LockboxPath, LockboxProtection, SecretString,
    };
    let password = SecretString::try_from_slice(b"synthetic typed source password").unwrap();
    let mut source =
        Lockbox::create_in_memory(LockboxProtection::Password(&password), owner).unwrap();
    source
        .add_file_with_permissions(
            &LockboxPath::new("/docs/data").unwrap(),
            b"payload",
            0o640,
            false,
        )
        .unwrap();
    source
        .set_permissions(&LockboxPath::new("/docs").unwrap(), 0o750)
        .unwrap();
    source
        .create_dir(&LockboxPath::new("/empty").unwrap(), false)
        .unwrap();
    source
        .add_symlink(
            &LockboxPath::new("/link").unwrap(),
            &LockboxPath::new("/docs/data").unwrap(),
            false,
        )
        .unwrap();
    source
        .set_permissions(&LockboxPath::new("/link").unwrap(), 0o700)
        .unwrap();
    source.commit().unwrap();
    assert_eq!(
        source
            .read_file_range(&LockboxPath::new("/docs/data").unwrap(), 0, 7)
            .unwrap(),
        b"payload"
    );
    let mut options = ListOptions::new(&LockboxPath::new("/").unwrap());
    options.recursive = true;
    source
        .list(options)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            let target = (entry.kind == LockboxEntryKind::Symlink)
                .then(|| source.get_symlink_target(&entry.path).unwrap());
            super::dense_catalogue::Metadata { entry, target }
        })
        .collect()
}
fn canonical_dense_seed(
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
) -> StorageBackend {
    let storage = Files::create(
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        authority,
        signer,
        key(mode),
        65536,
        [Input {
            path: b"/docs/data".to_vec(),
            reader: Cursor::new(b"payload".to_vec()),
        }],
    )
    .unwrap();
    let mut source = Files::open(storage, archive(), mode, authority, key(mode)).unwrap();
    super::dense_image::from_candidate(
        &mut source,
        StorageBackend::memory(Vec::new()),
        authority,
        signer,
        key(mode),
        &[],
    )
    .unwrap()
}
fn check_filesystem_snapshot(
    storage: &StorageBackend,
    mode: FormatMode,
    authority: &Authority<'_>,
    expected: &[super::dense_catalogue::Metadata],
) {
    let mut image =
        super::dense_image::Image::open(storage.clone(), archive(), mode, authority, key(mode))
            .unwrap();
    assert_eq!(image.filesystem_metadata().unwrap(), expected);
    let mut bytes = Vec::new();
    image
        .read_range(b"/docs/data", 0, 7, |part| {
            bytes.extend_from_slice(part);
            Ok(())
        })
        .unwrap();
    assert_eq!(bytes, b"payload");
}
#[test]
fn dense_filesystem_metadata_preserves_public_nodes_permissions_and_no_change_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let entries = public_filesystem_metadata(&owner);
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                for padded in [false, true] {
                    let mode = mode(encrypted, signed, compressed, padded);
                    let authority = authority(mode, &public);
                    let signer = signed.then_some(&owner);
                    let mut storage = canonical_dense_seed(mode, &authority, signer);
                    let size = storage.len().unwrap();
                    assert!(super::dense_update::replace_filesystem_metadata(
                        &mut storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        &entries
                    )
                    .unwrap());
                    check_filesystem_snapshot(&storage, mode, &authority, &entries);
                    assert!(super::dense_update::return_inline(
                        &mut storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode)
                    )
                    .unwrap());
                    assert_eq!(storage.len().unwrap(), size);
                    check_filesystem_snapshot(&storage, mode, &authority, &entries);
                    let before = storage.read_all().unwrap();
                    assert!(!super::dense_update::replace_filesystem_metadata(
                        &mut storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        &entries
                    )
                    .unwrap());
                    assert_eq!(storage.read_all().unwrap(), before);
                    let mut next = entries.clone();
                    next.retain(|row| row.entry.path.as_str() != "/empty");
                    next.iter_mut()
                        .find(|row| row.target.is_some())
                        .unwrap()
                        .target = Some(crate::LockboxPath::new("/missing").unwrap());
                    next.iter_mut()
                        .find(|row| row.entry.path.as_str() == "/docs/data")
                        .unwrap()
                        .entry
                        .permissions = 0o600;
                    super::dense_update::replace_filesystem_metadata(
                        &mut storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        &next,
                    )
                    .unwrap();
                    super::dense_update::return_inline(
                        &mut storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                    )
                    .unwrap();
                    assert_eq!(storage.len().unwrap(), size);
                    check_filesystem_snapshot(&storage, mode, &authority, &next);
                    for bank in [0, publication::FAILURE_REGION] {
                        let mut damaged = StorageBackend::memory(storage.read_all().unwrap());
                        damaged
                            .write_at(bank, &vec![0; publication::FAILURE_REGION as usize])
                            .unwrap();
                        // Salvage must use the latest authenticated node membership,
                        // without resurrecting the removed directory or old target.
                        let mut sink = FilesystemSalvaged::default();
                        let report = super::dense_image::salvage_filesystem(
                            &damaged,
                            archive(),
                            mode,
                            &authority,
                            key(mode),
                            &mut sink,
                        )
                        .unwrap();
                        assert_eq!(sink.metadata.as_deref(), Some(next.as_slice()));
                        assert_eq!((report.complete, report.incomplete), (1, 0));
                        assert_eq!(sink.files.files[b"/docs/data".as_slice()], b"payload");
                        super::dense_update::recover(
                            &mut damaged,
                            archive(),
                            mode,
                            &authority,
                            key(mode),
                        )
                        .unwrap();
                        check_filesystem_snapshot(&damaged, mode, &authority, &next);
                    }
                    let mut sink = Salvaged::default();
                    assert!(matches!(
                        super::dense_image::salvage(
                            &storage,
                            archive(),
                            mode,
                            &authority,
                            key(mode),
                            &mut sink
                        ),
                        Err(Error::InvalidOperation(_))
                    ));
                    assert!(sink.files.is_empty()); // File-only sink must not silently drop typed nodes.
                }
            }
        }
    }
}
#[test]
fn dense_filesystem_metadata_refuses_ambiguous_or_incomplete_snapshots_before_writes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let entries = public_filesystem_metadata(&owner);
    let mode = mode(true, true, true, true);
    let authority = authority(mode, &public);
    let mut storage = canonical_dense_seed(mode, &authority, Some(&owner));
    let before = storage.read_all().unwrap();
    let mut invalid = Vec::new();
    let mut value = entries.clone();
    value.retain(|row| row.entry.path.as_str() != "/docs");
    invalid.push(value);
    let mut value = entries.clone();
    value.retain(|row| row.entry.path.as_str() != "/docs/data");
    invalid.push(value);
    let mut value = entries.clone();
    value.push(value[0].clone());
    invalid.push(value);
    let mut value = entries.clone();
    value[0].entry.permissions = 0o4755;
    invalid.push(value);
    let mut value = entries.clone();
    value
        .iter_mut()
        .find(|row| row.entry.len > 0)
        .unwrap()
        .entry
        .len += 1;
    invalid.push(value);
    let mut value = entries.clone();
    value
        .iter_mut()
        .find(|row| row.target.is_some())
        .unwrap()
        .target = None;
    invalid.push(value);
    let mut value = entries.clone();
    value[0].entry.path = crate::LockboxPath::from_unchecked_for_test("/bad/../path");
    invalid.push(value);
    let mut value = entries.clone();
    value[0].entry.path = crate::LockboxPath::from_unchecked_for_test("/e\u{301}");
    invalid.push(value);
    for rows in invalid {
        assert!(super::dense_update::replace_filesystem_metadata(
            &mut storage,
            archive(),
            mode,
            &authority,
            Some(&owner),
            key(mode),
            &rows
        )
        .is_err());
        assert_eq!(storage.read_all().unwrap(), before);
    }
    assert!(super::dense_update::edit(
        &mut storage,
        archive(),
        mode,
        &authority,
        Some(&owner),
        key(mode),
        b"/docs/data",
        b"/docs/data",
        Some(0o4755)
    )
    .is_err());
    assert_eq!(storage.read_all().unwrap(), before);
}
#[test]
fn dense_filesystem_metadata_recovery_selects_complete_old_or_typed_new_state() {
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let entries = public_filesystem_metadata(&owner);
    let mut cases = 0;
    for mode in [
        mode(false, false, false, false),
        mode(false, true, true, true),
        mode(true, false, false, true),
        mode(true, true, true, true),
    ] {
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let seed = canonical_dense_seed(mode, &authority, signer)
            .read_all()
            .unwrap();
        let mut observed = CrashStore::new(seed.clone(), None, 0, false);
        super::dense_update::replace_filesystem_metadata(
            &mut observed,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &entries,
        )
        .unwrap();
        for at in 0..observed.operations() {
            for prefix in [0, 97, usize::MAX] {
                for persist in [false, true] {
                    let mut failed = CrashStore::new(seed.clone(), Some(at), prefix, persist);
                    let _ = super::dense_update::replace_filesystem_metadata(
                        &mut failed,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        &entries,
                    );
                    let mut reopened = StorageBackend::memory(failed.durable());
                    super::dense_update::recover(
                        &mut reopened,
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap_or_else(|e| {
                        panic!("typed at={at} prefix={prefix} sync={persist}: {e}")
                    });
                    let mut image = super::dense_image::Image::open(
                        reopened,
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap();
                    match image.filesystem_metadata() {
                        Ok(actual) => assert_eq!(actual, entries),
                        Err(Error::InvalidOperation(_)) => {
                            assert_eq!(image.info(b"/docs/data").unwrap().unwrap().len, 7)
                        }
                        Err(error) => panic!("unexpected typed state error: {error}"),
                    }
                    let mut bytes = Vec::new();
                    image
                        .read_range(b"/docs/data", 0, 7, |part| {
                            bytes.extend_from_slice(part);
                            Ok(())
                        })
                        .unwrap();
                    assert_eq!(bytes, b"payload");
                    cases += 1;
                }
            }
        }
    }
    println!("DENSE_FILESYSTEM_METADATA_POWER_LOSS_CASES {cases}");
}

#[derive(Default)]
struct FilesystemSalvaged {
    files: Salvaged,
    metadata: Option<Vec<super::dense_catalogue::Metadata>>,
    reject_metadata: bool,
}
impl recovery::Sink for FilesystemSalvaged {
    fn begin(&mut self, path: &[u8], len: u64) -> Result<()> {
        assert!(self.metadata.is_some());
        self.files.begin(path, len)
    }
    fn data(&mut self, bytes: &[u8]) -> Result<()> {
        self.files.data(bytes)
    }
    fn finish(&mut self, complete: bool) -> Result<()> {
        self.files.finish(complete)
    }
}
impl super::dense_image::FilesystemSink for FilesystemSalvaged {
    fn metadata(&mut self, entries: &[super::dense_catalogue::Metadata]) -> Result<()> {
        assert!(self.metadata.is_none());
        if self.reject_metadata {
            return Err(Error::InvalidInput("metadata sink refused".into()));
        }
        self.metadata = Some(entries.to_vec());
        Ok(())
    }
}

#[test]
fn dense_filesystem_salvage_preserves_nodes_even_when_payload_is_lost_all_modes() {
    use super::dense_image::salvage_filesystem;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let entries = public_filesystem_metadata(&owner);
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                for padded in [false, true] {
                    let mode = mode(encrypted, signed, compressed, padded);
                    let authority = authority(mode, &public);
                    let signer = signed.then_some(&owner);
                    let mut storage = canonical_dense_seed(mode, &authority, signer);
                    super::dense_update::replace_filesystem_metadata(
                        &mut storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        &entries,
                    )
                    .unwrap();
                    super::dense_update::return_inline(
                        &mut storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                    )
                    .unwrap();
                    let (anchor, body) = publication::shared::open_private(
                        &storage,
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap();
                    let codec = Codec::shared_packed(archive(), mode, key(mode)).unwrap();
                    let catalogue =
                        super::dense_catalogue::Catalogue::decode(&body, &codec, anchor.sealed_len)
                            .unwrap();
                    let fragment = &catalogue.files[0].fragments[0];
                    let at = catalogue.packs[fragment.pack].extent.start + fragment.relative as u64;
                    let original = storage.read_all().unwrap();
                    // There is no public CLI writer for this experimental layout.
                    // Inject damage into copies; the recovery backend forbids writes.
                    for damage in 0..5 {
                        let mut damaged = StorageBackend::memory(original.clone());
                        match damage {
                            1 => {
                                let byte = damaged.read_at(at, 1).unwrap()[0];
                                damaged.write_at(at, &[byte ^ 1]).unwrap();
                            }
                            2 => damaged.truncate(at).unwrap(),
                            3 => damaged.write_at(0, &vec![0; 65536]).unwrap(),
                            4 => damaged.write_at(65536, &vec![0; 65536]).unwrap(),
                            _ => (),
                        }
                        let before = damaged.read_all().unwrap();
                        let guarded = DenseReadFailure {
                            storage: damaged,
                            fail_at: u64::MAX,
                        };
                        let mut sink = FilesystemSalvaged::default();
                        let report = salvage_filesystem(
                            &guarded,
                            archive(),
                            mode,
                            &authority,
                            key(mode),
                            &mut sink,
                        )
                        .unwrap();
                        assert_eq!(sink.metadata.as_deref(), Some(entries.as_slice()));
                        let complete = damage != 1 && damage != 2;
                        assert_eq!(
                            (report.complete, report.incomplete),
                            (u64::from(complete), u64::from(!complete))
                        );
                        if complete {
                            assert_eq!(sink.files.files[b"/docs/data".as_slice()], b"payload");
                        } else {
                            assert!(sink.files.files.is_empty());
                            assert_eq!(sink.files.incomplete, vec![b"/docs/data".to_vec()]);
                        }
                        assert_eq!(guarded.storage.read_all().unwrap(), before);
                    }
                    let mut sink = FilesystemSalvaged {
                        reject_metadata: true,
                        ..Default::default()
                    };
                    assert!(
                        matches!(salvage_filesystem(&storage, archive(), mode, &authority, key(mode), &mut sink), Err(Error::InvalidInput(ref why)) if why == "metadata sink refused")
                    );
                    assert!(sink.metadata.is_none() && sink.files.active.is_none());
                    let guarded = DenseReadFailure {
                        storage: StorageBackend::memory(original.clone()),
                        fail_at: at,
                    };
                    let mut sink = FilesystemSalvaged::default();
                    assert!(
                        matches!(salvage_filesystem(&guarded, archive(), mode, &authority, key(mode), &mut sink), Err(Error::InvalidInput(ref why)) if why == "synthetic dense recovery I/O failure")
                    );
                    // Metadata was staged before the late error: the whole batch
                    // must be discarded by the caller, never installed piecemeal.
                    assert!(sink.metadata.is_some());
                    assert!(sink.files.files.is_empty());
                    let mut damaged = StorageBackend::memory(original);
                    damaged.write_at(16384, &vec![0; 49152]).unwrap();
                    damaged.write_at(81920, &vec![0; 49152]).unwrap();
                    let mut sink = FilesystemSalvaged::default();
                    assert!(salvage_filesystem(
                        &damaged,
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                        &mut sink
                    )
                    .is_err());
                    assert!(sink.metadata.is_none() && sink.files.active.is_none());
                    if encrypted {
                        assert!(salvage_filesystem(
                            &storage,
                            archive(),
                            mode,
                            &authority,
                            Some(&[99; 32]),
                            &mut sink
                        )
                        .is_err());
                        assert!(sink.metadata.is_none());
                    }
                    if signed {
                        let wrong = OwnerSigningKeyPair::generate().unwrap().public_key();
                        assert!(salvage_filesystem(
                            &storage,
                            archive(),
                            mode,
                            &Authority::Owner(&wrong),
                            key(mode),
                            &mut sink
                        )
                        .is_err());
                        assert!(sink.metadata.is_none());
                    }
                }
            }
        }
    }
}
