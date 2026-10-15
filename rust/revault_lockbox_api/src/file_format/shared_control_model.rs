//! Abstract ordering model, not a persisted encoding, allocator or crash harness.
//! Each record is valid, torn, or absent; authentication and durable writes are
//! assumptions supplied by the existing independently tested components. This
//! model checks root reachability and retirement ordering when private metadata
//! shares the two publication failure regions. It does not model journal overflow,
//! descendant graphs, key rotation, malicious input or arbitrary device failures.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Place {
    Inline,
    External,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Root {
    generation: u64,
    place: Place,
    contents: u8,
}
#[derive(Clone, Debug)]
struct State {
    anchors: [Option<Root>; 2],
    inline: [Option<u8>; 2],
    external: [Option<u8>; 2],
    // Distinguish unallocated tail from a zeroed owned allocation.
    tail_owned: bool,
    tail_allocated: bool,
}
impl State {
    fn initial() -> Self {
        Self {
            anchors: [Some(Root {
                generation: 1,
                place: Place::Inline,
                contents: 1,
            }); 2],
            inline: [Some(1); 2],
            external: [None; 2],
            tail_owned: false,
            tail_allocated: false,
        }
    }
    fn selected(&self) -> Option<Root> {
        self.anchors
            .iter()
            .flatten()
            .copied()
            .max_by_key(|r| r.generation)
    }
    fn metadata(&self, place: Place) -> &[Option<u8>; 2] {
        match place {
            Place::Inline => &self.inline,
            Place::External => &self.external,
        }
    }
    fn metadata_mut(&mut self, place: Place) -> &mut [Option<u8>; 2] {
        match place {
            Place::Inline => &mut self.inline,
            Place::External => &mut self.external,
        }
    }
    fn readable(&self) -> bool {
        self.selected()
            .is_some_and(|root| self.metadata(root.place).contains(&Some(root.contents)))
    }
    fn both_publish(&self, root: Root) -> bool {
        self.anchors == [Some(root); 2]
    }
    fn live_in(&self, place: Place) -> bool {
        self.anchors.iter().flatten().any(|r| r.place == place)
    }
    fn lose_region(&mut self, region: usize) {
        // Each external copy also occupies its own 64 KiB failure region.
        match region {
            0 | 1 => {
                self.anchors[region] = None;
                self.inline[region] = None;
            }
            2 | 3 => self.external[region - 2] = None,
            _ => unreachable!(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
enum Action {
    OwnTail,
    Write {
        place: Place,
        copy: usize,
        contents: u8,
    },
    Publish {
        copy: usize,
        root: Root,
    },
    Erase {
        place: Place,
        copy: usize,
    },
    TruncateTail,
}
impl Action {
    /// All completed actions here include a successful durability barrier.
    /// An interrupted record write can leave its target invalid instead. Cleanup
    /// permissions depend on both anchors, never merely the selected newest one.
    fn apply(self, state: &mut State, torn: bool) -> Result<(), &'static str> {
        match self {
            Self::OwnTail => {
                if !torn {
                    state.tail_owned = true;
                }
            }
            Self::Write {
                place,
                copy,
                contents,
            } => {
                if state.live_in(place) {
                    return Err("cannot overwrite a live metadata placement");
                }
                if place == Place::External {
                    if !state.tail_owned {
                        return Err("append tail has no durable preparation owner");
                    }
                    state.tail_allocated = true;
                }
                state.metadata_mut(place)[copy] = if torn { None } else { Some(contents) };
            }
            Self::Publish { copy, root } => {
                if state.metadata(root.place) != &[Some(root.contents); 2] {
                    return Err("both new metadata copies must be durable before publication");
                }
                let current = state.selected().ok_or("no publication")?;
                if root.generation != current.generation
                    && root.generation != current.generation + 1
                {
                    return Err("non-adjacent publication");
                }
                if root.generation == current.generation && root != current {
                    return Err("conflicting publication");
                }
                state.anchors[copy] = if torn { None } else { Some(root) };
            }
            Self::Erase { place, copy } => {
                let current = state.selected().ok_or("no publication")?;
                if !state.both_publish(current) || state.live_in(place) {
                    return Err(
                        "retirement requires both durable publications and no live reference",
                    );
                }
                // Torn zeroing is treated as invalid old metadata. This model
                // establishes readability, not byte-erasure completion.
                state.metadata_mut(place)[copy] = None;
            }
            Self::TruncateTail => {
                if state.live_in(Place::External) || state.external != [None; 2] {
                    return Err("cannot truncate live or unerased external metadata");
                }
                if !torn {
                    state.tail_allocated = false;
                    state.tail_owned = false;
                }
            }
        }
        Ok(())
    }
}
fn schedule() -> Vec<Action> {
    let external = Root {
        generation: 2,
        place: Place::External,
        contents: 2,
    };
    let inline = Root {
        generation: 3,
        place: Place::Inline,
        contents: 2,
    };
    vec![
        Action::OwnTail,
        Action::Write {
            place: Place::External,
            copy: 0,
            contents: 2,
        },
        Action::Write {
            place: Place::External,
            copy: 1,
            contents: 2,
        },
        Action::Publish {
            copy: 0,
            root: external,
        },
        Action::Publish {
            copy: 1,
            root: external,
        },
        Action::Erase {
            place: Place::Inline,
            copy: 0,
        },
        Action::Erase {
            place: Place::Inline,
            copy: 1,
        },
        Action::Write {
            place: Place::Inline,
            copy: 0,
            contents: 2,
        },
        Action::Write {
            place: Place::Inline,
            copy: 1,
            contents: 2,
        },
        Action::Publish {
            copy: 0,
            root: inline,
        },
        Action::Publish {
            copy: 1,
            root: inline,
        },
        Action::Erase {
            place: Place::External,
            copy: 0,
        },
        Action::Erase {
            place: Place::External,
            copy: 1,
        },
        Action::TruncateTail,
    ]
}
#[test]
fn shared_control_detour_preserves_reads_at_each_interruption_and_single_region_loss() {
    let mut state = State::initial();
    for (step, action) in schedule().into_iter().enumerate() {
        // A failed action may have persisted nothing, a torn record, or its full
        // record. A further failure-region loss is a separate fault experiment.
        for outcome in 0..3 {
            let mut crashed = state.clone();
            if outcome > 0 {
                action.apply(&mut crashed, outcome == 1).unwrap();
            }
            assert!(crashed.readable(), "step {step}, outcome {outcome}");
            assert!(!crashed.tail_allocated || crashed.tail_owned);
        }
        action.apply(&mut state, false).unwrap();
        for region in 0..4 {
            let mut damaged = state.clone();
            damaged.lose_region(region);
            assert!(damaged.readable(), "step {step}, lost region {region}");
        }
    }
    assert_eq!(
        state.selected().unwrap(),
        Root {
            generation: 3,
            place: Place::Inline,
            contents: 2
        }
    );
    assert_eq!(state.external, [None; 2]);
    assert!(!state.tail_allocated);
    assert!(!state.tail_owned);
}
#[test]
fn shared_control_rejects_in_place_replacement_and_early_retirement() {
    let initial = State::initial();
    for copy in 0..2 {
        assert!(Action::Write {
            place: Place::Inline,
            copy,
            contents: 2
        }
        .apply(&mut initial.clone(), false)
        .is_err());
    }
    assert!(Action::Write {
        place: Place::External,
        copy: 0,
        contents: 2
    }
    .apply(&mut initial.clone(), false)
    .is_err());
    let actions = schedule();
    let mut state = initial;
    for action in &actions[..4] {
        action.apply(&mut state, false).unwrap();
    }
    // The first new publication is readable, but cannot authorize erasing either
    // old inline copy while the peer can still select the old root after damage.
    assert_eq!(state.selected().unwrap().generation, 2);
    for copy in 0..2 {
        assert!(Action::Erase {
            place: Place::Inline,
            copy
        }
        .apply(&mut state.clone(), false)
        .is_err());
    }
    actions[4].apply(&mut state, false).unwrap();
    assert!(Action::Erase {
        place: Place::External,
        copy: 0
    }
    .apply(&mut state.clone(), false)
    .is_err());
    assert!(Action::TruncateTail.apply(&mut state, false).is_err());
}
