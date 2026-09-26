//! Match enumeration for an attached optimal-parser `dictMatchState`.

use super::{OptMatch, OptMatchTable};
use crate::encoding::levels::c_port::{
    hash_chain_match::{hash_ptr_mls, AttachedDictionarySearch},
    sequence_store::OffBase,
};

const ZSTD_OPT_NUM: usize = 1 << 12;

/// Continues an optimal match search in the fully sorted attached dictionary.
///
/// The active tree has already consumed part of `nb_compares`. C passes the
/// remaining budget into its dictionary match state, so this walk does the
/// same and appends only matches that improve on `best_length`.
#[allow(clippy::too_many_arguments)]
pub(super) fn collect_attached_dictionary_matches<const MLS: u32>(
    matches: &mut OptMatchTable,
    src: &[u8],
    ip: usize,
    block_end: usize,
    best_length: &mut usize,
    mut nb_compares: usize,
    match_end_idx: &mut usize,
    attached: AttachedDictionarySearch<'_>,
) {
    let dictionary_size = attached.src.len();
    let Some(dictionary_index_end) = attached.dictionary_index_start.checked_add(dictionary_size)
    else {
        return;
    };
    if nb_compares == 0
        || dictionary_size == 0
        || attached.active_dict_limit < dictionary_index_end
        || attached.active_dict_limit < attached.active_prefix_start
    {
        return;
    }

    let hash = hash_ptr_mls::<MLS>(src, ip, attached.params.hash_log);
    let mut match_index = attached.state.hash_table[hash] as usize;
    let tree_mask = (1_usize << (attached.params.chain_log - 1)) - 1;
    let dictionary_low = attached.dictionary_index_start;
    let dictionary_bt_low = if tree_mask < dictionary_size {
        dictionary_index_end - tree_mask
    } else {
        dictionary_low
    };
    let dictionary_index_delta = attached.active_dict_limit - dictionary_index_end;
    let active_index_delta = attached.active_dict_limit - attached.active_prefix_start;
    let mut common_smaller = 0_usize;
    let mut common_larger = 0_usize;

    while nb_compares > 0 && match_index > dictionary_low {
        nb_compares -= 1;
        let next_slot = 2 * (match_index & tree_mask);
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

        if match_length > *best_length {
            let translated_match_index = match_index + dictionary_index_delta - active_index_delta;
            debug_assert!(translated_match_index < ip);
            if match_length > match_end_idx.saturating_sub(translated_match_index) {
                *match_end_idx = translated_match_index + match_length;
            }
            *best_length = match_length;
            matches.push(OptMatch {
                off_base: OffBase::offset_to_c_value((ip - translated_match_index) as u32),
                len: match_length as u32,
            });
            if match_length > ZSTD_OPT_NUM || ip + match_length == block_end {
                break;
            }
        }

        if match_index <= dictionary_bt_low {
            break;
        }
        let match_byte = attached_dictionary_byte(
            src,
            attached.src,
            dictionary_pos + match_length,
            attached.active_prefix_start,
        );
        if match_byte < src[ip + match_length] {
            common_smaller = match_length;
            match_index = attached.state.chain_table[next_slot + 1] as usize;
        } else {
            common_larger = match_length;
            match_index = attached.state.chain_table[next_slot] as usize;
        }
    }
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
