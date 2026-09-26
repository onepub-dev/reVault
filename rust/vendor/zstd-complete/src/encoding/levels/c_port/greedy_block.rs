//! Adapters from C greedy sequences to the existing Rust block encoder.

use alloc::vec::Vec;
use core::ops::Range;

use super::block_emit::{
    append_prepared_block_or_raw, append_stored_block_or_raw, PreparedBlockEmission,
};
use super::block_policy::BlockEncodingPolicy;
use super::frame_state::BlockEncodeMode;
use super::greedy::{
    compress_block_btlazy2_no_dict_with_state_and_loaded_dict,
    compress_block_greedy_no_dict_with_state_and_loaded_dict,
    compress_block_lazy2_no_dict_with_state_and_loaded_dict,
    compress_block_lazy_no_dict_with_state_and_loaded_dict, GreedyBlockOutput, GreedyMatchState,
};
use super::params::CompressionParameters;
use super::sequence_store::{
    prepare_stored_literal_words_in, prepare_stored_sequence_words_in, prepared_block_into_words,
    prepared_words_into_block, RepeatOffsets,
};
use super::target_block::{encode_target_block_with_superblock_fallback, TargetBlockOptions};
use crate::{
    encoding::{
        blocks::{BlockCompressionConfig, PreparedBlock, StoredBlockRef},
        frame_compressor::{FseTableSnapshot, FseTables, OffsetHistory},
    },
    huff0::huff0_encoder::HuffmanTable,
};

mod special;

pub(super) use special::encode_special_block;

pub(crate) struct GreedyPreparedBlock {
    pub(crate) prepared: PreparedBlock,
    pub(crate) repeat_offsets: RepeatOffsets,
}

pub(crate) struct GreedyEncodedBlock {
    pub(crate) bytes: Vec<u8>,
    pub(crate) repeat_offsets: RepeatOffsets,
    pub(crate) new_huffman_table: Option<HuffmanTable>,
}

pub(crate) struct GreedyBlockEncodeContext<'a, 'table> {
    pub(crate) previous_huff_table: Option<&'table HuffmanTable>,
    pub(crate) fse_tables: &'a mut FseTables,
    pub(crate) offset_history: &'a mut OffsetHistory,
}

pub(crate) struct GreedyBlockSource<'a> {
    pub(crate) src: &'a [u8],
    pub(crate) block_range: Range<usize>,
    pub(crate) loaded_dict_end: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LazyBlockStrategy {
    Greedy,
    Lazy,
    Lazy2,
    BtLazy2,
}

pub(crate) fn prepare_block_greedy_no_dict(
    src: &[u8],
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
) -> GreedyPreparedBlock {
    prepare_block_hash_chain_no_dict(src, params, repeat_offsets, LazyBlockStrategy::Greedy)
}

fn prepare_block_hash_chain_no_dict(
    src: &[u8],
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    depth: LazyBlockStrategy,
) -> GreedyPreparedBlock {
    let mut state = GreedyMatchState::new();
    let output = compress_block_for_depth_with_state(
        src,
        0..src.len(),
        params,
        repeat_offsets,
        &mut state,
        depth,
        0,
    );
    let prepared = prepare_from_greedy_output(src, repeat_offsets, &output);

    GreedyPreparedBlock {
        prepared,
        repeat_offsets: output.repeat_offsets,
    }
}

pub(crate) fn prepare_block_greedy_no_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
) -> GreedyPreparedBlock {
    prepare_block_hash_chain_no_dict_with_state(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        LazyBlockStrategy::Greedy,
        0,
    )
}

fn prepare_block_hash_chain_no_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    depth: LazyBlockStrategy,
    loaded_dict_end: usize,
) -> GreedyPreparedBlock {
    let block = &src[block_range.clone()];
    let mut output = compress_block_for_depth_with_state(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        depth,
        loaded_dict_end,
    );
    let prepared = prepare_from_greedy_output(block, repeat_offsets, &output);
    let output_repeat_offsets = output.repeat_offsets;
    state.recycle_sequence_store(core::mem::take(&mut output.sequences));

    GreedyPreparedBlock {
        prepared,
        repeat_offsets: output_repeat_offsets,
    }
}

fn compress_block_for_depth_with_state(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    depth: LazyBlockStrategy,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    match depth {
        LazyBlockStrategy::Greedy => compress_block_greedy_no_dict_with_state_and_loaded_dict(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        LazyBlockStrategy::Lazy => compress_block_lazy_no_dict_with_state_and_loaded_dict(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        LazyBlockStrategy::Lazy2 => compress_block_lazy2_no_dict_with_state_and_loaded_dict(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        LazyBlockStrategy::BtLazy2 => compress_block_btlazy2_no_dict_with_state_and_loaded_dict(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_hash_chain_no_dict(
    src: &[u8],
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    context: GreedyBlockEncodeContext<'_, '_>,
    depth: LazyBlockStrategy,
) -> GreedyEncodedBlock {
    encode_block_hash_chain_no_dict_with_policy(
        src,
        last_block,
        params,
        config,
        repeat_offsets,
        context,
        depth,
        BlockEncodingPolicy::normal(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_hash_chain_no_dict_with_policy(
    src: &[u8],
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    context: GreedyBlockEncodeContext<'_, '_>,
    depth: LazyBlockStrategy,
    policy: BlockEncodingPolicy,
) -> GreedyEncodedBlock {
    let mut match_state = GreedyMatchState::new();
    encode_block_hash_chain_no_dict_with_state_and_policy_in_mode(
        GreedyBlockSource {
            src,
            block_range: 0..src.len(),
            loaded_dict_end: 0,
        },
        last_block,
        params,
        config,
        repeat_offsets,
        &mut match_state,
        context,
        depth,
        policy,
        BlockEncodeMode::Normal,
    )
}

pub(crate) fn encode_block_greedy_no_dict_with_state(
    source: GreedyBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    match_state: &mut GreedyMatchState,
    context: GreedyBlockEncodeContext<'_, '_>,
) -> GreedyEncodedBlock {
    encode_block_hash_chain_no_dict_with_state_and_policy(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        match_state,
        context,
        LazyBlockStrategy::Greedy,
        BlockEncodingPolicy::normal(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_hash_chain_no_dict_with_state(
    source: GreedyBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    match_state: &mut GreedyMatchState,
    context: GreedyBlockEncodeContext<'_, '_>,
    depth: LazyBlockStrategy,
) -> GreedyEncodedBlock {
    encode_block_hash_chain_no_dict_with_state_and_policy(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        match_state,
        context,
        depth,
        BlockEncodingPolicy::normal(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_hash_chain_no_dict_with_state_and_policy(
    source: GreedyBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    match_state: &mut GreedyMatchState,
    context: GreedyBlockEncodeContext<'_, '_>,
    depth: LazyBlockStrategy,
    policy: BlockEncodingPolicy,
) -> GreedyEncodedBlock {
    encode_block_hash_chain_no_dict_with_state_and_policy_in_mode(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        match_state,
        context,
        depth,
        policy,
        BlockEncodeMode::Normal,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_hash_chain_no_dict_with_state_and_policy_in_mode(
    source: GreedyBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    match_state: &mut GreedyMatchState,
    context: GreedyBlockEncodeContext<'_, '_>,
    depth: LazyBlockStrategy,
    policy: BlockEncodingPolicy,
    block_encode_mode: BlockEncodeMode,
) -> GreedyEncodedBlock {
    encode_block_hash_chain_no_dict_with_state_and_policy_in_mode_into(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        match_state,
        context,
        depth,
        policy,
        block_encode_mode,
        Vec::new(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_hash_chain_no_dict_with_state_and_policy_in_mode_into(
    source: GreedyBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    match_state: &mut GreedyMatchState,
    context: GreedyBlockEncodeContext<'_, '_>,
    depth: LazyBlockStrategy,
    policy: BlockEncodingPolicy,
    block_encode_mode: BlockEncodeMode,
    mut bytes: Vec<u8>,
) -> GreedyEncodedBlock {
    let block = &source.src[source.block_range.clone()];
    bytes.clear();

    if let Some(encoded) = encode_special_block(block, last_block, repeat_offsets, &mut bytes) {
        return encoded;
    }

    if let Some(target_size) = block_encode_mode.target_c_block_size() {
        let prepared = prepare_block_hash_chain_no_dict_with_state(
            source.src,
            source.block_range.clone(),
            params,
            repeat_offsets,
            match_state,
            depth,
            source.loaded_dict_end,
        );
        return encode_target_block_with_superblock_fallback(
            block,
            last_block,
            TargetBlockOptions {
                target_c_block_size: target_size,
                strategy: params.strategy,
                allow_rle: policy.allows_rle(),
                repeat_offsets,
            },
            &prepared,
            context,
            bytes,
        );
    }

    if config.uses_c_greedy_native_sequence_store() {
        let output = compress_block_for_depth_with_state(
            source.src,
            source.block_range,
            params,
            repeat_offsets,
            match_state,
            depth,
            source.loaded_dict_end,
        );
        return encode_stored_block(
            block,
            last_block,
            params,
            config,
            repeat_offsets,
            output,
            policy,
            context,
            match_state,
            bytes,
        );
    }

    let prepared = prepare_block_hash_chain_no_dict_with_state(
        source.src,
        source.block_range,
        params,
        repeat_offsets,
        match_state,
        depth,
        source.loaded_dict_end,
    );
    let previous_fse = context.fse_tables.snapshot_previous();
    let previous_offsets = *context.offset_history;
    encode_prepared_block(
        block,
        last_block,
        params,
        config,
        repeat_offsets,
        &prepared,
        policy,
        previous_fse,
        previous_offsets,
        context,
        bytes,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn encode_stored_block(
    block: &[u8],
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    mut stored: GreedyBlockOutput,
    policy: BlockEncodingPolicy,
    context: GreedyBlockEncodeContext<'_, '_>,
    match_state: &mut GreedyMatchState,
    mut bytes: Vec<u8>,
) -> GreedyEncodedBlock {
    let compressed_repeat_offsets = stored.repeat_offsets;
    let literal_store = prepare_stored_literal_words_in(
        block,
        &stored.sequences,
        stored.last_literals,
        match_state.take_prepared_store(),
    );
    let emission = append_stored_block_or_raw(
        block,
        last_block,
        params.strategy,
        policy,
        config,
        StoredBlockRef {
            literals: &literal_store.literals,
            sequences: &stored.sequences,
        },
        compressed_repeat_offsets,
        context.previous_huff_table,
        Some(&mut match_state.entropy_huffman_scratch),
        Some(&mut match_state.entropy_fse_scratch),
        context.fse_tables,
        context.offset_history,
        &mut bytes,
    );
    match_state.recycle_sequence_store(core::mem::take(&mut stored.sequences));
    match_state.recycle_prepared_store(literal_store);

    match emission {
        PreparedBlockEmission::Raw | PreparedBlockEmission::Rle => GreedyEncodedBlock {
            bytes,
            repeat_offsets,
            new_huffman_table: None,
        },
        PreparedBlockEmission::Compressed { new_huffman_table } => GreedyEncodedBlock {
            bytes,
            repeat_offsets: compressed_repeat_offsets,
            new_huffman_table,
        },
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn encode_prepared_block(
    block: &[u8],
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    prepared: &GreedyPreparedBlock,
    policy: BlockEncodingPolicy,
    previous_fse: FseTableSnapshot,
    previous_offsets: OffsetHistory,
    context: GreedyBlockEncodeContext<'_, '_>,
    mut bytes: Vec<u8>,
) -> GreedyEncodedBlock {
    let compressed_repeat_offsets = prepared.repeat_offsets;
    match append_prepared_block_or_raw(
        block,
        last_block,
        params.strategy,
        policy,
        config,
        prepared.prepared.as_ref(),
        previous_fse,
        previous_offsets,
        context.previous_huff_table,
        context.fse_tables,
        context.offset_history,
        &mut bytes,
    ) {
        PreparedBlockEmission::Raw | PreparedBlockEmission::Rle => GreedyEncodedBlock {
            bytes,
            repeat_offsets,
            new_huffman_table: None,
        },
        PreparedBlockEmission::Compressed { new_huffman_table } => GreedyEncodedBlock {
            bytes,
            repeat_offsets: compressed_repeat_offsets,
            new_huffman_table,
        },
    }
}

pub(super) fn prepare_from_greedy_output(
    src: &[u8],
    initial_repeat_offsets: RepeatOffsets,
    output: &GreedyBlockOutput,
) -> PreparedBlock {
    prepare_from_greedy_output_in(
        src,
        initial_repeat_offsets,
        output,
        PreparedBlock {
            literals: Vec::new(),
            sequences: Vec::new(),
        },
    )
}

pub(super) fn prepare_from_greedy_output_in(
    src: &[u8],
    initial_repeat_offsets: RepeatOffsets,
    output: &GreedyBlockOutput,
    prepared: PreparedBlock,
) -> PreparedBlock {
    prepared_words_into_block(prepare_stored_sequence_words_in(
        src,
        initial_repeat_offsets,
        &output.sequences,
        output.last_literals,
        prepared_block_into_words(prepared),
    ))
}
