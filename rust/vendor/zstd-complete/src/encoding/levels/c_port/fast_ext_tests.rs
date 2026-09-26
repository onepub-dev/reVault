use super::{
    fast::{compress_block_fast_no_dict_with_state_and_loaded_dict, FastMatchState},
    fast_ext::compress_block_fast_ext_dict_with_state,
    params::{CParamMode, CompressionParameters},
    sequence_store::{OffBase, RepeatCode, RepeatOffsets},
};
use alloc::{vec, vec::Vec};

fn fast_params() -> CompressionParameters {
    CompressionParameters::for_level_with_mode(1, 32, 32, CParamMode::NoAttachDict)
}

#[test]
fn ext_dict_invalidates_repeat_equal_to_dictionary_size_like_c() {
    let dict = b"abcdefgh";
    let source = b"abcdefghabcdefgh";
    let mut combined = Vec::new();
    combined.extend_from_slice(dict);
    combined.extend_from_slice(source);

    let params = fast_params();
    let mut state = FastMatchState::new();
    state.load_prefix(&combined, dict.len(), params);
    let output = compress_block_fast_ext_dict_with_state(
        &combined,
        dict.len()..combined.len(),
        dict.len(),
        params,
        RepeatOffsets::from_offsets(dict.len() as u32, 3, 7),
        &mut state,
        dict.len(),
    );

    assert!(!output
        .sequences
        .first()
        .is_some_and(|seq| matches!(seq.off_base(), OffBase::Repeat(RepeatCode::First))));
}

#[test]
fn ext_dict_finds_dictionary_offset_match() {
    let dict = b"prefix:shared-payload";
    let source = b"shared-payload shared-payload";
    let mut combined = Vec::new();
    combined.extend_from_slice(dict);
    combined.extend_from_slice(source);

    let params = fast_params();
    let mut state = FastMatchState::new();
    state.load_prefix(&combined, dict.len(), params);
    let output = compress_block_fast_ext_dict_with_state(
        &combined,
        dict.len()..combined.len(),
        dict.len(),
        params,
        RepeatOffsets::new(),
        &mut state,
        dict.len(),
    );

    assert!(output
        .sequences
        .iter()
        .any(|seq| matches!(seq.off_base(), OffBase::Offset(_))));
}

#[test]
fn ext_dict_falls_back_to_regular_when_dictionary_window_expired_like_c() {
    let dict = vec![b'a'; 2048];
    let source = vec![b'a'; 2048];
    let mut combined = Vec::new();
    combined.extend_from_slice(&dict);
    combined.extend_from_slice(&source);

    let params = fast_params();
    let mut ext_state = FastMatchState::new();
    ext_state.load_prefix(&combined, dict.len(), params);
    let mut regular_state = ext_state.clone();
    let block_range = dict.len()..combined.len();

    let ext = compress_block_fast_ext_dict_with_state(
        &combined,
        block_range.clone(),
        dict.len(),
        params,
        RepeatOffsets::new(),
        &mut ext_state,
        0,
    );
    let regular = compress_block_fast_no_dict_with_state_and_loaded_dict(
        &combined,
        block_range,
        params,
        RepeatOffsets::new(),
        &mut regular_state,
        0,
    );

    assert_eq!(ext, regular);
}
