use super::{
    ldm::{opt::LdmOptCursor, sequence::LdmRawSequence},
    opt_block::{compress_block_btopt_no_dict, compress_block_btultra_no_dict},
    opt_match::{OptMatch, OptMatchBounds},
    opt_parser::{
        collect_matches, compress_block_opt_ext_dict_with_state_and_ldm,
        compress_block_opt_no_dict_with_state_and_ldm,
    },
    opt_state::{OptBlockState, OptParserStrategy},
    params::{CompressionParameters, Strategy},
    sequence_store::{OffBase, RepeatOffsets},
};
use alloc::vec::Vec;

#[test]
fn btopt_parser_emits_sequences_for_repeated_data() {
    let data = b"alpha beta gamma alpha beta gamma alpha beta gamma";
    let params = btopt_params(data.len());

    let output = compress_block_btopt_no_dict(data, params, RepeatOffsets::new());

    assert!(!output.sequences.is_empty());
    assert!(output.last_literals < data.len() as u32);
}

#[test]
fn btultra_parser_emits_sequences_for_repeated_data() {
    let data = b"alpha beta gamma alpha beta gamma alpha beta gamma";
    let params = btultra_params(data.len());

    let output = compress_block_btultra_no_dict(data, params, RepeatOffsets::new());
    let covered = output
        .sequences
        .iter()
        .map(|sequence| sequence.lit_len + sequence.match_len)
        .sum::<u32>()
        + output.last_literals;

    assert_eq!(covered, data.len() as u32);
}

#[test]
fn btopt_parser_round_trips_sequence_coverage() {
    let data = b"GET /index.html 200 GET /index.html 200 GET /index.html 200";
    let params = btopt_params(data.len());
    let output = compress_block_btopt_no_dict(data, params, RepeatOffsets::new());

    let covered = output
        .sequences
        .iter()
        .map(|sequence| sequence.lit_len + sequence.match_len)
        .sum::<u32>()
        + output.last_literals;

    assert_eq!(covered, data.len() as u32);
}

#[test]
fn btopt_parser_state_spans_blocks() {
    let data = b"first-block-shared-payload first-block-shared-payload second-block-shared-payload first-block-shared-payload";
    let params = btopt_params(data.len());
    let split = data.len() / 2;
    let mut state = OptBlockState::new();

    let first = super::opt_block::compress_block_btopt_no_dict_with_state(
        data,
        0..split,
        params,
        RepeatOffsets::new(),
        &mut state,
    );
    let second = super::opt_block::compress_block_btopt_no_dict_with_state(
        data,
        split..data.len(),
        params,
        first.repeat_offsets,
        &mut state,
    );

    assert_eq!(
        first
            .sequences
            .iter()
            .map(|sequence| sequence.lit_len + sequence.match_len)
            .sum::<u32>()
            + first.last_literals,
        split as u32
    );
    assert_eq!(
        second
            .sequences
            .iter()
            .map(|sequence| sequence.lit_len + sequence.match_len)
            .sum::<u32>()
            + second.last_literals,
        (data.len() - split) as u32
    );
}

#[test]
fn btopt_parser_reuses_path_scratch_without_stale_entries() {
    let data = b"scratch-path-reuse scratch-path-reuse scratch-path-reuse";
    let params = btopt_params(data.len());
    let mut state = OptBlockState::new();
    state.path.reserve(32);
    let capacity = state.path.capacity();

    let output = super::opt_block::compress_block_btopt_no_dict_with_state(
        data,
        0..data.len(),
        params,
        RepeatOffsets::new(),
        &mut state,
    );

    assert!(!output.sequences.is_empty());
    assert_eq!(state.path.len(), 0);
    assert!(state.path.capacity() >= capacity);
}

#[test]
fn btultra_parser_emits_matches_for_structured_data() {
    let mut data = Vec::new();
    for i in 0..2048u32 {
        use core::fmt::Write;
        let mut line = alloc::string::String::new();
        writeln!(
            &mut line,
            "route=/v1/pkg/{:04} status={:03} bytes={:04} tag={}{}{}{}",
            i % 97,
            200 + (i % 17),
            1000 + (i % 311),
            char::from_u32(65 + (i % 7)).unwrap(),
            char::from_u32(65 + ((i + 1) % 11)).unwrap(),
            char::from_u32(65 + ((i + 2) % 13)).unwrap(),
            char::from_u32(65 + ((i + 3) % 17)).unwrap(),
        )
        .unwrap();
        data.extend_from_slice(line.as_bytes());
    }

    let params = CompressionParameters::for_level(16, data.len() as u64, 0);
    let mut state = OptBlockState::new();
    state.reset_for_frame(params);
    let output = super::opt_block::compress_block_btultra_no_dict_with_state(
        &data,
        0..data.len(),
        params,
        RepeatOffsets::new(),
        &mut state,
    );
    let match_bytes = output
        .sequences
        .iter()
        .map(|sequence| sequence.match_len)
        .sum::<u32>();
    let covered = output
        .sequences
        .iter()
        .map(|sequence| sequence.lit_len + sequence.match_len)
        .sum::<u32>()
        + output.last_literals;

    assert_eq!(covered, data.len() as u32);
    assert!(!output.sequences.is_empty());
    assert!(match_bytes > data.len() as u32 / 2);
}

#[test]
fn opt_match_collection_appends_ldm_candidate_like_c() {
    let data = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let params = btopt_params(data.len());
    let mut state = OptBlockState::new();
    state.reset_for_frame(params);
    state.match_state.ensure_tables(params);
    let sequences = [LdmRawSequence {
        offset: 4,
        lit_length: 4,
        match_length: 10,
    }];
    let mut cursor = LdmOptCursor::new(&sequences, data.len() as u32);

    let count = collect_matches(
        data,
        4,
        data.len(),
        RepeatOffsets::new().as_offsets(),
        false,
        4,
        params,
        &mut state,
        0,
        Some(&mut cursor),
        OptMatchBounds::no_dict(data.len(), params, 0),
    );

    assert_eq!(count, 1);
    assert_eq!(
        state.matches.as_slice(),
        [OptMatch {
            off_base: 7,
            len: 10,
        }]
    );
}

#[test]
fn btopt_loaded_dictionary_keeps_full_dictionary_valid_like_c() {
    let marker = b"early-opt-dictionary-marker-0123456789abcdef";
    let mut dictionary = deterministic_bytes(2048);
    dictionary[30..30 + marker.len()].copy_from_slice(marker);
    let source = [marker.as_slice(), b"-payload-tail".as_slice()].concat();
    let mut combined = dictionary.clone();
    combined.extend_from_slice(&source);
    let params = loaded_dictionary_params();
    let block_range = dictionary.len()..combined.len();

    let mut no_loaded_state = OptBlockState::new();
    super::opt_dict::load_prefix(&mut no_loaded_state, &combined, dictionary.len(), params);
    let no_loaded = super::opt_block::compress_block_btopt_no_dict_with_state(
        &combined,
        block_range.clone(),
        params,
        RepeatOffsets::new(),
        &mut no_loaded_state,
    );

    let mut loaded_state = OptBlockState::new();
    super::opt_dict::load_prefix(&mut loaded_state, &combined, dictionary.len(), params);
    let loaded = compress_block_opt_no_dict_with_state_and_ldm(
        &combined,
        block_range,
        params,
        RepeatOffsets::new(),
        &mut loaded_state,
        OptParserStrategy::BtOpt,
        None,
        dictionary.len(),
    );

    assert!(!has_loaded_dictionary_match(&no_loaded, params));
    assert!(has_loaded_dictionary_match(&loaded, params));
}

#[test]
fn btopt_ext_dict_rejects_overlap_repcode_like_c() {
    let dict = b"abcdefgh";
    let source = b"XfghXabcdefgh";
    let mut combined = Vec::new();
    combined.extend_from_slice(dict);
    combined.extend_from_slice(source);

    let params = loaded_dictionary_params();
    let mut state = OptBlockState::new();
    super::opt_dict::load_prefix(&mut state, &combined, dict.len(), params);
    let output = compress_block_opt_ext_dict_with_state_and_ldm(
        &combined,
        dict.len()..combined.len(),
        dict.len(),
        params,
        RepeatOffsets::from_offsets(4, 3, 7),
        &mut state,
        OptParserStrategy::BtOpt,
        None,
        dict.len(),
    );

    assert!(!output
        .sequences
        .first()
        .is_some_and(|seq| matches!(seq.off_base(), OffBase::Repeat(_))));
}

#[test]
fn btopt_ext_dict_finds_dictionary_offset_match() {
    let dict = b"prefix:shared-payload";
    let source = b"shared-payload shared-payload";
    let mut combined = Vec::new();
    combined.extend_from_slice(dict);
    combined.extend_from_slice(source);

    let params = loaded_dictionary_params();
    let mut state = OptBlockState::new();
    super::opt_dict::load_prefix(&mut state, &combined, dict.len(), params);
    let output = compress_block_opt_ext_dict_with_state_and_ldm(
        &combined,
        dict.len()..combined.len(),
        dict.len(),
        params,
        RepeatOffsets::new(),
        &mut state,
        OptParserStrategy::BtOpt,
        None,
        dict.len(),
    );

    assert!(output
        .sequences
        .iter()
        .any(|seq| matches!(seq.off_base(), OffBase::Offset(_))));
}

fn btopt_params(src_size: usize) -> CompressionParameters {
    let params = CompressionParameters::for_level(11, src_size as u64, 0);
    assert_eq!(params.strategy, Strategy::BtOpt);
    params
}

fn loaded_dictionary_params() -> CompressionParameters {
    CompressionParameters {
        window_log: 10,
        chain_log: 13,
        hash_log: 12,
        search_log: 4,
        min_match: 4,
        target_length: 16,
        strategy: Strategy::BtOpt,
    }
}

fn has_loaded_dictionary_match(
    output: &super::greedy::GreedyBlockOutput,
    params: CompressionParameters,
) -> bool {
    output.sequences.iter().any(|sequence| {
        matches!(
            sequence.off_base(),
            OffBase::Offset(offset) if offset as usize > (1_usize << params.window_log)
        )
    })
}

fn deterministic_bytes(len: usize) -> Vec<u8> {
    let mut state = 0x50B7_0C4D_u32;
    let mut bytes = Vec::with_capacity(len);
    for _ in 0..len {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        bytes.push(state as u8);
    }
    bytes
}

fn btultra_params(src_size: usize) -> CompressionParameters {
    let params = CompressionParameters::for_level(13, src_size as u64, 0);
    assert_eq!(params.strategy, Strategy::BtUltra);
    params
}
