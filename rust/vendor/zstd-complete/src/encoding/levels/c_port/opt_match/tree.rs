//! Binary-tree update and match enumeration for the optimal parser.

use super::{
    attached::collect_attached_dictionary_matches, collect_repcode_matches,
    collect_repcode_matches_no_dict, should_stop_after_best_match, OptMatch, OptMatchBounds,
    OptMatchTable,
};
use crate::encoding::levels::c_port::{
    greedy::GreedyMatchState,
    hash_chain_match::{
        count_match_no_dict, hash3_ptr, hash_ptr_mls, lowest_prefix_index,
        lowest_prefix_index_with_loaded_dict,
    },
    params::CompressionParameters,
    sequence_store::OffBase,
};

const ZSTD_OPT_NUM: usize = 1 << 12;

#[inline(always)]
pub(super) fn bt_get_all_matches_mls<
    const MLS: u32,
    const EXT_DICT: bool,
    const LOADED_DICT: bool,
    const ATTACHED_DICT: bool,
>(
    matches: &mut OptMatchTable,
    request: super::BtMatchRequest<'_>,
    state: &mut GreedyMatchState,
) {
    let super::BtMatchRequest {
        src,
        ip,
        block_end,
        rep,
        ll0,
        length_to_beat,
        params,
        bounds,
        attached_dictionary,
    } = request;

    update_tree_no_dict_mls::<MLS, LOADED_DICT>(
        src,
        ip,
        block_end,
        params,
        state,
        bounds.loaded_dict_end,
    );
    insert_bt_and_get_all_matches::<MLS, EXT_DICT, LOADED_DICT, ATTACHED_DICT>(
        matches,
        src,
        ip,
        block_end,
        rep,
        ll0,
        length_to_beat,
        params,
        state,
        bounds,
        attached_dictionary,
    );
}

#[inline(always)]
pub(super) fn update_tree_no_dict_mls<const MLS: u32, const LOADED_DICT: bool>(
    src: &[u8],
    target: usize,
    block_end: usize,
    params: CompressionParameters,
    state: &mut GreedyMatchState,
    loaded_dict_end: usize,
) {
    let mut idx = state.next_to_update;
    while idx < target {
        let forward = insert_bt1_no_dict::<MLS, LOADED_DICT>(
            src,
            idx,
            block_end,
            target,
            state,
            params,
            loaded_dict_end,
        );
        debug_assert!(forward > 0);
        idx += forward;
    }
    state.next_to_update = target;
}

#[inline(always)]
fn insert_bt1_no_dict<const MLS: u32, const LOADED_DICT: bool>(
    src: &[u8],
    ip: usize,
    block_end: usize,
    target: usize,
    state: &mut GreedyMatchState,
    params: CompressionParameters,
    loaded_dict_end: usize,
) -> usize {
    let hash_table = &mut state.hash_table;
    let hash = hash_ptr_mls::<MLS>(src, ip, params.hash_log);
    let chain_table = &mut state.chain_table;
    let dummy_slot = chain_table.len() - 1;
    let mut match_index = hash_table[hash] as usize;
    let mask = bt_mask(chain_table);
    let bt_low = ip.saturating_sub(mask);
    let window_low = lowest_match_index::<LOADED_DICT>(target, params, loaded_dict_end);
    let mut common_smaller = 0_usize;
    let mut common_larger = 0_usize;
    let mut smaller_slot = tree_slot(ip, mask);
    let mut larger_slot = smaller_slot + 1;
    let mut match_end_idx = ip + 9;
    let mut best_length = 8_usize;
    let mut nb_compares = 1_usize << params.search_log;
    hash_table[hash] = stored_value(ip);

    while nb_compares > 0 && match_index > window_low {
        nb_compares -= 1;
        let current_match_index = match_index - 1;
        let next_slot = tree_slot(current_match_index, mask);

        let mut match_length = common_smaller.min(common_larger);
        match_length += count_match_no_dict(
            src,
            ip + match_length,
            current_match_index + match_length,
            block_end,
        );

        if match_length > best_length {
            best_length = match_length;
            if match_length > match_end_idx - current_match_index {
                match_end_idx = current_match_index + match_length;
            }
        }

        if ip + match_length == block_end {
            break;
        }

        let current_byte = src[ip + match_length];
        if src[current_match_index + match_length] < current_byte {
            chain_table[smaller_slot] = stored_value(current_match_index);
            common_smaller = match_length;
            if current_match_index <= bt_low {
                smaller_slot = dummy_slot;
                break;
            }
            smaller_slot = next_slot + 1;
            match_index = chain_table[next_slot + 1] as usize;
        } else {
            chain_table[larger_slot] = stored_value(current_match_index);
            common_larger = match_length;
            if current_match_index <= bt_low {
                larger_slot = dummy_slot;
                break;
            }
            larger_slot = next_slot;
            match_index = chain_table[next_slot] as usize;
        }
    }

    chain_table[smaller_slot] = 0;
    chain_table[larger_slot] = 0;

    let positions = best_length.saturating_sub(384).min(192);
    positions.max(match_end_idx - (ip + 8))
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn insert_bt_and_get_all_matches<
    const MLS: u32,
    const EXT_DICT: bool,
    const LOADED_DICT: bool,
    const ATTACHED_DICT: bool,
>(
    matches: &mut OptMatchTable,
    src: &[u8],
    ip: usize,
    block_end: usize,
    rep: [u32; 3],
    ll0: bool,
    length_to_beat: u32,
    params: CompressionParameters,
    state: &mut GreedyMatchState,
    bounds: OptMatchBounds,
    attached_dictionary: Option<
        crate::encoding::levels::c_port::hash_chain_match::AttachedDictionarySearch<'_>,
    >,
) {
    let sufficient_len = params.target_length.min((ZSTD_OPT_NUM - 1) as u32) as usize;
    let min_match = if MLS == 3 { 3 } else { 4 };
    let hash = hash_ptr_mls::<MLS>(src, ip, params.hash_log);
    let mut match_index = state.hash_table[hash] as usize;
    let mask = bt_mask(&state.chain_table);
    let bt_low = ip.saturating_sub(mask);
    let window_low = lowest_match_index::<LOADED_DICT>(ip, params, bounds.loaded_dict_end);
    let match_low = window_low;
    let mut common_smaller = 0_usize;
    let mut common_larger = 0_usize;
    let mut smaller_slot = tree_slot(ip, mask);
    let mut larger_slot = smaller_slot + 1;
    let mut match_end_idx = ip + 9;
    let mut best_length = length_to_beat.saturating_sub(1) as usize;
    let mut nb_compares = 1_usize << params.search_log;

    debug_assert_eq!(ATTACHED_DICT, attached_dictionary.is_some());
    if EXT_DICT || ATTACHED_DICT {
        debug_assert!(bounds.is_ext_dict() || ATTACHED_DICT);
        collect_repcode_matches(
            matches,
            src,
            ip,
            block_end,
            rep,
            ll0,
            min_match,
            bounds,
            window_low,
            sufficient_len,
            &mut best_length,
        );
    } else {
        debug_assert!(!bounds.is_ext_dict());
        collect_repcode_matches_no_dict(
            matches,
            src,
            ip,
            block_end,
            rep,
            ll0,
            min_match,
            window_low,
            sufficient_len,
            &mut best_length,
        );
    }
    if should_stop_after_best_match(matches, ip, block_end, sufficient_len) {
        return;
    }

    if MLS == 3 && best_length < 3 {
        if let Some(match_index3) = insert_and_find_first_index_hash3(src, ip, state) {
            let within_price_heuristic = ip - match_index3 < (1 << 18);
            if (match_index3 >= match_low) & within_price_heuristic {
                let len = count_match_no_dict(src, ip, match_index3, block_end);
                if len >= 3 {
                    best_length = len;
                    matches.clear();
                    matches.push(OptMatch {
                        off_base: OffBase::offset_to_c_value((ip - match_index3) as u32),
                        len: len as u32,
                    });
                    if (len > sufficient_len) | (ip + len == block_end) {
                        state.next_to_update = ip + 1;
                        return;
                    }
                }
            }
        }
    }

    let hash_table = &mut state.hash_table;
    let chain_table = &mut state.chain_table;
    let dummy_slot = chain_table.len() - 1;
    hash_table[hash] = stored_value(ip);

    while nb_compares > 0 && match_index > match_low {
        nb_compares -= 1;
        let current_match_index = match_index - 1;
        let next_slot = tree_slot(current_match_index, mask);
        let mut match_length = common_smaller.min(common_larger);
        match_length += count_match_no_dict(
            src,
            ip + match_length,
            current_match_index + match_length,
            block_end,
        );

        if match_length > best_length {
            if match_length > match_end_idx - current_match_index {
                match_end_idx = current_match_index + match_length;
            }
            best_length = match_length;
            matches.push(OptMatch {
                off_base: OffBase::offset_to_c_value((ip - current_match_index) as u32),
                len: match_length as u32,
            });
            if (match_length > ZSTD_OPT_NUM) | (ip + match_length == block_end) {
                if ATTACHED_DICT {
                    nb_compares = 0;
                }
                break;
            }
        }

        let current_byte = src[ip + match_length];
        if src[current_match_index + match_length] < current_byte {
            chain_table[smaller_slot] = stored_value(current_match_index);
            common_smaller = match_length;
            if current_match_index <= bt_low {
                smaller_slot = dummy_slot;
                break;
            }
            smaller_slot = next_slot + 1;
            match_index = chain_table[next_slot + 1] as usize;
        } else {
            chain_table[larger_slot] = stored_value(current_match_index);
            common_larger = match_length;
            if current_match_index <= bt_low {
                larger_slot = dummy_slot;
                break;
            }
            larger_slot = next_slot;
            match_index = chain_table[next_slot] as usize;
        }
    }

    chain_table[smaller_slot] = 0;
    chain_table[larger_slot] = 0;
    if ATTACHED_DICT {
        let attached = attached_dictionary
            .expect("attached dictionary specialization requires attached search state");
        collect_attached_dictionary_matches::<MLS>(
            matches,
            src,
            ip,
            block_end,
            &mut best_length,
            nb_compares,
            &mut match_end_idx,
            attached,
        );
    }
    state.next_to_update = match_end_idx - 8;
}

fn insert_and_find_first_index_hash3(
    src: &[u8],
    ip: usize,
    state: &mut GreedyMatchState,
) -> Option<usize> {
    if state.hash_log3 == 0 {
        return None;
    }

    let mut idx = state.next_to_update3;
    while idx < ip {
        state.hash_table3[hash3_ptr(src, idx, state.hash_log3)] = stored_value(idx);
        idx += 1;
    }

    state.next_to_update3 = ip;
    stored_index(state.hash_table3[hash3_ptr(src, ip, state.hash_log3)])
}

#[inline(always)]
fn bt_mask(chain_table: &[u32]) -> usize {
    let tree_slots = chain_table.len() - 1;
    debug_assert!(tree_slots.is_power_of_two());
    (tree_slots >> 1) - 1
}

#[inline(always)]
fn tree_slot(index: usize, mask: usize) -> usize {
    2 * (index & mask)
}

#[inline(always)]
fn lowest_match_index<const LOADED_DICT: bool>(
    pos: usize,
    params: CompressionParameters,
    loaded_dict_end: usize,
) -> usize {
    if LOADED_DICT {
        lowest_prefix_index_with_loaded_dict(pos, params.window_log, loaded_dict_end)
    } else {
        debug_assert_eq!(loaded_dict_end, 0);
        lowest_prefix_index(pos, params.window_log)
    }
}

#[inline(always)]
fn stored_value(index: usize) -> u32 {
    debug_assert!(index < u32::MAX as usize);
    index as u32 + 1
}

#[inline(always)]
fn stored_index(value: u32) -> Option<usize> {
    value.checked_sub(1).map(|index| index as usize)
}
