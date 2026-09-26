//! Target-compressed-block-size adapter for fast blocks.

use alloc::vec::Vec;

use super::{FastBlockEncodeContext, FastBlockEncoding, FastPreparedBlock};
use crate::encoding::levels::c_port::{
    block_policy::BlockEncodingPolicy,
    greedy_block::{GreedyBlockEncodeContext, GreedyPreparedBlock},
    params::CompressionParameters,
    sequence_store::RepeatOffsets,
    target_block::{encode_target_block_with_superblock_fallback, TargetBlockOptions},
};

#[allow(clippy::too_many_arguments)]
pub(super) fn append_target_block(
    block: &[u8],
    last_block: bool,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    prepared: FastPreparedBlock,
    policy: BlockEncodingPolicy,
    context: FastBlockEncodeContext<'_, '_>,
    target_size: usize,
    output: &mut Vec<u8>,
) -> FastBlockEncoding {
    let encoded = encode_target_block_with_superblock_fallback(
        block,
        last_block,
        TargetBlockOptions {
            target_c_block_size: target_size,
            strategy: params.strategy,
            allow_rle: policy.allows_rle(),
            repeat_offsets,
        },
        &GreedyPreparedBlock {
            prepared: prepared.prepared,
            repeat_offsets: prepared.repeat_offsets,
        },
        GreedyBlockEncodeContext {
            previous_huff_table: context.previous_huff_table,
            fse_tables: context.fse_tables,
            offset_history: context.offset_history,
        },
        Vec::new(),
    );
    output.extend_from_slice(&encoded.bytes);
    FastBlockEncoding {
        repeat_offsets: encoded.repeat_offsets,
        new_huffman_table: encoded.new_huffman_table,
    }
}
