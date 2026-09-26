use alloc::{vec, vec::Vec};

use crate::encoding::frame_compressor::OffsetHistory;
use crate::encoding::levels::c_port::sequence_store::StoredSequence;
use crate::{blocks::sequence_section::Sequence, fse::fse_encoder::FSETable};

use super::{
    builder::{build_sequence_table, build_sequence_table_from_counts, TableBuilder},
    FseTableMode, SequenceModeSearchConfig,
};
use crate::encoding::blocks::compressed::{
    c_sequence::CSequenceValues, literal_length_code, match_length_code, offset_code,
    sequence_cost::CodeCounts, PreparedSequence, EXACT_SEQUENCE_TABLE_MIN_LOG,
};
mod c_fast;
mod policy;

use crate::fse::fse_encoder::FSETableBuildScratch;
pub(in crate::encoding::blocks::compressed) use c_fast::{
    choose_c_dfast_compact_sequence_table_modes_from_prepared,
    choose_c_dfast_compact_sequence_table_modes_from_stored,
    choose_c_dfast_compact_sequence_table_modes_from_stored_with_final_history,
};
use policy::{
    choose_c_cost_table, choose_c_cost_table_with_scratch, choose_c_fast_table,
    choose_c_fast_table_with_scratch, sequence_table_selection_policy,
};

#[derive(Clone, Copy)]
struct TableModeCandidateConfig {
    max_log: u8,
    repeat_table_max_sequences: usize,
    predefined_max_sequences: usize,
    selection_policy: TableSelectionPolicy,
    table_builder: TableBuilder,
}

#[derive(Clone, Copy)]
enum TableSelectionPolicy {
    Legacy,
    CFast { default_norm_log: u8 },
    CCost,
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(in crate::encoding::blocks::compressed) fn choose_table<'a>(
    previous: Option<&'a FSETable>,
    default_table: &'a FSETable,
    sequences: &[Sequence],
    code: impl Fn(&Sequence) -> u8 + Copy,
    max_log: u8,
    repeat_table_max_sequences: usize,
    predefined_max_sequences: usize,
) -> FseTableMode<'a> {
    choose_table_with_policy(
        previous,
        false,
        default_table,
        sequences,
        code,
        max_log,
        repeat_table_max_sequences,
        predefined_max_sequences,
        TableSelectionPolicy::Legacy,
        TableBuilder::Full,
    )
}

#[allow(clippy::too_many_arguments)]
fn choose_table_with_policy<'a>(
    previous: Option<&'a FSETable>,
    previous_repeat_valid: bool,
    default_table: &'a FSETable,
    sequences: &[Sequence],
    code: impl Fn(&Sequence) -> u8 + Copy,
    max_log: u8,
    repeat_table_max_sequences: usize,
    predefined_max_sequences: usize,
    selection_policy: TableSelectionPolicy,
    table_builder: TableBuilder,
) -> FseTableMode<'a> {
    match selection_policy {
        TableSelectionPolicy::CFast { default_norm_log } => {
            let counts = CodeCounts::from_codes(sequences.iter().map(code));
            let last_code = code(&sequences[sequences.len() - 1]);
            return choose_c_fast_table(
                previous,
                previous_repeat_valid,
                default_table,
                sequences.len(),
                last_code,
                &counts,
                max_log,
                repeat_table_max_sequences,
                predefined_max_sequences,
                default_norm_log,
                table_builder,
            );
        }
        TableSelectionPolicy::CCost => {
            let counts = CodeCounts::from_codes(sequences.iter().map(code));
            let last_code = code(&sequences[sequences.len() - 1]);
            return choose_c_cost_table(
                previous,
                default_table,
                sequences.len(),
                last_code,
                &counts,
                max_log,
                table_builder,
            );
        }
        TableSelectionPolicy::Legacy => {}
    }

    choose_legacy_table(
        previous,
        default_table,
        sequences,
        code,
        max_log,
        repeat_table_max_sequences,
        predefined_max_sequences,
        table_builder,
    )
}

#[allow(clippy::too_many_arguments)]
fn choose_legacy_table<'a>(
    previous: Option<&'a FSETable>,
    default_table: &'a FSETable,
    sequences: &[Sequence],
    code: impl Fn(&Sequence) -> u8 + Copy,
    max_log: u8,
    repeat_table_max_sequences: usize,
    predefined_max_sequences: usize,
    table_builder: TableBuilder,
) -> FseTableMode<'a> {
    let first_code = code(&sequences[0]);
    let all_same_code = sequences
        .iter()
        .skip(1)
        .all(|sequence| code(sequence) == first_code);

    if all_same_code && sequences.len() > 2 {
        return FseTableMode::Rle(first_code);
    }

    if sequences.len() <= predefined_max_sequences
        && sequences
            .iter()
            .all(|sequence| default_table.can_encode_symbol(code(sequence)))
    {
        return FseTableMode::Predefined(default_table);
    }

    if all_same_code {
        return FseTableMode::Rle(first_code);
    }

    if let Some(previous) = previous {
        if sequences.len() < repeat_table_max_sequences
            && sequences
                .iter()
                .all(|sequence| previous.can_encode_symbol(code(sequence)))
        {
            return FseTableMode::RepeatLast(previous);
        }
    }

    FseTableMode::Encoded(build_sequence_table(
        sequences,
        code,
        max_log,
        table_builder,
    ))
}

#[allow(clippy::too_many_arguments)]
fn choose_c_fast_table_from_sequences<'a>(
    previous: Option<&'a FSETable>,
    previous_repeat_valid: bool,
    default_table: &'a FSETable,
    sequences: &[Sequence],
    code: impl Fn(&Sequence) -> u8 + Copy,
    max_log: u8,
    repeat_table_max_sequences: usize,
    predefined_max_sequences: usize,
    default_norm_log: u8,
    table_builder: TableBuilder,
) -> FseTableMode<'a> {
    let counts = CodeCounts::from_codes(sequences.iter().map(code));
    let last_code = code(&sequences[sequences.len() - 1]);
    choose_c_fast_table(
        previous,
        previous_repeat_valid,
        default_table,
        sequences.len(),
        last_code,
        &counts,
        max_log,
        repeat_table_max_sequences,
        predefined_max_sequences,
        default_norm_log,
        table_builder,
    )
}

#[allow(clippy::too_many_arguments)]
fn choose_c_cost_table_from_sequences<'a>(
    previous: Option<&'a FSETable>,
    default_table: &'a FSETable,
    sequences: &[Sequence],
    code: impl Fn(&Sequence) -> u8 + Copy,
    max_log: u8,
    table_builder: TableBuilder,
) -> FseTableMode<'a> {
    let counts = CodeCounts::from_codes(sequences.iter().map(code));
    let last_code = code(&sequences[sequences.len() - 1]);
    choose_c_cost_table(
        previous,
        default_table,
        sequences.len(),
        last_code,
        &counts,
        max_log,
        table_builder,
    )
}

#[allow(clippy::too_many_arguments)]
fn choose_table_with_policy_from_counts<'a>(
    previous: Option<&'a FSETable>,
    previous_repeat_valid: bool,
    default_table: &'a FSETable,
    sequence_count: usize,
    last_code: u8,
    counts: &CodeCounts,
    max_log: u8,
    repeat_table_max_sequences: usize,
    predefined_max_sequences: usize,
    selection_policy: TableSelectionPolicy,
    table_builder: TableBuilder,
    scratch: Option<&mut FSETableBuildScratch>,
) -> FseTableMode<'a> {
    match selection_policy {
        TableSelectionPolicy::CFast { default_norm_log } => {
            return match scratch {
                Some(scratch) => choose_c_fast_table_with_scratch(
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
                    scratch,
                ),
                None => choose_c_fast_table(
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
                ),
            };
        }
        TableSelectionPolicy::CCost => {
            return match scratch {
                Some(scratch) => choose_c_cost_table_with_scratch(
                    previous,
                    default_table,
                    sequence_count,
                    last_code,
                    counts,
                    max_log,
                    table_builder,
                    scratch,
                ),
                None => choose_c_cost_table(
                    previous,
                    default_table,
                    sequence_count,
                    last_code,
                    counts,
                    max_log,
                    table_builder,
                ),
            };
        }
        TableSelectionPolicy::Legacy => {}
    }

    let all_same_code = counts.most_frequent() == counts.total();
    let first_code = counts.max_symbol() as u8;

    if all_same_code && sequence_count > 2 {
        return FseTableMode::Rle(first_code);
    }
    if sequence_count <= predefined_max_sequences && counts.default_allowed(default_table) {
        return FseTableMode::Predefined(default_table);
    }
    if all_same_code {
        return FseTableMode::Rle(first_code);
    }
    if let Some(previous) = previous {
        if sequence_count < repeat_table_max_sequences && counts.default_allowed(previous) {
            return FseTableMode::RepeatLast(previous);
        }
    }

    FseTableMode::Encoded(build_sequence_table_from_counts(
        counts,
        last_code,
        max_log,
        table_builder,
    ))
}

#[allow(clippy::too_many_arguments)]
fn candidate_table_modes<'a>(
    previous: Option<&'a FSETable>,
    default_table: &'a FSETable,
    sequences: &[Sequence],
    code: impl Fn(&Sequence) -> u8 + Copy,
    config: TableModeCandidateConfig,
) -> Vec<FseTableMode<'a>> {
    let heuristic = choose_table_with_policy(
        previous,
        false,
        default_table,
        sequences,
        code,
        config.max_log,
        config.repeat_table_max_sequences,
        config.predefined_max_sequences,
        config.selection_policy,
        config.table_builder,
    );

    let mut candidates = vec![heuristic];
    let first_code = code(&sequences[0]);
    let all_same_code = sequences
        .iter()
        .skip(1)
        .all(|sequence| code(sequence) == first_code);

    if sequences.len() <= config.predefined_max_sequences
        && sequences
            .iter()
            .all(|sequence| default_table.can_encode_symbol(code(sequence)))
    {
        candidates.push(FseTableMode::Predefined(default_table));
    }

    if let Some(previous) = previous {
        if sequences.len() < config.repeat_table_max_sequences
            && sequences
                .iter()
                .all(|sequence| previous.can_encode_symbol(code(sequence)))
        {
            candidates.push(FseTableMode::RepeatLast(previous));
        }
    }

    if all_same_code {
        if sequences.len() > 2 {
            candidates.push(FseTableMode::Rle(first_code));
        }
    } else {
        let exact_min_log = EXACT_SEQUENCE_TABLE_MIN_LOG.min(config.max_log);
        for candidate_max_log in exact_min_log..=config.max_log {
            candidates.push(FseTableMode::Encoded(build_sequence_table(
                sequences,
                code,
                candidate_max_log,
                config.table_builder,
            )));
        }
    }

    candidates
}

pub(in crate::encoding::blocks::compressed) fn choose_sequence_table_modes<'a>(
    sequences: &[Sequence],
    config: SequenceModeSearchConfig<'a>,
) -> (FseTableMode<'a>, FseTableMode<'a>, FseTableMode<'a>) {
    choose_sequence_table_modes_with_builder(sequences, config, TableBuilder::Full)
}

pub(in crate::encoding::blocks::compressed) fn choose_c_sequence_table_modes_from_prepared<'a>(
    sequences: &[PreparedSequence],
    config: SequenceModeSearchConfig<'a>,
) -> (FseTableMode<'a>, FseTableMode<'a>, FseTableMode<'a>) {
    choose_c_sequence_table_modes(sequences, config, None)
}

pub(in crate::encoding::blocks::compressed) fn choose_c_sequence_table_modes_from_stored<'a>(
    sequences: &[StoredSequence],
    config: SequenceModeSearchConfig<'a>,
) -> (FseTableMode<'a>, FseTableMode<'a>, FseTableMode<'a>) {
    choose_c_sequence_table_modes(sequences, config, None)
}

pub(in crate::encoding::blocks::compressed) fn choose_c_sequence_table_modes_from_stored_with_scratch<
    'a,
>(
    sequences: &[StoredSequence],
    config: SequenceModeSearchConfig<'a>,
    scratch: &mut FSETableBuildScratch,
) -> (FseTableMode<'a>, FseTableMode<'a>, FseTableMode<'a>) {
    choose_c_sequence_table_modes(sequences, config, Some(scratch))
}

fn choose_c_sequence_table_modes<'a, S: CSequenceValues>(
    sequences: &[S],
    config: SequenceModeSearchConfig<'a>,
    mut scratch: Option<&mut FSETableBuildScratch>,
) -> (FseTableMode<'a>, FseTableMode<'a>, FseTableMode<'a>) {
    debug_assert!(!config.exact_sequence_mode_search);
    debug_assert!(!sequences.is_empty());
    let llml_policy =
        sequence_table_selection_policy(config.c_fast_heuristics, config.c_cost_model, 6);
    let offset_policy =
        sequence_table_selection_policy(config.c_fast_heuristics, config.c_cost_model, 5);

    let ll_counts = CodeCounts::from_codes(
        sequences
            .iter()
            .map(|sequence| literal_length_code(sequence.literal_length())),
    );
    let ll_mode = choose_table_with_policy_from_counts(
        config.ll_previous,
        config.ll_repeat_valid,
        config.ll_default,
        sequences.len(),
        literal_length_code(sequences[sequences.len() - 1].literal_length()),
        &ll_counts,
        9,
        config.repeat_table_max_sequences,
        config.llml_predefined_max_sequences,
        llml_policy,
        TableBuilder::Full,
        scratch.as_deref_mut(),
    );

    let ml_counts = CodeCounts::from_codes(
        sequences
            .iter()
            .map(|sequence| match_length_code(sequence.match_length())),
    );
    let ml_mode = choose_table_with_policy_from_counts(
        config.ml_previous,
        config.ml_repeat_valid,
        config.ml_default,
        sequences.len(),
        match_length_code(sequences[sequences.len() - 1].match_length()),
        &ml_counts,
        9,
        config.repeat_table_max_sequences,
        config.llml_predefined_max_sequences,
        llml_policy,
        TableBuilder::Full,
        scratch.as_deref_mut(),
    );

    let of_counts = CodeCounts::from_codes(sequences.iter().map(|sequence| {
        let offset_value = sequence.offset_value();
        debug_assert!(offset_value != 0);
        offset_code(offset_value)
    }));
    let last_offset_value = sequences[sequences.len() - 1].offset_value();
    debug_assert!(last_offset_value != 0);
    let of_mode = choose_table_with_policy_from_counts(
        config.of_previous,
        config.of_repeat_valid,
        config.of_default,
        sequences.len(),
        offset_code(last_offset_value),
        &of_counts,
        config.of_max_log,
        config.repeat_table_max_sequences,
        config.of_predefined_max_sequences,
        offset_policy,
        TableBuilder::Full,
        scratch,
    );
    (ll_mode, ml_mode, of_mode)
}

/// Retained Fast `ZSTD_seqToCodes()` and
/// `ZSTD_buildSequencesStatistics()` boundary. The compact count workspace is
/// deliberately DFast-only: matched hardware counters show that Fast benefits
/// from the three statically bounded 256-symbol tables even though they require
/// more initialization.
#[inline(never)]
pub(in crate::encoding::blocks::compressed) fn choose_c_fast_sequence_table_modes_from_prepared<
    'a,
>(
    sequences: &[PreparedSequence],
    config: SequenceModeSearchConfig<'a>,
    offset_history: OffsetHistory,
    scratch: Option<&mut FSETableBuildScratch>,
) -> (
    FseTableMode<'a>,
    FseTableMode<'a>,
    FseTableMode<'a>,
    OffsetHistory,
) {
    choose_c_fast_sequence_table_modes_from_records::<_, true>(
        sequences,
        config,
        offset_history,
        scratch,
    )
}

pub(in crate::encoding::blocks::compressed) fn choose_c_fast_sequence_table_modes_from_stored<
    'a,
>(
    sequences: &[StoredSequence],
    config: SequenceModeSearchConfig<'a>,
    offset_history: OffsetHistory,
    scratch: Option<&mut FSETableBuildScratch>,
) -> (
    FseTableMode<'a>,
    FseTableMode<'a>,
    FseTableMode<'a>,
    OffsetHistory,
) {
    choose_c_fast_sequence_table_modes_from_records::<_, true>(
        sequences,
        config,
        offset_history,
        scratch,
    )
}

pub(in crate::encoding::blocks::compressed) fn choose_c_fast_sequence_table_modes_from_stored_with_final_history<
    'a,
>(
    sequences: &[StoredSequence],
    config: SequenceModeSearchConfig<'a>,
    final_offset_history: OffsetHistory,
    scratch: Option<&mut FSETableBuildScratch>,
) -> (
    FseTableMode<'a>,
    FseTableMode<'a>,
    FseTableMode<'a>,
    OffsetHistory,
) {
    choose_c_fast_sequence_table_modes_from_records::<_, false>(
        sequences,
        config,
        final_offset_history,
        scratch,
    )
}

#[inline(never)]
fn choose_c_fast_sequence_table_modes_from_records<
    'a,
    S: CSequenceValues,
    const REPLAY_HISTORY: bool,
>(
    sequences: &[S],
    config: SequenceModeSearchConfig<'a>,
    mut offset_history: OffsetHistory,
    mut scratch: Option<&mut FSETableBuildScratch>,
) -> (
    FseTableMode<'a>,
    FseTableMode<'a>,
    FseTableMode<'a>,
    OffsetHistory,
) {
    debug_assert!(!config.exact_sequence_mode_search);
    debug_assert!(config.c_fast_heuristics);
    debug_assert!(!config.c_cost_model);
    debug_assert!(!sequences.is_empty());

    let mut ll_counts = CodeCounts::new();
    let mut ml_counts = CodeCounts::new();
    let mut of_counts = CodeCounts::new();
    for &sequence in sequences {
        let literal_length = sequence.literal_length();
        ll_counts.add_code_untracked(literal_length_code(literal_length));
        ml_counts.add_code_untracked(match_length_code(sequence.match_length()));
        let offset_value = sequence.offset_value();
        debug_assert!(offset_value != 0);
        of_counts.add_code_untracked(offset_code(offset_value));
        if REPLAY_HISTORY {
            let raw_offset =
                offset_history.update_from_c_offset_value(offset_value, literal_length);
            debug_assert!(sequence
                .expected_raw_offset()
                .is_none_or(|expected| expected == raw_offset));
        }
    }
    ll_counts.finish_untracked(sequences.len(), 35);
    ml_counts.finish_untracked(sequences.len(), 52);
    of_counts.finish_untracked(sequences.len(), 31);

    let last = sequences[sequences.len() - 1];
    let last_offset_value = last.offset_value();
    debug_assert!(last_offset_value != 0);
    let ll_mode = match scratch.as_deref_mut() {
        Some(scratch) => choose_c_fast_table_with_scratch(
            config.ll_previous,
            config.ll_repeat_valid,
            config.ll_default,
            sequences.len(),
            literal_length_code(last.literal_length()),
            &ll_counts,
            9,
            config.repeat_table_max_sequences,
            config.llml_predefined_max_sequences,
            6,
            TableBuilder::Full,
            scratch,
        ),
        None => choose_c_fast_table(
            config.ll_previous,
            config.ll_repeat_valid,
            config.ll_default,
            sequences.len(),
            literal_length_code(last.literal_length()),
            &ll_counts,
            9,
            config.repeat_table_max_sequences,
            config.llml_predefined_max_sequences,
            6,
            TableBuilder::Full,
        ),
    };
    let ml_mode = match scratch.as_deref_mut() {
        Some(scratch) => choose_c_fast_table_with_scratch(
            config.ml_previous,
            config.ml_repeat_valid,
            config.ml_default,
            sequences.len(),
            match_length_code(last.match_length()),
            &ml_counts,
            9,
            config.repeat_table_max_sequences,
            config.llml_predefined_max_sequences,
            6,
            TableBuilder::Full,
            scratch,
        ),
        None => choose_c_fast_table(
            config.ml_previous,
            config.ml_repeat_valid,
            config.ml_default,
            sequences.len(),
            match_length_code(last.match_length()),
            &ml_counts,
            9,
            config.repeat_table_max_sequences,
            config.llml_predefined_max_sequences,
            6,
            TableBuilder::Full,
        ),
    };
    let of_mode = match scratch {
        Some(scratch) => choose_c_fast_table_with_scratch(
            config.of_previous,
            config.of_repeat_valid,
            config.of_default,
            sequences.len(),
            offset_code(last_offset_value),
            &of_counts,
            config.of_max_log,
            config.repeat_table_max_sequences,
            config.of_predefined_max_sequences,
            5,
            TableBuilder::Full,
            scratch,
        ),
        None => choose_c_fast_table(
            config.of_previous,
            config.of_repeat_valid,
            config.of_default,
            sequences.len(),
            offset_code(last_offset_value),
            &of_counts,
            config.of_max_log,
            config.repeat_table_max_sequences,
            config.of_predefined_max_sequences,
            5,
            TableBuilder::Full,
        ),
    };
    (ll_mode, ml_mode, of_mode, offset_history)
}

pub(in crate::encoding::blocks::compressed) fn choose_sequence_table_modes_for_estimate<'a>(
    sequences: &[Sequence],
    config: SequenceModeSearchConfig<'a>,
) -> (FseTableMode<'a>, FseTableMode<'a>, FseTableMode<'a>) {
    let table_builder = if config.exact_sequence_mode_search {
        TableBuilder::Full
    } else {
        TableBuilder::ProbabilityOnly
    };
    choose_sequence_table_modes_with_builder(sequences, config, table_builder)
}

#[allow(clippy::too_many_arguments)]
pub(in crate::encoding::blocks::compressed) fn choose_sequence_table_modes_for_estimate_from_counts_with_scratch<
    'a,
>(
    sequence_count: usize,
    ll_counts: &CodeCounts,
    ll_last_code: u8,
    ml_counts: &CodeCounts,
    ml_last_code: u8,
    of_counts: &CodeCounts,
    of_last_code: u8,
    config: SequenceModeSearchConfig<'a>,
    scratch: &mut FSETableBuildScratch,
) -> (FseTableMode<'a>, FseTableMode<'a>, FseTableMode<'a>) {
    debug_assert!(!config.exact_sequence_mode_search);
    let table_builder = TableBuilder::ProbabilityOnly;
    let llml_policy =
        sequence_table_selection_policy(config.c_fast_heuristics, config.c_cost_model, 6);
    let offset_policy =
        sequence_table_selection_policy(config.c_fast_heuristics, config.c_cost_model, 5);

    let ll = choose_table_with_policy_from_counts(
        config.ll_previous,
        config.ll_repeat_valid,
        config.ll_default,
        sequence_count,
        ll_last_code,
        ll_counts,
        9,
        config.repeat_table_max_sequences,
        config.llml_predefined_max_sequences,
        llml_policy,
        table_builder,
        Some(&mut *scratch),
    );
    let ml = choose_table_with_policy_from_counts(
        config.ml_previous,
        config.ml_repeat_valid,
        config.ml_default,
        sequence_count,
        ml_last_code,
        ml_counts,
        9,
        config.repeat_table_max_sequences,
        config.llml_predefined_max_sequences,
        llml_policy,
        table_builder,
        Some(&mut *scratch),
    );
    let of = choose_table_with_policy_from_counts(
        config.of_previous,
        config.of_repeat_valid,
        config.of_default,
        sequence_count,
        of_last_code,
        of_counts,
        config.of_max_log,
        config.repeat_table_max_sequences,
        config.of_predefined_max_sequences,
        offset_policy,
        table_builder,
        Some(scratch),
    );
    (ll, ml, of)
}

fn choose_sequence_table_modes_with_builder<'a>(
    sequences: &[Sequence],
    config: SequenceModeSearchConfig<'a>,
    table_builder: TableBuilder,
) -> (FseTableMode<'a>, FseTableMode<'a>, FseTableMode<'a>) {
    let llml_policy =
        sequence_table_selection_policy(config.c_fast_heuristics, config.c_cost_model, 6);
    let offset_policy =
        sequence_table_selection_policy(config.c_fast_heuristics, config.c_cost_model, 5);

    if config.exact_sequence_mode_search {
        return choose_exact_sequence_table_modes(
            sequences,
            config,
            table_builder,
            llml_policy,
            offset_policy,
        );
    }

    match (llml_policy, offset_policy) {
        (
            TableSelectionPolicy::CFast {
                default_norm_log: 6,
            },
            TableSelectionPolicy::CFast {
                default_norm_log: 5,
            },
        ) => choose_c_fast_sequence_table_modes(sequences, config, table_builder),
        (TableSelectionPolicy::CCost, TableSelectionPolicy::CCost) => {
            choose_c_cost_sequence_table_modes(sequences, config, table_builder)
        }
        (TableSelectionPolicy::Legacy, TableSelectionPolicy::Legacy) => {
            choose_legacy_sequence_table_modes(sequences, config, table_builder)
        }
        _ => unreachable!("LL/ML and offset selection policies must agree"),
    }
}

#[inline(never)]
fn choose_c_fast_sequence_table_modes<'a>(
    sequences: &[Sequence],
    config: SequenceModeSearchConfig<'a>,
    table_builder: TableBuilder,
) -> (FseTableMode<'a>, FseTableMode<'a>, FseTableMode<'a>) {
    (
        choose_c_fast_table_from_sequences(
            config.ll_previous,
            config.ll_repeat_valid,
            config.ll_default,
            sequences,
            |seq| literal_length_code(seq.ll),
            9,
            config.repeat_table_max_sequences,
            config.llml_predefined_max_sequences,
            6,
            table_builder,
        ),
        choose_c_fast_table_from_sequences(
            config.ml_previous,
            config.ml_repeat_valid,
            config.ml_default,
            sequences,
            |seq| match_length_code(seq.ml),
            9,
            config.repeat_table_max_sequences,
            config.llml_predefined_max_sequences,
            6,
            table_builder,
        ),
        choose_c_fast_table_from_sequences(
            config.of_previous,
            config.of_repeat_valid,
            config.of_default,
            sequences,
            |seq| offset_code(seq.of),
            config.of_max_log,
            config.repeat_table_max_sequences,
            config.of_predefined_max_sequences,
            5,
            table_builder,
        ),
    )
}

#[inline(never)]
fn choose_c_cost_sequence_table_modes<'a>(
    sequences: &[Sequence],
    config: SequenceModeSearchConfig<'a>,
    table_builder: TableBuilder,
) -> (FseTableMode<'a>, FseTableMode<'a>, FseTableMode<'a>) {
    (
        choose_c_cost_table_from_sequences(
            config.ll_previous,
            config.ll_default,
            sequences,
            |seq| literal_length_code(seq.ll),
            9,
            table_builder,
        ),
        choose_c_cost_table_from_sequences(
            config.ml_previous,
            config.ml_default,
            sequences,
            |seq| match_length_code(seq.ml),
            9,
            table_builder,
        ),
        choose_c_cost_table_from_sequences(
            config.of_previous,
            config.of_default,
            sequences,
            |seq| offset_code(seq.of),
            config.of_max_log,
            table_builder,
        ),
    )
}

#[inline(never)]
fn choose_legacy_sequence_table_modes<'a>(
    sequences: &[Sequence],
    config: SequenceModeSearchConfig<'a>,
    table_builder: TableBuilder,
) -> (FseTableMode<'a>, FseTableMode<'a>, FseTableMode<'a>) {
    (
        choose_legacy_table(
            config.ll_previous,
            config.ll_default,
            sequences,
            |seq| literal_length_code(seq.ll),
            9,
            config.repeat_table_max_sequences,
            config.llml_predefined_max_sequences,
            table_builder,
        ),
        choose_legacy_table(
            config.ml_previous,
            config.ml_default,
            sequences,
            |seq| match_length_code(seq.ml),
            9,
            config.repeat_table_max_sequences,
            config.llml_predefined_max_sequences,
            table_builder,
        ),
        choose_legacy_table(
            config.of_previous,
            config.of_default,
            sequences,
            |seq| offset_code(seq.of),
            config.of_max_log,
            config.repeat_table_max_sequences,
            config.of_predefined_max_sequences,
            table_builder,
        ),
    )
}

#[inline(never)]
fn choose_exact_sequence_table_modes<'a>(
    sequences: &[Sequence],
    config: SequenceModeSearchConfig<'a>,
    table_builder: TableBuilder,
    llml_policy: TableSelectionPolicy,
    offset_policy: TableSelectionPolicy,
) -> (FseTableMode<'a>, FseTableMode<'a>, FseTableMode<'a>) {
    debug_assert!(config.exact_sequence_mode_search);
    let ll_candidates = candidate_table_modes(
        config.ll_previous,
        config.ll_default,
        sequences,
        |seq| literal_length_code(seq.ll),
        TableModeCandidateConfig {
            max_log: 9,
            repeat_table_max_sequences: config.repeat_table_max_sequences,
            predefined_max_sequences: config.llml_predefined_max_sequences,
            selection_policy: llml_policy,
            table_builder,
        },
    );
    let ml_candidates = candidate_table_modes(
        config.ml_previous,
        config.ml_default,
        sequences,
        |seq| match_length_code(seq.ml),
        TableModeCandidateConfig {
            max_log: 9,
            repeat_table_max_sequences: config.repeat_table_max_sequences,
            predefined_max_sequences: config.llml_predefined_max_sequences,
            selection_policy: llml_policy,
            table_builder,
        },
    );
    let of_candidates = candidate_table_modes(
        config.of_previous,
        config.of_default,
        sequences,
        |seq| offset_code(seq.of),
        TableModeCandidateConfig {
            max_log: config.of_max_log,
            repeat_table_max_sequences: config.repeat_table_max_sequences,
            predefined_max_sequences: config.of_predefined_max_sequences,
            selection_policy: offset_policy,
            table_builder,
        },
    );

    let mut ll_candidates = ll_candidates;
    let mut ml_candidates = ml_candidates;
    let mut of_candidates = of_candidates;
    let mut best_ll = 0usize;
    let mut best_ml = 0usize;
    let mut best_of = 0usize;
    let mut best_size = super::exact_sequence_section_size(
        sequences,
        &ll_candidates[0],
        &ml_candidates[0],
        &of_candidates[0],
    );

    for (ll_idx, ll_mode) in ll_candidates.iter().enumerate() {
        for (ml_idx, ml_mode) in ml_candidates.iter().enumerate() {
            for (of_idx, of_mode) in of_candidates.iter().enumerate() {
                let size = super::exact_sequence_section_size(sequences, ll_mode, ml_mode, of_mode);
                if size < best_size {
                    best_ll = ll_idx;
                    best_ml = ml_idx;
                    best_of = of_idx;
                    best_size = size;
                }
            }
        }
    }

    (
        ll_candidates.swap_remove(best_ll),
        ml_candidates.swap_remove(best_ml),
        of_candidates.swap_remove(best_of),
    )
}
