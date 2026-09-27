//! Internal graph fixtures: no public CLI can construct this experimental layout.
use super::*;
use crate::storage::StorageBackend;
fn initial() -> (Anchor, Vec<Extent>, Vec<Vacant>) {
    let (_, mut anchor) = super::super::tests::fixture(super::super::tests::mode(false, false));
    anchor.keys = RootRef::default();
    anchor.sealed_len = 3 * FAILURE_REGION;
    let packs = vec![Extent {
        start: REGION_LEN as u64,
        len: FAILURE_REGION,
        digest: [1; 32],
    }];
    (anchor, packs, key_slots())
}
fn key_slots() -> Vec<Vacant> {
    [0, FAILURE_REGION]
        .into_iter()
        .map(|bank| Vacant {
            span: Span {
                start: bank + KEYS_START,
                len: 4096,
            },
            kind: VacantKind::Free,
        })
        .collect()
}
fn advance(old: &Anchor, index: RootRef, sealed: u64) -> Anchor {
    Anchor {
        generation: old.generation + 1,
        previous: commitment(old).unwrap(),
        object_root: index.digest,
        index,
        sealed_len: sealed,
        ..old.clone()
    }
}
fn external(old: &Anchor) -> (Anchor, Vec<Vacant>) {
    let index = RootRef {
        primary: 3 * FAILURE_REGION,
        mirror: 4 * FAILURE_REGION,
        len: PRIVATE_BYTES as u64,
        digest: [2; 32],
    };
    let mut vacant = key_slots();
    for bank in [0, FAILURE_REGION] {
        vacant.push(Vacant {
            span: Span {
                start: bank + PRIVATE_START,
                len: PRIVATE_BYTES as u64,
            },
            kind: VacantKind::Pending,
        });
    }
    for start in [index.primary, index.mirror] {
        vacant.push(Vacant {
            span: Span {
                start: start + index.len,
                len: FAILURE_REGION - index.len,
            },
            kind: VacantKind::Free,
        });
    }
    (advance(old, index, 5 * FAILURE_REGION), vacant)
}
#[test]
fn graph_accounts_for_every_byte_and_refuses_control_payload_aliases() {
    let (anchor, packs, vacant) = initial();
    let graph = Graph::derive(&anchor, &packs, &vacant).unwrap();
    assert_eq!(
        graph
            .claims
            .values()
            .map(|claim| claim.span.len)
            .sum::<u64>(),
        anchor.sealed_len
    );
    assert!(Graph::derive(&anchor, &packs, &vacant[..1]).is_err());
    assert!(Graph::derive(&anchor, &[], &vacant).is_err());
    assert!(Graph::derive(&anchor, &[packs[0], packs[0]], &vacant).is_err());
    let mut malformed = packs.clone();
    malformed[0].start += 1;
    assert!(Graph::derive(&anchor, &malformed, &vacant).is_err());
    malformed[0].start = u64::MAX;
    assert!(Graph::derive(&anchor, &malformed, &vacant).is_err());
    malformed[0] = Extent {
        start: PRIVATE_START,
        len: PRIVATE_BYTES as u64,
        digest: anchor.index.digest,
    };
    assert!(Graph::derive(&anchor, &malformed, &vacant).is_err());
    let mut alias = vacant.clone();
    alias.push(Vacant {
        span: Span {
            start: 0,
            len: 8192,
        },
        kind: VacantKind::Free,
    });
    assert!(Graph::derive(&anchor, &packs, &alias).is_err());
    let mut combined = vacant.clone();
    combined[0].span.len += PRIVATE_BYTES as u64;
    assert!(Graph::derive(&anchor, &packs, &combined).is_err());
    let mut unsupported = anchor.clone();
    unsupported.allocation = RootRef {
        primary: 5 * FAILURE_REGION,
        mirror: 6 * FAILURE_REGION,
        len: 4096,
        digest: [3; 32],
    };
    unsupported.sealed_len = 7 * FAILURE_REGION;
    assert!(Graph::derive(&unsupported, &packs, &vacant).is_err());
}
#[test]
fn inline_external_inline_transition_requires_reservation_retirement_and_erasure() {
    let (anchor, packs, vacant) = initial();
    let old = Graph::derive(&anchor, &packs, &vacant).unwrap();
    let (external, extvacant) = external(&anchor);
    let staged = Graph::derive(&external, &packs, &extvacant).unwrap();
    let plan = old.transition_to(&staged).unwrap();
    assert_eq!(
        plan.append,
        Some(Span {
            start: anchor.sealed_len,
            len: 2 * FAILURE_REGION
        })
    );
    assert_eq!(plan.writes.len(), 2);
    assert!(plan.erase_before_write.is_empty());
    assert_eq!(
        plan.retire_after_publication,
        vec![
            Span {
                start: PRIVATE_START,
                len: PRIVATE_BYTES as u64
            },
            Span {
                start: FAILURE_REGION + PRIVATE_START,
                len: PRIVATE_BYTES as u64
            }
        ]
    );
    let root = RootRef {
        digest: [4; 32],
        ..anchor.index
    };
    let returned = advance(&external, root, external.sealed_len);
    let mut retired = key_slots();
    for start in [external.index.primary, external.index.mirror] {
        retired.push(Vacant {
            span: Span {
                start,
                len: PRIVATE_BYTES as u64,
            },
            kind: VacantKind::Pending,
        });
        retired.push(Vacant {
            span: Span {
                start: start + PRIVATE_BYTES as u64,
                len: FAILURE_REGION - PRIVATE_BYTES as u64,
            },
            kind: VacantKind::Free,
        });
    }
    let inline = Graph::derive(&returned, &packs, &retired).unwrap();
    let plan = staged.transition_to(&inline).unwrap();
    assert!(plan.append.is_none());
    assert_eq!(plan.writes.len(), 2);
    assert_eq!(
        plan.erase_before_write,
        vec![
            Span {
                start: PRIVATE_START,
                len: PRIVATE_BYTES as u64
            },
            Span {
                start: FAILURE_REGION + PRIVATE_START,
                len: PRIVATE_BYTES as u64
            }
        ]
    );
    assert_eq!(plan.retire_after_publication.len(), 2);
    // Returning inline does not make it safe to truncate a still-sealed tail.
    let shrink = advance(&returned, root, anchor.sealed_len);
    let shrink = Graph::derive(&shrink, &packs, &vacant).unwrap();
    assert!(inline.transition_to(&shrink).is_err());
    // Erased external slots may become free; the plan records the required proof.
    let mut free = key_slots();
    free.push(Vacant {
        span: Span {
            start: anchor.sealed_len,
            len: 2 * FAILURE_REGION,
        },
        kind: VacantKind::Free,
    });
    let reclaimed = advance(&returned, root, returned.sealed_len);
    let reclaimed = Graph::derive(&reclaimed, &packs, &free).unwrap();
    let plan = inline.transition_to(&reclaimed).unwrap();
    assert_eq!(plan.prove_erased.len(), 2);
    assert!(plan.writes.is_empty());
}
#[test]
fn transitions_refuse_in_place_mutation_early_free_and_unlinked_publication() {
    let (anchor, packs, vacant) = initial();
    let old = Graph::derive(&anchor, &packs, &vacant).unwrap();
    let changed = advance(
        &anchor,
        RootRef {
            digest: [9; 32],
            ..anchor.index
        },
        anchor.sealed_len,
    );
    assert!(old
        .transition_to(&Graph::derive(&changed, &packs, &vacant).unwrap())
        .is_err());
    let (ext, mut vacant) = external(&anchor);
    for entry in &mut vacant {
        entry.kind = VacantKind::Free;
    }
    assert!(old
        .transition_to(&Graph::derive(&ext, &packs, &vacant).unwrap())
        .is_err());
    let (mut ext, vacant) = external(&anchor);
    ext.previous = [8; 32];
    assert!(old
        .transition_to(&Graph::derive(&ext, &packs, &vacant).unwrap())
        .is_err());
    ext.previous = commitment(&anchor).unwrap();
    ext.generation += 1;
    assert!(old
        .transition_to(&Graph::derive(&ext, &packs, &vacant).unwrap())
        .is_err());
    // A same-address payload rewrite is also forbidden even if metadata is COW.
    let (ext, vacant) = external(&anchor);
    let mut changed = packs.clone();
    changed[0].digest = [8; 32];
    assert!(old
        .transition_to(&Graph::derive(&ext, &changed, &vacant).unwrap())
        .is_err());
}
#[test]
fn free_bytes_are_checked_but_pending_secret_bytes_are_not_misclassified_as_free() {
    let (anchor, packs, vacant) = initial();
    let graph = Graph::derive(&anchor, &packs, &vacant).unwrap();
    let mut storage = StorageBackend::memory(vec![0; anchor.sealed_len as usize]);
    graph.verify_free(&storage).unwrap();
    storage
        .write_at(KEYS_START, b"retired wrapping bytes")
        .unwrap();
    assert!(graph.verify_free(&storage).is_err());
    storage.write_at(KEYS_START, &vec![0; 4096]).unwrap();
    let (ext, vacant) = external(&anchor);
    storage
        .append(&vec![0; (ext.sealed_len - anchor.sealed_len) as usize])
        .unwrap();
    storage
        .write_at(PRIVATE_START, b"retired private names and secrets")
        .unwrap();
    Graph::derive(&ext, &packs, &vacant)
        .unwrap()
        .verify_free(&storage)
        .unwrap();
    storage
        .write_at(
            ext.index.primary + ext.index.len,
            b"unexpected allocation tail",
        )
        .unwrap();
    assert!(Graph::derive(&ext, &packs, &vacant)
        .unwrap()
        .verify_free(&storage)
        .is_err());
}
#[test]
fn metadata_tail_transition_preserves_payload_and_proves_removed_suffix() {
    let (initial, packs, vacant) = initial();
    let (external, external_vacant) = external(&initial);
    let old = Graph::derive(&external, &packs, &external_vacant).unwrap();
    let next = advance(
        &external,
        RootRef {
            digest: [8; 32],
            ..initial.index
        },
        initial.sealed_len,
    );
    let inline = Graph::derive(&next, &packs, &vacant).unwrap();
    assert!(old.transition_to(&inline).is_err());
    let plan = old.metadata_tail_transition_to(&inline).unwrap();
    assert_eq!(
        plan.truncate,
        Some(Span {
            start: initial.sealed_len,
            len: 2 * FAILURE_REGION
        })
    );
    assert_eq!(plan.writes.len(), 2);
    assert_eq!(plan.retire_after_publication.len(), 2);
    assert!(plan.append.is_none());
    let mut substituted = packs.clone();
    substituted[0].digest = [42; 32];
    assert!(old
        .metadata_tail_transition_to(&Graph::derive(&next, &substituted, &vacant).unwrap())
        .is_err());
    let removed = Anchor {
        sealed_len: REGION_LEN as u64,
        ..next.clone()
    };
    assert!(old
        .metadata_tail_transition_to(&Graph::derive(&removed, &[], &vacant).unwrap())
        .is_err());
    let stale = Anchor {
        previous: [99; 32],
        ..next
    };
    assert!(old
        .metadata_tail_transition_to(&Graph::derive(&stale, &packs, &vacant).unwrap())
        .is_err());
}
