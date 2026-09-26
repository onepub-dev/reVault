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
                codec: Codec::new(archive(), mode, None).unwrap(),
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
