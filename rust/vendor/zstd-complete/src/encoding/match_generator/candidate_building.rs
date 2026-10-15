use super::*;

impl MatchGenerator {
    #[inline(always)]
    pub(super) fn consider_window_candidate(
        &self,
        match_entry: &WindowEntry,
        match_index: usize,
        context: &MatchCandidateContext<'_>,
        meta: WindowCandidateMeta,
        candidate: &mut Option<MatchCandidate>,
        block_len: usize,
    ) -> bool {
        let Some(found) = self.match_candidate(match_entry, match_index, context, meta) else {
            return false;
        };
        if self.reject_composer_window_candidate(found) {
            return false;
        }
        if self.reject_lockfile_zero_literal_window_candidate(found, context) {
            return false;
        }
        if !found.worth_emitting(context.min_non_repeat_match_len) {
            return false;
        }

        let improved = candidate
            .map(|current| self.window_candidate_is_better_than(found, current, meta))
            .unwrap_or(true);
        if improved {
            *candidate = Some(found);
        }

        found.start_idx + found.match_len == block_len
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn consider_window_candidate_with_tracking(
        &self,
        match_entry: &WindowEntry,
        match_index: usize,
        context: &MatchCandidateContext<'_>,
        meta: WindowCandidateMeta,
        candidate: &mut Option<MatchCandidate>,
        block_len: usize,
        _current_long_hash_active: bool,
    ) -> bool {
        let Some(found) = self.match_candidate(match_entry, match_index, context, meta) else {
            return false;
        };
        if self.reject_composer_window_candidate(found) {
            return false;
        }
        if self.reject_lockfile_zero_literal_window_candidate(found, context) {
            return false;
        }
        if !found.worth_emitting(context.min_non_repeat_match_len) {
            return false;
        }

        let improved = candidate
            .map(|current| self.window_candidate_is_better_than(found, current, meta))
            .unwrap_or(true);
        if improved {
            *candidate = Some(found);
        }

        let should_break = found.start_idx + found.match_len == block_len;

        #[cfg(test)]
        if _current_long_hash_active {
            if improved {
                if let Some(selected) = candidate {
                    self.diagnostics
                        .borrow_mut()
                        .record_current_long_hash_improvement(*selected);
                }
            }
            if should_break {
                self.diagnostics
                    .borrow_mut()
                    .record_current_long_hash_end_break(found, improved);
            }
        }

        should_break
    }

    #[inline(always)]
    pub(super) fn match_candidate(
        &self,
        match_entry: &WindowEntry,
        match_index: usize,
        context: &MatchCandidateContext<'_>,
        _meta: WindowCandidateMeta,
    ) -> Option<MatchCandidate> {
        if !Self::has_min_match_at_index(match_entry, match_index, context) {
            return None;
        }

        let offset = match_entry.base_offset + context.suffix_idx - match_index;
        let match_len = self.match_len_at_offset_with_prefix(offset, context, MIN_MATCH_LEN);
        if match_len < MIN_MATCH_LEN {
            return None;
        }
        let (start_idx, match_len) = self.extend_match_backwards(offset, match_len, context);

        #[cfg(debug_assertions)]
        {
            let unprocessed = context.last_entry_len - context.suffix_idx;
            let current_start = context.concat_window.len() - unprocessed;
            let current_match_start = current_start - (context.suffix_idx - start_idx);
            let match_start = current_match_start - offset;
            let match_end = match_start + match_len;
            let check_slice = &context.concat_window[match_start..match_end];
            let current_end = start_idx + match_len;
            debug_assert_eq!(check_slice, &self.last_entry().data[start_idx..current_end]);
        }

        Some(MatchCandidate {
            start_idx,
            offset,
            match_len,
            repeat_offset: false,
            #[cfg(test)]
            source: match _meta.kind {
                WindowCandidateKind::Newest => CandidateSource::WindowCurrentNewest {
                    entry_distance: _meta.entry_distance,
                },
                WindowCandidateKind::SecondNewest => CandidateSource::WindowCurrentSecondNewest {
                    entry_distance: _meta.entry_distance,
                },
                WindowCandidateKind::Oldest => CandidateSource::WindowCurrentOldest {
                    entry_distance: _meta.entry_distance,
                },
            },
        })
    }

    #[inline(always)]
    pub(super) fn reject_lockfile_zero_literal_window_candidate(
        &self,
        found: MatchCandidate,
        context: &MatchCandidateContext<'_>,
    ) -> bool {
        #[cfg(feature = "std")]
        {
            let zero_literals = context.suffix_idx == context.anchor_idx;
            if matches!(
                matcher_tuning_overrides().lockfile_zero_literal_window_disable,
                Some(true)
            ) {
                return self.current_block_is_dictionary_lockfile_text
                    && !found.repeat_offset
                    && zero_literals;
            }
            let Some(max_match_len) =
                matcher_tuning_overrides().lockfile_zero_literal_window_max_match_len
            else {
                return false;
            };
            let Some(min_offset_bits) =
                matcher_tuning_overrides().lockfile_zero_literal_window_min_offset_bits
            else {
                return false;
            };
            self.current_block_is_dictionary_lockfile_text
                && !found.repeat_offset
                && zero_literals
                && found.match_len <= max_match_len
                && self.non_repeat_offset_code_bits(found.offset) >= min_offset_bits
        }
        #[cfg(not(feature = "std"))]
        {
            let _ = (found, context);
            false
        }
    }

    #[inline(always)]
    pub(super) fn reject_composer_window_candidate(&self, found: MatchCandidate) -> bool {
        #[cfg(feature = "std")]
        {
            matches!(
                matcher_tuning_overrides().composer_window_disable,
                Some(true)
            ) && self.current_block_is_composer_dictionary_text
                && !found.repeat_offset
        }
        #[cfg(not(feature = "std"))]
        {
            let _ = found;
            false
        }
    }

    #[inline(always)]
    pub(super) fn allow_repeat_candidate(&self, literal_len: usize, repeat_idx: usize) -> bool {
        #[cfg(not(feature = "std"))]
        let _ = repeat_idx;

        if self.current_block_is_composer_dictionary_text && literal_len == 0 {
            #[cfg(feature = "std")]
            if let Some(limit) =
                matcher_tuning_overrides().composer_zero_literal_repeat_candidate_limit
            {
                return repeat_idx < limit;
            }
        }
        true
    }

    #[inline(always)]
    pub(super) fn repeat_offset_can_match_at(
        &self,
        suffix_idx: usize,
        previous_window_len: usize,
    ) -> bool {
        let literal_len = suffix_idx - self.last_idx_in_sequence;
        for (_, offset) in self.repeat_offset_candidates(literal_len) {
            if Self::repeat_offset_is_available(offset, previous_window_len, suffix_idx)
                && self.has_min_match_at_index_offset(suffix_idx, offset)
            {
                return true;
            }
        }
        false
    }

    #[inline(always)]
    pub(super) fn repeat_offset_is_available(
        offset: usize,
        previous_window_len: usize,
        suffix_idx: usize,
    ) -> bool {
        offset != 0 && offset <= previous_window_len + suffix_idx
    }

    #[inline(always)]
    pub(super) fn no_match_probe_step(&self, suffix_idx: usize) -> usize {
        if self.is_text_block {
            if matches!(self.file_type_hint, CompressionFileType::DictionaryText) {
                if self.uses_dictionary_lockfile_second_newest_path() {
                    #[cfg(feature = "std")]
                    if let Some(value) = matcher_tuning_overrides().lockfile_probe_step {
                        return value;
                    }
                    return SHORT_LINE_TEXT_NO_MATCH_PROBE_STEP;
                }
                if self.current_block_is_composer_dictionary_text {
                    #[cfg(feature = "std")]
                    if let Some(value) = matcher_tuning_overrides().composer_probe_step {
                        return value;
                    }
                    return COMPOSER_JSON_LOCKFILE_NO_MATCH_PROBE_STEP;
                }
                return 1;
            }
            if self.is_short_line_text {
                match self.file_type_hint {
                    CompressionFileType::CodeText
                        if self.last_entry().data.len() <= CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN =>
                    {
                        return 1;
                    }
                    CompressionFileType::ConfigText
                        if self.current_block_is_tsconfig_json_config_text =>
                    {
                        #[cfg(feature = "std")]
                        if let Some(value) = matcher_tuning_overrides().tsconfig_probe_step {
                            return value;
                        }
                        return TSCONFIG_JSON_TEXT_NO_MATCH_PROBE_STEP;
                    }
                    CompressionFileType::ConfigText
                        if self.current_block_is_structured_json_config_text
                            && self.last_entry().data.len()
                                <= STRUCTURED_JSON_CONFIG_DENSE_PROBE_MAX_BLOCK_LEN =>
                    {
                        #[cfg(feature = "std")]
                        if let Some(value) = matcher_tuning_overrides().structured_json_probe_step {
                            return value;
                        }
                        return 1;
                    }
                    CompressionFileType::ConfigText
                        if self.last_entry().data.len()
                            <= CONFIG_TEXT_DENSE_PROBE_MAX_BLOCK_LEN =>
                    {
                        return 1;
                    }
                    _ => {}
                }
            }
            if self.is_short_line_text {
                return SHORT_LINE_TEXT_NO_MATCH_PROBE_STEP;
            }
            return TEXT_NO_MATCH_PROBE_STEP;
        }

        if self.use_fast_small_dense_binary_probe
            && self.last_entry().data.len() <= FASTEST_DENSE_BINARY_PROBE_MAX_BLOCK_LEN
        {
            return 1;
        }

        let base_step = NO_MATCH_PROBE_STEP;
        if !self.adaptive_binary_no_match_probe {
            return base_step;
        }

        let literal_run_len = suffix_idx - self.last_idx_in_sequence;
        base_step + (literal_run_len >> BEST_BINARY_NO_MATCH_SEARCH_STRENGTH)
    }
}
