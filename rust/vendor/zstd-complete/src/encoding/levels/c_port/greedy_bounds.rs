//! Dictionary and prefix bounds for the C greedy/lazy match loop.

use super::{
    hash_chain_match::{
        count_match, count_match_no_dict, lowest_prefix_index_with_loaded_dict, read32,
    },
    params::CompressionParameters,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct LazyDictionaryBounds {
    pub(super) dict_limit: usize,
    pub(super) prefix_start_index: usize,
    pub(super) loaded_dict_end: usize,
    pub(super) ext_dict: bool,
    dict_start_index: usize,
}

impl LazyDictionaryBounds {
    pub(super) fn no_dict(
        block_end: usize,
        params: CompressionParameters,
        loaded_dict_end: usize,
    ) -> Self {
        let prefix_start_index =
            lowest_prefix_index_with_loaded_dict(block_end, params.window_log, loaded_dict_end);
        Self {
            dict_limit: prefix_start_index,
            dict_start_index: prefix_start_index,
            prefix_start_index,
            loaded_dict_end,
            ext_dict: false,
        }
    }

    pub(super) fn ext_dict(
        block_end: usize,
        dict_limit: usize,
        params: CompressionParameters,
        loaded_dict_end: usize,
    ) -> Self {
        let dict_start_index =
            lowest_prefix_index_with_loaded_dict(block_end, params.window_log, loaded_dict_end);
        Self {
            dict_limit,
            dict_start_index,
            prefix_start_index: dict_limit.max(dict_start_index),
            loaded_dict_end,
            ext_dict: true,
        }
    }

    pub(super) fn low_match_index<const EXT_DICT: bool>(self, match_index: usize) -> usize {
        debug_assert_eq!(self.ext_dict, EXT_DICT);
        if EXT_DICT && match_index < self.dict_limit {
            self.dict_start_index
        } else {
            self.prefix_start_index
        }
    }

    pub(super) fn rep_match_length<const EXT_DICT: bool>(
        self,
        src: &[u8],
        current: usize,
        offset: usize,
        params: CompressionParameters,
        block_end: usize,
    ) -> Option<usize> {
        if offset == 0 || current < offset {
            return None;
        }
        let rep_index = current - offset;
        debug_assert_eq!(self.ext_dict, EXT_DICT);
        if EXT_DICT {
            let window_low = lowest_prefix_index_with_loaded_dict(
                current,
                params.window_log,
                self.loaded_dict_end,
            );
            if offset > current.saturating_sub(window_low)
                || !index_overlap_check(self.dict_limit, rep_index)
            {
                return None;
            }
        }
        if rep_index + 4 > src.len() || current + 4 > src.len() {
            return None;
        }
        (read32(src, rep_index) == read32(src, current))
            .then(|| count_match(src, current + 4, rep_index + 4, block_end) + 4)
    }
}

/// C's direct no-dictionary repeat probe used by every Greedy/Lazy depth.
///
/// The parser disables offsets outside its active prefix before entering the
/// loop and only advances toward `block_end`, so a nonzero repeat offset always
/// identifies an earlier source position. Keeping this separate from the
/// external-dictionary bounds path lets each generated parser inline C's
/// compare/count continuation without an `Option` or shared call boundary.
#[inline(always)]
pub(super) fn rep_match_length_no_dict(
    src: &[u8],
    current: usize,
    offset: usize,
    block_end: usize,
) -> usize {
    if offset == 0 {
        return 0;
    }
    debug_assert!(current >= offset);
    debug_assert!(current + 4 <= block_end);
    debug_assert!(block_end <= src.len());
    let rep_index = current - offset;
    if read32(src, rep_index) != read32(src, current) {
        return 0;
    }
    count_match_no_dict(src, current + 4, rep_index + 4, block_end) + 4
}

fn index_overlap_check(prefix_lowest_index: usize, rep_index: usize) -> bool {
    prefix_lowest_index.wrapping_sub(1).wrapping_sub(rep_index) >= 3
}

#[cfg(test)]
mod tests {
    use super::rep_match_length_no_dict;

    #[test]
    fn direct_no_dict_repeat_probe_matches_to_block_end() {
        let source = b"abcdefghabcdefgh";

        assert_eq!(rep_match_length_no_dict(source, 8, 8, source.len()), 8);
        assert_eq!(rep_match_length_no_dict(source, 8, 0, source.len()), 0);
    }

    #[test]
    fn direct_no_dict_repeat_probe_rejects_four_byte_mismatch() {
        let source = b"abcdefghXbcdefgh";

        assert_eq!(rep_match_length_no_dict(source, 8, 8, source.len()), 0);
    }
}
