use crate::fse::fse_encoder::{
    build_probability_table_for_estimate, build_probability_table_for_estimate_with_scratch,
    build_table_from_probabilities, build_table_from_probabilities_with_scratch,
    normalize_counts_with_table_log, normalize_counts_with_table_log_into,
    normalize_u32_counts_with_table_log_into, optimal_table_log, optimal_table_log_u32, FSETable,
    FSETableBuildScratch,
};

use crate::blocks::sequence_section::Sequence;
use crate::encoding::blocks::compressed::sequence_cost::CodeCounts;

#[derive(Clone, Copy)]
pub(in crate::encoding::blocks::compressed) enum TableBuilder {
    Full,
    ProbabilityOnly,
}

pub(in crate::encoding::blocks::compressed) fn build_sequence_table(
    sequences: &[Sequence],
    code: impl Fn(&Sequence) -> u8 + Copy,
    max_log: u8,
    table_builder: TableBuilder,
) -> FSETable {
    let mut counts = [0usize; 256];
    for sequence in sequences {
        counts[usize::from(code(sequence))] += 1;
    }

    let last_code = usize::from(code(&sequences[sequences.len() - 1]));
    build_sequence_table_from_raw_counts(&mut counts, last_code, max_log, table_builder, None)
}

pub(in crate::encoding::blocks::compressed) fn build_sequence_table_from_counts(
    counts: &CodeCounts,
    last_code: u8,
    max_log: u8,
    table_builder: TableBuilder,
) -> FSETable {
    let mut counts = *counts.counts();
    build_sequence_table_from_raw_counts(
        &mut counts,
        usize::from(last_code),
        max_log,
        table_builder,
        None,
    )
}

pub(in crate::encoding::blocks::compressed) fn build_sequence_table_from_counts_with_scratch(
    counts: &CodeCounts,
    last_code: u8,
    max_log: u8,
    table_builder: TableBuilder,
    scratch: &mut FSETableBuildScratch,
) -> FSETable {
    let mut counts = *counts.counts();
    build_sequence_table_from_raw_counts(
        &mut counts,
        usize::from(last_code),
        max_log,
        table_builder,
        Some(scratch),
    )
}

/// DFast counterpart of C's `ZSTD_buildCTable()` input boundary.
///
/// The caller has already counted one of the bounded sequence alphabets and
/// carries C's `nbSeq`, `max`, and last symbol. Keeping those values avoids
/// widening the compact `unsigned` count lane to machine words or rescanning
/// it before normalization.
pub(in crate::encoding::blocks::compressed) fn build_c_fast_sequence_table_from_u32_counts(
    counts: &mut [u32],
    total: u32,
    max_symbol: u32,
    last_code: u32,
    max_log: u8,
    table_builder: TableBuilder,
) -> FSETable {
    build_c_fast_sequence_table_from_u32_counts_impl(
        counts,
        total,
        max_symbol,
        last_code,
        max_log,
        table_builder,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::encoding::blocks::compressed) fn build_c_fast_sequence_table_from_u32_counts_with_scratch(
    counts: &mut [u32],
    total: u32,
    max_symbol: u32,
    last_code: u32,
    max_log: u8,
    table_builder: TableBuilder,
    scratch: &mut FSETableBuildScratch,
) -> FSETable {
    build_c_fast_sequence_table_from_u32_counts_impl(
        counts,
        total,
        max_symbol,
        last_code,
        max_log,
        table_builder,
        Some(scratch),
    )
}

#[allow(clippy::too_many_arguments)]
fn build_c_fast_sequence_table_from_u32_counts_impl(
    counts: &mut [u32],
    total: u32,
    max_symbol: u32,
    last_code: u32,
    max_log: u8,
    table_builder: TableBuilder,
    scratch: Option<&mut FSETableBuildScratch>,
) -> FSETable {
    debug_assert!(total > 0);
    debug_assert!(last_code <= max_symbol);
    debug_assert!((max_symbol as usize) < counts.len());
    debug_assert_eq!(counts[..=max_symbol as usize].iter().sum::<u32>(), total);

    let table_log = optimal_table_log_u32(max_log, total, max_symbol);
    let last_code = last_code as usize;
    let adjusted_total = if counts[last_code] > 1 {
        counts[last_code] -= 1;
        total - 1
    } else {
        total
    };
    let low_prob_count = if adjusted_total >= 2048 { -1 } else { 1 };
    let counts = &counts[..=max_symbol as usize];
    let mut probabilities = [0_i32; 256];
    let probabilities = &mut probabilities[..counts.len()];
    let acc_log = normalize_u32_counts_with_table_log_into(
        counts,
        adjusted_total,
        table_log,
        max_log,
        low_prob_count,
        true,
        probabilities,
    );
    build_table_from_probabilities_with_builder_and_scratch(
        probabilities,
        acc_log,
        table_builder,
        scratch,
    )
}

fn build_sequence_table_from_raw_counts(
    counts: &mut [usize; 256],
    last_code: usize,
    max_log: u8,
    table_builder: TableBuilder,
    scratch: Option<&mut FSETableBuildScratch>,
) -> FSETable {
    let original_total = counts.iter().sum::<usize>();
    let original_max_symbol = counts
        .iter()
        .rposition(|count| *count != 0)
        .unwrap_or(last_code);
    let table_log = optimal_table_log(max_log, original_total, original_max_symbol);

    if counts[last_code] > 1 {
        counts[last_code] -= 1;
    }

    let max_symbol = counts
        .iter()
        .rposition(|count| *count != 0)
        .unwrap_or(last_code);
    let adjusted_total = counts[..=max_symbol].iter().sum::<usize>();
    let low_prob_count = if adjusted_total >= 2048 { -1 } else { 1 };
    let counts = &counts[..=max_symbol];
    let mut probabilities = [0_i32; 256];
    let probabilities = &mut probabilities[..counts.len()];
    let acc_log = if scratch.is_some() {
        normalize_counts_with_table_log_into(
            counts,
            adjusted_total,
            table_log,
            max_log,
            low_prob_count,
            true,
            probabilities,
        )
    } else {
        let (owned, acc_log) = normalize_counts_with_table_log(
            counts,
            adjusted_total,
            table_log,
            max_log,
            low_prob_count,
            true,
        );
        probabilities.copy_from_slice(&owned);
        acc_log
    };
    build_table_from_probabilities_with_builder_and_scratch(
        probabilities,
        acc_log,
        table_builder,
        scratch,
    )
}

pub(in crate::encoding::blocks::compressed) fn build_table_from_probabilities_with_builder(
    probs: &[i32],
    acc_log: u8,
    table_builder: TableBuilder,
) -> FSETable {
    build_table_from_probabilities_with_builder_and_scratch(probs, acc_log, table_builder, None)
}

pub(in crate::encoding::blocks::compressed) fn build_table_from_probabilities_with_builder_and_scratch(
    probs: &[i32],
    acc_log: u8,
    table_builder: TableBuilder,
    scratch: Option<&mut FSETableBuildScratch>,
) -> FSETable {
    match table_builder {
        TableBuilder::Full => match scratch {
            Some(scratch) => build_table_from_probabilities_with_scratch(probs, acc_log, scratch),
            None => build_table_from_probabilities(probs, acc_log),
        },
        TableBuilder::ProbabilityOnly => match scratch {
            Some(scratch) => {
                build_probability_table_for_estimate_with_scratch(probs, acc_log, scratch)
            }
            None => build_probability_table_for_estimate(probs, acc_log),
        },
    }
}
