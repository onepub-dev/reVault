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
