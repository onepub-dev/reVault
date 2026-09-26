//! Block-level entry points for the no-dictionary C optimal strategies.

use core::ops::Range;

use super::{
    greedy::GreedyBlockOutput,
    opt_parser::compress_block_opt_no_dict_with_state,
    opt_state::{OptBlockState, OptParserStrategy},
    params::CompressionParameters,
    sequence_store::RepeatOffsets,
};

pub(crate) fn compress_block_btopt_no_dict(
    src: &[u8],
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
) -> GreedyBlockOutput {
    let mut state = OptBlockState::new();
    compress_block_btopt_no_dict_with_state(src, 0..src.len(), params, repeat_offsets, &mut state)
}

pub(crate) fn compress_block_btultra_no_dict(
    src: &[u8],
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
) -> GreedyBlockOutput {
    let mut state = OptBlockState::new();
    compress_block_opt_no_dict_with_state(
        src,
        0..src.len(),
        params,
        repeat_offsets,
        &mut state,
        OptParserStrategy::BtUltra,
    )
}

pub(crate) fn compress_block_btopt_no_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut OptBlockState,
) -> GreedyBlockOutput {
    compress_block_opt_no_dict_with_state(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        OptParserStrategy::BtOpt,
    )
}

pub(crate) fn compress_block_btultra_no_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut OptBlockState,
) -> GreedyBlockOutput {
    compress_block_opt_no_dict_with_state(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        OptParserStrategy::BtUltra,
    )
}

pub(crate) fn prime_btultra2_stats_no_dict(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    state: &mut OptBlockState,
) {
    let mut prime_state = OptBlockState::new();
    prime_state.price_state = state.price_state.clone();
    let _ = compress_block_btultra_no_dict_with_state(
        src,
        block_range,
        params,
        RepeatOffsets::new(),
        &mut prime_state,
    );
    state.price_state = prime_state.price_state;
}
