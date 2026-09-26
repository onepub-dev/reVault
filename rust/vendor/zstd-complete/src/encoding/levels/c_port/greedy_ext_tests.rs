use alloc::vec::Vec;

use super::{
    bt_match::load_attached_dictionary_binary_tree,
    greedy::{GreedyBlockOutput, GreedyMatchState},
    greedy_dict::{load_binary_tree_prefix, load_prefix},
    greedy_ext::{
        compress_block_attached_row_dict_with_state,
        compress_block_btlazy2_attached_dict_with_state,
        compress_block_btlazy2_ext_dict_with_state,
        compress_block_greedy_attached_row_dict_with_state,
        compress_block_greedy_ext_dict_with_state, compress_block_lazy2_ext_dict_with_state,
        compress_block_lazy_ext_dict_with_state,
    },
    params::{CompressionParameters, Strategy},
    row_match::load_dictionary_rows,
    sequence_store::{OffBase, RepeatCode, RepeatOffsets},
};

fn chain_params(strategy: Strategy) -> CompressionParameters {
    CompressionParameters {
        window_log: 14,
        chain_log: 13,
        hash_log: 15,
        search_log: 4,
        min_match: 4,
        target_length: 0,
        strategy,
    }
}

fn btlazy2_params() -> CompressionParameters {
    CompressionParameters {
        window_log: 17,
        chain_log: 18,
        hash_log: 17,
        search_log: 5,
        min_match: 4,
        target_length: 8,
        strategy: Strategy::BtLazy2,
    }
}

fn row_params() -> CompressionParameters {
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

#[test]
fn greedy_ext_dict_rejects_overlap_repcode_like_c() {
    let dict = b"abcdefgh";
    let source = b"XfghXabcdefgh";
    let mut combined = Vec::new();
    combined.extend_from_slice(dict);
    combined.extend_from_slice(source);

    let params = chain_params(Strategy::Greedy);
    let mut state = GreedyMatchState::new();
    load_prefix(&mut state, &combined, dict.len(), params);
    let output = compress_block_greedy_ext_dict_with_state(
        &combined,
        dict.len()..combined.len(),
        dict.len(),
        params,
        RepeatOffsets::from_offsets(4, 3, 7),
        &mut state,
        dict.len(),
    );

    assert!(!output
        .sequences
        .first()
        .is_some_and(|seq| matches!(seq.off_base(), OffBase::Repeat(RepeatCode::First))));
}

#[test]
fn greedy_ext_dict_finds_dictionary_offset_match() {
    let output = run_dictionary_match(Strategy::Greedy);

    assert_has_offset_match(&output);
}

#[test]
fn lazy_ext_dict_finds_dictionary_offset_match() {
    let output = run_dictionary_match(Strategy::Lazy);

    assert_has_offset_match(&output);
}

#[test]
fn lazy2_ext_dict_finds_dictionary_offset_match() {
    let output = run_dictionary_match(Strategy::Lazy2);

    assert_has_offset_match(&output);
}

#[test]
fn greedy_attached_row_dict_finds_dictionary_offset_match() {
    let dict = b"prefix:shared-payload";
    let source = b"shared-payload shared-payload";
    let mut combined = Vec::new();
    combined.extend_from_slice(dict);
    combined.extend_from_slice(source);

    let params = row_params();
    let mut dictionary_state = GreedyMatchState::new();
    dictionary_state.ensure_tables(params);
    load_dictionary_rows(dict, dict.len() - 8, params, 4, &mut dictionary_state);

    let mut active_state = GreedyMatchState::new();
    active_state.ensure_tables(params);
    active_state.next_to_update = dict.len();

    let output = compress_block_greedy_attached_row_dict_with_state(
        &combined,
        dict.len()..combined.len(),
        dict.len(),
        dict.len(),
        params,
        RepeatOffsets::new(),
        &mut active_state,
        dict,
        &dictionary_state,
        params,
        0,
    );

    assert_has_offset_match(&output);
}

#[test]
fn lazy_attached_row_dict_finds_dictionary_offset_match() {
    assert_has_offset_match(&run_attached_row_dictionary_match(Strategy::Lazy, 1));
}

#[test]
fn lazy2_attached_row_dict_finds_dictionary_offset_match() {
    assert_has_offset_match(&run_attached_row_dictionary_match(Strategy::Lazy2, 2));
}

#[test]
fn btlazy2_attached_dict_finds_dictionary_offset_match() {
    const INDEX_BASE: usize = 2;
    let dict = b"prefix:shared-payload-prefix:shared-payload";
    let source = b"shared-payload shared-payload";
    let mut combined = Vec::new();
    combined.extend_from_slice(dict);
    combined.extend_from_slice(source);

    let params = btlazy2_params();
    let mut dictionary_state = GreedyMatchState::new();
    dictionary_state.ensure_tables(params);
    dictionary_state.next_to_update = INDEX_BASE;
    load_attached_dictionary_binary_tree(
        dict,
        dict.len() - 8,
        INDEX_BASE,
        params,
        params.min_match,
        &mut dictionary_state,
    );
    dictionary_state.next_to_update = dict.len() + INDEX_BASE;

    let mut active_state = GreedyMatchState::new();
    active_state.ensure_tables(params);
    active_state.next_to_update = dict.len();

    let output = compress_block_btlazy2_attached_dict_with_state(
        &combined,
        dict.len()..combined.len(),
        dict.len() + INDEX_BASE,
        dict.len(),
        params,
        RepeatOffsets::new(),
        &mut active_state,
        dict,
        &dictionary_state,
        params,
        INDEX_BASE,
    );

    assert_has_offset_match(&output);
}

fn run_attached_row_dictionary_match(strategy: Strategy, depth: u32) -> GreedyBlockOutput {
    let dict = b"prefix:shared-payload";
    let source = b"shared-payload shared-payload";
    let mut combined = Vec::new();
    combined.extend_from_slice(dict);
    combined.extend_from_slice(source);

    let params = CompressionParameters {
        strategy,
        ..row_params()
    };
    let mut dictionary_state = GreedyMatchState::new();
    dictionary_state.ensure_tables(params);
    load_dictionary_rows(dict, dict.len() - 8, params, 4, &mut dictionary_state);

    let mut active_state = GreedyMatchState::new();
    active_state.ensure_tables(params);
    active_state.next_to_update = dict.len();

    macro_rules! compress_at_depth {
        ($depth:expr) => {
            compress_block_attached_row_dict_with_state::<$depth>(
                &combined,
                dict.len()..combined.len(),
                dict.len(),
                dict.len(),
                params,
                RepeatOffsets::new(),
                &mut active_state,
                dict,
                &dictionary_state,
                params,
                0,
            )
        };
    }

    match depth {
        0 => compress_at_depth!(0),
        1 => compress_at_depth!(1),
        2 => compress_at_depth!(2),
        _ => unreachable!("test depth is greedy, lazy, or lazy2"),
    }
}

#[test]
fn btlazy2_ext_dict_finds_dictionary_offset_match() {
    let dict = b"prefix:shared-payload";
    let source = b"shared-payload shared-payload";
    let mut combined = Vec::new();
    combined.extend_from_slice(dict);
    combined.extend_from_slice(source);

    let params = btlazy2_params();
    let mut state = GreedyMatchState::new();
    load_binary_tree_prefix(&mut state, &combined, dict.len(), params);
    let output = compress_block_btlazy2_ext_dict_with_state(
        &combined,
        dict.len()..combined.len(),
        dict.len(),
        params,
        RepeatOffsets::new(),
        &mut state,
        dict.len(),
    );

    assert_has_offset_match(&output);
}

fn run_dictionary_match(strategy: Strategy) -> GreedyBlockOutput {
    let dict = b"prefix:shared-payload";
    let source = b"shared-payload shared-payload";
    let mut combined = Vec::new();
    combined.extend_from_slice(dict);
    combined.extend_from_slice(source);

    let params = chain_params(strategy);
    let mut state = GreedyMatchState::new();
    load_prefix(&mut state, &combined, dict.len(), params);
    match strategy {
        Strategy::Greedy => compress_block_greedy_ext_dict_with_state(
            &combined,
            dict.len()..combined.len(),
            dict.len(),
            params,
            RepeatOffsets::new(),
            &mut state,
            dict.len(),
        ),
        Strategy::Lazy => compress_block_lazy_ext_dict_with_state(
            &combined,
            dict.len()..combined.len(),
            dict.len(),
            params,
            RepeatOffsets::new(),
            &mut state,
            dict.len(),
        ),
        Strategy::Lazy2 => compress_block_lazy2_ext_dict_with_state(
            &combined,
            dict.len()..combined.len(),
            dict.len(),
            params,
            RepeatOffsets::new(),
            &mut state,
            dict.len(),
        ),
        _ => unreachable!("test only covers hash-chain lazy strategies"),
    }
}

fn assert_has_offset_match(output: &GreedyBlockOutput) {
    assert!(output
        .sequences
        .iter()
        .any(|seq| matches!(seq.off_base(), OffBase::Offset(_))));
}
