use alloc::vec::Vec;

use crate::{
    bit_io::BitWriter,
    blocks::sequence_section::Sequence,
    encoding::{
        blocks::PreparedSequence,
        frame_compressor::{FseTables, OffsetHistory},
    },
    fse::fse_encoder::{build_rle_table, FSETable, SharedFSETable},
};

use super::{
    config::BlockCompressionConfig,
    literal_length_code, match_length_code, offset_code,
    sequence_bitstream::encode_sequences_for_history_into,
    sequence_codes::{encode_literal_length, encode_match_len, encode_offset},
    sequence_cost::{cross_entropy_cost, repeat_table_cost, CodeCounts},
    sequence_sections::{CompressedSequenceTables, SequenceTableMode, SequenceTableModes},
    sequence_tables::{choose_sequence_table_modes, FseTableMode, SequenceModeSearchConfig},
};

pub(crate) fn select_c_sequence_table_modes(
    sequences: &[PreparedSequence],
    fse_tables: &FseTables,
    offset_history: OffsetHistory,
    strategy: u8,
) -> SequenceTableModes {
    let mut offset_history = offset_history;
    let mut encoded_sequences = Vec::with_capacity(sequences.len());
    encode_sequences_for_history_into(sequences, &mut offset_history, &mut encoded_sequences);
    let config = BlockCompressionConfig::for_c_strategy(strategy);
    let llml_predefined_max_sequences = config
        .file_type_small_sequence_predefined_llml_max_sequences
        .unwrap_or(16);
    let (ll, ml, of) = choose_sequence_table_modes(
        &encoded_sequences,
        SequenceModeSearchConfig {
            ll_previous: fse_tables.ll_previous.as_deref(),
            ll_repeat_valid: fse_tables.ll_repeat_valid,
            ll_default: &fse_tables.ll_default,
            ml_previous: fse_tables.ml_previous.as_deref(),
            ml_repeat_valid: fse_tables.ml_repeat_valid,
            ml_default: &fse_tables.ml_default,
            of_previous: fse_tables.of_previous.as_deref(),
            of_repeat_valid: fse_tables.of_repeat_valid,
            of_default: &fse_tables.of_default,
            repeat_table_max_sequences: config.repeat_table_max_sequences,
            llml_predefined_max_sequences,
            of_predefined_max_sequences: config.offset_predefined_max_sequences,
            of_max_log: config.offset_table_max_log,
            exact_sequence_mode_search: false,
            c_fast_heuristics: config.c_fast_sequence_table_heuristics,
            c_cost_model: config.c_cost_sequence_table_selection,
        },
    );

    SequenceTableModes {
        ll: sequence_table_mode_from_fse_mode(&ll),
        ml: sequence_table_mode_from_fse_mode(&ml),
        of: sequence_table_mode_from_fse_mode(&of),
    }
}

pub(crate) fn prime_sequence_tables_for_repeat(
    sequences: &[PreparedSequence],
    modes: SequenceTableModes,
    compressed_tables: Option<&CompressedSequenceTables>,
    fse_tables: &mut FseTables,
    offset_history: OffsetHistory,
) -> Option<()> {
    if sequences.is_empty() {
        return Some(());
    }

    let mut offset_history = offset_history;
    let mut encoded_sequences = Vec::with_capacity(sequences.len());
    encode_sequences_for_history_into(sequences, &mut offset_history, &mut encoded_sequences);
    let ll_repeat_valid = repeat_valid_after_mode(modes.ll, fse_tables.ll_repeat_valid);
    let ml_repeat_valid = repeat_valid_after_mode(modes.ml, fse_tables.ml_repeat_valid);
    let of_repeat_valid = repeat_valid_after_mode(modes.of, fse_tables.of_repeat_valid);

    fse_tables.ll_previous = Some(sequence_repeat_table(
        modes.ll,
        &encoded_sequences,
        compressed_tables.map(|tables| &tables.ll),
        &fse_tables.ll_default,
        fse_tables.ll_previous.clone(),
        |sequence| literal_length_code(sequence.ll),
    )?);
    fse_tables.ml_previous = Some(sequence_repeat_table(
        modes.ml,
        &encoded_sequences,
        compressed_tables.map(|tables| &tables.ml),
        &fse_tables.ml_default,
        fse_tables.ml_previous.clone(),
        |sequence| match_length_code(sequence.ml),
    )?);
    fse_tables.of_previous = Some(sequence_repeat_table(
        modes.of,
        &encoded_sequences,
        compressed_tables.map(|tables| &tables.of),
        &fse_tables.of_default,
        fse_tables.of_previous.clone(),
        |sequence| offset_code(sequence.of),
    )?);
    fse_tables.ll_repeat_valid = ll_repeat_valid;
    fse_tables.ml_repeat_valid = ml_repeat_valid;
    fse_tables.of_repeat_valid = of_repeat_valid;
    Some(())
}

pub(crate) fn finish_sequence_tables_after_superblock(
    modes: SequenceTableModes,
    compressed_tables: Option<&CompressedSequenceTables>,
    previous_fse_tables: &FseTables,
    fse_tables: &mut FseTables,
) -> Option<()> {
    fse_tables.ll_previous = external_sequence_table(
        modes.ll,
        compressed_tables.map(|tables| &tables.ll),
        previous_fse_tables.ll_previous.clone(),
    )?;
    fse_tables.ml_previous = external_sequence_table(
        modes.ml,
        compressed_tables.map(|tables| &tables.ml),
        previous_fse_tables.ml_previous.clone(),
    )?;
    fse_tables.of_previous = external_sequence_table(
        modes.of,
        compressed_tables.map(|tables| &tables.of),
        previous_fse_tables.of_previous.clone(),
    )?;
    fse_tables.ll_repeat_valid =
        repeat_valid_after_mode(modes.ll, previous_fse_tables.ll_repeat_valid);
    fse_tables.ml_repeat_valid =
        repeat_valid_after_mode(modes.ml, previous_fse_tables.ml_repeat_valid);
    fse_tables.of_repeat_valid =
        repeat_valid_after_mode(modes.of, previous_fse_tables.of_repeat_valid);
    Some(())
}

fn repeat_valid_after_mode(mode: SequenceTableMode, previous_valid: bool) -> bool {
    matches!(mode, SequenceTableMode::Repeat) && previous_valid
}

pub(crate) fn estimate_superblock_sequence_section_size(
    sequences: &[PreparedSequence],
    modes: SequenceTableModes,
    compressed_tables: Option<&CompressedSequenceTables>,
    fse_tables: &FseTables,
    offset_history: OffsetHistory,
    write_entropy: bool,
) -> Option<usize> {
    const SEQUENCES_SECTION_HEADER_SIZE: usize = 3;

    if sequences.is_empty() {
        return Some(SEQUENCES_SECTION_HEADER_SIZE);
    }

    let mut offset_history = offset_history;
    let mut encoded_sequences = Vec::with_capacity(sequences.len());
    encode_sequences_for_history_into(sequences, &mut offset_history, &mut encoded_sequences);
    let (of_estimate, ll_estimate, ml_estimate) = sequence_symbol_estimates(&encoded_sequences);

    let mut estimate = SEQUENCES_SECTION_HEADER_SIZE
        + estimate_symbol_type_size(
            modes.of,
            &of_estimate,
            compressed_tables.map(|tables| &tables.of),
            &fse_tables.of_default,
            fse_tables.of_previous.as_deref(),
        )?
        + estimate_symbol_type_size(
            modes.ll,
            &ll_estimate,
            compressed_tables.map(|tables| &tables.ll),
            &fse_tables.ll_default,
            fse_tables.ll_previous.as_deref(),
        )?
        + estimate_symbol_type_size(
            modes.ml,
            &ml_estimate,
            compressed_tables.map(|tables| &tables.ml),
            &fse_tables.ml_default,
            fse_tables.ml_previous.as_deref(),
        )?;

    if write_entropy {
        estimate +=
            sequence_table_definition_size(modes.ll, compressed_tables.map(|tables| &tables.ll))?;
        estimate +=
            sequence_table_definition_size(modes.of, compressed_tables.map(|tables| &tables.of))?;
        estimate +=
            sequence_table_definition_size(modes.ml, compressed_tables.map(|tables| &tables.ml))?;
    }

    Some(estimate)
}

fn sequence_symbol_estimates(
    sequences: &[Sequence],
) -> (SymbolEstimate, SymbolEstimate, SymbolEstimate) {
    let mut of_estimate = SymbolEstimate::new();
    let mut ll_estimate = SymbolEstimate::new();
    let mut ml_estimate = SymbolEstimate::new();

    for sequence in sequences {
        let (of_code, _, of_bits) = encode_offset(sequence.of);
        of_estimate.counts.add_code(of_code);
        of_estimate.extra_bits += of_bits;

        let (ll_code, _, ll_bits) = encode_literal_length(sequence.ll);
        ll_estimate.counts.add_code(ll_code);
        ll_estimate.extra_bits += ll_bits;

        let (ml_code, _, ml_bits) = encode_match_len(sequence.ml);
        ml_estimate.counts.add_code(ml_code);
        ml_estimate.extra_bits += ml_bits;
    }

    (of_estimate, ll_estimate, ml_estimate)
}

struct SymbolEstimate {
    counts: CodeCounts,
    extra_bits: usize,
}

impl SymbolEstimate {
    fn new() -> Self {
        Self {
            counts: CodeCounts::new(),
            extra_bits: 0,
        }
    }
}

fn estimate_symbol_type_size(
    mode: SequenceTableMode,
    estimate: &SymbolEstimate,
    compressed_table: Option<&FSETable>,
    default_table: &FSETable,
    previous_table: Option<&FSETable>,
) -> Option<usize> {
    let symbol_bits = match mode {
        SequenceTableMode::Predefined => cross_entropy_cost(default_table, &estimate.counts),
        SequenceTableMode::Rle => Some(0),
        SequenceTableMode::Compressed => repeat_table_cost(compressed_table?, &estimate.counts),
        SequenceTableMode::Repeat => repeat_table_cost(previous_table?, &estimate.counts),
    }
    .unwrap_or(estimate.counts.total() * 10);
    Some((symbol_bits + estimate.extra_bits) / 8)
}

fn sequence_table_definition_size(
    mode: SequenceTableMode,
    compressed_table: Option<&FSETable>,
) -> Option<usize> {
    match mode {
        SequenceTableMode::Predefined | SequenceTableMode::Repeat => Some(0),
        SequenceTableMode::Rle => Some(1),
        SequenceTableMode::Compressed => Some(fse_table_definition_size(compressed_table?)),
    }
}

fn fse_table_definition_size(table: &FSETable) -> usize {
    let mut bytes = Vec::new();
    let mut writer = BitWriter::from(&mut bytes);
    table.write_table(&mut writer);
    writer.flush();
    bytes.len()
}

fn sequence_repeat_table(
    mode: SequenceTableMode,
    sequences: &[Sequence],
    compressed_table: Option<&FSETable>,
    default_table: &FSETable,
    previous_table: Option<SharedFSETable>,
    code: impl Fn(Sequence) -> u8,
) -> Option<SharedFSETable> {
    match mode {
        SequenceTableMode::Predefined => Some(SharedFSETable::new(default_table.clone())),
        SequenceTableMode::Rle => uniform_code(sequences, code)
            .map(build_rle_table)
            .map(SharedFSETable::new),
        SequenceTableMode::Compressed => compressed_table.cloned().map(SharedFSETable::new),
        SequenceTableMode::Repeat => previous_table,
    }
}

fn external_sequence_table(
    mode: SequenceTableMode,
    compressed_table: Option<&FSETable>,
    previous_table: Option<SharedFSETable>,
) -> Option<Option<SharedFSETable>> {
    match mode {
        SequenceTableMode::Predefined | SequenceTableMode::Rle => Some(None),
        SequenceTableMode::Compressed => {
            compressed_table.cloned().map(SharedFSETable::new).map(Some)
        }
        SequenceTableMode::Repeat => Some(previous_table),
    }
}

fn uniform_code(sequences: &[Sequence], code: impl Fn(Sequence) -> u8) -> Option<u8> {
    let first = code(*sequences.first()?);
    sequences
        .iter()
        .all(|sequence| code(*sequence) == first)
        .then_some(first)
}

fn sequence_table_mode_from_fse_mode(mode: &FseTableMode<'_>) -> SequenceTableMode {
    match mode {
        FseTableMode::Predefined(_) => SequenceTableMode::Predefined,
        FseTableMode::Rle(_) => SequenceTableMode::Rle,
        FseTableMode::Encoded(_) => SequenceTableMode::Compressed,
        FseTableMode::RepeatLast(_) => SequenceTableMode::Repeat,
    }
}
