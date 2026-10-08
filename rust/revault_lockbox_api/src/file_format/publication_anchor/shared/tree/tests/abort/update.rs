use super::*;

fn replacement(count: u32, value: u8) -> Vec<Entry> {
    (0..count)
        .map(|i| Entry::new(1, &i.to_be_bytes(), &[value; 128]).unwrap())
        .collect()
}
fn reopened(bytes: Vec<u8>, mode: FormatMode, authority: &Authority<'_>) -> Vec<Vec<u8>> {
    let storage = StorageBackend::memory(bytes);
    let tree = Tree::open(&storage, archive(), mode, authority, key(mode)).unwrap();
    let mut values = Vec::new();
    tree.visit(&storage, |entry| {
        assert_eq!(entry.key.as_slice(), (values.len() as u32).to_be_bytes());
        values.push(entry.value.to_vec());
        Ok(())
    })
    .unwrap();
    assert_eq!(
        storage.read_at(PAYLOAD, FAILURE_REGION as usize).unwrap(),
        vec![37; FAILURE_REGION as usize]
    );
    values
}

#[test]
fn authenticated_tree_update_lifecycle_and_no_change_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (seed, _, _) = fixture(mode, &owner, false);
        let mut storage = StorageBackend::memory(seed);
        for (count, value) in [(16, 42), (2048, 43), (1024, 44), (2, 45), (0, 46)] {
            assert!(rewrite_records(
                &mut storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                replacement(count, value)
            )
            .unwrap_or_else(|error| panic!("mode={bits} count={count}: {error}")));
            let bytes = storage.read_all().unwrap();
            assert_eq!(
                reopened(bytes.clone(), mode, &authority),
                vec![vec![value; 128]; count as usize]
            );
            assert!(!rewrite_records(
                &mut storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                replacement(count, value)
            )
            .unwrap());
            assert_eq!(storage.read_all().unwrap(), bytes);
            recover_update(&mut storage, archive(), mode, &authority, key(mode)).unwrap();
        }
    }
}

#[test]
fn authenticated_tree_update_interruptions_select_only_old_or_new_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (seed, _, _) = fixture(mode, &owner, false);
        let old = reopened(seed.clone(), mode, &authority);
        let new = vec![vec![42; 128]; 16];
        let run = |storage: &mut CrashStore| {
            rewrite_records(
                storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                replacement(16, 42),
            )
        };
        let mut observed = CrashStore::new(seed.clone(), None, 0, false);
        run(&mut observed).unwrap();
        for at in 0..observed.operations() {
            for prefix in [0, 97, usize::MAX] {
                for persist in [false, true] {
                    let mut failed = CrashStore::new(seed.clone(), Some(at), prefix, persist);
                    let _ = run(&mut failed);
                    let mut storage = StorageBackend::memory(failed.durable());
                    recover_update(&mut storage, archive(), mode, &authority, key(mode))
                        .unwrap_or_else(|error| {
                            panic!("mode={bits} at={at} prefix={prefix} persist={persist}: {error}")
                        });
                    recover_update(&mut storage, archive(), mode, &authority, key(mode)).unwrap();
                    let values = reopened(storage.read_all().unwrap(), mode, &authority);
                    assert!(values == old || values == new);
                    cases += 1;
                }
            }
        }
    }
    println!("AUTHENTICATED_TREE_UPDATE_POWER_LOSS_CASES {cases}");
}

#[test]
fn authenticated_tree_update_refuses_malformed_records_without_mutation() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, true, true, false);
    let authority = authority(mode, &public);
    let (seed, _, _) = fixture(mode, &owner, false);
    for records in [
        vec![Entry::new(0, b"ownership", b"forged").unwrap()],
        vec![
            Entry::new(1, b"same", b"a").unwrap(),
            Entry::new(1, b"same", b"b").unwrap(),
        ],
        vec![
            Entry::new(1, b"z", b"a").unwrap(),
            Entry::new(1, b"a", b"b").unwrap(),
        ],
    ] {
        let mut guarded = ReadOnly(StorageBackend::memory(seed.clone()));
        assert!(rewrite_records(
            &mut guarded,
            archive(),
            mode,
            &authority,
            Some(&owner),
            None,
            records
        )
        .is_err());
    }
}

#[test]
fn authenticated_tree_update_reuse_stabilizes_metadata_cycles_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (seed, _, _) = fixture(mode, &owner, false);
        let mut storage = StorageBackend::memory(seed);
        let initial = storage.len().unwrap();
        let mut stable = 0;
        for cycle in 0..40 {
            rewrite_records(
                &mut storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                replacement(128, cycle),
            )
            .unwrap_or_else(|error| panic!("mode={bits} cycle={cycle}: {error}"));
            assert_eq!(
                reopened(storage.read_all().unwrap(), mode, &authority),
                vec![vec![cycle; 128]; 128]
            );
            if cycle == 3 {
                stable = storage.len().unwrap();
            }
            if cycle > 3 {
                assert_eq!(storage.len().unwrap(), stable, "mode={bits} cycle={cycle}");
            }
        }
        println!("TREE_REUSE_AGING mode={bits} initial_bytes={initial} stabilized_bytes={stable} cycles=40");
    }
}

#[test]
fn authenticated_tree_update_uses_overflow_for_large_reuse_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let mut storage = Aligned(StorageBackend::memory(vec![0; REGION_LEN]));
        let payload = vec![37; FAILURE_REGION as usize];
        storage.append(&payload).unwrap();
        storage
            .append(&vec![0; 160 * FAILURE_REGION as usize])
            .unwrap();
        let name = |kind: u8, start: u64| [vec![kind], start.to_be_bytes().to_vec()].concat();
        let rows = vec![
            Entry::new(
                0,
                &name(0, FREE_START),
                &(160 * FAILURE_REGION).to_le_bytes(),
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
        let change = Index::new(archive(), mode, key(mode))
            .unwrap()
            .build_sorted(&mut storage, rows)
            .unwrap();
        initialize(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &manifest(change.root),
            &[],
        )
        .unwrap();
        let initial = storage.len().unwrap();
        if [4, 15].contains(&bits) {
            let seed = storage.0.read_all().unwrap();
            let run = |storage: &mut CrashStore| {
                let records = (0..6144u32)
                    .map(|i| Entry::new(1, &i.to_be_bytes(), &[42; 1024]).unwrap())
                    .collect();
                rewrite_records(
                    storage,
                    archive(),
                    mode,
                    &authority,
                    mode.signed().then_some(&owner),
                    key(mode),
                    records,
                )
            };
            let mut observed = CrashStore::new(seed.clone(), None, 0, false);
            run(&mut observed).unwrap();
            // Exhaust the final cleanup operations with a real large-reservation
            // writer; full staging/abort tears are covered by the small fixture.
            for at in observed.operations().saturating_sub(20)..observed.operations() {
                for prefix in [0, 97, usize::MAX] {
                    for persist in [false, true] {
                        let mut failed = CrashStore::new(seed.clone(), Some(at), prefix, persist);
                        let _ = run(&mut failed);
                        let mut reopened_store = StorageBackend::memory(failed.durable());
                        recover_update(&mut reopened_store, archive(), mode, &authority, key(mode)).unwrap_or_else(|error| panic!("large mode={bits} at={at} prefix={prefix} persist={persist}: {error}"));
                        let values = reopened(reopened_store.read_all().unwrap(), mode, &authority);
                        assert!(values.is_empty() || values == vec![vec![42; 1024]; 6144]);
                    }
                }
            }
        }
        let records = (0..6144u32)
            .map(|i| Entry::new(1, &i.to_be_bytes(), &[42; 1024]).unwrap())
            .collect();
        rewrite_records(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            records,
        )
        .unwrap();
        assert_eq!(
            reopened(storage.0.read_all().unwrap(), mode, &authority),
            vec![vec![42; 1024]; 6144]
        );
        let tree = Tree::open(&storage, archive(), mode, &authority, key(mode)).unwrap();
        // The two reused arena copies were actually allocated and retired by
        // the connected writer, not manufactured by the test recovery helper.
        for start in [FREE_START, FREE_START + FAILURE_REGION] {
            assert!(tree.graph.pending().contains(&Span {
                start,
                len: FAILURE_REGION
            }));
            assert!(storage
                .read_at(start, FAILURE_REGION as usize)
                .unwrap()
                .iter()
                .all(|byte| *byte == 0));
        }
        println!(
            "TREE_OVERFLOW_REUSE mode={bits} initial_bytes={initial} completed_bytes={}",
            storage.len().unwrap()
        );
        let mut stable = 0;
        for cycle in 0..8u8 {
            let value = cycle + 43;
            let records = (0..6144u32)
                .map(|i| Entry::new(1, &i.to_be_bytes(), &[value; 1024]).unwrap())
                .collect();
            rewrite_records(
                &mut storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                records,
            )
            .unwrap();
            assert_eq!(
                reopened(storage.0.read_all().unwrap(), mode, &authority),
                vec![vec![value; 1024]; 6144]
            );
            if cycle == 3 {
                stable = storage.len().unwrap();
            }
            if cycle > 3 {
                assert_eq!(
                    storage.len().unwrap(),
                    stable,
                    "mode={bits} overflow cycle={cycle}"
                );
            }
        }
        println!("TREE_OVERFLOW_AGING mode={bits} stabilized_bytes={stable} additional_cycles=8");
    }
}

#[test]
fn authenticated_tree_update_reused_arena_abort_interruptions_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let mut storage = Aligned(StorageBackend::memory(vec![0; REGION_LEN]));
        let payload = vec![37; FAILURE_REGION as usize];
        storage.append(&payload).unwrap();
        storage
            .append(&vec![0; 3 * FAILURE_REGION as usize])
            .unwrap();
        let name = |kind: u8, start: u64| [vec![kind], start.to_be_bytes().to_vec()].concat();
        let mut rows = vec![
            Entry::new(0, &name(0, FREE_START), &(3 * FAILURE_REGION).to_le_bytes()),
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
        let anchor = initialize(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &manifest(change.root),
            &[],
        )
        .unwrap();
        let seed = storage.0.read_all().unwrap();
        let arena = [
            Reservation {
                namespace: FREE,
                base: FREE_START,
                start: FREE_START,
                len: FAILURE_REGION,
            },
            Reservation {
                namespace: FREE,
                base: FREE_START,
                start: FREE_START + FAILURE_REGION,
                len: FAILURE_REGION,
            },
        ];
        let mut reservations = arena.to_vec();
        reservations.extend((0..156).map(|i| Reservation {
            namespace: FREE,
            base: FREE_START,
            start: FREE_START + 2 * FAILURE_REGION + i * 8,
            len: 8,
        }));
        let run = |storage: &mut CrashStore| -> Result<()> {
            OverflowSession::open(storage, archive(), mode, key(mode))?.begin_reused(
                storage,
                commitment(&anchor)?,
                anchor.sealed_len,
                arena,
                reservations.clone(),
            )?;
            storage.write_at(FREE_START + 2 * FAILURE_REGION, &vec![91; 156 * 8])?;
            storage.sync()?;
            recover_update(storage, archive(), mode, &authority, key(mode))
        };
        let mut observed = CrashStore::new(seed.clone(), None, 0, false);
        run(&mut observed).unwrap();
        let mut linked = StorageBackend::memory(seed.clone());
        OverflowSession::open(&linked, archive(), mode, key(mode))
            .unwrap()
            .begin_reused(
                &mut linked,
                commitment(&anchor).unwrap(),
                anchor.sealed_len,
                arena,
                reservations.clone(),
            )
            .unwrap();
        for at in [0, FAILURE_REGION, arena[0].start, arena[1].start] {
            let mut damaged = StorageBackend::memory(linked.read_all().unwrap());
            damaged
                .write_at(at, &vec![0; FAILURE_REGION as usize])
                .unwrap();
            recover_update(&mut damaged, archive(), mode, &authority, key(mode)).unwrap();
            check(
                damaged.read_all().unwrap(),
                mode,
                &authority,
                anchor.sealed_len,
            );
        }
        for at in 0..observed.operations() {
            for prefix in [0, 97, usize::MAX] {
                for persist in [false, true] {
                    let mut failed = CrashStore::new(seed.clone(), Some(at), prefix, persist);
                    let _ = run(&mut failed);
                    let mut reopened = StorageBackend::memory(failed.durable());
                    recover_update(&mut reopened, archive(), mode, &authority, key(mode))
                        .unwrap_or_else(|error| {
                            panic!("mode={bits} at={at} prefix={prefix} persist={persist}: {error}")
                        });
                    recover_update(&mut reopened, archive(), mode, &authority, key(mode)).unwrap();
                    check(
                        reopened.read_all().unwrap(),
                        mode,
                        &authority,
                        anchor.sealed_len,
                    );
                    cases += 1;
                }
            }
        }
    }
    println!("REUSED_ARENA_ABORT_POWER_LOSS_CASES {cases}");
}
