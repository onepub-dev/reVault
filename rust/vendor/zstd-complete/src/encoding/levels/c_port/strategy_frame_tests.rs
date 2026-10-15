use alloc::vec::Vec;

use super::{
    dictionary::PreparedDictionary,
    params::Strategy,
    strategy_frame::{
        encode_frame_no_dict, encode_frame_no_dict_with_target_c_block_size,
        encode_frame_with_dictionary, encode_frame_with_dictionary_and_target_c_block_size,
        encode_frame_with_prepared_dictionary, strategy_for_level,
        strategy_for_level_with_dictionary,
    },
    test_dictionary::{dictionary_content, full_dictionary_fixture, DICT_ID},
};
use crate::{
    blocks::block::BlockType,
    common::MAX_BLOCK_SIZE,
    decoding::{dictionary::Dictionary, FrameDecoder},
};

#[test]
fn strategy_frame_routes_level_one_to_fast() {
    let data = b"level-one-fast-level-one-fast-level-one-fast";

    assert_eq!(strategy_for_level(1, data.len()), Strategy::Fast);
    assert_round_trips(&encode_frame_no_dict(data, 1), data);
}

#[test]
fn strategy_frame_routes_negative_levels_to_fast() {
    let data = b"negative-level-fast-negative-level-fast";

    assert_eq!(strategy_for_level(-5, data.len()), Strategy::Fast);
    assert_round_trips(&encode_frame_no_dict(data, -5), data);
}

#[test]
fn strategy_frame_routes_default_level_to_double_fast() {
    let data = b"default-level-double-fast-default-level-double-fast";

    assert_eq!(strategy_for_level(0, data.len()), Strategy::DFast);
    assert_round_trips(&encode_frame_no_dict(data, 0), data);
}

#[test]
fn strategy_frame_routes_level_three_to_double_fast() {
    let data = b"level-three-double-fast-level-three-double-fast";

    assert_eq!(strategy_for_level(3, data.len()), Strategy::DFast);
    assert_round_trips(&encode_frame_no_dict(data, 3), data);
}

#[test]
fn strategy_frame_routes_greedy_levels_to_greedy() {
    let mut data = Vec::new();
    while data.len() < (MAX_BLOCK_SIZE as usize * 2) + 1024 {
        data.extend_from_slice(b"strategy=greedy route=/sync status=202 bytes=1874\n");
    }
    data.truncate((MAX_BLOCK_SIZE as usize * 2) + 1024);

    assert_eq!(strategy_for_level(5, data.len()), Strategy::Greedy);
    assert_round_trips(&encode_frame_no_dict(&data, 5), &data);
}

#[test]
fn strategy_frame_presplit_uses_one_shot_remaining_input() {
    let mut data = Vec::new();
    while data.len() < MAX_BLOCK_SIZE as usize * 4 {
        data.extend_from_slice(b"the quick brown fox jumps over the lazy dog\n");
    }
    data.truncate(MAX_BLOCK_SIZE as usize * 4);

    let encoded = encode_frame_no_dict(&data, 5);

    assert!(count_frame_blocks(&encoded) > 8);
    assert_round_trips(&encoded, &data);
}

#[test]
fn strategy_frame_routes_lazy_levels_to_lazy() {
    let mut data = Vec::new();
    while data.len() < (MAX_BLOCK_SIZE as usize * 2) + 1024 {
        data.extend_from_slice(b"strategy=lazy route=/sync status=202 bytes=1874\n");
    }
    data.truncate((MAX_BLOCK_SIZE as usize * 2) + 1024);

    assert_eq!(strategy_for_level(6, data.len()), Strategy::Lazy);
    assert_round_trips(&encode_frame_no_dict(&data, 6), &data);
}

#[test]
fn strategy_frame_routes_lazy2_levels_to_lazy2() {
    let mut data = Vec::new();
    while data.len() < (MAX_BLOCK_SIZE as usize * 2) + 1024 {
        data.extend_from_slice(b"strategy=lazy2 route=/sync status=202 bytes=1874\n");
    }
    data.truncate((MAX_BLOCK_SIZE as usize * 2) + 1024);

    assert_eq!(strategy_for_level(8, data.len()), Strategy::Lazy2);
    assert_round_trips(&encode_frame_no_dict(&data, 8), &data);
}

#[test]
fn strategy_frame_routes_btlazy2_levels_to_btlazy2() {
    let mut data = Vec::new();
    while data.len() < (MAX_BLOCK_SIZE as usize * 2) + 1024 {
        data.extend_from_slice(b"strategy=btlazy2 route=/sync status=202 bytes=1874\n");
    }
    data.truncate((MAX_BLOCK_SIZE as usize * 2) + 1024);

    assert_eq!(strategy_for_level(13, data.len()), Strategy::BtLazy2);
    assert_round_trips(&encode_frame_no_dict(&data, 13), &data);
}

#[test]
fn strategy_frame_routes_btopt_levels_to_btopt() {
    let mut data = Vec::new();
    while data.len() < (MAX_BLOCK_SIZE as usize * 2) + 1024 {
        data.extend_from_slice(b"strategy=btopt route=/archive status=200 bytes=1874\n");
    }
    data.truncate((MAX_BLOCK_SIZE as usize * 2) + 1024);

    assert_eq!(strategy_for_level(16, data.len()), Strategy::BtOpt);
    assert_round_trips(&encode_frame_no_dict(&data, 16), &data);
}

#[test]
fn strategy_frame_routes_btultra_levels_to_btultra() {
    let mut data = Vec::new();
    while data.len() < (MAX_BLOCK_SIZE as usize * 2) + 1024 {
        data.extend_from_slice(b"strategy=btultra route=/archive status=200 bytes=1874\n");
    }
    data.truncate((MAX_BLOCK_SIZE as usize * 2) + 1024);

    assert_eq!(strategy_for_level(18, data.len()), Strategy::BtUltra);
    assert_round_trips(&encode_frame_no_dict(&data, 18), &data);
}

#[test]
fn strategy_frame_routes_btultra2_levels_to_btultra2() {
    let mut data = Vec::new();
    while data.len() < (MAX_BLOCK_SIZE as usize * 2) + 1024 {
        data.extend_from_slice(b"strategy=btultra2 route=/archive status=200 bytes=1874\n");
    }
    data.truncate((MAX_BLOCK_SIZE as usize * 2) + 1024);

    assert_eq!(strategy_for_level(19, data.len()), Strategy::BtUltra2);
    assert_round_trips(&encode_frame_no_dict(&data, 19), &data);
}

#[test]
fn strategy_frame_round_trips_multiple_blocks() {
    let mut data = Vec::new();
    while data.len() < (MAX_BLOCK_SIZE as usize * 2) + 2048 {
        data.extend_from_slice(b"strategy=double-fast route=/events status=200 bytes=915\n");
    }
    data.truncate((MAX_BLOCK_SIZE as usize * 2) + 2048);

    let encoded = encode_frame_no_dict(&data, 3);

    assert!(count_frame_blocks(&encoded) > 1);
    assert_round_trips(&encoded, &data);
}

#[test]
fn strategy_frame_corpus_size_regressions_round_trip() {
    let level_one = include_bytes!("../../../../decodecorpus_files/z000021");
    let level_five = include_bytes!("../../../../decodecorpus_files/z000033");
    let z000040 = include_bytes!("../../../../decodecorpus_files/z000040");

    let encoded = encode_frame_no_dict(level_one, 1);
    assert_round_trips(&encoded, level_one);
    assert!(
        encoded.len() <= 32_000,
        "level 1 corpus literal table regressed to {} bytes",
        encoded.len()
    );

    let encoded = encode_frame_no_dict(level_five, 5);
    assert_round_trips(&encoded, level_five);
    assert!(
        encoded.len() <= 514_500,
        "level 5 corpus literal table regressed to {} bytes",
        encoded.len()
    );

    let encoded = encode_frame_no_dict(z000040, 1);
    assert_round_trips(&encoded, z000040);
    assert!(
        encoded.len() <= 39_320,
        "level 1 fast hash sentinel regressed to {} bytes",
        encoded.len()
    );

    let encoded = encode_frame_no_dict(z000040, 3);
    assert_round_trips(&encoded, z000040);
    assert!(
        encoded.len() <= 38_400,
        "level 3 double-fast hash insertion regressed to {} bytes",
        encoded.len()
    );

    let encoded = encode_frame_no_dict(z000040, 8);
    assert_round_trips(&encoded, z000040);
    assert!(
        encoded.len() <= 32_600,
        "level 8 lazy repcode search regressed to {} bytes",
        encoded.len()
    );
}

#[test]
fn strategy_frame_does_not_emit_rle_first_block_like_c() {
    let data = [0x61; 4096];

    for level in [1, 3, 5, 6, 8, 13, 16, 18, 19] {
        let encoded = encode_frame_no_dict(&data, level);

        assert_ne!(first_frame_block_type(&encoded), BlockType::RLE);
        assert_round_trips(&encoded, &data);
    }
}

#[test]
fn strategy_frame_target_c_block_size_round_trips_supported_strategies() {
    let mut data = Vec::new();
    while data.len() < (MAX_BLOCK_SIZE as usize * 2) + 1536 {
        data.extend_from_slice(b"target-mode route=/archive status=200 bytes=1874\n");
    }
    data.truncate((MAX_BLOCK_SIZE as usize * 2) + 1536);

    for level in [1, 3, 5, 6, 8, 13, 16, 18, 19] {
        let encoded = encode_frame_no_dict_with_target_c_block_size(&data, level, 2048)
            .expect("target mode should be supported for no-dictionary strategies");

        assert_eq!(first_frame_block_type(&encoded), BlockType::Compressed);
        assert_round_trips(&encoded, &data);
    }
}

#[test]
fn strategy_frame_target_c_block_size_rejects_invalid_sizes() {
    let data = b"target mode target mode target mode target mode";

    assert_eq!(
        encode_frame_no_dict_with_target_c_block_size(data, 16, 128 * 1024 + 1),
        None
    );
}

#[test]
fn strategy_frame_routes_dictionary_levels_and_round_trips() {
    let dict = full_dictionary_fixture();
    let data = dictionary_payload();

    for level in [1, 3, 5, 6, 8, 13, 16, 18, 19] {
        let encoded = encode_frame_with_dictionary(&data, level, &dict).unwrap();
        let (header, _) = crate::decoding::frame::read_frame_header(encoded.as_slice())
            .expect("frame header should parse");

        assert_eq!(header.dictionary_id(), Some(DICT_ID));
        assert_round_trips_with_dictionary(&encoded, &data, &dict);
    }
}

#[test]
fn strategy_frame_routes_prepared_dictionary_levels_and_round_trips() {
    let dict = full_dictionary_fixture();
    let prepared = PreparedDictionary::from_bytes(&dict)
        .unwrap()
        .expect("full dictionary");
    let data = dictionary_payload();

    for level in [1, 3, 5, 6, 8, 13, 16, 18, 19] {
        let encoded = encode_frame_with_prepared_dictionary(&data, level, &prepared);
        let (header, _) = crate::decoding::frame::read_frame_header(encoded.as_slice())
            .expect("frame header should parse");

        assert_eq!(header.dictionary_id(), Some(DICT_ID));
        assert_round_trips_with_dictionary(&encoded, &data, &dict);
    }
}

#[test]
fn strategy_frame_dictionary_target_c_block_size_round_trips_optimal_strategy() {
    let dict = full_dictionary_fixture();
    let data = dictionary_payload();

    let encoded = encode_frame_with_dictionary_and_target_c_block_size(&data, 16, &dict, 2048)
        .unwrap()
        .expect("level 16 dictionary target mode is target-wired");
    let (header, _) = crate::decoding::frame::read_frame_header(encoded.as_slice())
        .expect("frame header should parse");

    assert_eq!(header.dictionary_id(), Some(DICT_ID));
    assert_round_trips_with_dictionary(&encoded, &data, &dict);
}

#[test]
fn strategy_frame_dictionary_target_c_block_size_round_trips_hash_chain_strategies() {
    let dict = full_dictionary_fixture();
    let data = dictionary_payload();
    let mut checked = 0;

    for level in 1..=22 {
        if !matches!(
            strategy_for_level_with_dictionary(level, data.len(), dict.len()),
            Strategy::Greedy | Strategy::Lazy | Strategy::Lazy2 | Strategy::BtLazy2
        ) {
            continue;
        }
        checked += 1;

        let encoded =
            encode_frame_with_dictionary_and_target_c_block_size(&data, level, &dict, 2048)
                .unwrap()
                .expect("hash-chain dictionary target mode is target-wired");
        let (header, _) = crate::decoding::frame::read_frame_header(encoded.as_slice())
            .expect("frame header should parse");

        assert_eq!(header.dictionary_id(), Some(DICT_ID));
        assert_round_trips_with_dictionary(&encoded, &data, &dict);
    }

    assert!(checked > 0, "fixture should exercise hash-chain levels");
}

#[test]
fn strategy_frame_dictionary_target_c_block_size_round_trips_fast_strategies() {
    let dict = full_dictionary_fixture();
    let data = dictionary_payload();
    let mut checked = 0;

    for level in 1..=22 {
        if !matches!(
            strategy_for_level_with_dictionary(level, data.len(), dict.len()),
            Strategy::Fast | Strategy::DFast
        ) {
            continue;
        }
        checked += 1;

        let encoded =
            encode_frame_with_dictionary_and_target_c_block_size(&data, level, &dict, 2048)
                .unwrap()
                .expect("fast dictionary target mode is target-wired");
        let (header, _) = crate::decoding::frame::read_frame_header(encoded.as_slice())
            .expect("frame header should parse");

        assert_eq!(header.dictionary_id(), Some(DICT_ID));
        assert_round_trips_with_dictionary(&encoded, &data, &dict);
    }

    assert!(checked > 0, "fixture should exercise fast levels");
}

#[test]
fn strategy_frame_dictionary_target_c_block_size_rejects_invalid_sizes() {
    let dict = full_dictionary_fixture();
    let data = dictionary_payload();

    let encoded =
        encode_frame_with_dictionary_and_target_c_block_size(&data, 1, &dict, 128 * 1024 + 1)
            .expect("dictionary parses");

    assert_eq!(encoded, None);
}

#[test]
fn strategy_frame_short_auto_dictionary_falls_back_to_no_dict_like_c() {
    let data = b"short-dict-fallback short-dict-fallback";
    let encoded = encode_frame_with_dictionary(data, 3, b"short").unwrap();
    let (header, _) = crate::decoding::frame::read_frame_header(encoded.as_slice())
        .expect("frame header should parse");

    assert_eq!(header.dictionary_id(), None);
    assert_round_trips(&encoded, data);
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

fn dictionary_payload() -> Vec<u8> {
    let mut data = Vec::new();
    for _ in 0..10 {
        data.extend_from_slice(dictionary_content());
    }
    data
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
