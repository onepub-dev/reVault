//! Internal fixtures are necessary because no public CLI writes this profile.
//! Unlike compact session fixtures, these use the selected authenticated tree
//! and real allocation records. Stored payload and records are reopened afresh.
use super::*;
use crate::file_format::preparation_journal::compact::session::{InlineSession, OverflowSession};
use crate::file_format::preparation_journal::tests::CrashStore;
use crate::file_format::preparation_journal::{Reservation, FREE, PENDING};

mod commit;
mod update;

const PAYLOAD: u64 = REGION_LEN as u64;
const FREE_START: u64 = PAYLOAD + FAILURE_REGION;

fn fixture(
    mode: FormatMode,
    owner: &OwnerSigningKeyPair,
    pending: bool,
) -> (Vec<u8>, Vec<Reservation>, Vec<RootRef>) {
    let mut storage = Aligned(StorageBackend::memory(vec![0; REGION_LEN]));
    let payload = vec![37; FAILURE_REGION as usize];
    storage.append(&payload).unwrap();
    storage
        .append(&vec![0; 2 * FAILURE_REGION as usize])
        .unwrap();
    let name = |kind: u8, start: u64| [vec![kind], start.to_be_bytes().to_vec()].concat();
    let mut rows = vec![
        Entry::new(
            0,
            &name(u8::from(pending), FREE_START),
            &(2 * FAILURE_REGION).to_le_bytes(),
        ),
        Entry::new(
            0,
            &name(2, PAYLOAD),
            &[
                FAILURE_REGION.to_le_bytes().to_vec(),
                strong_checksum(&payload).to_vec(),
            ]
            .concat(),
        ),
    ];
    rows.extend(entries(8));
    let change = Index::new(archive(), mode, key(mode))
        .unwrap()
        .build_sorted(&mut storage, rows)
        .unwrap();
    let public = owner.public_key();
    initialize(
        &mut storage,
        archive(),
        mode,
        &authority(mode, &public),
        mode.signed().then_some(owner),
        key(mode),
        &manifest(change.root),
        &[],
    )
    .unwrap();
    let reservations = (0..156)
        .map(|i| Reservation {
            namespace: if pending { PENDING } else { FREE },
            base: FREE_START,
            start: FREE_START + i * 8,
            len: 8,
        })
        .collect();
    (storage.0.read_all().unwrap(), reservations, change.created)
}

fn stage(
    storage: &mut impl Storage,
    mode: FormatMode,
    authority: &Authority<'_>,
    reservations: Vec<Reservation>,
) -> Result<()> {
    let tree = Tree::open(storage, archive(), mode, authority, key(mode))?;
    let mut session = OverflowSession::open(storage, archive(), mode, key(mode))?;
    session.begin(
        storage,
        commitment(&tree.anchor)?,
        tree.anchor.sealed_len,
        reservations,
    )?;
    // Model abandoned writes to graph-authorized reusable space.
    storage.write_at(FREE_START, &vec![91; 156 * 8])?;
    storage.sync()
}

fn check(bytes: Vec<u8>, mode: FormatMode, authority: &Authority<'_>, sealed: u64) {
    let storage = StorageBackend::memory(bytes);
    assert_eq!(storage.len().unwrap(), sealed);
    let tree = Tree::open(&storage, archive(), mode, authority, key(mode)).unwrap();
    let mut count = 0u32;
    tree.visit(&storage, |entry| {
        assert_eq!(entry.key.as_slice(), count.to_be_bytes());
        assert_eq!(entry.value.as_slice(), vec![(count % 251) as u8; 128]);
        count += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(count, 8);
    assert_eq!(
        storage.read_at(PAYLOAD, FAILURE_REGION as usize).unwrap(),
        vec![37; FAILURE_REGION as usize]
    );
    assert!(storage
        .read_at(FREE_START, 2 * FAILURE_REGION as usize)
        .unwrap()
        .iter()
        .all(|b| *b == 0));
}

#[derive(Clone, Debug)]
struct ReadOnly(StorageBackend);
impl Storage for ReadOnly {
    fn len(&self) -> Result<u64> {
        self.0.len()
    }
    fn read_at(&self, at: u64, len: usize) -> Result<Vec<u8>> {
        self.0.read_at(at, len)
    }
    fn read_at_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        self.0.read_at_into(at, out)
    }
    fn append(&mut self, _: &[u8]) -> Result<u64> {
        panic!("invalid recovery appended")
    }
    fn write_at(&mut self, _: u64, _: &[u8]) -> Result<()> {
        panic!("invalid recovery wrote")
    }
    fn truncate(&mut self, _: u64) -> Result<()> {
        panic!("invalid recovery truncated")
    }
    fn sync(&self) -> Result<()> {
        panic!("invalid recovery synchronized")
    }
}

#[test]
fn authenticated_overflow_abort_reopens_all_modes_and_each_copy_loss() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (seed, reservations, pages) = fixture(mode, &owner, bits & 1 != 0);
        let sealed = seed.len() as u64;
        let mut storage = StorageBackend::memory(seed);
        stage(&mut storage, mode, &authority, reservations).unwrap();
        assert!(InlineSession::open(&storage, archive(), mode, key(mode)).is_err());
        let arena = OverflowSession::open(&storage, archive(), mode, key(mode))
            .unwrap()
            .arena()
            .unwrap();
        let original = storage.read_all().unwrap();
        let losses = [0, FAILURE_REGION, arena.primary, arena.mirror]
            .into_iter()
            .chain(pages.iter().flat_map(|page| [page.primary, page.mirror]));
        for at in std::iter::once(None).chain(losses.map(Some)) {
            let mut damaged = StorageBackend::memory(original.clone());
            if let Some(at) = at {
                damaged
                    .write_at(at, &vec![0; FAILURE_REGION as usize])
                    .unwrap();
            }
            recover_abort(&mut damaged, archive(), mode, &authority, key(mode)).unwrap();
            recover_abort(&mut damaged, archive(), mode, &authority, key(mode)).unwrap();
            check(damaged.read_all().unwrap(), mode, &authority, sealed);
        }
    }
}

#[test]
fn authenticated_overflow_abort_refuses_invalid_authority_ownership_and_aliases_unchanged() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, true, true, false);
    let authority = authority(mode, &public);
    let (seed, reservations, _) = fixture(mode, &owner, false);
    let sealed = seed.len() as u64;
    let tree = Tree::open(
        &StorageBackend::memory(seed.clone()),
        archive(),
        mode,
        &authority,
        None,
    )
    .unwrap();
    let base = commitment(&tree.anchor).unwrap();
    for case in 0..9 {
        let mut storage = StorageBackend::memory(seed.clone());
        let mut values = reservations.clone();
        match case {
            0 => values[0].start = PAYLOAD,
            1 => values[1] = values[0],
            2 => values[0].base += 1,
            3 => values[0].namespace = PENDING,
            4 => values[0].len = u64::MAX,
            _ => (),
        }
        let journal_base = if case == 7 { [93; 32] } else { base };
        if case == 7 {
            let stub = crate::file_format::preparation_journal::compact::initial_stub(
                archive(),
                mode,
                None,
                journal_base,
            )
            .unwrap();
            for bank in [0, FAILURE_REGION] {
                storage.write_at(bank + 8192, &stub).unwrap();
            }
        }
        OverflowSession::open(&storage, archive(), mode, None)
            .unwrap()
            .begin(&mut storage, journal_base, sealed, values)
            .unwrap();
        if case == 5 {
            storage.write_at(FREE_START + 2048, &[1]).unwrap();
        }
        if case == 6 {
            storage.append(&[1]).unwrap();
        }
        if case == 8 {
            // Valid plaintext journal checksums do not grant authority. Substitute
            // a valid arena into sealed reusable space.
            let arena = storage.read_at(sealed, FAILURE_REGION as usize).unwrap();
            for at in [FREE_START, FREE_START + FAILURE_REGION] {
                storage.write_at(at, &arena).unwrap();
            }
            for bank in [0, FAILURE_REGION] {
                let mut stub = storage.read_at(bank + 8192, 4096).unwrap();
                stub[121..129].copy_from_slice(&FREE_START.to_le_bytes());
                stub[129..137].copy_from_slice(&(FREE_START + FAILURE_REGION).to_le_bytes());
                let digest = strong_checksum(&stub[..4064]);
                stub[4064..].copy_from_slice(&digest);
                storage.write_at(bank + 8192, &stub).unwrap();
            }
        }
        let before = storage.read_all().unwrap();
        let mut guarded = ReadOnly(storage);
        assert!(
            recover_abort(&mut guarded, archive(), mode, &authority, None).is_err(),
            "case {case}"
        );
        assert_eq!(guarded.0.read_all().unwrap(), before, "case {case}");
    }
    let mut storage = StorageBackend::memory(seed);
    stage(&mut storage, mode, &authority, reservations).unwrap();
    let before = storage.read_all().unwrap();
    let wrong = OwnerSigningKeyPair::generate().unwrap().public_key();
    let mut guarded = ReadOnly(storage);
    assert!(recover_abort(
        &mut guarded,
        archive(),
        mode,
        &Authority::Owner(&wrong),
        None
    )
    .is_err());
    assert_eq!(guarded.0.read_all().unwrap(), before);
}

#[test]
fn authenticated_overflow_abort_rejects_malformed_committed_graph_before_writes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, true, true, false);
    let authority = authority(mode, &public);
    let (storage, _, _) = seed(
        mode,
        &owner,
        vec![Entry::new(0, &[0; 9], &4096u64.to_le_bytes())],
    );
    let mut guarded = ReadOnly(storage.0);
    assert!(recover_abort(&mut guarded, archive(), mode, &authority, None).is_err());
}

#[test]
fn authenticated_overflow_abort_mirrors_selected_publication_before_cleanup() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, true, true, false);
    let authority = authority(mode, &public);
    let (seed, reservations, _) = fixture(mode, &owner, false);
    let sealed = seed.len() as u64;
    let mut storage = StorageBackend::memory(seed);
    let tree = Tree::open(&storage, archive(), mode, &authority, None).unwrap();
    let mut next = tree.anchor.clone();
    next.generation += 1;
    next.previous = commitment(&tree.anchor).unwrap();
    let encoded = encode_in(&next, &authority, Some(&owner), Layout::Shared).unwrap();
    // Leave an older valid publication in the other bank, simulating a selected
    // generation whose publication mirroring was interrupted.
    storage.write_at(0, &encoded).unwrap();
    let base = commitment(&next).unwrap();
    let idle =
        crate::file_format::preparation_journal::compact::initial_stub(archive(), mode, None, base)
            .unwrap();
    for bank in [0, FAILURE_REGION] {
        storage.write_at(bank + 8192, &idle).unwrap();
    }
    stage(&mut storage, mode, &authority, reservations).unwrap();
    let checkpoint = storage.read_all().unwrap();
    let mut observed = CrashStore::new(checkpoint.clone(), None, 0, false);
    recover_abort(&mut observed, archive(), mode, &authority, None).unwrap();
    for at in 0..observed.operations() {
        for persist in [false, true] {
            let mut failed = CrashStore::new(checkpoint.clone(), Some(at), 97, persist);
            let _ = recover_abort(&mut failed, archive(), mode, &authority, None);
            let durable = failed.durable();
            // The first reservation changes only after both new publication
            // records have reached stable storage.
            if durable[FREE_START as usize] != 91 {
                assert_eq!(&durable[..encoded.len()], encoded.as_slice());
                assert_eq!(
                    &durable[FAILURE_REGION as usize..FAILURE_REGION as usize + encoded.len()],
                    encoded.as_slice()
                );
            }
            let mut reopened = StorageBackend::memory(durable);
            recover_abort(&mut reopened, archive(), mode, &authority, None).unwrap();
            check(reopened.read_all().unwrap(), mode, &authority, sealed);
        }
    }
}

#[test]
fn authenticated_overflow_abort_survives_interrupted_staging_and_recovery_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (seed, reservations, _) = fixture(mode, &owner, bits & 1 != 0);
        let sealed = seed.len() as u64;
        let mut staged = StorageBackend::memory(seed.clone());
        stage(&mut staged, mode, &authority, reservations.clone()).unwrap();
        for (checkpoint, staging) in [(seed, true), (staged.read_all().unwrap(), false)] {
            let run = |storage: &mut CrashStore| -> Result<()> {
                if staging {
                    stage(storage, mode, &authority, reservations.clone())?;
                }
                recover_abort(storage, archive(), mode, &authority, key(mode))
            };
            let mut observed = CrashStore::new(checkpoint.clone(), None, 0, false);
            run(&mut observed).unwrap();
            for at in 0..observed.operations() {
                for prefix in [0, 97, usize::MAX] {
                    for persist in [false, true] {
                        let mut failed =
                            CrashStore::new(checkpoint.clone(), Some(at), prefix, persist);
                        let _ = run(&mut failed);
                        let mut reopened = StorageBackend::memory(failed.durable());
                        recover_abort(&mut reopened, archive(), mode, &authority, key(mode)).unwrap_or_else(|error| panic!("mode={bits} staging={staging} at={at} prefix={prefix} persist={persist}: {error}"));
                        recover_abort(&mut reopened, archive(), mode, &authority, key(mode))
                            .unwrap();
                        check(reopened.read_all().unwrap(), mode, &authority, sealed);
                        cases += 1;
                    }
                }
            }
        }
    }
    println!("AUTHENTICATED_OVERFLOW_ABORT_POWER_LOSS_CASES {cases}");
}
