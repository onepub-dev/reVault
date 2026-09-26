use crate::{
    encoding::blocks::compressed::sequence_cost::{
        cross_entropy_cost_up_to, entropy_cost_up_to, repeat_table_cost_up_to, CodeCounts,
    },
    fse::fse_encoder::{
        ncount_cost_from_probabilities, normalized_probabilities_from_counts,
        normalized_probabilities_from_counts_into, FSETable, FSETableBuildScratch,
    },
};

use crate::encoding::blocks::compressed::sequence_tables::{
    builder::{
        build_sequence_table_from_counts, build_sequence_table_from_counts_with_scratch,
        build_table_from_probabilities_with_builder,
        build_table_from_probabilities_with_builder_and_scratch, TableBuilder,
    },
    FseTableMode,
};

use super::TableSelectionPolicy;

#[allow(clippy::too_many_arguments)]
pub(super) fn choose_c_fast_table<'a>(
    previous: Option<&'a FSETable>,
    previous_repeat_valid: bool,
    default_table: &'a FSETable,
    sequence_count: usize,
    last_code: u8,
    counts: &CodeCounts,
    max_log: u8,
    repeat_table_max_sequences: usize,
    predefined_max_sequences: usize,
    default_norm_log: u8,
    table_builder: TableBuilder,
) -> FseTableMode<'a> {
    choose_c_fast_table_impl(
        previous,
        previous_repeat_valid,
        default_table,
        sequence_count,
        last_code,
        counts,
        max_log,
        repeat_table_max_sequences,
        predefined_max_sequences,
        default_norm_log,
        table_builder,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn choose_c_fast_table_with_scratch<'a>(
    previous: Option<&'a FSETable>,
    previous_repeat_valid: bool,
    default_table: &'a FSETable,
    sequence_count: usize,
    last_code: u8,
    counts: &CodeCounts,
    max_log: u8,
    repeat_table_max_sequences: usize,
    predefined_max_sequences: usize,
    default_norm_log: u8,
    table_builder: TableBuilder,
    scratch: &mut FSETableBuildScratch,
) -> FseTableMode<'a> {
    choose_c_fast_table_impl(
        previous,
        previous_repeat_valid,
        default_table,
        sequence_count,
        last_code,
        counts,
        max_log,
        repeat_table_max_sequences,
        predefined_max_sequences,
        default_norm_log,
        table_builder,
        Some(scratch),
    )
}

#[allow(clippy::too_many_arguments)]
fn choose_c_fast_table_impl<'a>(
    previous: Option<&'a FSETable>,
    previous_repeat_valid: bool,
    default_table: &'a FSETable,
    sequence_count: usize,
    last_code: u8,
    counts: &CodeCounts,
    max_log: u8,
    repeat_table_max_sequences: usize,
    predefined_max_sequences: usize,
    default_norm_log: u8,
    table_builder: TableBuilder,
    scratch: Option<&mut FSETableBuildScratch>,
) -> FseTableMode<'a> {
    if counts.most_frequent() == counts.total() {
        let first_code = counts.max_symbol() as u8;
        let default_allowed = default_table.can_encode_symbol(first_code);
        return if default_allowed && sequence_count <= 2 {
            FseTableMode::Predefined(default_table)
        } else {
            FseTableMode::Rle(first_code)
        };
    }

    let default_allowed = counts.default_allowed(default_table);

    if default_allowed {
        if let Some(previous) = previous {
            if previous_repeat_valid
                && sequence_count < repeat_table_max_sequences
                && counts.default_allowed(previous)
            {
                return FseTableMode::RepeatLast(previous);
            }
        }

        if sequence_count < predefined_max_sequences
            || counts.most_frequent() < (sequence_count >> (usize::from(default_norm_log) - 1))
        {
            return FseTableMode::Predefined(default_table);
        }
    }

    FseTableMode::Encoded(match scratch {
        Some(scratch) => build_sequence_table_from_counts_with_scratch(
            counts,
            last_code,
            max_log,
            table_builder,
            scratch,
        ),
        None => build_sequence_table_from_counts(counts, last_code, max_log, table_builder),
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn choose_c_cost_table<'a>(
    previous: Option<&'a FSETable>,
    default_table: &'a FSETable,
    sequence_count: usize,
    last_code: u8,
    counts: &CodeCounts,
    max_log: u8,
    table_builder: TableBuilder,
) -> FseTableMode<'a> {
    if counts.most_frequent() == counts.total() {
        let first_code = counts.max_symbol() as u8;
        let default_allowed = default_table.can_encode_symbol(first_code);
        return if default_allowed && sequence_count <= 2 {
            FseTableMode::Predefined(default_table)
        } else {
            FseTableMode::Rle(first_code)
        };
    }

    let max_symbol = counts.max_symbol();
    let (encoded_probs, encoded_acc_log) =
        normalized_probabilities_from_counts(&counts.counts()[..=max_symbol], max_log, true);
    let compressed_cost = ncount_cost_from_probabilities(&encoded_probs, encoded_acc_log)
        + entropy_cost_up_to(counts, max_symbol);
    let basic_cost = cross_entropy_cost_up_to(default_table, counts, max_symbol);
    let repeat_cost =
        previous.and_then(|previous| repeat_table_cost_up_to(previous, counts, max_symbol));

    if basic_cost.is_some_and(|basic_cost| {
        basic_cost <= repeat_cost.unwrap_or(usize::MAX) && basic_cost <= compressed_cost
    }) {
        return FseTableMode::Predefined(default_table);
    }

    if let Some(previous) = previous {
        if repeat_cost.is_some_and(|repeat_cost| repeat_cost <= compressed_cost) {
            return FseTableMode::RepeatLast(previous);
        }
    }

    let encoded_table = if counts.counts()[usize::from(last_code)] > 1 {
        build_sequence_table_from_counts(counts, last_code, max_log, table_builder)
    } else {
        build_table_from_probabilities_with_builder(&encoded_probs, encoded_acc_log, table_builder)
    };
    FseTableMode::Encoded(encoded_table)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn choose_c_cost_table_with_scratch<'a>(
    previous: Option<&'a FSETable>,
    default_table: &'a FSETable,
    sequence_count: usize,
    last_code: u8,
    counts: &CodeCounts,
    max_log: u8,
    table_builder: TableBuilder,
    scratch: &mut FSETableBuildScratch,
) -> FseTableMode<'a> {
    if counts.most_frequent() == counts.total() {
        let first_code = counts.max_symbol() as u8;
        let default_allowed = default_table.can_encode_symbol(first_code);
        return if default_allowed && sequence_count <= 2 {
            FseTableMode::Predefined(default_table)
        } else {
            FseTableMode::Rle(first_code)
        };
    }

    let max_symbol = counts.max_symbol();
    let raw_counts = &counts.counts()[..=max_symbol];
    let mut encoded_probs = [0_i32; 256];
    let encoded_probs = &mut encoded_probs[..raw_counts.len()];
    let encoded_acc_log =
        normalized_probabilities_from_counts_into(raw_counts, max_log, true, encoded_probs);
    let compressed_cost = ncount_cost_from_probabilities(encoded_probs, encoded_acc_log)
        + entropy_cost_up_to(counts, max_symbol);
    let basic_cost = cross_entropy_cost_up_to(default_table, counts, max_symbol);
    let repeat_cost =
        previous.and_then(|previous| repeat_table_cost_up_to(previous, counts, max_symbol));

    if basic_cost.is_some_and(|basic_cost| {
        basic_cost <= repeat_cost.unwrap_or(usize::MAX) && basic_cost <= compressed_cost
    }) {
        return FseTableMode::Predefined(default_table);
    }

    if let Some(previous) = previous {
        if repeat_cost.is_some_and(|repeat_cost| repeat_cost <= compressed_cost) {
            return FseTableMode::RepeatLast(previous);
        }
    }

    let encoded_table = if raw_counts[usize::from(last_code)] > 1 {
        build_sequence_table_from_counts_with_scratch(
            counts,
            last_code,
            max_log,
            table_builder,
            scratch,
        )
    } else {
        build_table_from_probabilities_with_builder_and_scratch(
            encoded_probs,
            encoded_acc_log,
            table_builder,
            Some(scratch),
        )
    };
    FseTableMode::Encoded(encoded_table)
}

pub(super) fn sequence_table_selection_policy(
    c_fast_heuristics: bool,
    c_cost_model: bool,
    default_norm_log: u8,
) -> TableSelectionPolicy {
    if c_fast_heuristics {
        TableSelectionPolicy::CFast { default_norm_log }
    } else if c_cost_model {
        TableSelectionPolicy::CCost
    } else {
        TableSelectionPolicy::Legacy
    }
}
