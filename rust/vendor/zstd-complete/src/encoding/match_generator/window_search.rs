use super::*;

impl MatchGenerator {
    #[inline(always)]
    pub(super) fn best_window_candidate(
        &self,
        key_value: u64,
        context: &MatchCandidateContext<'_>,
        block_len: usize,
    ) -> Option<MatchCandidate> {
        let mut candidate = self.best_current_long_hash_candidate(context, block_len);
        #[cfg(test)]
        let long_hash_found = candidate;
        if candidate
            .is_some_and(|candidate| candidate.match_len >= BEST_CURRENT_LONG_HASH_SKIP_OLDER_LEN)
        {
            #[cfg(test)]
            if let Some(found) = long_hash_found {
                self.diagnostics
                    .borrow_mut()
                    .record_current_long_hash_outcome(found, candidate);
            }
            return candidate;
        }
        let current_long_hash_active = candidate.is_some();
        let skip_current_entry = candidate.is_some();
        if self.uniform_suffix_len_log == Some(self.last_entry().suffixes.len_log) {
            let slot_key = self.last_entry().suffixes.slot_key(key_value);
            for (entry_distance, match_entry) in self
                .window
                .iter()
                .rev()
                .enumerate()
                .skip(skip_current_entry as usize)
            {
                let skip_newest_for_entry = current_long_hash_active
                    && entry_distance >= BEST_CURRENT_LONG_HASH_DISTANT_NEWEST_ENTRY_START;
                let Some(candidates) = match_entry.suffixes.candidates_for_slot_key(slot_key)
                else {
                    continue;
                };
                if self.prefer_oldest_first_window_probe {
                    let prefer_newest_first_for_entry =
                        current_long_hash_active && entry_distance == 1;

                    if prefer_newest_first_for_entry {
                        if !skip_newest_for_entry {
                            if let Some(match_index) = candidates.newest {
                                if self.consider_window_candidate_with_tracking(
                                    match_entry,
                                    match_index,
                                    context,
                                    WindowCandidateMeta {
                                        entry_distance,
                                        kind: WindowCandidateKind::Newest,
                                    },
                                    &mut candidate,
                                    block_len,
                                    current_long_hash_active,
                                ) {
                                    break;
                                }
                            }
                        }
                    } else if self.consider_window_candidate_with_tracking(
                        match_entry,
                        candidates.oldest,
                        context,
                        WindowCandidateMeta {
                            entry_distance,
                            kind: WindowCandidateKind::Oldest,
                        },
                        &mut candidate,
                        block_len,
                        current_long_hash_active,
                    ) {
                        break;
                    }

                    if self.should_track_second_newest_for_current_entry() {
                        if let Some(match_index) = self.best_second_newest_candidate(
                            entry_distance,
                            slot_key.index,
                            context,
                            candidate,
                        ) {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::SecondNewest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                    }

                    if prefer_newest_first_for_entry {
                        if self.consider_window_candidate_with_tracking(
                            match_entry,
                            candidates.oldest,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::Oldest,
                            },
                            &mut candidate,
                            block_len,
                            current_long_hash_active,
                        ) {
                            break;
                        }
                    } else if !skip_newest_for_entry {
                        if let Some(match_index) = candidates.newest {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::Newest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                    }
                } else {
                    if self.prefer_lockfile_second_newest_before_newest()
                        && self.should_track_second_newest_for_current_entry()
                    {
                        if let Some(match_index) = self.best_second_newest_candidate(
                            entry_distance,
                            slot_key.index,
                            context,
                            candidate,
                        ) {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::SecondNewest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                        if let Some(match_index) = self.best_third_newest_candidate(
                            entry_distance,
                            slot_key.index,
                            context,
                        ) {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::SecondNewest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                        if let Some(match_index) = self.best_fourth_newest_candidate(
                            entry_distance,
                            slot_key.index,
                            context,
                        ) {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::SecondNewest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                        if let Some(match_index) = self.best_fifth_newest_candidate(
                            entry_distance,
                            slot_key.index,
                            context,
                        ) {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::SecondNewest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                        if let Some(match_index) = self.best_sixth_newest_candidate(
                            entry_distance,
                            slot_key.index,
                            context,
                        ) {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::SecondNewest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                        if let Some(match_index) = self.best_seventh_newest_candidate(
                            entry_distance,
                            slot_key.index,
                            context,
                        ) {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::SecondNewest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                        if let Some(match_index) = self.best_eighth_newest_candidate(
                            entry_distance,
                            slot_key.index,
                            context,
                        ) {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::SecondNewest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                        if let Some(match_index) = self.best_ninth_newest_candidate(
                            entry_distance,
                            slot_key.index,
                            context,
                        ) {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::SecondNewest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                        if let Some(match_index) = self.best_tenth_newest_candidate(
                            entry_distance,
                            slot_key.index,
                            context,
                        ) {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::SecondNewest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                        if let Some(match_index) = self.best_eleventh_newest_candidate(
                            entry_distance,
                            slot_key.index,
                            context,
                        ) {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::SecondNewest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                        if let Some(match_index) = self.best_twelfth_newest_candidate(
                            entry_distance,
                            slot_key.index,
                            context,
                        ) {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::SecondNewest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                        if let Some(match_index) = self.best_thirteenth_newest_candidate(
                            entry_distance,
                            slot_key.index,
                            context,
                        ) {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::SecondNewest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                    }

                    if !skip_newest_for_entry {
                        if let Some(match_index) = candidates.newest {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::Newest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                    }

                    if !self.prefer_lockfile_second_newest_before_newest()
                        && self.should_track_second_newest_for_current_entry()
                    {
                        if let Some(match_index) = self.best_second_newest_candidate(
                            entry_distance,
                            slot_key.index,
                            context,
                            candidate,
                        ) {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::SecondNewest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                    }

                    if self.consider_window_candidate_with_tracking(
                        match_entry,
                        candidates.oldest,
                        context,
                        WindowCandidateMeta {
                            entry_distance,
                            kind: WindowCandidateKind::Oldest,
                        },
                        &mut candidate,
                        block_len,
                        current_long_hash_active,
                    ) {
                        break;
                    }
                }
            }
            #[cfg(test)]
            if let Some(found) = long_hash_found {
                self.diagnostics
                    .borrow_mut()
                    .record_current_long_hash_outcome(found, candidate);
            }
            return candidate;
        }

        for (entry_distance, match_entry) in self
            .window
            .iter()
            .rev()
            .enumerate()
            .skip(skip_current_entry as usize)
        {
            let skip_newest_for_entry = current_long_hash_active
                && entry_distance >= BEST_CURRENT_LONG_HASH_DISTANT_NEWEST_ENTRY_START;
            let Some(candidates) = match_entry.suffixes.candidates_for_key_value(key_value) else {
                continue;
            };
            if self.prefer_oldest_first_window_probe {
                let prefer_newest_first_for_entry = current_long_hash_active && entry_distance == 1;

                if prefer_newest_first_for_entry {
                    if !skip_newest_for_entry {
                        if let Some(match_index) = candidates.newest {
                            if self.consider_window_candidate_with_tracking(
                                match_entry,
                                match_index,
                                context,
                                WindowCandidateMeta {
                                    entry_distance,
                                    kind: WindowCandidateKind::Newest,
                                },
                                &mut candidate,
                                block_len,
                                current_long_hash_active,
                            ) {
                                break;
                            }
                        }
                    }
                } else if self.consider_window_candidate_with_tracking(
                    match_entry,
                    candidates.oldest,
                    context,
                    WindowCandidateMeta {
                        entry_distance,
                        kind: WindowCandidateKind::Oldest,
                    },
                    &mut candidate,
                    block_len,
                    current_long_hash_active,
                ) {
                    break;
                }

                if self.should_track_second_newest_for_current_entry() {
                    if let Some(match_index) = self.best_second_newest_candidate(
                        entry_distance,
                        match_entry.suffixes.slot_key(key_value).index,
                        context,
                        candidate,
                    ) {
                        if self.consider_window_candidate_with_tracking(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::SecondNewest,
                            },
                            &mut candidate,
                            block_len,
                            current_long_hash_active,
                        ) {
                            break;
                        }
                    }
                }

                if prefer_newest_first_for_entry {
                    if self.consider_window_candidate_with_tracking(
                        match_entry,
                        candidates.oldest,
                        context,
                        WindowCandidateMeta {
                            entry_distance,
                            kind: WindowCandidateKind::Oldest,
                        },
                        &mut candidate,
                        block_len,
                        current_long_hash_active,
                    ) {
                        break;
                    }
                } else if !skip_newest_for_entry {
                    if let Some(match_index) = candidates.newest {
                        if self.consider_window_candidate_with_tracking(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::Newest,
                            },
                            &mut candidate,
                            block_len,
                            current_long_hash_active,
                        ) {
                            break;
                        }
                    }
                }
            } else {
                if self.prefer_lockfile_second_newest_before_newest()
                    && self.should_track_second_newest_for_current_entry()
                {
                    if let Some(match_index) = self.best_second_newest_candidate(
                        entry_distance,
                        match_entry.suffixes.slot_key(key_value).index,
                        context,
                        candidate,
                    ) {
                        if self.consider_window_candidate(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::SecondNewest,
                            },
                            &mut candidate,
                            block_len,
                        ) {
                            break;
                        }
                    }
                    if let Some(match_index) = self.best_third_newest_candidate(
                        entry_distance,
                        match_entry.suffixes.slot_key(key_value).index,
                        context,
                    ) {
                        if self.consider_window_candidate(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::SecondNewest,
                            },
                            &mut candidate,
                            block_len,
                        ) {
                            break;
                        }
                    }
                    if let Some(match_index) = self.best_fourth_newest_candidate(
                        entry_distance,
                        match_entry.suffixes.slot_key(key_value).index,
                        context,
                    ) {
                        if self.consider_window_candidate(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::SecondNewest,
                            },
                            &mut candidate,
                            block_len,
                        ) {
                            break;
                        }
                    }
                    if let Some(match_index) = self.best_fifth_newest_candidate(
                        entry_distance,
                        match_entry.suffixes.slot_key(key_value).index,
                        context,
                    ) {
                        if self.consider_window_candidate(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::SecondNewest,
                            },
                            &mut candidate,
                            block_len,
                        ) {
                            break;
                        }
                    }
                    if let Some(match_index) = self.best_sixth_newest_candidate(
                        entry_distance,
                        match_entry.suffixes.slot_key(key_value).index,
                        context,
                    ) {
                        if self.consider_window_candidate(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::SecondNewest,
                            },
                            &mut candidate,
                            block_len,
                        ) {
                            break;
                        }
                    }
                    if let Some(match_index) = self.best_seventh_newest_candidate(
                        entry_distance,
                        match_entry.suffixes.slot_key(key_value).index,
                        context,
                    ) {
                        if self.consider_window_candidate(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::SecondNewest,
                            },
                            &mut candidate,
                            block_len,
                        ) {
                            break;
                        }
                    }
                    if let Some(match_index) = self.best_eighth_newest_candidate(
                        entry_distance,
                        match_entry.suffixes.slot_key(key_value).index,
                        context,
                    ) {
                        if self.consider_window_candidate(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::SecondNewest,
                            },
                            &mut candidate,
                            block_len,
                        ) {
                            break;
                        }
                    }
                    if let Some(match_index) = self.best_ninth_newest_candidate(
                        entry_distance,
                        match_entry.suffixes.slot_key(key_value).index,
                        context,
                    ) {
                        if self.consider_window_candidate(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::SecondNewest,
                            },
                            &mut candidate,
                            block_len,
                        ) {
                            break;
                        }
                    }
                    if let Some(match_index) = self.best_tenth_newest_candidate(
                        entry_distance,
                        match_entry.suffixes.slot_key(key_value).index,
                        context,
                    ) {
                        if self.consider_window_candidate(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::SecondNewest,
                            },
                            &mut candidate,
                            block_len,
                        ) {
                            break;
                        }
                    }
                    if let Some(match_index) = self.best_eleventh_newest_candidate(
                        entry_distance,
                        match_entry.suffixes.slot_key(key_value).index,
                        context,
                    ) {
                        if self.consider_window_candidate(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::SecondNewest,
                            },
                            &mut candidate,
                            block_len,
                        ) {
                            break;
                        }
                    }
                    if let Some(match_index) = self.best_twelfth_newest_candidate(
                        entry_distance,
                        match_entry.suffixes.slot_key(key_value).index,
                        context,
                    ) {
                        if self.consider_window_candidate(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::SecondNewest,
                            },
                            &mut candidate,
                            block_len,
                        ) {
                            break;
                        }
                    }
                    if let Some(match_index) = self.best_thirteenth_newest_candidate(
                        entry_distance,
                        match_entry.suffixes.slot_key(key_value).index,
                        context,
                    ) {
                        if self.consider_window_candidate(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::SecondNewest,
                            },
                            &mut candidate,
                            block_len,
                        ) {
                            break;
                        }
                    }
                }

                if !skip_newest_for_entry {
                    if let Some(match_index) = candidates.newest {
                        if self.consider_window_candidate_with_tracking(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::Newest,
                            },
                            &mut candidate,
                            block_len,
                            current_long_hash_active,
                        ) {
                            break;
                        }
                    }
                }

                if !self.prefer_lockfile_second_newest_before_newest()
                    && self.should_track_second_newest_for_current_entry()
                {
                    if let Some(match_index) = self.best_second_newest_candidate(
                        entry_distance,
                        match_entry.suffixes.slot_key(key_value).index,
                        context,
                        candidate,
                    ) {
                        if self.consider_window_candidate(
                            match_entry,
                            match_index,
                            context,
                            WindowCandidateMeta {
                                entry_distance,
                                kind: WindowCandidateKind::SecondNewest,
                            },
                            &mut candidate,
                            block_len,
                        ) {
                            break;
                        }
                    }
                }

                if self.consider_window_candidate_with_tracking(
                    match_entry,
                    candidates.oldest,
                    context,
                    WindowCandidateMeta {
                        entry_distance,
                        kind: WindowCandidateKind::Oldest,
                    },
                    &mut candidate,
                    block_len,
                    current_long_hash_active,
                ) {
                    break;
                }
            }
        }

        #[cfg(test)]
        if let Some(found) = long_hash_found {
            self.diagnostics
                .borrow_mut()
                .record_current_long_hash_outcome(found, candidate);
        }
        candidate
    }
}
