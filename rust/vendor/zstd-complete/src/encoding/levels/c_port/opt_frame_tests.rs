use alloc::vec::Vec;

use super::{
    cctx_params::{CctxParameters, ParamSwitch},
    dictionary::{parse_dictionary, DictionaryContentType, ParsedDictionary},
    opt_frame::{
        encode_frame_btopt_with_dictionary, encode_frame_btultra2_with_dictionary,
        encode_frame_btultra_with_dictionary, encode_frame_opt_no_dict_with_cctx,
    },
    test_dictionary::{dictionary_content, full_dictionary_fixture, DICT_ID},
};
use crate::{
    blocks::block::BlockType,
    decoding::{dictionary::Dictionary, FrameDecoder},
};

#[test]
fn btopt_frame_with_dictionary_writes_dict_id_and_round_trips() {
    let encoded = encode_with_fixture_dictionary(16, encode_frame_btopt_with_dictionary);

    assert_dictionary_frame_round_trips(&encoded);
}

#[test]
fn btultra_frame_with_dictionary_writes_dict_id_and_round_trips() {
    let encoded = encode_with_fixture_dictionary(18, encode_frame_btultra_with_dictionary);

    assert_dictionary_frame_round_trips(&encoded);
}

#[test]
fn btultra2_frame_with_dictionary_writes_dict_id_and_round_trips() {
    let encoded = encode_with_fixture_dictionary(19, encode_frame_btultra2_with_dictionary);

    assert_dictionary_frame_round_trips(&encoded);
}

#[test]
fn btultra2_dictionary_path_uses_btultra_block_compressor_like_c() {
    let dict_bytes = full_dictionary_fixture();
    let parsed = parse_dictionary(&dict_bytes, DictionaryContentType::Auto, false)
        .unwrap()
        .expect("full dictionary");
    let data = dictionary_payload();

    let btultra = encode_frame_btultra_with_dictionary(&data, 19, parsed.clone());
    let btultra2 = encode_frame_btultra2_with_dictionary(&data, 19, parsed);

    assert_eq!(btultra2, btultra);
}

#[test]
fn target_c_block_size_uses_basic_superblock_when_sequence_block_fits() {
    let data = repeated_no_dict_payload();
    let mut target_cctx = CctxParameters::for_level(16, data.len() as u64, 0);
    target_cctx.post_block_splitter = ParamSwitch::Enable;
    assert!(target_cctx.set_target_c_block_size(2048));

    let target_encoded = encode_frame_opt_no_dict_with_cctx(&data, target_cctx);

    assert_eq!(
        first_frame_block_type(&target_encoded),
        BlockType::Compressed
    );
    assert_round_trips(&target_encoded, &data);
}

fn encode_with_fixture_dictionary(
    level: i32,
    encode: fn(&[u8], i32, ParsedDictionary<'_>) -> Vec<u8>,
) -> Vec<u8> {
    let dict_bytes = full_dictionary_fixture();
    let parsed = parse_dictionary(&dict_bytes, DictionaryContentType::Auto, false)
        .unwrap()
        .expect("full dictionary");
    let data = dictionary_payload();

    encode(&data, level, parsed)
}

fn assert_round_trips(encoded: &[u8], expected: &[u8]) {
    let mut decoded = Vec::with_capacity(expected.len());
    let mut decoder = FrameDecoder::new();
    decoder.decode_all_to_vec(encoded, &mut decoded).unwrap();

    assert_eq!(decoded, expected);
}

fn assert_dictionary_frame_round_trips(encoded: &[u8]) {
    let dict_bytes = full_dictionary_fixture();
    let data = dictionary_payload();
    let (header, _) =
        crate::decoding::frame::read_frame_header(encoded).expect("frame header should parse");

    assert_eq!(header.dictionary_id(), Some(DICT_ID));
    assert_eq!(first_frame_block_type(encoded), BlockType::Compressed);
    assert_round_trips_with_dictionary(encoded, &data, &dict_bytes);
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

fn first_frame_block_type(encoded: &[u8]) -> BlockType {
    let (_, frame_header_size) =
        crate::decoding::frame::read_frame_header(encoded).expect("frame header should parse");
    let mut block_decoder = crate::decoding::block_decoder::new();
    let (header, _) = block_decoder
        .read_block_header(&encoded[frame_header_size as usize..])
        .expect("block header should parse");

    header.block_type
}

fn dictionary_payload() -> Vec<u8> {
    let mut data = Vec::new();
    for _ in 0..10 {
        data.extend_from_slice(dictionary_content());
    }
    data
}

fn repeated_no_dict_payload() -> Vec<u8> {
    let mut data = Vec::new();
    for _ in 0..64 {
        data.extend_from_slice(b"target-c-block-size-opt-frame ");
    }
    data
}
