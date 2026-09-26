//! Block encoder adapter for an attached optimal-parser dictionary.

use core::ops::Range;

use super::{
    encode_stored_opt_block, prefer_stored_leading_repcode_literal,
    prefer_target_leading_repcode_literal,
};
use crate::encoding::{
    blocks::BlockCompressionConfig,
    levels::c_port::{
        block_policy::BlockEncodingPolicy,
        frame_state::BlockEncodeMode,
        greedy_block::{
            encode_prepared_block, encode_special_block, prepare_from_greedy_output_in,
            GreedyBlockEncodeContext, GreedyEncodedBlock, GreedyPreparedBlock,
        },
        ldm::opt::LdmOptCursor,
        opt_match::OptAttachedDictionary,
        opt_parser::compress_block_opt_attached_dict_with_state_and_ldm,
        opt_state::{OptBlockState, OptParserStrategy},
        params::CompressionParameters,
        post_split::encode_split_block,
        sequence_store::RepeatOffsets,
        target_block::{encode_target_block_with_superblock_fallback, TargetBlockOptions},
    },
};

pub(crate) struct OptAttachedDictBlockSource<'a> {
    pub(crate) src: &'a [u8],
    pub(crate) block_range: Range<usize>,
    pub(crate) dictionary: OptAttachedDictionary,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_opt_attached_dict_with_state_and_policy_and_ldm_in_mode(
    source: OptAttachedDictBlockSource<'_>,
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

    let mut output = compress_block_opt_attached_dict_with_state_and_ldm(
        source.src,
        source.block_range.clone(),
        params,
        repeat_offsets,
        opt_state,
        strategy,
        ldm_cursor,
        source.dictionary,
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
