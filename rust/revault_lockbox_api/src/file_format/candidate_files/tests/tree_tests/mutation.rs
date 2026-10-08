use super::*;
mod dense;
fn seed(
    mode: FormatMode,
    authority: &Authority<'_>,
    owner: &OwnerSigningKeyPair,
) -> StorageBackend {
    let signer = mode.signed().then_some(owner);
    let storage = Files::create(
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        authority,
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
    let mut source = Files::open(storage, archive(), mode, authority, key(mode)).unwrap();
    let mut dense = super::super::super::dense_image::from_candidate(
        &mut source,
        StorageBackend::memory(Vec::new()),
        authority,
        signer,
        key(mode),
        &[],
    )
    .unwrap();
    let mut entries = public_filesystem_metadata(owner);
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
    super::super::super::dense_update::replace_filesystem_metadata(
        &mut dense,
        archive(),
        mode,
        authority,
        signer,
        key(mode),
        &entries,
    )
    .unwrap();
    tree_image::from_dense(
        &dense,
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        authority,
        signer,
        key(mode),
    )
    .unwrap()
}
fn inspect(storage: &StorageBackend, mode: FormatMode, authority: &Authority<'_>) -> bool {
    let mut opened = TreeImage::open(
        StorageBackend::memory(storage.read_all().unwrap()),
        archive(),
        mode,
        authority,
        key(mode),
    )
    .unwrap();
    let entries = opened.image.filesystem_metadata().unwrap();
    let present = entries
        .iter()
        .any(|entry| entry.entry.path.as_str() == "/docs/data");
    let mut bytes = Vec::new();
    opened
        .image
        .read_range(b"/docs/neighbor", 0, 8, |part| {
            bytes.extend_from_slice(part);
            Ok(())
        })
        .unwrap();
    assert_eq!(bytes, b"neighbor");
    if present {
        bytes.clear();
        opened
            .image
            .read_range(b"/docs/data", 0, 7, |part| {
                bytes.extend_from_slice(part);
                Ok(())
            })
            .unwrap();
        assert_eq!(bytes, b"payload");
    }
    present
}
fn read_file(image: &mut TreeImage<StorageBackend>, path: &[u8]) -> Vec<u8> {
    let len = image
        .image
        .catalogue
        .files
        .iter()
        .find(|file| file.path.as_slice() == path)
        .unwrap()
        .info
        .len;
    let mut bytes = Vec::new();
    image
        .image
        .read_range(path, 0, len, |part| {
            bytes.extend_from_slice(part);
            Ok(())
        })
        .unwrap();
    bytes
}
fn inputs(data: &[u8]) -> Vec<Input<Cursor<Vec<u8>>>> {
    vec![
        Input {
            path: b"/docs/data".to_vec(),
            reader: Cursor::new(data.to_vec()),
        },
        Input {
            path: b"/docs/new".to_vec(),
            reader: Cursor::new(b"addition".to_vec()),
        },
        Input {
            path: b"/docs/zero".to_vec(),
            reader: Cursor::new(Vec::new()),
        },
    ]
}
#[test]
fn typed_tree_payload_add_replace_no_change_and_invalid_plans_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let bytes: Vec<u8> = (0..140000).map(|i| (i % 251) as u8).collect();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = seed(mode, &authority, &owner);
        let opened =
            TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
        let initial = storage.len().unwrap();
        let old = opened.image.catalogue.packs[0].extent;
        let id = opened.image.catalogue.files[0].info.id;
        let before = storage.read_all().unwrap();
        for attempt in 0..3 {
            let mut retired = vec![old];
            let mut base = shared::commitment(&opened.tree.anchor).unwrap();
            match attempt {
                0 => base[0] ^= 1,
                1 => retired.push(old),
                _ => retired[0].start = opened.tree.anchor.index.primary,
            }
            let plan = shared::tree::PayloadPlan {
                base,
                retired,
                bytes: Vec::new(),
                rebind: Box::new(|_| Ok(Vec::new())),
            };
            assert!(shared::tree::rewrite_payload_records(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                Vec::new(),
                plan
            )
            .is_err());
            assert_eq!(storage.read_all().unwrap(), before);
        }
        let invalid = vec![Input {
            path: b"/missing/child".to_vec(),
            reader: Cursor::new(b"invalid parent".to_vec()),
        }];
        assert!(tree_image::update_files(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            invalid,
            &[]
        )
        .is_err());
        assert_eq!(storage.read_all().unwrap(), before);
        assert!(tree_image::update_files(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            inputs(&bytes),
            &[]
        )
        .unwrap());
        let mut reopened = TreeImage::open(
            StorageBackend::memory(storage.read_all().unwrap()),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        assert_eq!(reopened.image.catalogue.files[0].info.id, id);
        assert_eq!(read_file(&mut reopened, b"/docs/data"), bytes);
        assert_eq!(read_file(&mut reopened, b"/docs/new"), b"addition");
        assert_eq!(read_file(&mut reopened, b"/docs/neighbor"), b"neighbor");
        assert!(read_file(&mut reopened, b"/docs/zero").is_empty());
        assert!(storage
            .read_at(old.start, old.len as usize)
            .unwrap()
            .iter()
            .all(|byte| *byte == 0));
        let before = storage.read_all().unwrap();
        assert!(!tree_image::update_files(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            inputs(&bytes),
            &[]
        )
        .unwrap());
        assert_eq!(storage.read_all().unwrap(), before);
        for bank in [0, FAILURE_REGION] {
            let mut lost = storage.clone();
            lost.write_at(bank, &vec![0; FAILURE_REGION as usize])
                .unwrap();
            tree_image::recover(&mut lost, archive(), mode, &authority, key(mode)).unwrap();
            let mut reopened =
                TreeImage::open(lost, archive(), mode, &authority, key(mode)).unwrap();
            assert_eq!(read_file(&mut reopened, b"/docs/data"), bytes);
            assert_eq!(read_file(&mut reopened, b"/docs/neighbor"), b"neighbor");
        }
        println!("TYPED_TREE_PAYLOAD_UPDATE mode={bits} initial_bytes={initial} completed_bytes={} changed_logical_bytes={}", storage.len().unwrap(), bytes.len() + 8);
    }
}

#[test]
fn typed_tree_payload_source_change_is_non_destructive() {
    struct Changing(Cursor<Vec<u8>>);
    impl std::io::Read for Changing {
        fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
            self.0.read(out)
        }
    }
    impl std::io::Seek for Changing {
        fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
            if matches!(pos, std::io::SeekFrom::Start(_)) {
                self.0.get_mut()[0] ^= 1;
            }
            self.0.seek(pos)
        }
    }
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(true, true, true, true);
    let authority = authority(mode, &public);
    let mut storage = seed(mode, &authority, &owner);
    let before = storage.read_all().unwrap();
    let input = Input {
        path: b"/docs/data".to_vec(),
        reader: Changing(Cursor::new(b"changed source".to_vec())),
    };
    assert!(tree_image::update_files(
        &mut storage,
        archive(),
        mode,
        &authority,
        Some(&owner),
        key(mode),
        vec![input],
        &[]
    )
    .is_err());
    assert_eq!(storage.read_all().unwrap(), before);
}

#[test]
fn typed_tree_payload_update_interruptions_keep_atomic_file_set() {
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for bits in [0, 3, 12, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let seed = seed(mode, &authority, &owner).read_all().unwrap();
        let run = |storage: &mut CrashStore| {
            tree_image::update_files(
                storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                inputs(b"replacement"),
                &[],
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
                    tree_image::recover(&mut reopened, archive(), mode, &authority, key(mode))
                        .unwrap_or_else(|e| {
                            panic!("mode={bits} at={at} prefix={prefix} persist={persist}: {e}")
                        });
                    let mut image = TreeImage::open(
                        StorageBackend::memory(reopened.read_all().unwrap()),
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap();
                    let new = image.image.catalogue.files.len() == 4;
                    assert_eq!(image.image.catalogue.files.len(), if new { 4 } else { 2 });
                    assert_eq!(
                        read_file(&mut image, b"/docs/data"),
                        if new {
                            b"replacement".as_slice()
                        } else {
                            b"payload".as_slice()
                        }
                    );
                    assert_eq!(read_file(&mut image, b"/docs/neighbor"), b"neighbor");
                    if new {
                        assert_eq!(read_file(&mut image, b"/docs/new"), b"addition");
                        assert!(read_file(&mut image, b"/docs/zero").is_empty());
                    }
                    tree_image::recover(&mut reopened, archive(), mode, &authority, key(mode))
                        .unwrap();
                    cases += 1;
                }
            }
        }
    }
    println!("TYPED_TREE_PAYLOAD_UPDATE_FAULTS {cases}");
}

#[test]
fn typed_tree_payload_reuse_stabilizes_bounded_lifecycle_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = seed(mode, &authority, &owner);
        let initial = storage.len().unwrap();
        let mut stable = 0;
        let mut reused = 0;
        for cycle in 0..32 {
            let data = vec![
                if cycle % 2 == 0 { 13 } else { 29 };
                if cycle % 3 == 0 { 70000 } else { 2000 }
            ];
            let prior = storage.len().unwrap();
            let old = TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
                .unwrap()
                .tree
                .graph
                .payloads();
            tree_image::update_files(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                inputs(&data),
                &[],
            )
            .unwrap();
            let mut image = TreeImage::open(
                StorageBackend::memory(storage.read_all().unwrap()),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap();
            assert_eq!(read_file(&mut image, b"/docs/data"), data);
            assert_eq!(read_file(&mut image, b"/docs/neighbor"), b"neighbor");
            reused += image
                .tree
                .graph
                .payloads()
                .iter()
                .filter(|extent| extent.start < prior && !old.contains(extent))
                .count();
            if cycle % 2 == 0 {
                tree_image::remove_files(
                    &mut storage,
                    archive(),
                    mode,
                    &authority,
                    signer,
                    key(mode),
                    &[b"/docs/new".to_vec(), b"/docs/zero".to_vec()],
                )
                .unwrap();
            }
            if cycle == 15 {
                stable = storage.len().unwrap();
            }
            if cycle > 15 {
                assert_eq!(storage.len().unwrap(), stable, "mode={bits} cycle={cycle}");
            }
        }
        assert!(reused > 0, "mode={bits}");
        println!("TYPED_TREE_PAYLOAD_REUSE mode={bits} cycles=32 initial_bytes={initial} stable_bytes={stable} reused_pack_count={reused}");
    }
}

#[test]
fn typed_tree_payload_reused_writes_recover_without_neighbor_loss() {
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for bits in [0, 3, 12, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = seed(mode, &authority, &owner);
        for cycle in 0..6 {
            tree_image::update_files(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                inputs(&[cycle; 2000]),
                &[],
            )
            .unwrap();
        }
        let old = TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
            .unwrap()
            .tree
            .graph
            .payloads();
        let prior = storage.len().unwrap();
        let seed = storage.read_all().unwrap();
        let run = |storage: &mut CrashStore| {
            tree_image::update_files(
                storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                inputs(&[11; 2000]),
                &[],
            )
        };
        let mut observed = CrashStore::new(seed.clone(), None, 0, false);
        run(&mut observed).unwrap();
        let complete = StorageBackend::memory(observed.durable());
        let selected = TreeImage::open(complete, archive(), mode, &authority, key(mode)).unwrap();
        assert!(selected
            .tree
            .graph
            .payloads()
            .iter()
            .any(|extent| extent.start < prior && !old.contains(extent)));
        for at in 0..observed.operations() {
            for prefix in [0, 97, usize::MAX] {
                for persist in [false, true] {
                    let mut failed = CrashStore::new(seed.clone(), Some(at), prefix, persist);
                    let _ = run(&mut failed);
                    let mut reopened = StorageBackend::memory(failed.durable());
                    tree_image::recover(&mut reopened, archive(), mode, &authority, key(mode))
                        .unwrap_or_else(|e| {
                            panic!("mode={bits} at={at} prefix={prefix} persist={persist}: {e}")
                        });
                    let mut image = TreeImage::open(
                        StorageBackend::memory(reopened.read_all().unwrap()),
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap();
                    let bytes = read_file(&mut image, b"/docs/data");
                    assert!(bytes == vec![5; 2000] || bytes == vec![11; 2000]);
                    assert_eq!(read_file(&mut image, b"/docs/neighbor"), b"neighbor");
                    tree_image::recover(&mut reopened, archive(), mode, &authority, key(mode))
                        .unwrap();
                    cases += 1;
                }
            }
        }
    }
    println!("TYPED_TREE_REUSED_PAYLOAD_FAULTS {cases}");
}

#[test]
fn typed_tree_payload_updates_preserve_large_typed_catalogues_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = seed(mode, &authority, &owner);
        let metadata = TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
            .unwrap()
            .image
            .filesystem_metadata()
            .unwrap();
        let large = grown(&metadata);
        tree_image::replace_metadata(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &large,
        )
        .unwrap();
        tree_image::update_files(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            inputs(b"replacement"),
            &[],
        )
        .unwrap();
        let mut image = TreeImage::open(
            StorageBackend::memory(storage.read_all().unwrap()),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        assert_eq!(
            image.image.filesystem_metadata().unwrap().len(),
            large.len() + 2
        );
        assert_eq!(read_file(&mut image, b"/docs/data"), b"replacement");
        assert_eq!(read_file(&mut image, b"/docs/neighbor"), b"neighbor");
        tree_image::remove_files(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &[b"/docs/new".to_vec(), b"/docs/zero".to_vec()],
        )
        .unwrap();
        assert_eq!(
            TreeImage::open(storage, archive(), mode, &authority, key(mode))
                .unwrap()
                .image
                .filesystem_metadata()
                .unwrap()
                .len(),
            large.len()
        );
    }
}

#[test]
fn typed_tree_payload_overflow_preparation_recovers_sampled_interruptions() {
    use crate::file_format::preparation_journal::compact::session::OverflowSession;
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    let mut linked = 0;
    for (bits, inline) in [(2, false), (11, false), (2, true), (11, true)] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = if inline {
            dense::dense_seed(mode, &authority, &owner)
        } else {
            seed(mode, &authority, &owner)
        };
        let payload = |byte| vec![byte; 12 * 1024 * 1024];
        for byte in 0..4 {
            tree_image::update_files(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                inputs(&payload(byte)),
                &[],
            )
            .unwrap();
        }
        let seed = storage.read_all().unwrap();
        let run = |storage: &mut CrashStore| {
            tree_image::update_files(
                storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                inputs(&payload(19)),
                &[],
            )
        };
        let mut observed = CrashStore::new(seed.clone(), None, 0, false);
        run(&mut observed).unwrap();
        let count = observed.operations();
        // Bounded samples span staging through cleanup; the small reused-write
        // fixture exhausts every operation separately. Do not call this exhaustive.
        let points: std::collections::BTreeSet<_> = (0..8)
            .map(|i| i * count / 8)
            .chain(count.saturating_sub(4)..count)
            .collect();
        let mut mode_linked = 0;
        for at in points {
            for prefix in [0, 97] {
                for persist in [false, true] {
                    let mut failed = CrashStore::new(seed.clone(), Some(at), prefix, persist);
                    let _ = run(&mut failed);
                    let mut reopened = StorageBackend::memory(failed.durable());
                    if OverflowSession::open(&reopened, archive(), mode, key(mode))
                        .is_ok_and(|session| session.arena().is_some())
                    {
                        mode_linked += 1;
                    }
                    tree_image::recover(&mut reopened, archive(), mode, &authority, key(mode))
                        .unwrap_or_else(|e| {
                            panic!("mode={bits} at={at} prefix={prefix} persist={persist}: {e}")
                        });
                    let (_, bytes, _) = dense::selected(&reopened, mode, &authority);
                    assert!(bytes == payload(3) || bytes == payload(19));
                    tree_image::recover(&mut reopened, archive(), mode, &authority, key(mode))
                        .unwrap();
                    cases += 1;
                }
            }
        }
        assert!(
            mode_linked > 0,
            "fixture must exercise a linked overflow arena: mode={bits} inline={inline}"
        );
        linked += mode_linked;
    }
    println!(
        "TYPED_TREE_PAYLOAD_OVERFLOW_SAMPLED_FAULTS {cases} linked_arena_observations={linked}"
    );
}
#[test]
fn typed_tree_payload_removal_preserves_neighbors_and_erases_old_pack_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let wrong = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = seed(mode, &authority, &owner);
        let before = storage.read_all().unwrap();
        for paths in [
            vec![b"/docs/data/../data".to_vec()],
            vec![b"/docs/data".to_vec(), b"/docs/data".to_vec()],
        ] {
            assert!(tree_image::remove_files(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &paths
            )
            .is_err());
            assert_eq!(storage.read_all().unwrap(), before);
        }
        let paths = vec![b"/docs/data".to_vec()];
        if mode.signed() {
            assert!(tree_image::remove_files(
                &mut storage,
                archive(),
                mode,
                &authority,
                Some(&wrong),
                key(mode),
                &paths
            )
            .is_err());
            assert_eq!(storage.read_all().unwrap(), before);
        }
        let opened =
            TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
        assert_eq!(opened.image.catalogue.packs.len(), 1);
        let old = opened.image.catalogue.packs[0].extent;
        assert!(tree_image::remove_files(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &paths
        )
        .unwrap());
        assert!(!inspect(&storage, mode, &authority));
        assert!(storage
            .read_at(old.start, old.len as usize)
            .unwrap()
            .iter()
            .all(|byte| *byte == 0));
        let before = storage.read_all().unwrap();
        assert!(!tree_image::remove_files(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &paths
        )
        .unwrap());
        assert_eq!(storage.read_all().unwrap(), before);
        for bank in [0, FAILURE_REGION] {
            let mut lost = storage.clone();
            lost.write_at(bank, &vec![0; FAILURE_REGION as usize])
                .unwrap();
            tree_image::recover(&mut lost, archive(), mode, &authority, key(mode)).unwrap();
            assert!(!inspect(&lost, mode, &authority));
        }
        assert!(tree_image::remove_files(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &[b"/docs/neighbor".to_vec()]
        )
        .unwrap());
        let opened = TreeImage::open(storage, archive(), mode, &authority, key(mode)).unwrap();
        assert!(opened.image.catalogue.files.is_empty());
        assert!(opened.tree.graph.payloads().is_empty());
    }
}
#[test]
fn typed_tree_payload_removal_interruptions_keep_selected_membership() {
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for bits in [0, 3, 12, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let seed = seed(mode, &authority, &owner).read_all().unwrap();
        let run = |storage: &mut CrashStore| {
            tree_image::remove_files(
                storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                &[b"/docs/data".to_vec()],
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
                    tree_image::recover(&mut reopened, archive(), mode, &authority, key(mode))
                        .unwrap_or_else(|e| {
                            panic!("mode={bits} at={at} prefix={prefix} persist={persist}: {e}")
                        });
                    let selected = inspect(&reopened, mode, &authority);
                    tree_image::recover(&mut reopened, archive(), mode, &authority, key(mode))
                        .unwrap();
                    assert_eq!(inspect(&reopened, mode, &authority), selected);
                    cases += 1;
                }
            }
        }
    }
    println!("TYPED_TREE_PAYLOAD_REMOVAL_FAULTS {cases}");
}
