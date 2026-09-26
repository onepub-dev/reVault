//! Dictionary prefix loading for the C greedy/lazy/lazy2/btlazy2 paths.

use super::{
    bt_match::load_dictionary_binary_tree,
    greedy::GreedyMatchState,
    hash_chain_match::load_dictionary_hash_chain,
    params::CompressionParameters,
    row_match::{load_dictionary_rows, row_match_finder_enabled},
};

const HASH_READ_SIZE: usize = 8;

pub(crate) fn load_prefix(
    state: &mut GreedyMatchState,
    src: &[u8],
    prefix_len: usize,
    params: CompressionParameters,
) {
    debug_assert!(prefix_len <= src.len());
    if prefix_len <= HASH_READ_SIZE {
        return;
    }

    state.ensure_tables(params);
    let target = prefix_len - HASH_READ_SIZE;
    let min_match = params.min_match.clamp(4, 6);

    if row_match_finder_enabled(params) {
        state.tag_table.fill(0);
        load_dictionary_rows(src, target, params, min_match, state);
    } else {
        load_dictionary_hash_chain(src, target, params, min_match, state);
    }

    state.next_to_update = prefix_len;
    state.next_to_update3 = prefix_len;
}

pub(crate) fn load_binary_tree_prefix(
    state: &mut GreedyMatchState,
    src: &[u8],
    prefix_len: usize,
    params: CompressionParameters,
) {
    debug_assert!(prefix_len <= src.len());
    if prefix_len <= HASH_READ_SIZE {
        return;
    }

    state.ensure_tables(params);
    let target = prefix_len - HASH_READ_SIZE;
    let min_match = params.min_match.clamp(4, 6);
    load_dictionary_binary_tree(src, target, prefix_len, params, min_match, state);
    state.next_to_update = prefix_len;
    state.next_to_update3 = prefix_len;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::levels::c_port::params::Strategy;

    fn chain_params() -> CompressionParameters {
        CompressionParameters {
            window_log: 14,
            chain_log: 13,
            hash_log: 15,
            search_log: 4,
            min_match: 4,
            target_length: 0,
            strategy: Strategy::Greedy,
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
            strategy: Strategy::Lazy,
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

    #[test]
    fn hash_chain_loader_stops_at_c_dictionary_target_then_marks_prefix_loaded() {
        let data = b"abcdefghabcdefghabcdefghabcdefgh";
        let params = chain_params();
        let mut state = GreedyMatchState::new();

        load_prefix(&mut state, data, data.len(), params);

        assert_eq!(state.next_to_update, data.len());
        assert_eq!(state.next_to_update3, data.len());
        assert!(!state.chain_table.is_empty());
        assert!(state.hash_table.iter().any(|&index| index > 0));
    }

    #[test]
    fn row_loader_clears_tags_and_marks_prefix_loaded() {
        let data = b"row-dictionary-prefix-row-dictionary-prefix-row";
        let params = row_params();
        let mut state = GreedyMatchState::new();
        state.ensure_tables(params);
        state.tag_table.fill(0xFF);

        load_prefix(&mut state, data, data.len(), params);

        assert_eq!(state.next_to_update, data.len());
        assert_eq!(state.next_to_update3, data.len());
        assert!(state.tag_table.iter().any(|&tag| tag != 0));
        assert!(state.hash_table.iter().any(|&index| index > 0));
    }

    #[test]
    fn binary_tree_loader_uses_dubt_update_then_marks_prefix_loaded() {
        let data = b"abcdefghabcdefghabcdefghabcdefgh";
        let params = btlazy2_params();
        let mut state = GreedyMatchState::new();

        load_binary_tree_prefix(&mut state, data, data.len(), params);

        assert_eq!(state.next_to_update, data.len());
        assert_eq!(state.next_to_update3, data.len());
        assert!(!state.chain_table.is_empty());
        assert!(state.hash_table.iter().any(|&index| index > 0));
    }
}
