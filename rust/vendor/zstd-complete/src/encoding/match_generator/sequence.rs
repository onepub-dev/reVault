use super::*;

impl MatchGenerator {
    /// Processes bytes in the current window until either a match is found or no more matches can be found
    /// * If a match is found handle_sequence is called with the Triple variant
    /// * If no more matches can be found but there are bytes still left handle_sequence is called with the Literals variant
    /// * If no more matches can be found and no more bytes are left this returns false
    pub(super) fn next_sequence(
        &mut self,
        mut handle_sequence: impl for<'a> FnMut(Sequence<'a>),
    ) -> bool {
        if self.use_best_text_repeat_pipeline() {
            return self.next_sequence_best_text(&mut handle_sequence);
        }

        loop {
            let block_len = self.last_entry().data.len();

            // We already reached the end of the window, check if we need to return a Literals{}
            if self.suffix_idx >= block_len {
                if self.last_idx_in_sequence != self.suffix_idx {
                    let last_entry_idx = self.last_entry_index();
                    let literals = &self.window[last_entry_idx].data[self.last_idx_in_sequence..];
                    self.last_idx_in_sequence = self.suffix_idx;
                    handle_sequence(Sequence::Literals { literals });
                    return true;
                } else {
                    return false;
                }
            }

            // If the remaining data is smaller than the minimum match length we can stop and return a Literals{}
            let last_entry_idx = self.last_entry_index();
            let last_entry = &self.window[last_entry_idx];
            let data_slice = &last_entry.data[self.suffix_idx..];
            if data_slice.len() < MIN_MATCH_LEN {
                let last_idx_in_sequence = self.last_idx_in_sequence;
                self.last_idx_in_sequence = block_len;
                self.suffix_idx = block_len;
                handle_sequence(Sequence::Literals {
                    literals: &last_entry.data[last_idx_in_sequence..],
                });
                return true;
            }

            // This is the key we are looking to find a match for
            let key_value = SuffixStore::key_value(&data_slice[..MIN_MATCH_LEN]);

            // Look in each window entry
            let match_context = MatchCandidateContext {
                suffix_idx: self.suffix_idx,
                anchor_idx: self.last_idx_in_sequence,
                min_non_repeat_match_len: self.min_non_repeat_match_len,
                data_slice,
                #[cfg(debug_assertions)]
                last_entry_len: block_len,
                #[cfg(debug_assertions)]
                concat_window: &self.concat_window,
            };

            let previous_window_len = self.window_size - block_len;
            let literal_len = self.suffix_idx - self.last_idx_in_sequence;
            let mut candidate = None;
            for (repeat_idx, &(_repeat_kind, offset)) in self
                .repeat_offset_candidates(literal_len)
                .iter()
                .enumerate()
            {
                if !self.allow_repeat_candidate(literal_len, repeat_idx) {
                    continue;
                }
                if !Self::repeat_offset_is_available(offset, previous_window_len, self.suffix_idx) {
                    continue;
                }
                let Some(verified_prefix_len) =
                    self.verified_min_match_prefix_len(offset, &match_context)
                else {
                    continue;
                };
                let match_len = self.match_len_at_offset_with_prefix(
                    offset,
                    &match_context,
                    verified_prefix_len,
                );
                if match_len >= MIN_MATCH_LEN {
                    let found = MatchCandidate {
                        start_idx: self.suffix_idx,
                        offset,
                        match_len,
                        repeat_offset: true,
                        #[cfg(test)]
                        source: CandidateSource::RepeatCurrent(_repeat_kind),
                    };
                    if candidate
                        .map(|current| self.candidate_is_better_than(found, current))
                        .unwrap_or(true)
                    {
                        candidate = Some(found);
                    }
                }
            }

            if self.prefer_binary_next_position_repeat_lookahead
                || (self.prefer_fast_binary_next_position_repeat_lookahead && !self.is_text_block)
            {
                let should_probe = match candidate {
                    None => true,
                    Some(current) => {
                        !current.repeat_offset
                            && current.start_idx == self.suffix_idx
                            && current.match_len == MIN_MATCH_LEN
                    }
                };
                if should_probe {
                    if let Some(next_candidate) = self.best_repeat_candidate_at(
                        self.suffix_idx + 1,
                        self.last_idx_in_sequence,
                        previous_window_len,
                        block_len,
                    ) {
                        #[cfg(test)]
                        let next_candidate = MatchCandidate {
                            source: CandidateSource::RepeatNextPosition(
                                next_candidate.source_repeat_kind(),
                            ),
                            ..next_candidate
                        };
                        match candidate {
                            None => {
                                #[cfg(test)]
                                self.diagnostics
                                    .borrow_mut()
                                    .record_repeat_next_position_selection(
                                        next_candidate.source_repeat_kind(),
                                        RepeatNextPositionSelectionReason::NoCurrentCandidate,
                                    );
                                candidate = Some(next_candidate);
                            }
                            Some(current)
                                if self.candidate_is_better_than(next_candidate, current) =>
                            {
                                #[cfg(test)]
                                self.diagnostics
                                    .borrow_mut()
                                    .record_repeat_next_position_selection(
                                        next_candidate.source_repeat_kind(),
                                        RepeatNextPositionSelectionReason::BeatsCurrentMinNonRepeat,
                                    );
                                candidate = Some(next_candidate);
                            }
                            _ => {}
                        }
                    }
                }
            }

            #[cfg(test)]
            if let Some(found) = candidate.filter(|found| found.repeat_offset) {
                self.diagnostics
                    .borrow_mut()
                    .record_repeat_best_before_window(
                        found.source_repeat_kind(),
                        found.start_idx == self.last_idx_in_sequence,
                    );
            }
            let repeat_match_reaches_end_or_is_long = candidate
                .is_some_and(|found| self.repeat_match_can_skip_window_search(found, block_len));
            if !repeat_match_reaches_end_or_is_long {
                if let Some(window_candidate) =
                    self.best_window_candidate(key_value, &match_context, block_len)
                {
                    #[cfg(test)]
                    if let Some(found) = candidate.filter(|found| found.repeat_offset) {
                        if self.candidate_is_better_than(window_candidate, found) {
                            self.diagnostics
                                .borrow_mut()
                                .record_repeat_best_before_window_overridden_by_window(
                                    found.source_repeat_kind(),
                                );
                        }
                    }
                    if candidate
                        .map(|current| self.candidate_is_better_than(window_candidate, current))
                        .unwrap_or(true)
                    {
                        candidate = Some(window_candidate);
                    }
                }
                if self.prefer_binary_next_position_lookahead {
                    candidate = self.prefer_next_position_window_candidate(candidate, block_len);
                }
            }

            candidate = self.prefer_lockfile_zero_literal_next_position_candidate(
                candidate,
                previous_window_len,
                block_len,
            );

            if let Some(candidate) = candidate {
                self.emit_candidate(candidate, &mut handle_sequence);
                return true;
            }

            let suffix_idx = self.suffix_idx;
            let probe_step = self.no_match_probe_step(suffix_idx);
            let can_skip_next_probe = suffix_idx + probe_step + MIN_MATCH_LEN <= block_len
                && (1..probe_step).all(|skip| {
                    !self.repeat_offset_can_match_at(suffix_idx + skip, previous_window_len)
                });
            self.add_suffix_at(suffix_idx);
            let step = if can_skip_next_probe {
                for skip in 1..probe_step {
                    self.add_suffix_at(suffix_idx + skip);
                }
                probe_step
            } else {
                1
            };
            self.suffix_idx += step;
        }
    }

    #[inline(never)]
    pub(super) fn next_sequence_best_text(
        &mut self,
        mut handle_sequence: impl for<'a> FnMut(Sequence<'a>),
    ) -> bool {
        loop {
            let block_len = self.last_entry().data.len();

            if self.suffix_idx >= block_len {
                if self.last_idx_in_sequence != self.suffix_idx {
                    let last_entry_idx = self.last_entry_index();
                    let literals = &self.window[last_entry_idx].data[self.last_idx_in_sequence..];
                    self.last_idx_in_sequence = self.suffix_idx;
                    handle_sequence(Sequence::Literals { literals });
                    return true;
                } else {
                    return false;
                }
            }

            let last_entry_idx = self.last_entry_index();
            let last_entry = &self.window[last_entry_idx];
            let data_slice = &last_entry.data[self.suffix_idx..];
            if data_slice.len() < MIN_MATCH_LEN {
                let last_idx_in_sequence = self.last_idx_in_sequence;
                self.last_idx_in_sequence = block_len;
                self.suffix_idx = block_len;
                handle_sequence(Sequence::Literals {
                    literals: &last_entry.data[last_idx_in_sequence..],
                });
                return true;
            }

            let key_value = SuffixStore::key_value(&data_slice[..MIN_MATCH_LEN]);
            let match_context = MatchCandidateContext {
                suffix_idx: self.suffix_idx,
                anchor_idx: self.last_idx_in_sequence,
                min_non_repeat_match_len: self.min_non_repeat_match_len,
                data_slice,
                #[cfg(debug_assertions)]
                last_entry_len: block_len,
                #[cfg(debug_assertions)]
                concat_window: &self.concat_window,
            };

            let previous_window_len = self.window_size - block_len;
            let mut candidate =
                self.best_text_repeat_candidate(&match_context, previous_window_len, block_len);

            #[cfg(test)]
            if let Some(found) = candidate.filter(|found| found.repeat_offset) {
                self.diagnostics
                    .borrow_mut()
                    .record_repeat_best_before_window(
                        found.source_repeat_kind(),
                        found.start_idx == self.last_idx_in_sequence,
                    );
            }
            let repeat_match_reaches_end_or_is_long = candidate
                .is_some_and(|found| self.repeat_match_can_skip_window_search(found, block_len));
            if !repeat_match_reaches_end_or_is_long {
                if let Some(window_candidate) =
                    self.best_window_candidate(key_value, &match_context, block_len)
                {
                    #[cfg(test)]
                    if let Some(found) = candidate.filter(|found| found.repeat_offset) {
                        if self.candidate_is_better_than(window_candidate, found) {
                            self.diagnostics
                                .borrow_mut()
                                .record_repeat_best_before_window_overridden_by_window(
                                    found.source_repeat_kind(),
                                );
                        }
                    }
                    if candidate
                        .map(|current| self.candidate_is_better_than(window_candidate, current))
                        .unwrap_or(true)
                    {
                        candidate = Some(window_candidate);
                    }
                }
                if self.prefer_binary_next_position_lookahead {
                    candidate = self.prefer_next_position_window_candidate(candidate, block_len);
                }
            }

            candidate = self.prefer_lockfile_zero_literal_next_position_candidate(
                candidate,
                previous_window_len,
                block_len,
            );

            if let Some(candidate) = candidate {
                self.emit_candidate(candidate, &mut handle_sequence);
                return true;
            }

            let suffix_idx = self.suffix_idx;
            let probe_step = self.no_match_probe_step(suffix_idx);
            let can_skip_next_probe = suffix_idx + probe_step + MIN_MATCH_LEN <= block_len
                && (1..probe_step).all(|skip| {
                    !self.repeat_offset_can_match_at(suffix_idx + skip, previous_window_len)
                });
            self.add_suffix_at(suffix_idx);
            let step = if can_skip_next_probe {
                for skip in 1..probe_step {
                    self.add_suffix_at(suffix_idx + skip);
                }
                probe_step
            } else {
                1
            };
            self.suffix_idx += step;
        }
    }

    #[inline(always)]
    pub(super) fn has_min_match_at_index_offset(&self, suffix_idx: usize, offset: usize) -> bool {
        let source_relative = suffix_idx as isize - offset as isize;
        let Some(source) = self.slice_at_relative(source_relative) else {
            return false;
        };

        if source.len() < MIN_MATCH_LEN {
            return true;
        }

        source[..MIN_MATCH_LEN] == self.last_entry().data[suffix_idx..suffix_idx + MIN_MATCH_LEN]
    }

    #[inline(always)]
    pub(super) fn repeat_offset_candidates(
        &self,
        literal_len: usize,
    ) -> [(RepeatCandidateKind, usize); 3] {
        if literal_len > 0 {
            [
                (
                    RepeatCandidateKind::First,
                    self.offset_history.newest as usize,
                ),
                (
                    RepeatCandidateKind::Second,
                    self.offset_history.second as usize,
                ),
                (
                    RepeatCandidateKind::Third,
                    self.offset_history.third as usize,
                ),
            ]
        } else {
            [
                (
                    RepeatCandidateKind::Second,
                    self.offset_history.second as usize,
                ),
                (
                    RepeatCandidateKind::Third,
                    self.offset_history.third as usize,
                ),
                (
                    RepeatCandidateKind::First,
                    self.offset_history.newest.saturating_sub(1) as usize,
                ),
            ]
        }
    }

    #[inline(always)]
    pub(super) fn has_min_match_at_index(
        match_entry: &WindowEntry,
        match_index: usize,
        context: &MatchCandidateContext<'_>,
    ) -> bool {
        match_entry
            .data
            .get(match_index..match_index + MIN_MATCH_LEN)
            .is_some_and(|source| source == &context.data_slice[..MIN_MATCH_LEN])
    }

    #[inline(always)]
    pub(super) fn has_long_match_at_index(
        match_entry: &WindowEntry,
        match_index: usize,
        context: &MatchCandidateContext<'_>,
    ) -> bool {
        match_entry
            .data
            .get(match_index..match_index + 8)
            .is_some_and(|source| source == &context.data_slice[..8])
    }
}
