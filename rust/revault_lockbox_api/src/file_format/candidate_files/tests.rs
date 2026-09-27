use super::*;
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
    let storage = from_candidate(
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
