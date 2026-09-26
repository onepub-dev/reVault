//! Sequence-section helpers for target-compressed-size superblocks.

use alloc::vec::Vec;

use super::{
    params::Strategy,
    superblock::{EntropyTableMode, SequenceEntropyModes, SubBlockSequenceEmission},
};
use crate::encoding::{
    blocks::{
        append_compressed_sequence_section, append_predefined_sequence_section,
        append_repeat_sequence_section, append_rle_sequence_section,
        append_sequence_section_with_table_modes, build_compressed_sequence_tables,
        estimate_superblock_sequence_section_size, finish_sequence_tables_after_superblock,
        prime_sequence_tables_for_repeat, select_c_sequence_table_modes, CompressedSequenceTables,
        PreparedSequence, SequenceTableMode, SequenceTableModes,
    },
    frame_compressor::{FseTables, OffsetHistory},
};

pub(super) fn append_sub_block_sequences(
    sequences: &[PreparedSequence],
    _modes: SequenceEntropyModes,
    _write_entropy: bool,
    output: &mut Vec<u8>,
) -> Option<SubBlockSequenceEmission> {
    if !sequences.is_empty() {
        return None;
    }

    output.push(0);
    Some(SubBlockSequenceEmission {
        byte_size: 1,
        entropy_written: false,
    })
}

pub(super) fn append_supported_sub_block_sequences(
    sequences: &[PreparedSequence],
    modes: SequenceEntropyModes,
    write_entropy: bool,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    output: &mut Vec<u8>,
) -> Option<SubBlockSequenceEmission> {
    let compressed_tables = write_entropy
        .then(|| build_compressed_sequence_tables_for_modes(sequences, modes, *offset_history))
        .flatten();
    append_supported_sub_block_sequences_with_tables(
        sequences,
        modes,
        compressed_tables.as_ref(),
        write_entropy,
        fse_tables,
        offset_history,
        output,
    )
}

pub(super) fn append_supported_sub_block_sequences_with_tables(
    sequences: &[PreparedSequence],
    modes: SequenceEntropyModes,
    compressed_tables: Option<&CompressedSequenceTables>,
    write_entropy: bool,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    output: &mut Vec<u8>,
) -> Option<SubBlockSequenceEmission> {
    if sequences.is_empty() {
        return append_sub_block_sequences(sequences, modes, write_entropy, output);
    }
    if !write_entropy {
        return append_repeat_sequence_section(sequences, fse_tables, offset_history, output).map(
            |byte_size| SubBlockSequenceEmission {
                byte_size,
                entropy_written: true,
            },
        );
    }

    let byte_size = if sequence_modes_are(modes, EntropyTableMode::Basic) {
        append_predefined_sequence_section(sequences, fse_tables, offset_history, output)?
    } else if sequence_modes_are(modes, EntropyTableMode::Rle) {
        append_rle_sequence_section(sequences, offset_history, output)?
    } else if sequence_modes_are(modes, EntropyTableMode::Repeat) {
        append_repeat_sequence_section(sequences, fse_tables, offset_history, output)?
    } else if sequence_modes_are(modes, EntropyTableMode::Compressed) {
        if let Some(compressed_tables) = compressed_tables {
            append_sequence_section_with_table_modes(
                sequences,
                sequence_table_modes(modes),
                Some(compressed_tables),
                fse_tables,
                offset_history,
                output,
            )?
        } else {
            append_compressed_sequence_section(sequences, fse_tables, offset_history, output)?
        }
    } else {
        if sequence_modes_need_compressed_tables(modes) && compressed_tables.is_none() {
            return None;
        }
        append_sequence_section_with_table_modes(
            sequences,
            sequence_table_modes(modes),
            compressed_tables,
            fse_tables,
            offset_history,
            output,
        )?
    };
    Some(SubBlockSequenceEmission {
        byte_size,
        entropy_written: true,
    })
}

pub(super) fn build_compressed_sequence_tables_for_modes(
    sequences: &[PreparedSequence],
    modes: SequenceEntropyModes,
    offset_history: OffsetHistory,
) -> Option<CompressedSequenceTables> {
    if !sequence_modes_need_compressed_tables(modes) {
        return None;
    }
    build_compressed_sequence_tables(sequences, offset_history)
}

pub(super) fn prime_sequence_entropy_tables_for_repeat(
    sequences: &[PreparedSequence],
    modes: SequenceEntropyModes,
    compressed_tables: Option<&CompressedSequenceTables>,
    fse_tables: &mut FseTables,
    offset_history: OffsetHistory,
) -> Option<()> {
    prime_sequence_tables_for_repeat(
        sequences,
        sequence_table_modes(modes),
        compressed_tables,
        fse_tables,
        offset_history,
    )
}

pub(super) fn finish_sequence_entropy_tables_after_superblock(
    modes: SequenceEntropyModes,
    compressed_tables: Option<&CompressedSequenceTables>,
    previous_fse_tables: &FseTables,
    fse_tables: &mut FseTables,
) -> Option<()> {
    finish_sequence_tables_after_superblock(
        sequence_table_modes(modes),
        compressed_tables,
        previous_fse_tables,
        fse_tables,
    )
}

pub(super) fn estimate_sequence_entropy_section_size(
    sequences: &[PreparedSequence],
    modes: SequenceEntropyModes,
    compressed_tables: Option<&CompressedSequenceTables>,
    fse_tables: &FseTables,
    offset_history: OffsetHistory,
    write_entropy: bool,
) -> Option<usize> {
    estimate_superblock_sequence_section_size(
        sequences,
        sequence_table_modes(modes),
        compressed_tables,
        fse_tables,
        offset_history,
        write_entropy,
    )
}

pub(super) fn need_sequence_entropy_tables(modes: SequenceEntropyModes) -> bool {
    [modes.ll, modes.ml, modes.of]
        .iter()
        .any(|mode| matches!(mode, EntropyTableMode::Compressed | EntropyTableMode::Rle))
}

pub(super) fn select_sequence_entropy_modes(
    sequences: &[PreparedSequence],
    fse_tables: &FseTables,
    offset_history: OffsetHistory,
    strategy: Strategy,
) -> SequenceEntropyModes {
    let modes =
        select_c_sequence_table_modes(sequences, fse_tables, offset_history, strategy as u8);
    sequence_entropy_modes(modes)
}

fn sequence_modes_are(modes: SequenceEntropyModes, mode: EntropyTableMode) -> bool {
    modes.ll == mode && modes.ml == mode && modes.of == mode
}

pub(super) fn sequence_modes_need_compressed_tables(modes: SequenceEntropyModes) -> bool {
    [modes.ll, modes.ml, modes.of]
        .iter()
        .any(|mode| matches!(mode, EntropyTableMode::Compressed))
}

fn sequence_table_modes(modes: SequenceEntropyModes) -> SequenceTableModes {
    SequenceTableModes {
        ll: sequence_table_mode(modes.ll),
        ml: sequence_table_mode(modes.ml),
        of: sequence_table_mode(modes.of),
    }
}

fn sequence_table_mode(mode: EntropyTableMode) -> SequenceTableMode {
    match mode {
        EntropyTableMode::Basic => SequenceTableMode::Predefined,
        EntropyTableMode::Rle => SequenceTableMode::Rle,
        EntropyTableMode::Compressed => SequenceTableMode::Compressed,
        EntropyTableMode::Repeat => SequenceTableMode::Repeat,
    }
}

fn sequence_entropy_modes(modes: SequenceTableModes) -> SequenceEntropyModes {
    SequenceEntropyModes {
        ll: sequence_entropy_mode(modes.ll),
        ml: sequence_entropy_mode(modes.ml),
        of: sequence_entropy_mode(modes.of),
    }
}

fn sequence_entropy_mode(mode: SequenceTableMode) -> EntropyTableMode {
    match mode {
        SequenceTableMode::Predefined => EntropyTableMode::Basic,
        SequenceTableMode::Rle => EntropyTableMode::Rle,
        SequenceTableMode::Compressed => EntropyTableMode::Compressed,
        SequenceTableMode::Repeat => EntropyTableMode::Repeat,
    }
}
