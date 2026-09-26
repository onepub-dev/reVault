//! Optimal frame strategy and no-dictionary block dispatch.

use crate::encoding::{
    blocks::BlockCompressionConfig,
    levels::c_port::{
        block_policy::BlockEncodingPolicy,
        frame_state::BlockEncodeMode,
        greedy_block::{GreedyBlockEncodeContext, GreedyBlockSource, GreedyEncodedBlock},
        ldm::opt::LdmOptCursor,
        opt_encode::{
            encode_block_btopt_no_dict_with_state_and_policy,
            encode_block_btultra_no_dict_with_state_and_policy,
            encode_block_opt_no_dict_with_state_and_policy_and_ldm_in_mode,
        },
        opt_state::{OptBlockState, OptParserStrategy},
        params::{CompressionParameters, Strategy},
        sequence_store::RepeatOffsets,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OptFrameStrategy {
    BtOpt,
    BtUltra,
    BtUltra2,
}

pub(super) fn selected_opt_frame_strategy(strategy: Strategy) -> OptFrameStrategy {
    match strategy {
        Strategy::BtOpt => OptFrameStrategy::BtOpt,
        Strategy::BtUltra => OptFrameStrategy::BtUltra,
        Strategy::BtUltra2 => OptFrameStrategy::BtUltra2,
        _ => unreachable!("only optimal strategies route through opt_frame"),
    }
}

pub(super) fn opt_parser_strategy(strategy: OptFrameStrategy) -> OptParserStrategy {
    match strategy {
        OptFrameStrategy::BtOpt => OptParserStrategy::BtOpt,
        OptFrameStrategy::BtUltra | OptFrameStrategy::BtUltra2 => OptParserStrategy::BtUltra,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn encode_block_opt_no_dict_with_state(
    source: GreedyBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    opt_state: &mut OptBlockState,
    context: GreedyBlockEncodeContext<'_, '_>,
    strategy: OptFrameStrategy,
    block_encode_mode: BlockEncodeMode,
    policy: BlockEncodingPolicy,
    ldm_cursor: Option<&mut LdmOptCursor<'_>>,
) -> GreedyEncodedBlock {
    let target_mode = block_encode_mode.target_c_block_size().is_some();
    match strategy {
        OptFrameStrategy::BtOpt => {
            if ldm_cursor.is_some() || target_mode {
                encode_block_opt_no_dict_with_state_and_policy_and_ldm_in_mode(
                    source,
                    last_block,
                    params,
                    config,
                    repeat_offsets,
                    opt_state,
                    context,
                    OptParserStrategy::BtOpt,
                    block_encode_mode,
                    policy,
                    ldm_cursor,
                )
            } else {
                encode_block_btopt_no_dict_with_state_and_policy(
                    source,
                    last_block,
                    params,
                    config,
                    repeat_offsets,
                    opt_state,
                    context,
                    block_encode_mode.split_block_enabled(),
                    policy,
                )
            }
        }
        OptFrameStrategy::BtUltra | OptFrameStrategy::BtUltra2 => {
            debug_assert!(matches!(
                params.strategy,
                Strategy::BtUltra | Strategy::BtUltra2
            ));
            if strategy == OptFrameStrategy::BtUltra2 || ldm_cursor.is_some() || target_mode {
                encode_block_opt_no_dict_with_state_and_policy_and_ldm_in_mode(
                    source,
                    last_block,
                    params,
                    config,
                    repeat_offsets,
                    opt_state,
                    context,
                    OptParserStrategy::BtUltra,
                    block_encode_mode,
                    policy,
                    ldm_cursor,
                )
            } else {
                encode_block_btultra_no_dict_with_state_and_policy(
                    source,
                    last_block,
                    params,
                    config,
                    repeat_offsets,
                    opt_state,
                    context,
                    block_encode_mode.split_block_enabled(),
                    policy,
                )
            }
        }
    }
}
