//! Final target-block acceptance gate ported from
//! `ZSTD_compressBlock_targetCBlockSize_body()`.

use alloc::vec::Vec;

use super::{
    block_emit::append_raw_block, greedy_block::GreedyEncodedBlock, params::Strategy,
    sequence_store::RepeatOffsets, superblock::should_accept_target_superblock,
};
use crate::encoding::frame_compressor::{FseTableSnapshot, FseTables, OffsetHistory};

pub(super) struct TargetAcceptanceContext<'a, 'b> {
    pub(super) block: &'a [u8],
    pub(super) last_block: bool,
    pub(super) strategy: Strategy,
    pub(super) repeat_offsets: RepeatOffsets,
    pub(super) initial_bytes: &'a [u8],
    pub(super) fse_tables: &'b mut FseTables,
    pub(super) offset_history: &'b mut OffsetHistory,
    pub(super) previous_fse: FseTableSnapshot,
    pub(super) previous_offsets: OffsetHistory,
}

pub(super) fn accept_target_or_raw_fallback(
    encoded: GreedyEncodedBlock,
    context: TargetAcceptanceContext<'_, '_>,
) -> GreedyEncodedBlock {
    let compressed_size = encoded
        .bytes
        .len()
        .saturating_sub(context.initial_bytes.len());
    if should_accept_target_superblock(compressed_size, context.block.len(), context.strategy) {
        return encoded;
    }

    context.fse_tables.restore_previous(context.previous_fse);
    *context.offset_history = context.previous_offsets;
    encode_target_block_raw_fallback(
        context.block,
        context.last_block,
        context.repeat_offsets,
        context.initial_bytes.to_vec(),
    )
}

pub(super) fn encode_target_block_raw_fallback(
    block: &[u8],
    last_block: bool,
    repeat_offsets: RepeatOffsets,
    mut bytes: Vec<u8>,
) -> GreedyEncodedBlock {
    append_raw_block(block, last_block, &mut bytes);
    GreedyEncodedBlock {
        bytes,
        repeat_offsets,
        new_huffman_table: None,
    }
}
