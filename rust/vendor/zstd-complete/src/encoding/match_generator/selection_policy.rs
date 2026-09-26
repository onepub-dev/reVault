use super::*;

impl MatchGenerator {
    #[inline(always)]
    pub(super) fn should_track_second_newest_for_current_entry(&self) -> bool {
        (self.use_second_newest_probe
            || self.uses_dictionary_current_entry_second_newest_path()
            || self.uses_code_text_current_entry_second_newest_path()
            || self.uses_dictionary_lockfile_second_newest_path()
            || (self.use_fast_binary_small_second_newest
                && !self.is_text_block
                && (self.last_entry().data.len() <= FASTEST_SECOND_NEWEST_MAX_BLOCK_LEN
                    || (matches!(self.file_type_hint, CompressionFileType::Unknown)
                        && self.last_entry().data.len()
                            <= FASTEST_UNKNOWN_SECOND_NEWEST_MAX_BLOCK_LEN))))
            && (self.min_non_repeat_match_len == MIN_MATCH_LEN
                || self.uses_code_text_current_entry_second_newest_path())
            && (self.last_entry().data.len() >= BEST_SECOND_NEWEST_MIN_BLOCK_LEN
                || self.use_fast_binary_small_second_newest)
    }

    #[inline(always)]
    pub(super) fn should_track_current_long_hash(&self) -> bool {
        self.use_second_newest_probe
            && self.min_non_repeat_match_len == MIN_MATCH_LEN
            && self.last_entry().data.len() >= BEST_CURRENT_LONG_HASH_MIN_BLOCK_LEN
    }

    #[inline(always)]
    pub(super) fn use_best_text_repeat_pipeline(&self) -> bool {
        self.is_text_block
            && (self.use_text_repeat_pipeline
                || matches!(self.file_type_hint, CompressionFileType::DictionaryText))
    }

    #[inline(always)]
    pub(super) fn uses_dictionary_current_entry_second_newest_path(&self) -> bool {
        matches!(self.file_type_hint, CompressionFileType::DictionaryText)
            && self.is_text_block
            && !self.current_block_is_dictionary_lockfile_text
            && self.last_entry().data.len() >= BEST_SECOND_NEWEST_MIN_BLOCK_LEN
    }

    #[inline(always)]
    pub(super) fn uses_code_text_current_entry_second_newest_path(&self) -> bool {
        matches!(self.file_type_hint, CompressionFileType::CodeText)
            && self.is_short_line_text
            && self.last_entry().data.len() >= BEST_SECOND_NEWEST_MIN_BLOCK_LEN
            && self.last_entry().data.len() <= CODE_TEXT_SECOND_NEWEST_MAX_BLOCK_LEN
    }

    #[inline(always)]
    pub(super) fn uses_dictionary_lockfile_second_newest_path(&self) -> bool {
        self.current_block_is_dictionary_lockfile_text
    }

    #[inline(always)]
    pub(super) fn prefer_lockfile_second_newest_before_newest(&self) -> bool {
        self.uses_dictionary_lockfile_second_newest_path()
    }

    #[inline(always)]
    pub(super) fn use_complementary_end_insertion_for_current_block(&self) -> bool {
        self.use_complementary_end_insertion
    }

    #[inline(always)]
    pub(super) fn uses_large_unknown_fastest_path(&self) -> bool {
        self.prefer_fast_binary_next_position_repeat_lookahead
            && matches!(self.file_type_hint, CompressionFileType::Unknown)
            && !self.is_text_block
            && self.min_non_repeat_match_len == MIN_MATCH_LEN
            && self.last_entry().data.len() >= FASTEST_UNKNOWN_SECOND_NEWEST_MAX_BLOCK_LEN
    }

    #[inline(always)]
    pub(super) fn repeat_match_len_margin(&self) -> usize {
        let mut margin = REPEAT_MATCH_LEN_MARGIN;
        if self.uses_large_unknown_fastest_path() {
            margin += 3;
        }
        margin
    }

    #[inline(always)]
    pub(super) fn candidate_is_better_than(
        &self,
        found: MatchCandidate,
        current: MatchCandidate,
    ) -> bool {
        if matches!(self.file_type_hint, CompressionFileType::DictionaryText) {
            if self.prefer_lockfile_smaller_offset_same_end(found, current) {
                return true;
            }
            if self.prefer_lockfile_smaller_offset_same_end(current, found) {
                return false;
            }
            if self.prefer_lockfile_repeat_kind_same_start(found, current) {
                return true;
            }
            if self.prefer_lockfile_repeat_kind_same_start(current, found) {
                return false;
            }
            if self.prefer_composer_repeat_kind_same_start(found, current) {
                return true;
            }
            if self.prefer_composer_repeat_kind_same_start(current, found) {
                return false;
            }
            if self.prefer_dictionary_smaller_offset_same_start(found, current) {
                return true;
            }
            if self.prefer_dictionary_smaller_offset_same_start(current, found) {
                return false;
            }
        }
        if self.uses_large_unknown_fastest_path() {
            if self.prefer_large_unknown_smaller_offset(found, current) {
                return true;
            }
            if self.prefer_large_unknown_smaller_offset(current, found) {
                return false;
            }
        }

        if found.repeat_offset != current.repeat_offset {
            let margin = self.repeat_match_len_margin();
            if found.repeat_offset {
                return found.match_len + margin >= current.match_len;
            }
            return found.match_len > current.match_len + margin;
        }

        found.match_len > current.match_len
            || (found.match_len == current.match_len
                && (found.start_idx < current.start_idx
                    || (found.start_idx == current.start_idx && found.offset < current.offset)))
    }

    #[inline(always)]
    pub(super) fn window_candidate_is_better_than(
        &self,
        found: MatchCandidate,
        current: MatchCandidate,
        meta: WindowCandidateMeta,
    ) -> bool {
        if self.keep_lockfile_current_candidate_over_oldest(found, current, meta) {
            return false;
        }
        if self.keep_large_unknown_current_candidate_over_newest(found, current, meta) {
            return false;
        }
        if self.keep_large_unknown_current_candidate_over_oldest(found, current, meta) {
            return false;
        }
        self.candidate_is_better_than(found, current)
    }

    pub(super) fn keep_lockfile_current_candidate_over_oldest(
        &self,
        found: MatchCandidate,
        current: MatchCandidate,
        meta: WindowCandidateMeta,
    ) -> bool {
        self.uses_dictionary_lockfile_second_newest_path()
            && matches!(meta.kind, WindowCandidateKind::Oldest)
            && !found.repeat_offset
            && !current.repeat_offset
            && current.offset < found.offset
            && found.match_len < current.match_len + LOCKFILE_OLDEST_DISPLACEMENT_MIN_GAIN
    }

    #[inline(always)]
    pub(super) fn keep_large_unknown_current_candidate_over_newest(
        &self,
        found: MatchCandidate,
        current: MatchCandidate,
        meta: WindowCandidateMeta,
    ) -> bool {
        self.uses_large_unknown_fastest_path()
            && matches!(meta.kind, WindowCandidateKind::Newest)
            && !found.repeat_offset
            && !current.repeat_offset
            && current.offset < found.offset
            && found.match_len < current.match_len + LARGE_UNKNOWN_NEWEST_DISPLACEMENT_MIN_GAIN
    }

    #[inline(always)]
    pub(super) fn keep_large_unknown_current_candidate_over_oldest(
        &self,
        found: MatchCandidate,
        current: MatchCandidate,
        meta: WindowCandidateMeta,
    ) -> bool {
        self.uses_large_unknown_fastest_path()
            && matches!(meta.kind, WindowCandidateKind::Oldest)
            && !found.repeat_offset
            && !current.repeat_offset
            && current.offset < found.offset
            && found.match_len < current.match_len + LARGE_UNKNOWN_OLDEST_DISPLACEMENT_MIN_GAIN
    }

    #[inline(always)]
    pub(super) fn dictionary_same_start_bits_gain_min(&self) -> usize {
        #[cfg(feature = "std")]
        if let Some(value) = matcher_tuning_overrides().dictionary_same_start_bits_gain_min {
            return value;
        }
        DICTIONARY_SMALLER_OFFSET_BITS_GAIN_MIN
    }

    #[inline(always)]
    pub(super) fn dictionary_same_start_match_loss_max(&self) -> usize {
        #[cfg(feature = "std")]
        if let Some(value) = matcher_tuning_overrides().dictionary_same_start_match_loss_max {
            return value;
        }
        DICTIONARY_SMALLER_OFFSET_MATCH_LOSS_MAX
    }

    #[inline(always)]
    pub(super) fn lockfile_same_end_bits_gain_min(&self) -> usize {
        #[cfg(feature = "std")]
        if let Some(value) = matcher_tuning_overrides().lockfile_same_end_bits_gain_min {
            return value;
        }
        DICTIONARY_SMALLER_OFFSET_BITS_GAIN_MIN
    }

    #[inline(always)]
    pub(super) fn lockfile_same_end_match_loss_max(&self) -> usize {
        #[cfg(feature = "std")]
        if let Some(value) = matcher_tuning_overrides().lockfile_same_end_match_loss_max {
            return value;
        }
        LOCKFILE_SAME_END_SMALLER_OFFSET_MATCH_LOSS_MAX
    }

    #[inline(always)]
    pub(super) fn lockfile_repeat_kind_match_loss_max(&self) -> usize {
        #[cfg(feature = "std")]
        if let Some(value) = matcher_tuning_overrides().lockfile_repeat_kind_match_loss_max {
            return value;
        }
        LOCKFILE_REPEAT_KIND_MATCH_LOSS_MAX
    }

    #[inline(always)]
    pub(super) fn composer_repeat_kind_match_loss_max(&self) -> usize {
        #[cfg(feature = "std")]
        if let Some(value) = matcher_tuning_overrides().composer_repeat_kind_match_loss_max {
            return value;
        }
        COMPOSER_REPEAT_KIND_MATCH_LOSS_MAX
    }

    #[inline(always)]
    pub(super) fn composer_repeat_kind_zero_literals_only(&self) -> bool {
        #[cfg(feature = "std")]
        if let Some(value) = matcher_tuning_overrides().composer_repeat_kind_zero_literals_only {
            return value;
        }
        false
    }

    pub(super) fn prefer_dictionary_smaller_offset_same_start(
        &self,
        preferred: MatchCandidate,
        other: MatchCandidate,
    ) -> bool {
        if !matches!(self.file_type_hint, CompressionFileType::DictionaryText)
            || preferred.repeat_offset
            || other.repeat_offset
            || preferred.start_idx != other.start_idx
            || preferred.offset >= other.offset
            || preferred.match_len + self.dictionary_same_start_match_loss_max() < other.match_len
        {
            return false;
        }

        let preferred_bits = self.non_repeat_offset_code_bits(preferred.offset);
        let other_bits = self.non_repeat_offset_code_bits(other.offset);
        other_bits >= preferred_bits + self.dictionary_same_start_bits_gain_min()
    }

    pub(super) fn prefer_lockfile_repeat_kind_same_start(
        &self,
        preferred: MatchCandidate,
        other: MatchCandidate,
    ) -> bool {
        if !self.current_block_is_dictionary_lockfile_text
            || !preferred.repeat_offset
            || !other.repeat_offset
            || preferred.start_idx != other.start_idx
            || preferred.start_idx != self.suffix_idx
            || preferred.match_len + self.lockfile_repeat_kind_match_loss_max() < other.match_len
        {
            return false;
        }

        let literal_len = self.suffix_idx - self.last_idx_in_sequence;
        let Some(preferred_rank) = self.repeat_kind_preference_rank(preferred.offset, literal_len)
        else {
            return false;
        };
        let Some(other_rank) = self.repeat_kind_preference_rank(other.offset, literal_len) else {
            return false;
        };
        preferred_rank < other_rank
    }

    pub(super) fn prefer_lockfile_smaller_offset_same_end(
        &self,
        preferred: MatchCandidate,
        other: MatchCandidate,
    ) -> bool {
        if !self.current_block_is_dictionary_lockfile_text
            || preferred.repeat_offset
            || other.repeat_offset
            || preferred.offset >= other.offset
            || preferred.match_len + self.lockfile_same_end_match_loss_max() < other.match_len
        {
            return false;
        }

        let preferred_end = preferred.start_idx + preferred.match_len;
        let other_end = other.start_idx + other.match_len;
        if preferred_end != other_end || preferred.start_idx <= other.start_idx {
            return false;
        }

        let preferred_bits = self.non_repeat_offset_code_bits(preferred.offset);
        let other_bits = self.non_repeat_offset_code_bits(other.offset);
        other_bits >= preferred_bits + self.lockfile_same_end_bits_gain_min()
    }

    pub(super) fn prefer_composer_repeat_kind_same_start(
        &self,
        preferred: MatchCandidate,
        other: MatchCandidate,
    ) -> bool {
        let literal_len = self.suffix_idx - self.last_idx_in_sequence;
        if !self.current_block_is_composer_dictionary_text
            || !preferred.repeat_offset
            || !other.repeat_offset
            || preferred.start_idx != other.start_idx
            || preferred.start_idx != self.suffix_idx
            || (self.composer_repeat_kind_zero_literals_only() && literal_len != 0)
            || preferred.match_len + self.composer_repeat_kind_match_loss_max() < other.match_len
        {
            return false;
        }

        let Some(preferred_rank) = self.repeat_kind_preference_rank(preferred.offset, literal_len)
        else {
            return false;
        };
        let Some(other_rank) = self.repeat_kind_preference_rank(other.offset, literal_len) else {
            return false;
        };
        preferred_rank < other_rank
    }

    pub(super) fn repeat_kind_preference_rank(
        &self,
        offset: usize,
        literal_len: usize,
    ) -> Option<usize> {
        self.repeat_offset_candidates(literal_len)
            .iter()
            .position(|(_, candidate_offset)| *candidate_offset == offset)
    }

    pub(super) fn prefer_large_unknown_smaller_offset(
        &self,
        preferred: MatchCandidate,
        other: MatchCandidate,
    ) -> bool {
        if !self.uses_large_unknown_fastest_path()
            || preferred.repeat_offset
            || other.repeat_offset
            || preferred.offset >= other.offset
            || preferred.match_len + LARGE_UNKNOWN_SMALLER_OFFSET_MATCH_LOSS_MAX < other.match_len
        {
            return false;
        }

        let preferred_bits = self.non_repeat_offset_code_bits(preferred.offset);
        let other_bits = self.non_repeat_offset_code_bits(other.offset);
        other_bits >= preferred_bits + LARGE_UNKNOWN_SMALLER_OFFSET_BITS_GAIN_MIN
    }

    #[inline(always)]
    pub(super) fn non_repeat_offset_code_bits(&self, offset: usize) -> usize {
        Self::bounded_u32(offset + 3).ilog2() as usize
    }

    #[inline(always)]
    pub(super) fn allow_repeat_length_early_exit(&self) -> bool {
        !self.uses_large_unknown_fastest_path()
    }

    #[inline(always)]
    pub(super) fn allow_repeat_block_end_early_exit(&self) -> bool {
        true
    }

    #[inline(always)]
    pub(super) fn repeat_match_can_skip_window_search(
        &self,
        candidate: MatchCandidate,
        block_len: usize,
    ) -> bool {
        candidate.repeat_offset
            && ((self.allow_repeat_block_end_early_exit()
                && candidate.start_idx + candidate.match_len == block_len)
                || (self.allow_repeat_length_early_exit()
                    && candidate.match_len >= REPEAT_SEARCH_EARLY_EXIT_LEN))
    }
}
