use alloc::vec::Vec;

use crate::{
    blocks::sequence_section::{MAX_LITERAL_LENGTH_CODE, MAX_MATCH_LENGTH_CODE, MAX_OFFSET_CODE},
    encoding::{
        blocks::{literal_length_code, match_length_code, offset_code, PreparedBlockRef},
        frame_compressor::OffsetHistory,
    },
};

const BEST_SPLIT_MIN_SEQUENCES: usize = 300;
const BEST_SPLIT_MIN_ESTIMATED_GAIN_BITS: f64 = 512.0;
pub(super) const BEST_SPLIT_MAX_PARTITIONS: usize = 8;

#[derive(Clone, Copy)]
pub(super) struct PreparedRange<'a> {
    pub(super) prepared: PreparedBlockRef<'a>,
    pub(super) data: &'a [u8],
}

struct SplitEstimate {
    mid: usize,
}

fn best_mid_split_estimate(
    prepared: PreparedBlockRef<'_>,
    initial_offsets: OffsetHistory,
) -> Option<SplitEstimate> {
    if prepared.sequences.len() < BEST_SPLIT_MIN_SEQUENCES {
        return None;
    }

    let mid = best_split_mid_by_decompressed_bytes(prepared);
    if mid == 0 || mid == prepared.sequences.len() {
        return None;
    }

    let first_literals_len: usize = prepared.sequences[..mid]
        .iter()
        .map(|sequence| sequence.ll as usize)
        .sum();

    let mut whole_literal_counts = [0u32; 256];
    let mut first_literal_counts = [0u32; 256];
    let mut second_literal_counts = [0u32; 256];
    for (idx, &byte) in prepared.literals.iter().enumerate() {
        whole_literal_counts[byte as usize] += 1;
        if idx < first_literals_len {
            first_literal_counts[byte as usize] += 1;
        } else {
            second_literal_counts[byte as usize] += 1;
        }
    }

    let mut whole_ll_counts = [0u32; MAX_LITERAL_LENGTH_CODE as usize + 1];
    let mut first_ll_counts = [0u32; MAX_LITERAL_LENGTH_CODE as usize + 1];
    let mut second_ll_counts = [0u32; MAX_LITERAL_LENGTH_CODE as usize + 1];
    let mut whole_ml_counts = [0u32; MAX_MATCH_LENGTH_CODE as usize + 1];
    let mut first_ml_counts = [0u32; MAX_MATCH_LENGTH_CODE as usize + 1];
    let mut second_ml_counts = [0u32; MAX_MATCH_LENGTH_CODE as usize + 1];
    let mut whole_of_counts = [0u32; MAX_OFFSET_CODE as usize + 1];
    let mut first_of_counts = [0u32; MAX_OFFSET_CODE as usize + 1];
    let mut second_of_counts = [0u32; MAX_OFFSET_CODE as usize + 1];

    let mut offset_history = initial_offsets;
    for (idx, sequence) in prepared.sequences.iter().enumerate() {
        let ll_code = literal_length_code(sequence.ll);
        let ml_code = match_length_code(sequence.ml);
        let offset_value = offset_history.encode_offset_value(sequence.raw_offset, sequence.ll);
        let of_code = offset_code(offset_value);

        whole_ll_counts[ll_code as usize] += 1;
        whole_ml_counts[ml_code as usize] += 1;
        whole_of_counts[of_code as usize] += 1;

        let (ll_counts, ml_counts, of_counts) = if idx < mid {
            (
                &mut first_ll_counts,
                &mut first_ml_counts,
                &mut first_of_counts,
            )
        } else {
            (
                &mut second_ll_counts,
                &mut second_ml_counts,
                &mut second_of_counts,
            )
        };
        ll_counts[ll_code as usize] += 1;
        ml_counts[ml_code as usize] += 1;
        of_counts[of_code as usize] += 1;
    }

    let whole_bits = estimated_entropy_bits(&whole_literal_counts, prepared.literals.len())
        + estimated_entropy_bits(&whole_ll_counts, prepared.sequences.len())
        + estimated_entropy_bits(&whole_ml_counts, prepared.sequences.len())
        + estimated_entropy_bits(&whole_of_counts, prepared.sequences.len());
    let split_bits = estimated_entropy_bits(&first_literal_counts, first_literals_len)
        + estimated_entropy_bits(
            &second_literal_counts,
            prepared.literals.len().saturating_sub(first_literals_len),
        )
        + estimated_entropy_bits(&first_ll_counts, mid)
        + estimated_entropy_bits(&second_ll_counts, prepared.sequences.len() - mid)
        + estimated_entropy_bits(&first_ml_counts, mid)
        + estimated_entropy_bits(&second_ml_counts, prepared.sequences.len() - mid)
        + estimated_entropy_bits(&first_of_counts, mid)
        + estimated_entropy_bits(&second_of_counts, prepared.sequences.len() - mid);

    if split_bits + BEST_SPLIT_MIN_ESTIMATED_GAIN_BITS >= whole_bits {
        return None;
    }

    Some(SplitEstimate { mid })
}

pub(super) fn best_split_mid_by_decompressed_bytes(prepared: PreparedBlockRef<'_>) -> usize {
    let target = prepared_decompressed_len(prepared) / 2;
    split_mid_by_target_decompressed_bytes(prepared, target)
}

fn prepared_decompressed_len(prepared: PreparedBlockRef<'_>) -> usize {
    prepared.literals.len()
        + prepared
            .sequences
            .iter()
            .map(|sequence| sequence.ml as usize)
            .sum::<usize>()
}

fn split_mid_by_target_decompressed_bytes(prepared: PreparedBlockRef<'_>, target: usize) -> usize {
    let mut total = 0usize;
    let mut best_mid = prepared.sequences.len() / 2;
    let mut best_distance = usize::MAX;

    for (idx, sequence) in prepared.sequences.iter().enumerate() {
        total += sequence.ll as usize + sequence.ml as usize;
        let mid = idx + 1;
        if mid == prepared.sequences.len() {
            break;
        }

        let distance = total.abs_diff(target);
        if distance < best_distance {
            best_distance = distance;
            best_mid = mid;
        }
    }

    best_mid
}

pub(super) fn derive_best_partitions<'a>(
    range: PreparedRange<'a>,
    initial_offsets: OffsetHistory,
    remaining_partitions: usize,
    partitions: &mut Vec<PreparedRange<'a>>,
) -> usize {
    if remaining_partitions <= 1 {
        partitions.push(range);
        return 1;
    }

    let Some(estimate) = best_mid_split_estimate(range.prepared, initial_offsets) else {
        partitions.push(range);
        return 1;
    };

    let (first, second) = split_prepared_range(range, estimate.mid);
    let second_offsets = advance_offset_history(initial_offsets, first.prepared.sequences);
    let used_left =
        derive_best_partitions(first, initial_offsets, remaining_partitions - 1, partitions);
    let remaining_for_right = remaining_partitions.saturating_sub(used_left);
    if remaining_for_right == 0 {
        return used_left;
    }
    let used_right =
        derive_best_partitions(second, second_offsets, remaining_for_right, partitions);
    used_left + used_right
}

pub(super) fn advance_offset_history(
    mut offset_history: OffsetHistory,
    sequences: &[crate::encoding::blocks::PreparedSequence],
) -> OffsetHistory {
    for sequence in sequences {
        offset_history.update_after_match(sequence.raw_offset, sequence.ll != 0);
    }
    offset_history
}

fn estimated_entropy_bits(counts: &[u32], total: usize) -> f64 {
    if total <= 1 {
        return 0.0;
    }

    let total_log2 = entropy_log2(total);
    let mut bits = 0.0;
    for &count in counts {
        if count == 0 {
            continue;
        }
        let count_f = count as f64;
        bits += count_f * (total_log2 - entropy_log2(count as usize));
    }
    bits
}

#[cfg(feature = "std")]
fn entropy_log2(value: usize) -> f64 {
    (value as f64).log2()
}

#[cfg(not(feature = "std"))]
fn entropy_log2(value: usize) -> f64 {
    value.ilog2() as f64
}

pub(super) fn split_prepared_range(
    range: PreparedRange<'_>,
    mid: usize,
) -> (PreparedRange<'_>, PreparedRange<'_>) {
    let first_literals_len: usize = range.prepared.sequences[..mid]
        .iter()
        .map(|sequence| sequence.ll as usize)
        .sum();
    let first_match_len: usize = range.prepared.sequences[..mid]
        .iter()
        .map(|sequence| sequence.ml as usize)
        .sum();
    let first_data_len = first_literals_len + first_match_len;

    let first = PreparedRange {
        prepared: PreparedBlockRef {
            literals: &range.prepared.literals[..first_literals_len],
            sequences: &range.prepared.sequences[..mid],
        },
        data: &range.data[..first_data_len],
    };
    let second = PreparedRange {
        prepared: PreparedBlockRef {
            literals: &range.prepared.literals[first_literals_len..],
            sequences: &range.prepared.sequences[mid..],
        },
        data: &range.data[first_data_len..],
    };
    (first, second)
}
