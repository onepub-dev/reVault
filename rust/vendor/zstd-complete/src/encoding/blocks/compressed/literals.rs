use alloc::vec::Vec;

use crate::{bit_io::BitWriter, fse::fse_encoder::FSETableBuildScratch, huff0::huff0_encoder};

mod stats;

pub(super) use stats::LiteralStats;

pub(super) const COMPRESS_LITERALS_SIZE_MIN: usize = 64;
pub(super) const REPEAT_LITERALS_SIZE_MIN: usize = 6;
pub(super) const HUFFMAN_4_STREAMS_MIN: usize = 256;
pub(super) const REPEAT_SINGLE_STREAM_LITERALS_MAX: usize = 1024;
pub(super) const FAST_LITERAL_MIN_GAIN_LOG: u32 = 6;
const SUSPECT_INCOMPRESSIBLE_LITERAL_RATIO: usize = 20;
const SUSPECT_INCOMPRESSIBLE_SAMPLE_SIZE: usize = 4096;
const SUSPECT_INCOMPRESSIBLE_SAMPLE_RATIO: usize = 10;

pub(super) fn should_compress_literals(
    len: usize,
    previous_table_is_valid: bool,
    compression_min_size: usize,
) -> bool {
    let min_size = if previous_table_is_valid {
        REPEAT_LITERALS_SIZE_MIN
    } else {
        compression_min_size
    };
    len >= min_size
}

pub(super) fn raw_literals(literals: &[u8], writer: &mut BitWriter<&mut Vec<u8>>) {
    writer.write_bits(0u8, 2); // Raw_Literals_Block
    match literals.len() {
        0..=31 => {
            writer.write_bits(0u8, 1);
            writer.write_bits(literals.len() as u32, 5);
        }
        32..=4095 => {
            writer.write_bits(0b01u8, 2);
            writer.write_bits(literals.len() as u32, 12);
        }
        4096..=1_048_575 => {
            writer.write_bits(0b11u8, 2);
            writer.write_bits(literals.len() as u32, 20);
        }
        _ => panic!("literal section exceeds zstd raw literals size limit"),
    }
    writer.append_bytes(literals);
}

pub(super) fn rle_literals(literals: &[u8], writer: &mut BitWriter<&mut Vec<u8>>) {
    debug_assert!(!literals.is_empty());
    writer.write_bits(1u8, 2); // RLE_Literals_Block
    match literals.len() {
        0..=31 => {
            writer.write_bits(0u8, 1);
            writer.write_bits(literals.len() as u32, 5);
        }
        32..=4095 => {
            writer.write_bits(0b01u8, 2);
            writer.write_bits(literals.len() as u32, 12);
        }
        4096..=1_048_575 => {
            writer.write_bits(0b11u8, 2);
            writer.write_bits(literals.len() as u32, 20);
        }
        _ => panic!("literal section exceeds zstd RLE literals size limit"),
    }
    writer.write_bits(literals[0], 8);
}

pub(super) fn compress_literals(
    literals: &[u8],
    last_table: Option<&huff0_encoder::HuffmanTable>,
    previous_table_is_valid: bool,
    options: LiteralCompressionOptions,
    writer: &mut BitWriter<&mut Vec<u8>>,
) -> Option<huff0_encoder::HuffmanTable> {
    compress_literals_with_scratch(
        literals,
        last_table,
        previous_table_is_valid,
        options,
        None,
        None,
        writer,
    )
}

pub(super) fn compress_literals_with_scratch(
    literals: &[u8],
    last_table: Option<&huff0_encoder::HuffmanTable>,
    previous_table_is_valid: bool,
    options: LiteralCompressionOptions,
    mut huffman_scratch: Option<&mut huff0_encoder::HuffmanBuildScratch>,
    fse_build_scratch: Option<&mut FSETableBuildScratch>,
    writer: &mut BitWriter<&mut Vec<u8>>,
) -> Option<huff0_encoder::HuffmanTable> {
    if huffman_scratch.is_some()
        && !huffman_scratch
            .as_deref()
            .is_some_and(huff0_encoder::HuffmanBuildScratch::is_workspace_backed)
        && !huff0_encoder::reuses_fast_huffman_scratch()
    {
        huffman_scratch = None;
    }
    let reset_idx = writer.index();

    if options.prefer_valid_repeat
        && try_preferred_repeat_literals(
            literals,
            last_table,
            previous_table_is_valid,
            reset_idx,
            writer,
        )
    {
        return None;
    }

    if options.suspect_uncompressible && sampled_literals_likely_incompressible(literals) {
        raw_literals(literals, writer);
        return None;
    }

    let force_single_stream = options
        .force_single_stream_max_literals
        .is_some_and(|max_literals| literals.len() <= max_literals);
    let (size_format, size_bits) =
        compressed_literals_size_format(literals.len(), force_single_stream);
    let four_streams = size_format != 0;
    // C's size model uses the combined histogram. Per-stream counts are only
    // needed by the small-table search or the legacy exact-stream estimator.
    let collect_stream_counts =
        four_streams && (options.search_smallest_table || !options.c_literal_cost_model);
    let (literal_stats, four_stream_counts) =
        LiteralStats::from_literals_with_stream_counts(literals, collect_stream_counts);
    if literal_stats.largest() == literals.len()
        || literal_stats.likely_incompressible(literals.len())
    {
        if !literals.is_empty() && literal_stats.largest() == literals.len() {
            rle_literals(literals, writer);
        } else {
            raw_literals(literals, writer);
        }
        return None;
    }

    let header_len = compressed_literals_header_len(size_format);
    let new_encoder_table = if options.search_smallest_table {
        huff0_encoder::HuffmanTable::build_smallest_from_counts_with_stream_counts(
            literal_stats.counts(),
            four_stream_counts.as_ref(),
        )
    } else if options.c_literal_cost_model {
        let table_log = huff0_encoder::HuffmanTable::c_fast_table_log(
            literals.len(),
            literal_stats.counts().len() - 1,
        );
        match (huffman_scratch.as_deref_mut(), fse_build_scratch) {
            (Some(huffman_scratch), Some(fse_scratch)) => {
                huff0_encoder::HuffmanTable::build_from_counts_with_max_bits_and_workspaces(
                    literal_stats.counts(),
                    table_log,
                    huffman_scratch,
                    fse_scratch,
                )
            }
            (Some(scratch), None) => {
                huff0_encoder::HuffmanTable::build_from_counts_with_max_bits_and_scratch(
                    literal_stats.counts(),
                    table_log,
                    scratch,
                )
            }
            (None, _) => huff0_encoder::HuffmanTable::build_from_counts_with_max_bits(
                literal_stats.counts(),
                table_log,
            ),
        }
    } else {
        match (huffman_scratch.as_deref_mut(), fse_build_scratch) {
            (Some(huffman_scratch), Some(fse_scratch)) => {
                huff0_encoder::HuffmanTable::build_from_counts_with_max_bits_and_workspaces(
                    literal_stats.counts(),
                    11,
                    huffman_scratch,
                    fse_scratch,
                )
            }
            (Some(scratch), None) => huff0_encoder::HuffmanTable::build_from_counts_with_scratch(
                literal_stats.counts(),
                scratch,
            ),
            (None, _) => huff0_encoder::HuffmanTable::build_from_counts(literal_stats.counts()),
        }
    };
    let new_len = if options.c_literal_cost_model {
        c_estimated_huffman_len(&new_encoder_table, literal_stats.counts(), four_streams)
            + new_encoder_table.table_description_len()
    } else if let Some(four_stream_counts) = &four_stream_counts {
        new_encoder_table.encoded_len_from_stream_counts(four_stream_counts, true)
    } else {
        new_encoder_table.encoded_len_from_counts(literal_stats.counts(), true)
    };
    let new_choice = LiteralEncodingChoice {
        encoder_table: &new_encoder_table,
        new_table: true,
        estimated_len: new_len,
        size_format,
        size_bits,
        header_len,
    };
    let choice = last_table
        .and_then(|previous_table| {
            repeat_huffman_choice(
                previous_table,
                &literal_stats,
                literals,
                four_stream_counts.as_ref(),
                new_choice,
                previous_table_is_valid,
                options,
            )
        })
        .unwrap_or(new_choice);

    if if options.c_literal_cost_model {
        choice.estimated_len >= literals.len()
    } else {
        !literal_estimate_has_enough_gain(choice.estimated_len, choice.header_len, literals.len())
    } {
        raw_literals(literals, writer);
        if let Some(scratch) = huffman_scratch.as_deref_mut() {
            scratch.recycle_table(new_encoder_table);
        }
        return None;
    }

    write_compressed_literals(
        literals,
        choice.encoder_table,
        choice.new_table,
        choice.size_format,
        choice.size_bits,
        writer,
    );
    let total_len = (writer.index() - reset_idx) / 8;

    // If encoded len is bigger than the raw literals we are better off just writing the raw literals here
    if total_len >= literals.len() {
        writer.reset_to(reset_idx);
        raw_literals(literals, writer);
        if let Some(scratch) = huffman_scratch.as_deref_mut() {
            scratch.recycle_table(new_encoder_table);
        }
        None
    } else if choice.new_table {
        Some(new_encoder_table)
    } else {
        if let Some(scratch) = huffman_scratch {
            scratch.recycle_table(new_encoder_table);
        }
        None
    }
}

#[inline(never)]
fn try_preferred_repeat_literals(
    literals: &[u8],
    last_table: Option<&huff0_encoder::HuffmanTable>,
    previous_table_is_valid: bool,
    reset_idx: usize,
    writer: &mut BitWriter<&mut Vec<u8>>,
) -> bool {
    if !previous_table_is_valid || literals.len() > REPEAT_SINGLE_STREAM_LITERALS_MAX {
        return false;
    }
    if !literals.is_empty() && literals.iter().all(|literal| *literal == literals[0]) {
        rle_literals(literals, writer);
        return true;
    }
    let Some(previous_table) = last_table else {
        return false;
    };

    let (size_format, size_bits) =
        compressed_literals_repeat_size_format(literals.len(), false, true);
    write_compressed_literals(
        literals,
        previous_table,
        false,
        size_format,
        size_bits,
        writer,
    );
    let total_len = (writer.index() - reset_idx) / 8;
    if total_len >= literals.len() {
        writer.reset_to(reset_idx);
        raw_literals(literals, writer);
    }
    true
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct LiteralCompressionOptions {
    pub(super) search_smallest_table: bool,
    pub(super) force_single_stream_max_literals: Option<usize>,
    pub(super) suspect_uncompressible: bool,
    pub(super) c_literal_cost_model: bool,
    pub(super) prefer_valid_repeat: bool,
}

fn c_estimated_huffman_len(
    table: &huff0_encoder::HuffmanTable,
    counts: &[usize],
    four_streams: bool,
) -> usize {
    table.estimated_compressed_size_from_counts(counts) + if four_streams { 6 } else { 0 }
}

pub(super) fn suspect_uncompressible_literals(literal_len: usize, sequence_count: usize) -> bool {
    sequence_count == 0 || literal_len / sequence_count >= SUSPECT_INCOMPRESSIBLE_LITERAL_RATIO
}

fn sampled_literals_likely_incompressible(literals: &[u8]) -> bool {
    if literals.len() < SUSPECT_INCOMPRESSIBLE_SAMPLE_SIZE * SUSPECT_INCOMPRESSIBLE_SAMPLE_RATIO {
        return false;
    }

    let begin = &literals[..SUSPECT_INCOMPRESSIBLE_SAMPLE_SIZE];
    let end = &literals[literals.len() - SUSPECT_INCOMPRESSIBLE_SAMPLE_SIZE..];
    let largest_total = largest_symbol_count(begin) + largest_symbol_count(end);
    largest_total <= ((2 * SUSPECT_INCOMPRESSIBLE_SAMPLE_SIZE) >> 7) + 4
}

fn largest_symbol_count(literals: &[u8]) -> usize {
    let mut counts = [0usize; 256];
    let mut largest = 0usize;
    for &symbol in literals {
        let count = &mut counts[usize::from(symbol)];
        *count += 1;
        largest = largest.max(*count);
    }
    largest
}

#[derive(Clone, Copy)]
struct LiteralEncodingChoice<'table> {
    encoder_table: &'table huff0_encoder::HuffmanTable,
    new_table: bool,
    estimated_len: usize,
    size_format: u8,
    size_bits: usize,
    header_len: usize,
}

impl LiteralEncodingChoice<'_> {
    fn total_estimated_len(self) -> usize {
        self.estimated_len + self.header_len
    }
}

fn repeat_huffman_choice<'table>(
    previous_table: &'table huff0_encoder::HuffmanTable,
    literal_stats: &LiteralStats,
    literals: &[u8],
    four_stream_counts: Option<&[[usize; 256]; 4]>,
    new_choice: LiteralEncodingChoice<'_>,
    previous_table_is_valid: bool,
    options: LiteralCompressionOptions,
) -> Option<LiteralEncodingChoice<'table>> {
    if !previous_table.can_encode_counts(literal_stats.counts()) {
        return None;
    }

    let force_single_stream = options
        .force_single_stream_max_literals
        .is_some_and(|max_literals| literals.len() <= max_literals);
    let (size_format, size_bits) = compressed_literals_repeat_size_format(
        literals.len(),
        force_single_stream,
        previous_table_is_valid,
    );
    let header_len = compressed_literals_header_len(size_format);
    let four_streams = size_format != 0;
    let estimated_len = if options.c_literal_cost_model {
        c_estimated_huffman_len(previous_table, literal_stats.counts(), four_streams)
    } else if four_streams {
        if let Some(four_stream_counts) = four_stream_counts {
            previous_table.encoded_len_from_stream_counts(four_stream_counts, false)
        } else {
            previous_table.encoded_len(literals, false, true)
        }
    } else {
        previous_table.encoded_len_from_counts(literal_stats.counts(), false)
    };
    let use_previous = if options.c_literal_cost_model {
        estimated_len < literals.len()
            && (estimated_len <= new_choice.estimated_len
                || new_choice.encoder_table.table_description_len() + 12 >= literals.len())
    } else {
        estimated_len < literals.len()
            && estimated_len + header_len <= new_choice.total_estimated_len()
    };
    if use_previous {
        Some(LiteralEncodingChoice {
            encoder_table: previous_table,
            new_table: false,
            estimated_len,
            size_format,
            size_bits,
            header_len,
        })
    } else {
        None
    }
}

pub(super) fn compressed_literals_size_format(
    len: usize,
    force_single_stream: bool,
) -> (u8, usize) {
    if force_single_stream && len < HUFFMAN_4_STREAMS_MIN * 4 {
        return (0b00u8, 10);
    }

    match len {
        0..HUFFMAN_4_STREAMS_MIN => (0b00u8, 10),
        HUFFMAN_4_STREAMS_MIN..1024 => (0b01, 10),
        1024..16384 => (0b10, 14),
        16384..262144 => (0b11, 18),
        _ => panic!("literal section exceeds zstd compressed literals size limit"),
    }
}

fn compressed_literals_repeat_size_format(
    len: usize,
    force_single_stream: bool,
    previous_table_is_valid: bool,
) -> (u8, usize) {
    if (force_single_stream && len < HUFFMAN_4_STREAMS_MIN * 4)
        || (previous_table_is_valid && len < REPEAT_SINGLE_STREAM_LITERALS_MAX)
        || len < HUFFMAN_4_STREAMS_MIN
    {
        return (0b00, 10);
    }

    compressed_literals_size_format(len, false)
}

pub(super) fn compressed_literals_header_len(size_format: u8) -> usize {
    match size_format {
        0b00 | 0b01 => 3,
        0b10 => 4,
        0b11 => 5,
        _ => unreachable!(),
    }
}

pub(super) fn literal_min_gain(len: usize) -> usize {
    (len >> FAST_LITERAL_MIN_GAIN_LOG) + 2
}

pub(super) fn literal_estimate_has_enough_gain(
    estimated_len: usize,
    header_len: usize,
    literal_len: usize,
) -> bool {
    estimated_len < literal_len.saturating_sub(literal_min_gain(literal_len))
        && estimated_len + header_len < literal_len
}

pub(in crate::encoding::blocks::compressed) fn write_compressed_literals(
    literals: &[u8],
    encoder_table: &huff0_encoder::HuffmanTable,
    new_table: bool,
    size_format: u8,
    size_bits: usize,
    writer: &mut BitWriter<&mut Vec<u8>>,
) {
    if new_table {
        writer.write_bits(2u8, 2); // compressed literals type
    } else {
        writer.write_bits(3u8, 2); // treeless compressed literals type
    }

    writer.write_bits(size_format, 2);
    writer.write_bits(literals.len() as u32, size_bits);
    let size_index = writer.index();
    writer.write_bits(0u32, size_bits);
    let index_before = writer.index();
    let mut encoder = huff0_encoder::HuffmanEncoder::new(encoder_table, writer);
    if size_format == 0 {
        encoder.encode(literals, new_table)
    } else {
        encoder.encode4x(literals, new_table)
    };
    let encoded_len = (writer.index() - index_before) / 8;
    writer.change_bits(size_index, encoded_len as u64, size_bits);
}
