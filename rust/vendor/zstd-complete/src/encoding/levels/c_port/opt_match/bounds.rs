//! Match-window and dictionary-coordinate bounds for the optimal parser.

use crate::encoding::levels::c_port::{
    greedy::GreedyMatchState,
    hash_chain_match::{
        count_match, equal_min_match, lowest_prefix_index_with_loaded_dict,
        AttachedDictionarySearch,
    },
    params::CompressionParameters,
};

#[derive(Clone, Copy)]
pub(in crate::encoding::levels::c_port) struct OptAttachedDictionary {
    dictionary_start: usize,
    dictionary_size: usize,
    params: CompressionParameters,
    dictionary_index_start: usize,
    active_dict_limit: usize,
    active_prefix_start: usize,
}

impl OptAttachedDictionary {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::encoding::levels::c_port) fn new(
        dictionary_start: usize,
        dictionary_size: usize,
        params: CompressionParameters,
        dictionary_index_start: usize,
        active_dict_limit: usize,
        active_prefix_start: usize,
    ) -> Self {
        Self {
            dictionary_start,
            dictionary_size,
            params,
            dictionary_index_start,
            active_dict_limit,
            active_prefix_start,
        }
    }

    pub(in crate::encoding::levels::c_port) fn search<'a>(
        self,
        src: &'a [u8],
        state: &'a GreedyMatchState,
    ) -> AttachedDictionarySearch<'a> {
        let dictionary_end = self.dictionary_start + self.dictionary_size;
        AttachedDictionarySearch {
            src: &src[self.dictionary_start..dictionary_end],
            state,
            params: self.params,
            dictionary_index_start: self.dictionary_index_start,
            active_dict_limit: self.active_dict_limit,
            active_prefix_start: self.active_prefix_start,
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::encoding::levels::c_port) struct OptMatchBounds {
    dict_limit: usize,
    prefix_start_index: usize,
    pub(in crate::encoding::levels::c_port) loaded_dict_end: usize,
    ext_dict: bool,
    attached_dictionary: Option<OptAttachedDictionary>,
}

impl OptMatchBounds {
    pub(in crate::encoding::levels::c_port) fn no_dict(
        block_end: usize,
        params: CompressionParameters,
        loaded_dict_end: usize,
    ) -> Self {
        let prefix_start_index =
            lowest_prefix_index_with_loaded_dict(block_end, params.window_log, loaded_dict_end);
        Self {
            dict_limit: prefix_start_index,
            prefix_start_index,
            loaded_dict_end,
            ext_dict: false,
            attached_dictionary: None,
        }
    }

    pub(in crate::encoding::levels::c_port) fn ext_dict(
        block_end: usize,
        dict_limit: usize,
        params: CompressionParameters,
        loaded_dict_end: usize,
    ) -> Self {
        let dict_start_index =
            lowest_prefix_index_with_loaded_dict(block_end, params.window_log, loaded_dict_end);
        Self {
            dict_limit,
            prefix_start_index: dict_limit.max(dict_start_index),
            loaded_dict_end,
            ext_dict: true,
            attached_dictionary: None,
        }
    }

    pub(in crate::encoding::levels::c_port) fn attached(
        block_end: usize,
        attached_dictionary: OptAttachedDictionary,
    ) -> Self {
        let prefix_start_index = attached_dictionary.active_prefix_start;
        debug_assert!(prefix_start_index <= block_end);
        debug_assert!(prefix_start_index <= attached_dictionary.active_dict_limit);
        Self {
            dict_limit: prefix_start_index,
            prefix_start_index,
            loaded_dict_end: 0,
            ext_dict: false,
            attached_dictionary: Some(attached_dictionary),
        }
    }

    pub(in crate::encoding::levels::c_port) fn prefix_start_index(self) -> usize {
        self.prefix_start_index
    }

    pub(in crate::encoding::levels::c_port) fn is_ext_dict(self) -> bool {
        self.ext_dict
    }

    pub(in crate::encoding::levels::c_port) fn has_loaded_dict(self) -> bool {
        self.loaded_dict_end != 0
    }

    pub(in crate::encoding::levels::c_port) fn attached_dictionary(
        self,
    ) -> Option<OptAttachedDictionary> {
        self.attached_dictionary
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::encoding::levels::c_port) fn rep_match_length(
        self,
        src: &[u8],
        ip: usize,
        rep_offset: usize,
        min_match: u32,
        block_end: usize,
        window_low: usize,
    ) -> Option<usize> {
        if rep_offset == 0 || rep_offset > ip {
            return None;
        }
        let rep_index = ip - rep_offset;
        if self.attached_dictionary.is_some() {
            if rep_index >= self.prefix_start_index {
                if rep_index < window_low {
                    return None;
                }
            } else if !index_overlap_check(self.prefix_start_index, rep_index) {
                return None;
            }
        } else if self.ext_dict {
            if rep_index >= self.dict_limit {
                if rep_index < window_low {
                    return None;
                }
            } else if rep_offset > ip.saturating_sub(window_low)
                || !index_overlap_check(self.dict_limit, rep_index)
            {
                return None;
            }
        } else if rep_index < window_low {
            return None;
        }

        if rep_index + min_match as usize > src.len()
            || ip + min_match as usize > src.len()
            || !equal_min_match(src, ip, rep_index, min_match)
        {
            return None;
        }

        Some(
            count_match(
                src,
                ip + min_match as usize,
                rep_index + min_match as usize,
                block_end,
            ) + min_match as usize,
        )
    }
}

fn index_overlap_check(prefix_lowest_index: usize, rep_index: usize) -> bool {
    prefix_lowest_index.wrapping_sub(1).wrapping_sub(rep_index) >= 3
}
