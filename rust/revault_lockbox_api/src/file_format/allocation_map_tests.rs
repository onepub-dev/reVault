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
    let mode = mode(false, false, false);
    let index = Index::new(archive(), mode, None).unwrap();
    let mut storage = StorageBackend::memory(Vec::new());
    create_empty(
        &mut storage,
        archive(),
        mode,
        &Authority::Checksum,
        None,
        None,
    )
    .unwrap();
    let mut tx = Transaction::begin(storage, archive(), mode, &Authority::Checksum, None).unwrap();
    let payload = tx.append_encoded_extent(b"shared allocation").unwrap();
    let value = OwnedRecord::encode(b"shared", &[payload]).unwrap();
    tx.replace_all_sorted((0u32..1500).map(|n| Entry::new(1, &n.to_be_bytes(), &value)))
        .unwrap();
    let (storage, next) = tx.commit(&Authority::Checksum, None).unwrap();
    let snapshot = Snapshot::inspect(&storage, &next, &index).unwrap();
    assert_eq!(snapshot.accounting.payload, payload.len);
    let mut gap_storage = storage.clone();
    gap_storage.append(b"unowned").unwrap();
    let gap_anchor = Anchor {
        sealed_len: gap_storage.len().unwrap(),
        ..next.clone()
    };
    assert!(Snapshot::inspect(&gap_storage, &gap_anchor, &index).is_err());
    let mut child = None;
    index
        .visit_owned(&storage, next.index, next.sealed_len, |event| {
            if let Visit::Page(page) = event {
                if page != next.index {
                    child = Some(page);
                }
            }
            Ok(())
        })
        .unwrap();
    let child = child.unwrap();
    let (forged, forged_anchor) =
        forge_retired_child(storage, &next, child.primary, child.len, FREE);
    assert!(Snapshot::inspect(&forged, &forged_anchor, &index).is_err());
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
    let (mut storage, next) =
        forge_retired_child(storage, &base, child.primary, child.len, PENDING);
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
    assert!(repair_metadata(&mut storage, archive(), mode, &authority, None).is_err());
    assert_eq!(storage.read_all().unwrap(), before);
    assert_eq!(
        child.read_verified(&storage).unwrap().len(),
        child.len as usize
    );
}

// Deliberately bypass the allocator's publication audit to construct inconsistent
// authenticated ownership. The public CLI cannot produce this condition.
fn forge_retired_child(
    storage: StorageBackend,
    base: &Anchor,
    start: u64,
    len: u64,
    namespace: u8,
) -> (StorageBackend, Anchor) {
    let index = Index::new(archive(), base.mode, None).unwrap();
    let snapshot = Snapshot::inspect(&storage, base, &index).unwrap();
    let mut prepared =
        PreparedStore::begin(storage, archive(), base.mode, &Authority::Checksum, None).unwrap();
    let current = prepared.len().unwrap();
    let arena = Extent {
        start: align_region(current).unwrap(),
        len: 2 * FAILURE_REGION,
        digest: [0; 32],
    };
    append_zeros(&mut prepared, arena.start - current + arena.len).unwrap();
    let mut next = Anchor {
        generation: base.generation + 1,
        previous: base.commitment().unwrap(),
        sealed_len: prepared.len().unwrap(),
        ..base.clone()
    };
    let mut live = Snapshot::live(&prepared, &next, &index).unwrap();
    live.insert(
        Claim {
            extent: arena,
            kind: Kind::Reserve,
        },
        next.sealed_len,
    )
    .unwrap();
    let mut records = gap_records(&live, &snapshot, &Claims::default(), next.sealed_len).unwrap();
    records.push(Entry::new(ARENA, &arena.start.to_be_bytes(), &arena.len.to_le_bytes()).unwrap());
    records.push(Entry::new(namespace, &start.to_be_bytes(), &len.to_le_bytes()).unwrap());
    records.sort_by(|a, b| (a.namespace, a.key.as_slice()).cmp(&(b.namespace, b.key.as_slice())));
    let mut writer = ArenaWriter {
        storage: prepared,
        arena,
        position: Arc::new(Mutex::new(arena.start)),
    };
    next.allocation = index
        .build_sorted(&mut writer, records.into_iter().map(Ok))
        .unwrap()
        .root;
    (writer.storage.into_inner().unwrap(), next)
}

fn regional_fixture(
    mode: FormatMode,
    owner: &OwnerSigningKeyPair,
) -> (StorageBackend, Anchor, [Extent; 2]) {
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let signer = mode.signed().then_some(owner);
    let mut storage = StorageBackend::memory(Vec::new());
    create_empty(&mut storage, archive(), mode, &authority, signer, key(mode)).unwrap();
    let mut tx = Transaction::begin(storage, archive(), mode, &authority, key(mode)).unwrap();
    let first = tx
        .append_encoded_extent(&stored(mode, &vec![0x25; 70000]))
        .unwrap();
    let second = tx
        .append_encoded_extent(&stored(mode, &vec![0x37; 75000]))
        .unwrap();
    tx.replace_all_sorted((0u32..1500).map(|n| {
        let extents = match n {
            0 => vec![first],
            1499 => vec![second],
            _ => Vec::new(),
        };
        Entry::new(
            1,
            &n.to_be_bytes(),
            &OwnedRecord::encode(&[n as u8; 32], &extents)?,
        )
    }))
    .unwrap();
    tx.put_key_record(1, b"recipient", b"synthetic key record", &[])
        .unwrap();
    let (storage, anchor) = tx.commit(&authority, signer).unwrap();
    audit(&storage, &anchor);
    (storage, anchor, [first, second])
}
#[test]
fn every_single_aligned_region_loss_preserves_membership_and_intact_payloads() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for encrypted in [false, true] {
        for signed in [false, true] {
            for padded in [false, true] {
                let mode = mode(encrypted, signed, padded);
                let authority = authority(mode, &public);
                let (storage, anchor, extents) = regional_fixture(mode, &owner);
                let before = storage.read_all().unwrap();
                let index = Index::new(archive(), mode, key(mode)).unwrap();
                for start in (0..anchor.sealed_len).step_by(FAILURE_REGION as usize) {
                    let end = (start + FAILURE_REGION).min(anchor.sealed_len);
                    let mut damaged = StorageBackend::memory(before.clone());
                    damaged
                        .write_at(start, &vec![0x6d; (end - start) as usize])
                        .unwrap();
                    let selected =
                        publication::select(&damaged, archive(), mode, &authority).unwrap();
                    assert_eq!(selected.anchor, anchor);
                    let mut seen = 0;
                    let recovered = index
                        .recover(&damaged, anchor.index, anchor.sealed_len, |entry| {
                            let n = u32::from_be_bytes(entry.key.as_slice().try_into().unwrap());
                            assert_eq!(
                                OwnedRecord::decode(&entry.value)?.metadata.as_slice(),
                                &[n as u8; 32]
                            );
                            seen += 1;
                            Ok(())
                        })
                        .unwrap();
                    assert_eq!(seen, 1500);
                    assert_eq!(recovered.unavailable, 0);
                    let payload_before = extents
                        .map(|extent| damaged.read_at(extent.start, extent.len as usize).unwrap());
                    let (repaired,_)=repair_metadata(&mut damaged,archive(),mode,&authority,key(mode)).unwrap_or_else(|e|panic!("encrypted={encrypted}, signed={signed}, padded={padded}, region={start}: {e}"));
                    assert_eq!(repaired, anchor);
                    audit(&damaged, &anchor);
                    for (i, extent) in extents.iter().enumerate() {
                        let bytes = damaged.read_at(extent.start, extent.len as usize).unwrap();
                        assert_eq!(bytes, payload_before[i], "metadata repair modified payload");
                        if extent.start < end && extent.end().unwrap() > start {
                            assert_ne!(strong_checksum(&bytes), extent.digest);
                        } else {
                            let name = if i == 0 { 0u32 } else { 1499u32 };
                            assert_eq!(
                                content(&damaged, &anchor, &name.to_be_bytes()).unwrap(),
                                vec![
                                    if i == 0 { 0x25 } else { 0x37 };
                                    if i == 0 { 70000 } else { 75000 }
                                ]
                            );
                        }
                    }
                    let completed = damaged.read_all().unwrap();
                    let (_, again) =
                        repair_metadata(&mut damaged, archive(), mode, &authority, key(mode))
                            .unwrap();
                    assert_eq!(again.copies, 0);
                    assert_eq!(again.zeroed_unused_bytes, 0);
                    assert_eq!(damaged.read_all().unwrap(), completed);
                    cases += 1;
                }
            }
        }
    }
    eprintln!("single aligned-region damage cases: {cases}");
}
#[test]
fn loss_of_both_authority_copies_fails_closed_and_does_not_repair_from_history() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(true, true, false);
    let authority = authority(mode, &public);
    let (storage, anchor, _) = regional_fixture(mode, &owner);
    let bytes = storage.read_all().unwrap();
    for offsets in [
        [0, publication::SLOT_STRIDE as u64],
        [anchor.index.primary, anchor.index.mirror],
    ] {
        let mut damaged = StorageBackend::memory(bytes.clone());
        for offset in offsets {
            damaged.write_at(offset, &[0x4a; 32]).unwrap();
        }
        let before = damaged.read_all().unwrap();
        assert!(repair_metadata(&mut damaged, archive(), mode, &authority, key(mode)).is_err());
        assert_eq!(damaged.read_all().unwrap(), before);
    }
    let index = Index::new(archive(), mode, key(mode)).unwrap();
    let mut leaf = None;
    index
        .visit_owned(&storage, anchor.index, anchor.sealed_len, |event| {
            if let Visit::Page(page) = event {
                if page != anchor.index && leaf.is_none() {
                    leaf = Some(page);
                }
            }
            Ok(())
        })
        .unwrap();
    let leaf = leaf.unwrap();
    let mut damaged = StorageBackend::memory(bytes);
    for offset in [leaf.primary, leaf.mirror] {
        damaged
            .write_at(offset, &vec![0x91; leaf.len as usize])
            .unwrap();
    }
    let before = damaged.read_all().unwrap();
    assert!(repair_metadata(&mut damaged, archive(), mode, &authority, key(mode)).is_err());
    assert_eq!(damaged.read_all().unwrap(), before);
    let report = index
        .recover(&damaged, anchor.index, anchor.sealed_len, |_| Ok(()))
        .unwrap();
    assert!(report.unavailable > 0);
    assert!(report.recovered > 0);
    assert_eq!(report.recovered + report.unavailable, 1500);
}
#[test]
fn separated_pair_geometry_rejects_shared_regions_and_crossing_nodes() {
    let good = RootRef {
        primary: DATA_START,
        mirror: DATA_START + FAILURE_REGION,
        len: 32,
        digest: [0; 32],
    };
    separated(good).unwrap();
    for bad in [
        RootRef {
            mirror: good.primary + 32,
            ..good
        },
        RootRef {
            primary: good.primary + FAILURE_REGION - 16,
            ..good
        },
        RootRef {
            len: FAILURE_REGION + 1,
            ..good
        },
        RootRef { len: 0, ..good },
    ] {
        assert!(separated(bad).is_err());
    }
}

// Candidate storage is test-only; no public CLI can create these archives or
// inject torn physical writes. Every fault resumes through the same repair API.
#[test]
fn metadata_repair_survives_each_mutation_failure_and_power_loss() {
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut atomic_cases = 0;
    let mut power_cases = 0;
    for mode in [mode(false, false, false), mode(true, true, true)] {
        let authority = authority(mode, &public);
        let (original, anchor, extents) = regional_fixture(mode, &owner);
        let original = original.read_all().unwrap();
        for start in (0..anchor.sealed_len).step_by(FAILURE_REGION as usize) {
            let end = (start + FAILURE_REGION).min(anchor.sealed_len);
            let mut damaged = original.clone();
            damaged[start as usize..end as usize].fill(0x6d);
            let verify = |bytes: Vec<u8>| {
                let mut reopened = StorageBackend::memory(bytes);
                let (selected, _) =
                    repair_metadata(&mut reopened, archive(), mode, &authority, key(mode)).unwrap();
                assert_eq!(selected, anchor);
                audit(&reopened, &anchor);
                let index = Index::new(archive(), mode, key(mode)).unwrap();
                let mut count = 0;
                index
                    .visit(&reopened, anchor.index, anchor.sealed_len, |entry| {
                        let n = u32::from_be_bytes(entry.key.as_slice().try_into().unwrap());
                        assert_eq!(
                            OwnedRecord::decode(&entry.value)?.metadata.as_slice(),
                            &[n as u8; 32]
                        );
                        count += 1;
                        Ok(())
                    })
                    .unwrap();
                assert_eq!(count, 1500);
                for extent in extents {
                    let end = extent.end().unwrap() as usize;
                    assert_eq!(
                        reopened.read_at(extent.start, extent.len as usize).unwrap(),
                        damaged[extent.start as usize..end]
                    );
                }
            };
            let mut probe = Observed::new(damaged.clone());
            repair_metadata(&mut probe, archive(), mode, &authority, key(mode)).unwrap();
            for fail in 0..probe.count() {
                let mut failed = Observed::new(damaged.clone());
                failed.fail(fail);
                assert!(
                    repair_metadata(&mut failed, archive(), mode, &authority, key(mode)).is_err()
                );
                verify(failed.read_all().unwrap());
                atomic_cases += 1;
            }
            let mut probe = CrashStore::new(damaged.clone(), None, 0, false);
            repair_metadata(&mut probe, archive(), mode, &authority, key(mode)).unwrap();
            for fail in 0..probe.operations() {
                for prefix in [0, 1, 4096, 65536] {
                    for persist in [false, true] {
                        let mut failed =
                            CrashStore::new(damaged.clone(), Some(fail), prefix, persist);
                        assert!(repair_metadata(
                            &mut failed,
                            archive(),
                            mode,
                            &authority,
                            key(mode)
                        )
                        .is_err());
                        verify(failed.durable());
                        power_cases += 1;
                    }
                }
            }
        }
    }
    eprintln!(
        "metadata repair atomic failure cases: {atomic_cases}; power-loss cases: {power_cases}"
    );
}
