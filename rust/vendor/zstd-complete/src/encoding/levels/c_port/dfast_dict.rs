//! Dictionary prefix loading for the double-fast match state.

use alloc::vec;

use super::{
    dfast::DFastMatchState,
    dfast_helpers::{hash_ptr, HASH_READ_SIZE},
    params::CompressionParameters,
};

const SHORT_CACHE_TAG_BITS: u32 = 8;
const SHORT_CACHE_TAG_MASK: usize = (1 << SHORT_CACHE_TAG_BITS) - 1;
const FAST_HASH_FILL_STEP: usize = 3;

pub(crate) fn load_prefix(
    state: &mut DFastMatchState,
    src: &[u8],
    prefix_len: usize,
    params: CompressionParameters,
) {
    debug_assert!(prefix_len <= src.len());
    if prefix_len <= HASH_READ_SIZE {
        return;
    }

    state.ensure_tables(params);
    let iend = prefix_len - HASH_READ_SIZE;
    let mut ip = 0_usize;

    while ip + FAST_HASH_FILL_STEP - 1 <= iend {
        let hash_small = hash_ptr(src, ip, params.chain_log, params.min_match);
        let hash_long = hash_ptr(src, ip, params.hash_log, 8);
        state.hash_small[hash_small] = ip as u32;
        state.hash_long[hash_long] = ip as u32;
        ip += FAST_HASH_FILL_STEP;
    }
}

pub(crate) fn load_cdict_copy_prefix(
    state: &mut DFastMatchState,
    src: &[u8],
    prefix_len: usize,
    params: CompressionParameters,
) {
    debug_assert!(prefix_len <= src.len());
    if prefix_len <= HASH_READ_SIZE {
        return;
    }

    state.ensure_tables(params);
    state.hash_small.fill(0);
    state.hash_long.fill(0);

    let mut tagged_long = vec![0_u32; state.hash_long.len()];
    let iend = prefix_len - HASH_READ_SIZE;
    let mut ip = 0_usize;

    while ip + FAST_HASH_FILL_STEP - 1 <= iend {
        for step in 0..FAST_HASH_FILL_STEP {
            let pos = ip + step;
            let small_hash_and_tag = hash_ptr(
                src,
                pos,
                params.chain_log + SHORT_CACHE_TAG_BITS,
                params.min_match,
            );
            if step == 0 {
                state.hash_small[table_index(small_hash_and_tag)] = pos as u32;
            }

            let long_hash_and_tag = hash_ptr(src, pos, params.hash_log + SHORT_CACHE_TAG_BITS, 8);
            let long_slot = table_index(long_hash_and_tag);
            if step == 0 || tagged_long[long_slot] == 0 {
                tagged_long[long_slot] = tagged_index(long_hash_and_tag, pos);
            }
        }
        ip += FAST_HASH_FILL_STEP;
    }

    for (dst, tagged) in state.hash_long.iter_mut().zip(tagged_long) {
        *dst = tagged >> SHORT_CACHE_TAG_BITS;
    }
}

fn table_index(hash_and_tag: usize) -> usize {
    hash_and_tag >> SHORT_CACHE_TAG_BITS
}

fn tagged_index(hash_and_tag: usize, index: usize) -> u32 {
    debug_assert!(index <= (u32::MAX >> SHORT_CACHE_TAG_BITS) as usize);
    let tag = hash_and_tag & SHORT_CACHE_TAG_MASK;
    ((index as u32) << SHORT_CACHE_TAG_BITS) | tag as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::levels::c_port::params::Strategy;

    #[test]
    fn double_fast_prefix_loader_fills_every_third_position_like_c_fast_load() {
        let data = b"abcdefghijklmnopqrstuvwxyabcdefghijklmnopqrstuvwxy";
        let params = params();
        let mut state = DFastMatchState::new();

        load_prefix(&mut state, data, data.len(), params);

        let hash0_small = hash_ptr(data, 0, params.chain_log, params.min_match);
        let hash3_small = hash_ptr(data, 3, params.chain_log, params.min_match);
        let hash1_small = hash_ptr(data, 1, params.chain_log, params.min_match);
        let hash0_long = hash_ptr(data, 0, params.hash_log, 8);
        let hash3_long = hash_ptr(data, 3, params.hash_log, 8);

        assert_eq!(state.hash_small[hash0_small], 0);
        assert_eq!(state.hash_small[hash3_small], 3);
        assert_eq!(state.hash_small[hash1_small], 0);
        assert_eq!(state.hash_long[hash0_long], 0);
        assert_eq!(state.hash_long[hash3_long], 3);
    }

    #[test]
    fn double_fast_cdict_copy_loader_uses_full_large_hash_like_c() {
        let data = b"abcdefghijklmnopqrstuvwxyabcdefghijklmnopqrstuvwxy";
        let params = params();
        let mut state = DFastMatchState::new();

        load_cdict_copy_prefix(&mut state, data, data.len(), params);

        let hash0_small = hash_ptr(
            data,
            0,
            params.chain_log + SHORT_CACHE_TAG_BITS,
            params.min_match,
        ) >> SHORT_CACHE_TAG_BITS;
        let hash1_small = hash_ptr(
            data,
            1,
            params.chain_log + SHORT_CACHE_TAG_BITS,
            params.min_match,
        ) >> SHORT_CACHE_TAG_BITS;
        let hash0_long =
            hash_ptr(data, 0, params.hash_log + SHORT_CACHE_TAG_BITS, 8) >> SHORT_CACHE_TAG_BITS;
        let hash1_long =
            hash_ptr(data, 1, params.hash_log + SHORT_CACHE_TAG_BITS, 8) >> SHORT_CACHE_TAG_BITS;

        assert_eq!(state.hash_small[hash0_small], 0);
        assert_eq!(state.hash_small[hash1_small], 0);
        assert_eq!(state.hash_long[hash0_long], 0);
        assert_eq!(state.hash_long[hash1_long], 1);
    }

    fn params() -> CompressionParameters {
        CompressionParameters {
            window_log: 17,
            chain_log: 12,
            hash_log: 13,
            search_log: 1,
            min_match: 5,
            target_length: 0,
            strategy: Strategy::DFast,
        }
    }
}
