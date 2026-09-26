//! Single-sub-block target-compressed-size candidates.

use super::{
    greedy_block::{GreedyEncodedBlock, GreedyPreparedBlock},
    params::Strategy,
    superblock::{
        append_sequence_sub_block, append_supported_sub_block_sequences, should_commit_sub_block,
        EntropyTableMode, SequenceEntropyModes,
    },
    target_modes::sequence_modes_clear_previous,
};
use crate::{
    blocks::block::BlockType,
    encoding::{
        block_header::BlockHeader,
        blocks::{append_huffman_literal_section_with_optimal_depth, HuffmanLiteralMode},
        frame_compressor::{FseTables, OffsetHistory},
    },
    huff0::huff0_encoder::HuffmanTable,
};

const BLOCK_HEADER_SIZE: usize = 3;

#[allow(clippy::too_many_arguments)]
pub(super) fn try_huffman_literal_only_sub_block(
    block: &[u8],
    last_block: bool,
    prepared: &GreedyPreparedBlock,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    bytes: &[u8],
    strategy: Strategy,
    repeat_offsets: super::sequence_store::RepeatOffsets,
) -> Option<GreedyEncodedBlock> {
    let previous_offsets = *offset_history;
    let previous_fse = fse_tables.snapshot_previous();
    let mut candidate = bytes.to_vec();
    let block_start = candidate.len();
    candidate.extend_from_slice(&[0; BLOCK_HEADER_SIZE]);
    let content_start = candidate.len();
    let Some(literal_emission) = append_huffman_literal_section_with_optimal_depth(
        prepared.prepared.literals.as_slice(),
        None,
        HuffmanLiteralMode::Compressed,
        strategy >= Strategy::BtUltra,
        &mut candidate,
    ) else {
        fse_tables.restore_previous(previous_fse);
        *offset_history = previous_offsets;
        return None;
    };
    let Some(sequence_emission) = append_supported_sub_block_sequences(
        &[],
        super::target_modes::basic_sequence_modes(),
        false,
        fse_tables,
        offset_history,
        &mut candidate,
    ) else {
        fse_tables.restore_previous(previous_fse);
        *offset_history = previous_offsets;
        return None;
    };

    let content_size = candidate.len() - content_start;
    debug_assert_eq!(
        literal_emission.byte_size + sequence_emission.byte_size,
        content_size
    );
    let header = BlockHeader {
        last_block,
        block_type: BlockType::Compressed,
        block_size: content_size as u32,
    };
    candidate[block_start..content_start].copy_from_slice(&header.serialize_to_bytes());
    if !should_commit_sub_block(BLOCK_HEADER_SIZE + content_size, block.len()) {
        fse_tables.restore_previous(previous_fse);
        *offset_history = previous_offsets;
        return None;
    }

    Some(GreedyEncodedBlock {
        bytes: candidate,
        repeat_offsets,
        new_huffman_table: literal_emission.new_huffman_table,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn try_huffman_sequence_sub_block(
    block: &[u8],
    last_block: bool,
    prepared: &GreedyPreparedBlock,
    previous_huff_table: Option<&HuffmanTable>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    bytes: &[u8],
    literal_mode: HuffmanLiteralMode,
    strategy: Strategy,
    sequence_modes: SequenceEntropyModes,
) -> Option<GreedyEncodedBlock> {
    let previous_offsets = *offset_history;
    let previous_fse = fse_tables.snapshot_previous();
    let mut candidate = bytes.to_vec();
    let block_start = candidate.len();
    candidate.extend_from_slice(&[0; BLOCK_HEADER_SIZE]);
    let content_start = candidate.len();
    let Some(literal_emission) = append_huffman_literal_section_with_optimal_depth(
        prepared.prepared.literals.as_slice(),
        previous_huff_table,
        literal_mode,
        strategy >= Strategy::BtUltra,
        &mut candidate,
    ) else {
        fse_tables.restore_previous(previous_fse);
        *offset_history = previous_offsets;
        return None;
    };
    let Some(sequence_emission) = append_supported_sub_block_sequences(
        prepared.prepared.sequences.as_slice(),
        sequence_modes,
        true,
        fse_tables,
        offset_history,
        &mut candidate,
    ) else {
        fse_tables.restore_previous(previous_fse);
        *offset_history = previous_offsets;
        return None;
    };

    let content_size = candidate.len() - content_start;
    debug_assert_eq!(
        literal_emission.byte_size + sequence_emission.byte_size,
        content_size
    );
    let header = BlockHeader {
        last_block,
        block_type: BlockType::Compressed,
        block_size: content_size as u32,
    };
    candidate[block_start..content_start].copy_from_slice(&header.serialize_to_bytes());

    if should_commit_sub_block(BLOCK_HEADER_SIZE + content_size, block.len()) {
        if sequence_modes_clear_previous(sequence_modes) {
            fse_tables.reset();
        }
        Some(GreedyEncodedBlock {
            bytes: candidate,
            repeat_offsets: prepared.repeat_offsets,
            new_huffman_table: literal_emission.new_huffman_table,
        })
    } else {
        fse_tables.restore_previous(previous_fse);
        *offset_history = previous_offsets;
        None
    }
}

pub(super) fn try_sequence_sub_block(
    block: &[u8],
    last_block: bool,
    prepared: &GreedyPreparedBlock,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    bytes: &[u8],
    sequence_modes: SequenceEntropyModes,
) -> Option<GreedyEncodedBlock> {
    let previous_offsets = *offset_history;
    let previous_fse = fse_tables.snapshot_previous();
    let mut candidate = bytes.to_vec();
    let Some(emission) = append_sequence_sub_block(
        prepared.prepared.literals.as_slice(),
        prepared.prepared.sequences.as_slice(),
        last_block,
        EntropyTableMode::Basic,
        sequence_modes,
        true,
        true,
        fse_tables,
        offset_history,
        &mut candidate,
    ) else {
        fse_tables.restore_previous(previous_fse);
        *offset_history = previous_offsets;
        return None;
    };
    if should_commit_sub_block(emission.byte_size, block.len()) {
        if sequence_modes_clear_previous(sequence_modes) {
            fse_tables.reset();
        }
        Some(GreedyEncodedBlock {
            bytes: candidate,
            repeat_offsets: prepared.repeat_offsets,
            new_huffman_table: None,
        })
    } else {
        fse_tables.restore_previous(previous_fse);
        *offset_history = previous_offsets;
        None
    }
}
