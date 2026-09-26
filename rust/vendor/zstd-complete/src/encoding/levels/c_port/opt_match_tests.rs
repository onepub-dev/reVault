use super::{
    bt_match::load_attached_dictionary_binary_tree,
    greedy::GreedyMatchState,
    opt_match::{
        bt_get_all_matches_no_dict, BtMatchRequest, OptAttachedDictionary, OptMatch,
        OptMatchBounds, OptMatchTable,
    },
    params::{CompressionParameters, Strategy},
    sequence_store::{OffBase, RepeatCode, RepeatOffsets},
};

fn params_for_min_match(min_match: u32) -> CompressionParameters {
    CompressionParameters {
        window_log: 18,
        chain_log: 15,
        hash_log: 15,
        search_log: 4,
        min_match,
        target_length: 16,
        strategy: Strategy::BtOpt,
    }
}

#[test]
fn opt_match_collector_reports_repcodes_before_tree_matches() {
    let data = b"abcabcabcxyz";
    let params = params_for_min_match(4);
    let mut state = GreedyMatchState::new();
    state.ensure_tables(params);
    let mut matches = OptMatchTable::new();

    bt_get_all_matches_no_dict(
        &mut matches,
        request(
            data,
            3,
            RepeatOffsets::from_offsets(3, 4, 8),
            false,
            4,
            params,
        ),
        &mut state,
    );

    assert_eq!(
        matches.as_slice(),
        [OptMatch {
            off_base: OffBase::Repeat(RepeatCode::First).to_c_value(),
            len: 6,
        }]
    );
}

#[test]
fn opt_match_collector_reports_increasing_tree_matches() {
    let data = b"xabcdefghijabcdefghij-tail";
    let params = params_for_min_match(4);
    let mut state = GreedyMatchState::new();
    state.ensure_tables(params);
    let mut matches = OptMatchTable::new();

    bt_get_all_matches_no_dict(
        &mut matches,
        request(data, 11, RepeatOffsets::new(), false, 4, params),
        &mut state,
    );

    assert_eq!(
        matches.last(),
        Some(&OptMatch {
            off_base: OffBase::Offset(10).to_c_value(),
            len: 10,
        })
    );
}

#[test]
fn opt_match_collector_can_match_source_index_zero() {
    let data = b"abcd-----------------------abcd-tail";
    let params = params_for_min_match(4);
    let mut state = GreedyMatchState::new();
    state.ensure_tables(params);
    let mut matches = OptMatchTable::new();

    bt_get_all_matches_no_dict(
        &mut matches,
        request(data, 27, RepeatOffsets::new(), false, 4, params),
        &mut state,
    );

    assert_eq!(
        matches.last(),
        Some(&OptMatch {
            off_base: OffBase::Offset(27).to_c_value(),
            len: 5,
        })
    );
}

#[test]
fn opt_match_collector_uses_hash3_for_min_match_three() {
    let data = b"xabc---abcXYZ";
    let params = params_for_min_match(3);
    let mut state = GreedyMatchState::new();
    state.ensure_tables(params);
    let mut matches = OptMatchTable::new();

    bt_get_all_matches_no_dict(
        &mut matches,
        request(data, 7, RepeatOffsets::new(), false, 3, params),
        &mut state,
    );

    assert_eq!(
        matches.as_slice(),
        [OptMatch {
            off_base: OffBase::Offset(6).to_c_value(),
            len: 3,
        }]
    );
}

#[test]
fn opt_match_collector_enumerates_attached_dictionary_tree_matches() {
    const INDEX_BASE: usize = 2;

    let dictionary = b"prefix-abcdefghij-dictionary-padding";
    let mut combined = dictionary.to_vec();
    combined.extend_from_slice(b"xabcdefghij-tail");
    let params = params_for_min_match(4);

    let mut dictionary_state = GreedyMatchState::new();
    dictionary_state.ensure_tables(params);
    dictionary_state.next_to_update = INDEX_BASE;
    load_attached_dictionary_binary_tree(
        dictionary,
        dictionary.len() - 8,
        INDEX_BASE,
        params,
        params.min_match,
        &mut dictionary_state,
    );
    dictionary_state.next_to_update = dictionary.len() + INDEX_BASE;

    let mut active_state = GreedyMatchState::new();
    active_state.ensure_tables(params);
    active_state.next_to_update = dictionary.len();
    let metadata = OptAttachedDictionary::new(
        0,
        dictionary.len(),
        params,
        INDEX_BASE,
        dictionary.len() + INDEX_BASE,
        dictionary.len(),
    );
    let bounds = OptMatchBounds::attached(combined.len(), metadata);
    let ip = dictionary.len() + 1;
    let mut matches = OptMatchTable::new();

    super::opt_match::bt_get_all_matches_mls::<4, false, false, true>(
        &mut matches,
        BtMatchRequest {
            src: &combined,
            ip,
            block_end: combined.len(),
            rep: RepeatOffsets::new().as_offsets(),
            ll0: false,
            length_to_beat: 4,
            params,
            bounds,
            attached_dictionary: Some(metadata.search(&combined, &dictionary_state)),
        },
        &mut active_state,
    );

    assert_eq!(
        matches.last(),
        Some(&OptMatch {
            off_base: OffBase::Offset((ip - 7) as u32).to_c_value(),
            len: 11,
        })
    );
}

fn request<'a>(
    src: &'a [u8],
    ip: usize,
    rep: RepeatOffsets,
    ll0: bool,
    length_to_beat: u32,
    params: CompressionParameters,
) -> BtMatchRequest<'a> {
    BtMatchRequest {
        src,
        ip,
        block_end: src.len(),
        rep: rep.as_offsets(),
        ll0,
        length_to_beat,
        params,
        bounds: OptMatchBounds::no_dict(src.len(), params, 0),
        attached_dictionary: None,
    }
}
