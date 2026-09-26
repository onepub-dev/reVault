use super::super::super::c_sequence::CSequenceValues;
use super::super::{
    builder::{
        build_c_fast_sequence_table_from_u32_counts,
        build_c_fast_sequence_table_from_u32_counts_with_scratch, TableBuilder,
    },
    FseTableMode, SequenceModeSearchConfig,
};
use crate::encoding::levels::c_port::sequence_store::StoredSequence;
use crate::{
    encoding::{
        blocks::compressed::{
            literal_length_code, match_length_code, offset_code, PreparedSequence,
        },
        frame_compressor::OffsetHistory,
    },
    fse::fse_encoder::{FSETable, FSETableBuildScratch},
};
use core::convert::TryFrom;

const CODE_LANE_WIDTH: usize = 64;
const LL_LANE: u8 = 0;
const ML_LANE: u8 = 64;
const OF_LANE: u8 = 128;

#[derive(Clone, Copy, Default)]
struct U32CountStats {
    total: u32,
    most_frequent: u32,
    max_symbol: u32,
}

impl U32CountStats {
    #[inline(always)]
    fn add_count(workspace: &mut [u32; 256], lane: u8, code: u8) {
        debug_assert!(usize::from(code) < CODE_LANE_WIDTH);
        workspace[usize::from(code | lane)] += 1;
    }

    fn from_counts(counts: &[u32], total: u32) -> Self {
        let mut most_frequent = 0;
        let mut max_symbol = 0;
        for (symbol, count) in counts.iter().copied().enumerate() {
            most_frequent = most_frequent.max(count);
            if count != 0 {
                max_symbol = symbol as u32;
            }
        }
        Self {
            total,
            most_frequent,
            max_symbol,
        }
    }
}

/// C's Fast/DFast `ZSTD_seqToCodes()` and
/// `ZSTD_buildSequencesStatistics()` transaction, adapted to direct prepared
/// records.
///
/// C reuses one bounded count workspace for its three small alphabets. Rust
/// keeps the retained single record walk, but places LL, ML, and OF counts in
/// three non-overlapping 64-symbol lanes of one safe 256-entry array. The lane
/// index is always a `u8`, so every access is statically in bounds without
/// unchecked indexing. Table construction follows C's LL, OF, ML order and
/// consumes the already-known total and maximum symbol directly.
#[inline(never)]
pub(in crate::encoding::blocks::compressed) fn choose_c_dfast_compact_sequence_table_modes_from_prepared<
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
    choose_c_dfast_u32_sequence_table_modes::<_, true>(sequences, config, offset_history, scratch)
}

pub(in crate::encoding::blocks::compressed) fn choose_c_dfast_compact_sequence_table_modes_from_stored<
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
    choose_c_dfast_u32_sequence_table_modes::<_, true>(sequences, config, offset_history, scratch)
}

pub(in crate::encoding::blocks::compressed) fn choose_c_dfast_compact_sequence_table_modes_from_stored_with_final_history<
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
    choose_c_dfast_u32_sequence_table_modes::<_, false>(
        sequences,
        config,
        final_offset_history,
        scratch,
    )
}

/// C's complete unsigned-count path from `ZSTD_seqToCodes()` through the
/// three `ZSTD_buildCTable()` calls. The active DFast function never widens
/// its compact count workspace or normalization arithmetic.
#[inline(never)]
fn choose_c_dfast_u32_sequence_table_modes<'a, S: CSequenceValues, const REPLAY_HISTORY: bool>(
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

    let mut workspace = [0u32; 256];
    for &sequence in sequences {
        let literal_length = sequence.literal_length();
        U32CountStats::add_count(&mut workspace, LL_LANE, literal_length_code(literal_length));
        U32CountStats::add_count(
            &mut workspace,
            ML_LANE,
            match_length_code(sequence.match_length()),
        );
        let offset_value = sequence.offset_value();
        debug_assert!(offset_value != 0);
        U32CountStats::add_count(&mut workspace, OF_LANE, offset_code(offset_value));
        if REPLAY_HISTORY {
            let raw_offset =
                offset_history.update_from_c_offset_value(offset_value, literal_length);
            debug_assert!(sequence
                .expected_raw_offset()
                .is_none_or(|expected| expected == raw_offset));
        }
    }
    let total = u32::try_from(sequences.len())
        .expect("a Zstandard block cannot contain more than u32::MAX sequences");
    let ll_stats = U32CountStats::from_counts(&workspace[..CODE_LANE_WIDTH], total);
    let ml_stats =
        U32CountStats::from_counts(&workspace[CODE_LANE_WIDTH..2 * CODE_LANE_WIDTH], total);
    let of_stats =
        U32CountStats::from_counts(&workspace[2 * CODE_LANE_WIDTH..3 * CODE_LANE_WIDTH], total);

    let last = sequences[sequences.len() - 1];
    let last_offset_value = last.offset_value();
    debug_assert!(last_offset_value != 0);
    let ll_mode = choose_c_fast_table_from_u32_lane(
        config.ll_previous,
        config.ll_repeat_valid,
        config.ll_default,
        &mut workspace[..CODE_LANE_WIDTH],
        ll_stats,
        literal_length_code(last.literal_length()),
        9,
        config.repeat_table_max_sequences,
        config.llml_predefined_max_sequences,
        6,
        scratch.as_deref_mut(),
    );
    let of_mode = choose_c_fast_table_from_u32_lane(
        config.of_previous,
        config.of_repeat_valid,
        config.of_default,
        &mut workspace[2 * CODE_LANE_WIDTH..3 * CODE_LANE_WIDTH],
        of_stats,
        offset_code(last_offset_value),
        config.of_max_log,
        config.repeat_table_max_sequences,
        config.of_predefined_max_sequences,
        5,
        scratch.as_deref_mut(),
    );
    let ml_mode = choose_c_fast_table_from_u32_lane(
        config.ml_previous,
        config.ml_repeat_valid,
        config.ml_default,
        &mut workspace[CODE_LANE_WIDTH..2 * CODE_LANE_WIDTH],
        ml_stats,
        match_length_code(last.match_length()),
        9,
        config.repeat_table_max_sequences,
        config.llml_predefined_max_sequences,
        6,
        scratch,
    );

    (ll_mode, ml_mode, of_mode, offset_history)
}

#[allow(clippy::too_many_arguments)]
fn choose_c_fast_table_from_u32_lane<'a>(
    previous: Option<&'a FSETable>,
    previous_repeat_valid: bool,
    default_table: &'a FSETable,
    counts: &mut [u32],
    stats: U32CountStats,
    last_code: u8,
    max_log: u8,
    repeat_table_max_sequences: usize,
    predefined_max_sequences: usize,
    default_norm_log: u8,
    scratch: Option<&mut FSETableBuildScratch>,
) -> FseTableMode<'a> {
    debug_assert_eq!(stats.total, counts.iter().sum::<u32>());
    debug_assert!((stats.max_symbol as usize) < counts.len());

    if stats.most_frequent == stats.total {
        let first_code = stats.max_symbol as u8;
        let default_allowed = default_table.can_encode_symbol(first_code);
        return if default_allowed && stats.total <= 2 {
            FseTableMode::Predefined(default_table)
        } else {
            FseTableMode::Rle(first_code)
        };
    }

    let max_symbol = stats.max_symbol as usize;
    let default_allowed = counts[..=max_symbol]
        .iter()
        .enumerate()
        .all(|(symbol, count)| *count == 0 || default_table.can_encode_symbol(symbol as u8));

    if default_allowed {
        if let Some(previous) = previous {
            let repeat_allowed = previous_repeat_valid
                && stats.total < repeat_table_max_sequences as u32
                && counts[..=max_symbol]
                    .iter()
                    .enumerate()
                    .all(|(symbol, count)| *count == 0 || previous.can_encode_symbol(symbol as u8));
            if repeat_allowed {
                return FseTableMode::RepeatLast(previous);
            }
        }

        if stats.total < predefined_max_sequences as u32
            || stats.most_frequent < (stats.total >> (u32::from(default_norm_log) - 1))
        {
            return FseTableMode::Predefined(default_table);
        }
    }

    FseTableMode::Encoded(match scratch {
        Some(scratch) => build_c_fast_sequence_table_from_u32_counts_with_scratch(
            counts,
            stats.total,
            stats.max_symbol,
            u32::from(last_code),
            max_log,
            TableBuilder::Full,
            scratch,
        ),
        None => build_c_fast_sequence_table_from_u32_counts(
            counts,
            stats.total,
            stats.max_symbol,
            u32::from(last_code),
            max_log,
            TableBuilder::Full,
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bit_io::BitWriter,
        encoding::blocks::compressed::{encode_fse_table_modes, encode_table},
        encoding::levels::c_port::sequence_store::OffBase,
        fse::fse_encoder::{default_ll_table, default_ml_table, default_of_table},
    };
    use alloc::{vec, vec::Vec};

    #[test]
    fn compact_count_lanes_do_not_overlap() {
        let mut c_workspace = [0u32; 256];
        U32CountStats::add_count(&mut c_workspace, LL_LANE, 35);
        U32CountStats::add_count(&mut c_workspace, ML_LANE, 52);
        U32CountStats::add_count(&mut c_workspace, OF_LANE, 31);

        assert_eq!(c_workspace[35], 1);
        assert_eq!(c_workspace[64 + 52], 1);
        assert_eq!(c_workspace[128 + 31], 1);
        assert_eq!(c_workspace.iter().sum::<u32>(), 3);

        let ll = U32CountStats::from_counts(&c_workspace[..CODE_LANE_WIDTH], 1);
        let ml = U32CountStats::from_counts(&c_workspace[CODE_LANE_WIDTH..2 * CODE_LANE_WIDTH], 1);
        let of =
            U32CountStats::from_counts(&c_workspace[2 * CODE_LANE_WIDTH..3 * CODE_LANE_WIDTH], 1);
        assert_eq!((ll.total, ll.most_frequent, ll.max_symbol), (1, 1, 35));
        assert_eq!((ml.total, ml.most_frequent, ml.max_symbol), (1, 1, 52));
        assert_eq!((of.total, of.most_frequent, of.max_symbol), (1, 1, 31));
    }

    #[test]
    fn c_unsigned_dfast_transaction_matches_machine_word_control() {
        fn encode_modes(modes: &(FseTableMode<'_>, FseTableMode<'_>, FseTableMode<'_>)) -> Vec<u8> {
            let mut output = Vec::new();
            let mut writer = BitWriter::from(&mut output);
            writer.write_bits(encode_fse_table_modes(&modes.0, &modes.1, &modes.2), 8);
            encode_table(&modes.0, &mut writer);
            encode_table(&modes.2, &mut writer);
            encode_table(&modes.1, &mut writer);
            writer.flush();
            output
        }

        let ll_default = default_ll_table();
        let ml_default = default_ml_table();
        let of_default = default_of_table();
        let config = || SequenceModeSearchConfig {
            ll_previous: None,
            ll_repeat_valid: false,
            ll_default: &ll_default,
            ml_previous: None,
            ml_repeat_valid: false,
            ml_default: &ml_default,
            of_previous: None,
            of_repeat_valid: false,
            of_default: &of_default,
            repeat_table_max_sequences: 1000,
            llml_predefined_max_sequences: 56,
            of_predefined_max_sequences: 28,
            of_max_log: 8,
            exact_sequence_mode_search: false,
            c_fast_heuristics: true,
            c_cost_model: false,
        };

        let cases = [
            vec![StoredSequence::new(0, OffBase::Offset(1), 3); 2],
            (0..48)
                .map(|index| {
                    StoredSequence::new(index % 17, OffBase::Offset(index % 11 + 1), 3 + index % 29)
                })
                .collect(),
            (0..4096)
                .map(|index| {
                    StoredSequence::new(
                        (index * 37) % 131_072,
                        OffBase::Offset((index * 8191) % 1_048_573 + 1),
                        3 + (index * 53) % 131_072,
                    )
                })
                .collect(),
        ];

        for sequences in cases {
            let control =
                super::super::choose_c_sequence_table_modes_from_stored(&sequences, config());
            let candidate = choose_c_dfast_u32_sequence_table_modes::<_, true>(
                &sequences,
                config(),
                OffsetHistory::new(),
                None,
            );
            let mut expected_history = OffsetHistory::new();
            for sequence in &sequences {
                expected_history
                    .update_from_c_offset_value(sequence.off_base_value(), sequence.lit_len);
            }
            assert_eq!(candidate.3, expected_history);
            assert_eq!(
                encode_modes(&(candidate.0, candidate.1, candidate.2)),
                encode_modes(&control)
            );
        }
    }
}
