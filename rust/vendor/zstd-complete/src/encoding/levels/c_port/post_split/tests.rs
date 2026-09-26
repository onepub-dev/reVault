use alloc::{vec, vec::Vec};

use super::*;
use crate::blocks::block::BlockType;

#[test]
fn prepared_chunk_splits_literals_and_source_span() {
    let prepared = PreparedBlock {
        literals: b"aabbtail".to_vec(),
        sequences: vec![
            PreparedSequence {
                ll: 2,
                ml: 3,
                raw_offset: 4,
                encoded_offset_value: 0,
            },
            PreparedSequence {
                ll: 2,
                ml: 5,
                raw_offset: 7,
                encoded_offset_value: 0,
            },
        ],
    };
    let block = b"aa123bb45678tail";
    let prefixes = sequence_prefixes(&prepared);

    let first = prepared_chunk(block, &prepared, &prefixes, 0, 1);
    assert_eq!(first.source, b"aa123");
    assert_eq!(first.literals, b"aa");
    assert_eq!(first.sequences.len(), 1);

    let second = prepared_chunk(block, &prepared, &prefixes, 1, 2);
    assert_eq!(second.source, b"bb45678tail");
    assert_eq!(second.literals, b"bbtail");
    assert_eq!(second.sequences.len(), 1);
}

#[test]
fn partition_offcode_resolution_rewrites_repcodes_after_raw_partition_like_c() {
    let first_partition = PreparedBlock {
        literals: b"a".to_vec(),
        sequences: vec![PreparedSequence {
            ll: 1,
            ml: 3,
            raw_offset: 4,
            encoded_offset_value: OffBase::offset_to_c_value(4),
        }],
    };
    let second_partition = PreparedBlock {
        literals: b"b".to_vec(),
        sequences: vec![PreparedSequence {
            ll: 1,
            ml: 3,
            raw_offset: 4,
            encoded_offset_value: 1,
        }],
    };
    let initial_repeats = RepeatOffsets::new();
    let mut compression_repeats = initial_repeats;
    let mut decompression_repeats = initial_repeats;

    let _ = resolved_partition_sequences(
        &first_partition.sequences,
        &mut decompression_repeats,
        &mut compression_repeats,
    );
    decompression_repeats = initial_repeats;

    let second_sequences = resolved_partition_sequences(
        &second_partition.sequences,
        &mut decompression_repeats,
        &mut compression_repeats,
    );

    assert_eq!(
        second_sequences[0].off_base_value(),
        OffBase::offset_to_c_value(4)
    );
    assert_eq!(second_sequences[0].off_base(), OffBase::Offset(4));
    assert_eq!(decompression_repeats.as_offsets(), [4, 1, 4]);
    assert_eq!(compression_repeats.as_offsets(), [4, 1, 4]);
}

#[test]
fn partition_offcode_resolution_borrows_when_no_rewrite_is_needed() {
    let partition = PreparedBlock {
        literals: b"a".to_vec(),
        sequences: vec![PreparedSequence {
            ll: 1,
            ml: 3,
            raw_offset: 4,
            encoded_offset_value: OffBase::offset_to_c_value(4),
        }],
    };
    let mut compression_repeats = RepeatOffsets::new();
    let mut decompression_repeats = RepeatOffsets::new();

    let resolved = resolved_partition_sequences(
        &partition.sequences,
        &mut decompression_repeats,
        &mut compression_repeats,
    );

    assert_eq!(resolved.len(), 1);
    assert_eq!(decompression_repeats.as_offsets(), [4, 1, 4]);
    assert_eq!(compression_repeats.as_offsets(), [4, 1, 4]);
}

#[test]
fn derive_block_splits_refuses_tiny_sequence_counts() {
    let prepared = PreparedBlock {
        literals: vec![b'a'; 8],
        sequences: vec![
            PreparedSequence {
                ll: 1,
                ml: 3,
                raw_offset: 1,
                encoded_offset_value: 0,
            };
            4
        ],
    };
    let block = [b'a'; 20];
    let prefixes = sequence_prefixes(&prepared);
    let mut estimate_scratch = EstimateScratch::new();
    let splits = derive_block_splits(
        &block,
        &prepared,
        &prefixes,
        EstimateContext {
            strategy: Strategy::BtOpt,
            config: BlockCompressionConfig::for_c_strategy(7),
            fse_tables: &FseTables::new(),
            offset_history: OffsetHistory::new(),
            previous_huff_table: None,
        },
        &mut estimate_scratch,
    );

    assert!(splits.is_empty());
}

#[test]
fn derive_block_splits_requires_strictly_smaller_estimates_like_c() {
    assert!(!should_split(13_292, 12_781, 26_073, Strategy::BtOpt));
    assert!(!should_split(13_294, 12_781, 26_073, Strategy::BtOpt));
    assert!(!should_split(13_294, 12_781, 26_073, Strategy::BtUltra2));
    assert!(should_split(13_291, 12_781, 26_073, Strategy::BtOpt));
}

#[test]
fn derive_block_splits_finds_cheaper_halves() {
    let mut block = Vec::new();
    let mut literals = Vec::new();
    let mut sequences = Vec::new();
    for idx in 0..600 {
        let literal = if idx < 300 { b'a' } else { b'z' };
        block.extend_from_slice(&[literal; 4]);
        literals.push(literal);
        sequences.push(PreparedSequence {
            ll: 1,
            ml: 3,
            raw_offset: 1,
            encoded_offset_value: 0,
        });
    }
    let prepared = PreparedBlock {
        literals,
        sequences,
    };
    let prefixes = sequence_prefixes(&prepared);
    let mut estimate_scratch = EstimateScratch::new();

    let splits = derive_block_splits(
        &block,
        &prepared,
        &prefixes,
        EstimateContext {
            strategy: Strategy::BtOpt,
            config: BlockCompressionConfig::for_c_strategy(7),
            fse_tables: &FseTables::new(),
            offset_history: OffsetHistory::new(),
            previous_huff_table: None,
        },
        &mut estimate_scratch,
    );

    assert!(splits.contains(&300));
    assert_eq!(splits.last(), Some(&600));
}

#[test]
fn estimate_partition_size_does_not_use_rle_emission_cost_like_c() {
    let block = [0x44; 64];
    let prepared = PreparedBlock {
        literals: block.to_vec(),
        sequences: Vec::new(),
    };
    let prefixes = sequence_prefixes(&prepared);

    let estimate = estimate_partition_size(
        &block,
        &prepared,
        &prefixes,
        0,
        0,
        EstimateContext {
            strategy: Strategy::BtOpt,
            config: BlockCompressionConfig::for_c_strategy(7),
            fse_tables: &FseTables::new(),
            offset_history: OffsetHistory::new(),
            previous_huff_table: None,
        },
    );

    assert_eq!(estimate, 6);
}

#[test]
fn estimate_partition_size_keeps_tiny_literals_raw_like_c_entropy_stats() {
    let block = [0x44; 63];
    let prepared = PreparedBlock {
        literals: block.to_vec(),
        sequences: Vec::new(),
    };
    let prefixes = sequence_prefixes(&prepared);

    let estimate = estimate_partition_size(
        &block,
        &prepared,
        &prefixes,
        0,
        0,
        EstimateContext {
            strategy: Strategy::BtUltra2,
            config: BlockCompressionConfig::for_c_strategy(9),
            fse_tables: &FseTables::new(),
            offset_history: OffsetHistory::new(),
            previous_huff_table: None,
        },
    );

    assert_eq!(estimate, block.len() + 5);
}

#[test]
fn estimate_partition_size_does_not_cap_to_raw_block_size_like_c() {
    let mut block = Vec::with_capacity(1024);
    let mut state = 0x1234_5678_u32;
    for _ in 0..1024 {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        block.push(state as u8);
    }
    let prepared = PreparedBlock {
        literals: block.clone(),
        sequences: Vec::new(),
    };
    let prefixes = sequence_prefixes(&prepared);

    let estimate = estimate_partition_size(
        &block,
        &prepared,
        &prefixes,
        0,
        0,
        EstimateContext {
            strategy: Strategy::BtOpt,
            config: BlockCompressionConfig::for_c_strategy(7),
            fse_tables: &FseTables::new(),
            offset_history: OffsetHistory::new(),
            previous_huff_table: None,
        },
    );

    assert_eq!(estimate, block.len() + 5);
}

#[test]
fn partition_rle_obeys_first_block_policy_like_c() {
    let encoded = encode_rle_candidate_partition(BlockEncodingPolicy::frame_first_block());
    assert_ne!(parse_block_header(&encoded.bytes).1, BlockType::RLE);
}

#[test]
fn partition_can_emit_rle_after_first_block_like_c() {
    let encoded = encode_rle_candidate_partition(BlockEncodingPolicy::normal());
    let (last_block, block_type, block_size) = parse_block_header(&encoded.bytes);

    assert!(last_block);
    assert_eq!(block_type, BlockType::RLE);
    assert_eq!(block_size as usize, 32);
    assert_eq!(encoded.bytes, [0x03, 0x01, 0x00, 0x44]);
}

fn encode_rle_candidate_partition(policy: BlockEncodingPolicy) -> GreedyEncodedBlock {
    let block = [0x44; 32];
    let prepared = PreparedBlock {
        literals: block.to_vec(),
        sequences: Vec::new(),
    };
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    encode_partition(
        &block,
        true,
        RepeatOffsets::new(),
        prepared.as_ref(),
        PartitionEncodeContext {
            policy,
            strategy: Strategy::BtOpt,
            config: BlockCompressionConfig::for_c_strategy(Strategy::BtOpt as u8),
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
            previous_huff_table: None,
        },
    )
}

fn parse_block_header(bytes: &[u8]) -> (bool, BlockType, u32) {
    assert!(bytes.len() >= 3);
    let raw = u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16);
    let block_type = match (raw >> 1) & 0b11 {
        0 => BlockType::Raw,
        1 => BlockType::RLE,
        2 => BlockType::Compressed,
        _ => BlockType::Reserved,
    };
    (raw & 1 != 0, block_type, raw >> 3)
}
