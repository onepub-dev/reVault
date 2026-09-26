use alloc::vec::Vec;

use crate::{
    bit_io::BitWriter,
    blocks::sequence_section::Sequence,
    encoding::{
        blocks::PreparedSequence,
        frame_compressor::{FseTables, OffsetHistory},
    },
    fse::fse_encoder::FSETable,
};

use super::{
    literal_length_code, match_length_code, offset_code,
    sequence_bitstream::{
        apply_fse_table_update, byte_size_between, encode_seqnum, encode_sequences,
        encode_sequences_for_history_into, encode_table_count_size, fse_table_update,
        should_emit_raw_for_legacy_decoder,
    },
    sequence_tables::{
        build_sequence_table, encode_fse_table_modes, encode_table, FseTableMode, TableBuilder,
    },
};

#[derive(Clone)]
pub(crate) struct CompressedSequenceTables {
    pub(super) ll: FSETable,
    pub(super) ml: FSETable,
    pub(super) of: FSETable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SequenceTableMode {
    Predefined,
    Rle,
    Compressed,
    Repeat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SequenceTableModes {
    pub(crate) ll: SequenceTableMode,
    pub(crate) ml: SequenceTableMode,
    pub(crate) of: SequenceTableMode,
}

pub(crate) fn build_compressed_sequence_tables(
    sequences: &[PreparedSequence],
    offset_history: OffsetHistory,
) -> Option<CompressedSequenceTables> {
    if sequences.len() <= 1 {
        return None;
    }

    let mut offset_history = offset_history;
    let mut encoded_sequences = Vec::with_capacity(sequences.len());
    encode_sequences_for_history_into(sequences, &mut offset_history, &mut encoded_sequences);
    Some(CompressedSequenceTables {
        ll: build_sequence_table(
            &encoded_sequences,
            |seq| literal_length_code(seq.ll),
            9,
            TableBuilder::Full,
        ),
        ml: build_sequence_table(
            &encoded_sequences,
            |seq| match_length_code(seq.ml),
            9,
            TableBuilder::Full,
        ),
        of: build_sequence_table(
            &encoded_sequences,
            |seq| offset_code(seq.of),
            8,
            TableBuilder::Full,
        ),
    })
}

pub(crate) fn append_predefined_sequence_section(
    sequences: &[PreparedSequence],
    fse_tables: &FseTables,
    offset_history: &mut OffsetHistory,
    output: &mut Vec<u8>,
) -> Option<usize> {
    if sequences.is_empty() {
        output.push(0);
        return Some(1);
    }

    let previous_offsets = *offset_history;
    let mut encoded_sequences = Vec::with_capacity(sequences.len());
    encode_sequences_for_history_into(sequences, offset_history, &mut encoded_sequences);

    let ll_mode = FseTableMode::Predefined(&fse_tables.ll_default);
    let ml_mode = FseTableMode::Predefined(&fse_tables.ml_default);
    let of_mode = FseTableMode::Predefined(&fse_tables.of_default);
    if !sequence_modes_can_encode(&encoded_sequences, &ll_mode, &ml_mode, &of_mode) {
        *offset_history = previous_offsets;
        return None;
    }

    let start = output.len();
    let mut writer = BitWriter::from(output);
    encode_seqnum(encoded_sequences.len(), &mut writer);
    let sequence_head_index = writer.index() / 8;
    writer.write_bits(encode_fse_table_modes(&ll_mode, &ml_mode, &of_mode), 8);
    encode_sequences(
        &encoded_sequences,
        &mut writer,
        &ll_mode,
        &ml_mode,
        &of_mode,
    );
    writer.flush();

    let byte_size = writer.index() / 8 - start;
    if writer.index() / 8 - sequence_head_index < 4 {
        writer.reset_to(start * 8);
        *offset_history = previous_offsets;
        return None;
    }
    Some(byte_size)
}

pub(crate) fn append_rle_sequence_section(
    sequences: &[PreparedSequence],
    offset_history: &mut OffsetHistory,
    output: &mut Vec<u8>,
) -> Option<usize> {
    if sequences.is_empty() {
        output.push(0);
        return Some(1);
    }

    let previous_offsets = *offset_history;
    let mut encoded_sequences = Vec::with_capacity(sequences.len());
    encode_sequences_for_history_into(sequences, offset_history, &mut encoded_sequences);
    let Some((ll_symbol, ml_symbol, of_symbol)) = rle_sequence_symbols(&encoded_sequences) else {
        *offset_history = previous_offsets;
        return None;
    };

    let start = output.len();
    let mut writer = BitWriter::from(output);
    encode_seqnum(encoded_sequences.len(), &mut writer);
    let sequence_head_index = writer.index() / 8;
    let ll_mode = FseTableMode::Rle(ll_symbol);
    let ml_mode = FseTableMode::Rle(ml_symbol);
    let of_mode = FseTableMode::Rle(of_symbol);
    writer.write_bits(encode_fse_table_modes(&ll_mode, &ml_mode, &of_mode), 8);
    encode_table(&ll_mode, &mut writer);
    encode_table(&of_mode, &mut writer);
    encode_table(&ml_mode, &mut writer);
    encode_sequences(
        &encoded_sequences,
        &mut writer,
        &ll_mode,
        &ml_mode,
        &of_mode,
    );
    writer.flush();

    let byte_size = writer.index() / 8 - start;
    if writer.index() / 8 - sequence_head_index < 4 {
        writer.reset_to(start * 8);
        *offset_history = previous_offsets;
        return None;
    }
    Some(byte_size)
}

pub(crate) fn append_repeat_sequence_section(
    sequences: &[PreparedSequence],
    fse_tables: &FseTables,
    offset_history: &mut OffsetHistory,
    output: &mut Vec<u8>,
) -> Option<usize> {
    if sequences.is_empty() {
        output.push(0);
        return Some(1);
    }

    let ll_previous = fse_tables.ll_previous.as_deref()?;
    let ml_previous = fse_tables.ml_previous.as_deref()?;
    let of_previous = fse_tables.of_previous.as_deref()?;
    let previous_offsets = *offset_history;
    let mut encoded_sequences = Vec::with_capacity(sequences.len());
    encode_sequences_for_history_into(sequences, offset_history, &mut encoded_sequences);
    if !repeat_tables_can_encode(&encoded_sequences, ll_previous, ml_previous, of_previous) {
        *offset_history = previous_offsets;
        return None;
    }

    let start = output.len();
    let mut writer = BitWriter::from(output);
    encode_seqnum(encoded_sequences.len(), &mut writer);
    let sequence_head_index = writer.index() / 8;
    let ll_mode = FseTableMode::RepeatLast(ll_previous);
    let ml_mode = FseTableMode::RepeatLast(ml_previous);
    let of_mode = FseTableMode::RepeatLast(of_previous);
    writer.write_bits(encode_fse_table_modes(&ll_mode, &ml_mode, &of_mode), 8);
    encode_sequences(
        &encoded_sequences,
        &mut writer,
        &ll_mode,
        &ml_mode,
        &of_mode,
    );
    writer.flush();

    let byte_size = writer.index() / 8 - start;
    if writer.index() / 8 - sequence_head_index < 4 {
        writer.reset_to(start * 8);
        *offset_history = previous_offsets;
        return None;
    }
    Some(byte_size)
}

pub(crate) fn append_compressed_sequence_section(
    sequences: &[PreparedSequence],
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    output: &mut Vec<u8>,
) -> Option<usize> {
    if sequences.is_empty() {
        output.push(0);
        return Some(1);
    }
    if sequences.len() == 1 {
        return None;
    }

    let tables = build_compressed_sequence_tables(sequences, *offset_history)?;
    append_compressed_sequence_section_with_tables(
        sequences,
        &tables,
        fse_tables,
        offset_history,
        output,
    )
}

pub(crate) fn append_compressed_sequence_section_with_tables(
    sequences: &[PreparedSequence],
    tables: &CompressedSequenceTables,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    output: &mut Vec<u8>,
) -> Option<usize> {
    if sequences.is_empty() {
        output.push(0);
        return Some(1);
    }

    let previous_offsets = *offset_history;
    let previous_fse = fse_tables.snapshot_previous();
    let mut encoded_sequences = Vec::with_capacity(sequences.len());
    encode_sequences_for_history_into(sequences, offset_history, &mut encoded_sequences);
    let start = output.len();
    let mut writer = BitWriter::from(output);
    encode_seqnum(encoded_sequences.len(), &mut writer);
    let sequence_head_index = writer.index() / 8;
    let ll_mode = FseTableMode::Encoded(tables.ll.clone());
    let ml_mode = FseTableMode::Encoded(tables.ml.clone());
    let of_mode = FseTableMode::Encoded(tables.of.clone());
    writer.write_bits(encode_fse_table_modes(&ll_mode, &ml_mode, &of_mode), 8);

    let mut last_count_size = encode_table_count_size(&ll_mode, &mut writer);
    let off_count_size = encode_table_count_size(&of_mode, &mut writer);
    if off_count_size != 0 {
        last_count_size = off_count_size;
    }
    let ml_count_size = encode_table_count_size(&ml_mode, &mut writer);
    if ml_count_size != 0 {
        last_count_size = ml_count_size;
    }

    let bitstream_start = writer.index();
    encode_sequences(
        &encoded_sequences,
        &mut writer,
        &ll_mode,
        &ml_mode,
        &of_mode,
    );
    let bitstream_size = byte_size_between(bitstream_start, writer.index());
    writer.flush();

    if should_emit_raw_for_legacy_decoder(last_count_size, bitstream_size)
        || writer.index() / 8 - sequence_head_index < 4
    {
        writer.reset_to(start * 8);
        *offset_history = previous_offsets;
        fse_tables.restore_previous(previous_fse);
        return None;
    }

    let byte_size = writer.index() / 8 - start;
    apply_fse_table_update(
        &mut fse_tables.ll_previous,
        &mut fse_tables.ll_repeat_valid,
        fse_table_update(ll_mode),
    );
    apply_fse_table_update(
        &mut fse_tables.ml_previous,
        &mut fse_tables.ml_repeat_valid,
        fse_table_update(ml_mode),
    );
    apply_fse_table_update(
        &mut fse_tables.of_previous,
        &mut fse_tables.of_repeat_valid,
        fse_table_update(of_mode),
    );
    Some(byte_size)
}

pub(crate) fn append_sequence_section_with_table_modes(
    sequences: &[PreparedSequence],
    modes: SequenceTableModes,
    compressed_tables: Option<&CompressedSequenceTables>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    output: &mut Vec<u8>,
) -> Option<usize> {
    if sequences.is_empty() {
        output.push(0);
        return Some(1);
    }

    let previous_offsets = *offset_history;
    let previous_fse = fse_tables.snapshot_previous();
    let mut encoded_sequences = Vec::with_capacity(sequences.len());
    encode_sequences_for_history_into(sequences, offset_history, &mut encoded_sequences);

    let ll_mode = literal_length_mode(modes.ll, &encoded_sequences, compressed_tables, fse_tables)?;
    let ml_mode = match_length_mode(modes.ml, &encoded_sequences, compressed_tables, fse_tables)?;
    let of_mode = offset_mode(modes.of, &encoded_sequences, compressed_tables, fse_tables)?;
    if !sequence_modes_can_encode(&encoded_sequences, &ll_mode, &ml_mode, &of_mode) {
        *offset_history = previous_offsets;
        fse_tables.restore_previous(previous_fse);
        return None;
    }

    let start = output.len();
    let mut writer = BitWriter::from(output);
    encode_seqnum(encoded_sequences.len(), &mut writer);
    let sequence_head_index = writer.index() / 8;
    writer.write_bits(encode_fse_table_modes(&ll_mode, &ml_mode, &of_mode), 8);

    let mut last_count_size = encode_table_count_size(&ll_mode, &mut writer);
    let off_count_size = encode_table_count_size(&of_mode, &mut writer);
    if off_count_size != 0 {
        last_count_size = off_count_size;
    }
    let ml_count_size = encode_table_count_size(&ml_mode, &mut writer);
    if ml_count_size != 0 {
        last_count_size = ml_count_size;
    }

    let bitstream_start = writer.index();
    encode_sequences(
        &encoded_sequences,
        &mut writer,
        &ll_mode,
        &ml_mode,
        &of_mode,
    );
    let bitstream_size = byte_size_between(bitstream_start, writer.index());
    writer.flush();

    if should_emit_raw_for_legacy_decoder(last_count_size, bitstream_size)
        || writer.index() / 8 - sequence_head_index < 4
    {
        writer.reset_to(start * 8);
        *offset_history = previous_offsets;
        fse_tables.restore_previous(previous_fse);
        return None;
    }

    let byte_size = writer.index() / 8 - start;
    let ll_update = fse_table_update(ll_mode);
    let ml_update = fse_table_update(ml_mode);
    let of_update = fse_table_update(of_mode);
    apply_fse_table_update(
        &mut fse_tables.ll_previous,
        &mut fse_tables.ll_repeat_valid,
        ll_update,
    );
    apply_fse_table_update(
        &mut fse_tables.ml_previous,
        &mut fse_tables.ml_repeat_valid,
        ml_update,
    );
    apply_fse_table_update(
        &mut fse_tables.of_previous,
        &mut fse_tables.of_repeat_valid,
        of_update,
    );
    Some(byte_size)
}

fn literal_length_mode<'a>(
    mode: SequenceTableMode,
    sequences: &[Sequence],
    compressed_tables: Option<&CompressedSequenceTables>,
    fse_tables: &'a FseTables,
) -> Option<FseTableMode<'a>> {
    match mode {
        SequenceTableMode::Predefined => Some(FseTableMode::Predefined(&fse_tables.ll_default)),
        SequenceTableMode::Rle => {
            uniform_code(sequences, |sequence| literal_length_code(sequence.ll))
                .map(FseTableMode::Rle)
        }
        SequenceTableMode::Compressed => Some(FseTableMode::Encoded(compressed_tables?.ll.clone())),
        SequenceTableMode::Repeat => {
            Some(FseTableMode::RepeatLast(fse_tables.ll_previous.as_deref()?))
        }
    }
}

fn match_length_mode<'a>(
    mode: SequenceTableMode,
    sequences: &[Sequence],
    compressed_tables: Option<&CompressedSequenceTables>,
    fse_tables: &'a FseTables,
) -> Option<FseTableMode<'a>> {
    match mode {
        SequenceTableMode::Predefined => Some(FseTableMode::Predefined(&fse_tables.ml_default)),
        SequenceTableMode::Rle => {
            uniform_code(sequences, |sequence| match_length_code(sequence.ml))
                .map(FseTableMode::Rle)
        }
        SequenceTableMode::Compressed => Some(FseTableMode::Encoded(compressed_tables?.ml.clone())),
        SequenceTableMode::Repeat => {
            Some(FseTableMode::RepeatLast(fse_tables.ml_previous.as_deref()?))
        }
    }
}

fn offset_mode<'a>(
    mode: SequenceTableMode,
    sequences: &[Sequence],
    compressed_tables: Option<&CompressedSequenceTables>,
    fse_tables: &'a FseTables,
) -> Option<FseTableMode<'a>> {
    match mode {
        SequenceTableMode::Predefined => Some(FseTableMode::Predefined(&fse_tables.of_default)),
        SequenceTableMode::Rle => {
            uniform_code(sequences, |sequence| offset_code(sequence.of)).map(FseTableMode::Rle)
        }
        SequenceTableMode::Compressed => Some(FseTableMode::Encoded(compressed_tables?.of.clone())),
        SequenceTableMode::Repeat => {
            Some(FseTableMode::RepeatLast(fse_tables.of_previous.as_deref()?))
        }
    }
}

fn uniform_code(sequences: &[Sequence], code: impl Fn(Sequence) -> u8) -> Option<u8> {
    let first = code(*sequences.first()?);
    sequences
        .iter()
        .all(|sequence| code(*sequence) == first)
        .then_some(first)
}

fn repeat_tables_can_encode(
    sequences: &[Sequence],
    ll_table: &FSETable,
    ml_table: &FSETable,
    of_table: &FSETable,
) -> bool {
    sequences.iter().all(|sequence| {
        ll_table.can_encode_symbol(literal_length_code(sequence.ll))
            && ml_table.can_encode_symbol(match_length_code(sequence.ml))
            && of_table.can_encode_symbol(offset_code(sequence.of))
    })
}

fn sequence_modes_can_encode(
    sequences: &[Sequence],
    ll_mode: &FseTableMode<'_>,
    ml_mode: &FseTableMode<'_>,
    of_mode: &FseTableMode<'_>,
) -> bool {
    sequences.iter().all(|sequence| {
        fse_mode_can_encode(ll_mode, literal_length_code(sequence.ll))
            && fse_mode_can_encode(ml_mode, match_length_code(sequence.ml))
            && fse_mode_can_encode(of_mode, offset_code(sequence.of))
    })
}

fn fse_mode_can_encode(mode: &FseTableMode<'_>, symbol: u8) -> bool {
    match mode {
        FseTableMode::Predefined(table) | FseTableMode::RepeatLast(table) => {
            table.can_encode_symbol(symbol)
        }
        FseTableMode::Encoded(table) => table.can_encode_symbol(symbol),
        FseTableMode::Rle(rle_symbol) => *rle_symbol == symbol,
    }
}

fn rle_sequence_symbols(sequences: &[Sequence]) -> Option<(u8, u8, u8)> {
    let first = *sequences.first()?;
    let symbols = sequence_symbols(first);
    sequences
        .iter()
        .all(|sequence| sequence_symbols(*sequence) == symbols)
        .then_some(symbols)
}

fn sequence_symbols(sequence: Sequence) -> (u8, u8, u8) {
    (
        literal_length_code(sequence.ll),
        match_length_code(sequence.ml),
        offset_code(sequence.of),
    )
}
