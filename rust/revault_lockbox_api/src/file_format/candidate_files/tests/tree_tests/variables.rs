//! Internal adapter fixtures: no public CLI writes the experimental tree yet.
use super::*;
use crate::{SecretString, VariableName};
use tree_image::variables::{delete_variable, set_secret_variable, set_variable};

#[test]
fn typed_tree_variables_lifecycle_secure_read_and_file_coexistence_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let name = VariableName::new("/docs/data").unwrap();
    let other = VariableName::new("neighbor").unwrap();
    let mut boundary = "x".repeat(65535);
    boundary.push('🦀');
    let secret = SecretString::try_from_slice(boundary.as_bytes()).unwrap();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = super::mutation::seed(mode, &authority, &owner);
        assert!(
            TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
                .unwrap()
                .image
                .value_key
                .is_none()
        );
        assert!(set_variable(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &name,
            "normal"
        )
        .unwrap());
        assert!(set_variable(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &other,
            "neighbor"
        )
        .unwrap());
        let opened =
            TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
        assert_eq!(
            opened.get_variable(&name).unwrap().as_deref(),
            Some("normal")
        );
        assert!(opened.with_secret_variable(&name, |_| ()).is_err());
        assert!(set_secret_variable(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &name,
            &secret
        )
        .unwrap());
        let stable = storage.read_all().unwrap();
        assert!(!set_secret_variable(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &name,
            &secret
        )
        .unwrap());
        assert_eq!(storage.read_all().unwrap(), stable);
        assert!(set_variable(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &name,
            "downgrade"
        )
        .is_err());
        for conflict in ["/docs", "/docs/data/child"] {
            assert!(set_variable(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &VariableName::new(conflict).unwrap(),
                "invalid"
            )
            .is_err());
        }
        assert!(tree_image::return_inline(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode)
        )
        .is_err());
        assert_eq!(storage.read_all().unwrap(), stable);
        let opened =
            TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
        assert!(opened.image.value_key.is_some());
        assert!(opened.get_variable(&name).is_err());
        assert_eq!(
            opened
                .with_secret_variable(&name, |v| v.with_str(|s| assert_eq!(s, boundary)).unwrap())
                .unwrap(),
            Some(())
        );
        assert_eq!(
            opened.get_variable(&other).unwrap().as_deref(),
            Some("neighbor")
        );
        let mut metadata = opened.image.filesystem_metadata().unwrap();
        metadata
            .iter_mut()
            .find(|m| m.entry.path.as_str() == "/docs/neighbor")
            .unwrap()
            .entry
            .permissions = 0o600;
        assert!(tree_image::replace_metadata(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &metadata
        )
        .unwrap());
        // The file and variable namespaces are independent. Existing file writes
        // and removals must preserve selected variable metadata and ownership.
        assert!(tree_image::update_files(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            vec![Input {
                path: b"/docs/data".to_vec(),
                reader: Cursor::new(b"replacement".to_vec())
            }],
            &[]
        )
        .unwrap());
        assert!(tree_image::remove_files(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &[b"/docs/data".to_vec()]
        )
        .unwrap());
        let opened =
            TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
        assert!(opened.image.info(b"/docs/data").unwrap().is_none());
        opened
            .with_secret_variable(&name, |v| v.with_str(|s| assert_eq!(s, boundary)).unwrap())
            .unwrap()
            .unwrap();
        assert!(delete_variable(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &name
        )
        .unwrap());
        let deleted = storage.read_all().unwrap();
        assert!(!delete_variable(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &name
        )
        .unwrap());
        assert_eq!(storage.read_all().unwrap(), deleted);
        assert!(set_variable(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &name,
            "recreated"
        )
        .unwrap());
        let opened =
            TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
        assert_eq!(
            opened.get_variable(&name).unwrap().as_deref(),
            Some("recreated")
        );
        assert!(delete_variable(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &name
        )
        .unwrap());
        assert!(delete_variable(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &other
        )
        .unwrap());
        tree_image::return_inline(&mut storage, archive(), mode, &authority, signer, key(mode))
            .unwrap();
        assert!(super::super::super::dense_image::Image::open(
            storage,
            archive(),
            mode,
            &authority,
            key(mode)
        )
        .unwrap()
        .value_key
        .is_none());
    }
}

#[test]
fn typed_tree_variables_maximum_empty_and_one_over_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let name = VariableName::new("maximum").unwrap();
    let empty = SecretString::new();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = super::mutation::seed(mode, &authority, &owner);
        let value = SecretString::try_from_slice(&vec![b'm'; 1024 * 1024]).unwrap();
        set_secret_variable(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &name,
            &value,
        )
        .unwrap();
        drop(value);
        let opened =
            TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
        opened
            .with_secret_variable(&name, |v| {
                v.with_str(|s| {
                    assert_eq!(s.len(), 1024 * 1024);
                    assert!(s.bytes().all(|b| b == b'm'));
                })
                .unwrap()
            })
            .unwrap()
            .unwrap();
        let before = storage.read_all().unwrap();
        let too_large = SecretString::try_from_slice(&vec![b'm'; 1024 * 1024 + 1]).unwrap();
        assert!(matches!(
            set_secret_variable(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &name,
                &too_large
            ),
            Err(Error::SecurityLimitExceeded(_))
        ));
        drop(too_large);
        assert_eq!(storage.read_all().unwrap(), before);
        set_secret_variable(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &name,
            &empty,
        )
        .unwrap();
        let opened =
            TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
        opened
            .with_secret_variable(&name, |v| v.with_str(|s| assert!(s.is_empty())).unwrap())
            .unwrap()
            .unwrap();
    }
}
use crate::file_format::allocation_map::Extent;
use crate::file_format::preparation_journal::tests::CrashStore;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
struct Guarded<S> {
    inner: S,
    spans: Rc<RefCell<Vec<(u64, u64)>>>,
}
impl<S: Storage> Guarded<S> {
    fn new(inner: S, spans: Vec<(u64, u64)>) -> Self {
        Self {
            inner,
            spans: Rc::new(RefCell::new(spans)),
        }
    }
    fn track(&self, at: u64, bytes: &[u8]) {
        if bytes.starts_with(crate::file_format::page::PAGE_MAGIC) {
            // Register the entire attempted segment before a possible short write.
            self.spans.borrow_mut().push((at, at + bytes.len() as u64));
        }
    }
}
impl<S: Storage> Storage for Guarded<S> {
    fn len(&self) -> Result<u64> {
        self.inner.len()
    }
    fn read_at(&self, at: u64, len: usize) -> Result<Vec<u8>> {
        assert!(!self.spans.borrow().iter().any(|(start,end)| at < *end && *start < at+len as u64),
            "ordinary read overlaps a live, retired, or incompletely staged variable segment: {at}+{len}");
        self.inner.read_at(at, len)
    }
    fn read_at_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        self.inner.read_at_into(at, out)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        self.track(self.len()?, bytes);
        self.inner.append(bytes)
    }
    fn write_at(&mut self, at: u64, bytes: &[u8]) -> Result<()> {
        self.track(at, bytes);
        self.inner.write_at(at, bytes)
    }
    fn truncate(&mut self, len: u64) -> Result<()> {
        self.inner.truncate(len)
    }
    fn sync(&self) -> Result<()> {
        self.inner.sync()
    }
}
fn extents(storage: &StorageBackend, mode: FormatMode, authority: &Authority<'_>) -> Vec<Extent> {
    TreeImage::open(storage.clone(), archive(), mode, authority, key(mode))
        .unwrap()
        .image
        .catalogue
        .variables
        .into_iter()
        .flat_map(|v| v.layout.extents)
        .collect()
}
fn check_value<S: Storage>(
    storage: S,
    mode: FormatMode,
    authority: &Authority<'_>,
    name: &VariableName,
) -> Option<bool> {
    let mut opened = TreeImage::open(storage, archive(), mode, authority, key(mode)).unwrap();
    assert_eq!(
        opened
            .get_variable(&VariableName::new("retained").unwrap())
            .unwrap()
            .as_deref(),
        Some("stable")
    );
    let mut neighbor = Vec::new();
    opened
        .image
        .read_range(b"/docs/neighbor", 0, 8, |bytes| {
            neighbor.extend_from_slice(bytes);
            Ok(())
        })
        .unwrap();
    assert_eq!(neighbor, b"neighbor");
    opened
        .with_secret_variable(name, |v| {
            v.with_str(|s| {
                assert_eq!(s.len(), 65543);
                assert!(s.bytes().all(|b| b == s.as_bytes()[0]));
                assert!(matches!(s.as_bytes()[0], b'a' | b'b'));
                s.as_bytes()[0] == b'b'
            })
            .unwrap()
        })
        .unwrap()
}

#[test]
fn typed_tree_variables_guarded_transaction_and_recovery_faults() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let name = VariableName::new("secret").unwrap();
    let a = SecretString::try_from_slice(&vec![b'a'; 65543]).unwrap();
    let b = SecretString::try_from_slice(&vec![b'b'; 65543]).unwrap();
    let mut cases = 0;
    let mut recovery_cases = 0;
    for bits in [0, 1, 2, 3, 12, 13, 14, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut original = super::mutation::seed(mode, &authority, &owner);
        set_secret_variable(
            &mut original,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &name,
            &a,
        )
        .unwrap();
        set_variable(
            &mut original,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &VariableName::new("retained").unwrap(),
            "stable",
        )
        .unwrap();
        let opened =
            TreeImage::open(original.clone(), archive(), mode, &authority, key(mode)).unwrap();
        let target_ranges: Vec<_> = opened
            .image
            .catalogue
            .variables
            .iter()
            .find(|v| v.name == name)
            .unwrap()
            .layout
            .extents
            .iter()
            .map(|e| (e.start, e.start + e.len))
            .collect();
        let ranges: Vec<_> = extents(&original, mode, &authority)
            .iter()
            .map(|e| (e.start, e.start + e.len))
            .collect();
        let seed = original.read_all().unwrap();
        for delete in [false, true] {
            let run = |s: &mut Guarded<CrashStore>| {
                if delete {
                    delete_variable(s, archive(), mode, &authority, signer, key(mode), &name)
                } else {
                    set_secret_variable(
                        s,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        &name,
                        &b,
                    )
                }
            };
            let mut observed = Guarded::new(
                CrashStore::new(seed.clone(), None, 0, false),
                ranges.clone(),
            );
            run(&mut observed).unwrap();
            let count = observed.inner.operations();
            for at in 0..count {
                for prefix in [0, 97, usize::MAX] {
                    for persist in [false, true] {
                        let mut failed = Guarded::new(
                            CrashStore::new(seed.clone(), Some(at), prefix, persist),
                            ranges.clone(),
                        );
                        let _ = run(&mut failed);
                        let damaged = failed.inner.durable();
                        let spans = failed.spans.borrow().clone();
                        let mut recovered =
                            Guarded::new(StorageBackend::memory(damaged.clone()), spans.clone());
                        tree_image::recover(&mut recovered,archive(),mode,&authority,key(mode)).unwrap_or_else(|e| panic!("mode={bits} delete={delete} at={at} prefix={prefix} persist={persist}: {e}"));
                        let selected = check_value(recovered.clone(), mode, &authority, &name);
                        if delete {
                            assert_ne!(selected, Some(true));
                        } else {
                            assert!(selected.is_some());
                        }
                        tree_image::recover(&mut recovered, archive(), mode, &authority, key(mode))
                            .unwrap();
                        assert_eq!(
                            check_value(recovered.clone(), mode, &authority, &name),
                            selected
                        );
                        check_erasure(&recovered, selected, &ranges, &target_ranges, &spans);
                        cases += 1;
                        // Exhaust recovery operations for five deterministic cut
                        // positions per transition, not every cross product.
                        if prefix == 97
                            && persist
                            && [0, count / 4, count / 2, 3 * count / 4, count - 1].contains(&at)
                        {
                            let mut observed = Guarded::new(
                                CrashStore::new(damaged.clone(), None, 0, false),
                                spans.clone(),
                            );
                            tree_image::recover(
                                &mut observed,
                                archive(),
                                mode,
                                &authority,
                                key(mode),
                            )
                            .unwrap();
                            for recovery_at in 0..observed.inner.operations() {
                                for recovery_prefix in [0, 97, usize::MAX] {
                                    let mut interrupted = Guarded::new(
                                        CrashStore::new(
                                            damaged.clone(),
                                            Some(recovery_at),
                                            recovery_prefix,
                                            true,
                                        ),
                                        spans.clone(),
                                    );
                                    let _ = tree_image::recover(
                                        &mut interrupted,
                                        archive(),
                                        mode,
                                        &authority,
                                        key(mode),
                                    );
                                    let mut resumed = Guarded::new(
                                        StorageBackend::memory(interrupted.inner.durable()),
                                        interrupted.spans.borrow().clone(),
                                    );
                                    tree_image::recover(
                                        &mut resumed,
                                        archive(),
                                        mode,
                                        &authority,
                                        key(mode),
                                    )
                                    .unwrap();
                                    assert_eq!(
                                        check_value(resumed.clone(), mode, &authority, &name),
                                        selected
                                    );
                                    check_erasure(
                                        &resumed,
                                        selected,
                                        &ranges,
                                        &target_ranges,
                                        &resumed.spans.borrow(),
                                    );
                                    recovery_cases += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("TYPED_VARIABLE_FAULTS {cases} INTERRUPTED_RECOVERY {recovery_cases}");
}
#[test]
fn typed_tree_variables_selected_ownership_corruption_and_salvage_all_modes() {
    use tree_image::variables::salvage_variables;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let first = VariableName::new("first").unwrap();
    let second = VariableName::new("second").unwrap();
    let a = SecretString::try_from_slice(&[b'a'; 512]).unwrap();
    let b = SecretString::try_from_slice(&[b'b'; 512]).unwrap();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut seed = super::mutation::seed(mode, &authority, &owner);
        set_secret_variable(
            &mut seed,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &first,
            &a,
        )
        .unwrap();
        set_secret_variable(
            &mut seed,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &second,
            &b,
        )
        .unwrap();
        let opened = TreeImage::open(seed.clone(), archive(), mode, &authority, key(mode)).unwrap();
        let layouts: Vec<_> = opened
            .image
            .catalogue
            .variables
            .iter()
            .map(|v| v.layout.clone())
            .collect();
        let rows = opened.image.catalogue.tree_records().unwrap();
        for corruption in 0..9 {
            let mut changed = rows.clone();
            let at = changed.iter().position(|r| r.namespace == 6).unwrap();
            match corruption {
                0 => {
                    changed.remove(at);
                } // orphan payload claim
                1 => {
                    changed[at].value = zeroize::Zeroizing::new(layouts[1].encode_metadata());
                } // duplicate extent/identity
                2 => {
                    changed[at].value[24] ^= 1;
                } // selected revision vs page prefix
                3 => {
                    changed[at].value[34] ^= 1;
                } // sensitivity vs page prefix
                4 => {
                    changed[at].value[8] ^= 1;
                } // logical UUID vs page context
                5 => {
                    changed[at].key = zeroize::Zeroizing::new(b"first".to_vec());
                } // noncanonical name
                6 => {
                    changed[at].value[35] = 1;
                } // noncanonical reserved field
                7 => {
                    changed[at].key = zeroize::Zeroizing::new(b"/second/child".to_vec());
                } // namespace conflict
                _ => {
                    // Exchange same-sized complete extent descriptors. The
                    // selected payload union and every stored digest still pass;
                    // each selected logical identity must reject the other page.
                    assert_eq!(layouts[0].extents[0].len, layouts[1].extents[0].len);
                    let mut a = layouts[0].clone();
                    let mut b = layouts[1].clone();
                    std::mem::swap(&mut a.extents, &mut b.extents);
                    changed[at].value = zeroize::Zeroizing::new(a.encode_metadata());
                    changed[at + 1].value = zeroize::Zeroizing::new(b.encode_metadata());
                }
            }
            changed.sort_by(|a, b| {
                (a.namespace, a.key.as_slice()).cmp(&(b.namespace, b.key.as_slice()))
            });
            let mut bad = StorageBackend::memory(seed.read_all().unwrap());
            shared::tree::rewrite_records(
                &mut bad,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                changed,
            )
            .unwrap();
            let read = TreeImage::open(bad, archive(), mode, &authority, key(mode));
            match read {
                Err(_) => {}
                Ok(mut image) => assert!(image.image.verify_all().is_err()),
            }
        }
        let mut damaged = StorageBackend::memory(seed.read_all().unwrap());
        let extent = layouts[0].extents[0];
        let byte = damaged.read_at(extent.start + extent.len - 1, 1).unwrap()[0] ^ 1;
        damaged
            .write_at(extent.start + extent.len - 1, &[byte])
            .unwrap();
        if mode.plaintext() && mode.signed() {
            assert!(
                TreeImage::open(damaged.clone(), archive(), mode, &authority, key(mode)).is_err()
            );
        }
        let mut names = Vec::new();
        let lost = salvage_variables(
            &damaged,
            archive(),
            mode,
            &authority,
            key(mode),
            |name, _, value| {
                names.push(name.clone());
                value.with_str(|s| assert_eq!(s, "b".repeat(512)))?;
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(lost, vec![first.clone()]);
        assert_eq!(names, vec![second.clone()]);
        // Independent region loss is a different locality case from the
        // single-byte corruption above. Record co-loss instead of assuming it
        // isolates one variable; selected metadata still requires its own proof.
        let mut region_loss = StorageBackend::memory(seed.read_all().unwrap());
        let region = extent.start / FAILURE_REGION * FAILURE_REGION;
        region_loss
            .write_at(region, &vec![0; FAILURE_REGION as usize])
            .unwrap();
        let mut recovered = Vec::new();
        let lost = salvage_variables(
            &region_loss,
            archive(),
            mode,
            &authority,
            key(mode),
            |name, _, value| {
                value.with_str(|s| {
                    assert_eq!(
                        s,
                        if name == &first {
                            "a".repeat(512)
                        } else {
                            "b".repeat(512)
                        }
                    )
                })?;
                recovered.push(name.clone());
                Ok(())
            },
        )
        .unwrap();
        assert!(lost.contains(&first));
        assert_eq!(lost.len() + recovered.len(), 2);
        println!(
            "TYPED_VARIABLE_REGION_LOSS mode={} unavailable={} retained={}",
            mode.0,
            lost.len(),
            recovered.len()
        );
        delete_variable(
            &mut seed,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &first,
        )
        .unwrap();
        for bank in [0, FAILURE_REGION] {
            let mut damaged = StorageBackend::memory(seed.read_all().unwrap());
            damaged
                .write_at(bank, &vec![0; FAILURE_REGION as usize])
                .unwrap();
            let mut names = Vec::new();
            assert!(salvage_variables(
                &damaged,
                archive(),
                mode,
                &authority,
                key(mode),
                |name, _, _| {
                    names.push(name.clone());
                    Ok(())
                }
            )
            .unwrap()
            .is_empty());
            assert_eq!(names, vec![second.clone()]);
        }
        let current =
            TreeImage::open(seed.clone(), archive(), mode, &authority, key(mode)).unwrap();
        let mut damaged = StorageBackend::memory(seed.read_all().unwrap());
        for start in [
            current.tree.anchor.index.primary,
            current.tree.anchor.index.mirror,
        ] {
            damaged
                .write_at(start, &vec![0; current.tree.anchor.index.len as usize])
                .unwrap();
        }
        let mut called = false;
        assert!(salvage_variables(
            &damaged,
            archive(),
            mode,
            &authority,
            key(mode),
            |_, _, _| {
                called = true;
                Ok(())
            }
        )
        .is_err());
        assert!(!called);
    }
}

#[test]
fn typed_tree_variable_max_revision_can_be_deleted_but_not_replaced() {
    use crate::file_format::secure_segments;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, false, false, false);
    let authority = authority(mode, &public);
    let name = VariableName::new("last_revision").unwrap();
    let mut storage = super::mutation::seed(mode, &authority, &owner);
    let value = SecretString::try_from_slice(b"last").unwrap();
    set_secret_variable(
        &mut storage,
        archive(),
        mode,
        &authority,
        None,
        key(mode),
        &name,
        &value,
    )
    .unwrap();
    let opened = TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
    let bytes = crate::secret_vec::SecureVec::try_from_slice(b"last").unwrap();
    let encoded = secure_segments::encode(
        &bytes,
        archive(),
        mode,
        opened.image.value_key.as_ref().unwrap(),
        opened.image.catalogue.variables[0].layout.id,
        u64::MAX,
        crate::VariableSensitivity::Secret,
    )
    .unwrap();
    let base = shared::commitment(&opened.tree.anchor).unwrap();
    let mut catalogue = opened.image.catalogue;
    let retired = catalogue.variables[0].layout.extents.clone();
    catalogue.variables[0].layout = encoded.layout;
    let rows = catalogue.tree_records().unwrap();
    shared::tree::rewrite_secure_payload_records(
        &mut storage,
        archive(),
        mode,
        &authority,
        None,
        key(mode),
        rows,
        shared::tree::SecurePayloadPlan {
            base,
            retired,
            bytes: encoded.pages,
            rebind: Box::new(move |extents| {
                catalogue.variables[0].layout.rebind(extents)?;
                catalogue.tree_records()
            }),
        },
    )
    .unwrap();
    let before = storage.read_all().unwrap();
    assert!(!set_secret_variable(
        &mut storage,
        archive(),
        mode,
        &authority,
        None,
        key(mode),
        &name,
        &value
    )
    .unwrap());
    let next = SecretString::try_from_slice(b"next").unwrap();
    assert!(set_secret_variable(
        &mut storage,
        archive(),
        mode,
        &authority,
        None,
        key(mode),
        &name,
        &next
    )
    .is_err());
    assert_eq!(storage.read_all().unwrap(), before);
    assert!(delete_variable(
        &mut storage,
        archive(),
        mode,
        &authority,
        None,
        key(mode),
        &name
    )
    .unwrap());
    assert!(
        TreeImage::open(storage, archive(), mode, &authority, key(mode))
            .unwrap()
            .with_secret_variable(&name, |_| ())
            .unwrap()
            .is_none()
    );
}

fn check_erasure(
    storage: &impl Storage,
    selected: Option<bool>,
    original: &[(u64, u64)],
    retired: &[(u64, u64)],
    attempted: &[(u64, u64)],
) {
    // Every old segment no longer selected must be durably erased.
    if selected != Some(false) {
        for (start, end) in retired {
            let bytes = storage
                .read_at_secure(*start, (*end - *start) as usize)
                .unwrap();
            assert!(bytes.with_bytes(|v| v.iter().all(|b| *b == 0)).unwrap());
        }
    } else {
        // Aborted reservations, including partial writes outside the committed
        // graph, must be erased or absent after truncation, also on resume.
        let physical = storage.len().unwrap();
        for (start, end) in attempted.iter().filter(|span| !original.contains(span)) {
            if *start < physical {
                let bytes = storage
                    .read_at_secure(*start, (end.min(&physical) - start) as usize)
                    .unwrap();
                assert!(bytes.with_bytes(|v| v.iter().all(|b| *b == 0)).unwrap());
            }
        }
    }
}
