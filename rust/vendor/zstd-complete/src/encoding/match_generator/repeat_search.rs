use super::*;

impl MatchGenerator {
    #[inline(never)]
    pub(super) fn best_text_repeat_candidate(
        &self,
        context: &MatchCandidateContext<'_>,
        previous_window_len: usize,
        block_len: usize,
    ) -> Option<MatchCandidate> {
        let literal_len = context.suffix_idx - context.anchor_idx;
        let allow_next_position = context.suffix_idx + 1 + MIN_MATCH_LEN <= block_len;
        let next_context = if allow_next_position {
            let next_slice = &context.data_slice[1..];
            Some(MatchCandidateContext {
                suffix_idx: context.suffix_idx + 1,
                anchor_idx: context.anchor_idx,
                min_non_repeat_match_len: context.min_non_repeat_match_len,
                data_slice: next_slice,
                #[cfg(debug_assertions)]
                last_entry_len: block_len,
                #[cfg(debug_assertions)]
                concat_window: context.concat_window,
            })
        } else {
            None
        };

        let mut current_candidate = None;
        let mut next_candidate = None;

        for (repeat_idx, &(_repeat_kind, offset)) in self
            .repeat_offset_candidates(literal_len)
            .iter()
            .enumerate()
        {
            if !self.allow_repeat_candidate(literal_len, repeat_idx) {
                continue;
            }
            if Self::repeat_offset_is_available(offset, previous_window_len, context.suffix_idx) {
                let Some(verified_prefix_len) = self.verified_min_match_prefix_len(offset, context)
                else {
                    continue;
                };
                let match_len =
                    self.match_len_at_offset_with_prefix(offset, context, verified_prefix_len);
                if match_len >= MIN_MATCH_LEN {
                    let found = MatchCandidate {
                        start_idx: context.suffix_idx,
                        offset,
                        match_len,
                        repeat_offset: true,
                        #[cfg(test)]
                        source: CandidateSource::RepeatCurrent(_repeat_kind),
                    };
                    if current_candidate
                        .map(|current| self.candidate_is_better_than(found, current))
                        .unwrap_or(true)
                    {
                        current_candidate = Some(found);
                    }
                }
            }

            let Some(next_context) = next_context.as_ref() else {
                continue;
            };
            if current_candidate.is_some() {
                continue;
            }
            if !Self::repeat_offset_is_available(
                offset,
                previous_window_len,
                next_context.suffix_idx,
            ) {
                continue;
            }
            let Some(verified_prefix_len) =
                self.verified_min_match_prefix_len(offset, next_context)
            else {
                continue;
            };
            let match_len =
                self.match_len_at_offset_with_prefix(offset, next_context, verified_prefix_len);
            if match_len < MIN_MATCH_LEN {
                continue;
            }
            let found = MatchCandidate {
                start_idx: next_context.suffix_idx,
                offset,
                match_len,
                repeat_offset: true,
                #[cfg(test)]
                source: CandidateSource::RepeatCurrent(_repeat_kind),
            };
            if next_candidate
                .map(|current| self.candidate_is_better_than(found, current))
                .unwrap_or(true)
            {
                next_candidate = Some(found);
            }
        }

        #[cfg(test)]
        let next_candidate = next_candidate.map(|candidate| MatchCandidate {
            source: CandidateSource::RepeatNextPosition(candidate.source_repeat_kind()),
            ..candidate
        });

        current_candidate.or(next_candidate)
    }

    #[inline(always)]
    pub(super) fn best_repeat_candidate_at(
        &self,
        suffix_idx: usize,
        anchor_idx: usize,
        previous_window_len: usize,
        _block_len: usize,
    ) -> Option<MatchCandidate> {
        let last_entry = self.last_entry();
        let data_slice = last_entry.data.get(suffix_idx..)?;
        if data_slice.len() < MIN_MATCH_LEN {
            return None;
        }

        let context = MatchCandidateContext {
            suffix_idx,
            anchor_idx,
            min_non_repeat_match_len: self.min_non_repeat_match_len,
            data_slice,
            #[cfg(debug_assertions)]
            last_entry_len: _block_len,
            #[cfg(debug_assertions)]
            concat_window: &self.concat_window,
        };
        let literal_len = suffix_idx - anchor_idx;
        let mut candidate = None;
        for (repeat_idx, &(_repeat_kind, offset)) in self
            .repeat_offset_candidates(literal_len)
            .iter()
            .enumerate()
        {
            if !self.allow_repeat_candidate(literal_len, repeat_idx) {
                continue;
            }
            if !Self::repeat_offset_is_available(offset, previous_window_len, suffix_idx) {
                continue;
            }
            let Some(verified_prefix_len) = self.verified_min_match_prefix_len(offset, &context)
            else {
                continue;
            };
            let match_len =
                self.match_len_at_offset_with_prefix(offset, &context, verified_prefix_len);
            if match_len < MIN_MATCH_LEN {
                continue;
            }
            let found = MatchCandidate {
                start_idx: suffix_idx,
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

        candidate
    }

    #[inline(always)]
    pub(super) fn prefer_next_position_window_candidate(
        &self,
        candidate: Option<MatchCandidate>,
        block_len: usize,
    ) -> Option<MatchCandidate> {
        if !self.prefer_binary_next_position_lookahead
            || self.min_non_repeat_match_len != MIN_MATCH_LEN
            || self.suffix_idx + 1 + MIN_MATCH_LEN > block_len
        {
            return candidate;
        }

        let should_probe = match candidate {
            None => true,
            Some(current) => {
                !current.repeat_offset
                    && current.start_idx == self.suffix_idx
                    && current.match_len == MIN_MATCH_LEN
            }
        };
        if !should_probe {
            return candidate;
        }

        let last_entry = self.last_entry();
        let next_slice = &last_entry.data[self.suffix_idx + 1..];
        let next_context = MatchCandidateContext {
            suffix_idx: self.suffix_idx + 1,
            anchor_idx: self.last_idx_in_sequence,
            min_non_repeat_match_len: self.min_non_repeat_match_len,
            data_slice: next_slice,
            #[cfg(debug_assertions)]
            last_entry_len: block_len,
            #[cfg(debug_assertions)]
            concat_window: &self.concat_window,
        };
        let key_value = SuffixStore::key_value(&next_slice[..MIN_MATCH_LEN]);
        let Some(next_candidate) = self.best_window_candidate(key_value, &next_context, block_len)
        else {
            return candidate;
        };
        #[cfg(test)]
        let next_candidate = MatchCandidate {
            source: match next_candidate.source {
                CandidateSource::WindowCurrentNewest { entry_distance } => {
                    CandidateSource::WindowNextPositionNewest { entry_distance }
                }
                CandidateSource::WindowCurrentSecondNewest { entry_distance } => {
                    CandidateSource::WindowNextPositionSecondNewest { entry_distance }
                }
                CandidateSource::WindowCurrentOldest { entry_distance } => {
                    CandidateSource::WindowNextPositionOldest { entry_distance }
                }
                other => other,
            },
            ..next_candidate
        };

        match candidate {
            None if !next_candidate.repeat_offset => Some(next_candidate),
            Some(current)
                if !next_candidate.repeat_offset
                    && self.candidate_is_better_than(next_candidate, current) =>
            {
                Some(next_candidate)
            }
            Some(current) => Some(current),
            None => None,
        }
    }

    #[inline(always)]
    pub(super) fn lockfile_next_position_tuning(
        &self,
    ) -> Option<(usize, usize, usize, usize, usize, usize, usize)> {
        if !self.current_block_is_dictionary_lockfile_text {
            return None;
        }
        #[cfg(feature = "std")]
        {
            let overrides = matcher_tuning_overrides();
            let max_skip_literals = overrides
                .lockfile_next_position_max_skip_literals
                .unwrap_or(LOCKFILE_NEXT_POSITION_MAX_SKIP_LITERALS);
            let max_current_match_len = overrides
                .lockfile_next_position_max_current_match_len
                .unwrap_or(LOCKFILE_NEXT_POSITION_MAX_CURRENT_MATCH_LEN);
            let max_match_loss = overrides
                .lockfile_next_position_match_loss_max
                .unwrap_or(LOCKFILE_NEXT_POSITION_MATCH_LOSS_MAX);
            let literal_weight = overrides
                .lockfile_next_position_literal_weight
                .unwrap_or(LOCKFILE_NEXT_POSITION_LITERAL_WEIGHT);
            let match_reward = overrides
                .lockfile_next_position_match_reward
                .unwrap_or(LOCKFILE_NEXT_POSITION_MATCH_REWARD);
            let offset_weight = overrides
                .lockfile_next_position_offset_weight
                .unwrap_or(LOCKFILE_NEXT_POSITION_OFFSET_WEIGHT);
            let margin = overrides
                .lockfile_next_position_margin
                .unwrap_or(LOCKFILE_NEXT_POSITION_MARGIN);
            return Some((
                max_skip_literals.max(1),
                max_current_match_len.max(MIN_MATCH_LEN),
                max_match_loss,
                literal_weight.max(1),
                match_reward.max(1),
                offset_weight.max(1),
                margin,
            ));
        }
        #[allow(unreachable_code)]
        Some((
            LOCKFILE_NEXT_POSITION_MAX_SKIP_LITERALS,
            LOCKFILE_NEXT_POSITION_MAX_CURRENT_MATCH_LEN,
            LOCKFILE_NEXT_POSITION_MATCH_LOSS_MAX,
            LOCKFILE_NEXT_POSITION_LITERAL_WEIGHT,
            LOCKFILE_NEXT_POSITION_MATCH_REWARD,
            LOCKFILE_NEXT_POSITION_OFFSET_WEIGHT,
            LOCKFILE_NEXT_POSITION_MARGIN,
        ))
    }

    #[inline(always)]
    pub(super) fn prefer_lockfile_zero_literal_next_position_candidate(
        &self,
        candidate: Option<MatchCandidate>,
        previous_window_len: usize,
        block_len: usize,
    ) -> Option<MatchCandidate> {
        let Some((
            max_skip_literals,
            max_current_match_len,
            max_match_loss,
            literal_weight,
            match_reward,
            offset_weight,
            margin,
        )) = self.lockfile_next_position_tuning()
        else {
            return candidate;
        };
        let Some(current) = candidate else {
            return candidate;
        };
        if !self.current_block_is_dictionary_lockfile_text
            || self.suffix_idx != self.last_idx_in_sequence
            || current.start_idx != self.suffix_idx
            || current.match_len > max_current_match_len
        {
            return Some(current);
        }
        let current_cost = self.lockfile_estimated_local_path_cost(
            current,
            self.last_idx_in_sequence,
            literal_weight,
            match_reward,
            offset_weight,
        );
        let mut best_candidate = current;
        let mut best_cost = current_cost;

        for skip in 1..=max_skip_literals {
            if self.suffix_idx + skip + MIN_MATCH_LEN > block_len {
                break;
            }
            let Some(next_candidate) = self.best_candidate_at_position(
                self.suffix_idx + skip,
                self.last_idx_in_sequence,
                previous_window_len,
                block_len,
            ) else {
                continue;
            };
            if next_candidate.start_idx != self.suffix_idx + skip {
                continue;
            }
            if next_candidate.match_len + max_match_loss < current.match_len {
                continue;
            }

            let next_cost = self.lockfile_estimated_local_path_cost(
                next_candidate,
                self.last_idx_in_sequence,
                literal_weight,
                match_reward,
                offset_weight,
            );
            if next_cost + margin < best_cost {
                best_candidate = next_candidate;
                best_cost = next_cost;
            }
        }

        Some(best_candidate)
    }

    #[inline(always)]
    pub(super) fn best_candidate_at_position(
        &self,
        suffix_idx: usize,
        anchor_idx: usize,
        previous_window_len: usize,
        block_len: usize,
    ) -> Option<MatchCandidate> {
        let last_entry = self.last_entry();
        let data_slice = last_entry.data.get(suffix_idx..)?;
        if data_slice.len() < MIN_MATCH_LEN {
            return None;
        }

        let context = MatchCandidateContext {
            suffix_idx,
            anchor_idx,
            min_non_repeat_match_len: self.min_non_repeat_match_len,
            data_slice,
            #[cfg(debug_assertions)]
            last_entry_len: block_len,
            #[cfg(debug_assertions)]
            concat_window: &self.concat_window,
        };

        let mut candidate =
            self.best_repeat_candidate_at(suffix_idx, anchor_idx, previous_window_len, block_len);
        let key_value = SuffixStore::key_value(&data_slice[..MIN_MATCH_LEN]);
        if let Some(window_candidate) = self.best_window_candidate(key_value, &context, block_len) {
            if candidate
                .map(|current| self.candidate_is_better_than(window_candidate, current))
                .unwrap_or(true)
            {
                candidate = Some(window_candidate);
            }
        }
        candidate
    }

    #[inline(always)]
    pub(super) fn lockfile_estimated_local_parse_cost_with_history(
        &self,
        candidate: MatchCandidate,
        anchor_idx: usize,
        offset_history: &mut OffsetHistory,
        literal_weight: usize,
        match_reward: usize,
        offset_weight: usize,
    ) -> usize {
        let literal_len = candidate.start_idx.saturating_sub(anchor_idx);
        let offset_value =
            offset_history.encode_offset_value(candidate.offset as u32, literal_len as u32);
        let ll_num_bits = Self::local_literal_length_extra_bits(literal_len as u32);
        let ml_num_bits = Self::local_match_length_extra_bits(candidate.match_len as u32);
        let of_code = Self::local_offset_code(offset_value);
        let of_num_bits = of_code as usize;

        let explicit_bits = ll_num_bits + ml_num_bits + of_num_bits;
        let symbol_penalty = of_code as usize * offset_weight;
        let literal_penalty = candidate.start_idx.saturating_sub(anchor_idx) * literal_weight;
        let match_credit = candidate.match_len * match_reward;

        literal_penalty
            .saturating_add(explicit_bits)
            .saturating_add(symbol_penalty)
            .saturating_sub(match_credit)
    }

    #[inline(always)]
    pub(super) fn lockfile_estimated_local_path_cost(
        &self,
        candidate: MatchCandidate,
        anchor_idx: usize,
        literal_weight: usize,
        match_reward: usize,
        offset_weight: usize,
    ) -> usize {
        let mut offset_history = self.offset_history;
        self.lockfile_estimated_local_parse_cost_with_history(
            candidate,
            anchor_idx,
            &mut offset_history,
            literal_weight,
            match_reward,
            offset_weight,
        )
    }
}
