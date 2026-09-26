use alloc::vec::Vec;

use crate::{bit_io::BitWriter, blocks::sequence_section::Sequence, fse::fse_encoder::FSETable};

use super::encode_sequences;

mod builder;
mod selection;

pub(in crate::encoding::blocks::compressed) use builder::{build_sequence_table, TableBuilder};
#[cfg(test)]
pub(super) use selection::choose_table;
pub(super) use selection::{
    choose_c_dfast_compact_sequence_table_modes_from_prepared,
    choose_c_dfast_compact_sequence_table_modes_from_stored,
    choose_c_dfast_compact_sequence_table_modes_from_stored_with_final_history,
    choose_c_fast_sequence_table_modes_from_prepared,
    choose_c_fast_sequence_table_modes_from_stored,
    choose_c_fast_sequence_table_modes_from_stored_with_final_history,
    choose_c_sequence_table_modes_from_prepared, choose_c_sequence_table_modes_from_stored,
    choose_c_sequence_table_modes_from_stored_with_scratch, choose_sequence_table_modes,
    choose_sequence_table_modes_for_estimate,
    choose_sequence_table_modes_for_estimate_from_counts_with_scratch,
};

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub(super) enum FseTableMode<'a> {
    Predefined(&'a FSETable),
    Rle(u8),
    Encoded(FSETable),
    RepeatLast(&'a FSETable),
}

impl FseTableMode<'_> {
    pub(super) fn table(&self) -> Option<&FSETable> {
        match self {
            Self::Predefined(t) => Some(t),
            Self::RepeatLast(t) => Some(t),
            Self::Encoded(t) => Some(t),
            Self::Rle(_) => None,
        }
    }
}

pub(super) struct SequenceModeSearchConfig<'a> {
    pub(super) ll_previous: Option<&'a FSETable>,
    pub(super) ll_repeat_valid: bool,
    pub(super) ll_default: &'a FSETable,
    pub(super) ml_previous: Option<&'a FSETable>,
    pub(super) ml_repeat_valid: bool,
    pub(super) ml_default: &'a FSETable,
    pub(super) of_previous: Option<&'a FSETable>,
    pub(super) of_repeat_valid: bool,
    pub(super) of_default: &'a FSETable,
    pub(super) repeat_table_max_sequences: usize,
    pub(super) llml_predefined_max_sequences: usize,
    pub(super) of_predefined_max_sequences: usize,
    pub(super) of_max_log: u8,
    pub(super) exact_sequence_mode_search: bool,
    pub(super) c_fast_heuristics: bool,
    pub(super) c_cost_model: bool,
}

pub(super) fn exact_sequence_section_size(
    sequences: &[Sequence],
    ll_mode: &FseTableMode<'_>,
    ml_mode: &FseTableMode<'_>,
    of_mode: &FseTableMode<'_>,
) -> usize {
    let mut encoded = Vec::new();
    let mut writer = BitWriter::from(&mut encoded);
    writer.write_bits(encode_fse_table_modes(ll_mode, ml_mode, of_mode), 8);
    encode_table(ll_mode, &mut writer);
    encode_table(of_mode, &mut writer);
    encode_table(ml_mode, &mut writer);
    encode_sequences(sequences, &mut writer, ll_mode, ml_mode, of_mode);
    writer.flush();
    encoded.len()
}

pub(super) fn encode_table(mode: &FseTableMode<'_>, writer: &mut BitWriter<&mut Vec<u8>>) {
    match mode {
        FseTableMode::Predefined(_) => {}
        FseTableMode::Rle(symbol) => writer.write_bits(*symbol, 8),
        FseTableMode::RepeatLast(_) => {}
        FseTableMode::Encoded(table) => table.write_table(writer),
    }
}

pub(super) fn encode_fse_table_modes(
    ll_mode: &FseTableMode<'_>,
    ml_mode: &FseTableMode<'_>,
    of_mode: &FseTableMode<'_>,
) -> u8 {
    fn mode_to_bits(mode: &FseTableMode<'_>) -> u8 {
        match mode {
            FseTableMode::Predefined(_) => 0,
            FseTableMode::Rle(_) => 1,
            FseTableMode::Encoded(_) => 2,
            FseTableMode::RepeatLast(_) => 3,
        }
    }
    mode_to_bits(ll_mode) << 6 | mode_to_bits(of_mode) << 4 | mode_to_bits(ml_mode) << 2
}
