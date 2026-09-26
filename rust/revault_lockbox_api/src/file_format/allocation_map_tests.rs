//! Internal candidate protocol tests. Public CLI output does not use this format.
use super::*;
use crate::crypto::{open_with_nonce, seal_with_random_nonce};
use crate::storage::StorageBackend;
use crate::{
    Compression, EncryptionMode, LockboxFormatOptions, OwnerSigningPublicKey, SigningMode,
    SizePadding,
};
const KEY: &[u8; 32] = &[73; 32];
fn archive() -> LockboxId {
    LockboxId::from_bytes([62; 16])
}
fn mode(encrypted: bool, signed: bool, padded: bool) -> FormatMode {
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
        compression: Compression::None,
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
fn authority(mode: FormatMode, public: &OwnerSigningPublicKey) -> Authority<'_> {
    if mode.signed() {
        Authority::Owner(public)
    } else if mode.plaintext() {
        Authority::Checksum
    } else {
        Authority::Symmetric(KEY)
    }
}
fn stored(mode: FormatMode, plain: &[u8]) -> Vec<u8> {
    if mode.plaintext() {
        plain.to_vec()
    } else {
        let (nonce, body) = seal_with_random_nonce(plain, KEY, archive().as_bytes()).unwrap();
        [nonce.as_slice(), body.as_slice()].concat()
    }
}
fn record(storage: &impl Storage, anchor: &Anchor, name: &[u8]) -> Option<OwnedRecord> {
    Index::new(archive(), anchor.mode, key(anchor.mode))
        .unwrap()
        .get(storage, anchor.index, anchor.sealed_len, 1, name)
        .unwrap()
        .map(|e| OwnedRecord::decode(&e.value).unwrap())
}
fn content(storage: &impl Storage, anchor: &Anchor, name: &[u8]) -> Option<Vec<u8>> {
    record(storage, anchor, name).map(|record| {
        assert_eq!(record.extents.len(), 1);
        let extent = record.extents[0];
        let bytes = storage.read_at(extent.start, extent.len as usize).unwrap();
        assert_eq!(strong_checksum(&bytes), extent.digest);
        if anchor.mode.plaintext() {
            bytes
        } else {
            open_with_nonce(&bytes[12..], KEY, &bytes[..12], archive().as_bytes()).unwrap()
        }
    })
}
fn audit(storage: &impl Storage, anchor: &Anchor) -> Accounting {
    let index = Index::new(archive(), anchor.mode, key(anchor.mode)).unwrap();
    let snapshot = Snapshot::inspect(storage, anchor, &index).unwrap();
    snapshot.verify_reclaimed(storage).unwrap();
    let a = &snapshot.accounting;
    assert_eq!(
        a.total,
        a.fixed + a.payload + a.index + a.keys + a.allocation + a.reserve + a.free + a.pending
    );
    assert_eq!(a.total, storage.len().unwrap());
    snapshot.accounting
}
#[test]
fn candidate_lifecycle_accounts_shared_payload_keys_reuse_retirement_and_no_change_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for padded in [false, true] {
                let mode = mode(encrypted, signed, padded);
                let authority = authority(mode, &public);
                let signer = signed.then_some(&owner);
                let mut storage = StorageBackend::memory(Vec::new());
                let initial =
                    create_empty(&mut storage, archive(), mode, &authority, signer, key(mode))
                        .unwrap();
                audit(&storage, &initial);
                let mut tx =
                    Transaction::begin(storage, archive(), mode, &authority, key(mode)).unwrap();
                let pack = tx
                    .append_encoded_extent(&stored(mode, b"alpha|neighbour"))
                    .unwrap();
                tx.put(1, b"/alpha", b"slice 0..5", &[pack]).unwrap();
                tx.put(1, b"/neighbour", b"slice 6..15", &[pack]).unwrap();
                tx.put(2, b"/directory", b"directory attributes", &[])
                    .unwrap();
                tx.put_key_record(1, b"recipient", b"synthetic wrapped-key record", &[])
                    .unwrap();
                let (storage, anchor) = tx.commit(&authority, signer).unwrap();
                let stats = audit(&storage, &anchor);
                assert_eq!(stats.payload, pack.len);
                assert!(stats.keys > 0);
                assert!(stats.pending > 0);
                let storage = StorageBackend::memory(storage.read_all().unwrap());
                assert_eq!(
                    content(&storage, &anchor, b"/alpha").unwrap(),
                    b"alpha|neighbour"
                );
                // Exact repeat publishes no generation and adds no bytes or metadata pages.
                let before = storage.len().unwrap();
                let mut tx =
                    Transaction::begin(storage, archive(), mode, &authority, key(mode)).unwrap();
                tx.put(1, b"/alpha", b"slice 0..5", &[pack]).unwrap();
                let (storage, unchanged) = tx.commit(&authority, signer).unwrap();
                assert_eq!(unchanged, anchor);
                assert_eq!(storage.len().unwrap(), before);
                audit(&storage, &unchanged);
                let mut tx =
                    Transaction::begin(storage, archive(), mode, &authority, key(mode)).unwrap();
                let replacement = tx.append_encoded_extent(&stored(mode, b"new")).unwrap();
                assert!(
                    replacement.start < anchor.sealed_len,
                    "expected physical reuse"
                );
                tx.put(1, b"/alpha", b"whole extent", &[replacement])
                    .unwrap();
                let (storage, replaced) = tx.commit(&authority, signer).unwrap();
                audit(&storage, &replaced);
                assert_eq!(content(&storage, &replaced, b"/alpha").unwrap(), b"new");
                assert_eq!(
                    content(&storage, &replaced, b"/neighbour").unwrap(),
                    b"alpha|neighbour"
                );
                let mut tx =
                    Transaction::begin(storage, archive(), mode, &authority, key(mode)).unwrap();
                tx.remove(1, b"/neighbour").unwrap();
                let (storage, removed) = tx.commit(&authority, signer).unwrap();
                audit(&storage, &removed);
                assert!(record(&storage, &removed, b"/neighbour").is_none());
                assert!(storage
                    .read_at(pack.start, pack.len as usize)
                    .unwrap()
                    .iter()
                    .all(|b| *b == 0));
                assert_eq!(content(&storage, &removed, b"/alpha").unwrap(), b"new");
                let original_len = storage.len().unwrap();
                let mut tx =
                    Transaction::begin(storage, archive(), mode, &authority, key(mode)).unwrap();
                let abandoned = tx
                    .append_encoded_extent(&stored(mode, &vec![0x9a; 200000]))
                    .unwrap();
                tx.put(1, b"/unpublished", b"discard", &[abandoned])
                    .unwrap();
                let storage = tx.abort(&authority).unwrap();
                assert_eq!(storage.len().unwrap(), original_len);
                audit(&storage, &removed);
                assert!(record(&storage, &removed, b"/unpublished").is_none());
                assert_eq!(content(&storage, &removed, b"/alpha").unwrap(), b"new");
            }
        }
    }
}

#[test]
fn ownership_envelope_rejects_unbounded_lengths_and_bad_physical_references() {
    let extent = Extent {
        start: DATA_START,
        len: 4096,
        digest: [8; 32],
    };
    let encoded = OwnedRecord::encode(b"metadata", &[extent]).unwrap();
    let decoded = OwnedRecord::decode(&encoded).unwrap();
    assert_eq!(decoded.metadata.as_slice(), b"metadata");
    assert_eq!(decoded.extents, vec![extent]);
    for (offset, bytes) in [
        (8, vec![255, 255]),
        (10, u32::MAX.to_le_bytes().to_vec()),
        (14, 0u64.to_le_bytes().to_vec()),
        (22, 0u64.to_le_bytes().to_vec()),
    ] {
        let mut bad = encoded.to_vec();
        bad[offset..offset + bytes.len()].copy_from_slice(&bytes);
        assert!(OwnedRecord::decode(&bad).is_err());
    }
    assert!(OwnedRecord::encode(
        b"",
        &[Extent {
            start: u64::MAX,
            len: 2,
            ..extent
        }]
    )
    .is_err());
    assert!(OwnedRecord::encode(&vec![0; MAX_RECORD], &[]).is_err());
}

#[test]
fn audit_rejects_unowned_gaps_overlapping_payload_and_freed_descendant_pages() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let mode = mode(false, false, false);
    let mut storage = StorageBackend::memory(Vec::new());
    let index = Index::new(archive(), mode, None).unwrap();
    let initial = create_empty(
        &mut storage,
        archive(),
        mode,
        &Authority::Checksum,
        None,
        None,
    )
    .unwrap();
    let base = Snapshot::inspect(&storage, &initial, &index).unwrap();
    let bytes = b"shared allocation";
    let payload = Extent {
        start: storage.append(bytes).unwrap(),
        len: bytes.len() as u64,
        digest: strong_checksum(bytes),
    };
    let value = OwnedRecord::encode(b"shared", &[payload]).unwrap();
    let logical = index
        .build_sorted(
            &mut storage,
            (0u32..1500).map(|n| Entry::new(1, &n.to_be_bytes(), &value)),
        )
        .unwrap();
    assert!(logical.created.len() > 2);
    let mut next = Anchor {
        index: logical.root,
        object_root: logical.root.digest,
        sealed_len: storage.len().unwrap(),
        ..initial.clone()
    };
    let live = Snapshot::live(&storage, &next, &index).unwrap();
    let records = gap_records(&live, &base, &Claims::default(), next.sealed_len).unwrap();
    let valid = index
        .build_sorted(&mut storage, records.clone().into_iter().map(Ok))
        .unwrap();
    next.allocation = valid.root;
    next.sealed_len = storage.len().unwrap();
    let snapshot = Snapshot::inspect(&storage, &next, &index).unwrap();
    assert_eq!(snapshot.accounting.payload, payload.len);
    let mut gap_storage = storage.clone();
    gap_storage.append(b"unowned").unwrap();
    let gap_anchor = Anchor {
        sealed_len: gap_storage.len().unwrap(),
        ..next.clone()
    };
    assert!(Snapshot::inspect(&gap_storage, &gap_anchor, &index).is_err());
    // The journal's old direct-root exclusion alone could not detect a FREE
    // claim over a descendant. The complete graph audit must reject it.
    let child = logical
        .created
        .iter()
        .find(|r| **r != logical.root)
        .copied()
        .unwrap();
    let mut forged_records = records;
    forged_records
        .push(Entry::new(FREE, &child.primary.to_be_bytes(), &child.len.to_le_bytes()).unwrap());
    forged_records
        .sort_by(|a, b| (a.namespace, a.key.as_slice()).cmp(&(b.namespace, b.key.as_slice())));
    let forged = index
        .build_sorted(&mut storage, forged_records.into_iter().map(Ok))
        .unwrap();
    let forged_anchor = Anchor {
        allocation: forged.root,
        sealed_len: storage.len().unwrap(),
        ..next.clone()
    };
    assert!(Snapshot::inspect(&storage, &forged_anchor, &index).is_err());
    let mut overlap = Claims::default();
    overlap
        .insert(
            Claim {
                extent: payload,
                kind: Kind::Payload,
            },
            next.sealed_len,
        )
        .unwrap();
    assert!(overlap
        .insert(
            Claim {
                extent: Extent {
                    start: payload.start + 1,
                    len: payload.len - 1,
                    ..payload
                },
                kind: Kind::Payload
            },
            next.sealed_len
        )
        .is_err());
    assert!(overlap
        .insert(
            Claim {
                extent: Extent {
                    digest: [1; 32],
                    ..payload
                },
                kind: Kind::Payload
            },
            next.sealed_len
        )
        .is_err());
    let _ = owner;
}

#[derive(Clone, Debug)]
struct Observed(Arc<Mutex<StorageBackend>>);
impl Observed {
    fn new(bytes: Vec<u8>) -> Self {
        Self(Arc::new(Mutex::new(StorageBackend::memory(bytes))))
    }
    fn count(&self) -> usize {
        self.0.lock().unwrap().memory_operation_count()
    }
    fn fail(&self, at: usize) {
        self.0
            .lock()
            .unwrap()
            .fail_memory_operation_after_successes(at);
    }
}
impl Storage for Observed {
    fn len(&self) -> Result<u64> {
        self.0.lock().unwrap().len()
    }
    fn read_at(&self, o: u64, n: usize) -> Result<Vec<u8>> {
        self.0.lock().unwrap().read_at(o, n)
    }
    fn read_at_into(&self, o: u64, b: &mut [u8]) -> Result<()> {
        self.0.lock().unwrap().read_at_into(o, b)
    }
    fn append(&mut self, b: &[u8]) -> Result<u64> {
        self.0.lock().unwrap().append(b)
    }
    fn write_at(&mut self, o: u64, b: &[u8]) -> Result<()> {
        self.0.lock().unwrap().write_at(o, b)
    }
    fn truncate(&mut self, n: u64) -> Result<()> {
        self.0.lock().unwrap().truncate(n)
    }
    fn sync(&self) -> Result<()> {
        self.0.lock().unwrap().sync()
    }
}
fn seed(mode: FormatMode, owner: &OwnerSigningKeyPair) -> (Vec<u8>, Anchor) {
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let signer = mode.signed().then_some(owner);
    let mut storage = StorageBackend::memory(Vec::new());
    create_empty(&mut storage, archive(), mode, &authority, signer, key(mode)).unwrap();
    let mut tx = Transaction::begin(storage, archive(), mode, &authority, key(mode)).unwrap();
    let shared = tx
        .append_encoded_extent(&stored(mode, b"original shared bytes"))
        .unwrap();
    tx.put(1, b"/keep", b"old", &[shared]).unwrap();
    tx.put(1, b"/neighbour", b"old", &[shared]).unwrap();
    let spare = tx
        .append_encoded_extent(&stored(mode, &vec![0x31; 8192]))
        .unwrap();
    tx.put(1, b"/discard", b"old", &[spare]).unwrap();
    let (storage, _) = tx.commit(&authority, signer).unwrap();
    let mut tx = Transaction::begin(storage, archive(), mode, &authority, key(mode)).unwrap();
    tx.remove(1, b"/discard").unwrap();
    let (storage, anchor) = tx.commit(&authority, signer).unwrap();
    audit(&storage, &anchor);
    (storage.read_all().unwrap(), anchor)
}
fn mutate<S: Storage>(
    storage: S,
    base: &Anchor,
    owner: &OwnerSigningKeyPair,
    large: bool,
) -> Result<(S, Anchor)> {
    let public = owner.public_key();
    let authority = authority(base.mode, &public);
    let mut tx = Transaction::begin(storage, archive(), base.mode, &authority, key(base.mode))?;
    let bytes = stored(
        base.mode,
        &vec![
            0x83;
            if large {
                base.sealed_len as usize + 1
            } else {
                256
            }
        ],
    );
    let extent = tx.append_encoded_extent(&bytes)?;
    if large {
        assert!(extent.start >= base.sealed_len);
    } else {
        assert!(extent.start < base.sealed_len);
    }
    tx.put(1, b"/keep", b"new", &[extent])?;
    tx.remove(1, b"/neighbour")?;
    tx.put_key_record(3, b"recipient", b"new synthetic wrapped key", &[])?;
    tx.commit(&authority, base.mode.signed().then_some(owner))
}
fn verify_after_failure(
    storage: &mut impl Storage,
    base: &Anchor,
    owner: &OwnerSigningKeyPair,
    large: bool,
) {
    let public = owner.public_key();
    let authority = authority(base.mode, &public);
    let selected = recover(storage, archive(), base.mode, &authority, key(base.mode)).unwrap();
    audit(storage, &selected);
    if selected.generation == base.generation {
        assert_eq!(
            content(storage, &selected, b"/keep").unwrap(),
            b"original shared bytes"
        );
        assert_eq!(
            content(storage, &selected, b"/neighbour").unwrap(),
            b"original shared bytes"
        );
    } else {
        assert_eq!(selected.generation, base.generation + 1);
        assert_eq!(
            content(storage, &selected, b"/keep").unwrap(),
            vec![
                0x83;
                if large {
                    base.sealed_len as usize + 1
                } else {
                    256
                }
            ]
        );
        assert!(record(storage, &selected, b"/neighbour").is_none());
    }
    assert!(record(storage, &selected, b"/discard").is_none());
}
#[test]
fn allocator_faults_preserve_complete_old_or_new_ownership_for_reuse_and_append() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let mut cases = 0;
    for mode in [mode(false, false, false), mode(true, true, true)] {
        let (before, base) = seed(mode, &owner);
        for large in [false, true] {
            let probe = Observed::new(before.clone());
            mutate(probe.clone(), &base, &owner, large).unwrap();
            for fail in 0..probe.count() {
                let storage = Observed::new(before.clone());
                storage.fail(fail);
                assert!(
                    mutate(storage.clone(), &base, &owner, large).is_err(),
                    "large={large}, operation={fail}"
                );
                let mut reopened = Observed::new(storage.read_all().unwrap());
                verify_after_failure(&mut reopened, &base, &owner, large);
                cases += 1;
            }
        }
    }
    eprintln!("allocator atomic failure cases: {cases}");
}
#[test]
fn allocator_power_loss_preserves_ownership_and_reclamation() {
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let (before, base) = seed(mode(false, false, false), &owner);
    let mut cases = 0;
    for large in [false, true] {
        let probe = CrashStore::new(before.clone(), None, 0, false);
        mutate(probe.clone(), &base, &owner, large).unwrap();
        for fail in 0..probe.operations() {
            for prefix in [0, 1, 48, 4096, journal::SLOT_BYTES - 1, journal::SLOT_BYTES] {
                for persist in [false, true] {
                    let crashed = CrashStore::new(before.clone(), Some(fail), prefix, persist);
                    assert!(mutate(crashed.clone(), &base, &owner, large).is_err());
                    let mut reopened = Observed::new(crashed.durable());
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        verify_after_failure(&mut reopened, &base, &owner, large)
                    }));
                    assert!(
                        outcome.is_ok(),
                        "large={large}, operation={fail}, prefix={prefix}, persist={persist}"
                    );
                    cases += 1;
                }
            }
        }
    }
    eprintln!("allocator power-loss cases: {cases}");
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "manual file-backed allocator lifecycle CPU/RSS and physical accounting"]
fn allocation_aging_resource_probe() {
    fn usage() -> (i64, i64) {
        let mut value = std::mem::MaybeUninit::<libc::rusage>::uninit();
        // SAFETY: getrusage initializes the output on success, checked below.
        let status = unsafe { libc::getrusage(libc::RUSAGE_SELF, value.as_mut_ptr()) };
        assert_eq!(status, 0);
        // SAFETY: the successful call above initialized every field.
        let value = unsafe { value.assume_init() };
        (
            value.ru_utime.tv_sec * 1_000_000
                + value.ru_utime.tv_usec
                + value.ru_stime.tv_sec * 1_000_000
                + value.ru_stime.tv_usec,
            value.ru_maxrss,
        )
    }
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let cycles: usize = std::env::var("REVAULT_ALLOCATION_CYCLES")
        .unwrap()
        .parse()
        .unwrap();
    assert!((100..=1000).contains(&cycles));
    let encrypted = std::env::var("REVAULT_ALLOCATION_ENCRYPTED").unwrap() == "1";
    let signed = std::env::var("REVAULT_ALLOCATION_SIGNED").unwrap() == "1";
    let padded = std::env::var("REVAULT_ALLOCATION_PADDED").unwrap() == "1";
    let mode = mode(encrypted, signed, padded);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let signer = signed.then_some(&owner);
    let cleanup = Cleanup(std::env::temp_dir().join(format!(
        "revault-allocation-aging-{}.candidate",
        std::process::id()
    )));
    assert!(!cleanup.0.exists());
    let mut storage = StorageBackend::create_file(&cleanup.0, &[]).unwrap();
    let mut anchor =
        create_empty(&mut storage, archive(), mode, &authority, signer, key(mode)).unwrap();
    let mut expected: BTreeMap<Vec<u8>, (Vec<u8>, Extent)> = BTreeMap::new();
    let mut total_cpu = 0;
    let mut total_wall = 0;
    let mut max_len = 0;
    for cycle in 0..cycles {
        let before_len = storage.len().unwrap();
        let before_generation = anchor.generation;
        let start_cpu = usage().0;
        let start = std::time::Instant::now();
        let mut tx = Transaction::begin(storage, archive(), mode, &authority, key(mode)).unwrap();
        let no_change = matches!(cycle % 8, 1 | 7);
        match cycle % 8 {
            0 | 2 | 3 => {
                let len = match cycle % 8 {
                    0 => 4096,
                    2 => 12289,
                    _ => 31,
                };
                let plain = vec![(cycle % 251) as u8; len];
                let extent = tx.append_encoded_extent(&stored(mode, &plain)).unwrap();
                tx.put(1, b"/item", b"data", &[extent]).unwrap();
                expected.insert(b"/item".to_vec(), (plain, extent));
            }
            1 => {
                tx.put(1, b"/item", b"data", &[expected[b"/item".as_slice()].1])
                    .unwrap();
            }
            4 => {
                let shared = expected[b"/item".as_slice()].clone();
                tx.put(1, b"/alias", b"data", &[shared.1]).unwrap();
                expected.insert(b"/alias".to_vec(), shared);
                tx.put_key_record(1, b"recipient", &cycle.to_le_bytes(), &[])
                    .unwrap();
            }
            5 => {
                tx.remove(1, b"/item").unwrap();
                expected.remove(b"/item".as_slice());
            }
            6 | 7 => {
                tx.remove(1, b"/alias").unwrap();
                expected.remove(b"/alias".as_slice());
            }
            _ => unreachable!(),
        }
        let (completed, next) = tx.commit(&authority, signer).unwrap();
        let wall_us = start.elapsed().as_micros() as u64;
        let cpu_us = usage().0 - start_cpu;
        total_wall += wall_us;
        total_cpu += cpu_us;
        drop(completed);
        // Persisted bytes are verified using an independent file handle on every
        // cycle. No archive-sized clone is used by setup, mutation or verification.
        storage = StorageBackend::file_for_write(&cleanup.0).unwrap();
        anchor = recover(&mut storage, archive(), mode, &authority, key(mode)).unwrap();
        assert_eq!(anchor, next);
        let accounting = audit(&storage, &anchor);
        max_len = max_len.max(accounting.total);
        for name in [b"/item".as_slice(), b"/alias".as_slice()] {
            assert_eq!(
                content(&storage, &anchor, name),
                expected.get(name).map(|(bytes, _)| bytes.clone())
            );
        }
        if no_change {
            assert_eq!(anchor.generation, before_generation);
            assert_eq!(accounting.total, before_len);
        }
        println!(
            "ALLOCATION_CYCLE {}",
            serde_json::json!({"cycle":cycle,"encrypted":encrypted,"signed":signed,"padded_index":padded,"generation":anchor.generation,"no_change":no_change,"cpu_us":cpu_us,"wall_us":wall_us,"rss_kib":usage().1,"fixed":accounting.fixed,"payload":accounting.payload,"index":accounting.index,"keys":accounting.keys,"allocation":accounting.allocation,"reserve":accounting.reserve,"free":accounting.free,"pending":accounting.pending,"file_bytes":accounting.total})
        );
    }
    println!(
        "ALLOCATION_TOTAL {}",
        serde_json::json!({"cycles":cycles,"encrypted":encrypted,"signed":signed,"padded_index":padded,"cpu_us":total_cpu,"wall_us":total_wall,"rss_kib":usage().1,"max_file_bytes":max_len,"final_file_bytes":storage.len().unwrap()})
    );
}

#[test]
fn bulk_replacement_accounts_fragmented_free_space_and_retires_allocation_descendants() {
    let mode = mode(false, false, false);
    let authority = Authority::Checksum;
    let mut storage = StorageBackend::memory(Vec::new());
    create_empty(&mut storage, archive(), mode, &authority, None, None).unwrap();
    let mut tx = Transaction::begin(storage, archive(), mode, &authority, None).unwrap();
    let extents = (0..2400)
        .map(|_| tx.append_encoded_extent(b"p").unwrap())
        .collect::<Vec<_>>();
    tx.replace_all_sorted(extents.iter().enumerate().map(|(n, extent)| {
        Entry::new(
            1,
            &(n as u32).to_be_bytes(),
            &OwnedRecord::encode(b"file", &[*extent])?,
        )
    }))
    .unwrap();
    let (storage, first) = tx.commit(&authority, None).unwrap();
    assert_eq!(audit(&storage, &first).payload, 2400);
    let mut tx = Transaction::begin(storage, archive(), mode, &authority, None).unwrap();
    tx.replace_all_sorted(extents.iter().enumerate().filter(|(n, _)| n % 2 == 0).map(
        |(n, extent)| {
            Entry::new(
                1,
                &(n as u32).to_be_bytes(),
                &OwnedRecord::encode(b"file", &[*extent])?,
            )
        },
    ))
    .unwrap();
    let (storage, second) = tx.commit(&authority, None).unwrap();
    assert_eq!(audit(&storage, &second).payload, 1200);
    let index = Index::new(archive(), mode, None).unwrap();
    let mut old_map = Vec::new();
    index
        .visit_owned(&storage, second.allocation, second.sealed_len, |event| {
            if let Visit::Page(page) = event {
                old_map.push(page);
            }
            Ok(())
        })
        .unwrap();
    assert!(old_map.len() > 1);
    let mut tx = Transaction::begin(storage, archive(), mode, &authority, None).unwrap();
    // Create then discard an intermediate prepared path and payload; the final
    // map must clean these too, including if they reused previously free space.
    let abandoned = tx.append_encoded_extent(b"abandoned").unwrap();
    tx.put(1, b"temporary", b"discarded", &[abandoned]).unwrap();
    tx.replace_all_sorted([Entry::new(
        1,
        &0u32.to_be_bytes(),
        &OwnedRecord::encode(b"survivor", &[extents[0]]).unwrap(),
    )])
    .unwrap();
    let (storage, third) = tx.commit(&authority, None).unwrap();
    assert_eq!(audit(&storage, &third).payload, 1);
    for page in old_map {
        for start in [page.primary, page.mirror] {
            assert!(storage
                .read_at(start, page.len as usize)
                .unwrap()
                .iter()
                .all(|b| *b == 0));
        }
    }
    assert!(storage
        .read_at(abandoned.start, abandoned.len as usize)
        .unwrap()
        .iter()
        .all(|b| *b == 0));
    assert_eq!(storage.read_at(extents[0].start, 1).unwrap(), b"p");
}

#[derive(Clone, Debug)]
struct CountWrites(u64);
impl Storage for CountWrites {
    fn len(&self) -> Result<u64> {
        Ok(self.0)
    }
    fn read_at(&self, _o: u64, _n: usize) -> Result<Vec<u8>> {
        Err(Error::Io("write-count geometry fixture cannot read".into()))
    }
    fn read_at_into(&self, _o: u64, _b: &mut [u8]) -> Result<()> {
        Err(Error::Io("write-count geometry fixture cannot read".into()))
    }
    fn append(&mut self, b: &[u8]) -> Result<u64> {
        let offset = self.0;
        self.0 += b.len() as u64;
        Ok(offset)
    }
    fn write_at(&mut self, _o: u64, _b: &[u8]) -> Result<()> {
        Err(Error::Io(
            "write-count geometry fixture cannot overwrite".into(),
        ))
    }
    fn truncate(&mut self, _n: u64) -> Result<()> {
        Err(Error::Io(
            "write-count geometry fixture cannot truncate".into(),
        ))
    }
    fn sync(&self) -> Result<()> {
        Ok(())
    }
}
#[test]
fn allocation_arena_bound_covers_actual_bulk_geometry_and_branch_boundaries() {
    for encrypted in [false, true] {
        for padded in [false, true] {
            let mode = mode(encrypted, false, padded);
            let index = Index::new(archive(), mode, key(mode)).unwrap();
            for count in [0u64, 1, 1024, 1025, 100000] {
                let mut storage = CountWrites(DATA_START);
                index
                    .build_sorted(
                        &mut storage,
                        (0..count).map(|n| Entry::new(FREE, &n.to_be_bytes(), &8u64.to_le_bytes())),
                    )
                    .unwrap();
                assert_eq!(
                    storage.len().unwrap() - DATA_START,
                    index.fixed_record_size_bound(count, 8, 8).unwrap(),
                    "encrypted={encrypted}, padded={padded}, count={count}"
                );
            }
        }
    }
    // More than 872 leaves forces a second branch level. This fixture measures
    // actual encoded writes without retaining a 100+ MiB memory-backed file.
    let index = Index::new(archive(), mode(false, false, false), None).unwrap();
    let count = 893953;
    let mut storage = CountWrites(DATA_START);
    index
        .build_sorted(
            &mut storage,
            (0u64..count).map(|n| Entry::new(FREE, &n.to_be_bytes(), &8u64.to_le_bytes())),
        )
        .unwrap();
    assert_eq!(
        storage.len().unwrap() - DATA_START,
        index.fixed_record_size_bound(count, 8, 8).unwrap()
    );
}
#[test]
fn unused_control_arena_bytes_cannot_hide_payload() {
    let mode = mode(false, false, false);
    let authority = Authority::Checksum;
    let mut storage = StorageBackend::memory(Vec::new());
    create_empty(&mut storage, archive(), mode, &authority, None, None).unwrap();
    let mut tx = Transaction::begin(storage, archive(), mode, &authority, None).unwrap();
    let extent = tx.append_encoded_extent(b"content").unwrap();
    tx.put(1, b"/file", b"data", &[extent]).unwrap();
    let (mut storage, anchor) = tx.commit(&authority, None).unwrap();
    let index = Index::new(archive(), mode, None).unwrap();
    let snapshot = Snapshot::inspect(&storage, &anchor, &index).unwrap();
    let reserve = snapshot
        .claims
        .0
        .values()
        .find(|c| c.kind == Kind::Reserve)
        .unwrap()
        .extent;
    assert!(snapshot.accounting.reserve > 0);
    storage.write_at(reserve.start, b"x").unwrap();
    let before = storage.read_all().unwrap();
    assert!(recover(&mut storage, archive(), mode, &authority, None).is_err());
    assert_eq!(storage.read_all().unwrap(), before);
}

#[test]
fn recovery_audits_descendant_ownership_before_any_erasure() {
    let mode = mode(false, false, false);
    let authority = Authority::Checksum;
    let index = Index::new(archive(), mode, None).unwrap();
    let mut storage = StorageBackend::memory(Vec::new());
    create_empty(&mut storage, archive(), mode, &authority, None, None).unwrap();
    let mut tx = Transaction::begin(storage, archive(), mode, &authority, None).unwrap();
    let value = OwnedRecord::encode(b"data", &[]).unwrap();
    tx.replace_all_sorted((0u32..2400).map(|n| Entry::new(1, &n.to_be_bytes(), &value)))
        .unwrap();
    let (storage, base) = tx.commit(&authority, None).unwrap();
    let snapshot = Snapshot::inspect(&storage, &base, &index).unwrap();
    let mut child = None;
    index
        .visit_owned(&storage, base.index, base.sealed_len, |event| {
            if let Visit::Page(page) = event {
                if page != base.index {
                    child = Some(page);
                }
            }
            Ok(())
        })
        .unwrap();
    let child = child.unwrap();
    let mut prepared = PreparedStore::begin(storage, archive(), mode, &authority, None).unwrap();
    let live = Snapshot::live(&prepared, &base, &index).unwrap();
    let mut records = gap_records(&live, &snapshot, &Claims::default(), base.sealed_len).unwrap();
    records.push(
        Entry::new(
            PENDING,
            &child.primary.to_be_bytes(),
            &child.len.to_le_bytes(),
        )
        .unwrap(),
    );
    records.sort_by(|a, b| (a.namespace, a.key.as_slice()).cmp(&(b.namespace, b.key.as_slice())));
    let map = index
        .build_sorted(&mut prepared, records.into_iter().map(Ok))
        .unwrap();
    let next = Anchor {
        generation: base.generation + 1,
        previous: base.commitment().unwrap(),
        allocation: map.root,
        sealed_len: prepared.len().unwrap(),
        ..base.clone()
    };
    // Deliberately bypass the allocator's commit audit to simulate an internally
    // inconsistent published map. No public CLI can construct this condition.
    let mut storage = prepared.into_inner().unwrap();
    publication::publish(
        &mut storage,
        &next,
        &authority,
        None,
        Some(base.commitment().unwrap()),
    )
    .unwrap();
    let before = storage.read_all().unwrap();
    assert!(recover(&mut storage, archive(), mode, &authority, None).is_err());
    assert_eq!(storage.read_all().unwrap(), before);
    assert_eq!(
        child.read_verified(&storage).unwrap().len(),
        child.len as usize
    );
}
