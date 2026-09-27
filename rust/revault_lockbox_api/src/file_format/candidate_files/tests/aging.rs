//! Long-running candidate qualification; no public CLI writes candidate C yet.
use super::*;
type Added = Vec<(&'static [u8], Vec<u8>)>;

#[test]
#[ignore = "explicit 4-mode, 1000-cycle mixed file aging qualification"]
fn mixed_file_aging_preserves_contents_and_bounds_physical_growth() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut unstabilized = Vec::new();
    for (number, mode) in [
        mode(false, false, false, true),
        mode(true, true, true, true),
        mode(false, true, true, true),
        mode(true, false, false, false),
    ]
    .into_iter()
    .enumerate()
    {
        if let Ok(selected) = std::env::var("REVAULT_CANDIDATE_AGING_MODE") {
            if selected.parse::<usize>().unwrap() != number {
                continue;
            }
        }
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = Files::create(
            StorageBackend::memory(Vec::new()),
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            MAX_LOGICAL,
            update_inputs(&[(b"keep", &vec![0x39; 8192])]),
        )
        .unwrap();
        let mut expected = BTreeMap::from([(b"keep".to_vec(), vec![0x39; 8192])]);
        let mut high_water = 0;
        let mut no_change = 0;
        let mut observed_high_water = 0;
        for cycle in 0..1000 {
            if std::env::var_os("REVAULT_CANDIDATE_TRACE_ARENA").is_some() {
                eprintln!("CANDIDATE_AGING_OPERATION mode={number} cycle={cycle}");
            }
            let phase = cycle % 10;
            let bytes = |len: usize| -> Vec<u8> {
                (0..len)
                    .map(|offset| ((offset * 13 + offset / 251) % 251) as u8 ^ (cycle / 10) as u8)
                    .collect()
            };
            let (added, removed): (Added, Vec<Vec<u8>>) = match phase {
                0 => (
                    vec![(b"a", bytes(4096)), (b"b", bytes(131072)), (b"c", vec![])],
                    vec![],
                ),
                1 => (vec![(b"b", bytes(400000))], vec![]),
                2 => (vec![], vec![b"a".to_vec()]),
                3 => (vec![(b"b", bytes(17))], vec![]),
                4 => (vec![(b"c", bytes(8192))], vec![]),
                5 => (vec![], vec![b"b".to_vec()]),
                6 => (
                    vec![(b"c", expected[b"c".as_slice()].clone())],
                    vec![b"absent".to_vec()],
                ),
                7 => (vec![(b"d", bytes(65536))], vec![]),
                8 => (
                    vec![(b"e", bytes(20000))],
                    vec![b"c".to_vec(), b"d".to_vec()],
                ),
                _ => (vec![], vec![b"e".to_vec(), b"d".to_vec()]),
            };
            let unchanged = phase == 6;
            let before = unchanged.then(|| storage.read_all().unwrap());
            let inputs = added
                .iter()
                .map(|(name, content)| Input {
                    path: name.to_vec(),
                    reader: Cursor::new(content.as_slice()),
                })
                .collect();
            storage = Files::update(
                storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                MAX_LOGICAL,
                inputs,
                removed.clone(),
            )
            .unwrap();
            if let Some(before) = before {
                assert_eq!(
                    storage.read_all().unwrap(),
                    before,
                    "mode {number}, cycle {cycle}: no-change altered physical bytes"
                );
                no_change += 1;
            }
            for path in removed {
                expected.remove(&path);
            }
            for (path, content) in added {
                expected.insert(path.to_vec(), content);
            }
            // A fresh memory backend prevents relying on the prior handle/index.
            let mut reopened = Files::open(
                StorageBackend::memory(storage.read_all().unwrap()),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap();
            for path in [b"keep".as_slice(), b"a", b"b", b"c", b"d", b"e", b"absent"] {
                assert_file(&mut reopened, path, expected.get(path).map(Vec::as_slice));
            }
            let snapshot =
                Snapshot::inspect(&reopened.storage, &reopened.anchor, &reopened.index).unwrap();
            snapshot.verify_reclaimed(&reopened.storage).unwrap();
            let ranges = snapshot.reusable_ranges();
            let largest_reusable = ranges.iter().map(|r| r.len).max().unwrap_or(0);
            let largest_aligned = ranges
                .iter()
                .map(|r| {
                    let aligned = r.start.div_ceil(65536) * 65536;
                    (r.start + r.len).saturating_sub(aligned)
                })
                .max()
                .unwrap_or(0);
            let range_count = ranges.len();
            let a = snapshot.accounting;
            // The immutable committed map retains PENDING labels for retired
            // ranges even after journal cleanup has zeroed them. The byte audit
            // above covers both FREE and PENDING; a zero label count is not the
            // erasure contract and would require an unnecessary second commit.
            assert_eq!(a.total, reopened.storage.len().unwrap());
            assert_eq!(
                a.fixed
                    + a.payload
                    + a.index
                    + a.keys
                    + a.allocation
                    + a.reserve
                    + a.free
                    + a.pending,
                a.total
            );
            if cycle < 100 {
                high_water = high_water.max(a.total);
            } else if a.total > observed_high_water {
                unstabilized.push((number, cycle, high_water, a.total));
                println!(
                    "CANDIDATE_AGING_GROWTH {}",
                    serde_json::json!({
                        "mode":number,"cycle":cycle,"total":a.total,"initial_high_water":high_water,
                        "previous_high_water":observed_high_water,"accounting":format!("{a:?}"),
                        "reusable_ranges":range_count,"largest_reusable":largest_reusable,
                        "largest_aligned_reusable":largest_aligned
                    })
                );
            }
            observed_high_water = observed_high_water.max(a.total);
            if cycle % 100 == 99 {
                let mut stored_fragments = 0;
                reopened
                    .index
                    .visit(
                        &reopened.storage,
                        reopened.anchor.index,
                        reopened.anchor.sealed_len,
                        |entry| {
                            if entry.namespace == CHUNK {
                                let record = OwnedRecord::decode(&entry.value)?;
                                stored_fragments += Slice::decode(&record.metadata)?
                                    .physical(record.extents[0])?
                                    .len;
                            }
                            Ok(())
                        },
                    )
                    .unwrap();
                println!(
                    "CANDIDATE_MIXED_AGING {}",
                    serde_json::json!({
                        "mode":number,"cycles":cycle+1,"no_change_cycles":no_change,
                        "live_files":expected.len(),"logical_bytes":expected.values().map(Vec::len).sum::<usize>(),
                        "total":a.total,"fixed":a.fixed,"payload":a.payload,"stored_fragments":stored_fragments,
                        "pack_padding":a.payload-stored_fragments,"index":a.index,"keys":a.keys,
                        "allocation":a.allocation,"reserve":a.reserve,"free":a.free,"pending":a.pending,
                        "first_100_cycle_high_water":high_water
                    })
                );
            }
        }
    }
    assert!(
        unstabilized.is_empty(),
        "growth after the first 100 cycles: {unstabilized:?}"
    );
}
