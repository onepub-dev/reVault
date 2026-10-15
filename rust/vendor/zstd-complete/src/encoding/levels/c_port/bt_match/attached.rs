//! Fully sorted binary-tree support for an attached `dictMatchState`.

use super::{bt_mask, tree_slot, write_tree_slot};
use crate::encoding::levels::c_port::{
    greedy::GreedyMatchState,
    hash_chain_match::{count_match, hash_ptr, highbit32, AttachedDictionarySearch},
    params::CompressionParameters,
    sequence_store::OffBase,
};

/// Builds the fully sorted binary tree used by an attached C dictionary.
///
/// C dictionary indexes start at `ZSTD_WINDOW_START_INDEX`, while `src` starts
/// at dictionary byte zero. Keeping that translation explicit lets the match
/// finder preserve C's zero sentinel without adding padding bytes to the input.
pub(in crate::encoding::levels::c_port) fn load_attached_dictionary_binary_tree(
    src: &[u8],
    target: usize,
    index_base: usize,
    params: CompressionParameters,
    min_match: u32,
    state: &mut GreedyMatchState,
) {
    let target_index = index_base + target;
    let mut idx = state.next_to_update;

    debug_assert!(idx >= index_base);
    debug_assert!(target <= src.len());
    while idx < target_index {
        let forward = insert_dictionary_bt1(
            src,
            idx,
            src.len(),
            target_index,
            index_base,
            params,
            min_match,
            state,
        );
        debug_assert!(forward > 0);
        idx += forward;
    }
    state.next_to_update = target_index;
}

#[allow(clippy::too_many_arguments)]
fn insert_dictionary_bt1(
    src: &[u8],
    curr: usize,
    block_end: usize,
    target: usize,
    index_base: usize,
    params: CompressionParameters,
    min_match: u32,
    state: &mut GreedyMatchState,
) -> usize {
    let source_pos = curr - index_base;
    let hash = hash_ptr(src, source_pos, params.hash_log, min_match);
    let mut match_index = state.hash_table[hash] as usize;
    let mask = bt_mask(params);
    let bt_low = curr.saturating_sub(mask);
    let window_low = index_base.max(target.saturating_sub(1_usize << params.window_log));
    let mut common_smaller = 0_usize;
    let mut common_larger = 0_usize;
    let mut smaller_slot = Some(tree_slot(curr, mask));
    let mut larger_slot = Some(tree_slot(curr, mask) + 1);
    let mut match_end_idx = curr + 9;
    let mut best_length = 8_usize;
    let mut nb_compares = 1_usize << params.search_log;

    state.hash_table[hash] = curr as u32;
    while nb_compares > 0 && match_index >= window_low {
        nb_compares -= 1;
        let next_slot = tree_slot(match_index, mask);
        let match_pos = match_index - index_base;
        let mut match_length = common_smaller.min(common_larger);
        match_length += count_match(
            src,
            source_pos + match_length,
            match_pos + match_length,
            block_end,
        );

        if match_length > best_length {
            best_length = match_length;
            if match_length > match_end_idx - match_index {
                match_end_idx = match_index + match_length;
            }
        }
        if source_pos + match_length == block_end {
            break;
        }

        if src[match_pos + match_length] < src[source_pos + match_length] {
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
    let long_match_skip = best_length.saturating_sub(384).min(192);
    long_match_skip.max(match_end_idx - (curr + 8))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn find_better_attached_dictionary_match(
    src: &[u8],
    ip: usize,
    block_end: usize,
    off_base: &mut u32,
    mut best_length: usize,
    mut nb_compares: usize,
    min_match: u32,
    attached: AttachedDictionarySearch<'_>,
) -> usize {
    let dictionary_size = attached.src.len();
    let Some(dictionary_index_end) = attached.dictionary_index_start.checked_add(dictionary_size)
    else {
        return best_length;
    };
    if nb_compares == 0
        || dictionary_size == 0
        || attached.active_dict_limit < dictionary_index_end
        || attached.active_dict_limit < attached.active_prefix_start
    {
        return best_length;
    }

    let hash = hash_ptr(src, ip, attached.params.hash_log, min_match);
    let mut match_index = attached.state.hash_table[hash] as usize;
    let mask = bt_mask(attached.params);
    let dictionary_low = attached.dictionary_index_start;
    let dictionary_bt_low = if mask >= dictionary_size {
        dictionary_low
    } else {
        dictionary_index_end - mask
    };
    let dictionary_index_delta = attached.active_dict_limit - dictionary_index_end;
    let active_index_delta = attached.active_dict_limit - attached.active_prefix_start;
    let mut common_smaller = 0_usize;
    let mut common_larger = 0_usize;

    while nb_compares > 0 && match_index > dictionary_low {
        nb_compares -= 1;
        let next_slot = tree_slot(match_index, mask);
        let dictionary_pos = match_index - dictionary_low;
        let mut match_length = common_smaller.min(common_larger);
        match_length += count_attached_dictionary_match(
            src,
            ip + match_length,
            attached.src,
            dictionary_pos + match_length,
            block_end,
            attached.active_prefix_start,
        );

        if match_length > best_length {
            let translated_match_index = match_index + dictionary_index_delta - active_index_delta;
            let new_gain = highbit32((ip - translated_match_index + 1) as u32) as i32;
            let old_gain = highbit32(off_base.wrapping_add(1)) as i32;
            if (4 * (match_length - best_length)) as i32 > new_gain - old_gain {
                best_length = match_length;
                *off_base = OffBase::offset_to_c_value((ip - translated_match_index) as u32);
            }
            if ip + match_length == block_end {
                break;
            }
        }

        let match_byte = attached_dictionary_byte(
            src,
            attached.src,
            dictionary_pos + match_length,
            attached.active_prefix_start,
        );
        if match_byte < src[ip + match_length] {
            if match_index <= dictionary_bt_low {
                break;
            }
            common_smaller = match_length;
            match_index = attached.state.chain_table[next_slot + 1] as usize;
        } else {
            if match_index <= dictionary_bt_low {
                break;
            }
            common_larger = match_length;
            match_index = attached.state.chain_table[next_slot] as usize;
        }
    }

    best_length
}

fn count_attached_dictionary_match(
    src: &[u8],
    mut ip: usize,
    dictionary: &[u8],
    mut dictionary_pos: usize,
    block_end: usize,
    active_prefix_start: usize,
) -> usize {
    let start = ip;
    while ip < block_end
        && attached_dictionary_byte(src, dictionary, dictionary_pos, active_prefix_start) == src[ip]
    {
        ip += 1;
        dictionary_pos += 1;
    }
    ip - start
}

fn attached_dictionary_byte(
    src: &[u8],
    dictionary: &[u8],
    dictionary_pos: usize,
    active_prefix_start: usize,
) -> u8 {
    if dictionary_pos < dictionary.len() {
        dictionary[dictionary_pos]
    } else {
        src[active_prefix_start + dictionary_pos - dictionary.len()]
    }
}
