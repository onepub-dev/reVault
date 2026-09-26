//! No-dictionary greedy block compressor ported from `zstd_lazy.c`.

use core::ops::Range;

pub(crate) use super::greedy_state::GreedyMatchState;
use super::params::CompressionParameters;
use super::row_match::row_match_finder_enabled;
use super::sequence_store::{RepeatOffsets, StoredSequence};
use crate::workspace::ReusableVec;

mod lazy;

pub(super) use lazy::{
    compress_block_lazy_generic_with_state, compress_block_lazy_generic_with_state_and_attached,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GreedyBlockOutput {
    pub(crate) sequences: ReusableVec<StoredSequence>,
    pub(crate) last_literals: u32,
    pub(crate) repeat_offsets: RepeatOffsets,
}

pub(super) const SEARCH_HASH_CHAIN: u8 = 0;
pub(super) const SEARCH_BINARY_TREE: u8 = 1;
pub(super) const SEARCH_ROW_HASH: u8 = 2;

pub(crate) fn compress_block_greedy_no_dict(
    src: &[u8],
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
) -> GreedyBlockOutput {
    let mut state = GreedyMatchState::new();
    compress_block_greedy_no_dict_with_state(src, 0..src.len(), params, repeat_offsets, &mut state)
}

pub(crate) fn compress_block_greedy_no_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
) -> GreedyBlockOutput {
    compress_block_greedy_no_dict_with_state_and_loaded_dict(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        0,
    )
}

pub(crate) fn compress_block_greedy_no_dict_with_state_and_loaded_dict(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    compress_block_hash_or_row_no_dict_with_state::<0>(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        loaded_dict_end,
    )
}

pub(crate) fn compress_block_lazy_no_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
) -> GreedyBlockOutput {
    compress_block_lazy_no_dict_with_state_and_loaded_dict(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        0,
    )
}

pub(crate) fn compress_block_lazy_no_dict_with_state_and_loaded_dict(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    compress_block_hash_or_row_no_dict_with_state::<1>(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        loaded_dict_end,
    )
}

pub(crate) fn compress_block_lazy2_no_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
) -> GreedyBlockOutput {
    compress_block_lazy2_no_dict_with_state_and_loaded_dict(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        0,
    )
}

pub(crate) fn compress_block_lazy2_no_dict_with_state_and_loaded_dict(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    compress_block_hash_or_row_no_dict_with_state::<2>(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        loaded_dict_end,
    )
}

pub(crate) fn compress_block_btlazy2_no_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
) -> GreedyBlockOutput {
    compress_block_btlazy2_no_dict_with_state_and_loaded_dict(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        0,
    )
}

pub(crate) fn compress_block_btlazy2_no_dict_with_state_and_loaded_dict(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    lazy::compress_block_lazy_generic_no_dict_with_state::<SEARCH_BINARY_TREE, 2>(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        loaded_dict_end,
    )
}

fn compress_block_hash_chain_no_dict_with_state<const DEPTH: u32>(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
) -> GreedyBlockOutput {
    lazy::compress_block_lazy_generic_no_dict_with_state::<SEARCH_HASH_CHAIN, DEPTH>(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        0,
    )
}

fn compress_block_hash_or_row_no_dict_with_state<const DEPTH: u32>(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    if row_match_finder_enabled(params) {
        lazy::compress_block_lazy_generic_no_dict_with_state::<SEARCH_ROW_HASH, DEPTH>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        )
    } else {
        lazy::compress_block_lazy_generic_no_dict_with_state::<SEARCH_HASH_CHAIN, DEPTH>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        )
    }
}
