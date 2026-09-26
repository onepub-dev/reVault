//! C-style compressed block size estimates used by post-sequence block splitting.
//!
//! These helpers mirror `ZSTD_buildEntropyStatisticsAndEstimateSubBlockSize()`:
//! they build the candidate entropy tables, estimate literal and sequence stream
//! costs, and intentionally avoid writing the actual block payload.

use alloc::vec::Vec;
use core::fmt;

use crate::{
    bit_io::BitWriter,
    blocks::sequence_section::Sequence,
    encoding::frame_compressor::{FseTables, OffsetHistory},
    fse::fse_encoder::{FSETable, FSETableBuildScratch},
    huff0::huff0_encoder,
    workspace::{Arena, ArenaError, ArenaSize, ReusableVec},
};

use super::{
    literals::{
        compressed_literals_header_len, compressed_literals_size_format,
        suspect_uncompressible_literals, LiteralStats,
    },
    sequence_bitstream::encode_sequences_for_estimate_into,
    sequence_codes::{encode_literal_length, encode_match_len, encode_offset},
    sequence_cost::{cross_entropy_cost, repeat_table_cost, CodeCounts},
    sequence_tables::{
        choose_sequence_table_modes_for_estimate,
        choose_sequence_table_modes_for_estimate_from_counts_with_scratch, encode_table,
        FseTableMode, SequenceModeSearchConfig,
    },
    BlockCompressionConfig, PreparedBlockRef,
};

const ZSTD_BLOCK_HEADER_SIZE: usize = 3;

pub(crate) struct EstimateScratch {
    sequences: ReusableVec<Sequence>,
    table_bytes: ReusableVec<u8>,
    huffman: huff0_encoder::HuffmanBuildScratch,
    fse: FSETableBuildScratch,
}

impl fmt::Debug for EstimateScratch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EstimateScratch")
            .field("sequences_len", &self.sequences.len())
            .field("table_bytes_len", &self.table_bytes.len())
            .finish()
    }
}

impl EstimateScratch {
    pub(crate) fn new() -> Self {
        Self {
            sequences: ReusableVec::new(),
            table_bytes: ReusableVec::new(),
            huffman: huff0_encoder::HuffmanBuildScratch::new(),
            fse: FSETableBuildScratch::new(),
        }
    }

    pub(crate) fn add_workspace_size(
        size: &mut ArenaSize,
        max_sequences: usize,
    ) -> Result<(), ArenaError> {
        size.add::<Sequence>(max_sequences)?;
        size.add::<u8>(1024)?;
        huff0_encoder::HuffmanBuildScratch::add_workspace_size(size)?;
        FSETableBuildScratch::add_workspace_size(size)
    }

    pub(crate) fn new_in(arena: &mut Arena<'_>, max_sequences: usize) -> Result<Self, ArenaError> {
        Ok(Self {
            sequences: arena.allocate_reusable_vec(max_sequences)?,
            table_bytes: arena.allocate_reusable_vec(1024)?,
            huffman: huff0_encoder::HuffmanBuildScratch::new_in(arena)?,
            fse: FSETableBuildScratch::new_in(arena)?,
        })
    }
}

#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_est")]
#[cfg_attr(target_family = "windows", link_section = ".text$041.rz.est")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.041.ruzstd.block.estimate"
)]
pub(crate) fn estimate_prepared_block_size_with_sequences(
    config: BlockCompressionConfig,
    prepared: PreparedBlockRef<'_>,
    fse_tables: &FseTables,
    offset_history: OffsetHistory,
    previous_huff_table: Option<&huff0_encoder::HuffmanTable>,
    previous_huff_table_is_valid: bool,
    scratch: &mut EstimateScratch,
) -> usize {
    encode_sequences_for_estimate_into(prepared.sequences, offset_history, &mut scratch.sequences);
    let literal_size = estimate_literal_section_size(
        prepared.literals,
        previous_huff_table,
        previous_huff_table_is_valid,
        config,
        scratch.sequences.len(),
        &mut scratch.huffman,
        &mut scratch.fse,
    );
    let sequence_size = estimate_sequence_section_size(
        scratch.sequences.as_slice(),
        prepared.literals.len(),
        config,
        fse_tables,
        &mut scratch.table_bytes,
        &mut scratch.fse,
    );
    ZSTD_BLOCK_HEADER_SIZE + literal_size + sequence_size
}

fn estimate_literal_section_size(
    literals: &[u8],
    previous_table: Option<&huff0_encoder::HuffmanTable>,
    previous_table_is_valid: bool,
    config: BlockCompressionConfig,
    sequence_count: usize,
    huffman_scratch: &mut huff0_encoder::HuffmanBuildScratch,
    fse_scratch: &mut FSETableBuildScratch,
) -> usize {
    if config.literal_compression_disabled
        || literals.len() <= estimate_min_literals_to_compress(previous_table_is_valid)
    {
        return literals.len();
    }

    let force_single_stream = config
        .file_type_single_stream_huffman_max_literals
        .is_some_and(|max_literals| literals.len() <= max_literals);
    let (size_format, _) = compressed_literals_size_format(literals.len(), force_single_stream);
    let four_streams = size_format != 0;
    let search_smallest_table =
        config.search_smallest_huffman_table(literals.len(), sequence_count);
    let (literal_stats, stream_counts) = LiteralStats::from_literals_with_stream_counts(
        literals,
        four_streams && search_smallest_table,
    );

    if literal_stats.largest() == literals.len() {
        return usize::from(!literals.is_empty());
    }
    if literal_stats.likely_incompressible(literals.len())
        || suspect_uncompressible_literals(literals.len(), sequence_count)
            && sampled_literals_likely_incompressible(literals)
    {
        return literals.len();
    }

    let new_table = if search_smallest_table {
        huff0_encoder::HuffmanTable::build_smallest_from_counts_with_stream_counts(
            literal_stats.counts(),
            stream_counts.as_ref(),
        )
    } else if config.c_literal_cost_model {
        let table_log = huff0_encoder::HuffmanTable::c_fast_table_log(
            literals.len(),
            literal_stats.counts().len() - 1,
        );
        huff0_encoder::HuffmanTable::build_from_counts_with_max_bits_and_workspaces(
            literal_stats.counts(),
            table_log,
            huffman_scratch,
            fse_scratch,
        )
    } else {
        huff0_encoder::HuffmanTable::build_from_counts_with_max_bits_and_workspaces(
            literal_stats.counts(),
            11,
            huffman_scratch,
            fse_scratch,
        )
    };
    let new_content_size =
        estimate_huffman_content_size(&new_table, literal_stats.counts(), four_streams);
    let new_table_size = new_table.table_description_len();

    let estimated_size = if let Some(previous_table) = previous_table {
        if previous_table.can_encode_counts(literal_stats.counts()) {
            let old_content_size =
                estimate_huffman_content_size(previous_table, literal_stats.counts(), four_streams);
            if old_content_size < literals.len()
                && (old_content_size <= new_table_size + new_content_size
                    || new_table_size + 12 >= literals.len())
            {
                old_content_size + compressed_literals_header_len(size_format)
            } else if new_content_size + new_table_size >= literals.len() {
                literals.len()
            } else {
                new_content_size + new_table_size + compressed_literals_header_len(size_format)
            }
        } else if new_content_size + new_table_size >= literals.len() {
            literals.len()
        } else {
            new_content_size + new_table_size + compressed_literals_header_len(size_format)
        }
    } else if new_content_size + new_table_size >= literals.len() {
        literals.len()
    } else {
        new_content_size + new_table_size + compressed_literals_header_len(size_format)
    };
    huffman_scratch.recycle_table(new_table);
    estimated_size
}

fn estimate_min_literals_to_compress(has_previous_table: bool) -> usize {
    if has_previous_table {
        6
    } else {
        63
    }
}

fn estimate_huffman_content_size(
    table: &huff0_encoder::HuffmanTable,
    counts: &[usize],
    four_streams: bool,
) -> usize {
    // Match `HUF_estimateCompressedSize()`: estimate from the full histogram
    // without per-stream padding, then add the 4-stream jump table size.
    table.estimated_compressed_size_from_counts(counts) + if four_streams { 6 } else { 0 }
}

fn sampled_literals_likely_incompressible(literals: &[u8]) -> bool {
    const SAMPLE_SIZE: usize = 4096;
    const SAMPLE_RATIO: usize = 10;

    if literals.len() < SAMPLE_SIZE * SAMPLE_RATIO {
        return false;
    }

    let begin = &literals[..SAMPLE_SIZE];
    let end = &literals[literals.len() - SAMPLE_SIZE..];
    let largest_total = largest_symbol_count(begin) + largest_symbol_count(end);
    largest_total <= ((2 * SAMPLE_SIZE) >> 7) + 4
}

fn largest_symbol_count(literals: &[u8]) -> usize {
    let mut counts = [0usize; 256];
    let mut largest = 0usize;
    for &symbol in literals {
        counts[usize::from(symbol)] += 1;
        largest = largest.max(counts[usize::from(symbol)]);
    }
    largest
}

#[allow(clippy::too_many_arguments)]
fn estimate_sequence_section_size(
    sequences: &[Sequence],
    literal_len: usize,
    config: BlockCompressionConfig,
    fse_tables: &FseTables,
    table_bytes: &mut Vec<u8>,
    fse_scratch: &mut FSETableBuildScratch,
) -> usize {
    if sequences.is_empty() {
        return 2;
    }

    let llml_predefined_max_sequences = if literal_len >= super::COMPRESS_LITERALS_SIZE_MIN {
        config
            .file_type_small_sequence_predefined_llml_max_sequences
            .unwrap_or(16)
    } else {
        16
    };
    let (of_estimate, ll_estimate, ml_estimate) = sequence_symbol_estimates(sequences);
    let search_config = SequenceModeSearchConfig {
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
        exact_sequence_mode_search: config.exact_sequence_mode_search,
        c_fast_heuristics: config.c_fast_sequence_table_heuristics,
        c_cost_model: config.c_cost_sequence_table_selection,
    };
    let (ll_mode, ml_mode, of_mode) = if config.exact_sequence_mode_search {
        choose_sequence_table_modes_for_estimate(sequences, search_config)
    } else {
        choose_sequence_table_modes_for_estimate_from_counts_with_scratch(
            sequences.len(),
            &ll_estimate.counts,
            ll_estimate.last_code,
            &ml_estimate.counts,
            ml_estimate.last_code,
            &of_estimate.counts,
            of_estimate.last_code,
            search_config,
            fse_scratch,
        )
    };

    let estimated_size = sequence_header_size(sequences.len())
        + 1
        + table_definition_size(&ll_mode, table_bytes)
        + table_definition_size(&of_mode, table_bytes)
        + table_definition_size(&ml_mode, table_bytes)
        + estimate_symbol_stream_size(&of_mode, &fse_tables.of_default, &of_estimate)
        + estimate_symbol_stream_size(&ll_mode, &fse_tables.ll_default, &ll_estimate)
        + estimate_symbol_stream_size(&ml_mode, &fse_tables.ml_default, &ml_estimate);

    for mode in [ll_mode, ml_mode, of_mode] {
        if let FseTableMode::Encoded(table) = mode {
            fse_scratch.recycle_table(table);
        }
    }
    estimated_size
}

fn sequence_header_size(seqnum: usize) -> usize {
    match seqnum {
        1..=127 => 1,
        128..=0x7fff => 2,
        0x8000..=0x17eff => 3,
        _ => unreachable!("sequence count must fit zstd block limits"),
    }
}

fn table_definition_size(mode: &FseTableMode<'_>, bytes: &mut Vec<u8>) -> usize {
    match mode {
        FseTableMode::Predefined(_) | FseTableMode::RepeatLast(_) => return 0,
        FseTableMode::Rle(_) => return 1,
        FseTableMode::Encoded(_) => {}
    }

    bytes.clear();
    let mut writer = BitWriter::from(&mut *bytes);
    encode_table(mode, &mut writer);
    writer.flush();
    bytes.len()
}

fn sequence_symbol_estimates(
    sequences: &[Sequence],
) -> (SymbolEstimate, SymbolEstimate, SymbolEstimate) {
    let mut of_estimate = SymbolEstimate::new();
    let mut ll_estimate = SymbolEstimate::new();
    let mut ml_estimate = SymbolEstimate::new();

    for sequence in sequences {
        let (of_code, _, of_bits) = encode_offset(sequence.of);
        of_estimate.add_code(of_code, of_bits);

        let (ll_code, _, ll_bits) = encode_literal_length(sequence.ll);
        ll_estimate.add_code(ll_code, ll_bits);

        let (ml_code, _, ml_bits) = encode_match_len(sequence.ml);
        ml_estimate.add_code(ml_code, ml_bits);
    }

    (of_estimate, ll_estimate, ml_estimate)
}

struct SymbolEstimate {
    counts: CodeCounts,
    extra_bits: usize,
    last_code: u8,
}

impl SymbolEstimate {
    fn new() -> Self {
        Self {
            counts: CodeCounts::new(),
            extra_bits: 0,
            last_code: 0,
        }
    }

    fn add_code(&mut self, code: u8, extra_bits: usize) {
        self.counts.add_code(code);
        self.extra_bits += extra_bits;
        self.last_code = code;
    }
}

fn estimate_symbol_stream_size(
    mode: &FseTableMode<'_>,
    default_table: &FSETable,
    estimate: &SymbolEstimate,
) -> usize {
    let symbol_bits = match mode {
        FseTableMode::Predefined(_) => cross_entropy_cost(default_table, &estimate.counts),
        FseTableMode::Rle(_) => Some(0),
        FseTableMode::Encoded(table) => repeat_table_cost(table, &estimate.counts),
        FseTableMode::RepeatLast(table) => repeat_table_cost(table, &estimate.counts),
    }
    .unwrap_or(estimate.counts.total() * 10);
    (symbol_bits + estimate.extra_bits) >> 3
}
