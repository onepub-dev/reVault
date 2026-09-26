//! Block encoder adapters for C optimal strategies.

mod attached;
mod target_preference;

pub(crate) use attached::{
    encode_block_opt_attached_dict_with_state_and_policy_and_ldm_in_mode,
    OptAttachedDictBlockSource,
};

use target_preference::{
    prefer_stored_leading_repcode_literal, prefer_target_leading_repcode_literal,
};

use super::{
    block_emit::{append_stored_block_or_raw, PreparedBlockEmission},
    block_policy::BlockEncodingPolicy,
    frame_state::BlockEncodeMode,
    greedy::GreedyBlockOutput,
    greedy_block::{
        encode_prepared_block, encode_special_block, prepare_from_greedy_output_in,
        GreedyBlockEncodeContext, GreedyBlockSource, GreedyEncodedBlock, GreedyPreparedBlock,
    },
    greedy_ext_block::GreedyExtDictBlockSource,
    ldm::opt::LdmOptCursor,
    opt_parser::{
        compress_block_opt_ext_dict_with_state_and_ldm,
        compress_block_opt_no_dict_with_state_and_ldm,
    },
    opt_state::{OptBlockState, OptParserStrategy},
    params::CompressionParameters,
    post_split::encode_split_block,
    sequence_store::{prepare_stored_literal_words_in, RepeatOffsets},
    target_block::{encode_target_block_with_superblock_fallback, TargetBlockOptions},
};
use crate::encoding::blocks::{BlockCompressionConfig, StoredBlockRef};

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_btopt_no_dict_with_state(
    source: GreedyBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    opt_state: &mut OptBlockState,
    context: GreedyBlockEncodeContext<'_, '_>,
    post_block_splitter: bool,
) -> GreedyEncodedBlock {
    encode_block_btopt_no_dict_with_state_and_policy(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        opt_state,
        context,
        post_block_splitter,
        BlockEncodingPolicy::normal(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_btopt_no_dict_with_state_and_policy(
    source: GreedyBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    opt_state: &mut OptBlockState,
    context: GreedyBlockEncodeContext<'_, '_>,
    post_block_splitter: bool,
    policy: BlockEncodingPolicy,
) -> GreedyEncodedBlock {
    encode_block_opt_no_dict_with_state(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        opt_state,
        context,
        OptParserStrategy::BtOpt,
        post_block_splitter,
        policy,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_btultra_no_dict_with_state(
    source: GreedyBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    opt_state: &mut OptBlockState,
    context: GreedyBlockEncodeContext<'_, '_>,
    post_block_splitter: bool,
) -> GreedyEncodedBlock {
    encode_block_btultra_no_dict_with_state_and_policy(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        opt_state,
        context,
        post_block_splitter,
        BlockEncodingPolicy::normal(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_btultra_no_dict_with_state_and_policy(
    source: GreedyBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    opt_state: &mut OptBlockState,
    context: GreedyBlockEncodeContext<'_, '_>,
    post_block_splitter: bool,
    policy: BlockEncodingPolicy,
) -> GreedyEncodedBlock {
    encode_block_opt_no_dict_with_state(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        opt_state,
        context,
        OptParserStrategy::BtUltra,
        post_block_splitter,
        policy,
    )
}

#[allow(clippy::too_many_arguments)]
fn encode_block_opt_no_dict_with_state(
    source: GreedyBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    opt_state: &mut OptBlockState,
    context: GreedyBlockEncodeContext<'_, '_>,
    strategy: OptParserStrategy,
    post_block_splitter: bool,
    policy: BlockEncodingPolicy,
) -> GreedyEncodedBlock {
    encode_block_opt_no_dict_with_state_and_policy(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        opt_state,
        context,
        strategy,
        post_block_splitter,
        policy,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_opt_no_dict_with_state_and_policy(
    source: GreedyBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    opt_state: &mut OptBlockState,
    context: GreedyBlockEncodeContext<'_, '_>,
    strategy: OptParserStrategy,
    post_block_splitter: bool,
    policy: BlockEncodingPolicy,
) -> GreedyEncodedBlock {
    encode_block_opt_no_dict_with_state_and_policy_and_ldm(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        opt_state,
        context,
        strategy,
        post_block_splitter,
        policy,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_opt_no_dict_with_state_and_policy_and_ldm(
    source: GreedyBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    opt_state: &mut OptBlockState,
    context: GreedyBlockEncodeContext<'_, '_>,
    strategy: OptParserStrategy,
    post_block_splitter: bool,
    policy: BlockEncodingPolicy,
    ldm_cursor: Option<&mut LdmOptCursor<'_>>,
) -> GreedyEncodedBlock {
    encode_block_opt_no_dict_with_state_and_policy_and_ldm_in_mode(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        opt_state,
        context,
        strategy,
        block_encode_mode_from_splitter(post_block_splitter),
        policy,
        ldm_cursor,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_opt_no_dict_with_state_and_policy_and_ldm_in_mode(
    source: GreedyBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    opt_state: &mut OptBlockState,
    mut context: GreedyBlockEncodeContext<'_, '_>,
    strategy: OptParserStrategy,
    block_encode_mode: BlockEncodeMode,
    policy: BlockEncodingPolicy,
    ldm_cursor: Option<&mut LdmOptCursor<'_>>,
) -> GreedyEncodedBlock {
    let block = &source.src[source.block_range.clone()];
    let mut bytes = opt_state.take_block_bytes(block.len().saturating_add(3));

    if let Some(encoded) = encode_special_block(block, last_block, repeat_offsets, &mut bytes) {
        return encoded;
    }

    let mut output = compress_block_opt_no_dict_with_state_and_ldm(
        source.src,
        source.block_range.clone(),
        params,
        repeat_offsets,
        opt_state,
        strategy,
        ldm_cursor,
        source.loaded_dict_end,
    );
    if matches!(block_encode_mode, BlockEncodeMode::Normal)
        && config.uses_c_opt_native_sequence_store()
    {
        prefer_stored_leading_repcode_literal(
            source.src,
            source.block_range.start,
            block,
            last_block,
            repeat_offsets,
            &mut output,
        );
        return encode_stored_opt_block(
            block,
            last_block,
            params,
            config,
            repeat_offsets,
            output,
            policy,
            context,
            opt_state,
            bytes,
        );
    }
    let previous_fse = context.fse_tables.snapshot_previous();
    let previous_offsets = *context.offset_history;
    let prepared = prepare_from_greedy_output_in(
        block,
        repeat_offsets,
        &output,
        opt_state.take_prepared_block(),
    );
    let output_repeat_offsets = output.repeat_offsets;
    opt_state.recycle_sequences(output.sequences);
    let mut prepared = GreedyPreparedBlock {
        prepared,
        repeat_offsets: output_repeat_offsets,
    };
    prefer_target_leading_repcode_literal(
        source.src,
        source.block_range.start,
        block,
        last_block,
        repeat_offsets,
        &mut prepared,
    );
    if let Some(target_size) = block_encode_mode.target_c_block_size() {
        let encoded = encode_target_block_with_superblock_fallback(
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
        opt_state.recycle_prepared_block(prepared.prepared);
        return encoded;
    }
    if block_encode_mode.split_block_enabled() {
        match encode_split_block(
            block,
            last_block,
            policy,
            params.strategy,
            config,
            repeat_offsets,
            &prepared,
            previous_offsets,
            &mut context,
            &mut opt_state.post_split_scratch,
            &mut opt_state.entropy_huffman_scratch,
            &mut opt_state.entropy_fse_scratch,
            bytes,
        ) {
            Ok(encoded) => {
                opt_state.recycle_prepared_block(prepared.prepared);
                return encoded;
            }
            Err(returned) => bytes = returned,
        }
    }

    let encoded = encode_prepared_block(
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
    );
    opt_state.recycle_prepared_block(prepared.prepared);
    encoded
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_opt_ext_dict_with_state_and_policy_and_ldm(
    source: GreedyExtDictBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    opt_state: &mut OptBlockState,
    context: GreedyBlockEncodeContext<'_, '_>,
    strategy: OptParserStrategy,
    post_block_splitter: bool,
    policy: BlockEncodingPolicy,
    ldm_cursor: Option<&mut LdmOptCursor<'_>>,
) -> GreedyEncodedBlock {
    encode_block_opt_ext_dict_with_state_and_policy_and_ldm_in_mode(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        opt_state,
        context,
        strategy,
        block_encode_mode_from_splitter(post_block_splitter),
        policy,
        ldm_cursor,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_opt_ext_dict_with_state_and_policy_and_ldm_in_mode(
    source: GreedyExtDictBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    opt_state: &mut OptBlockState,
    mut context: GreedyBlockEncodeContext<'_, '_>,
    strategy: OptParserStrategy,
    block_encode_mode: BlockEncodeMode,
    policy: BlockEncodingPolicy,
    ldm_cursor: Option<&mut LdmOptCursor<'_>>,
) -> GreedyEncodedBlock {
    let block = &source.src[source.block_range.clone()];
    let mut bytes = opt_state.take_block_bytes(block.len().saturating_add(3));

    if let Some(encoded) = encode_special_block(block, last_block, repeat_offsets, &mut bytes) {
        return encoded;
    }

    let mut output = compress_block_opt_ext_dict_with_state_and_ldm(
        source.src,
        source.block_range.clone(),
        source.dict_limit,
        params,
        repeat_offsets,
        opt_state,
        strategy,
        ldm_cursor,
        source.loaded_dict_end,
    );
    if matches!(block_encode_mode, BlockEncodeMode::Normal)
        && config.uses_c_opt_native_sequence_store()
    {
        prefer_stored_leading_repcode_literal(
            source.src,
            source.block_range.start,
            block,
            last_block,
            repeat_offsets,
            &mut output,
        );
        return encode_stored_opt_block(
            block,
            last_block,
            params,
            config,
            repeat_offsets,
            output,
            policy,
            context,
            opt_state,
            bytes,
        );
    }
    let previous_fse = context.fse_tables.snapshot_previous();
    let previous_offsets = *context.offset_history;
    let prepared = prepare_from_greedy_output_in(
        block,
        repeat_offsets,
        &output,
        opt_state.take_prepared_block(),
    );
    let output_repeat_offsets = output.repeat_offsets;
    opt_state.recycle_sequences(output.sequences);
    let mut prepared = GreedyPreparedBlock {
        prepared,
        repeat_offsets: output_repeat_offsets,
    };
    prefer_target_leading_repcode_literal(
        source.src,
        source.block_range.start,
        block,
        last_block,
        repeat_offsets,
        &mut prepared,
    );
    if let Some(target_size) = block_encode_mode.target_c_block_size() {
        let encoded = encode_target_block_with_superblock_fallback(
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
        opt_state.recycle_prepared_block(prepared.prepared);
        return encoded;
    }
    if block_encode_mode.split_block_enabled() {
        match encode_split_block(
            block,
            last_block,
            policy,
            params.strategy,
            config,
            repeat_offsets,
            &prepared,
            previous_offsets,
            &mut context,
            &mut opt_state.post_split_scratch,
            &mut opt_state.entropy_huffman_scratch,
            &mut opt_state.entropy_fse_scratch,
            bytes,
        ) {
            Ok(encoded) => {
                opt_state.recycle_prepared_block(prepared.prepared);
                return encoded;
            }
            Err(returned) => bytes = returned,
        }
    }

    let encoded = encode_prepared_block(
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
    );
    opt_state.recycle_prepared_block(prepared.prepared);
    encoded
}

fn block_encode_mode_from_splitter(post_block_splitter: bool) -> BlockEncodeMode {
    if post_block_splitter {
        BlockEncodeMode::SplitBlock
    } else {
        BlockEncodeMode::Normal
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn encode_stored_opt_block(
    block: &[u8],
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    mut stored: GreedyBlockOutput,
    policy: BlockEncodingPolicy,
    context: GreedyBlockEncodeContext<'_, '_>,
    opt_state: &mut OptBlockState,
    mut bytes: alloc::vec::Vec<u8>,
) -> GreedyEncodedBlock {
    let compressed_repeat_offsets = stored.repeat_offsets;
    let literal_store = prepare_stored_literal_words_in(
        block,
        &stored.sequences,
        stored.last_literals,
        opt_state.take_literal_store(),
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
        Some(&mut opt_state.entropy_huffman_scratch),
        Some(&mut opt_state.entropy_fse_scratch),
        context.fse_tables,
        context.offset_history,
        &mut bytes,
    );
    opt_state.recycle_literal_store(literal_store);
    opt_state.recycle_sequences(core::mem::take(&mut stored.sequences));

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
