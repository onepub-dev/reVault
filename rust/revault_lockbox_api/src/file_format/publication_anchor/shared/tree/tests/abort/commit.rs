//! Real authenticated COW graph fixtures; there is no public CLI for this profile.
//! Publication is stopped after one bank to exercise roll-forward, never rollback.
use super::*;

struct Published {
    bytes: Vec<u8>,
    initial_len: u64,
    anchor: Anchor,
    pending: Vec<Span>,
    pages: Vec<RootRef>,
    arena: RootRef,
    publication: Vec<u8>,
}

fn published(mode: FormatMode, owner: &OwnerSigningKeyPair) -> Published {
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let (seed, reservations, old_pages) = fixture(mode, owner, false);
    let initial_len = seed.len() as u64;
    let mut storage = Aligned(StorageBackend::memory(seed));
    let old = Tree::open(&storage, archive(), mode, &authority, key(mode)).unwrap();
    stage(&mut storage, mode, &authority, reservations).unwrap();
    let arena = OverflowSession::open(&storage, archive(), mode, key(mode))
        .unwrap()
        .arena()
        .unwrap();
    let payload = vec![55; FAILURE_REGION as usize];
    storage.write_at(FREE_START, &payload).unwrap();
    // Allocate the next private slots before building ownership records; the
    // complete index follows them and its page claims come from traversal.
    let primary = storage.append(&vec![0; FAILURE_REGION as usize]).unwrap();
    let mirror = storage.append(&vec![0; FAILURE_REGION as usize]).unwrap();
    let mut pending = vec![
        Span {
            start: PRIVATE_START,
            len: PRIVATE_BYTES as u64,
        },
        Span {
            start: FAILURE_REGION + PRIVATE_START,
            len: PRIVATE_BYTES as u64,
        },
        Span {
            start: arena.primary,
            len: arena.len,
        },
        Span {
            start: arena.mirror,
            len: arena.len,
        },
    ];
    for page in old_pages {
        for start in [page.primary, page.mirror] {
            pending.push(Span {
                start,
                len: FAILURE_REGION,
            });
        }
    }
    let mut rows = Vec::new();
    let name = |kind: u8, start: u64| [vec![kind], start.to_be_bytes().to_vec()].concat();
    for span in &pending {
        rows.push(Entry::new(0, &name(1, span.start), &span.len.to_le_bytes()).unwrap());
    }
    for span in [
        Span {
            start: FREE_START + FAILURE_REGION,
            len: FAILURE_REGION,
        },
        Span {
            start: primary + PRIVATE_BYTES as u64,
            len: FAILURE_REGION - PRIVATE_BYTES as u64,
        },
        Span {
            start: mirror + PRIVATE_BYTES as u64,
            len: FAILURE_REGION - PRIVATE_BYTES as u64,
        },
    ] {
        rows.push(Entry::new(0, &name(0, span.start), &span.len.to_le_bytes()).unwrap());
    }
    for (start, bytes) in [
        (PAYLOAD, vec![37; FAILURE_REGION as usize]),
        (FREE_START, payload),
    ] {
        rows.push(
            Entry::new(
                0,
                &name(2, start),
                &[
                    FAILURE_REGION.to_le_bytes().to_vec(),
                    strong_checksum(&bytes).to_vec(),
                ]
                .concat(),
            )
            .unwrap(),
        );
    }
    for i in 0..16u32 {
        rows.push(Entry::new(1, &i.to_be_bytes(), &[(i + 17) as u8; 128]).unwrap());
    }
    rows.sort_by(|a, b| (a.namespace, a.key.as_slice()).cmp(&(b.namespace, b.key.as_slice())));
    let change = Index::new(archive(), mode, key(mode))
        .unwrap()
        .build_sorted(&mut storage, rows.into_iter().map(Ok))
        .unwrap();
    let body = manifest(change.root);
    let private = encode_private(archive(), mode, key(mode), &body).unwrap();
    for start in [primary, mirror] {
        storage.write_at(start, &private).unwrap();
    }
    let root = RootRef {
        primary,
        mirror,
        len: private.len() as u64,
        digest: strong_checksum(&private),
    };
    let anchor = Anchor {
        generation: old.anchor.generation + 1,
        previous: commitment(&old.anchor).unwrap(),
        sealed_len: storage.len().unwrap(),
        index: root,
        object_root: root.digest,
        ..old.anchor.clone()
    };
    let next =
        Tree::from_snapshot(&storage, archive(), mode, key(mode), anchor.clone(), &body).unwrap();
    old.graph.transition_to(&next.graph).unwrap();
    let publication = encode_in(
        &anchor,
        &authority,
        mode.signed().then_some(owner),
        Layout::Shared,
    )
    .unwrap();
    storage.sync().unwrap();
    storage.write_at(0, &publication).unwrap();
    storage.sync().unwrap();
    Published {
        bytes: storage.0.read_all().unwrap(),
        initial_len,
        anchor,
        pending,
        pages: change.created,
        arena,
        publication,
    }
}

fn check_published(
    bytes: Vec<u8>,
    mode: FormatMode,
    authority: &Authority<'_>,
    expected: &Published,
) {
    let storage = StorageBackend::memory(bytes);
    assert_eq!(storage.len().unwrap(), expected.anchor.sealed_len);
    let tree = Tree::open(&storage, archive(), mode, authority, key(mode)).unwrap();
    assert_eq!(
        commitment(&tree.anchor).unwrap(),
        commitment(&expected.anchor).unwrap()
    );
    let mut count = 0u32;
    tree.visit(&storage, |entry| {
        assert_eq!(entry.key.as_slice(), count.to_be_bytes());
        assert_eq!(entry.value.as_slice(), vec![(count + 17) as u8; 128]);
        count += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(count, 16);
    for (start, value) in [(PAYLOAD, 37), (FREE_START, 55)] {
        assert_eq!(
            storage.read_at(start, FAILURE_REGION as usize).unwrap(),
            vec![value; FAILURE_REGION as usize]
        );
    }
    for span in &expected.pending {
        assert!(storage
            .read_at(span.start, span.len as usize)
            .unwrap()
            .iter()
            .all(|byte| *byte == 0));
    }
}

#[test]
fn authenticated_overflow_commit_reopens_all_modes_and_each_copy_loss() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let expected = published(mode, &owner);
        // Before damaging either control bank, establish both selected banks;
        // single-published-slot interruption is covered separately below.
        let mut stable = StorageBackend::memory(expected.bytes.clone());
        stable
            .write_at(FAILURE_REGION, &expected.publication)
            .unwrap();
        let bytes = stable.read_all().unwrap();
        let losses = [
            0,
            FAILURE_REGION,
            expected.anchor.index.primary,
            expected.anchor.index.mirror,
            expected.arena.primary,
            expected.arena.mirror,
        ]
        .into_iter()
        .chain(
            expected
                .pages
                .iter()
                .flat_map(|page| [page.primary, page.mirror]),
        );
        for at in std::iter::once(None).chain(losses.map(Some)) {
            let mut damaged = StorageBackend::memory(bytes.clone());
            if let Some(at) = at {
                damaged
                    .write_at(at, &vec![0; FAILURE_REGION as usize])
                    .unwrap();
            }
            assert!(recover_abort(
                &mut ReadOnly(damaged.clone()),
                archive(),
                mode,
                &authority,
                key(mode)
            )
            .is_err());
            recover_commit(&mut damaged, archive(), mode, &authority, key(mode)).unwrap();
            recover_commit(&mut damaged, archive(), mode, &authority, key(mode)).unwrap();
            check_published(damaged.read_all().unwrap(), mode, &authority, &expected);
        }
    }
}

#[test]
fn authenticated_overflow_commit_survives_interruption_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let expected = published(mode, &owner);
        let mut observed = CrashStore::new(expected.bytes.clone(), None, 0, false);
        recover_commit(&mut observed, archive(), mode, &authority, key(mode)).unwrap();
        for at in 0..observed.operations() {
            for prefix in [0, 97, usize::MAX] {
                for persist in [false, true] {
                    let mut failed =
                        CrashStore::new(expected.bytes.clone(), Some(at), prefix, persist);
                    let _ = recover_commit(&mut failed, archive(), mode, &authority, key(mode));
                    let durable = failed.durable();
                    if durable[PRIVATE_START as usize] != expected.bytes[PRIVATE_START as usize] {
                        for bank in [0, FAILURE_REGION as usize] {
                            assert_eq!(
                                &durable[bank..bank + expected.publication.len()],
                                expected.publication.as_slice()
                            );
                        }
                    }
                    let mut reopened = StorageBackend::memory(durable);
                    recover_commit(&mut reopened, archive(), mode, &authority, key(mode))
                        .unwrap_or_else(|error| {
                            panic!("mode={bits} at={at} prefix={prefix} persist={persist}: {error}")
                        });
                    recover_commit(&mut reopened, archive(), mode, &authority, key(mode)).unwrap();
                    check_published(reopened.read_all().unwrap(), mode, &authority, &expected);
                    cases += 1;
                }
            }
        }
    }
    println!("AUTHENTICATED_OVERFLOW_COMMIT_POWER_LOSS_CASES {cases}");
}

#[test]
fn authenticated_overflow_commit_refuses_stale_and_unowned_arena_without_writes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, true, true, false);
    let authority = authority(mode, &public);
    let expected = published(mode, &owner);
    for case in 0..5 {
        let mut storage = StorageBackend::memory(expected.bytes.clone());
        match case {
            0 => {
                storage.write_at(FREE_START + FAILURE_REGION, &[1]).unwrap();
            }
            1 => {
                storage.append(&[0]).unwrap();
            }
            2 => {
                let mut wrong = expected.anchor.clone();
                wrong.previous = [93; 32];
                let encoded = encode_in(&wrong, &authority, Some(&owner), Layout::Shared).unwrap();
                for bank in [0, FAILURE_REGION] {
                    storage.write_at(bank, &encoded).unwrap();
                }
            }
            3 => {
                // A valid copied arena digest in selected payload space cannot
                // turn that live allocation into erase authority.
                let arena = storage
                    .read_at(expected.arena.primary, FAILURE_REGION as usize)
                    .unwrap();
                storage.write_at(FREE_START, &arena).unwrap();
                for bank in [0, FAILURE_REGION] {
                    let mut stub = storage.read_at(bank + 8192, 4096).unwrap();
                    stub[121..129].copy_from_slice(&FREE_START.to_le_bytes());
                    let digest = strong_checksum(&stub[..4064]);
                    stub[4064..].copy_from_slice(&digest);
                    storage.write_at(bank + 8192, &stub).unwrap();
                }
            }
            _ => {
                // Both selected descendants lost: never scan the old published
                // tree to manufacture a replacement selected membership.
                for start in [expected.pages[0].primary, expected.pages[0].mirror] {
                    storage
                        .write_at(start, &vec![0; FAILURE_REGION as usize])
                        .unwrap();
                }
            }
        }
        let before = storage.read_all().unwrap();
        let mut guarded = ReadOnly(storage);
        assert!(
            recover_commit(&mut guarded, archive(), mode, &authority, None).is_err(),
            "case {case}"
        );
        assert_eq!(guarded.0.read_all().unwrap(), before);
    }
}

#[test]
#[ignore = "bounded serial resource observation; run separately from regression tests"]
fn authenticated_overflow_commit_resource_observation() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in [4, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let expected = published(mode, &owner);
        let temporary = expected.bytes.len();
        let mut storage = StorageBackend::memory(expected.bytes.clone());
        let started = std::time::Instant::now();
        recover_commit(&mut storage, archive(), mode, &authority, key(mode)).unwrap();
        let elapsed = started.elapsed().as_nanos();
        let completed = storage.len().unwrap();
        check_published(storage.read_all().unwrap(), mode, &authority, &expected);
        println!("OVERFLOW_COMMIT_OBSERVATION mode={bits} initial_bytes={} temporary_bytes={temporary} completed_bytes={completed} pending_zero_bytes={} recovery_wall_ns={elapsed}", expected.initial_len, expected.pending.iter().map(|span| span.len).sum::<u64>());
    }
}
