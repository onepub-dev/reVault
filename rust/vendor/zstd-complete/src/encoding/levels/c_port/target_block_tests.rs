use alloc::{vec, vec::Vec};

use super::{
    greedy_block::{GreedyBlockEncodeContext, GreedyEncodedBlock, GreedyPreparedBlock},
    params::Strategy,
    sequence_store::RepeatOffsets,
    target_acceptance::{accept_target_or_raw_fallback, TargetAcceptanceContext},
    target_block::{encode_target_block_with_superblock_fallback, TargetBlockOptions},
    target_block_fixtures::{
        assert_repeats_rle_sequence_metadata_after_first_sub_block, decode_blocks,
        decode_compressed_block, high_entropy_literal_sequence_block,
        huffman_friendly_sequence_block, parse_block_header, parse_block_headers,
        raw_tail_sequence_block, split_friendly_sequence_block, zero_literal_tail_sequence_block,
    },
};
use crate::{
    blocks::{
        block::BlockType,
        literals_section::{LiteralsSection, LiteralsSectionType},
    },
    encoding::{
        blocks::{PreparedBlock, PreparedSequence},
        frame_compressor::{FseTables, OffsetHistory},
    },
    fse::fse_encoder::SharedFSETable,
    huff0::huff0_encoder::HuffmanTable,
};

#[test]
fn target_block_uses_huffman_literals_for_sequence_block() {
    let (data, prepared) = huffman_friendly_sequence_block();
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_target_block_with_superblock_fallback(
        &data,
        true,
        TargetBlockOptions {
            target_c_block_size: 2048,
            strategy: Strategy::BtOpt,
            allow_rle: false,
            repeat_offsets: RepeatOffsets::new(),
        },
        &prepared,
        GreedyBlockEncodeContext {
            previous_huff_table: None,
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
        Vec::new(),
    );
    let (last_block, block_type, block_size) = parse_block_header(&encoded.bytes);

    assert!(last_block);
    assert_eq!(block_type, BlockType::Compressed);
    assert_eq!(block_size as usize, encoded.bytes.len() - 3);
    assert_eq!(decode_compressed_block(&encoded.bytes), data);
    assert!(encoded.new_huffman_table.is_some());
}

#[test]
fn fastish_target_block_prefers_valid_previous_huffman_table() {
    let (data, prepared) = huffman_friendly_sequence_block();
    let previous_huff_table =
        HuffmanTable::build_from_counts(&literal_counts(&prepared.prepared.literals));
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_target_block_with_superblock_fallback(
        &data,
        true,
        TargetBlockOptions {
            target_c_block_size: 2048,
            strategy: Strategy::DFast,
            allow_rle: false,
            repeat_offsets: RepeatOffsets::new(),
        },
        &prepared,
        GreedyBlockEncodeContext {
            previous_huff_table: Some(&previous_huff_table),
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
        Vec::new(),
    );
    let (_, block_type, block_size) = parse_block_header(&encoded.bytes);
    assert_eq!(block_type, BlockType::Compressed);
    assert_eq!(block_size as usize, encoded.bytes.len() - 3);

    let mut literals = LiteralsSection::new();
    literals
        .parse_from_header(&encoded.bytes[3..])
        .expect("literal section should parse");
    assert!(matches!(literals.ls_type, LiteralsSectionType::Treeless));
    assert!(encoded.new_huffman_table.is_none());
}

#[test]
fn target_block_splits_huffman_literal_sequence_block_by_target_size() {
    let (data, prepared) = split_friendly_sequence_block();
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_target_block_with_superblock_fallback(
        &data,
        true,
        TargetBlockOptions {
            target_c_block_size: 1340,
            strategy: Strategy::BtOpt,
            allow_rle: false,
            repeat_offsets: RepeatOffsets::new(),
        },
        &prepared,
        GreedyBlockEncodeContext {
            previous_huff_table: None,
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
        Vec::new(),
    );
    let headers = parse_block_headers(&encoded.bytes);

    assert!(headers.len() > 1);
    assert!(headers
        .iter()
        .all(|header| header.0 == BlockType::Compressed));
    assert!(headers.last().is_some_and(|header| header.2));
    assert_eq!(decode_blocks(&encoded.bytes), data);
    assert!(encoded.new_huffman_table.is_some());
}

fn literal_counts(literals: &[u8]) -> [usize; 256] {
    let mut counts = [0; 256];
    for &literal in literals {
        counts[usize::from(literal)] += 1;
    }
    counts
}

#[test]
fn target_block_emits_zero_literal_raw_sections_inside_huffman_superblock() {
    let (data, prepared) = zero_literal_tail_sequence_block();
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_target_block_with_superblock_fallback(
        &data,
        true,
        TargetBlockOptions {
            target_c_block_size: 1340,
            strategy: Strategy::BtOpt,
            allow_rle: false,
            repeat_offsets: RepeatOffsets::new(),
        },
        &prepared,
        GreedyBlockEncodeContext {
            previous_huff_table: None,
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
        Vec::new(),
    );
    let headers = parse_block_headers(&encoded.bytes);

    assert!(
        headers.len() > 1,
        "expected multiple target sub-blocks, got {:?}",
        headers,
    );
    assert!(headers
        .iter()
        .all(|header| header.0 == BlockType::Compressed));
    assert!(headers.last().is_some_and(|header| header.2));
    assert!(has_zero_literal_raw_section_after_first_block(
        &encoded.bytes
    ));
}

#[test]
fn target_block_splits_basic_literal_sequence_block_by_target_size() {
    let (data, prepared) = high_entropy_literal_sequence_block();
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_target_block_with_superblock_fallback(
        &data,
        true,
        TargetBlockOptions {
            target_c_block_size: 1340,
            strategy: Strategy::BtOpt,
            allow_rle: false,
            repeat_offsets: RepeatOffsets::new(),
        },
        &prepared,
        GreedyBlockEncodeContext {
            previous_huff_table: None,
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
        Vec::new(),
    );
    let headers = parse_block_headers(&encoded.bytes);

    assert!(headers.len() > 1);
    assert!(headers
        .iter()
        .all(|header| header.0 == BlockType::Compressed));
    assert!(headers.last().is_some_and(|header| header.2));
    assert_eq!(decode_blocks(&encoded.bytes), data);
    assert!(encoded.new_huffman_table.is_none());
    assert_repeats_rle_sequence_metadata_after_first_sub_block(&encoded.bytes);
    assert!(fse_tables.ll_previous.is_none());
    assert!(fse_tables.ml_previous.is_none());
}

fn has_zero_literal_raw_section_after_first_block(encoded: &[u8]) -> bool {
    let mut pos = 0usize;
    let mut block_index = 0usize;
    while pos < encoded.len() {
        let (last_block, block_type, block_size) = parse_block_header(&encoded[pos..]);
        let content_start = pos + 3;
        let content_end = content_start + block_size as usize;
        if block_index > 0 && matches!(block_type, BlockType::Compressed) {
            let mut literals = LiteralsSection::new();
            if literals
                .parse_from_header(&encoded[content_start..content_end])
                .is_ok()
                && matches!(literals.ls_type, LiteralsSectionType::Raw)
                && literals.regenerated_size == 0
            {
                return true;
            }
        }
        pos = content_end;
        block_index += 1;
        if last_block {
            break;
        }
    }
    false
}

#[test]
fn target_block_emits_raw_tail_when_final_subblock_is_not_worthwhile() {
    let (data, prepared) = raw_tail_sequence_block();
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_target_block_with_superblock_fallback(
        &data,
        true,
        TargetBlockOptions {
            target_c_block_size: 1340,
            strategy: Strategy::BtOpt,
            allow_rle: false,
            repeat_offsets: RepeatOffsets::new(),
        },
        &prepared,
        GreedyBlockEncodeContext {
            previous_huff_table: None,
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
        Vec::new(),
    );
    let headers = parse_block_headers(&encoded.bytes);

    assert!(headers.len() > 1);
    assert!(headers[..headers.len() - 1]
        .iter()
        .all(|header| header.0 == BlockType::Compressed));
    assert_eq!(headers.last().map(|header| header.0), Some(BlockType::Raw));
    assert!(headers.last().is_some_and(|header| header.2));
    assert_eq!(decode_blocks(&encoded.bytes), data);
}

#[test]
fn target_block_uses_rle_shortcut_when_policy_allows_it() {
    let data = [0x5A; 64];
    let prepared = GreedyPreparedBlock {
        prepared: PreparedBlock {
            literals: vec![0x5A],
            sequences: vec![PreparedSequence {
                ll: 1,
                ml: 63,
                raw_offset: 1,
                encoded_offset_value: 0,
            }],
        },
        repeat_offsets: RepeatOffsets::new(),
    };
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_target_block_with_superblock_fallback(
        &data,
        true,
        TargetBlockOptions {
            target_c_block_size: 2048,
            strategy: Strategy::BtOpt,
            allow_rle: true,
            repeat_offsets: RepeatOffsets::new(),
        },
        &prepared,
        GreedyBlockEncodeContext {
            previous_huff_table: None,
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
        Vec::new(),
    );
    let (last_block, block_type, block_size) = parse_block_header(&encoded.bytes);

    assert!(last_block);
    assert_eq!(block_type, BlockType::RLE);
    assert_eq!(block_size as usize, data.len());
    assert_eq!(encoded.bytes, [0x03, 0x02, 0x00, 0x5A]);
    assert_eq!(encoded.repeat_offsets, RepeatOffsets::new());
    assert!(encoded.new_huffman_table.is_none());
}

#[test]
fn target_block_rle_shortcut_requires_c_maybe_rle_shape() {
    let data = [0x5A; 64];
    let prepared = GreedyPreparedBlock {
        prepared: PreparedBlock {
            literals: data.to_vec(),
            sequences: Vec::new(),
        },
        repeat_offsets: RepeatOffsets::new(),
    };
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_target_block_with_superblock_fallback(
        &data,
        true,
        TargetBlockOptions {
            target_c_block_size: 2048,
            strategy: Strategy::BtOpt,
            allow_rle: true,
            repeat_offsets: RepeatOffsets::new(),
        },
        &prepared,
        GreedyBlockEncodeContext {
            previous_huff_table: None,
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
        Vec::new(),
    );
    let (_, block_type, _) = parse_block_header(&encoded.bytes);

    assert_ne!(block_type, BlockType::RLE);
    assert_eq!(decode_blocks(&encoded.bytes), data);
}

#[test]
fn target_acceptance_rejects_candidate_without_c_minimum_gain() {
    let data = vec![0xA5; 128 * 1024];
    let mut fse_tables = FseTables::new();
    let previous_fse = fse_tables.snapshot_previous();
    fse_tables.ll_previous = Some(SharedFSETable::new(fse_tables.ll_default.clone()));
    fse_tables.ml_previous = Some(SharedFSETable::new(fse_tables.ml_default.clone()));
    fse_tables.of_previous = Some(SharedFSETable::new(fse_tables.of_default.clone()));
    let mut offset_history = OffsetHistory::new();
    let previous_offsets = offset_history;
    let encoded = GreedyEncodedBlock {
        bytes: vec![0; data.len()],
        repeat_offsets: RepeatOffsets::new(),
        new_huffman_table: None,
    };

    let fallback = accept_target_or_raw_fallback(
        encoded,
        TargetAcceptanceContext {
            block: &data,
            last_block: true,
            strategy: Strategy::Fast,
            repeat_offsets: RepeatOffsets::new(),
            initial_bytes: &[],
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
            previous_fse,
            previous_offsets,
        },
    );
    let (last_block, block_type, block_size) = parse_block_header(&fallback.bytes);

    assert!(last_block);
    assert_eq!(block_type, BlockType::Raw);
    assert_eq!(block_size as usize, data.len());
    assert_eq!(&fallback.bytes[3..], data);
    assert!(fse_tables.ll_previous.is_none());
    assert!(fse_tables.ml_previous.is_none());
    assert!(fse_tables.of_previous.is_none());
    assert_eq!(offset_history.as_offsets(), previous_offsets.as_offsets());
    assert_eq!(fallback.repeat_offsets, RepeatOffsets::new());
    assert!(fallback.new_huffman_table.is_none());
}
