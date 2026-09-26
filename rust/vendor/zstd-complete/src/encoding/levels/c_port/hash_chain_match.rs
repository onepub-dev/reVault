//! Hash-chain search primitives shared by the C greedy/lazy/lazy2 ports.

#![forbid(unsafe_code)]

pub(super) use super::unaligned::read32;
use super::{
    greedy::GreedyMatchState, params::CompressionParameters, sequence_store::OffBase,
    unaligned::read64,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AttachedDictionarySearch<'a> {
    pub(super) src: &'a [u8],
    pub(super) state: &'a GreedyMatchState,
    pub(super) params: CompressionParameters,
    pub(super) dictionary_index_start: usize,
    pub(super) active_dict_limit: usize,
    pub(super) active_prefix_start: usize,
}

impl<'a> AttachedDictionarySearch<'a> {
    pub(super) fn zero_based(
        src: &'a [u8],
        state: &'a GreedyMatchState,
        params: CompressionParameters,
        active_prefix_start: usize,
    ) -> Self {
        Self {
            src,
            state,
            params,
            dictionary_index_start: 0,
            active_dict_limit: active_prefix_start,
            active_prefix_start,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct MatchSearchConfig<'a> {
    pub(super) params: CompressionParameters,
    pub(super) min_match: u32,
    pub(super) loaded_dict_end: usize,
    pub(super) attached_dictionary: Option<AttachedDictionarySearch<'a>>,
}

impl<'a> MatchSearchConfig<'a> {
    pub(super) fn new(
        params: CompressionParameters,
        min_match: u32,
        loaded_dict_end: usize,
    ) -> Self {
        Self {
            params,
            min_match,
            loaded_dict_end,
            attached_dictionary: None,
        }
    }

    pub(super) fn with_attached_dictionary(
        mut self,
        attached_dictionary: AttachedDictionarySearch<'a>,
    ) -> Self {
        self.attached_dictionary = Some(attached_dictionary);
        self
    }

    pub(super) fn lowest_prefix_index(self, pos: usize) -> usize {
        lowest_prefix_index_with_loaded_dict(pos, self.params.window_log, self.loaded_dict_end)
    }
}

pub(super) fn hc_find_best_match(
    src: &[u8],
    ip: usize,
    block_end: usize,
    off_base: &mut u32,
    state: &mut GreedyMatchState,
    config: MatchSearchConfig<'_>,
) -> usize {
    let params = config.params;
    let chain_size = 1_usize << params.chain_log;
    let chain_mask = chain_size - 1;
    let curr = ip;
    let low_limit = config.lowest_prefix_index(curr);
    let min_chain = curr.saturating_sub(chain_size);
    let mut attempts = 1_usize << params.search_log;
    let mut ml = 3_usize;
    let mut match_index = insert_and_find_first_index(src, ip, params, config.min_match, state);

    while match_index >= low_limit && attempts > 0 {
        attempts -= 1;
        let current_ml = if read32(src, match_index + ml - 3) == read32(src, ip + ml - 3) {
            count_match(src, ip, match_index, block_end)
        } else {
            0
        };

        if current_ml > ml {
            ml = current_ml;
            *off_base = OffBase::offset_to_c_value((curr - match_index) as u32);
            if ip + current_ml == block_end {
                break;
            }
        }

        if match_index <= min_chain {
            break;
        }
        match_index = state.chain_table[match_index & chain_mask] as usize;
    }

    ml
}

fn insert_and_find_first_index(
    src: &[u8],
    ip: usize,
    params: CompressionParameters,
    min_match: u32,
    state: &mut GreedyMatchState,
) -> usize {
    let chain_mask = (1_usize << params.chain_log) - 1;
    let mut idx = state.next_to_update;

    while idx < ip {
        let hash = hash_ptr(src, idx, params.hash_log, min_match);
        state.chain_table[idx & chain_mask] = state.hash_table[hash];
        state.hash_table[hash] = idx as u32;
        idx += 1;
        if state.lazy_skipping {
            break;
        }
    }

    state.next_to_update = ip;
    state.hash_table[hash_ptr(src, ip, params.hash_log, min_match)] as usize
}

pub(super) fn load_dictionary_hash_chain(
    src: &[u8],
    target: usize,
    params: CompressionParameters,
    min_match: u32,
    state: &mut GreedyMatchState,
) {
    let _ = insert_and_find_first_index(src, target, params, min_match, state);
}

pub(super) use super::match_count::{count_match, count_match_no_dict};
pub(super) fn hash_ptr(src: &[u8], pos: usize, h_bits: u32, min_match: u32) -> usize {
    match min_match {
        5 => hash_ptr_mls::<5>(src, pos, h_bits),
        6 => hash_ptr_mls::<6>(src, pos, h_bits),
        _ => hash_ptr_mls::<4>(src, pos, h_bits),
    }
}

pub(super) fn hash_ptr_mls<const MIN_MATCH: u32>(src: &[u8], pos: usize, h_bits: u32) -> usize {
    match MIN_MATCH {
        5 => hash5(read64(src, pos), h_bits),
        6 => hash6(read64(src, pos), h_bits),
        _ => hash4(read32(src, pos), h_bits),
    }
}

pub(super) fn hash3_ptr(src: &[u8], pos: usize, h_bits: u32) -> usize {
    const PRIME_3_BYTES: u32 = 506_832_829;
    (read32(src, pos) << 8)
        .wrapping_mul(PRIME_3_BYTES)
        .wrapping_shr(32 - h_bits) as usize
}

pub(super) fn equal_min_match(src: &[u8], left: usize, right: usize, min_match: u32) -> bool {
    debug_assert!(matches!(min_match, 3 | 4));
    if min_match == 3 {
        return (read32(src, left) << 8) == (read32(src, right) << 8);
    }
    read32(src, left) == read32(src, right)
}

pub(super) fn lowest_prefix_index(pos: usize, window_log: u32) -> usize {
    pos.saturating_sub(1_usize << window_log)
}

pub(super) fn lowest_prefix_index_with_loaded_dict(
    pos: usize,
    window_log: u32,
    loaded_dict_end: usize,
) -> usize {
    let window_size = 1_usize << window_log;
    if loaded_dict_end != 0 && pos <= loaded_dict_end.saturating_add(window_size) {
        0
    } else {
        pos.saturating_sub(window_size)
    }
}

pub(super) fn highbit32(value: u32) -> u32 {
    debug_assert!(value > 0);
    u32::BITS - 1 - value.leading_zeros()
}

fn hash4(value: u32, h_bits: u32) -> usize {
    const PRIME_4_BYTES: u32 = 2_654_435_761;
    value.wrapping_mul(PRIME_4_BYTES).wrapping_shr(32 - h_bits) as usize
}

fn hash5(value: u64, h_bits: u32) -> usize {
    const PRIME_5_BYTES: u64 = 889_523_592_379;
    ((value << (64 - 40)).wrapping_mul(PRIME_5_BYTES) >> (64 - h_bits)) as usize
}

fn hash6(value: u64, h_bits: u32) -> usize {
    const PRIME_6_BYTES: u64 = 227_718_039_650_203;
    ((value << (64 - 48)).wrapping_mul(PRIME_6_BYTES) >> (64 - h_bits)) as usize
}

#[cfg(test)]
mod tests {
    use super::equal_min_match;

    #[test]
    fn min_match_three_ignores_fourth_byte_like_c() {
        let data = b"abcXabcY";

        assert!(equal_min_match(data, 0, 4, 3));
        assert!(!equal_min_match(data, 0, 4, 4));
    }
}
