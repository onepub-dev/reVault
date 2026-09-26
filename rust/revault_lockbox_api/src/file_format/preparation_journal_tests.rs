//! Internal protocol tests: the public CLI does not activate the candidate format.
use super::*;
use crate::file_format::authenticated_index::Entry;
use crate::file_format::publication_anchor::RootRef;
use crate::storage::StorageBackend;
use crate::{
    Compression, EncryptionMode, LockboxFormatOptions, OwnerSigningPublicKey, SigningMode,
    SizePadding,
};
const KEY: &[u8; 32] = &[22; 32];
const OLD: &[u8] = b"original committed bytes";
const NEW: &[u8] = b"replacement committed bytes";
#[derive(Clone, Debug)]
struct Observed(Arc<Mutex<StorageBackend>>, Arc<Mutex<Counts>>);
#[derive(Default, Clone, Debug)]
struct Counts {
    reads: u64,
    read_bytes: u64,
    writes: u64,
    write_bytes: u64,
    syncs: u64,
    truncates: u64,
}
impl Observed {
    fn new(bytes: Vec<u8>) -> Self {
        Self::backend(StorageBackend::memory(bytes))
    }
    fn backend(storage: StorageBackend) -> Self {
        Self(
            Arc::new(Mutex::new(storage)),
            Arc::new(Mutex::new(Counts::default())),
        )
    }
    fn stats(&self) -> Counts {
        self.1.lock().unwrap().clone()
    }
    fn reset_stats(&self) {
        *self.1.lock().unwrap() = Counts::default();
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
        {
            let mut c = self.1.lock().unwrap();
            c.reads += 1;
            c.read_bytes += n as u64;
        }
        self.0.lock().unwrap().read_at(o, n)
    }
    fn read_at_into(&self, o: u64, out: &mut [u8]) -> Result<()> {
        out.copy_from_slice(&self.read_at(o, out.len())?);
        Ok(())
    }
    fn append(&mut self, b: &[u8]) -> Result<u64> {
        {
            let mut c = self.1.lock().unwrap();
            c.writes += 1;
            c.write_bytes += b.len() as u64;
        }
        self.0.lock().unwrap().append(b)
    }
    fn write_at(&mut self, o: u64, b: &[u8]) -> Result<()> {
        {
            let mut c = self.1.lock().unwrap();
            c.writes += 1;
            c.write_bytes += b.len() as u64;
        }
        self.0.lock().unwrap().write_at(o, b)
    }
    fn truncate(&mut self, n: u64) -> Result<()> {
        self.1.lock().unwrap().truncates += 1;
        self.0.lock().unwrap().truncate(n)
    }
    fn sync(&self) -> Result<()> {
        self.1.lock().unwrap().syncs += 1;
        self.0.lock().unwrap().sync()
    }
}
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
        size_padding: SizePadding::None,
    })
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
fn key(mode: FormatMode) -> Option<&'static [u8]> {
    (!mode.plaintext()).then_some(KEY.as_slice())
}
fn descriptor(offset: u64, bytes: &[u8]) -> Vec<u8> {
    [
        offset.to_le_bytes().as_slice(),
        &(bytes.len() as u64).to_le_bytes(),
        &strong_checksum(bytes),
    ]
    .concat()
}
fn range(namespace: u8, start: u64, len: u64) -> Entry {
    Entry::new(namespace, &start.to_be_bytes(), &len.to_le_bytes()).unwrap()
}
struct Fixture {
    storage: Observed,
    anchor: Anchor,
    free: u64,
    live: u64,
    metadata: Vec<RootRef>,
}
fn fixture(mode: FormatMode, owner: &OwnerSigningKeyPair) -> Fixture {
    fixture_size(mode, owner, 8192)
}
fn fixture_size(mode: FormatMode, owner: &OwnerSigningKeyPair, free_len: usize) -> Fixture {
    let mut storage = Observed::new(vec![0; DATA_START as usize]);
    let free = storage.append(&vec![0; free_len]).unwrap();
    let live = storage.append(OLD).unwrap();
    let archive = LockboxId::from_bytes([82; 16]);
    let index = Index::new(archive, mode, key(mode)).unwrap();
    let logical = index
        .build_sorted(
            &mut storage,
            [Entry::new(1, b"/keep", &descriptor(live, OLD))],
        )
        .unwrap();
    let allocation = index
        .build_sorted(&mut storage, [Ok(range(FREE, free, free_len as u64))])
        .unwrap();
    let anchor = Anchor {
        archive,
        generation: 1,
        mode,
        sealed_len: storage.len().unwrap(),
        object_root: logical.root.digest,
        previous: [0; 32],
        index: logical.root,
        allocation: allocation.root,
        keys: RootRef::default(),
    };
    let public = owner.public_key();
    let authority = authority(mode, &public);
    publication::publish(
        &mut storage,
        &anchor,
        &authority,
        mode.signed().then_some(owner),
        None,
    )
    .unwrap();
    initialize(&mut storage, archive, mode, &authority, key(mode)).unwrap();
    let metadata = logical
        .created
        .into_iter()
        .chain(allocation.created)
        .collect();
    Fixture {
        storage,
        anchor,
        free,
        live,
        metadata,
    }
}
fn verify(storage: &impl Storage, anchor: &Anchor, expected: &[u8]) {
    let index = Index::new(anchor.archive, anchor.mode, key(anchor.mode)).unwrap();
    let entry = index
        .get(storage, anchor.index, anchor.sealed_len, 1, b"/keep")
        .unwrap()
        .unwrap();
    let offset = u64::from_le_bytes(entry.value[..8].try_into().unwrap());
    let len = u64::from_le_bytes(entry.value[8..16].try_into().unwrap());
    let bytes = storage.read_at(offset, len as usize).unwrap();
    assert_eq!(bytes, expected);
    assert_eq!(strong_checksum(&bytes), entry.value[16..]);
}
fn perform<S: Storage>(
    f: &Fixture,
    storage: S,
    owner: &OwnerSigningKeyPair,
    commit: bool,
) -> Result<()> {
    let public = owner.public_key();
    let authority = authority(f.anchor.mode, &public);
    let mut prepared = PreparedStore::begin(
        storage,
        f.anchor.archive,
        f.anchor.mode,
        &authority,
        key(f.anchor.mode),
    )?;
    prepared.reserve(&[Reservation {
        namespace: FREE,
        base: f.free,
        start: f.free,
        len: 4096,
    }])?;
    prepared.write_at(f.free, NEW)?;
    let garbage = prepared.append(b"unpublished tail bytes")?;
    prepared.write_at(garbage, b"UN")?;
    let index = Index::new(f.anchor.archive, f.anchor.mode, key(f.anchor.mode))?;
    let sealed = prepared.len()?;
    let logical = index.put(
        &mut prepared,
        f.anchor.index,
        sealed,
        Entry::new(1, b"/keep", &descriptor(f.free, NEW))?,
    )?;
    if !commit {
        prepared.abort(&authority)?;
        return Ok(());
    }
    let mut records = vec![
        range(FREE, f.free + NEW.len() as u64, 8192 - NEW.len() as u64),
        range(PENDING, f.live, OLD.len() as u64),
        range(PENDING, garbage, b"unpublished tail bytes".len() as u64),
    ];
    for r in &f.metadata {
        records.push(range(PENDING, r.primary, r.len));
        records.push(range(PENDING, r.mirror, r.len));
    }
    records.sort_by(|a, b| (a.namespace, a.key.as_slice()).cmp(&(b.namespace, b.key.as_slice())));
    let allocation = index.build_sorted(&mut prepared, records.into_iter().map(Ok))?;
    let next = Anchor {
        generation: 2,
        sealed_len: prepared.len()?,
        object_root: logical.root.digest,
        previous: f.anchor.commitment()?,
        index: logical.root,
        allocation: allocation.root,
        ..f.anchor.clone()
    };
    prepared.commit(&next, &authority, f.anchor.mode.signed().then_some(owner))?;
    prepared.into_inner()?;
    Ok(())
}
#[test]
fn preparation_abort_and_commit_reopen_all_modes_and_erase_only_authorized_bytes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            let mode = mode(encrypted, signed);
            let f = fixture(mode, &owner);
            let authority = authority(mode, &public);
            perform(&f, f.storage.clone(), &owner, false).unwrap();
            let mut reopened = Observed::new(f.storage.read_all().unwrap());
            recover(&mut reopened, f.anchor.archive, mode, &authority, key(mode)).unwrap();
            assert_eq!(reopened.len().unwrap(), f.anchor.sealed_len);
            verify(&reopened, &f.anchor, OLD);
            assert!(reopened
                .read_at(f.free, 8192)
                .unwrap()
                .iter()
                .all(|b| *b == 0));
            perform(&f, reopened.clone(), &owner, true).unwrap();
            let mut reopened = Observed::new(reopened.read_all().unwrap());
            recover(&mut reopened, f.anchor.archive, mode, &authority, key(mode)).unwrap();
            let selected =
                publication::select(&reopened, f.anchor.archive, mode, &authority).unwrap();
            assert_eq!(selected.anchor.generation, 2);
            verify(&reopened, &selected.anchor, NEW);
            assert!(reopened
                .read_at(f.live, OLD.len())
                .unwrap()
                .iter()
                .all(|b| *b == 0));
            for r in &f.metadata {
                for offset in [r.primary, r.mirror] {
                    assert!(reopened
                        .read_at(offset, r.len as usize)
                        .unwrap()
                        .iter()
                        .all(|b| *b == 0));
                }
            }
            let bytes = reopened.read_all().unwrap();
            recover(&mut reopened, f.anchor.archive, mode, &authority, key(mode)).unwrap();
            assert_eq!(reopened.read_all().unwrap(), bytes);
        }
    }
}
#[test]
fn every_mutating_failure_during_abort_or_commit_can_resume_to_a_complete_generation() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let f = fixture(mode(true, true), &owner);
    let authority = authority(f.anchor.mode, &public);
    let bytes = f.storage.read_all().unwrap();
    for commit in [false, true] {
        let probe = Observed::new(bytes.clone());
        perform(&f, probe.clone(), &owner, commit).unwrap();
        let operations = probe.count();
        assert!(operations > 20);
        for fail in 0..operations {
            let storage = Observed::new(bytes.clone());
            storage.fail(fail);
            assert!(
                perform(&f, storage.clone(), &owner, commit).is_err(),
                "commit={commit} operation={fail}"
            );
            let mut reopened = Observed::new(storage.read_all().unwrap());
            recover(
                &mut reopened,
                f.anchor.archive,
                f.anchor.mode,
                &authority,
                key(f.anchor.mode),
            )
            .unwrap_or_else(|e| panic!("commit={commit} operation={fail}: {e}"));
            let selected =
                publication::select(&reopened, f.anchor.archive, f.anchor.mode, &authority)
                    .unwrap();
            assert_eq!(reopened.len().unwrap(), selected.anchor.sealed_len);
            if selected.anchor.generation == 1 {
                verify(&reopened, &selected.anchor, OLD);
                assert!(reopened
                    .read_at(f.free, 8192)
                    .unwrap()
                    .iter()
                    .all(|b| *b == 0));
            } else {
                assert!(commit);
                assert_eq!(selected.anchor.generation, 2);
                verify(&reopened, &selected.anchor, NEW);
                assert!(reopened
                    .read_at(f.live, OLD.len())
                    .unwrap()
                    .iter()
                    .all(|b| *b == 0));
            }
        }
    }
}
#[test]
fn unreserved_writes_overlapping_leases_and_stale_handles_cannot_modify_committed_state() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let f = fixture(mode(false, true), &owner);
    let authority = authority(f.anchor.mode, &public);
    let mut prepared = PreparedStore::begin(
        f.storage.clone(),
        f.anchor.archive,
        f.anchor.mode,
        &authority,
        None,
    )
    .unwrap();
    assert!(prepared.write_at(f.live, b"bad").is_err());
    assert!(prepared.write_at(0, b"bad").is_err());
    assert!(prepared.truncate(0).is_err());
    let lease = Reservation {
        namespace: FREE,
        base: f.free,
        start: f.free,
        len: 4096,
    };
    prepared.reserve(&[lease]).unwrap();
    let mut stale = prepared.clone();
    prepared
        .write_at(f.free, b"prepared private bytes")
        .unwrap();
    prepared.abort(&authority).unwrap();
    assert!(stale.write_at(f.free, b"stale").is_err());
    assert!(stale.append(b"stale").is_err());
    verify(&f.storage, &f.anchor, OLD);
    let mut prepared = PreparedStore::begin(
        f.storage.clone(),
        f.anchor.archive,
        f.anchor.mode,
        &authority,
        None,
    )
    .unwrap();
    prepared.reserve(&[lease]).unwrap();
    assert!(prepared.reserve(&[lease]).is_err());
    assert!(prepared.write_at(f.free, b"after failure").is_err());
    drop(prepared);
    let mut reopened = f.storage.clone();
    recover(
        &mut reopened,
        f.anchor.archive,
        f.anchor.mode,
        &authority,
        None,
    )
    .unwrap();
    verify(&reopened, &f.anchor, OLD);
}
#[test]
fn forged_journal_does_not_authorize_erasing_live_or_control_ranges() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let f = fixture(mode(false, true), &owner);
    let authority = authority(f.anchor.mode, &public);
    let context = Context::new(f.anchor.archive, f.anchor.mode, None).unwrap();
    let before = f.storage.read_all().unwrap();
    for bad in [
        Reservation {
            namespace: FREE,
            base: f.live,
            start: f.live,
            len: OLD.len() as u64,
        },
        Reservation {
            namespace: FREE,
            base: f.free,
            start: 0,
            len: 4096,
        },
        Reservation {
            namespace: FREE,
            base: f.free,
            start: f.anchor.index.primary,
            len: 16,
        },
        Reservation {
            namespace: FREE,
            base: f.free,
            start: u64::MAX,
            len: 8,
        },
    ] {
        let mut storage = Observed::new(before.clone());
        let old = context.select(&storage).unwrap();
        // The journal is not erasure authority, even with a valid checksum.
        context
            .transition(
                &mut storage,
                &old,
                f.anchor.commitment().unwrap(),
                true,
                vec![bad],
            )
            .unwrap();
        assert!(recover(
            &mut storage,
            f.anchor.archive,
            f.anchor.mode,
            &authority,
            None
        )
        .is_err());
        verify(&storage, &f.anchor, OLD);
        assert_eq!(storage.read_at(f.live, OLD.len()).unwrap(), OLD);
    }
}

#[test]
fn large_interrupted_rollback_resumes_after_the_durable_cleanup_checkpoint() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let f = fixture_size(mode(false, false), &owner, 12 * 1024 * 1024);
    let authority = authority(f.anchor.mode, &public);
    let mut prepared = PreparedStore::begin(
        f.storage.clone(),
        f.anchor.archive,
        f.anchor.mode,
        &authority,
        None,
    )
    .unwrap();
    prepared
        .reserve(&[Reservation {
            namespace: FREE,
            base: f.free,
            start: f.free,
            len: 12 * 1024 * 1024,
        }])
        .unwrap();
    prepared
        .write_at(f.free, &vec![0x5c; 12 * 1024 * 1024])
        .unwrap();
    // Publication sync + 128 zero writes + zero sync + five progress operations.
    f.storage.fail(135);
    assert!(prepared.abort(&authority).is_err());
    drop(prepared);
    let context = Context::new(f.anchor.archive, f.anchor.mode, None).unwrap();
    let checkpoint = context.select(&f.storage).unwrap();
    assert_eq!(checkpoint.record.cleanup_bytes, CLEANUP_CHECKPOINT);
    let mut reopened = Observed::new(f.storage.read_all().unwrap());
    reopened.reset_stats();
    recover(
        &mut reopened,
        f.anchor.archive,
        f.anchor.mode,
        &authority,
        None,
    )
    .unwrap();
    assert_eq!(
        reopened.stats().write_bytes,
        4 * 1024 * 1024 + 2 * SLOT_BYTES as u64
    );
    assert!(reopened
        .read_at(f.free, 12 * 1024 * 1024)
        .unwrap()
        .iter()
        .all(|b| *b == 0));
    verify(&reopened, &f.anchor, OLD);
}

// A power-loss model, distinct from process death: successful writes remain
// volatile until sync. A failed operation may leave a durable write prefix or
// may have persisted all prior writes before reporting a failed sync.
#[derive(Clone, Debug)]
pub(crate) struct CrashStore(Arc<Mutex<CrashState>>);
#[derive(Debug)]
struct CrashState {
    volatile: Vec<u8>,
    durable: Vec<u8>,
    operations: usize,
    fail_at: Option<usize>,
    prefix: usize,
    persist_sync: bool,
}
impl CrashStore {
    pub(crate) fn new(
        bytes: Vec<u8>,
        fail_at: Option<usize>,
        prefix: usize,
        persist_sync: bool,
    ) -> Self {
        Self(Arc::new(Mutex::new(CrashState {
            volatile: bytes.clone(),
            durable: bytes,
            operations: 0,
            fail_at,
            prefix,
            persist_sync,
        })))
    }
    pub(crate) fn durable(&self) -> Vec<u8> {
        self.0.lock().unwrap().durable.clone()
    }
    pub(crate) fn operations(&self) -> usize {
        self.0.lock().unwrap().operations
    }
}
impl CrashState {
    fn failure(&mut self) -> bool {
        let fail = self.fail_at == Some(self.operations);
        self.operations += 1;
        fail
    }
    fn write(&mut self, offset: usize, bytes: &[u8]) -> Result<()> {
        let fail = self.failure();
        let len = if fail {
            bytes.len().min(self.prefix)
        } else {
            bytes.len()
        };
        if len != 0 {
            self.volatile
                .resize(self.volatile.len().max(offset + len), 0);
            self.volatile[offset..offset + len].copy_from_slice(&bytes[..len]);
            if fail {
                self.durable.resize(self.durable.len().max(offset + len), 0);
                self.durable[offset..offset + len].copy_from_slice(&bytes[..len]);
            }
        }
        if fail {
            Err(Error::Io("simulated power loss".into()))
        } else {
            Ok(())
        }
    }
}
impl Storage for CrashStore {
    fn len(&self) -> Result<u64> {
        Ok(self.0.lock().unwrap().volatile.len() as u64)
    }
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        let state = self.0.lock().unwrap();
        let start = usize::try_from(offset).map_err(|_| Error::Truncated)?;
        let end = start.checked_add(len).ok_or(Error::Truncated)?;
        state
            .volatile
            .get(start..end)
            .map(|b| b.to_vec())
            .ok_or(Error::Truncated)
    }
    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        out.copy_from_slice(&self.read_at(offset, out.len())?);
        Ok(())
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        let mut state = self.0.lock().unwrap();
        let offset = state.volatile.len();
        state.write(offset, bytes)?;
        Ok(offset as u64)
    }
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        let offset = usize::try_from(offset).map_err(|_| Error::Truncated)?;
        let mut state = self.0.lock().unwrap();
        if offset
            .checked_add(bytes.len())
            .is_none_or(|end| end > state.volatile.len())
        {
            return Err(Error::Truncated);
        }
        state.write(offset, bytes)
    }
    fn truncate(&mut self, len: u64) -> Result<()> {
        let mut state = self.0.lock().unwrap();
        let fail = state.failure();
        if !fail || state.persist_sync {
            state.volatile.truncate(len as usize);
            if fail {
                state.durable.truncate(len as usize);
            }
        }
        if fail {
            Err(Error::Io("simulated power loss".into()))
        } else {
            Ok(())
        }
    }
    fn sync(&self) -> Result<()> {
        let mut state = self.0.lock().unwrap();
        let fail = state.failure();
        if !fail || state.persist_sync {
            state.durable = state.volatile.clone();
        }
        if fail {
            Err(Error::Io("simulated power loss".into()))
        } else {
            Ok(())
        }
    }
}
fn check_recovered(
    f: &Fixture,
    storage: &mut impl Storage,
    public: &OwnerSigningPublicKey,
    may_commit: bool,
) -> u64 {
    let authority = authority(f.anchor.mode, public);
    recover(
        storage,
        f.anchor.archive,
        f.anchor.mode,
        &authority,
        key(f.anchor.mode),
    )
    .unwrap();
    let selected =
        publication::select(storage, f.anchor.archive, f.anchor.mode, &authority).unwrap();
    assert_eq!(storage.len().unwrap(), selected.anchor.sealed_len);
    match selected.anchor.generation {
        1 => {
            verify(storage, &selected.anchor, OLD);
            assert!(storage
                .read_at(f.free, 8192)
                .unwrap()
                .iter()
                .all(|b| *b == 0));
        }
        2 => {
            assert!(may_commit);
            verify(storage, &selected.anchor, NEW);
            assert!(storage
                .read_at(f.live, OLD.len())
                .unwrap()
                .iter()
                .all(|b| *b == 0));
            for root in &f.metadata {
                for offset in [root.primary, root.mirror] {
                    assert!(storage
                        .read_at(offset, root.len as usize)
                        .unwrap()
                        .iter()
                        .all(|b| *b == 0));
                }
            }
        }
        other => panic!("unexpected generation {other}"),
    }
    selected.anchor.generation
}
#[test]
fn power_loss_at_each_mutation_recovers_after_partial_writes_and_uncertain_syncs() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let f = fixture(mode(false, false), &owner);
    let before = f.storage.read_all().unwrap();
    let public = owner.public_key();
    let authority = authority(f.anchor.mode, &public);
    let prefixes = [
        0,
        1,
        32,
        48,
        4096,
        SLOT_BYTES - 33,
        SLOT_BYTES - 1,
        SLOT_BYTES,
    ];
    let mut cases = 0;
    let mut pending_old = None;
    let mut pending_new = None;
    for commit in [false, true] {
        let probe = CrashStore::new(before.clone(), None, 0, false);
        perform(&f, probe.clone(), &owner, commit).unwrap();
        for fail in 0..probe.operations() {
            for prefix in prefixes {
                for persist in [false, true] {
                    let crashed = CrashStore::new(before.clone(), Some(fail), prefix, persist);
                    assert!(
                        perform(&f, crashed.clone(), &owner, commit).is_err(),
                        "commit={commit}, operation={fail}"
                    );
                    let durable = crashed.durable();
                    let mut reopened = Observed::new(durable.clone());
                    let context = Context::new(f.anchor.archive, f.anchor.mode, None).unwrap();
                    let journal = context.select(&reopened).unwrap();
                    let selected =
                        publication::select(&reopened, f.anchor.archive, f.anchor.mode, &authority)
                            .unwrap();
                    if journal.record.active {
                        if selected.anchor.generation == 1
                            && !journal.record.reservations.is_empty()
                            && reopened.read_at(f.free, NEW.len()).unwrap() == NEW
                        {
                            pending_old = Some(durable.clone());
                        }
                        if selected.anchor.generation == 2
                            && reopened.read_at(f.live, OLD.len()).unwrap() == OLD
                        {
                            pending_new = Some(durable.clone());
                        }
                    }
                    // Preserve the case context in any assertion failure.
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        check_recovered(&f, &mut reopened, &public, commit)
                    }));
                    assert!(
                        outcome.is_ok(),
                        "commit={commit}, operation={fail}, prefix={prefix}, persist={persist}"
                    );
                    cases += 1;
                }
            }
        }
    }
    // Crash recovery itself at every operation, for both rollback and a won
    // publication whose retired bytes still need cleaning. Losing a cleanup
    // write or its completion marker must remain safely repeatable.
    for bytes in [
        pending_old.expect("durable abandoned reuse"),
        pending_new.expect("durable pending retirement"),
    ] {
        let mut probe = CrashStore::new(bytes.clone(), None, 0, false);
        recover(
            &mut probe,
            f.anchor.archive,
            f.anchor.mode,
            &authority,
            None,
        )
        .unwrap();
        for fail in 0..probe.operations() {
            for prefix in prefixes {
                for persist in [false, true] {
                    let mut crashed = CrashStore::new(bytes.clone(), Some(fail), prefix, persist);
                    assert!(recover(
                        &mut crashed,
                        f.anchor.archive,
                        f.anchor.mode,
                        &authority,
                        None
                    )
                    .is_err());
                    let mut reopened = Observed::new(crashed.durable());
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        check_recovered(&f, &mut reopened, &public, true)
                    }));
                    assert!(
                        outcome.is_ok(),
                        "recovery operation={fail}, prefix={prefix}, persist={persist}"
                    );
                    cases += 1;
                }
            }
        }
    }
    eprintln!("preparation power-loss cases: {cases}");
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "manual fresh-process journal CPU/RSS and I/O experiment"]
fn preparation_journal_resource_probe() {
    fn usage() -> (i64, i64) {
        let mut value = std::mem::MaybeUninit::<libc::rusage>::uninit();
        // SAFETY: getrusage initializes the output on success, checked below.
        let status = unsafe { libc::getrusage(libc::RUSAGE_SELF, value.as_mut_ptr()) };
        assert_eq!(status, 0);
        // SAFETY: the successful getrusage call above initialized every field.
        let value = unsafe { value.assume_init() };
        (
            value.ru_utime.tv_sec * 1_000_000
                + value.ru_utime.tv_usec
                + value.ru_stime.tv_sec * 1_000_000
                + value.ru_stime.tv_usec,
            value.ru_maxrss,
        )
    }
    fn counts(c: Counts) -> serde_json::Value {
        serde_json::json!({"reads":c.reads,"read_bytes":c.read_bytes,"writes":c.writes,"write_bytes":c.write_bytes,"syncs":c.syncs,"truncates":c.truncates})
    }
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let count: u64 = std::env::var("REVAULT_JOURNAL_FREE_RANGES")
        .unwrap()
        .parse()
        .unwrap();
    assert!([1, 1000, 100000].contains(&count));
    let encrypted = std::env::var("REVAULT_JOURNAL_ENCRYPTED").unwrap() == "1";
    let signed = std::env::var("REVAULT_JOURNAL_SIGNED").unwrap() == "1";
    let mode = mode(encrypted, signed);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let path = std::env::temp_dir().join(format!(
        "revault-preparation-probe-{}-{}.lbx",
        std::process::id(),
        count
    ));
    assert!(!path.exists());
    let cleanup = Cleanup(path);
    let mut storage = Observed::backend(
        StorageBackend::create_file(&cleanup.0, &vec![0; DATA_START as usize]).unwrap(),
    );
    // Sparse synthetic free-space inventory, not a production archive. Each
    // reusable 4 KiB interval has a protected 4 KiB neighbour. Setup and index
    // construction are excluded from the timed failed-operation measurement.
    storage.truncate(DATA_START + count * 8192).unwrap();
    let live = storage.append(OLD).unwrap();
    let archive = LockboxId::from_bytes([82; 16]);
    let index = Index::new(archive, mode, key(mode)).unwrap();
    let logical = index
        .build_sorted(
            &mut storage,
            [Entry::new(1, b"/keep", &descriptor(live, OLD))],
        )
        .unwrap();
    let allocation = index
        .build_sorted(
            &mut storage,
            (0..count).map(|i| Ok(range(FREE, DATA_START + i * 8192, 4096))),
        )
        .unwrap();
    let anchor = Anchor {
        archive,
        generation: 1,
        mode,
        sealed_len: storage.len().unwrap(),
        object_root: logical.root.digest,
        previous: [0; 32],
        index: logical.root,
        allocation: allocation.root,
        keys: RootRef::default(),
    };
    publication::publish(
        &mut storage,
        &anchor,
        &authority,
        signed.then_some(&owner),
        None,
    )
    .unwrap();
    initialize(&mut storage, archive, mode, &authority, key(mode)).unwrap();
    let free = DATA_START + (count / 2) * 8192;
    storage.reset_stats();
    let start_cpu = usage().0;
    let start = std::time::Instant::now();
    let mut prepared =
        PreparedStore::begin(storage.clone(), archive, mode, &authority, key(mode)).unwrap();
    let begin = storage.stats();
    storage.reset_stats();
    prepared
        .reserve(&[Reservation {
            namespace: FREE,
            base: free,
            start: free,
            len: 4096,
        }])
        .unwrap();
    let reserve = storage.stats();
    storage.reset_stats();
    prepared.write_at(free, &[0x71; 4096]).unwrap();
    let payload = storage.stats();
    storage.reset_stats();
    prepared.abort(&authority).unwrap();
    let abort = storage.stats();
    let wall_us = start.elapsed().as_micros();
    let (cpu, rss) = usage();
    // Verify through a separately opened file handle after the timed operation.
    drop(prepared);
    drop(storage);
    let mut reopened = StorageBackend::file_for_write(&cleanup.0).unwrap();
    recover(&mut reopened, archive, mode, &authority, key(mode)).unwrap();
    let selected = publication::select(&reopened, archive, mode, &authority).unwrap();
    assert_eq!(selected.anchor.generation, 1);
    assert_eq!(reopened.len().unwrap(), anchor.sealed_len);
    verify(&reopened, &selected.anchor, OLD);
    assert!(reopened
        .read_at(free, 8192)
        .unwrap()
        .iter()
        .all(|b| *b == 0));
    println!(
        "JOURNAL_RESOURCE {}",
        serde_json::json!({"free_ranges":count,"encrypted":encrypted,"signed":signed,"wall_us":wall_us,"cpu_us":cpu-start_cpu,"rss_kib":rss,"file_bytes":anchor.sealed_len,"retained_extra_bytes":reopened.len().unwrap()-anchor.sealed_len,"fixed_journal_bytes":2*SLOT_BYTES,"begin":counts(begin),"reserve":counts(reserve),"payload":counts(payload),"abort":counts(abort)})
    );
}

#[test]
fn repeated_abandoned_operations_keep_fixed_control_size_and_protected_neighbours() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let f = fixture(mode(false, false), &owner);
    let authority = authority(f.anchor.mode, &public);
    let mut storage = f.storage.clone();
    // This is journal aging only, not the full archive lifecycle/space audit.
    for iteration in 0..1000 {
        let mut prepared = PreparedStore::begin(
            storage.clone(),
            f.anchor.archive,
            f.anchor.mode,
            &authority,
            None,
        )
        .unwrap();
        prepared
            .reserve(&[Reservation {
                namespace: FREE,
                base: f.free,
                start: f.free + 1024,
                len: 4096,
            }])
            .unwrap();
        prepared.write_at(f.free + 1024, &[0x93; 4096]).unwrap();
        prepared.append(&[0x27; 33]).unwrap();
        if iteration % 2 == 0 {
            prepared.abort(&authority).unwrap();
        } else {
            prepared.sync().unwrap();
            drop(prepared);
            recover(
                &mut storage,
                f.anchor.archive,
                f.anchor.mode,
                &authority,
                None,
            )
            .unwrap();
        }
        assert_eq!(storage.len().unwrap(), f.anchor.sealed_len);
        assert!(storage
            .read_at(f.free, 8192)
            .unwrap()
            .iter()
            .all(|b| *b == 0));
    }
    verify(&storage, &f.anchor, OLD);
}

#[test]
fn independent_journal_vector_and_noncanonical_records_are_checked() {
    fn hex(value: &str) -> Vec<u8> {
        value
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }
    // Independently constructed with Python struct + hashlib SHA-256, without
    // calling this codec. Fixed header, 142-byte body prefix, 65314 zero bytes.
    let mut vector=hex("5256345052453032020007015252525252525252525252525252525200000100000000000000000000000000b0ff0000");
    vector.extend(hex("070000000000000001010101010101010101010101010101010101010101010101010101010101010202020202020202020202020202020202020202020202020202020202020202010000000000000000000000000000000000000000000000000000000000000000000000000000000001000000f0000004000000000000040400000000000010000000000000"));
    vector.extend(vec![0; 65314]);
    vector.extend(hex(
        "f132005608a64f82abcd39df58187374b099e4f674f91a3a9ba8402a4c9c5cdb",
    ));
    assert_eq!(vector.len(), SLOT_BYTES);
    assert_eq!(
        strong_checksum(&vector).as_slice(),
        hex("7016099545bc7b7c19adea9651e12545debf256e95b21314ec642cf141771152")
    );
    let context = Context::new(LockboxId::from_bytes([82; 16]), mode(false, false), None).unwrap();
    let record = context.decode(&vector).unwrap();
    assert_eq!(record.sequence, 7);
    assert_eq!(
        record.reservations,
        vec![Reservation {
            namespace: FREE,
            base: DATA_START,
            start: DATA_START + 1024,
            len: 4096
        }]
    );
    assert_eq!(context.encode(&record).unwrap(), vector);
    for (offset, value) in [
        (8, 1),
        (10, 0),
        (12, 0),
        (28, 1),
        (32, 1),
        (44, 0),
        (48, 0),
        (120, 2),
        (121, 1),
        (163, 1),
        (CHECKSUM - 1, 1),
    ] {
        let mut bad = vector.clone();
        bad[offset] = value;
        let checksum = strong_checksum(&bad[..CHECKSUM]);
        bad[CHECKSUM..].copy_from_slice(&checksum);
        assert!(context.decode(&bad).is_err(), "offset {offset}");
    }
    for len in [0, HEADER, HEADER + 141, CHECKSUM, SLOT_BYTES - 1] {
        assert!(context.decode(&vector[..len]).is_err());
    }
    let mut too_many = record.clone();
    too_many
        .reservations
        .resize(MAX_RESERVATIONS + 1, record.reservations[0]);
    assert!(context.encode(&too_many).is_err());
    let encrypted = Context::new(context.archive, mode(true, false), Some(KEY)).unwrap();
    let ciphertext = encrypted.encode(&record).unwrap();
    let wrong_key = Context::new(context.archive, mode(true, false), Some(&[23; 32])).unwrap();
    assert!(wrong_key.decode(&ciphertext).is_err());
    assert!(context.decode(&ciphertext).is_err());
}

#[derive(Debug, Clone)]
struct ProcessStore {
    inner: StorageBackend,
    calls: Arc<std::sync::atomic::AtomicUsize>,
    stop: usize,
}
impl ProcessStore {
    fn boundary(&self) {
        if self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == self.stop {
            use std::io::Write;
            println!("\nREVAULT_PREPARATION_READY");
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
    fn read_at(&self, o: u64, n: usize) -> Result<Vec<u8>> {
        self.inner.read_at(o, n)
    }
    fn read_at_into(&self, o: u64, b: &mut [u8]) -> Result<()> {
        self.inner.read_at_into(o, b)
    }
    fn append(&mut self, b: &[u8]) -> Result<u64> {
        let o = self.inner.append(b)?;
        self.boundary();
        Ok(o)
    }
    fn write_at(&mut self, o: u64, b: &[u8]) -> Result<()> {
        self.inner.write_at(o, b)?;
        self.boundary();
        Ok(())
    }
    fn truncate(&mut self, n: u64) -> Result<()> {
        self.inner.truncate(n)?;
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
fn preparation_process_child() {
    let Some(path) = std::env::var_os("REVAULT_PREPARATION_CHILD_PATH") else {
        return;
    };
    let path = std::path::PathBuf::from(path);
    let stop = std::env::var("REVAULT_PREPARATION_CHILD_STOP")
        .unwrap()
        .parse()
        .unwrap();
    let commit = std::env::var("REVAULT_PREPARATION_CHILD_COMMIT").unwrap() == "1";
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let f = fixture(mode(true, true), &owner);
    // Only the synthetic public verification key crosses the process boundary.
    std::fs::write(path.with_extension("public"), owner.public_key().to_bytes()).unwrap();
    let initial = f.storage.read_all().unwrap();
    std::fs::write(path.with_extension("base"), &initial).unwrap();
    let inner = StorageBackend::create_file(&path, &initial).unwrap();
    inner.sync().unwrap();
    let storage = ProcessStore {
        inner,
        calls: Default::default(),
        stop,
    };
    perform(&f, storage, &owner, commit).unwrap();
    panic!("did not reach preparation stop {stop}");
}
#[test]
fn preparation_abort_and_commit_survive_process_death_at_every_mutation() {
    use std::io::BufRead;
    use std::process::{Command, Stdio};
    struct Directory(std::path::PathBuf);
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let directory = Directory(std::env::temp_dir().join(format!(
            "revault-preparation-death-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    std::fs::create_dir(&directory.0).unwrap();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let mode = mode(true, true);
    let mut cases = 0;
    for commit in [false, true] {
        let f = fixture(mode, &owner);
        let probe = Observed::new(f.storage.read_all().unwrap());
        perform(&f, probe.clone(), &owner, commit).unwrap();
        for stop in 0..probe.count() {
            let path = directory.0.join(format!("{commit}-{stop}.candidate"));
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "file_format::preparation_journal::tests::preparation_process_child",
                    "--nocapture",
                ])
                .env("REVAULT_PREPARATION_CHILD_PATH", &path)
                .env("REVAULT_PREPARATION_CHILD_STOP", stop.to_string())
                .env(
                    "REVAULT_PREPARATION_CHILD_COMMIT",
                    if commit { "1" } else { "0" },
                )
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap();
            let stdout = child.stdout.take().unwrap();
            let (send, receive) = std::sync::mpsc::channel();
            let reader = std::thread::spawn(move || {
                let found = std::io::BufReader::new(stdout)
                    .lines()
                    .any(|line| line.is_ok_and(|line| line == "REVAULT_PREPARATION_READY"));
                let _ = send.send(found);
            });
            let ready = receive.recv_timeout(std::time::Duration::from_secs(30));
            let _ = child.kill();
            let status = child.wait().unwrap();
            reader.join().unwrap();
            assert!(
                matches!(ready, Ok(true)),
                "did not reach commit={commit}, stop={stop}: {status}"
            );
            assert!(!status.success());
            let public = OwnerSigningPublicKey::from_bytes(
                &std::fs::read(path.with_extension("public")).unwrap(),
            )
            .unwrap();
            let base = Observed::new(std::fs::read(path.with_extension("base")).unwrap());
            let anchor =
                publication::select(&base, f.anchor.archive, mode, &Authority::Owner(&public))
                    .unwrap()
                    .anchor;
            let metadata = vec![anchor.index, anchor.allocation];
            let child_fixture = Fixture {
                storage: base,
                anchor,
                free: DATA_START,
                live: DATA_START + 8192,
                metadata,
            };
            let mut reopened = StorageBackend::file_for_write(&path).unwrap();
            check_recovered(&child_fixture, &mut reopened, &public, commit);
            cases += 1;
        }
    }
    eprintln!("preparation process-death cases: {cases}");
}

#[test]
fn power_loss_around_cleanup_checkpoint_never_skips_undurable_zeros() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let f = fixture_size(mode(false, false), &owner, 12 * 1024 * 1024);
    let authority = authority(f.anchor.mode, &public);
    let mut prepared = PreparedStore::begin(
        f.storage.clone(),
        f.anchor.archive,
        f.anchor.mode,
        &authority,
        None,
    )
    .unwrap();
    prepared
        .reserve(&[Reservation {
            namespace: FREE,
            base: f.free,
            start: f.free,
            len: 12 * 1024 * 1024,
        }])
        .unwrap();
    prepared
        .write_at(f.free, &vec![0x38; 12 * 1024 * 1024])
        .unwrap();
    prepared.sync().unwrap();
    drop(prepared);
    let initial = f.storage.read_all().unwrap();
    let context = Context::new(f.anchor.archive, f.anchor.mode, None).unwrap();
    // The last zero write, its sync, both progress copies and their syncs, then
    // the first write after progress. A failed sync may or may not have persisted.
    for fail in 128..=135 {
        for prefix in [0, SLOT_BYTES - 1, SLOT_BYTES] {
            for persist in [false, true] {
                let mut crashed = CrashStore::new(initial.clone(), Some(fail), prefix, persist);
                assert!(recover(
                    &mut crashed,
                    f.anchor.archive,
                    f.anchor.mode,
                    &authority,
                    None
                )
                .is_err());
                let mut reopened = Observed::new(crashed.durable());
                let selected = context.select(&reopened).unwrap();
                let progress = selected.record.cleanup_bytes;
                assert!(progress == 0 || progress == CLEANUP_CHECKPOINT);
                recover(
                    &mut reopened,
                    f.anchor.archive,
                    f.anchor.mode,
                    &authority,
                    None,
                )
                .unwrap();
                let checkpoint_writes = if progress == 0 {
                    2 * SLOT_BYTES as u64
                } else {
                    0
                };
                let mirror_repair = if selected.copies == 3 {
                    0
                } else {
                    SLOT_BYTES as u64
                };
                assert_eq!(
                    reopened.stats().write_bytes,
                    12 * 1024 * 1024 - progress
                        + 2 * SLOT_BYTES as u64
                        + checkpoint_writes
                        + mirror_repair,
                    "fail={fail}, prefix={prefix}, persist={persist}"
                );
                assert!(reopened
                    .read_at(f.free, 12 * 1024 * 1024)
                    .unwrap()
                    .iter()
                    .all(|b| *b == 0));
                verify(&reopened, &f.anchor, OLD);
            }
        }
    }
}

#[test]
fn adjoining_reservations_coalesce_without_rechecking_written_bytes_or_admitting_overlap() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let f = fixture(mode(false, false), &owner);
    let authority = Authority::Checksum;
    let mut prepared = PreparedStore::begin(
        f.storage.clone(),
        f.anchor.archive,
        f.anchor.mode,
        &authority,
        None,
    )
    .unwrap();
    for offset in 0..4096 {
        prepared
            .reserve(&[Reservation {
                namespace: FREE,
                base: f.free,
                start: f.free + offset,
                len: 1,
            }])
            .unwrap();
        prepared.write_at(f.free + offset, &[1]).unwrap();
    }
    let context = Context::new(f.anchor.archive, f.anchor.mode, None).unwrap();
    let selected = context.select(&f.storage).unwrap();
    assert_eq!(
        selected.record.reservations,
        vec![Reservation {
            namespace: FREE,
            base: f.free,
            start: f.free,
            len: 4096
        }]
    );
    assert!(prepared
        .reserve(&[Reservation {
            namespace: FREE,
            base: f.free,
            start: f.free + 4095,
            len: 2
        }])
        .is_err());
    drop(prepared);
    let mut reopened = f.storage.clone();
    recover(
        &mut reopened,
        f.anchor.archive,
        f.anchor.mode,
        &authority,
        None,
    )
    .unwrap();
    assert!(reopened
        .read_at(f.free, 8192)
        .unwrap()
        .iter()
        .all(|b| *b == 0));
    verify(&reopened, &f.anchor, OLD);
}
