//! External-dictionary entry points for the C greedy/lazy/lazy2 ports.

use core::ops::Range;

use super::{
    greedy::{
        compress_block_lazy_generic_with_state,
        compress_block_lazy_generic_with_state_and_attached, GreedyBlockOutput, GreedyMatchState,
        SEARCH_BINARY_TREE, SEARCH_HASH_CHAIN, SEARCH_ROW_HASH,
    },
    greedy_bounds::LazyDictionaryBounds,
    hash_chain_match::AttachedDictionarySearch,
    params::CompressionParameters,
    row_match::row_match_finder_enabled,
    sequence_store::RepeatOffsets,
};

pub(crate) fn compress_block_greedy_ext_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    dict_limit: usize,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    compress_block_hash_or_row_ext_dict_with_state::<0>(
        src,
        block_range,
        dict_limit,
        params,
        repeat_offsets,
        state,
        loaded_dict_end,
    )
}

pub(crate) fn compress_block_lazy_ext_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    dict_limit: usize,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    compress_block_hash_or_row_ext_dict_with_state::<1>(
        src,
        block_range,
        dict_limit,
        params,
        repeat_offsets,
        state,
        loaded_dict_end,
    )
}

pub(crate) fn compress_block_lazy2_ext_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    dict_limit: usize,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    compress_block_hash_or_row_ext_dict_with_state::<2>(
        src,
        block_range,
        dict_limit,
        params,
        repeat_offsets,
        state,
        loaded_dict_end,
    )
}

pub(crate) fn compress_block_btlazy2_ext_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    dict_limit: usize,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    compress_block_lazy_generic_with_state::<SEARCH_BINARY_TREE, 2>(
        src,
        block_range.clone(),
        params,
        repeat_offsets,
        state,
        LazyDictionaryBounds::ext_dict(block_range.end, dict_limit, params, loaded_dict_end),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compress_block_greedy_attached_row_dict_with_state<'a>(
    src: &'a [u8],
    block_range: Range<usize>,
    active_dict_limit: usize,
    active_prefix_start: usize,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    dictionary_src: &'a [u8],
    dictionary_state: &'a GreedyMatchState,
    dictionary_params: CompressionParameters,
    dictionary_index_start: usize,
) -> GreedyBlockOutput {
    compress_block_attached_row_dict_with_state::<0>(
        src,
        block_range,
        active_dict_limit,
        active_prefix_start,
        params,
        repeat_offsets,
        state,
        dictionary_src,
        dictionary_state,
        dictionary_params,
        dictionary_index_start,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compress_block_attached_row_dict_with_state<'a, const DEPTH: u32>(
    src: &'a [u8],
    block_range: Range<usize>,
    active_dict_limit: usize,
    active_prefix_start: usize,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    dictionary_src: &'a [u8],
    dictionary_state: &'a GreedyMatchState,
    dictionary_params: CompressionParameters,
    dictionary_index_start: usize,
) -> GreedyBlockOutput {
    debug_assert!(row_match_finder_enabled(params));
    compress_block_lazy_generic_with_state_and_attached::<SEARCH_ROW_HASH, DEPTH>(
        src,
        block_range.clone(),
        params,
        repeat_offsets,
        state,
        LazyDictionaryBounds::no_dict(block_range.end, params, active_dict_limit),
        AttachedDictionarySearch {
            src: dictionary_src,
            state: dictionary_state,
            params: dictionary_params,
            dictionary_index_start,
            active_dict_limit,
            active_prefix_start,
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compress_block_btlazy2_attached_dict_with_state<'a>(
    src: &'a [u8],
    block_range: Range<usize>,
    active_dict_limit: usize,
    active_prefix_start: usize,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    dictionary_src: &'a [u8],
    dictionary_state: &'a GreedyMatchState,
    dictionary_params: CompressionParameters,
    dictionary_index_start: usize,
) -> GreedyBlockOutput {
    debug_assert!(!row_match_finder_enabled(params));
    compress_block_lazy_generic_with_state_and_attached::<SEARCH_BINARY_TREE, 2>(
        src,
        block_range.clone(),
        params,
        repeat_offsets,
        state,
        LazyDictionaryBounds::no_dict(block_range.end, params, active_dict_limit),
        AttachedDictionarySearch {
            src: dictionary_src,
            state: dictionary_state,
            params: dictionary_params,
            dictionary_index_start,
            active_dict_limit,
            active_prefix_start,
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn compress_block_hash_or_row_ext_dict_with_state<const DEPTH: u32>(
    src: &[u8],
    block_range: Range<usize>,
    dict_limit: usize,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    let bounds =
        LazyDictionaryBounds::ext_dict(block_range.end, dict_limit, params, loaded_dict_end);
    if row_match_finder_enabled(params) {
        compress_block_lazy_generic_with_state::<SEARCH_ROW_HASH, DEPTH>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            bounds,
        )
    } else {
        compress_block_lazy_generic_with_state::<SEARCH_HASH_CHAIN, DEPTH>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            bounds,
        )
    }
}
