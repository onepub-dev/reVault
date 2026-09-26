//! Target-compressed-block-size helpers ported from
//! `zstd_compress_superblock.c`.

use alloc::vec::Vec;

#[cfg(test)]
pub(super) use super::superblock_sequences::need_sequence_entropy_tables;
pub(super) use super::superblock_sequences::{
    append_sub_block_sequences, append_supported_sub_block_sequences, select_sequence_entropy_modes,
};
pub(super) use super::superblock_sequences::{
    append_supported_sub_block_sequences_with_tables, build_compressed_sequence_tables_for_modes,
    estimate_sequence_entropy_section_size, finish_sequence_entropy_tables_after_superblock,
    prime_sequence_entropy_tables_for_repeat,
};
use super::{block_policy::min_compression_gain, params::Strategy};
use crate::{
    bit_io::BitWriter,
    blocks::block::BlockType,
    encoding::{
        block_header::BlockHeader,
        blocks::PreparedSequence,
        frame_compressor::{FseTables, OffsetHistory},
    },
};

const BYTESCALE: usize = 256;
const ENTROPY_HEADER_BUDGET: usize = 120 * BYTESCALE;
const BLOCK_HEADER_SIZE: usize = 3;
const LITERAL_HEADER_ENTROPY_GUESS: usize = 200;

pub(super) const TARGET_CBLOCK_SIZE_MIN: usize = 1340;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct EstimatedSubBlockSize {
    pub(super) literal_size: usize,
    pub(super) block_size: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SubBlockBudgetPlan {
    pub(super) avg_lit_cost: usize,
    pub(super) avg_seq_cost: usize,
    pub(super) nb_sub_blocks: usize,
    pub(super) avg_block_budget: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EntropyTableMode {
    Basic,
    Rle,
    Compressed,
    Repeat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SequenceEntropyModes {
    pub(super) ll: EntropyTableMode,
    pub(super) ml: EntropyTableMode,
    pub(super) of: EntropyTableMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PlannedSubBlock {
    pub(super) start_sequence: usize,
    pub(super) end_sequence: usize,
    pub(super) literal_size: usize,
    pub(super) decompressed_size: usize,
    pub(super) last: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SubBlockLiteralEmission {
    pub(super) byte_size: usize,
    pub(super) entropy_written: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SubBlockSequenceEmission {
    pub(super) byte_size: usize,
    pub(super) entropy_written: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SubBlockEmission {
    pub(super) byte_size: usize,
    pub(super) literal_entropy_written: bool,
    pub(super) sequence_entropy_written: bool,
}

// Preserve the C-shaped guarded division used by this benchmark-sensitive path.
#[allow(unknown_lints)]
#[allow(clippy::manual_checked_ops)]
pub(super) fn sub_block_budget_plan(
    estimate: EstimatedSubBlockSize,
    nb_literals: usize,
    nb_sequences: usize,
    target_c_block_size: usize,
    src_size: usize,
) -> Option<SubBlockBudgetPlan> {
    debug_assert!(nb_sequences > 0);
    debug_assert!(estimate.literal_size <= estimate.block_size);
    if estimate.block_size > src_size {
        return None;
    }

    let target = target_c_block_size.max(TARGET_CBLOCK_SIZE_MIN);
    let nb_sub_blocks = target_sub_block_count(estimate.block_size, target);
    let avg_lit_cost = if nb_literals > 0 {
        (estimate.literal_size * BYTESCALE) / nb_literals
    } else {
        BYTESCALE
    };
    let avg_seq_cost = ((estimate.block_size - estimate.literal_size) * BYTESCALE) / nb_sequences;
    let avg_block_budget = (estimate.block_size * BYTESCALE) / nb_sub_blocks;

    Some(SubBlockBudgetPlan {
        avg_lit_cost,
        avg_seq_cost,
        nb_sub_blocks,
        avg_block_budget,
    })
}

pub(super) fn should_commit_sub_block(compressed_size: usize, decompressed_size: usize) -> bool {
    compressed_size > 0 && compressed_size < decompressed_size
}

pub(super) fn should_accept_target_superblock(
    compressed_size: usize,
    src_size: usize,
    strategy: Strategy,
) -> bool {
    let max_compressed_size = src_size.saturating_sub(min_compression_gain(src_size, strategy));
    compressed_size > 0 && compressed_size < max_compressed_size + BLOCK_HEADER_SIZE
}

pub(super) fn sub_block_literal_header_size(literal_size: usize, write_entropy: bool) -> usize {
    let entropy_guess = if write_entropy {
        LITERAL_HEADER_ENTROPY_GUESS
    } else {
        0
    };
    3 + usize::from(literal_size >= 1024 - entropy_guess)
        + usize::from(literal_size >= 16 * 1024 - entropy_guess)
}

pub(super) fn append_sub_block_literals(
    literals: &[u8],
    mode: EntropyTableMode,
    _write_entropy: bool,
    output: &mut Vec<u8>,
) -> Option<SubBlockLiteralEmission> {
    let start = output.len();
    let mut writer = BitWriter::from(output);
    match mode {
        EntropyTableMode::Basic => write_raw_literals(literals, &mut writer),
        EntropyTableMode::Rle => {
            if literals.is_empty() {
                write_raw_literals(literals, &mut writer);
            } else {
                write_rle_literals(literals, &mut writer);
            }
        }
        EntropyTableMode::Compressed | EntropyTableMode::Repeat => return None,
    }
    writer.flush();

    Some(SubBlockLiteralEmission {
        byte_size: writer.index() / 8 - start,
        entropy_written: false,
    })
}

pub(super) fn append_literal_only_sub_block(
    literals: &[u8],
    last_block: bool,
    literal_mode: EntropyTableMode,
    sequence_modes: SequenceEntropyModes,
    write_literal_entropy: bool,
    write_sequence_entropy: bool,
    output: &mut Vec<u8>,
) -> Option<SubBlockEmission> {
    let block_start = output.len();
    output.extend_from_slice(&[0; BLOCK_HEADER_SIZE]);
    let content_start = output.len();
    let Some(literal_emission) =
        append_sub_block_literals(literals, literal_mode, write_literal_entropy, output)
    else {
        output.truncate(block_start);
        return None;
    };
    let Some(sequence_emission) =
        append_sub_block_sequences(&[], sequence_modes, write_sequence_entropy, output)
    else {
        output.truncate(block_start);
        return None;
    };

    let content_size = output.len() - content_start;
    let header = BlockHeader {
        last_block,
        block_type: BlockType::Compressed,
        block_size: content_size as u32,
    };
    output[block_start..content_start].copy_from_slice(&header.serialize_to_bytes());

    Some(SubBlockEmission {
        byte_size: BLOCK_HEADER_SIZE + content_size,
        literal_entropy_written: literal_emission.entropy_written,
        sequence_entropy_written: sequence_emission.entropy_written,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_sequence_sub_block(
    literals: &[u8],
    sequences: &[PreparedSequence],
    last_block: bool,
    literal_mode: EntropyTableMode,
    sequence_modes: SequenceEntropyModes,
    write_literal_entropy: bool,
    write_sequence_entropy: bool,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    output: &mut Vec<u8>,
) -> Option<SubBlockEmission> {
    let previous_offsets = *offset_history;
    let block_start = output.len();
    output.extend_from_slice(&[0; BLOCK_HEADER_SIZE]);
    let content_start = output.len();
    let Some(literal_emission) =
        append_sub_block_literals(literals, literal_mode, write_literal_entropy, output)
    else {
        output.truncate(block_start);
        return None;
    };
    let Some(sequence_emission) = append_supported_sub_block_sequences(
        sequences,
        sequence_modes,
        write_sequence_entropy,
        fse_tables,
        offset_history,
        output,
    ) else {
        *offset_history = previous_offsets;
        output.truncate(block_start);
        return None;
    };

    let content_size = output.len() - content_start;
    let header = BlockHeader {
        last_block,
        block_type: BlockType::Compressed,
        block_size: content_size as u32,
    };
    output[block_start..content_start].copy_from_slice(&header.serialize_to_bytes());

    Some(SubBlockEmission {
        byte_size: BLOCK_HEADER_SIZE + content_size,
        literal_entropy_written: literal_emission.entropy_written,
        sequence_entropy_written: sequence_emission.entropy_written,
    })
}

fn write_raw_literals(literals: &[u8], writer: &mut BitWriter<&mut Vec<u8>>) {
    writer.write_bits(0u8, 2);
    write_raw_or_rle_literal_size(literals.len(), writer);
    writer.append_bytes(literals);
}

fn write_rle_literals(literals: &[u8], writer: &mut BitWriter<&mut Vec<u8>>) {
    debug_assert!(!literals.is_empty());
    writer.write_bits(1u8, 2);
    write_raw_or_rle_literal_size(literals.len(), writer);
    writer.write_bits(literals[0], 8);
}

fn write_raw_or_rle_literal_size(len: usize, writer: &mut BitWriter<&mut Vec<u8>>) {
    match len {
        0..=31 => {
            writer.write_bits(0u8, 1);
            writer.write_bits(len as u32, 5);
        }
        32..=4095 => {
            writer.write_bits(0b01u8, 2);
            writer.write_bits(len as u32, 12);
        }
        4096..=1_048_575 => {
            writer.write_bits(0b11u8, 2);
            writer.write_bits(len as u32, 20);
        }
        _ => panic!("literal section exceeds zstd raw/RLE literals size limit"),
    }
}

pub(super) fn plan_sub_blocks(
    sequences: &[PreparedSequence],
    total_literal_size: usize,
    plan: SubBlockBudgetPlan,
    mut compressed_size_for: impl FnMut(PlannedSubBlock) -> usize,
) -> Vec<PlannedSubBlock> {
    debug_assert!(!sequences.is_empty());

    let mut sub_blocks = Vec::new();
    let mut start_sequence = 0;
    let mut literal_consumed = 0;

    for block_idx in 0..plan.nb_sub_blocks.saturating_sub(1) {
        let remaining = &sequences[start_sequence..];
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
        if end_sequence == sequences.len() {
            break;
        }

        let candidate = planned_sub_block(
            sequences,
            start_sequence,
            end_sequence,
            count_literals(&sequences[start_sequence..end_sequence]),
            false,
        );
        let compressed_size = compressed_size_for(candidate);
        if should_commit_sub_block(compressed_size, candidate.decompressed_size) {
            literal_consumed += candidate.literal_size;
            sub_blocks.push(candidate);
            start_sequence = end_sequence;
        }
    }

    debug_assert!(literal_consumed <= total_literal_size);
    sub_blocks.push(planned_sub_block(
        sequences,
        start_sequence,
        sequences.len(),
        total_literal_size - literal_consumed,
        true,
    ));
    sub_blocks
}

fn planned_sub_block(
    sequences: &[PreparedSequence],
    start_sequence: usize,
    end_sequence: usize,
    literal_size: usize,
    last: bool,
) -> PlannedSubBlock {
    PlannedSubBlock {
        start_sequence,
        end_sequence,
        literal_size,
        decompressed_size: decompressed_size(
            &sequences[start_sequence..end_sequence],
            literal_size,
            last,
        ),
        last,
    }
}

pub(super) fn count_literals(sequences: &[PreparedSequence]) -> usize {
    sequences.iter().map(|sequence| sequence.ll as usize).sum()
}

pub(super) fn decompressed_size(
    sequences: &[PreparedSequence],
    literal_size: usize,
    last_sub_block: bool,
) -> usize {
    let match_length_sum = sequences
        .iter()
        .map(|sequence| sequence.ml as usize)
        .sum::<usize>();
    let literal_length_sum = count_literals(sequences);
    if last_sub_block {
        debug_assert!(literal_length_sum <= literal_size);
    } else {
        debug_assert_eq!(literal_length_sum, literal_size);
    }
    match_length_sum + literal_size
}

pub(super) fn target_sub_block_count(
    estimated_block_size: usize,
    target_c_block_size: usize,
) -> usize {
    let target = target_c_block_size.max(TARGET_CBLOCK_SIZE_MIN);
    ((estimated_block_size + (target / 2)) / target).max(1)
}

pub(super) fn size_block_sequences(
    sequences: &[PreparedSequence],
    target_budget: usize,
    avg_lit_cost: usize,
    avg_seq_cost: usize,
    first_sub_block: bool,
) -> usize {
    debug_assert!(!sequences.is_empty());

    let mut budget = if first_sub_block {
        ENTROPY_HEADER_BUDGET
    } else {
        0
    };

    budget += sequences[0].ll as usize * avg_lit_cost + avg_seq_cost;
    if budget > target_budget {
        return 1;
    }

    let mut in_size = sequences[0].ll as usize + sequences[0].ml as usize;
    for (idx, sequence) in sequences.iter().enumerate().skip(1) {
        let current_cost = sequence.ll as usize * avg_lit_cost + avg_seq_cost;
        budget += current_cost;
        in_size += sequence.ll as usize + sequence.ml as usize;
        if budget > target_budget && budget < in_size * BYTESCALE {
            return idx;
        }
    }

    sequences.len()
}

#[cfg(test)]
mod tests;
