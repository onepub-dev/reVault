//! Optimal-parser binary-tree match collection ported from `zstd_opt.c`.

use super::{greedy::GreedyMatchState, params::CompressionParameters};

mod attached;
mod bounds;
mod repcodes;
mod request;
mod table;
mod tree;

pub(super) use bounds::{OptAttachedDictionary, OptMatchBounds};
pub(super) use repcodes::{
    collect_repcode_matches, collect_repcode_matches_no_dict, should_stop_after_best_match,
};
pub(super) use request::BtMatchRequest;
pub(super) use table::{OptMatch, OptMatchTable};

pub(super) fn bt_get_all_matches_no_dict(
    matches: &mut OptMatchTable,
    request: BtMatchRequest<'_>,
    state: &mut GreedyMatchState,
) {
    let params = request.params;
    let mls = params.min_match.clamp(3, 6);
    match mls {
        3 => bt_get_all_matches_mls::<3, false, false, false>(matches, request, state),
        4 => bt_get_all_matches_mls::<4, false, false, false>(matches, request, state),
        5 => bt_get_all_matches_mls::<5, false, false, false>(matches, request, state),
        6 => bt_get_all_matches_mls::<6, false, false, false>(matches, request, state),
        _ => unreachable!("mls is clamped to 3..=6"),
    }
}

#[inline(always)]
pub(super) fn bt_get_all_matches_no_dict_mls<const MLS: u32>(
    matches: &mut OptMatchTable,
    request: BtMatchRequest<'_>,
    state: &mut GreedyMatchState,
) {
    bt_get_all_matches_mls::<MLS, false, false, false>(matches, request, state);
}

#[inline(always)]
pub(super) fn bt_get_all_matches_mls<
    const MLS: u32,
    const EXT_DICT: bool,
    const LOADED_DICT: bool,
    const ATTACHED_DICT: bool,
>(
    matches: &mut OptMatchTable,
    request: BtMatchRequest<'_>,
    state: &mut GreedyMatchState,
) {
    matches.clear();
    if request.ip < state.next_to_update {
        return;
    }

    debug_assert_tables_ready(state, request.params);
    tree::bt_get_all_matches_mls::<MLS, EXT_DICT, LOADED_DICT, ATTACHED_DICT>(
        matches, request, state,
    );
}

pub(super) fn update_tree_no_dict(
    src: &[u8],
    target: usize,
    block_end: usize,
    mls: u32,
    params: CompressionParameters,
    state: &mut GreedyMatchState,
    loaded_dict_end: usize,
) {
    match (loaded_dict_end != 0, mls) {
        (false, 3) => tree::update_tree_no_dict_mls::<3, false>(
            src,
            target,
            block_end,
            params,
            state,
            loaded_dict_end,
        ),
        (false, 4) => tree::update_tree_no_dict_mls::<4, false>(
            src,
            target,
            block_end,
            params,
            state,
            loaded_dict_end,
        ),
        (false, 5) => tree::update_tree_no_dict_mls::<5, false>(
            src,
            target,
            block_end,
            params,
            state,
            loaded_dict_end,
        ),
        (false, 6) => tree::update_tree_no_dict_mls::<6, false>(
            src,
            target,
            block_end,
            params,
            state,
            loaded_dict_end,
        ),
        (true, 3) => tree::update_tree_no_dict_mls::<3, true>(
            src,
            target,
            block_end,
            params,
            state,
            loaded_dict_end,
        ),
        (true, 4) => tree::update_tree_no_dict_mls::<4, true>(
            src,
            target,
            block_end,
            params,
            state,
            loaded_dict_end,
        ),
        (true, 5) => tree::update_tree_no_dict_mls::<5, true>(
            src,
            target,
            block_end,
            params,
            state,
            loaded_dict_end,
        ),
        (true, 6) => tree::update_tree_no_dict_mls::<6, true>(
            src,
            target,
            block_end,
            params,
            state,
            loaded_dict_end,
        ),
        _ => unreachable!("mls is clamped to 3..=6"),
    }
}

#[cfg(debug_assertions)]
fn debug_assert_tables_ready(state: &GreedyMatchState, params: CompressionParameters) {
    debug_assert_eq!(state.hash_log, params.hash_log);
    debug_assert_eq!(state.chain_log, params.chain_log);
    debug_assert_eq!(state.hash_table.len(), 1_usize << params.hash_log);
    debug_assert_eq!(state.chain_table.len(), (1_usize << params.chain_log) + 1);

    let hash_log3 = if params.min_match == 3 {
        params.window_log.min(17)
    } else {
        0
    };
    debug_assert_eq!(state.hash_log3, hash_log3);
    let hash3_size = if hash_log3 > 0 {
        1_usize << hash_log3
    } else {
        0
    };
    debug_assert_eq!(state.hash_table3.len(), hash3_size);
}

#[cfg(not(debug_assertions))]
#[inline(always)]
fn debug_assert_tables_ready(_state: &GreedyMatchState, _params: CompressionParameters) {}
