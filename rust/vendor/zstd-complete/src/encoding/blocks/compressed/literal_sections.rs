use alloc::vec::Vec;

use crate::{bit_io::BitWriter, huff0::huff0_encoder::HuffmanTable};

use super::literals::{
    compressed_literals_header_len, raw_literals, write_compressed_literals, LiteralStats,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HuffmanLiteralMode {
    Compressed,
    Repeat,
}

pub(crate) struct HuffmanLiteralSectionEmission {
    pub(crate) byte_size: usize,
    pub(crate) new_huffman_table: Option<HuffmanTable>,
    pub(crate) entropy_written: bool,
}

pub(crate) fn build_huffman_literal_table_with_optimal_depth(
    literals: &[u8],
    optimal_depth: bool,
) -> Option<HuffmanTable> {
    if literals.is_empty() {
        return None;
    }

    let (stats, _) = LiteralStats::from_literals_with_stream_counts(literals, false);
    if stats.largest() == literals.len() || stats.likely_incompressible(literals.len()) {
        return None;
    }
    let table = if optimal_depth {
        HuffmanTable::build_c_optimal_depth_from_counts(stats.counts())
    } else {
        let table_log = HuffmanTable::c_fast_table_log(literals.len(), stats.counts().len() - 1);
        HuffmanTable::build_from_counts_with_max_bits(stats.counts(), table_log)
    };
    let estimated_size =
        table.estimated_compressed_size_from_counts(stats.counts()) + table.table_description_len();
    if estimated_size >= literals.len() {
        return None;
    }
    Some(table)
}

pub(crate) fn append_huffman_literal_section_with_optimal_depth(
    literals: &[u8],
    previous_table: Option<&HuffmanTable>,
    mode: HuffmanLiteralMode,
    optimal_depth: bool,
    output: &mut Vec<u8>,
) -> Option<HuffmanLiteralSectionEmission> {
    if literals.is_empty() {
        return Some(append_raw_literal_section(literals, output));
    }

    let table = match mode {
        HuffmanLiteralMode::Compressed => {
            build_huffman_literal_table_with_optimal_depth(literals, optimal_depth)?
        }
        HuffmanLiteralMode::Repeat => previous_table
            .filter(|table| {
                let (stats, _) = LiteralStats::from_literals_with_stream_counts(literals, false);
                table.can_encode_counts(stats.counts())
            })?
            .clone(),
    };

    append_huffman_literal_section_with_table(literals, &table, mode, output)
}

pub(crate) fn append_huffman_literal_section_with_table(
    literals: &[u8],
    table: &HuffmanTable,
    mode: HuffmanLiteralMode,
    output: &mut Vec<u8>,
) -> Option<HuffmanLiteralSectionEmission> {
    if literals.is_empty() {
        return Some(append_raw_literal_section(literals, output));
    }
    if matches!(mode, HuffmanLiteralMode::Repeat) {
        let (stats, _) = LiteralStats::from_literals_with_stream_counts(literals, false);
        if !table.can_encode_counts(stats.counts()) {
            return None;
        }
    }

    let start = output.len();
    let mut writer = BitWriter::from(output);
    let (size_format, size_bits) = sub_block_huffman_size_format(
        literals.len(),
        matches!(mode, HuffmanLiteralMode::Compressed),
    );

    write_compressed_literals(
        literals,
        table,
        matches!(mode, HuffmanLiteralMode::Compressed),
        size_format,
        size_bits,
        &mut writer,
    );
    writer.flush();

    let byte_size = writer.index() / 8 - start;
    let header_size = compressed_literals_header_len(size_format);
    let compressed_size = byte_size.checked_sub(header_size)?;
    if header_size < compressed_literal_header_size(compressed_size)
        || (matches!(mode, HuffmanLiteralMode::Repeat) && compressed_size >= literals.len())
    {
        writer.reset_to(start * 8);
        raw_literals(literals, &mut writer);
        writer.flush();
        return Some(HuffmanLiteralSectionEmission {
            byte_size: writer.index() / 8 - start,
            new_huffman_table: None,
            entropy_written: false,
        });
    }

    Some(HuffmanLiteralSectionEmission {
        byte_size,
        new_huffman_table: matches!(mode, HuffmanLiteralMode::Compressed).then_some(table.clone()),
        entropy_written: true,
    })
}

fn append_raw_literal_section(
    literals: &[u8],
    output: &mut Vec<u8>,
) -> HuffmanLiteralSectionEmission {
    let start = output.len();
    let mut writer = BitWriter::from(output);
    raw_literals(literals, &mut writer);
    writer.flush();
    HuffmanLiteralSectionEmission {
        byte_size: writer.index() / 8 - start,
        new_huffman_table: None,
        entropy_written: false,
    }
}

pub(crate) fn estimate_huffman_literal_section_with_table(
    literals: &[u8],
    table: &HuffmanTable,
    write_entropy: bool,
) -> Option<usize> {
    if literals.is_empty() {
        return None;
    }

    let (stats, _) = LiteralStats::from_literals_with_stream_counts(literals, false);
    if !table.can_encode_counts(stats.counts()) {
        return None;
    }

    let table_size = if write_entropy {
        table.table_description_len()
    } else {
        0
    };
    // C's superblock planner uses HUF_estimateCompressedSize() plus a
    // hard-coded 3-byte literal header. It does not include the emitted
    // 4-stream jump table in this rough estimate.
    Some(
        table.estimated_compressed_size_from_counts(stats.counts())
            + table_size
            + compressed_literal_header_size_for_estimate(),
    )
}

fn sub_block_huffman_size_format(literal_size: usize, write_entropy: bool) -> (u8, usize) {
    let entropy_guess = if write_entropy { 200 } else { 0 };
    if literal_size >= 16 * 1024usize - entropy_guess {
        (0b11, 18)
    } else if literal_size >= 1024usize - entropy_guess {
        (0b10, 14)
    } else {
        (0b00, 10)
    }
}

fn compressed_literal_header_size_for_estimate() -> usize {
    3
}

fn compressed_literal_header_size(compressed_size: usize) -> usize {
    3 + usize::from(compressed_size >= 1024) + usize::from(compressed_size >= 16 * 1024)
}
