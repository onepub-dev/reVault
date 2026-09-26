use super::*;
use crate::storage::StorageBackend;
use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
use std::cell::RefCell;
use std::rc::Rc;

fn mode(encrypted: bool, signed: bool) -> FormatMode {
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
        size_padding: SizePadding::Default,
    })
}
fn roots(storage: &mut impl Storage, bytes: &[u8]) -> RootRef {
    let primary = storage.append(bytes).unwrap();
    let mirror = storage.append(bytes).unwrap();
    RootRef {
        primary,
        mirror,
        len: bytes.len() as u64,
        digest: strong_checksum(bytes),
    }
}
fn next(storage: &mut impl Storage, mode: FormatMode, old: Option<&Anchor>) -> Anchor {
    let generation = old.map_or(1, |old| old.generation + 1);
    let index = roots(
        storage,
        format!("root for generation {generation}").as_bytes(),
    );
    Anchor {
        archive: LockboxId::from_bytes([31; 16]),
        generation,
        mode,
        sealed_len: storage.len().unwrap(),
        object_root: index.digest,
        previous: old.map_or([0; 32], |old| old.commitment().unwrap()),
        index,
        allocation: RootRef::default(),
        keys: RootRef::default(),
    }
}
fn refresh_checksum(bytes: &mut [u8]) {
    let checksum = strong_checksum(&bytes[..CHECKSUM_START]);
    bytes[CHECKSUM_START..].copy_from_slice(&checksum);
}

#[test]
fn mirrored_publications_round_trip_all_modes_and_ignore_prepared_tail_records() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            let mode = mode(encrypted, signed);
            let authority = if signed {
                Authority::Owner(&public)
            } else if encrypted {
                Authority::Symmetric(b"synthetic key")
            } else {
                Authority::Checksum
            };
            let signer = signed.then_some(&owner);
            let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
            let first = next(&mut storage, mode, None);
            let published = publish(&mut storage, &first, &authority, signer, None).unwrap();
            assert_eq!(published.anchor, first);
            let initial = select(&storage, first.archive, mode, &authority).unwrap();
            assert_eq!(initial.copies, 3);
            let mut second = next(&mut storage, mode, Some(&first));
            // A valid signed publication-shaped record outside the two selected
            // slots is prepared data, not authority to select another generation.
            storage
                .append(&encode(&second, &authority, signer).unwrap())
                .unwrap();
            assert_eq!(
                select(&storage, first.archive, mode, &authority)
                    .unwrap()
                    .anchor,
                first
            );
            second.sealed_len = storage.len().unwrap();
            let published = publish(
                &mut storage,
                &second,
                &authority,
                signer,
                Some(published.commitment),
            )
            .unwrap();
            assert_eq!(published.anchor, second);
            let reopened = StorageBackend::memory(storage.read_all().unwrap());
            let selected = select(&reopened, first.archive, mode, &authority).unwrap();
            assert_eq!(selected.anchor, second);
            assert_eq!(selected.copies, 3);
            assert_eq!(
                selected.anchor.index.read_verified(&reopened).unwrap(),
                b"root for generation 2"
            );
        }
    }
}

#[test]
fn owner_mode_archive_and_every_signed_field_are_pinned() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let attacker = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let attacker_public = attacker.public_key();
    let authority = Authority::Owner(&public);
    let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
    let anchor = next(&mut storage, mode(true, true), None);
    let encoded = encode(&anchor, &authority, Some(&owner)).unwrap();
    for offset in [
        8, 10, 12, 16, 24, 40, 48, 56, 88, 120, 128, 136, 144, 176, 232,
    ] {
        let mut modified = encoded.clone();
        modified[offset] ^= 1;
        refresh_checksum(&mut modified);
        assert!(
            decode(&modified, anchor.archive, anchor.mode, &authority).is_err(),
            "offset {offset}"
        );
    }
    assert!(decode(
        &encoded,
        LockboxId::from_bytes([32; 16]),
        anchor.mode,
        &authority
    )
    .is_err());
    assert!(decode(
        &encoded,
        anchor.archive,
        mode(true, false),
        &Authority::Symmetric(b"known content key")
    )
    .is_err());
    let forged = encode(
        &anchor,
        &Authority::Owner(&attacker_public),
        Some(&attacker),
    )
    .unwrap();
    assert!(decode(&forged, anchor.archive, anchor.mode, &authority).is_err());
    for (offset, bytes) in [(288, u32::MAX.to_le_bytes()), (324, u32::MAX.to_le_bytes())] {
        let mut oversized = encoded.clone();
        oversized[offset..offset + 4].copy_from_slice(&bytes);
        refresh_checksum(&mut oversized);
        assert!(decode(&oversized, anchor.archive, anchor.mode, &authority).is_err());
    }
    let mut reserved = encoded.clone();
    reserved[299] = 1;
    refresh_checksum(&mut reserved);
    assert!(decode(&reserved, anchor.archive, anchor.mode, &authority).is_err());
    for invalid in [
        RootRef {
            primary: 0,
            ..anchor.index
        },
        RootRef {
            mirror: anchor.index.primary,
            ..anchor.index
        },
        RootRef {
            len: u64::MAX,
            ..anchor.index
        },
        RootRef {
            primary: u64::MAX,
            ..anchor.index
        },
    ] {
        let mut bad = anchor.clone();
        bad.index = invalid;
        assert!(encode(&bad, &authority, Some(&owner)).is_err());
    }
}

#[test]
fn damaged_roots_never_authorize_rollback_to_an_older_publication() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = Authority::Owner(&public);
    let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
    let first = next(&mut storage, mode(false, true), None);
    let first_pub = publish(&mut storage, &first, &authority, Some(&owner), None).unwrap();
    let second = next(&mut storage, first.mode, Some(&first));
    publish(
        &mut storage,
        &second,
        &authority,
        Some(&owner),
        Some(first_pub.commitment),
    )
    .unwrap();
    // Tail loss leaves the selected generation authenticated and one index copy
    // intact. It is not permission to resurrect a prior membership set.
    storage
        .truncate(second.index.primary + second.index.len)
        .unwrap();
    let selected = select(&storage, first.archive, first.mode, &authority).unwrap();
    assert_eq!(selected.anchor.generation, 2);
    assert_eq!(
        selected.anchor.index.read_verified(&storage).unwrap(),
        b"root for generation 2"
    );
    assert!(ensure_mirrored(
        &mut storage,
        first.archive,
        first.mode,
        &authority,
        selected.commitment
    )
    .is_err());
    storage
        .write_at(second.index.primary, &vec![0; second.index.len as usize])
        .unwrap();
    assert!(selected.anchor.index.read_verified(&storage).is_err());
    assert_eq!(
        select(&storage, first.archive, first.mode, &authority)
            .unwrap()
            .anchor
            .generation,
        2
    );
}

#[test]
fn one_slot_can_be_lost_but_conflicting_or_unlinked_valid_slots_are_rejected() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = Authority::Owner(&public);
    let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
    let first = next(&mut storage, mode(false, true), None);
    let base = encode(&first, &authority, Some(&owner)).unwrap();
    for slot in 0..2 {
        storage.write_at(0, &vec![0; REGION_LEN]).unwrap();
        storage.write_at((slot * SLOT_LEN) as u64, &base).unwrap();
        assert_eq!(
            select(&storage, first.archive, first.mode, &authority)
                .unwrap()
                .anchor,
            first
        );
        ensure_mirrored(
            &mut storage,
            first.archive,
            first.mode,
            &authority,
            first.commitment().unwrap(),
        )
        .unwrap();
        assert_eq!(
            select(&storage, first.archive, first.mode, &authority)
                .unwrap()
                .copies,
            3
        );
    }
    let mut fork = first.clone();
    fork.object_root[0] ^= 1;
    storage
        .write_at(
            SLOT_LEN as u64,
            &encode(&fork, &authority, Some(&owner)).unwrap(),
        )
        .unwrap();
    assert!(select(&storage, first.archive, first.mode, &authority).is_err());
    let mut unlinked = next(&mut storage, first.mode, Some(&first));
    unlinked.previous[0] ^= 1;
    storage
        .write_at(
            SLOT_LEN as u64,
            &encode(&unlinked, &authority, Some(&owner)).unwrap(),
        )
        .unwrap();
    assert!(select(&storage, first.archive, first.mode, &authority).is_err());
    storage.write_at(0, &vec![0; REGION_LEN]).unwrap();
    assert!(select(&storage, first.archive, first.mode, &authority).is_err());
}

#[test]
fn every_mutating_storage_failure_preserves_a_selected_complete_generation() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = Authority::Owner(&public);
    let mut base = StorageBackend::memory(vec![0; REGION_LEN]);
    let first = next(&mut base, mode(true, true), None);
    let old = publish(&mut base, &first, &authority, Some(&owner), None).unwrap();
    let second = next(&mut base, first.mode, Some(&first));
    let bytes = base.read_all().unwrap();
    base.reset_memory_operation_count();
    publish(
        &mut base,
        &second,
        &authority,
        Some(&owner),
        Some(old.commitment),
    )
    .unwrap();
    let count = base.memory_operation_count();
    assert!(count >= 5);
    for failure in 0..count {
        let mut broken = StorageBackend::memory(bytes.clone());
        broken.fail_memory_operation_after_successes(failure);
        assert!(
            publish(
                &mut broken,
                &second,
                &authority,
                Some(&owner),
                Some(old.commitment)
            )
            .is_err(),
            "failure {failure}"
        );
        let mut reopened = StorageBackend::memory(broken.read_all().unwrap());
        let selected = select(&reopened, first.archive, first.mode, &authority).unwrap();
        assert!(
            selected.anchor == first || selected.anchor == second,
            "failure {failure}"
        );
        selected.anchor.verify_dependencies(&reopened).unwrap();
        let token = ensure_mirrored(
            &mut reopened,
            first.archive,
            first.mode,
            &authority,
            selected.commitment,
        )
        .unwrap();
        assert_eq!(token.anchor, selected.anchor);
        assert_eq!(
            select(&reopened, first.archive, first.mode, &authority)
                .unwrap()
                .copies,
            3
        );
    }
}

#[derive(Debug, Clone)]
struct CrashStore(Rc<RefCell<CrashState>>);
#[derive(Debug, Clone)]
struct CrashState {
    volatile: Vec<u8>,
    durable: Vec<u8>,
    operations: usize,
    fail: Option<usize>,
    persist: bool,
    prefix: usize,
}
impl CrashStore {
    fn new(bytes: Vec<u8>) -> Self {
        Self(Rc::new(RefCell::new(CrashState {
            volatile: bytes.clone(),
            durable: bytes,
            operations: 0,
            fail: None,
            persist: false,
            prefix: 0,
        })))
    }
    fn fail(&self, operation: usize, persist: bool, prefix: usize) {
        let mut state = self.0.borrow_mut();
        state.operations = 0;
        state.fail = Some(operation);
        state.persist = persist;
        state.prefix = prefix;
    }
    fn crash(&self) -> StorageBackend {
        StorageBackend::memory(self.0.borrow().durable.clone())
    }
}
impl Storage for CrashStore {
    fn len(&self) -> Result<u64> {
        Ok(self.0.borrow().volatile.len() as u64)
    }
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        let mut bytes = vec![0; len];
        self.read_at_into(offset, &mut bytes)?;
        Ok(bytes)
    }
    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        let state = self.0.borrow();
        let start = offset as usize;
        let bytes = state
            .volatile
            .get(start..start + out.len())
            .ok_or(Error::Truncated)?;
        out.copy_from_slice(bytes);
        Ok(())
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        let mut state = self.0.borrow_mut();
        let offset = state.volatile.len();
        state.volatile.extend_from_slice(bytes);
        Ok(offset as u64)
    }
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        let mut state = self.0.borrow_mut();
        let operation = state.operations;
        state.operations += 1;
        let start = offset as usize;
        if state.fail == Some(operation) {
            let count = state.prefix.min(bytes.len());
            state.volatile[start..start + count].copy_from_slice(&bytes[..count]);
            if state.persist {
                state.durable[start..start + count].copy_from_slice(&bytes[..count]);
            }
            return Err(Error::Io("injected torn write".into()));
        }
        state.volatile[start..start + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }
    fn truncate(&mut self, len: u64) -> Result<()> {
        self.0.borrow_mut().volatile.truncate(len as usize);
        Ok(())
    }
    fn sync(&self) -> Result<()> {
        let mut state = self.0.borrow_mut();
        let operation = state.operations;
        state.operations += 1;
        let fail = state.fail == Some(operation);
        if !fail || state.persist {
            state.durable = state.volatile.clone();
        }
        if fail {
            Err(Error::Io("injected sync failure".into()))
        } else {
            Ok(())
        }
    }
}

#[test]
fn power_loss_and_failed_final_sync_do_not_grant_cleanup_early() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = Authority::Owner(&public);
    let mut base = CrashStore::new(vec![0; REGION_LEN]);
    let first = next(&mut base, mode(false, true), None);
    let old = publish(&mut base, &first, &authority, Some(&owner), None).unwrap();
    let original = base.0.borrow().durable.clone();
    for failure in 0..5 {
        for persisted in [false, true] {
            for prefix in [
                0,
                1,
                288,
                AUTH_START,
                4096,
                CHECKSUM_START,
                SLOT_LEN - 1,
                SLOT_LEN,
            ] {
                let mut storage = CrashStore::new(original.clone());
                let second = next(&mut storage, first.mode, Some(&first));
                storage.fail(failure, persisted, prefix);
                assert!(publish(
                    &mut storage,
                    &second,
                    &authority,
                    Some(&owner),
                    Some(old.commitment)
                )
                .is_err());
                let crashed = storage.crash();
                let selected = select(&crashed, first.archive, first.mode, &authority).unwrap();
                assert!(selected.anchor == first || selected.anchor == second);
                selected.anchor.verify_dependencies(&crashed).unwrap();
            }
        }
    }
    let mut storage = CrashStore::new(original);
    let second = next(&mut storage, first.mode, Some(&first));
    storage.fail(4, false, 0); // Final sync fails after both readable writes.
    assert!(publish(
        &mut storage,
        &second,
        &authority,
        Some(&owner),
        Some(old.commitment)
    )
    .is_err());
    let selected = select(&storage, first.archive, first.mode, &authority).unwrap();
    assert_eq!(selected.copies, 3);
    storage.fail(0, false, 0);
    assert!(ensure_mirrored(
        &mut storage,
        first.archive,
        first.mode,
        &authority,
        selected.commitment
    )
    .is_err());
    storage.0.borrow_mut().fail = None;
    ensure_mirrored(
        &mut storage,
        first.archive,
        first.mode,
        &authority,
        selected.commitment,
    )
    .unwrap();
    assert_eq!(
        select(&storage.crash(), first.archive, first.mode, &authority)
            .unwrap()
            .copies,
        3
    );
}

#[derive(Debug, Clone)]
struct ProcessStore {
    inner: StorageBackend,
    calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    stop: usize,
}
impl ProcessStore {
    fn boundary(&self) {
        if self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == self.stop {
            use std::io::Write;
            println!("REVAULT_PUBLICATION_READY");
            std::io::stdout().flush().unwrap();
            loop {
                std::thread::park();
            }
        }
    }
}
impl Storage for ProcessStore {
    fn len(&self) -> Result<u64> {
        self.inner.len()
    }
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        self.inner.read_at(offset, len)
    }
    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        self.inner.read_at_into(offset, out)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        self.inner.append(bytes)
    }
    fn truncate(&mut self, len: u64) -> Result<()> {
        self.inner.truncate(len)
    }
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        self.inner.write_at(offset, bytes)?;
        self.boundary();
        Ok(())
    }
    fn sync(&self) -> Result<()> {
        self.inner.sync()?;
        self.boundary();
        Ok(())
    }
}

#[test]
fn publication_process_child() {
    let Some(file) = std::env::var_os("REVAULT_PUBLICATION_TEST_PATH") else {
        return;
    };
    let file = std::path::PathBuf::from(file);
    let stop = std::env::var("REVAULT_PUBLICATION_TEST_STOP")
        .unwrap()
        .parse()
        .unwrap();
    let resume = std::env::var("REVAULT_PUBLICATION_TEST_ROLE").unwrap() == "resume";
    let mode = mode(false, true);
    if resume {
        let public = OwnerSigningPublicKey::from_bytes(
            &std::fs::read(file.with_extension("public")).unwrap(),
        )
        .unwrap();
        let authority = Authority::Owner(&public);
        let inner = StorageBackend::file_for_write(&file).unwrap();
        let selected = select(&inner, LockboxId::from_bytes([31; 16]), mode, &authority).unwrap();
        let mut storage = ProcessStore {
            inner,
            stop,
            calls: Default::default(),
        };
        ensure_mirrored(
            &mut storage,
            selected.anchor.archive,
            mode,
            &authority,
            selected.commitment,
        )
        .unwrap();
    } else {
        let owner = OwnerSigningKeyPair::generate().unwrap();
        let public = owner.public_key();
        let authority = Authority::Owner(&public);
        // Only the public verification key crosses process boundaries. The
        // generated synthetic private signing key never leaves this child.
        std::fs::write(file.with_extension("public"), public.to_bytes()).unwrap();
        let mut inner = StorageBackend::create_file(&file, &vec![0; REGION_LEN]).unwrap();
        let first = next(&mut inner, mode, None);
        let old = publish(&mut inner, &first, &authority, Some(&owner), None).unwrap();
        let second = next(&mut inner, mode, Some(&first));
        let mut storage = ProcessStore {
            inner,
            stop,
            calls: Default::default(),
        };
        publish(
            &mut storage,
            &second,
            &authority,
            Some(&owner),
            Some(old.commitment),
        )
        .unwrap();
    }
    panic!("publication checkpoint was not reached");
}

fn kill_at_boundary(file: &std::path::Path, role: &str, stop: usize) {
    use std::io::BufRead;
    use std::process::{Command, Stdio};
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "file_format::publication_anchor::tests::publication_process_child",
            "--nocapture",
        ])
        .env("REVAULT_PUBLICATION_TEST_PATH", file)
        .env("REVAULT_PUBLICATION_TEST_ROLE", role)
        .env("REVAULT_PUBLICATION_TEST_STOP", stop.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let (send, receive) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        let found = std::io::BufReader::new(stdout)
            .lines()
            .any(|line| line.is_ok_and(|line| line == "REVAULT_PUBLICATION_READY"));
        let _ = send.send(found);
    });
    let ready = receive.recv_timeout(std::time::Duration::from_secs(30));
    let _ = child.kill();
    let status = child.wait().unwrap();
    reader.join().unwrap();
    assert!(
        matches!(ready, Ok(true)),
        "did not reach {role}/{stop}: {status}"
    );
    assert!(!status.success());
}

#[test]
fn publication_and_mirror_repair_survive_real_process_death() {
    // This is a storage-protocol unit test. No public CLI can create this
    // unactivated candidate format or stop at its internal sync boundaries.
    struct Directory(std::path::PathBuf);
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let directory = Directory(std::env::temp_dir().join(
        format!("revault-publication-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()),
    ));
    std::fs::create_dir(&directory.0).unwrap();
    for (role, stops) in [("publish", 5), ("resume", 3)] {
        for stop in 0..stops {
            let file = directory.0.join(format!("{role}-{stop}.candidate"));
            if role == "resume" {
                kill_at_boundary(&file, "publish", 2);
            }
            kill_at_boundary(&file, role, stop);
            let public = OwnerSigningPublicKey::from_bytes(
                &std::fs::read(file.with_extension("public")).unwrap(),
            )
            .unwrap();
            let authority = Authority::Owner(&public);
            let mut storage = StorageBackend::file_for_write(&file).unwrap();
            let selected = select(
                &storage,
                LockboxId::from_bytes([31; 16]),
                mode(false, true),
                &authority,
            )
            .unwrap();
            assert!(selected.anchor.generation == 1 || selected.anchor.generation == 2);
            if role == "resume" || stop >= 2 {
                assert_eq!(selected.anchor.generation, 2);
            }
            let bytes = selected.anchor.index.read_verified(&storage).unwrap();
            assert_eq!(
                bytes,
                format!("root for generation {}", selected.anchor.generation).as_bytes()
            );
            ensure_mirrored(
                &mut storage,
                selected.anchor.archive,
                selected.anchor.mode,
                &authority,
                selected.commitment,
            )
            .unwrap();
            drop(storage);
            let reopened = StorageBackend::file(&file).unwrap();
            let completed = select(
                &reopened,
                selected.anchor.archive,
                selected.anchor.mode,
                &authority,
            )
            .unwrap();
            assert_eq!(completed.copies, 3);
            assert_eq!(completed.commitment, selected.commitment);
        }
    }
}

#[test]
fn symmetric_authentication_rejects_wrong_keys_modified_fields_and_short_tags() {
    let mode = mode(true, false);
    let authority = Authority::Symmetric(b"synthetic publication key");
    let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
    let first = next(&mut storage, mode, None);
    let encoded = encode(&first, &authority, None).unwrap();
    assert!(decode(
        &encoded,
        first.archive,
        mode,
        &Authority::Symmetric(b"wrong key")
    )
    .is_err());
    let mut modified = encoded.clone();
    modified[56] ^= 1;
    refresh_checksum(&mut modified);
    assert!(decode(&modified, first.archive, mode, &authority).is_err());
    for len in [0u32, 1, 23, 24, 31, 33] {
        let mut shortened = encoded.clone();
        shortened[288..292].copy_from_slice(&len.to_le_bytes());
        shortened[AUTH_START + len as usize..CHECKSUM_START].fill(0);
        refresh_checksum(&mut shortened);
        assert!(decode(&shortened, first.archive, mode, &authority).is_err());
    }
}

#[test]
fn slot_read_errors_do_not_authorize_selecting_the_other_generation() {
    let mode = mode(false, false);
    let authority = Authority::Checksum;
    let mut base = StorageBackend::memory(vec![0; REGION_LEN]);
    let first = next(&mut base, mode, None);
    publish(&mut base, &first, &authority, None, None).unwrap();
    let second = next(&mut base, mode, Some(&first));
    base.write_at(SLOT_LEN as u64, &encode(&second, &authority, None).unwrap())
        .unwrap();
    #[derive(Debug, Clone)]
    struct ReadFault {
        inner: StorageBackend,
        offset: u64,
    }
    impl Storage for ReadFault {
        fn len(&self) -> Result<u64> {
            self.inner.len()
        }
        fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
            if offset == self.offset {
                return Err(Error::Io("injected slot read failure".into()));
            }
            self.inner.read_at(offset, len)
        }
        fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
            out.copy_from_slice(&self.read_at(offset, out.len())?);
            Ok(())
        }
        fn append(&mut self, bytes: &[u8]) -> Result<u64> {
            self.inner.append(bytes)
        }
        fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
            self.inner.write_at(offset, bytes)
        }
        fn truncate(&mut self, len: u64) -> Result<()> {
            self.inner.truncate(len)
        }
        fn sync(&self) -> Result<()> {
            self.inner.sync()
        }
    }
    for offset in [0, SLOT_LEN as u64] {
        let broken = ReadFault {
            inner: StorageBackend::memory(base.read_all().unwrap()),
            offset,
        };
        assert!(select(&broken, first.archive, mode, &authority).is_err());
    }
}

#[test]
fn every_byte_prefix_of_a_torn_slot_keeps_an_old_or_new_authenticated_root() {
    let mode = mode(false, false);
    let authority = Authority::Checksum;
    let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
    let first = next(&mut storage, mode, None);
    let old = encode(&first, &authority, None).unwrap();
    let second = next(&mut storage, mode, Some(&first));
    let new = encode(&second, &authority, None).unwrap();
    for overwriting_mirror in [false, true] {
        let other = if overwriting_mirror { &new } else { &old };
        for prefix in 0..=SLOT_LEN {
            let mut torn = old.clone();
            torn[..prefix].copy_from_slice(&new[..prefix]);
            storage.write_at(0, &torn).unwrap();
            storage.write_at(SLOT_LEN as u64, other).unwrap();
            let selected = select(&storage, first.archive, mode, &authority).unwrap();
            assert!(
                selected.anchor == first || selected.anchor == second,
                "prefix {prefix}"
            );
            if overwriting_mirror {
                assert_eq!(selected.anchor, second);
            }
            selected.anchor.verify_dependencies(&storage).unwrap();
        }
    }
}

#[test]
fn independent_checksum_and_hmac_vectors_match_exact_bytes() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/publication_anchor_v1.json"
    ))
    .unwrap();
    let unhex = |text: &str| {
        (0..text.len())
            .step_by(2)
            .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
            .collect::<Vec<_>>()
    };
    for vector in fixture["vectors"].as_array().unwrap() {
        let mode = FormatMode::parse(vector["mode"].as_u64().unwrap() as u16).unwrap();
        let authority = if mode.plaintext() {
            Authority::Checksum
        } else {
            Authority::Symmetric(fixture["synthetic_key"].as_str().unwrap().as_bytes())
        };
        let mut bytes = vec![0; SLOT_LEN];
        bytes[..PREFIX_LEN].copy_from_slice(&unhex(vector["prefix_hex"].as_str().unwrap()));
        let auth = unhex(vector["auth_hex"].as_str().unwrap());
        bytes[PREFIX_LEN..PREFIX_LEN + 4].copy_from_slice(&(auth.len() as u32).to_le_bytes());
        bytes[AUTH_START..AUTH_START + auth.len()].copy_from_slice(&auth);
        bytes[CHECKSUM_START..].copy_from_slice(&unhex(vector["checksum_hex"].as_str().unwrap()));
        let decoded = decode(&bytes, LockboxId::from_bytes([31; 16]), mode, &authority).unwrap();
        assert_eq!(decoded.generation, 1);
        assert_eq!(decoded.sealed_len, 16390);
        assert_eq!(decoded.index.primary, 16384);
        assert_eq!(decoded.index.mirror, 16387);
        assert_eq!(decoded.index.len, 3);
        assert_eq!(decoded.index.digest, strong_checksum(b"abc"));
        assert_eq!(
            decoded.commitment().unwrap().as_slice(),
            unhex(vector["commitment_hex"].as_str().unwrap())
        );
        assert_eq!(encode(&decoded, &authority, None).unwrap(), bytes);
    }
}

#[test]
fn preparation_signatures_cannot_be_reused_as_publication_authority() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = Authority::Owner(&public);
    let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
    let anchor = next(&mut storage, mode(true, true), None);
    let preparation_message = b"revault-experiment-owner-root-v1\0prepared membership only";
    let signatures = owner.sign(preparation_message);
    crate::signing::verify_commit_signatures(preparation_message, &signatures).unwrap();
    let mut encoded = encode(&anchor, &authority, Some(&owner)).unwrap();
    let mut position = AUTH_START + 2;
    for signature in signatures {
        let key_len =
            u32::from_le_bytes(encoded[position + 2..position + 6].try_into().unwrap()) as usize;
        let signature_len =
            u32::from_le_bytes(encoded[position + 6..position + 10].try_into().unwrap()) as usize;
        position += 10 + key_len;
        assert_eq!(signature_len, signature.signature.len());
        encoded[position..position + signature_len].copy_from_slice(&signature.signature);
        position += signature_len;
    }
    refresh_checksum(&mut encoded);
    assert!(decode(&encoded, anchor.archive, anchor.mode, &authority).is_err());
}
