//! Internal metadata-only fixtures; no public CLI writes this experimental tree.
use super::super::variables::Guarded;
use super::lifecycle::{collect, fixture};
use super::*;
use std::collections::BTreeMap;
use tree_image::forms::move_records;
fn path(s: &str) -> LockboxPath {
    LockboxPath::new(s).unwrap()
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Identity {
    path: LockboxPath,
    texts: Vec<Vec<u8>>,
}
type Identities = BTreeMap<[u8; 16], Identity>;
fn identities<S: Storage>(storage: S, mode: FormatMode, authority: &Authority<'_>) -> Identities {
    let opened = TreeImage::open(storage, archive(), mode, authority, key(mode)).unwrap();
    opened
        .image
        .catalogue
        .forms
        .records
        .into_iter()
        .map(|r| {
            let mut texts = vec![r.name.encode_metadata()];
            for f in r.fields {
                texts.push(f.label.encode_metadata());
                texts.push(f.value.encode_metadata());
            }
            (
                r.id,
                Identity {
                    path: r.path,
                    texts,
                },
            )
        })
        .collect()
}
#[derive(Clone, Debug)]
struct Stable<S> {
    inner: Guarded<S>,
    payloads: Vec<(u64, u64)>,
}
impl<S: Storage> Stable<S> {
    fn check(&self, at: u64, bytes: &[u8]) {
        assert!(
            !self
                .payloads
                .iter()
                .any(|(a, b)| at < *b && *a < at + bytes.len() as u64),
            "move writes live payload"
        );
        assert!(
            !bytes.starts_with(crate::file_format::page::PAGE_MAGIC),
            "move stages value page"
        );
    }
}
impl<S: Storage> Storage for Stable<S> {
    fn len(&self) -> Result<u64> {
        self.inner.len()
    }
    fn read_at(&self, at: u64, len: usize) -> Result<Vec<u8>> {
        self.inner.read_at(at, len)
    }
    fn read_at_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        self.inner.read_at_into(at, out)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        self.check(self.len()?, bytes);
        self.inner.append(bytes)
    }
    fn write_at(&mut self, at: u64, bytes: &[u8]) -> Result<()> {
        self.check(at, bytes);
        self.inner.write_at(at, bytes)
    }
    fn truncate(&mut self, len: u64) -> Result<()> {
        assert!(self.payloads.iter().all(|(_, end)| *end <= len));
        self.inner.truncate(len)
    }
    fn sync(&self) -> Result<()> {
        self.inner.sync()
    }
}
fn guard<S: Storage>(
    inner: S,
    original: &Guarded<StorageBackend>,
    mode: FormatMode,
    authority: &Authority<'_>,
) -> Stable<S> {
    let opened = TreeImage::open(original.clone(), archive(), mode, authority, key(mode)).unwrap();
    Stable {
        inner: Guarded::new(inner, original.spans.borrow().clone()),
        payloads: opened
            .tree
            .graph
            .payloads()
            .iter()
            .map(|e| (e.start, e.start + e.len))
            .collect(),
    }
}
fn renamed(mut records: Vec<FormRecord>, moves: &[(LockboxPath, LockboxPath)]) -> Vec<FormRecord> {
    for r in &mut records {
        if let Some((_, to)) = moves.iter().find(|(from, _)| *from == r.path) {
            r.path = to.clone();
        }
    }
    records.sort_by(|a, b| a.path.cmp(&b.path));
    records
}
fn check(
    storage: &Stable<StorageBackend>,
    mode: FormatMode,
    authority: &Authority<'_>,
    definitions: &[FormDefinition],
    records: &[FormRecord],
    original: &Identities,
    bytes: &[u8],
) {
    let (report, defs, got) = collect(storage, mode, authority).unwrap();
    assert_eq!(report, tree_image::forms::SalvageReport::default());
    assert_eq!(defs, definitions);
    assert_eq!(got, records);
    let actual = identities(storage.clone(), mode, authority);
    assert_eq!(&actual, original);
    let mut opened =
        TreeImage::open(storage.clone(), archive(), mode, authority, key(mode)).unwrap();
    assert_eq!(
        opened
            .get_variable(&VariableName::new("retained").unwrap())
            .unwrap()
            .as_deref(),
        Some("retained variable")
    );
    for (name, expected) in [
        (b"/docs/data".as_slice(), b"payload".as_slice()),
        (b"/docs/neighbor".as_slice(), b"neighbor".as_slice()),
    ] {
        let mut actual = Vec::new();
        opened
            .image
            .read_range(name, 0, expected.len() as u64, |b| {
                actual.extend_from_slice(b);
                Ok(())
            })
            .unwrap();
        assert_eq!(actual, expected);
    }
    let after = storage.inner.inner.read_all().unwrap();
    for (start, end) in &storage.payloads {
        assert_eq!(
            &after[*start as usize..*end as usize],
            &bytes[*start as usize..*end as usize]
        );
    }
}
#[test]
fn typed_form_moves_metadata_only_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let (original, definitions, mut records) = fixture(mode, &authority, &owner);
        let bytes = original.inner.read_all().unwrap();
        let mut identities = identities(original.clone(), mode, &authority);
        let mut storage = guard(
            StorageBackend::memory(bytes.clone()),
            &original,
            mode,
            &authority,
        );
        for moves in [Vec::new(), vec![(path("/forms/0"), path("/forms/0"))]] {
            let before = storage.inner.inner.read_all().unwrap();
            assert!(!move_records(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &moves
            )
            .unwrap());
            assert_eq!(storage.inner.inner.read_all().unwrap(), before);
        }
        for moves in [
            vec![(path("/missing"), path("/new"))],
            vec![
                (path("/forms/0"), path("/a")),
                (path("/forms/0"), path("/b")),
            ],
            vec![
                (path("/forms/0"), path("/same")),
                (path("/forms/1"), path("/same")),
            ],
            vec![(path("/forms/0"), path("/forms/2"))],
            vec![(path("/forms/0"), path("/"))],
            vec![(path("/forms/0"), path("/docs/data/missing/child"))],
            vec![
                (path("/forms/0"), path("/would/create/new")),
                (path("/missing"), path("/elsewhere")),
            ],
        ] {
            let before = storage.inner.inner.read_all().unwrap();
            assert!(move_records(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &moves
            )
            .is_err());
            assert_eq!(storage.inner.inner.read_all().unwrap(), before);
        }
        for moves in [
            vec![
                (path("/forms/0"), path("/forms/1")),
                (path("/forms/1"), path("/forms/0")),
            ],
            vec![
                (path("/forms/0"), path("/forms/1")),
                (path("/forms/1"), path("/new/deep/record")),
            ],
            vec![(path("/forms/2"), path("/docs/data"))],
            vec![(path("/docs/data"), path("/docs/data/child"))],
            vec![(path("/forms/1"), path("/new/deep/record/child"))],
        ] {
            assert!(move_records(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &moves
            )
            .unwrap());
            records = renamed(records, &moves);
            for state in identities.values_mut() {
                if let Some((_, to)) = moves.iter().find(|(from, _)| *from == state.path) {
                    state.path = to.clone();
                }
            }
            check(
                &storage,
                mode,
                &authority,
                &definitions,
                &records,
                &identities,
                &bytes,
            );
        }
        for bank in [0, FAILURE_REGION] {
            let mut damaged = guard(
                StorageBackend::memory(storage.inner.inner.read_all().unwrap()),
                &original,
                mode,
                &authority,
            );
            damaged
                .inner
                .inner
                .write_at(bank, &vec![0; FAILURE_REGION as usize])
                .unwrap();
            check(
                &damaged,
                mode,
                &authority,
                &definitions,
                &records,
                &identities,
                &bytes,
            );
        }
        let mut damaged = guard(
            StorageBackend::memory(storage.inner.inner.read_all().unwrap()),
            &original,
            mode,
            &authority,
        );
        let at = damaged.inner.spans.borrow()[0].0;
        let byte = damaged.inner.inner.read_at(at, 1).unwrap()[0] ^ 1;
        damaged.inner.inner.write_at(at, &[byte]).unwrap();
        let before = damaged.inner.inner.read_all().unwrap();
        assert!(move_records(
            &mut damaged,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &[]
        )
        .is_err());
        assert_eq!(damaged.inner.inner.read_all().unwrap(), before);
    }
}

fn directories<S: Storage>(
    storage: S,
    mode: FormatMode,
    authority: &Authority<'_>,
) -> Vec<LockboxPath> {
    TreeImage::open(storage, archive(), mode, authority, key(mode))
        .unwrap()
        .image
        .filesystem_metadata()
        .unwrap()
        .into_iter()
        .filter(|m| m.entry.kind == crate::LockboxEntryKind::Directory)
        .map(|m| m.entry.path)
        .collect()
}
#[test]
fn typed_form_moves_atomic_parent_and_payload_recovery_faults() {
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    let mut recovery_cases = 0;
    for bits in [0, 1, 2, 3, 12, 13, 14, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let (original, definitions, old) = fixture(mode, &authority, &owner);
        let bytes = original.inner.read_all().unwrap();
        let old_ids = identities(original.clone(), mode, &authority);
        let old_dirs = directories(original.clone(), mode, &authority);
        let moves = vec![
            (path("/forms/0"), path("/forms/1")),
            (path("/forms/1"), path("/forms/0")),
            (path("/forms/2"), path("/new/deep/record")),
        ];
        let new = renamed(old.clone(), &moves);
        let mut new_ids = old_ids.clone();
        for state in new_ids.values_mut() {
            if let Some((_, to)) = moves.iter().find(|(from, _)| *from == state.path) {
                state.path = to.clone();
            }
        }
        let run = |s: &mut Stable<CrashStore>| {
            move_records(s, archive(), mode, &authority, signer, key(mode), &moves)
        };
        let mut observed = guard(
            CrashStore::new(bytes.clone(), None, 0, false),
            &original,
            mode,
            &authority,
        );
        run(&mut observed).unwrap();
        let count = observed.inner.inner.operations();
        let new_dirs = directories(
            StorageBackend::memory(observed.inner.inner.durable()),
            mode,
            &authority,
        );
        assert!(new_dirs.contains(&path("/new")) && new_dirs.contains(&path("/new/deep")));
        assert_ne!(new_dirs, old_dirs);
        let verify = |s: &Stable<StorageBackend>| {
            let actual = identities(s.clone(), mode, &authority);
            let committed = actual == new_ids;
            assert!(committed || actual == old_ids);
            check(
                s,
                mode,
                &authority,
                &definitions,
                if committed { &new } else { &old },
                if committed { &new_ids } else { &old_ids },
                &bytes,
            );
            assert_eq!(
                directories(s.clone(), mode, &authority),
                if committed {
                    new_dirs.clone()
                } else {
                    old_dirs.clone()
                }
            );
            let recovered = s.inner.inner.read_all().unwrap();
            if !committed && recovered.len() > bytes.len() {
                assert!(recovered[bytes.len()..].iter().all(|b| *b == 0));
            }
            committed
        };
        for at in 0..count {
            for prefix in [0, 97, usize::MAX] {
                for persist in [false, true] {
                    let mut failed = guard(
                        CrashStore::new(bytes.clone(), Some(at), prefix, persist),
                        &original,
                        mode,
                        &authority,
                    );
                    let _ = run(&mut failed);
                    let damaged = failed.inner.inner.durable();
                    let mut recovered = guard(
                        StorageBackend::memory(damaged.clone()),
                        &original,
                        mode,
                        &authority,
                    );
                    tree_image::recover(&mut recovered, archive(), mode, &authority, key(mode))
                        .unwrap_or_else(|e| {
                            panic!("mode={bits} at={at} prefix={prefix} persist={persist}: {e}")
                        });
                    let selected = verify(&recovered);
                    tree_image::recover(&mut recovered, archive(), mode, &authority, key(mode))
                        .unwrap();
                    assert_eq!(verify(&recovered), selected);
                    cases += 1;
                    if prefix == 97 && persist && [0, count / 2, count - 1].contains(&at) {
                        let mut observed = guard(
                            CrashStore::new(damaged.clone(), None, 0, false),
                            &original,
                            mode,
                            &authority,
                        );
                        tree_image::recover(&mut observed, archive(), mode, &authority, key(mode))
                            .unwrap();
                        for cut in 0..observed.inner.inner.operations() {
                            for written in [0, 97, usize::MAX] {
                                let mut interrupted = guard(
                                    CrashStore::new(damaged.clone(), Some(cut), written, true),
                                    &original,
                                    mode,
                                    &authority,
                                );
                                let _ = tree_image::recover(
                                    &mut interrupted,
                                    archive(),
                                    mode,
                                    &authority,
                                    key(mode),
                                );
                                let mut resumed = guard(
                                    StorageBackend::memory(interrupted.inner.inner.durable()),
                                    &original,
                                    mode,
                                    &authority,
                                );
                                tree_image::recover(
                                    &mut resumed,
                                    archive(),
                                    mode,
                                    &authority,
                                    key(mode),
                                )
                                .unwrap();
                                assert_eq!(verify(&resumed), selected);
                                recovery_cases += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    println!("TYPED_FORM_MOVE_FAULTS cases={cases} interrupted_recovery={recovery_cases}");
}
