use alloc::vec::Vec;

use super::greedy::{
    compress_block_btlazy2_no_dict_with_state,
    compress_block_btlazy2_no_dict_with_state_and_loaded_dict, compress_block_greedy_no_dict,
    compress_block_greedy_no_dict_with_state,
    compress_block_greedy_no_dict_with_state_and_loaded_dict,
    compress_block_lazy2_no_dict_with_state, compress_block_lazy_no_dict_with_state,
    GreedyBlockOutput, GreedyMatchState,
};
use super::greedy_block::{
    encode_block_hash_chain_no_dict, encode_block_hash_chain_no_dict_with_state,
    prepare_block_greedy_no_dict, prepare_block_greedy_no_dict_with_state,
    GreedyBlockEncodeContext, GreedyBlockSource, GreedyPreparedBlock, LazyBlockStrategy,
};
use super::params::{CompressionParameters, Strategy};
use super::sequence_store::{OffBase, RepeatCode, RepeatOffsets, StoredSequence};
use super::target_block::{encode_target_block_with_superblock_fallback, TargetBlockOptions};
use crate::blocks::block::BlockType;
use crate::common::MAX_BLOCK_SIZE;
use crate::encoding::blocks::{BlockCompressionConfig, PreparedBlock, PreparedSequence};
use crate::encoding::frame_compressor::{FseTables, OffsetHistory};
use crate::encoding::CompressionLevel;
use crate::fse::fse_encoder::SharedFSETable;

fn greedy_params(src_len: usize) -> CompressionParameters {
    CompressionParameters::for_level(greedy_level(src_len), src_len as u64, 0)
}

fn lazy_params(src_len: usize) -> CompressionParameters {
    CompressionParameters::for_level(lazy_level(src_len), src_len as u64, 0)
}

fn lazy2_params(src_len: usize) -> CompressionParameters {
    CompressionParameters::for_level(lazy2_level(src_len), src_len as u64, 0)
}

fn btlazy2_params(src_len: usize) -> CompressionParameters {
    CompressionParameters::for_level(btlazy2_level(src_len), src_len as u64, 0)
}

fn greedy_level(src_len: usize) -> i32 {
    if src_len <= 16 * 1024 {
        4
    } else {
        5
    }
}

fn large_window_greedy_params() -> CompressionParameters {
    CompressionParameters {
        window_log: 18,
        chain_log: 16,
        hash_log: 16,
        search_log: 5,
        min_match: 4,
        target_length: 0,
        strategy: Strategy::Greedy,
    }
}

fn lazy_level(src_len: usize) -> i32 {
    if src_len <= 16 * 1024 {
        5
    } else {
        6
    }
}

fn lazy2_level(src_len: usize) -> i32 {
    if src_len <= 16 * 1024 {
        6
    } else {
        8
    }
}

fn btlazy2_level(src_len: usize) -> i32 {
    if src_len <= 16 * 1024 {
        9
    } else {
        13
    }
}

#[test]
fn greedy_no_dict_keeps_tiny_blocks_as_last_literals() {
    let data = b"abcdefgh";

    let output =
        compress_block_greedy_no_dict(data, greedy_params(data.len()), RepeatOffsets::new());

    assert!(output.sequences.is_empty());
    assert_eq!(output.last_literals, data.len() as u32);
    assert_eq!(output.repeat_offsets, RepeatOffsets::new());
}

#[test]
fn greedy_no_dict_emits_repcode_at_next_position() {
    let data = b"aaaaaaaaaaaaaaaa";

    let output =
        compress_block_greedy_no_dict(data, greedy_params(data.len()), RepeatOffsets::new());

    assert_eq!(
        output.sequences,
        [StoredSequence::new(
            2,
            OffBase::Repeat(RepeatCode::First),
            14
        )]
    );
    assert_eq!(output.last_literals, 0);
    assert_eq!(output.repeat_offsets, RepeatOffsets::new());
}

#[test]
fn greedy_no_dict_uses_hash_chain_match() {
    let data = b"abcde12345abcde12345-tail";

    let output =
        compress_block_greedy_no_dict(data, greedy_params(data.len()), RepeatOffsets::new());

    assert_eq!(
        output.sequences,
        [StoredSequence::new(10, OffBase::Offset(10), 10)]
    );
    assert_eq!(output.last_literals, 5);
    assert_eq!(output.repeat_offsets.as_offsets(), [10, 1, 8]);
}

#[test]
fn greedy_no_dict_uses_row_matchfinder_when_window_is_large() {
    let mut data = Vec::new();
    data.extend_from_slice(b"abcdefghijklmnopqrst");
    data.extend_from_slice(b"row-match-finder-window");
    data.extend_from_slice(b"01234567890123456789");
    data.extend_from_slice(b"row-match-finder-window");
    data.extend_from_slice(b"-tail-with-room-for-cache");
    let mut state = GreedyMatchState::new();

    let output = compress_block_greedy_no_dict_with_state(
        &data,
        0..data.len(),
        large_window_greedy_params(),
        RepeatOffsets::new(),
        &mut state,
    );

    assert!(!state.tag_table.is_empty());
    assert!(output
        .sequences
        .iter()
        .any(|sequence| { matches!(sequence.off_base(), OffBase::Offset(43)) }));
}

#[test]
fn greedy_row_matchfinder_stops_at_c_cache_limit() {
    let data = b"abcde12345abcde12345-tail";
    let mut state = GreedyMatchState::new();

    let output = compress_block_greedy_no_dict_with_state(
        data,
        0..data.len(),
        large_window_greedy_params(),
        RepeatOffsets::new(),
        &mut state,
    );

    assert!(output.sequences.is_empty());
    assert!(!state.tag_table.is_empty());
    assert_eq!(output.last_literals, data.len() as u32);
}

#[test]
fn lazy_no_dict_uses_hash_chain_match() {
    let data = b"abcde12345abcde12345-tail";

    assert_eq!(lazy_params(data.len()).strategy, Strategy::Lazy);
    let output = lazy_output(data);

    assert_eq!(
        output.sequences,
        [StoredSequence::new(10, OffBase::Offset(10), 10)]
    );
    assert_eq!(output.last_literals, 5);
    assert_eq!(output.repeat_offsets.as_offsets(), [10, 1, 8]);
}

#[test]
fn lazy2_no_dict_uses_hash_chain_match() {
    let data = b"abcde12345abcde12345-tail";

    assert_eq!(lazy2_params(data.len()).strategy, Strategy::Lazy2);
    let output = lazy2_output(data);

    assert_eq!(
        output.sequences,
        [StoredSequence::new(10, OffBase::Offset(10), 10)]
    );
    assert_eq!(output.last_literals, 5);
    assert_eq!(output.repeat_offsets.as_offsets(), [10, 1, 8]);
}

#[test]
fn btlazy2_no_dict_uses_binary_tree_match() {
    let data = b"abcde12345abcde12345-tail";

    assert_eq!(btlazy2_params(data.len()).strategy, Strategy::BtLazy2);
    let output = btlazy2_output(data);

    assert_eq!(
        output.sequences,
        [StoredSequence::new(10, OffBase::Offset(10), 10)]
    );
    assert_eq!(output.last_literals, 5);
    assert_eq!(output.repeat_offsets.as_offsets(), [10, 1, 8]);
}

#[test]
fn greedy_no_dict_state_finds_previous_block_prefix_match() {
    let marker = b"greedy-cross-block-marker:0123456789abcdef";
    let mut data = deterministic_bytes(MAX_BLOCK_SIZE as usize);
    for pos in [4096, 24576, MAX_BLOCK_SIZE as usize - 1536] {
        data[pos..pos + marker.len()].copy_from_slice(marker);
    }
    let second_block_start = data.len();
    data.extend_from_slice(marker);
    data.extend_from_slice(&deterministic_bytes(512));

    let params = greedy_params(data.len());
    let mut state = GreedyMatchState::new();
    let first = compress_block_greedy_no_dict_with_state(
        &data,
        0..second_block_start,
        params,
        RepeatOffsets::new(),
        &mut state,
    );
    let second = compress_block_greedy_no_dict_with_state(
        &data,
        second_block_start..data.len(),
        params,
        first.repeat_offsets,
        &mut state,
    );

    assert!(second.sequences.iter().any(|sequence| matches!(
        sequence.off_base(),
        OffBase::Offset(offset) if sequence.lit_len == 0
            && offset as usize >= marker.len()
    )));
}

#[test]
fn greedy_loaded_dictionary_keeps_full_dictionary_valid_like_c() {
    let marker = b"early-greedy-dictionary-marker-0123456789abcdef";
    let mut dictionary = deterministic_bytes(2048);
    dictionary[30..30 + marker.len()].copy_from_slice(marker);
    let source = [marker.as_slice(), b"-payload-tail".as_slice()].concat();
    let mut combined = dictionary.clone();
    combined.extend_from_slice(&source);
    let params = loaded_dictionary_params(Strategy::Greedy);
    let block_range = dictionary.len()..combined.len();

    let mut no_loaded_state = GreedyMatchState::new();
    super::greedy_dict::load_prefix(&mut no_loaded_state, &combined, dictionary.len(), params);
    let no_loaded = compress_block_greedy_no_dict_with_state(
        &combined,
        block_range.clone(),
        params,
        RepeatOffsets::new(),
        &mut no_loaded_state,
    );

    let mut loaded_state = GreedyMatchState::new();
    super::greedy_dict::load_prefix(&mut loaded_state, &combined, dictionary.len(), params);
    let loaded = compress_block_greedy_no_dict_with_state_and_loaded_dict(
        &combined,
        block_range,
        params,
        RepeatOffsets::new(),
        &mut loaded_state,
        dictionary.len(),
    );

    assert!(!has_loaded_dictionary_match(&no_loaded, params));
    assert_loaded_dictionary_match(&loaded, params);
}

#[test]
fn btlazy2_loaded_dictionary_keeps_full_dictionary_valid_like_c() {
    let marker = b"early-binary-tree-dictionary-marker-0123456789abcdef";
    let mut dictionary = deterministic_bytes(2048);
    dictionary[30..30 + marker.len()].copy_from_slice(marker);
    let source = [marker.as_slice(), b"-payload-tail".as_slice()].concat();
    let mut combined = dictionary.clone();
    combined.extend_from_slice(&source);
    let params = loaded_dictionary_params(Strategy::BtLazy2);
    let block_range = dictionary.len()..combined.len();

    let mut no_loaded_state = GreedyMatchState::new();
    super::greedy_dict::load_binary_tree_prefix(
        &mut no_loaded_state,
        &combined,
        dictionary.len(),
        params,
    );
    let no_loaded = compress_block_btlazy2_no_dict_with_state(
        &combined,
        block_range.clone(),
        params,
        RepeatOffsets::new(),
        &mut no_loaded_state,
    );

    let mut loaded_state = GreedyMatchState::new();
    super::greedy_dict::load_binary_tree_prefix(
        &mut loaded_state,
        &combined,
        dictionary.len(),
        params,
    );
    let loaded = compress_block_btlazy2_no_dict_with_state_and_loaded_dict(
        &combined,
        block_range,
        params,
        RepeatOffsets::new(),
        &mut loaded_state,
        dictionary.len(),
    );

    assert!(!has_loaded_dictionary_match(&no_loaded, params));
    assert_loaded_dictionary_match(&loaded, params);
}

#[test]
fn greedy_no_dict_prepared_block_resolves_sequences() {
    let data = b"abcde12345abcde12345-tail";

    let prepared =
        prepare_block_greedy_no_dict(data, greedy_params(data.len()), RepeatOffsets::new());

    assert_eq!(prepared.prepared.literals, b"abcde12345-tail");
    assert_eq!(prepared.prepared.sequences.len(), 1);
    let sequence = prepared.prepared.sequences[0];
    assert_eq!(sequence.ll, 10);
    assert_eq!(sequence.ml, 10);
    assert_eq!(sequence.raw_offset, 10);
    assert_eq!(prepared.repeat_offsets.as_offsets(), [10, 1, 8]);
}

#[test]
fn greedy_state_reuses_sequence_store_after_block_preparation() {
    let block = b"abcde12345abcde12345-tail";
    let data = [block.as_slice(), block.as_slice()].concat();
    let params = greedy_params(data.len());
    let mut state = GreedyMatchState::new();

    let first_block = prepare_block_greedy_no_dict_with_state(
        &data,
        0..block.len(),
        params,
        RepeatOffsets::new(),
        &mut state,
    );
    let first = state.sequence_store_allocation();
    assert!(first.1 > 0);

    let _ = prepare_block_greedy_no_dict_with_state(
        &data,
        block.len()..data.len(),
        params,
        first_block.repeat_offsets,
        &mut state,
    );

    assert_eq!(state.sequence_store_allocation(), first);
}

#[test]
fn greedy_normal_native_entropy_recycles_sequence_store_across_blocks() {
    let block = b"abcde12345abcde12345-tail";
    let data = [block.as_slice(), block.as_slice()].concat();
    let params = greedy_params(data.len());
    let config = BlockCompressionConfig::for_c_strategy(params.strategy as u8);
    let mut state = GreedyMatchState::new();
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let first_block = encode_block_hash_chain_no_dict_with_state(
        GreedyBlockSource {
            src: &data,
            block_range: 0..block.len(),
            loaded_dict_end: 0,
        },
        false,
        params,
        config,
        RepeatOffsets::new(),
        &mut state,
        GreedyBlockEncodeContext {
            previous_huff_table: None,
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
        LazyBlockStrategy::Greedy,
    );
    let first_allocation = state.sequence_store_allocation();
    assert!(first_allocation.1 > 0);

    let _ = encode_block_hash_chain_no_dict_with_state(
        GreedyBlockSource {
            src: &data,
            block_range: block.len()..data.len(),
            loaded_dict_end: 0,
        },
        true,
        params,
        config,
        first_block.repeat_offsets,
        &mut state,
        GreedyBlockEncodeContext {
            previous_huff_table: first_block.new_huffman_table.as_ref(),
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
        LazyBlockStrategy::Greedy,
    );

    assert_eq!(state.sequence_store_allocation(), first_allocation);
}

#[test]
fn greedy_hidden_block_emits_compressed_block() {
    let data = b"abcde12345abcde12345-tail";
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_block_hash_chain_no_dict(
        data,
        true,
        greedy_params(data.len()),
        BlockCompressionConfig::for_level(CompressionLevel::Default),
        RepeatOffsets::new(),
        GreedyBlockEncodeContext {
            previous_huff_table: None,
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
        LazyBlockStrategy::Greedy,
    );
    let (last_block, block_type, block_size) = parse_block_header(&encoded.bytes);

    assert!(last_block);
    assert_eq!(block_type, BlockType::Compressed);
    assert_eq!(block_size as usize, encoded.bytes.len() - 3);
    assert_eq!(encoded.repeat_offsets.as_offsets(), [10, 1, 8]);
}

#[test]
fn greedy_hidden_block_falls_back_to_raw_when_not_smaller() {
    let data = b"abcdefgh";
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_block_hash_chain_no_dict(
        data,
        false,
        greedy_params(data.len()),
        BlockCompressionConfig::for_level(CompressionLevel::Default),
        RepeatOffsets::new(),
        GreedyBlockEncodeContext {
            previous_huff_table: None,
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
        LazyBlockStrategy::Greedy,
    );
    let (last_block, block_type, block_size) = parse_block_header(&encoded.bytes);

    assert!(!last_block);
    assert_eq!(block_type, BlockType::Raw);
    assert_eq!(block_size as usize, data.len());
    assert_eq!(&encoded.bytes[3..], data);
    assert_eq!(encoded.repeat_offsets, RepeatOffsets::new());
}

#[test]
fn greedy_hidden_block_emits_rle_for_single_byte_run() {
    let data = [0x6D; 256];
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_block_hash_chain_no_dict(
        &data,
        true,
        greedy_params(data.len()),
        BlockCompressionConfig::for_level(CompressionLevel::Default),
        RepeatOffsets::new(),
        GreedyBlockEncodeContext {
            previous_huff_table: None,
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
        LazyBlockStrategy::Greedy,
    );
    let (last_block, block_type, block_size) = parse_block_header(&encoded.bytes);

    assert!(last_block);
    assert_eq!(block_type, BlockType::RLE);
    assert_eq!(block_size as usize, data.len());
    assert_eq!(encoded.bytes, [0x03, 0x08, 0x00, 0x6D]);
    assert_eq!(encoded.repeat_offsets, RepeatOffsets::new());
}

#[test]
fn target_block_uses_literal_only_superblock_for_rle_literals() {
    let data = [0x5A; 64];
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();
    let prepared = GreedyPreparedBlock {
        prepared: PreparedBlock {
            literals: data.to_vec(),
            sequences: Vec::new(),
        },
        repeat_offsets: RepeatOffsets::new(),
    };

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
    assert_eq!(encoded.repeat_offsets, RepeatOffsets::new());
}

#[test]
fn target_block_keeps_raw_fallback_for_literal_only_non_rle_literals() {
    let mut data = Vec::new();
    for idx in 0..64 {
        data.push(idx);
    }
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();
    let prepared = GreedyPreparedBlock {
        prepared: PreparedBlock {
            literals: data.clone(),
            sequences: Vec::new(),
        },
        repeat_offsets: RepeatOffsets::new(),
    };

    let encoded = encode_target_block_with_superblock_fallback(
        &data,
        false,
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

    assert!(!last_block);
    assert_eq!(block_type, BlockType::Raw);
    assert_eq!(block_size as usize, data.len());
    assert_eq!(&encoded.bytes[3..], data);
}

#[test]
fn target_block_uses_huffman_literal_only_superblock_for_small_text() {
    let data = b"[workspace]\nresolver = \"3\"\nmembers = [\"ruzstd\", \"cli\", \"tools\"]\n";
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();
    let prepared = GreedyPreparedBlock {
        prepared: PreparedBlock {
            literals: data.to_vec(),
            sequences: Vec::new(),
        },
        repeat_offsets: RepeatOffsets::new(),
    };

    let encoded = encode_target_block_with_superblock_fallback(
        data,
        true,
        TargetBlockOptions {
            target_c_block_size: 2048,
            strategy: Strategy::BtUltra2,
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
    assert!(encoded.bytes.len() < data.len() + 3);
    assert!(encoded.new_huffman_table.is_some());
    assert_eq!(decode_compressed_block(&encoded.bytes), data);
}

#[test]
fn target_block_uses_sequence_metadata_for_sequence_block() {
    let mut data = Vec::new();
    for _ in 0..100 {
        data.extend_from_slice(b"abc");
    }
    let mut sequences = Vec::new();
    sequences.push(PreparedSequence {
        ll: 3,
        ml: 3,
        raw_offset: 3,
        encoded_offset_value: 0,
    });
    for _ in 1..99 {
        sequences.push(PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 0,
        });
    }
    let prepared = GreedyPreparedBlock {
        prepared: PreparedBlock {
            literals: b"abc".to_vec(),
            sequences,
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
    assert_eq!(offset_history.as_offsets(), (3, 3, 1));
}

#[test]
fn target_block_prefers_basic_sequence_modes_for_single_uniform_sequence() {
    let data = Vec::from([0x07; 1271]);
    let prepared = GreedyPreparedBlock {
        prepared: PreparedBlock {
            literals: Vec::from([0x07]),
            sequences: Vec::from([PreparedSequence {
                ll: 1,
                ml: 1270,
                raw_offset: 1,
                encoded_offset_value: 1,
            }]),
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
            strategy: Strategy::BtUltra2,
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
    assert_eq!(encoded.bytes.len(), 11);
    assert_eq!(decode_compressed_block(&encoded.bytes), data);
}

#[test]
fn target_block_accepts_uniform_sequence_code_superblock() {
    let mut data = Vec::new();
    let mut literals = Vec::new();
    let mut sequences = Vec::new();
    for idx in 0..80 {
        let chunk = [
            b'a' + (idx % 20) as u8,
            b'A' + (idx % 20) as u8,
            b'0' + (idx % 10) as u8,
        ];
        literals.extend_from_slice(&chunk);
        data.extend_from_slice(&chunk);
        data.extend_from_slice(&chunk);
        sequences.push(PreparedSequence {
            ll: 3,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 6,
        });
    }
    let prepared = GreedyPreparedBlock {
        prepared: PreparedBlock {
            literals,
            sequences,
        },
        repeat_offsets: RepeatOffsets::new(),
    };
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_target_block_with_superblock_fallback(
        &data,
        false,
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

    assert!(!last_block);
    assert_eq!(block_type, BlockType::Compressed);
    assert_eq!(block_size as usize, encoded.bytes.len() - 3);
    assert_eq!(decode_compressed_block(&encoded.bytes), data);
    assert_eq!(offset_history.as_offsets(), (3, 3, 3));
}

#[test]
fn target_block_preserves_previous_fse_tables_for_repeat_sequence_metadata() {
    let mut data = Vec::new();
    for _ in 0..100 {
        data.extend_from_slice(b"abc");
    }
    let mut sequences = Vec::new();
    sequences.push(PreparedSequence {
        ll: 3,
        ml: 3,
        raw_offset: 3,
        encoded_offset_value: 0,
    });
    for _ in 1..99 {
        sequences.push(PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 0,
        });
    }
    let prepared = GreedyPreparedBlock {
        prepared: PreparedBlock {
            literals: b"abc".to_vec(),
            sequences,
        },
        repeat_offsets: RepeatOffsets::new(),
    };
    let mut fse_tables = FseTables::new();
    fse_tables.ll_previous = Some(SharedFSETable::new(fse_tables.ll_default.clone()));
    fse_tables.ml_previous = Some(SharedFSETable::new(fse_tables.ml_default.clone()));
    fse_tables.of_previous = Some(SharedFSETable::new(fse_tables.of_default.clone()));
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
    assert!(fse_tables.ll_previous.is_some());
    assert!(fse_tables.ml_previous.is_some());
    assert!(fse_tables.of_previous.is_some());
    assert_eq!(offset_history.as_offsets(), (3, 3, 1));
}

#[test]
fn greedy_hidden_tiny_rle_candidate_stays_raw_like_c() {
    let data = [0x6D; 6];
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_block_hash_chain_no_dict(
        &data,
        true,
        greedy_params(data.len()),
        BlockCompressionConfig::for_level(CompressionLevel::Default),
        RepeatOffsets::new(),
        GreedyBlockEncodeContext {
            previous_huff_table: None,
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
        LazyBlockStrategy::Greedy,
    );
    let (last_block, block_type, block_size) = parse_block_header(&encoded.bytes);

    assert!(last_block);
    assert_eq!(block_type, BlockType::Raw);
    assert_eq!(block_size as usize, data.len());
    assert_eq!(&encoded.bytes[3..], data);
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

fn decode_compressed_block(encoded: &[u8]) -> Vec<u8> {
    let mut block_decoder = crate::decoding::block_decoder::new();
    let (header, header_size) = block_decoder
        .read_block_header(encoded)
        .expect("block header should parse");
    assert_eq!(header.block_type, BlockType::Compressed);
    assert_eq!(
        header.content_size as usize,
        encoded.len() - header_size as usize
    );

    let mut scratch = crate::decoding::scratch::DecoderScratch::new(128 * 1024);
    block_decoder
        .decode_block_content(&header, &mut scratch, &encoded[header_size as usize..])
        .expect("block content should decode");
    scratch.buffer.drain()
}

fn lazy_output(data: &[u8]) -> GreedyBlockOutput {
    let mut state = GreedyMatchState::new();
    compress_block_lazy_no_dict_with_state(
        data,
        0..data.len(),
        lazy_params(data.len()),
        RepeatOffsets::new(),
        &mut state,
    )
}

fn lazy2_output(data: &[u8]) -> GreedyBlockOutput {
    let mut state = GreedyMatchState::new();
    compress_block_lazy2_no_dict_with_state(
        data,
        0..data.len(),
        lazy2_params(data.len()),
        RepeatOffsets::new(),
        &mut state,
    )
}

fn btlazy2_output(data: &[u8]) -> GreedyBlockOutput {
    let mut state = GreedyMatchState::new();
    compress_block_btlazy2_no_dict_with_state(
        data,
        0..data.len(),
        btlazy2_params(data.len()),
        RepeatOffsets::new(),
        &mut state,
    )
}

fn loaded_dictionary_params(strategy: Strategy) -> CompressionParameters {
    CompressionParameters {
        window_log: 10,
        chain_log: 13,
        hash_log: 12,
        search_log: 4,
        min_match: 4,
        target_length: 0,
        strategy,
    }
}

fn assert_loaded_dictionary_match(output: &GreedyBlockOutput, params: CompressionParameters) {
    assert!(has_loaded_dictionary_match(output, params));
}

fn has_loaded_dictionary_match(output: &GreedyBlockOutput, params: CompressionParameters) -> bool {
    output.sequences.iter().any(|sequence| {
        matches!(
            sequence.off_base(),
            OffBase::Offset(offset) if offset as usize > (1_usize << params.window_log)
        )
    })
}

fn deterministic_bytes(len: usize) -> Vec<u8> {
    let mut state = 0xA511_E9B3_u32;
    let mut bytes = Vec::with_capacity(len);
    for _ in 0..len {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        bytes.push(state as u8);
    }
    bytes
}
