use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::sync::OnceLock;

mod c_sequence;
mod config;
mod estimate;
mod literal_sections;
mod literals;
mod sequence_bitstream;
mod sequence_codes;
mod sequence_cost;
mod sequence_sections;
mod sequence_superblock;
mod sequence_tables;

pub(crate) use config::BlockCompressionConfig;
pub(crate) use estimate::{estimate_prepared_block_size_with_sequences, EstimateScratch};
pub(crate) use literal_sections::{
    append_huffman_literal_section_with_optimal_depth, append_huffman_literal_section_with_table,
    build_huffman_literal_table_with_optimal_depth, estimate_huffman_literal_section_with_table,
    HuffmanLiteralMode,
};
use literals::{
    compress_literals, compress_literals_with_scratch, raw_literals, should_compress_literals,
    suspect_uncompressible_literals, LiteralCompressionOptions, COMPRESS_LITERALS_SIZE_MIN,
};
#[cfg(test)]
use literals::{
    compressed_literals_header_len, compressed_literals_size_format,
    literal_estimate_has_enough_gain, literal_min_gain, rle_literals, LiteralStats,
    REPEAT_LITERALS_SIZE_MIN,
};
use sequence_bitstream::{
    apply_fse_table_update, apply_fse_table_update_with_scratch, byte_size_between,
    encode_prepared_sequences, encode_seqnum, encode_sequences, encode_sequences_for_history,
    encode_stored_sequences, encode_table_count_size, fse_table_update, offset_to_u32,
    recycle_fse_table_update, should_emit_raw_for_legacy_decoder,
};
#[cfg(test)]
use sequence_codes::{encode_literal_length, encode_match_len, encode_offset};
pub(crate) use sequence_codes::{literal_length_code, match_length_code, offset_code};
pub(crate) use sequence_sections::{
    append_compressed_sequence_section, append_predefined_sequence_section,
    append_repeat_sequence_section, append_rle_sequence_section,
    append_sequence_section_with_table_modes, build_compressed_sequence_tables,
    CompressedSequenceTables, SequenceTableMode, SequenceTableModes,
};
pub(crate) use sequence_superblock::{
    estimate_superblock_sequence_section_size, finish_sequence_tables_after_superblock,
    prime_sequence_tables_for_repeat, select_c_sequence_table_modes,
};
use sequence_tables::{
    choose_c_dfast_compact_sequence_table_modes_from_prepared,
    choose_c_dfast_compact_sequence_table_modes_from_stored,
    choose_c_dfast_compact_sequence_table_modes_from_stored_with_final_history,
    choose_c_fast_sequence_table_modes_from_prepared,
    choose_c_fast_sequence_table_modes_from_stored,
    choose_c_fast_sequence_table_modes_from_stored_with_final_history,
    choose_c_sequence_table_modes_from_prepared, choose_c_sequence_table_modes_from_stored,
    choose_c_sequence_table_modes_from_stored_with_scratch, choose_sequence_table_modes,
    encode_fse_table_modes, SequenceModeSearchConfig,
};
#[cfg(test)]
use sequence_tables::{choose_table, encode_table, exact_sequence_section_size, FseTableMode};

#[cfg(test)]
mod tests;

use crate::encoding::levels::c_port::sequence_store::StoredSequence;
use crate::{
    bit_io::BitWriter,
    encoding::frame_compressor::{CompressState, FseTables, OffsetHistory},
    encoding::util::likely_dependency_json_lockfile_text,
    encoding::{CompressionFileProfile, Matcher, Sequence},
    fse::fse_encoder::FSETableBuildScratch,
    huff0::huff0_encoder,
};

const INITIAL_LITERALS_CAPACITY: usize = 1024;
const INITIAL_SEQUENCES_CAPACITY: usize = 256;
const SMALL_HUFFMAN_TABLE_SEARCH_MAX_LITERALS: usize = 256;
const SMALL_HUFFMAN_TABLE_SEARCH_MAX_SEQUENCES: usize = 2;
const FILE_TYPE_SMALL_HUFFMAN_TABLE_SEARCH_MAX_LITERALS: usize = 4 * 1024;
const EXACT_SEQUENCE_TABLE_MIN_LOG: u8 = 7;

#[derive(Clone, Debug)]
pub(crate) struct PreparedBlock {
    pub(crate) literals: Vec<u8>,
    pub(crate) sequences: Vec<PreparedSequence>,
}

pub(crate) struct CompressedBlockResult {
    pub(crate) new_huffman_table: Option<huff0_encoder::HuffmanTable>,
    pub(crate) should_emit_raw_block: bool,
}

struct PendingFseTableUpdates {
    ll: sequence_bitstream::FseTableUpdate,
    ml: sequence_bitstream::FseTableUpdate,
    of: sequence_bitstream::FseTableUpdate,
}

pub(crate) struct PendingStoredEntropyState {
    fse_updates: Option<PendingFseTableUpdates>,
    offset_history: Option<OffsetHistory>,
}

impl PendingStoredEntropyState {
    pub(crate) const fn new() -> Self {
        Self {
            fse_updates: None,
            offset_history: None,
        }
    }

    pub(crate) fn commit(
        &mut self,
        fse_tables: &mut FseTables,
        offset_history: &mut OffsetHistory,
        mut fse_build_scratch: Option<&mut FSETableBuildScratch>,
    ) {
        if let Some(updates) = self.fse_updates.take() {
            apply_fse_table_update_with_scratch(
                &mut fse_tables.ll_previous,
                &mut fse_tables.ll_repeat_valid,
                updates.ll,
                fse_build_scratch.as_deref_mut(),
            );
            apply_fse_table_update_with_scratch(
                &mut fse_tables.ml_previous,
                &mut fse_tables.ml_repeat_valid,
                updates.ml,
                fse_build_scratch.as_deref_mut(),
            );
            apply_fse_table_update_with_scratch(
                &mut fse_tables.of_previous,
                &mut fse_tables.of_repeat_valid,
                updates.of,
                fse_build_scratch,
            );
        }
        if let Some(next_offset_history) = self.offset_history.take() {
            *offset_history = next_offset_history;
        }
    }

    pub(crate) fn discard(&mut self, mut fse_build_scratch: Option<&mut FSETableBuildScratch>) {
        if let Some(updates) = self.fse_updates.take() {
            recycle_fse_table_update(updates.ll, fse_build_scratch.as_deref_mut());
            recycle_fse_table_update(updates.ml, fse_build_scratch.as_deref_mut());
            recycle_fse_table_update(updates.of, fse_build_scratch);
        }
        self.offset_history = None;
    }
}

#[cfg(feature = "std")]
static C_DEFER_STORED_ENTROPY_COMMIT: OnceLock<bool> = OnceLock::new();

pub(crate) fn defers_stored_entropy_commit() -> bool {
    #[cfg(feature = "std")]
    {
        *C_DEFER_STORED_ENTROPY_COMMIT.get_or_init(|| {
            std::env::var("RUZSTD_TUNE_C_DEFER_STORED_ENTROPY_COMMIT")
                .map(|value| !matches!(value.as_str(), "0" | "false" | "FALSE" | "off" | "OFF"))
                .unwrap_or(true)
        })
    }
    #[cfg(not(feature = "std"))]
    {
        true
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PreparedBlockRef<'a> {
    pub(crate) literals: &'a [u8],
    pub(crate) sequences: &'a [PreparedSequence],
}

#[derive(Clone, Copy)]
pub(crate) struct StoredBlockRef<'a> {
    pub(crate) literals: &'a [u8],
    pub(crate) sequences: &'a [StoredSequence],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct PreparedSequence {
    pub(crate) ll: u32,
    pub(crate) ml: u32,
    pub(crate) raw_offset: u32,
    /// C-port matchers store C's nonzero numeric `offBase` directly. Generic
    /// matchers use zero and let `OffsetHistory` choose an encoding.
    pub(crate) encoded_offset_value: u32,
}

impl PreparedBlock {
    pub(crate) fn as_ref(&self) -> PreparedBlockRef<'_> {
        PreparedBlockRef {
            literals: &self.literals,
            sequences: &self.sequences,
        }
    }
}

pub(crate) fn compress_block_with_config<M: Matcher>(
    state: &mut CompressState<M>,
    output: &mut Vec<u8>,
    config: BlockCompressionConfig,
) -> Option<huff0_encoder::HuffmanTable> {
    let mut config = config;
    if matches!(state.file_profile_hint, CompressionFileProfile::None)
        && likely_dependency_json_lockfile_text(state.matcher.get_last_space())
    {
        config.apply_dependency_json_lockfile_tuning();
    }
    let prepared = prepare_block(state);
    let previous_huff_table = state.last_huff_table.take();
    let result = compress_prepared_block(
        output,
        config,
        prepared.as_ref(),
        &mut state.fse_tables,
        &mut state.offset_history,
        previous_huff_table.as_ref(),
    );
    state.last_huff_table = previous_huff_table;
    result
}

pub(crate) fn prepare_block<M: Matcher>(state: &mut CompressState<M>) -> PreparedBlock {
    let mut literals_vec = Vec::with_capacity(INITIAL_LITERALS_CAPACITY);
    let mut sequences = Vec::with_capacity(INITIAL_SEQUENCES_CAPACITY);
    let (newest, second, third) = state.offset_history.as_offsets();
    state.matcher.set_repeat_offsets(newest, second, third);
    state.matcher.start_matching(|seq| match seq {
        Sequence::Literals { literals } => literals_vec.extend_from_slice(literals),
        Sequence::Triple {
            literals,
            offset,
            match_len,
        } => {
            literals_vec.extend_from_slice(literals);
            sequences.push(PreparedSequence {
                ll: literals.len() as u32,
                ml: match_len as u32,
                raw_offset: offset_to_u32(offset),
                encoded_offset_value: 0,
            });
        }
    });

    PreparedBlock {
        literals: literals_vec,
        sequences,
    }
}

pub(crate) fn compress_prepared_block(
    output: &mut Vec<u8>,
    config: BlockCompressionConfig,
    prepared: PreparedBlockRef<'_>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    previous_huff_table: Option<&huff0_encoder::HuffmanTable>,
) -> Option<huff0_encoder::HuffmanTable> {
    compress_prepared_block_with_huffman_repeat(
        output,
        config,
        prepared,
        fse_tables,
        offset_history,
        previous_huff_table,
        previous_huff_table.is_some(),
    )
}

pub(crate) fn compress_prepared_block_with_huffman_repeat(
    output: &mut Vec<u8>,
    config: BlockCompressionConfig,
    prepared: PreparedBlockRef<'_>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    previous_huff_table: Option<&huff0_encoder::HuffmanTable>,
    previous_huff_table_is_valid: bool,
) -> Option<huff0_encoder::HuffmanTable> {
    compress_prepared_block_with_stats(
        output,
        config,
        prepared,
        fse_tables,
        offset_history,
        previous_huff_table,
        previous_huff_table_is_valid,
    )
    .new_huffman_table
}

pub(crate) fn compress_prepared_block_with_stats(
    output: &mut Vec<u8>,
    config: BlockCompressionConfig,
    prepared: PreparedBlockRef<'_>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    previous_huff_table: Option<&huff0_encoder::HuffmanTable>,
    previous_huff_table_is_valid: bool,
) -> CompressedBlockResult {
    let mut result = CompressedBlockResult {
        new_huffman_table: None,
        should_emit_raw_block: false,
    };
    let mut next_offset_history = *offset_history;
    let sequences = encode_sequences_for_history(prepared.sequences, &mut next_offset_history);

    // literals section

    let mut writer = BitWriter::from(output);
    if !config.literal_compression_disabled
        && should_compress_literals(
            prepared.literals.len(),
            previous_huff_table_is_valid,
            config.literal_compression_min_size,
        )
    {
        let search_smallest_huffman_table =
            config.search_smallest_huffman_table(prepared.literals.len(), sequences.len());
        if let Some(table) = compress_literals(
            prepared.literals,
            previous_huff_table,
            previous_huff_table_is_valid,
            LiteralCompressionOptions {
                search_smallest_table: search_smallest_huffman_table,
                force_single_stream_max_literals: config
                    .file_type_single_stream_huffman_max_literals,
                suspect_uncompressible: suspect_uncompressible_literals(
                    prepared.literals.len(),
                    sequences.len(),
                ),
                c_literal_cost_model: config.c_literal_cost_model,
                prefer_valid_repeat: config.prefer_valid_repeat_huffman,
            },
            &mut writer,
        ) {
            result.new_huffman_table = Some(table);
        }
    } else {
        raw_literals(prepared.literals, &mut writer);
    }

    // sequences section

    if sequences.is_empty() {
        writer.write_bits(0u8, 8);
    } else {
        encode_seqnum(sequences.len(), &mut writer);

        // Choose the tables.
        let file_type_small_sequence_predefined_llml_max_sequences =
            if prepared.literals.len() >= COMPRESS_LITERALS_SIZE_MIN {
                config
                    .file_type_small_sequence_predefined_llml_max_sequences
                    .unwrap_or(16)
            } else {
                16
            };
        let (ll_mode, ml_mode, of_mode) = choose_sequence_table_modes(
            &sequences,
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
                llml_predefined_max_sequences:
                    file_type_small_sequence_predefined_llml_max_sequences,
                of_predefined_max_sequences: config.offset_predefined_max_sequences,
                of_max_log: config.offset_table_max_log,
                exact_sequence_mode_search: config.exact_sequence_mode_search,
                c_fast_heuristics: config.c_fast_sequence_table_heuristics,
                c_cost_model: config.c_cost_sequence_table_selection,
            },
        );

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
        encode_sequences(&sequences, &mut writer, &ll_mode, &ml_mode, &of_mode);
        let bitstream_size = byte_size_between(bitstream_start, writer.index());

        if should_emit_raw_for_legacy_decoder(last_count_size, bitstream_size) {
            result.should_emit_raw_block = true;
        }

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
    }
    writer.flush();
    *offset_history = next_offset_history;
    result
}

pub(crate) fn compress_c_prepared_block_with_stats(
    output: &mut Vec<u8>,
    config: BlockCompressionConfig,
    prepared: PreparedBlockRef<'_>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    previous_huff_table: Option<&huff0_encoder::HuffmanTable>,
    previous_huff_table_is_valid: bool,
) -> CompressedBlockResult {
    debug_assert!(!config.exact_sequence_mode_search);
    let mut result = CompressedBlockResult {
        new_huffman_table: None,
        should_emit_raw_block: false,
    };
    let mut next_offset_history = *offset_history;
    if !config.c_fast_sequence_emission {
        for sequence in prepared.sequences {
            let offset_value = sequence.encoded_offset_value;
            debug_assert!(offset_value != 0);
            let raw_offset =
                next_offset_history.update_from_c_offset_value(offset_value, sequence.ll);
            debug_assert_eq!(raw_offset, sequence.raw_offset);
        }
    }

    let mut writer = BitWriter::from(output);
    if !config.literal_compression_disabled
        && should_compress_literals(
            prepared.literals.len(),
            previous_huff_table_is_valid,
            config.literal_compression_min_size,
        )
    {
        let search_smallest_huffman_table =
            config.search_smallest_huffman_table(prepared.literals.len(), prepared.sequences.len());
        if let Some(table) = compress_literals(
            prepared.literals,
            previous_huff_table,
            previous_huff_table_is_valid,
            LiteralCompressionOptions {
                search_smallest_table: search_smallest_huffman_table,
                force_single_stream_max_literals: config
                    .file_type_single_stream_huffman_max_literals,
                suspect_uncompressible: suspect_uncompressible_literals(
                    prepared.literals.len(),
                    prepared.sequences.len(),
                ),
                c_literal_cost_model: config.c_literal_cost_model,
                prefer_valid_repeat: config.prefer_valid_repeat_huffman,
            },
            &mut writer,
        ) {
            result.new_huffman_table = Some(table);
        }
    } else {
        raw_literals(prepared.literals, &mut writer);
    }

    if prepared.sequences.is_empty() {
        writer.write_bits(0u8, 8);
    } else {
        encode_seqnum(prepared.sequences.len(), &mut writer);
        let file_type_small_sequence_predefined_llml_max_sequences =
            if prepared.literals.len() >= COMPRESS_LITERALS_SIZE_MIN {
                config
                    .file_type_small_sequence_predefined_llml_max_sequences
                    .unwrap_or(16)
            } else {
                16
            };
        let sequence_mode_config = SequenceModeSearchConfig {
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
            llml_predefined_max_sequences: file_type_small_sequence_predefined_llml_max_sequences,
            of_predefined_max_sequences: config.offset_predefined_max_sequences,
            of_max_log: config.offset_table_max_log,
            exact_sequence_mode_search: false,
            c_fast_heuristics: config.c_fast_sequence_table_heuristics,
            c_cost_model: config.c_cost_sequence_table_selection,
        };
        let (ll_mode, ml_mode, of_mode) = if config.c_fast_sequence_emission {
            let (ll_mode, ml_mode, of_mode, final_offset_history) =
                if config.c_dfast_compact_sequence_statistics {
                    choose_c_dfast_compact_sequence_table_modes_from_prepared(
                        prepared.sequences,
                        sequence_mode_config,
                        *offset_history,
                        None,
                    )
                } else {
                    choose_c_fast_sequence_table_modes_from_prepared(
                        prepared.sequences,
                        sequence_mode_config,
                        *offset_history,
                        None,
                    )
                };
            next_offset_history = final_offset_history;
            (ll_mode, ml_mode, of_mode)
        } else {
            choose_c_sequence_table_modes_from_prepared(prepared.sequences, sequence_mode_config)
        };

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
        encode_prepared_sequences(
            prepared.sequences,
            &mut writer,
            &ll_mode,
            &ml_mode,
            &of_mode,
            config.c_fast_sequence_emission,
        );
        let bitstream_size = byte_size_between(bitstream_start, writer.index());
        if should_emit_raw_for_legacy_decoder(last_count_size, bitstream_size) {
            result.should_emit_raw_block = true;
        }

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
    }
    writer.flush();
    *offset_history = next_offset_history;
    result
}

/// Compress a native C `SeqStore` representation directly.
///
/// This is the Rust analogue of carrying `seqStore_t` from match collection
/// through `ZSTD_buildSequencesStatistics()` and `ZSTD_encodeSequences()`.
/// The 12-byte stored records are never expanded into the 16-byte generic
/// prepared form; only the already-gathered literal stream is shared with the
/// normal literal compressor. C-port matchers carry their already-computed
/// final repeat state through the transactional entry points below; the
/// retained replay entry points remain as exact controls and direct utilities.
#[allow(clippy::too_many_arguments)]
pub(crate) fn compress_c_stored_block_with_stats(
    output: &mut Vec<u8>,
    config: BlockCompressionConfig,
    stored: StoredBlockRef<'_>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    previous_huff_table: Option<&huff0_encoder::HuffmanTable>,
    previous_huff_table_is_valid: bool,
    huffman_scratch: Option<&mut huff0_encoder::HuffmanBuildScratch>,
    fse_build_scratch: Option<&mut FSETableBuildScratch>,
) -> CompressedBlockResult {
    compress_c_stored_block_with_stats_impl::<false>(
        output,
        config,
        stored,
        fse_tables,
        offset_history,
        *offset_history,
        previous_huff_table,
        previous_huff_table_is_valid,
        huffman_scratch,
        fse_build_scratch,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compress_c_stored_block_deferred_with_stats(
    output: &mut Vec<u8>,
    config: BlockCompressionConfig,
    stored: StoredBlockRef<'_>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    previous_huff_table: Option<&huff0_encoder::HuffmanTable>,
    previous_huff_table_is_valid: bool,
    huffman_scratch: Option<&mut huff0_encoder::HuffmanBuildScratch>,
    fse_build_scratch: Option<&mut FSETableBuildScratch>,
    pending_state: &mut PendingStoredEntropyState,
) -> CompressedBlockResult {
    compress_c_stored_block_with_stats_impl::<false>(
        output,
        config,
        stored,
        fse_tables,
        offset_history,
        *offset_history,
        previous_huff_table,
        previous_huff_table_is_valid,
        huffman_scratch,
        fse_build_scratch,
        Some(pending_state),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compress_c_stored_block_with_matcher_history(
    output: &mut Vec<u8>,
    config: BlockCompressionConfig,
    stored: StoredBlockRef<'_>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    matcher_final_offset_history: OffsetHistory,
    previous_huff_table: Option<&huff0_encoder::HuffmanTable>,
    previous_huff_table_is_valid: bool,
    huffman_scratch: Option<&mut huff0_encoder::HuffmanBuildScratch>,
    fse_build_scratch: Option<&mut FSETableBuildScratch>,
) -> CompressedBlockResult {
    compress_c_stored_block_with_stats_impl::<true>(
        output,
        config,
        stored,
        fse_tables,
        offset_history,
        matcher_final_offset_history,
        previous_huff_table,
        previous_huff_table_is_valid,
        huffman_scratch,
        fse_build_scratch,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compress_c_stored_block_deferred_with_matcher_history(
    output: &mut Vec<u8>,
    config: BlockCompressionConfig,
    stored: StoredBlockRef<'_>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    matcher_final_offset_history: OffsetHistory,
    previous_huff_table: Option<&huff0_encoder::HuffmanTable>,
    previous_huff_table_is_valid: bool,
    huffman_scratch: Option<&mut huff0_encoder::HuffmanBuildScratch>,
    fse_build_scratch: Option<&mut FSETableBuildScratch>,
    pending_state: &mut PendingStoredEntropyState,
) -> CompressedBlockResult {
    compress_c_stored_block_with_stats_impl::<true>(
        output,
        config,
        stored,
        fse_tables,
        offset_history,
        matcher_final_offset_history,
        previous_huff_table,
        previous_huff_table_is_valid,
        huffman_scratch,
        fse_build_scratch,
        Some(pending_state),
    )
}

#[allow(clippy::too_many_arguments)]
fn compress_c_stored_block_with_stats_impl<const MATCHER_HISTORY: bool>(
    output: &mut Vec<u8>,
    config: BlockCompressionConfig,
    stored: StoredBlockRef<'_>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    matcher_final_offset_history: OffsetHistory,
    previous_huff_table: Option<&huff0_encoder::HuffmanTable>,
    previous_huff_table_is_valid: bool,
    huffman_scratch: Option<&mut huff0_encoder::HuffmanBuildScratch>,
    mut fse_build_scratch: Option<&mut FSETableBuildScratch>,
    mut pending_state: Option<&mut PendingStoredEntropyState>,
) -> CompressedBlockResult {
    debug_assert!(!config.exact_sequence_mode_search);
    if fse_build_scratch.is_some()
        && !fse_build_scratch
            .as_deref()
            .is_some_and(FSETableBuildScratch::has_shared_pool)
        && !crate::fse::fse_encoder::reuses_fast_fse_build_scratch()
    {
        fse_build_scratch = None;
    }
    let mut result = CompressedBlockResult {
        new_huffman_table: None,
        should_emit_raw_block: false,
    };
    let mut next_offset_history = if MATCHER_HISTORY {
        matcher_final_offset_history
    } else {
        *offset_history
    };
    if !MATCHER_HISTORY && !config.c_fast_sequence_emission {
        for sequence in stored.sequences {
            next_offset_history
                .update_from_c_offset_value(sequence.off_base_value(), sequence.lit_len);
        }
    }

    let mut writer = BitWriter::from(output);
    if !config.literal_compression_disabled
        && should_compress_literals(
            stored.literals.len(),
            previous_huff_table_is_valid,
            config.literal_compression_min_size,
        )
    {
        let search_smallest_huffman_table =
            config.search_smallest_huffman_table(stored.literals.len(), stored.sequences.len());
        if let Some(table) = compress_literals_with_scratch(
            stored.literals,
            previous_huff_table,
            previous_huff_table_is_valid,
            LiteralCompressionOptions {
                search_smallest_table: search_smallest_huffman_table,
                force_single_stream_max_literals: config
                    .file_type_single_stream_huffman_max_literals,
                suspect_uncompressible: suspect_uncompressible_literals(
                    stored.literals.len(),
                    stored.sequences.len(),
                ),
                c_literal_cost_model: config.c_literal_cost_model,
                prefer_valid_repeat: config.prefer_valid_repeat_huffman,
            },
            huffman_scratch,
            fse_build_scratch.as_deref_mut(),
            &mut writer,
        ) {
            result.new_huffman_table = Some(table);
        }
    } else {
        raw_literals(stored.literals, &mut writer);
    }

    if stored.sequences.is_empty() {
        writer.write_bits(0u8, 8);
    } else {
        encode_seqnum(stored.sequences.len(), &mut writer);
        let file_type_small_sequence_predefined_llml_max_sequences =
            if stored.literals.len() >= COMPRESS_LITERALS_SIZE_MIN {
                config
                    .file_type_small_sequence_predefined_llml_max_sequences
                    .unwrap_or(16)
            } else {
                16
            };
        let sequence_mode_config = SequenceModeSearchConfig {
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
            llml_predefined_max_sequences: file_type_small_sequence_predefined_llml_max_sequences,
            of_predefined_max_sequences: config.offset_predefined_max_sequences,
            of_max_log: config.offset_table_max_log,
            exact_sequence_mode_search: false,
            c_fast_heuristics: config.c_fast_sequence_table_heuristics,
            c_cost_model: config.c_cost_sequence_table_selection,
        };
        let (ll_mode, ml_mode, of_mode) = if config.c_fast_sequence_emission {
            let (ll_mode, ml_mode, of_mode, final_offset_history) =
                if MATCHER_HISTORY && config.c_dfast_compact_sequence_statistics {
                    choose_c_dfast_compact_sequence_table_modes_from_stored_with_final_history(
                        stored.sequences,
                        sequence_mode_config,
                        next_offset_history,
                        fse_build_scratch,
                    )
                } else if MATCHER_HISTORY {
                    choose_c_fast_sequence_table_modes_from_stored_with_final_history(
                        stored.sequences,
                        sequence_mode_config,
                        next_offset_history,
                        fse_build_scratch,
                    )
                } else if config.c_dfast_compact_sequence_statistics {
                    choose_c_dfast_compact_sequence_table_modes_from_stored(
                        stored.sequences,
                        sequence_mode_config,
                        *offset_history,
                        fse_build_scratch,
                    )
                } else {
                    choose_c_fast_sequence_table_modes_from_stored(
                        stored.sequences,
                        sequence_mode_config,
                        *offset_history,
                        fse_build_scratch,
                    )
                };
            next_offset_history = final_offset_history;
            (ll_mode, ml_mode, of_mode)
        } else if let Some(scratch) = fse_build_scratch {
            choose_c_sequence_table_modes_from_stored_with_scratch(
                stored.sequences,
                sequence_mode_config,
                scratch,
            )
        } else {
            choose_c_sequence_table_modes_from_stored(stored.sequences, sequence_mode_config)
        };

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
        encode_stored_sequences(
            stored.sequences,
            &mut writer,
            &ll_mode,
            &ml_mode,
            &of_mode,
            config.c_fast_sequence_emission,
        );
        let bitstream_size = byte_size_between(bitstream_start, writer.index());
        if should_emit_raw_for_legacy_decoder(last_count_size, bitstream_size) {
            result.should_emit_raw_block = true;
        }

        let ll_update = fse_table_update(ll_mode);
        let ml_update = fse_table_update(ml_mode);
        let of_update = fse_table_update(of_mode);
        if let Some(pending_state) = pending_state.as_mut() {
            pending_state.fse_updates = Some(PendingFseTableUpdates {
                ll: ll_update,
                ml: ml_update,
                of: of_update,
            });
        } else {
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
        }
    }
    writer.flush();
    if let Some(pending_state) = pending_state {
        pending_state.offset_history = Some(next_offset_history);
    } else {
        *offset_history = next_offset_history;
    }
    result
}
