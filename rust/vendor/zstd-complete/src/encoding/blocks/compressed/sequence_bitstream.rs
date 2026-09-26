use alloc::vec::Vec;
use core::convert::TryFrom;

use crate::{
    bit_io::BitWriter,
    encoding::{frame_compressor::OffsetHistory, levels::c_port::sequence_store::StoredSequence},
    fse::fse_encoder::{FSETable, FSETableBuildScratch, SharedFSETable},
};

use super::{
    c_sequence::CSequenceValues,
    sequence_codes::{encode_literal_length, encode_match_len, encode_offset},
    sequence_tables::{encode_table, FseTableMode},
    PreparedSequence,
};

pub(super) fn encode_table_count_size(
    mode: &FseTableMode<'_>,
    writer: &mut BitWriter<&mut Vec<u8>>,
) -> usize {
    let start = writer.index();
    encode_table(mode, writer);
    if matches!(mode, FseTableMode::Encoded(_)) {
        byte_size_between(start, writer.index())
    } else {
        0
    }
}

pub(super) fn byte_size_between(start_bits: usize, end_bits: usize) -> usize {
    debug_assert!(start_bits.is_multiple_of(8));
    debug_assert!(end_bits.is_multiple_of(8));
    (end_bits - start_bits) / 8
}

pub(super) fn should_emit_raw_for_legacy_decoder(
    last_count_size: usize,
    bitstream_size: usize,
) -> bool {
    // Mirrors zstd's compatibility guard for decoders <= 1.3.4, which
    // rejected compressed sequence tables when FSE_readNCount saw <4 bytes.
    last_count_size != 0 && last_count_size + bitstream_size < 4
}

pub(super) enum FseTableUpdate {
    Keep,
    Clear,
    Replace(FSETable),
}

pub(super) fn fse_table_update(mode: FseTableMode<'_>) -> FseTableUpdate {
    match mode {
        FseTableMode::Encoded(table) => FseTableUpdate::Replace(table),
        FseTableMode::Predefined(_) | FseTableMode::Rle(_) => FseTableUpdate::Clear,
        FseTableMode::RepeatLast(_) => FseTableUpdate::Keep,
    }
}

pub(super) fn apply_fse_table_update(
    previous: &mut Option<SharedFSETable>,
    repeat_valid: &mut bool,
    update: FseTableUpdate,
) {
    match update {
        FseTableUpdate::Keep => {}
        FseTableUpdate::Clear => {
            *previous = None;
            *repeat_valid = false;
        }
        FseTableUpdate::Replace(table) => {
            *previous = Some(SharedFSETable::new(table));
            *repeat_valid = false;
        }
    }
}

pub(super) fn apply_fse_table_update_with_scratch(
    previous: &mut Option<SharedFSETable>,
    repeat_valid: &mut bool,
    update: FseTableUpdate,
    mut scratch: Option<&mut FSETableBuildScratch>,
) {
    if !scratch
        .as_deref()
        .is_some_and(FSETableBuildScratch::has_shared_pool)
        && !crate::fse::fse_encoder::recycles_fast_fse_tables()
    {
        apply_fse_table_update(previous, repeat_valid, update);
        return;
    }

    match update {
        FseTableUpdate::Keep => {}
        FseTableUpdate::Clear => {
            recycle_previous_table(previous.take(), scratch);
            *repeat_valid = false;
        }
        FseTableUpdate::Replace(table) => {
            let table = match scratch.as_deref_mut() {
                Some(scratch) => scratch.share_table(table),
                None => SharedFSETable::new(table),
            };
            let old = previous.replace(table);
            recycle_previous_table(old, scratch);
            *repeat_valid = false;
        }
    }
}

pub(super) fn recycle_fse_table_update(
    update: FseTableUpdate,
    scratch: Option<&mut FSETableBuildScratch>,
) {
    if !scratch
        .as_deref()
        .is_some_and(FSETableBuildScratch::has_shared_pool)
        && !crate::fse::fse_encoder::recycles_fast_fse_tables()
    {
        return;
    }
    if let FseTableUpdate::Replace(table) = update {
        if let Some(scratch) = scratch {
            scratch.recycle_table(table);
        }
    }
}

fn recycle_previous_table(
    table: Option<SharedFSETable>,
    scratch: Option<&mut FSETableBuildScratch>,
) {
    let (Some(table), Some(scratch)) = (table, scratch) else {
        return;
    };
    if let Ok(table) = SharedFSETable::try_unwrap(table) {
        scratch.recycle_table(table);
    }
}

pub(super) fn encode_sequences_for_history(
    sequences: &[PreparedSequence],
    offset_history: &mut OffsetHistory,
) -> Vec<crate::blocks::sequence_section::Sequence> {
    let mut encoded = Vec::with_capacity(sequences.len());
    encode_sequences_for_history_into(sequences, offset_history, &mut encoded);
    encoded
}

pub(super) fn encode_sequences_for_history_into(
    sequences: &[PreparedSequence],
    offset_history: &mut OffsetHistory,
    encoded: &mut Vec<crate::blocks::sequence_section::Sequence>,
) {
    encoded.clear();
    encoded.reserve(sequences.len());
    let spare = encoded.spare_capacity_mut();
    debug_assert!(spare.len() >= sequences.len());
    for (slot, sequence) in spare.iter_mut().zip(sequences) {
        let of = if sequence.encoded_offset_value != 0 {
            let offset_value = sequence.encoded_offset_value;
            let raw_offset = offset_history.update_from_c_offset_value(offset_value, sequence.ll);
            debug_assert_eq!(raw_offset, sequence.raw_offset);
            offset_value
        } else {
            offset_history.encode_offset_value(sequence.raw_offset, sequence.ll)
        };
        slot.write(crate::blocks::sequence_section::Sequence {
            ll: sequence.ll,
            ml: sequence.ml,
            of,
        });
    }
    // SAFETY: the zip above writes exactly one element for every source
    // sequence into the contiguous spare prefix, and the reserve/debug
    // invariant proves that prefix is allocated.
    unsafe { encoded.set_len(sequences.len()) };
}

pub(super) fn encode_sequences_for_estimate_into(
    sequences: &[PreparedSequence],
    offset_history: OffsetHistory,
    encoded: &mut Vec<crate::blocks::sequence_section::Sequence>,
) {
    encoded.clear();

    if sequences
        .iter()
        .all(|sequence| sequence.encoded_offset_value != 0)
    {
        encoded.extend(sequences.iter().map(|sequence| {
            crate::blocks::sequence_section::Sequence {
                ll: sequence.ll,
                ml: sequence.ml,
                of: sequence.encoded_offset_value,
            }
        }));
        return;
    }

    let mut offset_history = offset_history;
    encode_sequences_for_history_into(sequences, &mut offset_history, encoded);
}

#[inline(always)]
pub(super) fn offset_to_u32(offset: usize) -> u32 {
    match u32::try_from(offset) {
        Ok(offset) => offset,
        Err(_) => unreachable!("match offsets are bounded by the compressor window"),
    }
}

pub(super) fn encode_sequences(
    sequences: &[crate::blocks::sequence_section::Sequence],
    writer: &mut BitWriter<&mut Vec<u8>>,
    ll_mode: &FseTableMode<'_>,
    ml_mode: &FseTableMode<'_>,
    of_mode: &FseTableMode<'_>,
) {
    if let (
        FseTableMode::Rle(ll_symbol),
        FseTableMode::Rle(ml_symbol),
        FseTableMode::Rle(of_symbol),
    ) = (ll_mode, ml_mode, of_mode)
    {
        encode_rle_sequences(sequences, writer, *ll_symbol, *ml_symbol, *of_symbol);
        return;
    }

    let ll_table = ll_mode.table();
    let ml_table = ml_mode.table();
    let of_table = of_mode.table();
    if let (Some(ll_table), Some(ml_table), Some(of_table)) = (ll_table, ml_table, of_table) {
        encode_table_sequences(sequences, writer, ll_table, ml_table, of_table);
        return;
    }

    let sequence = sequences[sequences.len() - 1];
    let (ll_code, ll_add_bits, ll_num_bits) = encode_literal_length(sequence.ll);
    let (of_code, of_add_bits, of_num_bits) = encode_offset(sequence.of);
    let (ml_code, ml_add_bits, ml_num_bits) = encode_match_len(sequence.ml);
    let mut ll_state = init_fse_state(ll_mode, ll_code);
    let mut ml_state = init_fse_state(ml_mode, ml_code);
    let mut of_state = init_fse_state(of_mode, of_code);

    write_sequence_extra_bits(
        writer,
        ll_add_bits,
        ll_num_bits,
        ml_add_bits,
        ml_num_bits,
        of_add_bits,
        of_num_bits,
    );

    // Encode backwards so the decoder reads the first sequence first.
    let mut sequence_idx = sequences.len() - 1;
    while sequence_idx > 0 {
        sequence_idx -= 1;
        let sequence = sequences[sequence_idx];
        let (ll_code, ll_add_bits, ll_num_bits) = encode_literal_length(sequence.ll);
        let (of_code, of_add_bits, of_num_bits) = encode_offset(sequence.of);
        let (ml_code, ml_add_bits, ml_num_bits) = encode_match_len(sequence.ml);

        {
            update_fse_state(of_table, &mut of_state, of_code, writer);
        }
        {
            update_fse_state(ml_table, &mut ml_state, ml_code, writer);
        }
        {
            update_fse_state(ll_table, &mut ll_state, ll_code, writer);
        }

        write_sequence_extra_bits(
            writer,
            ll_add_bits,
            ll_num_bits,
            ml_add_bits,
            ml_num_bits,
            of_add_bits,
            of_num_bits,
        );
    }
    flush_fse_state(ml_table, ml_state, writer);
    flush_fse_state(of_table, of_state, writer);
    flush_fse_state(ll_table, ll_state, writer);

    let bits_to_fill = writer.misaligned();
    if bits_to_fill == 0 {
        writer.write_bits(1u32, 8);
    } else {
        writer.write_bits(1u32, bits_to_fill);
    }
}

#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_seq0")]
#[cfg_attr(target_family = "windows", link_section = ".text$021.rz.seq0")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.021.ruzstd.sequence.emit"
)]
pub(super) fn encode_prepared_sequences(
    sequences: &[PreparedSequence],
    writer: &mut BitWriter<&mut Vec<u8>>,
    ll_mode: &FseTableMode<'_>,
    ml_mode: &FseTableMode<'_>,
    of_mode: &FseTableMode<'_>,
    c_fast_emission: bool,
) {
    encode_c_sequences(
        sequences,
        writer,
        ll_mode,
        ml_mode,
        of_mode,
        c_fast_emission,
    );
}

#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_seq2")]
#[cfg_attr(target_family = "windows", link_section = ".text$023.rz.seq2")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.023.ruzstd.sequence.emit.stored"
)]
pub(super) fn encode_stored_sequences(
    sequences: &[StoredSequence],
    writer: &mut BitWriter<&mut Vec<u8>>,
    ll_mode: &FseTableMode<'_>,
    ml_mode: &FseTableMode<'_>,
    of_mode: &FseTableMode<'_>,
    c_fast_emission: bool,
) {
    encode_c_sequences(
        sequences,
        writer,
        ll_mode,
        ml_mode,
        of_mode,
        c_fast_emission,
    );
}

fn encode_c_sequences<S: CSequenceValues>(
    sequences: &[S],
    writer: &mut BitWriter<&mut Vec<u8>>,
    ll_mode: &FseTableMode<'_>,
    ml_mode: &FseTableMode<'_>,
    of_mode: &FseTableMode<'_>,
    c_fast_emission: bool,
) {
    if let (
        FseTableMode::Rle(ll_symbol),
        FseTableMode::Rle(ml_symbol),
        FseTableMode::Rle(of_symbol),
    ) = (ll_mode, ml_mode, of_mode)
    {
        encode_rle_c_sequences(sequences, writer, *ll_symbol, *ml_symbol, *of_symbol);
        return;
    }

    let ll_table = ll_mode.table();
    let ml_table = ml_mode.table();
    let of_table = of_mode.table();
    if let (Some(ll_table), Some(ml_table), Some(of_table)) = (ll_table, ml_table, of_table) {
        if c_fast_emission {
            encode_c_fast_table_sequences(sequences, writer, ll_table, ml_table, of_table);
        } else {
            encode_table_c_sequences(sequences, writer, ll_table, ml_table, of_table);
        }
        return;
    }

    let sequence = sequences[sequences.len() - 1];
    let (ll_code, ll_add_bits, ll_num_bits) = encode_literal_length(sequence.literal_length());
    let offset_value = sequence.offset_value();
    debug_assert!(offset_value != 0);
    let (of_code, of_add_bits, of_num_bits) = encode_offset(offset_value);
    let (ml_code, ml_add_bits, ml_num_bits) = encode_match_len(sequence.match_length());
    let mut ll_state = init_fse_state(ll_mode, ll_code);
    let mut ml_state = init_fse_state(ml_mode, ml_code);
    let mut of_state = init_fse_state(of_mode, of_code);

    write_sequence_extra_bits(
        writer,
        ll_add_bits,
        ll_num_bits,
        ml_add_bits,
        ml_num_bits,
        of_add_bits,
        of_num_bits,
    );

    let mut sequence_idx = sequences.len() - 1;
    while sequence_idx > 0 {
        sequence_idx -= 1;
        let sequence = sequences[sequence_idx];
        let (ll_code, ll_add_bits, ll_num_bits) = encode_literal_length(sequence.literal_length());
        let offset_value = sequence.offset_value();
        debug_assert!(offset_value != 0);
        let (of_code, of_add_bits, of_num_bits) = encode_offset(offset_value);
        let (ml_code, ml_add_bits, ml_num_bits) = encode_match_len(sequence.match_length());

        update_fse_state(of_table, &mut of_state, of_code, writer);
        update_fse_state(ml_table, &mut ml_state, ml_code, writer);
        update_fse_state(ll_table, &mut ll_state, ll_code, writer);
        write_sequence_extra_bits(
            writer,
            ll_add_bits,
            ll_num_bits,
            ml_add_bits,
            ml_num_bits,
            of_add_bits,
            of_num_bits,
        );
    }
    flush_fse_state(ml_table, ml_state, writer);
    flush_fse_state(of_table, of_state, writer);
    flush_fse_state(ll_table, ll_state, writer);
    write_sequence_end_marker(writer);
}

/// Fast and DFast analogue of C's complete `ZSTD_encodeSequences_body()` hot
/// all-table transaction. Keeping this as one strategy-gated generated
/// function prevents its wider loop from perturbing Greedy/Lazy code while it
/// owns the three FSE transitions and the following sequence extra bits.
#[inline(never)]
fn encode_c_fast_table_sequences<S: CSequenceValues>(
    sequences: &[S],
    writer: &mut BitWriter<&mut Vec<u8>>,
    ll_table: &FSETable,
    ml_table: &FSETable,
    of_table: &FSETable,
) {
    #[cfg(target_arch = "x86_64")]
    if crate::cpu::bmi2_supported() {
        // SAFETY: the cached CPUID result proves BMI2 is available before the
        // target-feature function is entered.
        unsafe {
            encode_c_fast_table_sequences_bmi2(sequences, writer, ll_table, ml_table, of_table);
        }
        return;
    }

    encode_c_fast_table_sequences_body(sequences, writer, ll_table, ml_table, of_table);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "bmi2")]
#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_seq1")]
#[cfg_attr(target_family = "windows", link_section = ".text$022.rz.seq1")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.022.ruzstd.sequence.emit.bmi2"
)]
unsafe fn encode_c_fast_table_sequences_bmi2<S: CSequenceValues>(
    sequences: &[S],
    writer: &mut BitWriter<&mut Vec<u8>>,
    ll_table: &FSETable,
    ml_table: &FSETable,
    of_table: &FSETable,
) {
    encode_c_fast_table_sequences_body(sequences, writer, ll_table, ml_table, of_table);
}

#[inline(always)]
fn encode_c_fast_table_sequences_body<S: CSequenceValues>(
    sequences: &[S],
    writer: &mut BitWriter<&mut Vec<u8>>,
    ll_table: &FSETable,
    ml_table: &FSETable,
    of_table: &FSETable,
) {
    let sequence = sequences[sequences.len() - 1];
    let (ll_code, ll_add_bits, ll_num_bits) = encode_literal_length(sequence.literal_length());
    let offset_value = sequence.offset_value();
    debug_assert!(offset_value != 0);
    let (of_code, of_add_bits, of_num_bits) = encode_offset(offset_value);
    let (ml_code, ml_add_bits, ml_num_bits) = encode_match_len(sequence.match_length());
    let mut ll_state = ll_table.c_start_state_index(ll_code);
    let mut ml_state = ml_table.c_start_state_index(ml_code);
    let mut of_state = of_table.c_start_state_index(of_code);

    write_sequence_extra_bits(
        writer,
        ll_add_bits,
        ll_num_bits,
        ml_add_bits,
        ml_num_bits,
        of_add_bits,
        of_num_bits,
    );

    let mut sequence_idx = sequences.len() - 1;
    while sequence_idx > 0 {
        sequence_idx -= 1;
        let sequence = sequences[sequence_idx];
        let (ll_code, ll_add_bits, ll_num_bits) = encode_literal_length(sequence.literal_length());
        let offset_value = sequence.offset_value();
        debug_assert!(offset_value != 0);
        let (of_code, of_add_bits, of_num_bits) = encode_offset(offset_value);
        let (ml_code, ml_add_bits, ml_num_bits) = encode_match_len(sequence.match_length());

        let (of_bits, of_fse_bits, next_of_state) = of_table.encode_symbol(of_code, of_state);
        let (ml_bits, ml_fse_bits, next_ml_state) = ml_table.encode_symbol(ml_code, ml_state);
        let (ll_bits, ll_fse_bits, next_ll_state) = ll_table.encode_symbol(ll_code, ll_state);
        of_state = next_of_state;
        ml_state = next_ml_state;
        ll_state = next_ll_state;

        let of_fse_bits = usize::from(of_fse_bits);
        let ml_fse_bits = usize::from(ml_fse_bits);
        let ll_fse_bits = usize::from(ll_fse_bits);
        let fse_num_bits = of_fse_bits + ml_fse_bits + ll_fse_bits;
        debug_assert!(fse_num_bits <= 26);
        let fse_bits = u64::from(of_bits)
            | (u64::from(ml_bits) << of_fse_bits)
            | (u64::from(ll_bits) << (of_fse_bits + ml_fse_bits));

        write_c_fast_sequence_bits(
            writer,
            fse_bits,
            fse_num_bits,
            ll_add_bits,
            ll_num_bits,
            ml_add_bits,
            ml_num_bits,
            of_add_bits,
            of_num_bits,
        );
    }

    flush_table_fse_states(
        ml_table, ml_state, of_table, of_state, ll_table, ll_state, writer,
    );
    write_sequence_end_marker(writer);
}

/// C flushes after each sequence. The existing writer can preserve that bit
/// order while joining the FSE and extra-bit batches whenever their combined
/// width fits one container. The uncommon wider case is split exactly at the
/// 64-bit boundary.
#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn write_c_fast_sequence_bits(
    writer: &mut BitWriter<&mut Vec<u8>>,
    fse_bits: u64,
    fse_num_bits: usize,
    ll_bits: u32,
    ll_num_bits: usize,
    ml_bits: u32,
    ml_num_bits: usize,
    of_bits: u32,
    of_num_bits: usize,
) {
    let extra_num_bits = ll_num_bits + ml_num_bits + of_num_bits;
    debug_assert!(extra_num_bits < u64::BITS as usize);
    let extra_bits = u64::from(ll_bits)
        | (u64::from(ml_bits) << ll_num_bits)
        | (u64::from(of_bits) << (ll_num_bits + ml_num_bits));
    let total_num_bits = fse_num_bits + extra_num_bits;

    if total_num_bits <= u64::BITS as usize {
        writer.write_bits_64(fse_bits | (extra_bits << fse_num_bits), total_num_bits);
    } else {
        debug_assert!(fse_num_bits > 0);
        let bits_in_low_extra = u64::BITS as usize - fse_num_bits;
        writer.write_bits_64(fse_bits | (extra_bits << fse_num_bits), u64::BITS as usize);
        writer.write_bits_64(
            extra_bits >> bits_in_low_extra,
            total_num_bits - u64::BITS as usize,
        );
    }
}

fn encode_table_c_sequences<S: CSequenceValues>(
    sequences: &[S],
    writer: &mut BitWriter<&mut Vec<u8>>,
    ll_table: &FSETable,
    ml_table: &FSETable,
    of_table: &FSETable,
) {
    let sequence = sequences[sequences.len() - 1];
    let (ll_code, ll_add_bits, ll_num_bits) = encode_literal_length(sequence.literal_length());
    let offset_value = sequence.offset_value();
    debug_assert!(offset_value != 0);
    let (of_code, of_add_bits, of_num_bits) = encode_offset(offset_value);
    let (ml_code, ml_add_bits, ml_num_bits) = encode_match_len(sequence.match_length());
    let mut ll_state = ll_table.c_start_state_index(ll_code);
    let mut ml_state = ml_table.c_start_state_index(ml_code);
    let mut of_state = of_table.c_start_state_index(of_code);

    write_sequence_extra_bits(
        writer,
        ll_add_bits,
        ll_num_bits,
        ml_add_bits,
        ml_num_bits,
        of_add_bits,
        of_num_bits,
    );

    let mut sequence_idx = sequences.len() - 1;
    while sequence_idx > 0 {
        sequence_idx -= 1;
        let sequence = sequences[sequence_idx];
        let (ll_code, ll_add_bits, ll_num_bits) = encode_literal_length(sequence.literal_length());
        let offset_value = sequence.offset_value();
        debug_assert!(offset_value != 0);
        let (of_code, of_add_bits, of_num_bits) = encode_offset(offset_value);
        let (ml_code, ml_add_bits, ml_num_bits) = encode_match_len(sequence.match_length());

        update_table_fse_states(
            of_table,
            &mut of_state,
            of_code,
            ml_table,
            &mut ml_state,
            ml_code,
            ll_table,
            &mut ll_state,
            ll_code,
            writer,
        );
        write_sequence_extra_bits(
            writer,
            ll_add_bits,
            ll_num_bits,
            ml_add_bits,
            ml_num_bits,
            of_add_bits,
            of_num_bits,
        );
    }
    flush_table_fse_states(
        ml_table, ml_state, of_table, of_state, ll_table, ll_state, writer,
    );
    write_sequence_end_marker(writer);
}

fn encode_rle_c_sequences<S: CSequenceValues>(
    sequences: &[S],
    writer: &mut BitWriter<&mut Vec<u8>>,
    ll_symbol: u8,
    ml_symbol: u8,
    of_symbol: u8,
) {
    for &sequence in sequences.iter().rev() {
        let (ll_code, ll_add_bits, ll_num_bits) = encode_literal_length(sequence.literal_length());
        let offset_value = sequence.offset_value();
        debug_assert!(offset_value != 0);
        let (of_code, of_add_bits, of_num_bits) = encode_offset(offset_value);
        let (ml_code, ml_add_bits, ml_num_bits) = encode_match_len(sequence.match_length());
        debug_assert_eq!(ll_code, ll_symbol);
        debug_assert_eq!(ml_code, ml_symbol);
        debug_assert_eq!(of_code, of_symbol);
        write_sequence_extra_bits(
            writer,
            ll_add_bits,
            ll_num_bits,
            ml_add_bits,
            ml_num_bits,
            of_add_bits,
            of_num_bits,
        );
    }
    write_sequence_end_marker(writer);
}

#[inline(always)]
fn write_sequence_end_marker(writer: &mut BitWriter<&mut Vec<u8>>) {
    let bits_to_fill = writer.misaligned();
    if bits_to_fill == 0 {
        writer.write_bits(1u32, 8);
    } else {
        writer.write_bits(1u32, bits_to_fill);
    }
}

fn encode_table_sequences(
    sequences: &[crate::blocks::sequence_section::Sequence],
    writer: &mut BitWriter<&mut Vec<u8>>,
    ll_table: &FSETable,
    ml_table: &FSETable,
    of_table: &FSETable,
) {
    let sequence = sequences[sequences.len() - 1];
    let (ll_code, ll_add_bits, ll_num_bits) = encode_literal_length(sequence.ll);
    let (of_code, of_add_bits, of_num_bits) = encode_offset(sequence.of);
    let (ml_code, ml_add_bits, ml_num_bits) = encode_match_len(sequence.ml);
    let mut ll_state = ll_table.c_start_state_index(ll_code);
    let mut ml_state = ml_table.c_start_state_index(ml_code);
    let mut of_state = of_table.c_start_state_index(of_code);

    write_sequence_extra_bits(
        writer,
        ll_add_bits,
        ll_num_bits,
        ml_add_bits,
        ml_num_bits,
        of_add_bits,
        of_num_bits,
    );

    let mut sequence_idx = sequences.len() - 1;
    while sequence_idx > 0 {
        sequence_idx -= 1;
        let sequence = sequences[sequence_idx];
        let (ll_code, ll_add_bits, ll_num_bits) = encode_literal_length(sequence.ll);
        let (of_code, of_add_bits, of_num_bits) = encode_offset(sequence.of);
        let (ml_code, ml_add_bits, ml_num_bits) = encode_match_len(sequence.ml);

        update_table_fse_states(
            of_table,
            &mut of_state,
            of_code,
            ml_table,
            &mut ml_state,
            ml_code,
            ll_table,
            &mut ll_state,
            ll_code,
            writer,
        );

        write_sequence_extra_bits(
            writer,
            ll_add_bits,
            ll_num_bits,
            ml_add_bits,
            ml_num_bits,
            of_add_bits,
            of_num_bits,
        );
    }
    flush_table_fse_states(
        ml_table, ml_state, of_table, of_state, ll_table, ll_state, writer,
    );

    let bits_to_fill = writer.misaligned();
    if bits_to_fill == 0 {
        writer.write_bits(1u32, 8);
    } else {
        writer.write_bits(1u32, bits_to_fill);
    }
}

fn encode_rle_sequences(
    sequences: &[crate::blocks::sequence_section::Sequence],
    writer: &mut BitWriter<&mut Vec<u8>>,
    ll_symbol: u8,
    ml_symbol: u8,
    of_symbol: u8,
) {
    for sequence in sequences.iter().rev() {
        let (ll_code, ll_add_bits, ll_num_bits) = encode_literal_length(sequence.ll);
        let (of_code, of_add_bits, of_num_bits) = encode_offset(sequence.of);
        let (ml_code, ml_add_bits, ml_num_bits) = encode_match_len(sequence.ml);
        debug_assert_eq!(ll_code, ll_symbol);
        debug_assert_eq!(ml_code, ml_symbol);
        debug_assert_eq!(of_code, of_symbol);

        write_sequence_extra_bits(
            writer,
            ll_add_bits,
            ll_num_bits,
            ml_add_bits,
            ml_num_bits,
            of_add_bits,
            of_num_bits,
        );
    }

    let bits_to_fill = writer.misaligned();
    if bits_to_fill == 0 {
        writer.write_bits(1u32, 8);
    } else {
        writer.write_bits(1u32, bits_to_fill);
    }
}

fn init_fse_state(mode: &FseTableMode<'_>, symbol: u8) -> Option<u32> {
    match mode {
        FseTableMode::Rle(rle_symbol) => {
            debug_assert_eq!(*rle_symbol, symbol);
            None
        }
        _ => mode.table().map(|table| table.c_start_state_index(symbol)),
    }
}

fn update_fse_state(
    table: Option<&FSETable>,
    state: &mut Option<u32>,
    symbol: u8,
    writer: &mut BitWriter<&mut Vec<u8>>,
) {
    if let Some(table) = table {
        if let Some(current) = *state {
            let (bits, num_bits, next_state) = table.encode_symbol(symbol, current);
            writer.write_bits(u64::from(bits), usize::from(num_bits));
            *state = Some(next_state);
        } else {
            unreachable!("non-RLE FSE mode must have a state");
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn update_table_fse_states(
    of_table: &FSETable,
    of_state: &mut u32,
    of_symbol: u8,
    ml_table: &FSETable,
    ml_state: &mut u32,
    ml_symbol: u8,
    ll_table: &FSETable,
    ll_state: &mut u32,
    ll_symbol: u8,
    writer: &mut BitWriter<&mut Vec<u8>>,
) {
    let (of_bits, of_num_bits, next_of_state) = of_table.encode_symbol(of_symbol, *of_state);
    let (ml_bits, ml_num_bits, next_ml_state) = ml_table.encode_symbol(ml_symbol, *ml_state);
    let (ll_bits, ll_num_bits, next_ll_state) = ll_table.encode_symbol(ll_symbol, *ll_state);
    *of_state = next_of_state;
    *ml_state = next_ml_state;
    *ll_state = next_ll_state;
    let of_num_bits = usize::from(of_num_bits);
    let ml_num_bits = usize::from(ml_num_bits);
    let ll_num_bits = usize::from(ll_num_bits);
    let packed = u64::from(of_bits)
        | (u64::from(ml_bits) << of_num_bits)
        | (u64::from(ll_bits) << (of_num_bits + ml_num_bits));
    writer.write_bits_64(packed, of_num_bits + ml_num_bits + ll_num_bits);
}

fn flush_table_fse_states(
    ml_table: &FSETable,
    ml_state: u32,
    of_table: &FSETable,
    of_state: u32,
    ll_table: &FSETable,
    ll_state: u32,
    writer: &mut BitWriter<&mut Vec<u8>>,
) {
    let ml_bits = ml_table.state_bits(ml_state);
    let of_bits = of_table.state_bits(of_state);
    let ll_bits = ll_table.state_bits(ll_state);
    let ml_num_bits = usize::from(ml_table.acc_log());
    let of_num_bits = usize::from(of_table.acc_log());
    let ll_num_bits = usize::from(ll_table.acc_log());
    let packed = u64::from(ml_bits)
        | (u64::from(of_bits) << ml_num_bits)
        | (u64::from(ll_bits) << (ml_num_bits + of_num_bits));
    writer.write_bits_64(packed, ml_num_bits + of_num_bits + ll_num_bits);
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn write_sequence_extra_bits(
    writer: &mut BitWriter<&mut Vec<u8>>,
    ll_bits: u32,
    ll_num_bits: usize,
    ml_bits: u32,
    ml_num_bits: usize,
    of_bits: u32,
    of_num_bits: usize,
) {
    debug_assert!(ll_num_bits + ml_num_bits + of_num_bits < u64::BITS as usize);
    let packed = u64::from(ll_bits)
        | (u64::from(ml_bits) << ll_num_bits)
        | (u64::from(of_bits) << (ll_num_bits + ml_num_bits));
    writer.write_bits_64(packed, ll_num_bits + ml_num_bits + of_num_bits);
}

fn flush_fse_state(
    table: Option<&FSETable>,
    state: Option<u32>,
    writer: &mut BitWriter<&mut Vec<u8>>,
) {
    if let Some(table) = table {
        if let Some(state) = state {
            writer.write_bits(u64::from(table.state_bits(state)), table.acc_log() as usize);
        } else {
            unreachable!("non-RLE FSE mode must have a state");
        }
    }
}

pub(super) fn encode_seqnum(seqnum: usize, writer: &mut BitWriter<impl AsMut<Vec<u8>>>) {
    const UPPER_LIMIT: usize = 0xFFFF + 0x7F00;
    match seqnum {
        1..=127 => writer.write_bits(seqnum as u32, 8),
        128..=0x7FFF => {
            let upper = ((seqnum >> 8) | 0x80) as u8;
            let lower = seqnum as u8;
            writer.write_bits(upper, 8);
            writer.write_bits(lower, 8);
        }
        0x8000..=UPPER_LIMIT => {
            let encode = seqnum - 0x7F00;
            let upper = (encode >> 8) as u8;
            let lower = encode as u8;
            writer.write_bits(255u8, 8);
            writer.write_bits(upper, 8);
            writer.write_bits(lower, 8);
        }
        _ => unreachable!(),
    }
}
