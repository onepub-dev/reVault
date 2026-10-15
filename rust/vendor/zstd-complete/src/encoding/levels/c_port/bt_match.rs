//! Binary-tree match finder ported from the no-dictionary DUBT path.

mod attached;

pub(super) use attached::load_attached_dictionary_binary_tree;

use super::{
    greedy::GreedyMatchState,
    hash_chain_match::{count_match, hash_ptr, highbit32, MatchSearchConfig},
    params::CompressionParameters,
    sequence_store::OffBase,
};

const DUBT_UNSORTED_MARK: u32 = 1;

pub(super) fn bt_find_best_match<const ATTACHED_DICT: bool>(
    src: &[u8],
    ip: usize,
    block_end: usize,
    off_base: &mut u32,
    state: &mut GreedyMatchState,
    config: MatchSearchConfig<'_>,
) -> usize {
    if ip < state.next_to_update {
        return 0;
    }

    update_dubt(src, ip, config.params, config.min_match, state);
    dubt_find_best_match::<ATTACHED_DICT>(src, ip, block_end, off_base, state, config)
}

pub(super) fn load_dictionary_binary_tree(
    src: &[u8],
    target: usize,
    block_end: usize,
    params: CompressionParameters,
    min_match: u32,
    state: &mut GreedyMatchState,
) {
    update_dubt(src, target, params, min_match, state);
    debug_assert!(state.next_to_update <= block_end);
}

fn update_dubt(
    src: &[u8],
    target: usize,
    params: CompressionParameters,
    min_match: u32,
    state: &mut GreedyMatchState,
) {
    let bt_mask = bt_mask(params);
    let mut idx = state.next_to_update;

    while idx < target {
        let hash = hash_ptr(src, idx, params.hash_log, min_match);
        let match_index = state.hash_table[hash];
        let next_slot = 2 * (idx & bt_mask);

        state.hash_table[hash] = idx as u32;
        state.chain_table[next_slot] = match_index;
        state.chain_table[next_slot + 1] = DUBT_UNSORTED_MARK;
        idx += 1;
    }

    state.next_to_update = target;
}

fn dubt_find_best_match<const ATTACHED_DICT: bool>(
    src: &[u8],
    ip: usize,
    block_end: usize,
    off_base: &mut u32,
    state: &mut GreedyMatchState,
    config: MatchSearchConfig<'_>,
) -> usize {
    let params = config.params;
    let hash = hash_ptr(src, ip, params.hash_log, config.min_match);
    let mut match_index = state.hash_table[hash] as usize;
    let curr = ip;
    let window_low = config.lowest_prefix_index(curr);
    let mask = bt_mask(params);
    let bt_low = curr.saturating_sub(mask);
    let unsort_limit = bt_low.max(window_low);
    let mut nb_compares = 1_usize << params.search_log;
    let mut nb_candidates = nb_compares;
    let mut previous_candidate = 0_usize;

    while match_index > unsort_limit
        && tree_value(state, match_index, mask, 1) == DUBT_UNSORTED_MARK
        && nb_candidates > 1
    {
        set_tree_value(state, match_index, mask, 1, previous_candidate as u32);
        previous_candidate = match_index;
        match_index = tree_value(state, match_index, mask, 0) as usize;
        nb_candidates -= 1;
    }

    if match_index > unsort_limit && tree_value(state, match_index, mask, 1) == DUBT_UNSORTED_MARK {
        set_tree_value(state, match_index, mask, 0, 0);
        set_tree_value(state, match_index, mask, 1, 0);
    }

    match_index = previous_candidate;
    while match_index != 0 {
        let next_index = tree_value(state, match_index, mask, 1) as usize;
        insert_dubt1(
            src,
            match_index,
            block_end,
            nb_candidates,
            unsort_limit,
            state,
            config,
        );
        match_index = next_index;
        nb_candidates += 1;
    }

    let mut common_smaller = 0_usize;
    let mut common_larger = 0_usize;
    let mut smaller_slot = Some(tree_slot(curr, mask));
    let mut larger_slot = Some(tree_slot(curr, mask) + 1);
    let mut match_end_idx = curr + 9;
    let mut best_length = 0_usize;

    match_index = state.hash_table[hash] as usize;
    state.hash_table[hash] = curr as u32;

    while nb_compares > 0 && match_index > window_low {
        nb_compares -= 1;
        let next_slot = tree_slot(match_index, mask);
        let mut match_length = common_smaller.min(common_larger);
        match_length += count_match(
            src,
            ip + match_length,
            match_index + match_length,
            block_end,
        );

        if match_length > best_length {
            if match_length > match_end_idx - match_index {
                match_end_idx = match_index + match_length;
            }
            let gain_delta =
                highbit32((curr - match_index + 1) as u32) as i32 - highbit32(*off_base) as i32;
            if (4 * (match_length - best_length)) as i32 > gain_delta {
                best_length = match_length;
                *off_base = OffBase::offset_to_c_value((curr - match_index) as u32);
            }
            if ip + match_length == block_end {
                if ATTACHED_DICT {
                    nb_compares = 0;
                }
                break;
            }
        }

        if src[match_index + match_length] < src[ip + match_length] {
            write_tree_slot(state, smaller_slot, match_index as u32);
            common_smaller = match_length;
            if match_index <= bt_low {
                smaller_slot = None;
                break;
            }
            smaller_slot = Some(next_slot + 1);
            match_index = state.chain_table[next_slot + 1] as usize;
        } else {
            write_tree_slot(state, larger_slot, match_index as u32);
            common_larger = match_length;
            if match_index <= bt_low {
                larger_slot = None;
                break;
            }
            larger_slot = Some(next_slot);
            match_index = state.chain_table[next_slot] as usize;
        }
    }

    write_tree_slot(state, smaller_slot, 0);
    write_tree_slot(state, larger_slot, 0);
    debug_assert_eq!(config.attached_dictionary.is_some(), ATTACHED_DICT);
    if ATTACHED_DICT {
        let attached = config
            .attached_dictionary
            .expect("attached tree specialization carries dictionary state");
        best_length = attached::find_better_attached_dictionary_match(
            src,
            ip,
            block_end,
            off_base,
            best_length,
            nb_compares,
            config.min_match,
            attached,
        );
    }
    state.next_to_update = match_end_idx - 8;
    best_length
}

fn insert_dubt1(
    src: &[u8],
    curr: usize,
    block_end: usize,
    mut nb_compares: usize,
    bt_low: usize,
    state: &mut GreedyMatchState,
    config: MatchSearchConfig<'_>,
) {
    let params = config.params;
    let mask = bt_mask(params);
    let window_low = config.lowest_prefix_index(curr);
    let mut common_smaller = 0_usize;
    let mut common_larger = 0_usize;
    let curr_slot = tree_slot(curr, mask);
    let mut smaller_slot = Some(curr_slot);
    let mut larger_slot = Some(curr_slot + 1);
    let mut match_index = state.chain_table[curr_slot] as usize;

    while nb_compares > 0 && match_index > window_low {
        nb_compares -= 1;
        let next_slot = tree_slot(match_index, mask);
        let mut match_length = common_smaller.min(common_larger);
        match_length += count_match(
            src,
            curr + match_length,
            match_index + match_length,
            block_end,
        );

        if curr + match_length == block_end {
            break;
        }

        if src[match_index + match_length] < src[curr + match_length] {
            write_tree_slot(state, smaller_slot, match_index as u32);
            common_smaller = match_length;
            if match_index <= bt_low {
                smaller_slot = None;
                break;
            }
            smaller_slot = Some(next_slot + 1);
            match_index = state.chain_table[next_slot + 1] as usize;
        } else {
            write_tree_slot(state, larger_slot, match_index as u32);
            common_larger = match_length;
            if match_index <= bt_low {
                larger_slot = None;
                break;
            }
            larger_slot = Some(next_slot);
            match_index = state.chain_table[next_slot] as usize;
        }
    }

    write_tree_slot(state, smaller_slot, 0);
    write_tree_slot(state, larger_slot, 0);
}

fn bt_mask(params: CompressionParameters) -> usize {
    (1_usize << (params.chain_log - 1)) - 1
}

fn tree_slot(index: usize, mask: usize) -> usize {
    2 * (index & mask)
}

fn tree_value(state: &GreedyMatchState, index: usize, mask: usize, side: usize) -> u32 {
    state.chain_table[tree_slot(index, mask) + side]
}

fn set_tree_value(
    state: &mut GreedyMatchState,
    index: usize,
    mask: usize,
    side: usize,
    value: u32,
) {
    let slot = tree_slot(index, mask) + side;
    state.chain_table[slot] = value;
}

fn write_tree_slot(state: &mut GreedyMatchState, slot: Option<usize>, value: u32) {
    if let Some(slot) = slot {
        state.chain_table[slot] = value;
    }
}
