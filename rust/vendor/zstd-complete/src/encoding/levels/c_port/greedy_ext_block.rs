//! External-dictionary adapters for the C greedy/lazy block compressors.

use alloc::vec::Vec;
use core::ops::Range;

use super::{
    block_policy::BlockEncodingPolicy,
    frame_state::BlockEncodeMode,
    greedy::{GreedyBlockOutput, GreedyMatchState},
    greedy_block::{
        encode_prepared_block, encode_special_block, encode_stored_block,
        prepare_from_greedy_output, GreedyBlockEncodeContext, GreedyEncodedBlock,
        GreedyPreparedBlock, LazyBlockStrategy,
    },
    greedy_ext::{
        compress_block_attached_row_dict_with_state,
        compress_block_btlazy2_attached_dict_with_state,
        compress_block_btlazy2_ext_dict_with_state, compress_block_greedy_ext_dict_with_state,
        compress_block_lazy2_ext_dict_with_state, compress_block_lazy_ext_dict_with_state,
    },
    params::CompressionParameters,
    sequence_store::RepeatOffsets,
    target_block::{encode_target_block_with_superblock_fallback, TargetBlockOptions},
};
use crate::encoding::blocks::BlockCompressionConfig;

pub(crate) struct GreedyExtDictBlockSource<'a> {
    pub(crate) src: &'a [u8],
    pub(crate) block_range: Range<usize>,
    pub(crate) dict_limit: usize,
    pub(crate) loaded_dict_end: usize,
}

pub(crate) struct GreedyAttachedDictBlockSource<'a> {
    pub(crate) src: &'a [u8],
    pub(crate) block_range: Range<usize>,
    pub(crate) active_dict_limit: usize,
    pub(crate) active_prefix_start: usize,
    pub(crate) dictionary_src: &'a [u8],
    pub(crate) dictionary_state: &'a GreedyMatchState,
    pub(crate) dictionary_params: CompressionParameters,
    pub(crate) dictionary_index_start: usize,
}

fn prepare_block_hash_chain_ext_dict_with_state(
    source: GreedyExtDictBlockSource<'_>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    depth: LazyBlockStrategy,
) -> GreedyPreparedBlock {
    let block = &source.src[source.block_range.clone()];
    let mut output = compress_block_for_depth_ext_dict_with_state(
        source.src,
        source.block_range,
        source.dict_limit,
        params,
        repeat_offsets,
        state,
        depth,
        source.loaded_dict_end,
    );
    let prepared = prepare_from_greedy_output(block, repeat_offsets, &output);
    let output_repeat_offsets = output.repeat_offsets;
    state.recycle_sequence_store(core::mem::take(&mut output.sequences));

    GreedyPreparedBlock {
        prepared,
        repeat_offsets: output_repeat_offsets,
    }
}

fn prepare_block_attached_dict_with_state(
    source: GreedyAttachedDictBlockSource<'_>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    depth: LazyBlockStrategy,
) -> GreedyPreparedBlock {
    let block = &source.src[source.block_range.clone()];
    let mut output =
        compress_block_attached_dict_with_state(source, params, repeat_offsets, state, depth);
    let prepared = prepare_from_greedy_output(block, repeat_offsets, &output);
    let output_repeat_offsets = output.repeat_offsets;
    state.recycle_sequence_store(core::mem::take(&mut output.sequences));

    GreedyPreparedBlock {
        prepared,
        repeat_offsets: output_repeat_offsets,
    }
}

fn compress_block_attached_dict_with_state(
    source: GreedyAttachedDictBlockSource<'_>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    depth: LazyBlockStrategy,
) -> GreedyBlockOutput {
    match depth {
        LazyBlockStrategy::BtLazy2 => compress_block_btlazy2_attached_dict_with_state(
            source.src,
            source.block_range,
            source.active_dict_limit,
            source.active_prefix_start,
            params,
            repeat_offsets,
            state,
            source.dictionary_src,
            source.dictionary_state,
            source.dictionary_params,
            source.dictionary_index_start,
        ),
        LazyBlockStrategy::Greedy => compress_block_attached_row_dict_with_state::<0>(
            source.src,
            source.block_range,
            source.active_dict_limit,
            source.active_prefix_start,
            params,
            repeat_offsets,
            state,
            source.dictionary_src,
            source.dictionary_state,
            source.dictionary_params,
            source.dictionary_index_start,
        ),
        LazyBlockStrategy::Lazy => compress_block_attached_row_dict_with_state::<1>(
            source.src,
            source.block_range,
            source.active_dict_limit,
            source.active_prefix_start,
            params,
            repeat_offsets,
            state,
            source.dictionary_src,
            source.dictionary_state,
            source.dictionary_params,
            source.dictionary_index_start,
        ),
        LazyBlockStrategy::Lazy2 => compress_block_attached_row_dict_with_state::<2>(
            source.src,
            source.block_range,
            source.active_dict_limit,
            source.active_prefix_start,
            params,
            repeat_offsets,
            state,
            source.dictionary_src,
            source.dictionary_state,
            source.dictionary_params,
            source.dictionary_index_start,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn compress_block_for_depth_ext_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    dict_limit: usize,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    depth: LazyBlockStrategy,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    match depth {
        LazyBlockStrategy::Greedy => compress_block_greedy_ext_dict_with_state(
            src,
            block_range,
            dict_limit,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        LazyBlockStrategy::Lazy => compress_block_lazy_ext_dict_with_state(
            src,
            block_range,
            dict_limit,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        LazyBlockStrategy::Lazy2 => compress_block_lazy2_ext_dict_with_state(
            src,
            block_range,
            dict_limit,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        LazyBlockStrategy::BtLazy2 => compress_block_btlazy2_ext_dict_with_state(
            src,
            block_range,
            dict_limit,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_hash_chain_ext_dict_with_state_and_policy(
    source: GreedyExtDictBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    match_state: &mut GreedyMatchState,
    context: GreedyBlockEncodeContext<'_, '_>,
    depth: LazyBlockStrategy,
    policy: BlockEncodingPolicy,
) -> GreedyEncodedBlock {
    encode_block_hash_chain_ext_dict_with_state_and_policy_in_mode(
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
pub(crate) fn encode_block_hash_chain_ext_dict_with_state_and_policy_in_mode(
    source: GreedyExtDictBlockSource<'_>,
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
    encode_block_hash_chain_ext_dict_with_state_and_policy_in_mode_into(
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
pub(crate) fn encode_block_hash_chain_ext_dict_with_state_and_policy_in_mode_into(
    source: GreedyExtDictBlockSource<'_>,
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
        let prepared = prepare_block_hash_chain_ext_dict_with_state(
            source,
            params,
            repeat_offsets,
            match_state,
            depth,
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
        let output = compress_block_for_depth_ext_dict_with_state(
            source.src,
            source.block_range,
            source.dict_limit,
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

    let prepared = prepare_block_hash_chain_ext_dict_with_state(
        source,
        params,
        repeat_offsets,
        match_state,
        depth,
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
pub(crate) fn encode_block_attached_dict_with_state_and_policy_in_mode(
    source: GreedyAttachedDictBlockSource<'_>,
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
    let block = &source.src[source.block_range.clone()];
    let mut bytes = Vec::new();

    if let Some(encoded) = encode_special_block(block, last_block, repeat_offsets, &mut bytes) {
        return encoded;
    }

    if let Some(target_size) = block_encode_mode.target_c_block_size() {
        let prepared = prepare_block_attached_dict_with_state(
            source,
            params,
            repeat_offsets,
            match_state,
            depth,
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
        let output = compress_block_attached_dict_with_state(
            source,
            params,
            repeat_offsets,
            match_state,
            depth,
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

    let prepared =
        prepare_block_attached_dict_with_state(source, params, repeat_offsets, match_state, depth);
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
