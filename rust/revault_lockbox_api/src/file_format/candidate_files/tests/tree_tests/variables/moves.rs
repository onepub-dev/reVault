//! Internal metadata-only transaction fixtures; no public CLI writes this tree.
use super::*;
use crate::file_format::secure_segments::Layout;
use std::collections::BTreeMap;
use tree_image::variables::{move_variables, salvage_variables};

fn name(value: &str) -> VariableName {
    VariableName::new(value).unwrap()
}
type Names = BTreeMap<VariableName, Layout>;
fn names<S: Storage>(storage: S, mode: FormatMode, authority: &Authority<'_>) -> Names {
    AuditedTreeImage::open(storage, archive(), mode, authority, key(mode))
        .unwrap()
        .image
        .catalogue
        .variables
        .into_iter()
        .map(|v| (v.name, v.layout))
        .collect()
}
fn seed(
    mode: FormatMode,
    authority: &Authority<'_>,
    owner: &OwnerSigningKeyPair,
) -> StorageBackend {
    let mut storage = super::super::mutation::seed(mode, authority, owner);
    let signer = mode.signed().then_some(owner);
    for (key_name, value) in [
        ("alpha", "normal"),
        ("retained", "stable"),
        ("a_b", "sibling"),
    ] {
        set_variable(
            &mut storage,
            archive(),
            mode,
            authority,
            signer,
            key(mode),
            &name(key_name),
            value,
        )
        .unwrap();
    }
    set_secret_variable(
        &mut storage,
        archive(),
        mode,
        authority,
        signer,
        key(mode),
        &name("beta"),
        &SecretString::try_from_slice("secret🔐".as_bytes()).unwrap(),
    )
    .unwrap();
    storage
}
fn renamed(mut expected: Names, changes: &[(VariableName, VariableName)]) -> Names {
    let values: Vec<_> = changes
        .iter()
        .map(|(from, to)| (to.clone(), expected.remove(from).unwrap()))
        .collect();
    expected.extend(values);
    expected
}
#[derive(Clone, Debug)]
struct Stable<S> {
    inner: S,
    variables: Vec<(u64, u64)>,
    payloads: Vec<(u64, u64)>,
}
impl<S: Storage> Stable<S> {
    fn check_write(&self, at: u64, bytes: &[u8]) {
        assert!(
            !self
                .payloads
                .iter()
                .any(|(a, b)| at < *b && *a < at + bytes.len() as u64),
            "move writes live payload"
        );
        assert!(
            !bytes.starts_with(crate::file_format::page::PAGE_MAGIC),
            "move stages a value page"
        );
    }
}
impl<S: Storage> Storage for Stable<S> {
    fn len(&self) -> Result<u64> {
        self.inner.len()
    }
    fn read_at(&self, at: u64, len: usize) -> Result<Vec<u8>> {
        assert!(
            !self
                .variables
                .iter()
                .any(|(a, b)| at < *b && *a < at + len as u64),
            "ordinary secret segment read"
        );
        self.inner.read_at(at, len)
    }
    fn read_at_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        self.inner.read_at_into(at, out)
    }
    fn write_at(&mut self, at: u64, bytes: &[u8]) -> Result<()> {
        self.check_write(at, bytes);
        self.inner.write_at(at, bytes)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        self.check_write(self.len()?, bytes);
        self.inner.append(bytes)
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
    original: &StorageBackend,
    mode: FormatMode,
    authority: &Authority<'_>,
) -> Stable<S> {
    let opened =
        AuditedTreeImage::open(original.clone(), archive(), mode, authority, key(mode)).unwrap();
    Stable {
        inner,
        variables: opened
            .image
            .catalogue
            .variables
            .iter()
            .flat_map(|v| v.layout.extents.iter())
            .map(|e| (e.start, e.start + e.len))
            .collect(),
        payloads: opened
            .tree
            .graph
            .payloads()
            .iter()
            .map(|e| (e.start, e.start + e.len))
            .collect(),
    }
}
fn check<S: Storage>(
    storage: S,
    mode: FormatMode,
    authority: &Authority<'_>,
    expected: &Names,
    original: &Names,
) {
    let mut opened =
        AuditedTreeImage::open(storage, archive(), mode, authority, key(mode)).unwrap();
    let actual: Names = opened
        .image
        .catalogue
        .variables
        .iter()
        .map(|v| (v.name.clone(), v.layout.clone()))
        .collect();
    assert_eq!(&actual, expected);
    for (name, layout) in expected {
        let original_name = original
            .iter()
            .find(|(_, old)| old.id == layout.id)
            .unwrap()
            .0;
        if layout.sensitivity == crate::VariableSensitivity::Secret {
            assert!(opened.get_variable(name).is_err());
            assert_eq!(
                opened
                    .with_secret_variable(name, |v| v
                        .with_str(|s| assert_eq!(s, "secret🔐"))
                        .unwrap())
                    .unwrap(),
                Some(())
            );
        } else {
            let value = match original_name.as_str() {
                "/alpha" => "normal",
                "/retained" => "stable",
                "/a_b" => "sibling",
                _ => panic!("unexpected fixture"),
            };
            assert_eq!(opened.get_variable(name).unwrap().as_deref(), Some(value));
        }
    }
    let mut file = Vec::new();
    opened
        .image
        .read_range(b"/docs/neighbor", 0, 8, |b| {
            file.extend_from_slice(b);
            Ok(())
        })
        .unwrap();
    assert_eq!(file, b"neighbor");
}

#[test]
fn typed_tree_variable_moves_lifecycle_and_refusals_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let original = seed(mode, &authority, &owner);
        let before = original.read_all().unwrap();
        let initial = names(original.clone(), mode, &authority);
        let mut storage = guard(original.clone(), &original, mode, &authority);
        for noop in [vec![], vec![(name("alpha"), name("alpha"))]] {
            assert!(!move_variables(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &noop
            )
            .unwrap());
            assert_eq!(storage.inner.read_all().unwrap(), before);
        }
        for invalid in [
            vec![(name("alpha"), name("x")), (name("alpha"), name("y"))],
            vec![(name("missing"), name("missing"))],
            vec![(name("alpha"), name("x")), (name("beta"), name("x"))],
            vec![(name("alpha"), name("retained"))],
            vec![(name("alpha"), name("/retained/child"))],
            vec![(name("alpha"), name("a")), (name("beta"), name("/a/b"))],
        ] {
            assert!(move_variables(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &invalid
            )
            .is_err());
            assert_eq!(storage.inner.read_all().unwrap(), before);
        }
        if mode.signed() {
            assert!(move_variables(
                &mut storage,
                archive(),
                mode,
                &authority,
                None,
                key(mode),
                &[(name("alpha"), name("x"))]
            )
            .is_err());
            assert_eq!(storage.inner.read_all().unwrap(), before);
        }
        let mut expected = initial.clone();
        for changes in [
            vec![(name("alpha"), name("beta")), (name("beta"), name("alpha"))],
            vec![
                (name("beta"), name("/docs/neighbor")),
                (name("alpha"), name("/branch/secret")),
            ],
            vec![(name("/branch/secret"), name("branch"))],
        ] {
            assert!(move_variables(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &changes
            )
            .unwrap());
            expected = renamed(expected, &changes);
            check(storage.clone(), mode, &authority, &expected, &initial);
        }
        let after = storage.inner.read_all().unwrap();
        for (start, end) in &storage.payloads {
            assert_eq!(
                &after[*start as usize..*end as usize],
                &before[*start as usize..*end as usize]
            );
        }
        for bank in [0, FAILURE_REGION] {
            let mut damaged = StorageBackend::memory(after.clone());
            damaged
                .write_at(bank, &vec![0; FAILURE_REGION as usize])
                .unwrap();
            let mut found = Vec::new();
            assert!(salvage_variables(
                &guard(damaged, &original, mode, &authority),
                archive(),
                mode,
                &authority,
                key(mode),
                |n, _, v| {
                    v.with_str(|s| assert!(!s.is_empty()))?;
                    found.push(n.clone());
                    Ok(())
                }
            )
            .unwrap()
            .is_empty());
            assert_eq!(found, expected.keys().cloned().collect::<Vec<_>>());
        }
        // Required authentication still rejects damaged content, including a
        // no-op request; metadata-only does not mean unauthenticated payloads.
        let mut damaged = StorageBackend::memory(after);
        let at = initial[&name("beta")].extents[0].start;
        let byte = damaged.read_at(at, 1).unwrap()[0] ^ 1;
        damaged.write_at(at, &[byte]).unwrap();
        let stable = damaged.read_all().unwrap();
        assert!(move_variables(
            &mut damaged,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &[]
        )
        .is_err());
        assert_eq!(damaged.read_all().unwrap(), stable);
    }
}

#[test]
fn typed_tree_variable_moves_atomic_guarded_recovery_faults() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    let mut recovery_cases = 0;
    for bits in [0, 1, 2, 3, 12, 13, 14, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let original = seed(mode, &authority, &owner);
        let bytes = original.read_all().unwrap();
        let old = names(original.clone(), mode, &authority);
        let changes = vec![(name("alpha"), name("beta")), (name("beta"), name("moved"))];
        let new = renamed(old.clone(), &changes);
        let run = |s: &mut Stable<CrashStore>| {
            move_variables(s, archive(), mode, &authority, signer, key(mode), &changes)
        };
        let mut observed = guard(
            CrashStore::new(bytes.clone(), None, 0, false),
            &original,
            mode,
            &authority,
        );
        run(&mut observed).unwrap();
        let count = observed.inner.operations();
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
                    let damaged = failed.inner.durable();
                    let verify = |s: &Stable<StorageBackend>| {
                        let selected = names(s.clone(), mode, &authority);
                        assert!(selected == old || selected == new);
                        check(s.clone(), mode, &authority, &selected, &old);
                        let recovered = s.inner.read_all().unwrap();
                        for (start, end) in &s.payloads {
                            assert_eq!(
                                &recovered[*start as usize..*end as usize],
                                &bytes[*start as usize..*end as usize]
                            );
                        }
                        if selected == old && recovered.len() > bytes.len() {
                            assert!(recovered[bytes.len()..].iter().all(|b| *b == 0));
                        }
                        selected
                    };
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
                        for cut in 0..observed.inner.operations() {
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
                                    StorageBackend::memory(interrupted.inner.durable()),
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
    println!("TYPED_VARIABLE_MOVE_FAULTS cases={cases} interrupted_recovery={recovery_cases}");
}
