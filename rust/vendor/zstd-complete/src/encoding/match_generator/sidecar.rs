use core::convert::TryInto;

use super::*;

impl MatchGenerator {
    #[inline(always)]
    pub(super) fn best_current_long_hash_candidate(
        &self,
        context: &MatchCandidateContext<'_>,
        block_len: usize,
    ) -> Option<MatchCandidate> {
        if !self.should_track_current_long_hash()
            || context.suffix_idx != self.suffix_idx
            || context.data_slice.len() < 8
        {
            return None;
        }

        let long_key = u64::from_le_bytes(context.data_slice[..8].try_into().ok()?);
        let slot_index = self.last_entry().suffixes.slot_key(long_key).index;
        let match_index = self
            .current_long_hash
            .get(slot_index)
            .and_then(|idx| *idx)
            .map(|idx| idx.get() as usize - 1)?;

        let current_entry = self.last_entry();
        if !Self::has_long_match_at_index(current_entry, match_index, context) {
            return None;
        }

        let offset = context.suffix_idx - match_index;
        let match_len = self.match_len_at_offset_with_prefix(offset, context, 8);
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

        let candidate = MatchCandidate {
            start_idx,
            offset,
            match_len,
            repeat_offset: false,
            #[cfg(test)]
            source: CandidateSource::WindowCurrentLongHash,
        };
        if candidate.start_idx + candidate.match_len == block_len {
            return Some(candidate);
        }
        Some(candidate)
    }

    #[inline(always)]
    pub(super) fn best_second_newest_candidate(
        &self,
        entry_distance: usize,
        slot_index: usize,
        context: &MatchCandidateContext<'_>,
        candidate: Option<MatchCandidate>,
    ) -> Option<usize> {
        if !self.should_probe_second_newest(context, entry_distance, candidate) {
            return None;
        }

        self.current_second_newest_sidecar
            .get(slot_index)
            .and_then(|idx| *idx)
            .map(|idx| idx.get() as usize - 1)
    }

    #[inline(always)]
    pub(super) fn best_third_newest_candidate(
        &self,
        entry_distance: usize,
        slot_index: usize,
        context: &MatchCandidateContext<'_>,
    ) -> Option<usize> {
        if !self.should_probe_third_newest(context, entry_distance) {
            return None;
        }

        self.current_third_newest_sidecar
            .get(slot_index)
            .and_then(|idx| *idx)
            .map(|idx| idx.get() as usize - 1)
    }

    #[inline(always)]
    pub(super) fn best_fourth_newest_candidate(
        &self,
        entry_distance: usize,
        slot_index: usize,
        context: &MatchCandidateContext<'_>,
    ) -> Option<usize> {
        if !self.should_probe_fourth_newest(context, entry_distance) {
            return None;
        }

        self.current_fourth_newest_sidecar
            .get(slot_index)
            .and_then(|idx| *idx)
            .map(|idx| idx.get() as usize - 1)
    }

    #[inline(always)]
    pub(super) fn best_fifth_newest_candidate(
        &self,
        entry_distance: usize,
        slot_index: usize,
        context: &MatchCandidateContext<'_>,
    ) -> Option<usize> {
        if !self.should_probe_fifth_newest(context, entry_distance) {
            return None;
        }

        self.current_fifth_newest_sidecar
            .get(slot_index)
            .and_then(|idx| *idx)
            .map(|idx| idx.get() as usize - 1)
    }

    #[inline(always)]
    pub(super) fn best_sixth_newest_candidate(
        &self,
        entry_distance: usize,
        slot_index: usize,
        context: &MatchCandidateContext<'_>,
    ) -> Option<usize> {
        if !self.should_probe_sixth_newest(context, entry_distance) {
            return None;
        }

        self.current_sixth_newest_sidecar
            .get(slot_index)
            .and_then(|idx| *idx)
            .map(|idx| idx.get() as usize - 1)
    }

    #[inline(always)]
    pub(super) fn best_seventh_newest_candidate(
        &self,
        entry_distance: usize,
        slot_index: usize,
        context: &MatchCandidateContext<'_>,
    ) -> Option<usize> {
        if !self.should_probe_seventh_newest(context, entry_distance) {
            return None;
        }

        self.current_seventh_newest_sidecar
            .get(slot_index)
            .and_then(|idx| *idx)
            .map(|idx| idx.get() as usize - 1)
    }

    #[inline(always)]
    pub(super) fn best_eighth_newest_candidate(
        &self,
        entry_distance: usize,
        slot_index: usize,
        context: &MatchCandidateContext<'_>,
    ) -> Option<usize> {
        if !self.should_probe_eighth_newest(context, entry_distance) {
            return None;
        }

        self.current_eighth_newest_sidecar
            .get(slot_index)
            .and_then(|idx| *idx)
            .map(|idx| idx.get() as usize - 1)
    }

    #[inline(always)]
    pub(super) fn best_ninth_newest_candidate(
        &self,
        entry_distance: usize,
        slot_index: usize,
        context: &MatchCandidateContext<'_>,
    ) -> Option<usize> {
        if !self.should_probe_ninth_newest(context, entry_distance) {
            return None;
        }

        self.current_ninth_newest_sidecar
            .get(slot_index)
            .and_then(|idx| *idx)
            .map(|idx| idx.get() as usize - 1)
    }

    #[inline(always)]
    pub(super) fn best_tenth_newest_candidate(
        &self,
        entry_distance: usize,
        slot_index: usize,
        context: &MatchCandidateContext<'_>,
    ) -> Option<usize> {
        if !self.should_probe_tenth_newest(context, entry_distance) {
            return None;
        }

        self.current_tenth_newest_sidecar
            .get(slot_index)
            .and_then(|idx| *idx)
            .map(|idx| idx.get() as usize - 1)
    }

    #[inline(always)]
    pub(super) fn best_eleventh_newest_candidate(
        &self,
        entry_distance: usize,
        slot_index: usize,
        context: &MatchCandidateContext<'_>,
    ) -> Option<usize> {
        if !self.should_probe_eleventh_newest(context, entry_distance) {
            return None;
        }

        self.current_eleventh_newest_sidecar
            .get(slot_index)
            .and_then(|idx| *idx)
            .map(|idx| idx.get() as usize - 1)
    }

    #[inline(always)]
    pub(super) fn best_twelfth_newest_candidate(
        &self,
        entry_distance: usize,
        slot_index: usize,
        context: &MatchCandidateContext<'_>,
    ) -> Option<usize> {
        if !self.should_probe_twelfth_newest(context, entry_distance) {
            return None;
        }

        self.current_twelfth_newest_sidecar
            .get(slot_index)
            .and_then(|idx| *idx)
            .map(|idx| idx.get() as usize - 1)
    }

    #[inline(always)]
    pub(super) fn best_thirteenth_newest_candidate(
        &self,
        entry_distance: usize,
        slot_index: usize,
        context: &MatchCandidateContext<'_>,
    ) -> Option<usize> {
        if !self.should_probe_thirteenth_newest(context, entry_distance) {
            return None;
        }

        self.current_thirteenth_newest_sidecar
            .get(slot_index)
            .and_then(|idx| *idx)
            .map(|idx| idx.get() as usize - 1)
    }

    #[inline(always)]
    pub(super) fn should_probe_second_newest(
        &self,
        context: &MatchCandidateContext<'_>,
        entry_distance: usize,
        candidate: Option<MatchCandidate>,
    ) -> bool {
        #[cfg(feature = "std")]
        if self.uses_dictionary_lockfile_second_newest_path()
            && context.suffix_idx == context.anchor_idx
            && matches!(
                matcher_tuning_overrides().lockfile_second_newest_zero_literals,
                Some(false)
            )
        {
            return false;
        }
        let recent_entry_limit = if self.uses_dictionary_lockfile_second_newest_path() {
            LOCKFILE_SECOND_NEWEST_RECENT_ENTRY_LIMIT
        } else {
            BEST_SECOND_NEWEST_RECENT_ENTRY_LIMIT
        };
        self.should_track_second_newest_for_current_entry()
            && context.suffix_idx == self.suffix_idx
            && entry_distance < recent_entry_limit
            && (!self.use_fast_binary_small_second_newest || candidate.is_none())
    }

    #[inline(always)]
    pub(super) fn should_probe_third_newest(
        &self,
        context: &MatchCandidateContext<'_>,
        entry_distance: usize,
    ) -> bool {
        self.uses_dictionary_lockfile_second_newest_path()
            && context.suffix_idx == self.suffix_idx
            && entry_distance == 0
    }

    #[inline(always)]
    pub(super) fn should_probe_fourth_newest(
        &self,
        context: &MatchCandidateContext<'_>,
        entry_distance: usize,
    ) -> bool {
        self.uses_dictionary_lockfile_second_newest_path()
            && context.suffix_idx == self.suffix_idx
            && entry_distance == 0
    }

    #[inline(always)]
    pub(super) fn should_probe_fifth_newest(
        &self,
        context: &MatchCandidateContext<'_>,
        entry_distance: usize,
    ) -> bool {
        self.uses_dictionary_lockfile_second_newest_path()
            && context.suffix_idx == self.suffix_idx
            && entry_distance == 0
    }

    #[inline(always)]
    pub(super) fn should_probe_sixth_newest(
        &self,
        context: &MatchCandidateContext<'_>,
        entry_distance: usize,
    ) -> bool {
        self.uses_dictionary_lockfile_second_newest_path()
            && context.suffix_idx == self.suffix_idx
            && entry_distance == 0
    }

    #[inline(always)]
    pub(super) fn should_probe_seventh_newest(
        &self,
        context: &MatchCandidateContext<'_>,
        entry_distance: usize,
    ) -> bool {
        self.uses_dictionary_lockfile_second_newest_path()
            && context.suffix_idx == self.suffix_idx
            && entry_distance == 0
    }

    #[inline(always)]
    pub(super) fn should_probe_eighth_newest(
        &self,
        context: &MatchCandidateContext<'_>,
        entry_distance: usize,
    ) -> bool {
        self.uses_dictionary_lockfile_second_newest_path()
            && context.suffix_idx == self.suffix_idx
            && entry_distance == 0
    }

    #[inline(always)]
    pub(super) fn should_probe_ninth_newest(
        &self,
        context: &MatchCandidateContext<'_>,
        entry_distance: usize,
    ) -> bool {
        self.uses_dictionary_lockfile_second_newest_path()
            && context.suffix_idx == self.suffix_idx
            && entry_distance == 0
    }

    #[inline(always)]
    pub(super) fn should_probe_tenth_newest(
        &self,
        context: &MatchCandidateContext<'_>,
        entry_distance: usize,
    ) -> bool {
        self.uses_dictionary_lockfile_second_newest_path()
            && context.suffix_idx == self.suffix_idx
            && entry_distance == 0
    }

    #[inline(always)]
    pub(super) fn should_probe_eleventh_newest(
        &self,
        context: &MatchCandidateContext<'_>,
        entry_distance: usize,
    ) -> bool {
        self.uses_dictionary_lockfile_second_newest_path()
            && context.suffix_idx == self.suffix_idx
            && entry_distance == 0
    }

    #[inline(always)]
    pub(super) fn should_probe_twelfth_newest(
        &self,
        context: &MatchCandidateContext<'_>,
        entry_distance: usize,
    ) -> bool {
        self.uses_dictionary_lockfile_second_newest_path()
            && context.suffix_idx == self.suffix_idx
            && entry_distance == 0
    }

    #[inline(always)]
    pub(super) fn should_probe_thirteenth_newest(
        &self,
        context: &MatchCandidateContext<'_>,
        entry_distance: usize,
    ) -> bool {
        self.uses_dictionary_lockfile_second_newest_path()
            && context.suffix_idx == self.suffix_idx
            && entry_distance == 0
    }
}
