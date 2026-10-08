//! Direct candidate fixtures: public CLI does not write this experimental tree.
use super::super::dense_catalogue::Metadata;
use super::super::tree_image::{self, TreeImage};
use super::*;
use crate::file_format::publication_anchor::{shared, FAILURE_REGION};
mod fresh;
mod mutation;
mod open_reads;

fn exported(
    mode: FormatMode,
    authority: &Authority<'_>,
    owner: &OwnerSigningKeyPair,
    entries: &[Metadata],
) -> StorageBackend {
    let mut source = canonical_dense_seed(mode, authority, mode.signed().then_some(owner));
    super::super::dense_update::replace_filesystem_metadata(
        &mut source,
        archive(),
        mode,
        authority,
        mode.signed().then_some(owner),
        key(mode),
        entries,
    )
    .unwrap();
    let original = source.read_all().unwrap();
    let result = tree_image::from_dense(
        &source,
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        authority,
        mode.signed().then_some(owner),
        key(mode),
    )
    .unwrap();
    assert_eq!(source.read_all().unwrap(), original);
    result
}
fn check(
    storage: &StorageBackend,
    mode: FormatMode,
    authority: &Authority<'_>,
    entries: &[Metadata],
) {
    let mut image = TreeImage::open(
        StorageBackend::memory(storage.read_all().unwrap()),
        archive(),
        mode,
        authority,
        key(mode),
    )
    .unwrap();
    assert_eq!(image.image.filesystem_metadata().unwrap(), entries);
    let mut bytes = Vec::new();
    image
        .image
        .read_range(b"/docs/data", 0, 7, |part| {
            bytes.extend_from_slice(part);
            Ok(())
        })
        .unwrap();
    assert_eq!(bytes, b"payload");
}
fn grown(entries: &[Metadata]) -> Vec<Metadata> {
    let mut result = entries.to_vec();
    for i in 0..2500 {
        result.push(Metadata {
            entry: crate::LockboxEntry {
                path: crate::LockboxPath::new(format!(
                    "/directory-{i:04}-with-authenticated-metadata"
                ))
                .unwrap(),
                kind: crate::LockboxEntryKind::Directory,
                len: 0,
                permissions: 0o750,
            },
            target: None,
        });
    }
    result.sort_by(|a, b| a.entry.path.cmp(&b.entry.path));
    result
}
#[test]
fn typed_tree_filesystem_growth_deletion_permissions_and_copy_loss_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let original = public_filesystem_metadata(&owner);
    let large = grown(&original);
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let mut storage = exported(mode, &authority, &owner, &original);
        check(&storage, mode, &authority, &original);
        assert!(tree_image::replace_metadata(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &large
        )
        .unwrap());
        check(&storage, mode, &authority, &large);
        let before = storage.read_all().unwrap();
        assert!(!tree_image::replace_metadata(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &large
        )
        .unwrap());
        assert_eq!(storage.read_all().unwrap(), before);
        let mut changed = original.clone();
        changed
            .iter_mut()
            .find(|entry| entry.entry.kind == crate::LockboxEntryKind::File)
            .unwrap()
            .entry
            .permissions = 0o600;
        changed
            .iter_mut()
            .find(|entry| entry.target.is_some())
            .unwrap()
            .target = Some(crate::LockboxPath::new("/empty").unwrap());
        tree_image::replace_metadata(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &changed,
        )
        .unwrap();
        check(&storage, mode, &authority, &changed);
        let payload = TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
            .unwrap()
            .tree
            .graph
            .payloads()[0];
        let mut damaged = StorageBackend::memory(storage.read_all().unwrap());
        let byte = damaged.read_at(payload.start, 1).unwrap()[0] ^ 1;
        damaged.write_at(payload.start, &[byte]).unwrap();
        let opened = TreeImage::open(damaged, archive(), mode, &authority, key(mode));
        if mode.plaintext() && mode.signed() {
            assert!(opened.is_err());
        } else {
            assert!(opened
                .unwrap()
                .image
                .read_range(b"/docs/data", 0, 7, |_| Ok(()))
                .is_err());
        }
        for bank in [0, FAILURE_REGION] {
            let mut lost = StorageBackend::memory(storage.read_all().unwrap());
            lost.write_at(bank, &vec![0; FAILURE_REGION as usize])
                .unwrap();
            shared::tree::recover_update(&mut lost, archive(), mode, &authority, key(mode))
                .unwrap();
            check(&lost, mode, &authority, &changed);
        }
        println!(
            "TYPED_TREE_LIFECYCLE mode={bits} nodes={} completed_bytes={}",
            large.len(),
            storage.len().unwrap()
        );
    }
}

#[test]
fn typed_tree_interrupted_growth_keeps_complete_selected_metadata() {
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let entries = public_filesystem_metadata(&owner);
    let large = grown(&entries);
    let mut cases = 0;
    for bits in [0, 3, 12, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let seed = exported(mode, &authority, &owner, &entries)
            .read_all()
            .unwrap();
        let run = |storage: &mut CrashStore| {
            tree_image::replace_metadata(
                storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                &large,
            )
        };
        let mut observed = CrashStore::new(seed.clone(), None, 0, false);
        run(&mut observed).unwrap();
        for at in 0..observed.operations() {
            for prefix in [0, 97, usize::MAX] {
                for persist in [false, true] {
                    let mut failed = CrashStore::new(seed.clone(), Some(at), prefix, persist);
                    let _ = run(&mut failed);
                    let mut reopened = StorageBackend::memory(failed.durable());
                    shared::tree::recover_update(
                        &mut reopened,
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap_or_else(|e| {
                        panic!("mode={bits} at={at} prefix={prefix} persist={persist}: {e}")
                    });
                    let actual =
                        TreeImage::open(reopened.clone(), archive(), mode, &authority, key(mode))
                            .unwrap()
                            .image
                            .filesystem_metadata()
                            .unwrap();
                    assert!(actual == entries || actual == large);
                    check(&reopened, mode, &authority, &actual);
                    cases += 1;
                }
            }
        }
    }
    println!("TYPED_TREE_INTERRUPTION_CASES {cases}");
}

#[test]
fn typed_tree_rejects_selected_pack_claim_mismatch() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(true, true, false, false);
    let authority = authority(mode, &public);
    let entries = public_filesystem_metadata(&owner);
    let mut storage = exported(mode, &authority, &owner, &entries);
    let tree = shared::tree::Tree::open(&storage, archive(), mode, &authority, key(mode)).unwrap();
    let mut rows = Vec::new();
    tree.visit(&storage, |entry| {
        rows.push(Entry::new(entry.namespace, &entry.key, &entry.value)?);
        Ok(())
    })
    .unwrap();
    rows.iter_mut()
        .find(|entry| entry.namespace == 3)
        .unwrap()
        .value[8] ^= 1;
    shared::tree::rewrite_records(
        &mut storage,
        archive(),
        mode,
        &authority,
        Some(&owner),
        key(mode),
        rows,
    )
    .unwrap();
    let before = storage.read_all().unwrap();
    assert!(TreeImage::open(
        allocation::compaction::View(&storage),
        archive(),
        mode,
        &authority,
        key(mode)
    )
    .is_err());
    assert_eq!(storage.read_all().unwrap(), before);
}

#[test]
fn typed_tree_salvage_preserves_intact_neighbor_and_selected_membership_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut entries = public_filesystem_metadata(&owner);
    entries.push(Metadata {
        entry: crate::LockboxEntry {
            path: crate::LockboxPath::new("/docs/neighbor").unwrap(),
            kind: crate::LockboxEntryKind::File,
            len: 8,
            permissions: 0o640,
        },
        target: None,
    });
    entries.sort_by(|a, b| a.entry.path.cmp(&b.entry.path));
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let storage = Files::create(
            StorageBackend::memory(Vec::new()),
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            65536,
            [
                Input {
                    path: b"/docs/data".to_vec(),
                    reader: Cursor::new(b"payload".to_vec()),
                },
                Input {
                    path: b"/docs/neighbor".to_vec(),
                    reader: Cursor::new(b"neighbor".to_vec()),
                },
            ],
        )
        .unwrap();
        let mut source = Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
        let mut dense = super::super::dense_image::from_candidate(
            &mut source,
            StorageBackend::memory(Vec::new()),
            &authority,
            signer,
            key(mode),
            &[],
        )
        .unwrap();
        super::super::dense_update::replace_filesystem_metadata(
            &mut dense,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &entries,
        )
        .unwrap();
        let storage = tree_image::from_dense(
            &dense,
            StorageBackend::memory(Vec::new()),
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
        )
        .unwrap();
        let image =
            TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
        let file = &image.image.catalogue.files[0];
        let fragment = &file.fragments[0];
        let at = image.image.catalogue.packs[fragment.pack].extent.start + fragment.relative as u64;
        for damage in 0..5 {
            let mut damaged = storage.clone();
            match damage {
                1 => {
                    let byte = damaged.read_at(at, 1).unwrap()[0] ^ 1;
                    damaged.write_at(at, &[byte]).unwrap();
                }
                2 => {
                    damaged
                        .truncate(damaged.len().unwrap() - FAILURE_REGION)
                        .unwrap();
                }
                3 => {
                    damaged
                        .write_at(0, &vec![0; FAILURE_REGION as usize])
                        .unwrap();
                }
                4 => {
                    damaged
                        .write_at(FAILURE_REGION, &vec![0; FAILURE_REGION as usize])
                        .unwrap();
                }
                _ => (),
            }
            let before = damaged.read_all().unwrap();
            let mut sink = FilesystemSalvaged::default();
            let report = tree_image::salvage_filesystem(
                &allocation::compaction::View(&damaged),
                archive(),
                mode,
                &authority,
                key(mode),
                &mut sink,
            )
            .unwrap_or_else(|e| panic!("mode={bits} damage={damage}: {e}"));
            assert_eq!(report.complete, if damage == 1 { 1 } else { 2 });
            assert_eq!(report.incomplete, u64::from(damage == 1));
            assert_eq!(sink.metadata.as_ref().unwrap(), &entries);
            assert_eq!(damaged.read_all().unwrap(), before);
            assert_eq!(
                sink.files.files.get(b"/docs/neighbor".as_slice()).unwrap(),
                b"neighbor"
            );
        }
        // Delete a node, then lose selected membership. Older signed history
        // must not resurrect that node or permit any sink output.
        let mut selected = storage.clone();
        let mut changed = entries.clone();
        changed.retain(|entry| entry.entry.path.as_str() != "/link");
        tree_image::replace_metadata(
            &mut selected,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &changed,
        )
        .unwrap();
        let (_, body) =
            shared::open_private(&selected, archive(), mode, &authority, key(mode)).unwrap();
        for offset in [8, 16] {
            let start = u64::from_le_bytes(body[offset..offset + 8].try_into().unwrap());
            selected
                .write_at(start, &vec![0; FAILURE_REGION as usize])
                .unwrap();
        }
        let before = selected.read_all().unwrap();
        let mut sink = FilesystemSalvaged::default();
        assert!(tree_image::salvage_filesystem(
            &allocation::compaction::View(&selected),
            archive(),
            mode,
            &authority,
            key(mode),
            &mut sink
        )
        .is_err());
        assert!(sink.metadata.is_none());
        assert_eq!(selected.read_all().unwrap(), before);
    }
}

fn inline_seed(
    mode: FormatMode,
    authority: &Authority<'_>,
    owner: &OwnerSigningKeyPair,
    entries: &[Metadata],
) -> StorageBackend {
    let mut storage = canonical_dense_seed(mode, authority, mode.signed().then_some(owner));
    super::super::dense_update::replace_filesystem_metadata(
        &mut storage,
        archive(),
        mode,
        authority,
        mode.signed().then_some(owner),
        key(mode),
        entries,
    )
    .unwrap();
    super::super::dense_update::return_inline(
        &mut storage,
        archive(),
        mode,
        authority,
        mode.signed().then_some(owner),
        key(mode),
    )
    .unwrap();
    storage
}
fn check_selected(
    storage: &StorageBackend,
    mode: FormatMode,
    authority: &Authority<'_>,
) -> Vec<Metadata> {
    let (_, body) = shared::open_private(storage, archive(), mode, authority, key(mode)).unwrap();
    let mut image = if body.starts_with(b"RV4TRE01") {
        TreeImage::open(
            StorageBackend::memory(storage.read_all().unwrap()),
            archive(),
            mode,
            authority,
            key(mode),
        )
        .unwrap()
        .image
    } else {
        super::super::dense_image::Image::open(
            StorageBackend::memory(storage.read_all().unwrap()),
            archive(),
            mode,
            authority,
            key(mode),
        )
        .unwrap()
    };
    let mut payload = Vec::new();
    image
        .read_range(b"/docs/data", 0, 7, |part| {
            payload.extend_from_slice(part);
            Ok(())
        })
        .unwrap();
    assert_eq!(payload, b"payload");
    image.filesystem_metadata().unwrap()
}
#[test]
fn typed_tree_inline_growth_and_return_reclaim_tail_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let original = public_filesystem_metadata(&owner);
    let large = grown(&original);
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = inline_seed(mode, &authority, &owner, &original);
        let initial = storage.len().unwrap();
        let mut peak = initial;
        for _ in 0..2 {
            assert!(tree_image::grow_dense(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &large
            )
            .unwrap());
            assert_eq!(check_selected(&storage, mode, &authority), large);
            peak = peak.max(storage.len().unwrap());
            let before = storage.read_all().unwrap();
            assert!(tree_image::return_inline(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode)
            )
            .is_err());
            assert_eq!(storage.read_all().unwrap(), before);
            tree_image::replace_metadata(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &original,
            )
            .unwrap();
            peak = peak.max(storage.len().unwrap());
            assert!(tree_image::return_inline(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode)
            )
            .unwrap());
            assert_eq!(check_selected(&storage, mode, &authority), original);
            assert_eq!(storage.len().unwrap(), initial);
            let before = storage.read_all().unwrap();
            assert!(!tree_image::return_inline(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode)
            )
            .unwrap());
            assert_eq!(storage.read_all().unwrap(), before);
        }
        println!("TYPED_TREE_INLINE_TRANSITION mode={bits} initial_bytes={initial} observed_completed_peak={peak} returned_bytes={}", storage.len().unwrap());
    }
}

#[test]
fn typed_tree_inline_transition_interruptions_preserve_selected_filesystem() {
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let original = public_filesystem_metadata(&owner);
    let large = grown(&original);
    let mut cases = 0;
    for bits in [0, 3, 12, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        // Exercise the complete grow/shrink chain, and the two-publication
        // return from a freshly exported tree whose private root is inline.
        for fresh_tree in [false, true] {
            let seed = if fresh_tree {
                exported(mode, &authority, &owner, &original)
            } else {
                inline_seed(mode, &authority, &owner, &original)
            }
            .read_all()
            .unwrap();
            let run = |storage: &mut CrashStore| -> Result<()> {
                if !fresh_tree {
                    tree_image::grow_dense(
                        storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        &large,
                    )?;
                    tree_image::replace_metadata(
                        storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        &original,
                    )?;
                }
                tree_image::return_inline(storage, archive(), mode, &authority, signer, key(mode))?;
                Ok(())
            };
            let mut observed = CrashStore::new(seed.clone(), None, 0, false);
            run(&mut observed).unwrap();
            for at in 0..observed.operations() {
                for prefix in [0, 97, usize::MAX] {
                    for persist in [false, true] {
                        let mut failed = CrashStore::new(seed.clone(), Some(at), prefix, persist);
                        let _ = run(&mut failed);
                        let mut storage = StorageBackend::memory(failed.durable());
                        tree_image::recover(&mut storage, archive(), mode, &authority, key(mode)).unwrap_or_else(|e| panic!("mode={bits} fresh={fresh_tree} at={at} prefix={prefix} persist={persist}: {e}"));
                        tree_image::recover(&mut storage, archive(), mode, &authority, key(mode))
                            .unwrap();
                        let actual = check_selected(&storage, mode, &authority);
                        assert!(actual == original || actual == large);
                        cases += 1;
                    }
                }
            }
        }
    }
    println!("TYPED_TREE_INLINE_TRANSITION_FAULT_CASES {cases}");
}

#[test]
fn typed_tree_automatic_growth_retains_small_dense_metadata_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let original = public_filesystem_metadata(&owner);
    let large = grown(&original);
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = inline_seed(mode, &authority, &owner, &original);
        let mut small = original.clone();
        small
            .iter_mut()
            .find(|entry| entry.target.is_some())
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
            &small
        )
        .unwrap());
        assert!(
            shared::open_private(&storage, archive(), mode, &authority, key(mode))
                .unwrap()
                .1
                .starts_with(b"RV4DENS3")
        );
        assert_eq!(check_selected(&storage, mode, &authority), small);
        let before = storage.read_all().unwrap();
        assert!(!tree_image::replace_metadata(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &small
        )
        .unwrap());
        assert_eq!(storage.read_all().unwrap(), before);
        if mode.signed() {
            let wrong = OwnerSigningKeyPair::generate().unwrap();
            assert!(tree_image::replace_metadata(
                &mut storage,
                archive(),
                mode,
                &authority,
                Some(&wrong),
                key(mode),
                &original
            )
            .is_err());
            assert_eq!(storage.read_all().unwrap(), before);
        }
        assert!(tree_image::replace_metadata(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &large
        )
        .unwrap());
        check(&storage, mode, &authority, &large);
    }
}
#[test]
fn typed_tree_invalid_metadata_refuses_before_writes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(true, true, true, false);
    let authority = authority(mode, &public);
    let entries = public_filesystem_metadata(&owner);
    let seed = exported(mode, &authority, &owner, &entries);
    let mut invalid = vec![Vec::new()];
    let mut duplicate = entries.clone();
    duplicate.push(entries[0].clone());
    invalid.push(duplicate);
    let mut parent = entries.clone();
    parent.retain(|entry| entry.entry.path.as_str() != "/docs");
    invalid.push(parent);
    let mut length = entries.clone();
    length
        .iter_mut()
        .find(|entry| entry.entry.kind == crate::LockboxEntryKind::File)
        .unwrap()
        .entry
        .len += 1;
    invalid.push(length);
    let mut target = entries.clone();
    target
        .iter_mut()
        .find(|entry| entry.target.is_some())
        .unwrap()
        .target = None;
    invalid.push(target);
    for entries in invalid {
        let mut storage = seed.clone();
        let before = storage.read_all().unwrap();
        assert!(tree_image::replace_metadata(
            &mut storage,
            archive(),
            mode,
            &authority,
            Some(&owner),
            key(mode),
            &entries
        )
        .is_err());
        assert_eq!(storage.read_all().unwrap(), before);
    }
    // Reader and writer share the aggregate private-path budget. This check is
    // deliberately before duplicate/path relationship validation or any writes.
    let path =
        crate::LockboxPath::new(format!("/{}", vec!["p".repeat(250); 16].join("/"))).unwrap();
    let repeated = Metadata {
        entry: crate::LockboxEntry {
            path,
            kind: crate::LockboxEntryKind::Directory,
            len: 0,
            permissions: 0o700,
        },
        target: None,
    };
    let oversized = vec![repeated; 8400];
    let mut storage = seed.clone();
    let before = storage.read_all().unwrap();
    assert!(matches!(
        tree_image::replace_metadata(
            &mut storage,
            archive(),
            mode,
            &authority,
            Some(&owner),
            key(mode),
            &oversized
        ),
        Err(Error::SecurityLimitExceeded(_))
    ));
    assert_eq!(storage.read_all().unwrap(), before);
}
