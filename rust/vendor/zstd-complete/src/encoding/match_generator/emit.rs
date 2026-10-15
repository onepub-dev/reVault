use core::convert::TryInto;

use super::*;

impl MatchGenerator {
    /// Process bytes and add the suffixes to the suffix store up to a specific index
    #[inline(always)]
    pub(super) fn add_suffixes_till(&mut self, idx: usize) {
        if self.last_entry().data.len() < MIN_MATCH_LEN {
            return;
        }
        if !self.should_track_second_newest_for_current_entry() {
            let suffix_idx = self.suffix_idx;
            let last_entry = self.last_entry_mut();
            let slice = &last_entry.data[suffix_idx..idx];
            for (key_index, key) in slice.windows(MIN_MATCH_LEN).enumerate() {
                last_entry.suffixes.insert(key, suffix_idx + key_index);
            }
            return;
        }

        for insert_idx in self.suffix_idx..idx {
            self.add_suffix_at(insert_idx);
        }
    }

    #[cfg(test)]
    #[inline(always)]
    pub(super) fn add_suffixes_for_match(&mut self, idx: usize) {
        self.add_suffixes_for_match_with_dense_limit(idx, DENSE_MATCH_INDEX_LIMIT);
    }

    #[inline(always)]
    pub(super) fn add_suffixes_for_match_with_dense_limit(
        &mut self,
        idx: usize,
        dense_limit: usize,
    ) {
        if idx - self.suffix_idx <= dense_limit {
            self.add_suffixes_till(idx);
            return;
        }

        let suffix_idx = self.suffix_idx;
        self.add_suffix_at(suffix_idx);
        self.add_suffix_at(suffix_idx + 2);
        if self.use_complementary_end_insertion_for_current_block() {
            self.add_suffix_at(idx.saturating_sub(1));
        }
        self.add_suffix_at(idx.saturating_sub(SPARSE_MATCH_END_INDEX_BACKOFF));
    }

    #[inline(always)]
    pub(super) fn add_suffixes_for_sparse_best_match(&mut self, idx: usize) {
        let suffix_idx = self.suffix_idx;
        self.add_suffix_at(suffix_idx);
        self.add_suffix_at(suffix_idx + 2);
        if self.use_complementary_end_insertion_for_current_block() {
            self.add_suffix_at(idx.saturating_sub(1));
        }
        self.add_suffix_at(idx.saturating_sub(SPARSE_MATCH_END_INDEX_BACKOFF));
    }

    #[inline(always)]
    pub(super) fn emit_candidate(
        &mut self,
        candidate: MatchCandidate,
        handle_sequence: &mut impl for<'a> FnMut(Sequence<'a>),
    ) {
        let MatchCandidate {
            start_idx,
            offset,
            match_len,
            repeat_offset,
            ..
        } = candidate;
        let literals_empty = start_idx == self.last_idx_in_sequence;
        let sparse_match = start_idx + match_len - self.suffix_idx > DENSE_MATCH_INDEX_LIMIT;
        if sparse_match && start_idx > self.suffix_idx {
            self.add_suffix_at(start_idx);
        }
        if self.use_complementary_end_insertion_for_current_block()
            && literals_empty
            && repeat_offset
        {
            self.add_suffixes_for_sparse_best_match(start_idx + match_len);
        } else {
            self.add_suffixes_for_match_with_dense_limit(
                start_idx + match_len,
                DENSE_MATCH_INDEX_LIMIT,
            );
        }

        let last_entry_idx = self.last_entry_index();
        let last_entry = &self.window[last_entry_idx];
        let literals = &last_entry.data[self.last_idx_in_sequence..start_idx];
        #[cfg(test)]
        self.diagnostics
            .borrow_mut()
            .record(candidate, literals_empty);
        let offset_value = Self::bounded_u32(offset);
        self.offset_history
            .update_after_match(offset_value, !literals_empty);

        self.suffix_idx = start_idx + match_len;
        self.last_idx_in_sequence = self.suffix_idx;
        handle_sequence(Sequence::Triple {
            literals,
            offset,
            match_len,
        });
    }

    #[inline(always)]
    pub(super) fn add_suffix_at(&mut self, idx: usize) {
        let key_value = {
            let last_entry = self.last_entry();
            let Some(key) = last_entry.data.get(idx..idx + MIN_MATCH_LEN) else {
                return;
            };
            SuffixStore::key_value(key)
        };
        let track_second_newest = self.should_track_second_newest_for_current_entry();
        let track_third_newest = self.uses_dictionary_lockfile_second_newest_path();
        let sidecar_update = if track_second_newest {
            let last_entry = self.last_entry();
            let slot_index = last_entry.suffixes.slot_key(key_value).index;
            let previous_newest = last_entry.suffixes.slots[slot_index]
                .and_then(|slot| (slot.oldest != slot.newest).then_some(slot.newest));
            let previous_second_newest = if track_third_newest {
                self.current_second_newest_sidecar[slot_index]
            } else {
                None
            };
            let previous_third_newest = if track_third_newest {
                self.current_third_newest_sidecar[slot_index]
            } else {
                None
            };
            let previous_fourth_newest = if track_third_newest {
                self.current_fourth_newest_sidecar[slot_index]
            } else {
                None
            };
            let previous_fifth_newest = if track_third_newest {
                self.current_fifth_newest_sidecar[slot_index]
            } else {
                None
            };
            let previous_sixth_newest = if track_third_newest {
                self.current_sixth_newest_sidecar[slot_index]
            } else {
                None
            };
            let previous_seventh_newest = if track_third_newest {
                self.current_seventh_newest_sidecar[slot_index]
            } else {
                None
            };
            let previous_eighth_newest = if track_third_newest {
                self.current_eighth_newest_sidecar[slot_index]
            } else {
                None
            };
            let previous_ninth_newest = if track_third_newest {
                self.current_ninth_newest_sidecar[slot_index]
            } else {
                None
            };
            let previous_tenth_newest = if track_third_newest {
                self.current_tenth_newest_sidecar[slot_index]
            } else {
                None
            };
            let previous_eleventh_newest = if track_third_newest {
                self.current_eleventh_newest_sidecar[slot_index]
            } else {
                None
            };
            let previous_twelfth_newest = if track_third_newest {
                self.current_twelfth_newest_sidecar[slot_index]
            } else {
                None
            };
            Some((
                slot_index,
                previous_newest,
                previous_second_newest,
                previous_third_newest,
                previous_fourth_newest,
                previous_fifth_newest,
                previous_sixth_newest,
                previous_seventh_newest,
                previous_eighth_newest,
                previous_ninth_newest,
                previous_tenth_newest,
                previous_eleventh_newest,
                previous_twelfth_newest,
            ))
        } else {
            None
        };
        let long_hash_update = if self.should_track_current_long_hash() {
            let last_entry = self.last_entry();
            last_entry.data.get(idx..idx + 8).and_then(|key| {
                let key = u64::from_le_bytes(key.try_into().ok()?);
                Some((
                    last_entry.suffixes.slot_key(key).index,
                    SuffixStore::stored_index(idx),
                ))
            })
        } else {
            None
        };
        if let Some((
            slot_index,
            previous_newest,
            previous_second_newest,
            previous_third_newest,
            previous_fourth_newest,
            previous_fifth_newest,
            previous_sixth_newest,
            previous_seventh_newest,
            previous_eighth_newest,
            previous_ninth_newest,
            previous_tenth_newest,
            previous_eleventh_newest,
            previous_twelfth_newest,
        )) = sidecar_update
        {
            self.current_second_newest_sidecar[slot_index] = previous_newest;
            if track_third_newest {
                self.current_third_newest_sidecar[slot_index] = previous_second_newest;
                self.current_fourth_newest_sidecar[slot_index] = previous_third_newest;
                self.current_fifth_newest_sidecar[slot_index] = previous_fourth_newest;
                self.current_sixth_newest_sidecar[slot_index] = previous_fifth_newest;
                self.current_seventh_newest_sidecar[slot_index] = previous_sixth_newest;
                self.current_eighth_newest_sidecar[slot_index] = previous_seventh_newest;
                self.current_ninth_newest_sidecar[slot_index] = previous_eighth_newest;
                self.current_tenth_newest_sidecar[slot_index] = previous_ninth_newest;
                self.current_eleventh_newest_sidecar[slot_index] = previous_tenth_newest;
                self.current_twelfth_newest_sidecar[slot_index] = previous_eleventh_newest;
                self.current_thirteenth_newest_sidecar[slot_index] = previous_twelfth_newest;
            }
        }
        if let Some((slot_index, stored_index)) = long_hash_update {
            self.current_long_hash[slot_index] = Some(stored_index);
        }
        let last_entry = self.last_entry_mut();
        let Some(key) = last_entry.data.get(idx..idx + MIN_MATCH_LEN) else {
            return;
        };
        last_entry.suffixes.insert(key, idx);
    }

    /// Skip matching for the whole current window entry
    pub(super) fn skip_matching(&mut self) {
        let len = self.last_entry().data.len();
        self.add_suffixes_till(len);
        self.suffix_idx = len;
        self.last_idx_in_sequence = len;
    }

    pub(super) fn skip_matching_for_incompressible(&mut self) {
        let len = self.last_entry().data.len();
        self.suffix_idx = len;
        self.last_idx_in_sequence = len;
    }

    pub(super) fn skip_matching_for_rle(&mut self) {
        let len = self.last_entry().data.len();
        if len >= MIN_MATCH_LEN {
            let first_suffix = self.suffix_idx;
            self.add_suffix_at(first_suffix);
            let last_suffix = len - MIN_MATCH_LEN;
            if last_suffix != first_suffix {
                self.add_suffix_at(last_suffix);
            }
        }
        self.suffix_idx = len;
        self.last_idx_in_sequence = len;
    }

    /// Add a new window entry. Will panic if the last window entry hasn't been processed properly.
    /// If any resources are released by pushing the new entry they are returned via the callback
    pub(super) fn add_data(
        &mut self,
        data: Vec<u8>,
        suffixes: SuffixStore,
        reuse_space: impl FnMut(Vec<u8>, SuffixStore),
    ) {
        assert!(self.window.is_empty() || self.suffix_idx == self.last_entry().data.len());
        assert!(data.len() <= u32::MAX as usize);
        let len = data.len();
        let is_text_block = Self::likely_text(&data);
        let is_short_line_text = is_text_block && Self::likely_short_line_text(&data);
        let min_non_repeat_match_len = self.min_non_repeat_match_len_for_text(&data, is_text_block);
        let active_window_size = self.active_window_size_for_text_kind(is_text_block);
        self.reserve(data.len(), active_window_size, reuse_space);
        #[cfg(debug_assertions)]
        self.concat_window.extend_from_slice(&data);

        if let Some(last_len) = self.window.last().map(|last| last.data.len()) {
            for entry in self.window.iter_mut() {
                entry.base_offset += last_len;
            }
        }

        self.window.push(WindowEntry {
            data,
            suffixes,
            base_offset: 0,
        });
        let dictionary_lockfile_text =
            matches!(self.file_type_hint, CompressionFileType::DictionaryText)
                && is_text_block
                && min_non_repeat_match_len == MIN_MATCH_LEN
                && (matches!(self.file_profile_hint, CompressionFileProfile::CargoLock)
                    || Self::likely_lockfile_text(&self.last_entry().data));
        let composer_dictionary_text =
            matches!(self.file_type_hint, CompressionFileType::DictionaryText)
                && is_text_block
                && is_short_line_text
                && (matches!(self.file_profile_hint, CompressionFileProfile::ComposerLock)
                    || Self::likely_composer_dictionary_text(&self.last_entry().data));
        self.current_block_is_dictionary_lockfile_text = dictionary_lockfile_text;
        self.current_block_is_composer_dictionary_text =
            composer_dictionary_text && !dictionary_lockfile_text;
        self.current_block_is_structured_json_config_text =
            matches!(self.file_type_hint, CompressionFileType::ConfigText)
                && is_text_block
                && is_short_line_text
                && Self::likely_structured_json_config_text(&self.last_entry().data);
        self.current_block_is_tsconfig_json_config_text =
            matches!(self.file_type_hint, CompressionFileType::ConfigText)
                && is_text_block
                && is_short_line_text
                && Self::likely_tsconfig_json_config_text(&self.last_entry().data);
        let dictionary_current_entry_second_newest =
            matches!(self.file_type_hint, CompressionFileType::DictionaryText)
                && is_text_block
                && min_non_repeat_match_len == MIN_MATCH_LEN
                && len >= BEST_SECOND_NEWEST_MIN_BLOCK_LEN;
        let code_text_current_entry_second_newest =
            matches!(self.file_type_hint, CompressionFileType::CodeText)
                && is_short_line_text
                && (BEST_SECOND_NEWEST_MIN_BLOCK_LEN..=CODE_TEXT_SECOND_NEWEST_MAX_BLOCK_LEN)
                    .contains(&len);
        let track_second_newest = (self.use_second_newest_probe
            && min_non_repeat_match_len == MIN_MATCH_LEN
            && len >= BEST_SECOND_NEWEST_MIN_BLOCK_LEN)
            || dictionary_current_entry_second_newest
            || code_text_current_entry_second_newest
            || dictionary_lockfile_text
            || (self.use_fast_binary_small_second_newest
                && !is_text_block
                && min_non_repeat_match_len == MIN_MATCH_LEN
                && (len <= FASTEST_SECOND_NEWEST_MAX_BLOCK_LEN
                    || (matches!(self.file_type_hint, CompressionFileType::Unknown)
                        && len <= FASTEST_UNKNOWN_SECOND_NEWEST_MAX_BLOCK_LEN)));
        if track_second_newest {
            let sidecar_len = self.last_entry().suffixes.slots.len();
            self.current_second_newest_sidecar = alloc::vec![None; sidecar_len];
            if dictionary_lockfile_text {
                self.current_third_newest_sidecar = alloc::vec![None; sidecar_len];
                self.current_fourth_newest_sidecar = alloc::vec![None; sidecar_len];
                self.current_fifth_newest_sidecar = alloc::vec![None; sidecar_len];
                self.current_sixth_newest_sidecar = alloc::vec![None; sidecar_len];
                self.current_seventh_newest_sidecar = alloc::vec![None; sidecar_len];
                self.current_eighth_newest_sidecar = alloc::vec![None; sidecar_len];
                self.current_ninth_newest_sidecar = alloc::vec![None; sidecar_len];
                self.current_tenth_newest_sidecar = alloc::vec![None; sidecar_len];
                self.current_eleventh_newest_sidecar = alloc::vec![None; sidecar_len];
                self.current_twelfth_newest_sidecar = alloc::vec![None; sidecar_len];
                self.current_thirteenth_newest_sidecar = alloc::vec![None; sidecar_len];
            } else {
                self.current_third_newest_sidecar.clear();
                self.current_fourth_newest_sidecar.clear();
                self.current_fifth_newest_sidecar.clear();
                self.current_sixth_newest_sidecar.clear();
                self.current_seventh_newest_sidecar.clear();
                self.current_eighth_newest_sidecar.clear();
                self.current_ninth_newest_sidecar.clear();
                self.current_tenth_newest_sidecar.clear();
                self.current_eleventh_newest_sidecar.clear();
                self.current_twelfth_newest_sidecar.clear();
                self.current_thirteenth_newest_sidecar.clear();
            }
            if self.should_track_current_long_hash() {
                self.current_long_hash = alloc::vec![None; sidecar_len];
            } else {
                self.current_long_hash.clear();
            }
        } else {
            self.current_second_newest_sidecar.clear();
            self.current_third_newest_sidecar.clear();
            self.current_fourth_newest_sidecar.clear();
            self.current_fifth_newest_sidecar.clear();
            self.current_sixth_newest_sidecar.clear();
            self.current_seventh_newest_sidecar.clear();
            self.current_eighth_newest_sidecar.clear();
            self.current_ninth_newest_sidecar.clear();
            self.current_tenth_newest_sidecar.clear();
            self.current_eleventh_newest_sidecar.clear();
            self.current_twelfth_newest_sidecar.clear();
            self.current_thirteenth_newest_sidecar.clear();
            self.current_long_hash.clear();
        }
        let last_suffix_len_log = self.last_entry().suffixes.len_log;
        self.uniform_suffix_len_log = match (self.uniform_suffix_len_log, self.window.len()) {
            (_, 1) => Some(last_suffix_len_log),
            (Some(len_log), _) if len_log == last_suffix_len_log => Some(len_log),
            _ => None,
        };
        self.window_size += len;
        self.suffix_idx = 0;
        self.last_idx_in_sequence = 0;
        self.is_text_block = is_text_block;
        self.is_short_line_text = is_short_line_text;
        self.min_non_repeat_match_len = min_non_repeat_match_len;
    }

    /// Reserve space for a new window entry
    /// If any resources are released by pushing the new entry they are returned via the callback
    pub(super) fn reserve(
        &mut self,
        amount: usize,
        max_window_size: usize,
        mut reuse_space: impl FnMut(Vec<u8>, SuffixStore),
    ) {
        assert!(self.max_window_size >= amount);
        assert!(max_window_size >= amount);
        while self.window_size + amount > max_window_size {
            let removed = self.window.remove(0);
            self.window_size -= removed.data.len();
            #[cfg(debug_assertions)]
            self.concat_window.drain(0..removed.data.len());

            let WindowEntry {
                suffixes,
                data: leaked_vec,
                base_offset: _,
            } = removed;
            reuse_space(leaked_vec, suffixes);
        }
    }
}
