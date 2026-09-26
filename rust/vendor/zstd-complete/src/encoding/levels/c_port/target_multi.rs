//! Multi-sub-block target-compressed-size adapter.

use alloc::vec::Vec;

pub(super) use super::target_multi_basic::try_basic_literal_multi_sub_blocks;

use super::{
    block_emit::append_raw_block,
    greedy_block::{GreedyEncodedBlock, GreedyPreparedBlock},
    params::Strategy,
    sequence_store::{OffBase, RepeatOffsets},
    superblock::{
        append_supported_sub_block_sequences, append_supported_sub_block_sequences_with_tables,
        build_compressed_sequence_tables_for_modes, count_literals, decompressed_size,
        estimate_sequence_entropy_section_size, finish_sequence_entropy_tables_after_superblock,
        prime_sequence_entropy_tables_for_repeat, select_sequence_entropy_modes,
        should_commit_sub_block, size_block_sequences, sub_block_budget_plan, EntropyTableMode,
        EstimatedSubBlockSize, SequenceEntropyModes,
    },
};
use crate::{
    blocks::block::BlockType,
    encoding::{
        block_header::BlockHeader,
        blocks::{
            append_huffman_literal_section_with_table,
            build_huffman_literal_table_with_optimal_depth,
            estimate_huffman_literal_section_with_table, CompressedSequenceTables,
            HuffmanLiteralMode, PreparedSequence,
        },
        frame_compressor::{FseTables, OffsetHistory},
    },
    huff0::huff0_encoder::HuffmanTable,
};

const BLOCK_HEADER_SIZE: usize = 3;

#[derive(Clone, Copy)]
struct TargetSubBlockEmission {
    byte_size: usize,
    literal_entropy_written: bool,
    sequence_entropy_written: bool,
}

#[derive(Clone, Copy)]
pub(super) struct TargetMultiBlock<'a> {
    pub(super) block: &'a [u8],
    pub(super) last_block: bool,
    pub(super) target_c_block_size: usize,
    pub(super) strategy: Strategy,
    pub(super) initial_repeat_offsets: RepeatOffsets,
    pub(super) bytes: &'a [u8],
}

pub(super) fn try_huffman_literal_multi_sub_blocks(
    target: TargetMultiBlock<'_>,
    prepared: &GreedyPreparedBlock,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
) -> Option<GreedyEncodedBlock> {
    let huffman_table = build_huffman_literal_table_with_optimal_depth(
        prepared.prepared.literals.as_slice(),
        target.strategy >= Strategy::BtUltra,
    )?;
    let sequence_modes = select_sequence_entropy_modes(
        prepared.prepared.sequences.as_slice(),
        fse_tables,
        *offset_history,
        target.strategy,
    );
    let sequence_tables = build_compressed_sequence_tables_for_modes(
        prepared.prepared.sequences.as_slice(),
        sequence_modes,
        *offset_history,
    );
    let estimate = estimate_huffman_literal_sequence_block(
        prepared,
        &huffman_table,
        sequence_modes,
        sequence_tables.as_ref(),
        fse_tables,
        *offset_history,
    )?;
    let plan = sub_block_budget_plan(
        estimate,
        prepared.prepared.literals.len(),
        prepared.prepared.sequences.len(),
        target.target_c_block_size,
        target.block.len(),
    )?;
    if plan.nb_sub_blocks <= 1 {
        return None;
    }

    let previous_offsets = *offset_history;
    let previous_fse_tables = fse_tables.clone();
    let previous_fse = fse_tables.snapshot_previous();
    let mut candidate = target.bytes.to_vec();
    let mut literal_pos = 0usize;
    let mut start_sequence = 0usize;
    let mut decompressed_pos = 0usize;
    let mut literal_entropy_written = false;
    let mut sequence_entropy_written = false;

    for block_idx in 0..plan.nb_sub_blocks.saturating_sub(1) {
        let remaining = &prepared.prepared.sequences[start_sequence..];
        if remaining.is_empty() {
            break;
        }

        let sequence_count = size_block_sequences(
            remaining,
            plan.avg_block_budget,
            plan.avg_lit_cost,
            plan.avg_seq_cost,
            block_idx == 0,
        );
        let end_sequence = start_sequence + sequence_count;
        if end_sequence == prepared.prepared.sequences.len() {
            break;
        }

        let sequences = &prepared.prepared.sequences[start_sequence..end_sequence];
        let literal_size = count_literals(sequences);
        let decompressed_size = decompressed_size(sequences, literal_size, false);
        let Some(literals) = prepared
            .prepared
            .literals
            .get(literal_pos..literal_pos + literal_size)
        else {
            break;
        };
        let literal_mode = if literal_entropy_written {
            HuffmanLiteralMode::Repeat
        } else {
            HuffmanLiteralMode::Compressed
        };
        let write_sequence_entropy = !sequence_entropy_written;
        let sub_block_sequence_modes = if write_sequence_entropy {
            sequence_modes
        } else {
            repeat_sequence_modes()
        };
        let output_start = candidate.len();
        let offsets_before_sub_block = *offset_history;
        let fse_before_sub_block = fse_tables.snapshot_previous();
        let Some(emission) = append_huffman_sequence_sub_block(
            literals,
            sequences,
            false,
            &huffman_table,
            literal_mode,
            sequence_tables.as_ref(),
            sub_block_sequence_modes,
            write_sequence_entropy,
            fse_tables,
            offset_history,
            &mut candidate,
        ) else {
            candidate.truncate(output_start);
            fse_tables.restore_previous(fse_before_sub_block);
            *offset_history = offsets_before_sub_block;
            continue;
        };
        if should_commit_sub_block(emission.byte_size, decompressed_size) {
            if write_sequence_entropy
                && emission.sequence_entropy_written
                && prime_sequence_entropy_tables_for_repeat(
                    sequences,
                    sequence_modes,
                    sequence_tables.as_ref(),
                    fse_tables,
                    offsets_before_sub_block,
                )
                .is_none()
            {
                candidate.truncate(output_start);
                fse_tables.restore_previous(fse_before_sub_block);
                *offset_history = offsets_before_sub_block;
                continue;
            }
            literal_pos += literal_size;
            start_sequence = end_sequence;
            decompressed_pos += decompressed_size;
            if emission.literal_entropy_written {
                literal_entropy_written = true;
            }
            if emission.sequence_entropy_written {
                sequence_entropy_written = true;
            }
        } else {
            candidate.truncate(output_start);
            fse_tables.restore_previous(fse_before_sub_block);
            *offset_history = offsets_before_sub_block;
        }
    }

    if start_sequence == 0 {
        fse_tables.restore_previous(previous_fse);
        *offset_history = previous_offsets;
        return None;
    }

    let remaining_sequences = &prepared.prepared.sequences[start_sequence..];
    let remaining_literals = prepared.prepared.literals.get(literal_pos..)?;
    let remaining_decompressed = target.block.len().checked_sub(decompressed_pos)?;
    let literal_mode = if literal_entropy_written {
        HuffmanLiteralMode::Repeat
    } else {
        HuffmanLiteralMode::Compressed
    };
    let write_sequence_entropy = !sequence_entropy_written;
    let sub_block_sequence_modes = if write_sequence_entropy {
        sequence_modes
    } else {
        repeat_sequence_modes()
    };
    let output_start = candidate.len();
    let offsets_before_final = *offset_history;
    let fse_before_final = fse_tables.snapshot_previous();
    let Some(emission) = append_huffman_sequence_sub_block(
        remaining_literals,
        remaining_sequences,
        target.last_block,
        &huffman_table,
        literal_mode,
        sequence_tables.as_ref(),
        sub_block_sequence_modes,
        write_sequence_entropy,
        fse_tables,
        offset_history,
        &mut candidate,
    ) else {
        candidate.truncate(output_start);
        fse_tables.restore_previous(fse_before_final);
        *offset_history = offsets_before_final;
        append_raw_block(
            target.block.get(decompressed_pos..)?,
            target.last_block,
            &mut candidate,
        );
        finish_sequence_entropy_tables_after_superblock(
            sequence_modes,
            sequence_tables.as_ref(),
            &previous_fse_tables,
            fse_tables,
        )?;
        return Some(GreedyEncodedBlock {
            bytes: candidate,
            repeat_offsets: repeat_offsets_after_sequences(
                target.initial_repeat_offsets,
                &prepared.prepared.sequences[..start_sequence],
            ),
            new_huffman_table: literal_entropy_written.then_some(huffman_table),
        });
    };
    if !should_commit_sub_block(emission.byte_size, remaining_decompressed) {
        candidate.truncate(output_start);
        fse_tables.restore_previous(fse_before_final);
        *offset_history = offsets_before_final;
        append_raw_block(
            target.block.get(decompressed_pos..)?,
            target.last_block,
            &mut candidate,
        );
        finish_sequence_entropy_tables_after_superblock(
            sequence_modes,
            sequence_tables.as_ref(),
            &previous_fse_tables,
            fse_tables,
        )?;
        return Some(GreedyEncodedBlock {
            bytes: candidate,
            repeat_offsets: repeat_offsets_after_sequences(
                target.initial_repeat_offsets,
                &prepared.prepared.sequences[..start_sequence],
            ),
            new_huffman_table: literal_entropy_written.then_some(huffman_table),
        });
    }
    if emission.literal_entropy_written {
        literal_entropy_written = true;
    }

    finish_sequence_entropy_tables_after_superblock(
        sequence_modes,
        sequence_tables.as_ref(),
        &previous_fse_tables,
        fse_tables,
    )?;
    Some(GreedyEncodedBlock {
        bytes: candidate,
        repeat_offsets: prepared.repeat_offsets,
        new_huffman_table: literal_entropy_written.then_some(huffman_table),
    })
}

pub(super) fn repeat_offsets_after_sequences(
    mut repeat_offsets: RepeatOffsets,
    sequences: &[PreparedSequence],
) -> RepeatOffsets {
    for sequence in sequences {
        let off_base = OffBase::from_c_value(sequence.encoded_offset_value)
            .unwrap_or(OffBase::Offset(sequence.raw_offset));
        repeat_offsets.update(off_base, sequence.ll);
    }
    repeat_offsets
}

fn estimate_huffman_literal_sequence_block(
    prepared: &GreedyPreparedBlock,
    huffman_table: &HuffmanTable,
    sequence_modes: SequenceEntropyModes,
    sequence_tables: Option<&CompressedSequenceTables>,
    fse_tables: &FseTables,
    offset_history: OffsetHistory,
) -> Option<EstimatedSubBlockSize> {
    let literal_size = estimate_huffman_literal_section_with_table(
        prepared.prepared.literals.as_slice(),
        huffman_table,
        true,
    )?;
    let sequence_size = estimate_sequence_entropy_section_size(
        prepared.prepared.sequences.as_slice(),
        sequence_modes,
        sequence_tables,
        fse_tables,
        offset_history,
        true,
    )?;
    Some(EstimatedSubBlockSize {
        literal_size,
        block_size: BLOCK_HEADER_SIZE + literal_size + sequence_size,
    })
}

#[allow(clippy::too_many_arguments)]
fn append_huffman_sequence_sub_block(
    literals: &[u8],
    sequences: &[PreparedSequence],
    last_block: bool,
    huffman_table: &HuffmanTable,
    literal_mode: HuffmanLiteralMode,
    sequence_tables: Option<&CompressedSequenceTables>,
    sequence_modes: SequenceEntropyModes,
    write_sequence_entropy: bool,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    output: &mut Vec<u8>,
) -> Option<TargetSubBlockEmission> {
    let previous_offsets = *offset_history;
    let block_start = output.len();
    output.extend_from_slice(&[0; BLOCK_HEADER_SIZE]);
    let content_start = output.len();
    let Some(literal_emission) =
        append_huffman_literal_section_with_table(literals, huffman_table, literal_mode, output)
    else {
        output.truncate(block_start);
        return None;
    };
    let sequence_emission = if write_sequence_entropy {
        append_supported_sub_block_sequences_with_tables(
            sequences,
            sequence_modes,
            sequence_tables,
            true,
            fse_tables,
            offset_history,
            output,
        )
    } else {
        append_supported_sub_block_sequences(
            sequences,
            sequence_modes,
            false,
            fse_tables,
            offset_history,
            output,
        )
    };
    let Some(sequence_emission) = sequence_emission else {
        *offset_history = previous_offsets;
        output.truncate(block_start);
        return None;
    };

    let content_size = output.len() - content_start;
    debug_assert_eq!(
        literal_emission.byte_size + sequence_emission.byte_size,
        content_size
    );
    let header = BlockHeader {
        last_block,
        block_type: BlockType::Compressed,
        block_size: content_size as u32,
    };
    output[block_start..content_start].copy_from_slice(&header.serialize_to_bytes());

    Some(TargetSubBlockEmission {
        byte_size: BLOCK_HEADER_SIZE + content_size,
        literal_entropy_written: literal_emission.entropy_written,
        sequence_entropy_written: sequence_emission.entropy_written,
    })
}

fn repeat_sequence_modes() -> SequenceEntropyModes {
    SequenceEntropyModes {
        ll: EntropyTableMode::Repeat,
        ml: EntropyTableMode::Repeat,
        of: EntropyTableMode::Repeat,
    }
}
