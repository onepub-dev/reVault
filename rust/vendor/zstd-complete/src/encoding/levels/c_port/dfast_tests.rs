use alloc::vec::Vec;

use super::dfast::{
    compress_block_double_fast_no_dict, compress_block_double_fast_no_dict_with_state,
    compress_block_double_fast_no_dict_with_state_and_loaded_dict, DFastMatchState,
};
use super::dfast_block::{
    encode_block_double_fast_no_dict, encode_block_double_fast_no_dict_with_state,
    prepare_block_double_fast_no_dict, prepare_block_double_fast_no_dict_with_state,
    DFastBlockEncodeContext, DFastBlockSource,
};
use super::dfast_frame::{
    encode_frame_double_fast_no_dict, encode_frame_double_fast_with_dictionary,
    encode_single_block_frame_double_fast_no_dict,
};
use super::dictionary::{parse_dictionary, DictionaryContentType};
use super::params::{CompressionParameters, Strategy};
use super::sequence_store::{OffBase, RepeatCode, RepeatOffsets, StoredSequence};
use super::test_dictionary::{dictionary_content, full_dictionary_fixture, DICT_ID};
use crate::blocks::block::BlockType;
use crate::common::MAX_BLOCK_SIZE;
use crate::decoding::dictionary::Dictionary;
use crate::decoding::FrameDecoder;
use crate::encoding::blocks::BlockCompressionConfig;
use crate::encoding::frame_compressor::{FseTables, OffsetHistory};
use crate::encoding::CompressionLevel;

fn level3_params(src_len: usize) -> CompressionParameters {
    CompressionParameters::for_level(3, src_len as u64, 0)
}

#[test]
fn double_fast_no_dict_keeps_tiny_blocks_as_last_literals() {
    let data = b"abcdefgh";

    let output =
        compress_block_double_fast_no_dict(data, level3_params(data.len()), RepeatOffsets::new());

    assert!(output.sequences.is_empty());
    assert_eq!(output.last_literals, data.len() as u32);
    assert_eq!(output.repeat_offsets, RepeatOffsets::new());
}

#[test]
fn double_fast_no_dict_emits_repcode_at_next_position() {
    let data = b"aaaaaaaaaaaaaaaa";

    let output =
        compress_block_double_fast_no_dict(data, level3_params(data.len()), RepeatOffsets::new());

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
fn double_fast_no_dict_uses_long_match_over_short_match() {
    let data = b"abcde12345abcde12345-tail";

    let output =
        compress_block_double_fast_no_dict(data, level3_params(data.len()), RepeatOffsets::new());

    assert_eq!(
        output.sequences,
        [StoredSequence::new(10, OffBase::Offset(10), 10)]
    );
    assert_eq!(output.last_literals, 5);
    assert_eq!(output.repeat_offsets.as_offsets(), [10, 1, 8]);
}

#[test]
fn double_fast_loaded_dictionary_keeps_full_dictionary_valid_like_c() {
    let marker = b"early-dictionary-marker-0123456789abcdef";
    let mut dictionary = deterministic_bytes(2048);
    dictionary[30..30 + marker.len()].copy_from_slice(marker);
    let source = [marker.as_slice(), b"-payload-tail".as_slice()].concat();
    let mut combined = dictionary.clone();
    combined.extend_from_slice(&source);
    let params = CompressionParameters {
        window_log: 10,
        chain_log: 12,
        hash_log: 12,
        search_log: 1,
        min_match: 4,
        target_length: 0,
        strategy: Strategy::DFast,
    };
    let block_range = dictionary.len()..combined.len();

    let mut no_loaded_state = DFastMatchState::new();
    super::dfast_dict::load_prefix(&mut no_loaded_state, &combined, dictionary.len(), params);
    let no_loaded = compress_block_double_fast_no_dict_with_state(
        &combined,
        block_range.clone(),
        params,
        RepeatOffsets::new(),
        &mut no_loaded_state,
    );

    let mut loaded_state = DFastMatchState::new();
    super::dfast_dict::load_prefix(&mut loaded_state, &combined, dictionary.len(), params);
    let loaded = compress_block_double_fast_no_dict_with_state_and_loaded_dict(
        &combined,
        block_range,
        params,
        RepeatOffsets::new(),
        &mut loaded_state,
        dictionary.len(),
    );

    assert!(no_loaded.sequences.is_empty());
    assert!(loaded.sequences.iter().any(|sequence| {
        matches!(
            sequence.off_base(),
            OffBase::Offset(offset) if offset as usize > (1_usize << params.window_log)
        )
    }));
}

#[test]
fn double_fast_no_dict_prepared_block_resolves_sequences() {
    let data = b"abcde12345abcde12345-tail";

    let prepared =
        prepare_block_double_fast_no_dict(data, level3_params(data.len()), RepeatOffsets::new());

    assert_eq!(prepared.prepared.literals, b"abcde12345-tail");
    assert_eq!(prepared.prepared.sequences.len(), 1);
    let sequence = prepared.prepared.sequences[0];
    assert_eq!(sequence.ll, 10);
    assert_eq!(sequence.ml, 10);
    assert_eq!(sequence.raw_offset, 10);
    assert_eq!(prepared.repeat_offsets.as_offsets(), [10, 1, 8]);
}

#[test]
fn double_fast_state_reuses_sequence_store_after_block_preparation() {
    let block = b"abcde12345abcde12345-tail";
    let data = [block.as_slice(), block.as_slice()].concat();
    let params = level3_params(data.len());
    let mut state = DFastMatchState::new();

    let first_block = prepare_block_double_fast_no_dict_with_state(
        &data,
        0..block.len(),
        params,
        RepeatOffsets::new(),
        &mut state,
    );
    let first = state.sequence_store_allocation();
    assert!(first.1 > 0);

    let _ = prepare_block_double_fast_no_dict_with_state(
        &data,
        block.len()..data.len(),
        params,
        first_block.repeat_offsets,
        &mut state,
    );

    assert_eq!(state.sequence_store_allocation(), first);
}

#[test]
fn double_fast_hidden_block_emits_compressed_block() {
    let data = b"abcde12345abcde12345-tail";
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_block_double_fast_no_dict(
        data,
        true,
        level3_params(data.len()),
        BlockCompressionConfig::for_level(CompressionLevel::Default),
        RepeatOffsets::new(),
        DFastBlockEncodeContext {
            previous_huff_table: None,
            huffman_build_scratch: &mut Default::default(),
            fse_build_scratch: &mut Default::default(),
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
    );
    let (last_block, block_type, block_size) = parse_block_header(&encoded.bytes);

    assert!(last_block);
    assert_eq!(block_type, BlockType::Compressed);
    assert_eq!(block_size as usize, encoded.bytes.len() - 3);
    assert_eq!(encoded.repeat_offsets.as_offsets(), [10, 1, 8]);
}

#[test]
fn double_fast_hidden_block_emits_rle_for_single_byte_run() {
    let data = [0xC3; 256];
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_block_double_fast_no_dict(
        &data,
        true,
        level3_params(data.len()),
        BlockCompressionConfig::for_level(CompressionLevel::Default),
        RepeatOffsets::new(),
        DFastBlockEncodeContext {
            previous_huff_table: None,
            huffman_build_scratch: &mut Default::default(),
            fse_build_scratch: &mut Default::default(),
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
    );
    let (last_block, block_type, block_size) = parse_block_header(&encoded.bytes);

    assert!(last_block);
    assert_eq!(block_type, BlockType::RLE);
    assert_eq!(block_size as usize, data.len());
    assert_eq!(encoded.bytes, [0x03, 0x08, 0x00, 0xC3]);
    assert_eq!(encoded.repeat_offsets, RepeatOffsets::new());
}

#[test]
fn double_fast_hidden_tiny_rle_candidate_stays_raw_like_c() {
    let data = [0xC3; 6];
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_block_double_fast_no_dict(
        &data,
        true,
        level3_params(data.len()),
        BlockCompressionConfig::for_level(CompressionLevel::Default),
        RepeatOffsets::new(),
        DFastBlockEncodeContext {
            previous_huff_table: None,
            huffman_build_scratch: &mut Default::default(),
            fse_build_scratch: &mut Default::default(),
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
    );
    let (last_block, block_type, block_size) = parse_block_header(&encoded.bytes);

    assert!(last_block);
    assert_eq!(block_type, BlockType::Raw);
    assert_eq!(block_size as usize, data.len());
    assert_eq!(&encoded.bytes[3..], data);
}

#[test]
fn double_fast_hidden_block_falls_back_to_raw_when_not_smaller() {
    let data = b"abcdefgh";
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let encoded = encode_block_double_fast_no_dict(
        data,
        false,
        level3_params(data.len()),
        BlockCompressionConfig::for_level(CompressionLevel::Default),
        RepeatOffsets::new(),
        DFastBlockEncodeContext {
            previous_huff_table: None,
            huffman_build_scratch: &mut Default::default(),
            fse_build_scratch: &mut Default::default(),
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
    );
    let (last_block, block_type, block_size) = parse_block_header(&encoded.bytes);

    assert!(!last_block);
    assert_eq!(block_type, BlockType::Raw);
    assert_eq!(block_size as usize, data.len());
    assert_eq!(&encoded.bytes[3..], data);
    assert_eq!(encoded.repeat_offsets, RepeatOffsets::new());
}

#[test]
fn double_fast_state_reuses_prepared_allocations_through_compressed_and_raw_blocks() {
    let mut data = Vec::new();
    while data.len() < 4096 {
        data.extend_from_slice(b"tenant=beta method=GET route=/v2/search status=200 bytes=137\n");
    }
    data.truncate(4096);
    let raw_start = data.len();
    data.extend_from_slice(&[0x02, 0x13, 0x27, 0x3B, 0x51, 0x68, 0x80, 0x99]);

    let params = level3_params(data.len());
    let config = BlockCompressionConfig::for_level(CompressionLevel::Default);
    let mut state = DFastMatchState::new();
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();
    let first = encode_block_double_fast_no_dict_with_state(
        DFastBlockSource {
            src: &data,
            block_range: 0..raw_start,
            loaded_dict_end: 0,
        },
        false,
        params,
        config,
        RepeatOffsets::new(),
        &mut state,
        DFastBlockEncodeContext {
            previous_huff_table: None,
            huffman_build_scratch: &mut Default::default(),
            fse_build_scratch: &mut Default::default(),
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
    );
    assert_eq!(parse_block_header(&first.bytes).1, BlockType::Compressed);
    let allocation = state.prepared_store_allocation();
    assert!(allocation.0 .1 >= raw_start);
    assert!(allocation.1 .1 > 0);

    let second = encode_block_double_fast_no_dict_with_state(
        DFastBlockSource {
            src: &data,
            block_range: raw_start..data.len(),
            loaded_dict_end: 0,
        },
        true,
        params,
        config,
        first.repeat_offsets,
        &mut state,
        DFastBlockEncodeContext {
            previous_huff_table: first.new_huffman_table.as_ref(),
            huffman_build_scratch: &mut Default::default(),
            fse_build_scratch: &mut Default::default(),
            fse_tables: &mut fse_tables,
            offset_history: &mut offset_history,
        },
    );

    assert_eq!(parse_block_header(&second.bytes).1, BlockType::Raw);
    assert_eq!(state.prepared_store_allocation(), allocation);
}

#[test]
fn double_fast_state_finds_previous_block_prefix_match() {
    let marker = b"double-fast-cross-block-marker:0123456789abcdef";
    let mut data = deterministic_bytes(MAX_BLOCK_SIZE as usize);
    for pos in [2048, 16384, MAX_BLOCK_SIZE as usize - 1024] {
        data[pos..pos + marker.len()].copy_from_slice(marker);
    }
    let second_block_start = data.len();
    data.extend_from_slice(marker);
    data.extend_from_slice(&deterministic_bytes(512));

    let params = level3_params(data.len());
    let mut state = DFastMatchState::new();
    let first = compress_block_double_fast_no_dict_with_state(
        &data,
        0..second_block_start,
        params,
        RepeatOffsets::new(),
        &mut state,
    );
    let second = compress_block_double_fast_no_dict_with_state(
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
fn double_fast_hidden_frame_round_trips_compressed_block() {
    let data = b"abcde12345abcde12345-tail";
    let encoded = encode_single_block_frame_double_fast_no_dict(data, 3);

    assert_round_trips(&encoded, data);
}

#[test]
fn double_fast_hidden_frame_round_trips_rle_block() {
    let data = [0x37; 4096];
    let encoded = encode_single_block_frame_double_fast_no_dict(&data, 3);

    assert_round_trips(&encoded, &data);
}

#[test]
fn double_fast_hidden_frame_round_trips_multiple_blocks() {
    let mut data = Vec::new();
    while data.len() < (MAX_BLOCK_SIZE as usize * 2) + 1024 {
        data.extend_from_slice(b"tenant=beta method=GET route=/v2/search status=200 bytes=137\n");
    }
    data.truncate((MAX_BLOCK_SIZE as usize * 2) + 1024);

    let encoded = encode_frame_double_fast_no_dict(&data, 3);

    assert!(count_frame_blocks(&encoded) > 1);
    assert_round_trips(&encoded, &data);
}

#[test]
fn double_fast_frame_with_dictionary_writes_dict_id_and_round_trips() {
    let dict_bytes = full_dictionary_fixture();
    let parsed = parse_dictionary(&dict_bytes, DictionaryContentType::Auto, false)
        .unwrap()
        .expect("full dictionary");
    let mut data = Vec::new();
    for _ in 0..8 {
        data.extend_from_slice(dictionary_content());
    }

    let encoded = encode_frame_double_fast_with_dictionary(&data, 3, parsed);
    let (header, _) =
        crate::decoding::frame::read_frame_header(encoded.as_slice()).expect("frame header");

    assert_eq!(header.dictionary_id(), Some(DICT_ID));
    assert_eq!(first_frame_block_type(&encoded), BlockType::Compressed);
    assert_round_trips_with_dictionary(&encoded, &data, &dict_bytes);
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

fn assert_round_trips(encoded: &[u8], expected: &[u8]) {
    let mut decoded = Vec::with_capacity(expected.len());
    FrameDecoder::new()
        .decode_all_to_vec(encoded, &mut decoded)
        .unwrap();

    assert_eq!(decoded, expected);
}

fn assert_round_trips_with_dictionary(encoded: &[u8], expected: &[u8], dict: &[u8]) {
    let mut decoded = Vec::with_capacity(expected.len());
    let mut decoder = FrameDecoder::new();
    decoder
        .add_dict(Dictionary::decode_dict(dict).unwrap())
        .unwrap();
    decoder.decode_all_to_vec(encoded, &mut decoded).unwrap();

    assert_eq!(decoded, expected);
}

fn count_frame_blocks(encoded: &[u8]) -> usize {
    let (_, frame_header_size) =
        crate::decoding::frame::read_frame_header(encoded).expect("frame header should parse");
    let mut block_decoder = crate::decoding::block_decoder::new();
    let mut offset = frame_header_size as usize;
    let mut blocks = 0;

    loop {
        let (header, block_header_size) = block_decoder
            .read_block_header(&encoded[offset..])
            .expect("block header should parse");
        offset += block_header_size as usize + header.content_size as usize;
        blocks += 1;

        if header.last_block {
            break blocks;
        }
    }
}

fn first_frame_block_type(encoded: &[u8]) -> BlockType {
    let (_, frame_header_size) =
        crate::decoding::frame::read_frame_header(encoded).expect("frame header should parse");
    let mut block_decoder = crate::decoding::block_decoder::new();
    let (header, _) = block_decoder
        .read_block_header(&encoded[frame_header_size as usize..])
        .expect("block header should parse");

    header.block_type
}

fn deterministic_bytes(len: usize) -> Vec<u8> {
    let mut state = 0x9E37_79B9_u32;
    let mut bytes = Vec::with_capacity(len);
    for _ in 0..len {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        bytes.push(state as u8);
    }
    bytes
}
