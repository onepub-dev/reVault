//! Dictionary prefix loading for the C optimal parser strategies.

use super::{
    opt_match::update_tree_no_dict,
    opt_state::{OptBlockState, HASH_READ_SIZE},
    params::CompressionParameters,
};

pub(crate) fn load_prefix(
    state: &mut OptBlockState,
    src: &[u8],
    prefix_len: usize,
    params: CompressionParameters,
) {
    debug_assert!(prefix_len <= src.len());
    if prefix_len <= HASH_READ_SIZE {
        return;
    }

    state.match_state.ensure_tables(params);
    let target = prefix_len - HASH_READ_SIZE;
    let min_match = params.min_match.clamp(3, 6);
    update_tree_no_dict(
        src,
        target,
        prefix_len,
        min_match,
        params,
        &mut state.match_state,
        prefix_len,
    );
    state.match_state.next_to_update = prefix_len;
    state.match_state.next_to_update3 = prefix_len;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::levels::c_port::params::Strategy;

    fn btopt_params() -> CompressionParameters {
        CompressionParameters {
            window_log: 17,
            chain_log: 18,
            hash_log: 17,
            search_log: 4,
            min_match: 3,
            target_length: 32,
            strategy: Strategy::BtOpt,
        }
    }

    #[test]
    fn optimal_loader_updates_tree_then_marks_prefix_loaded_like_c() {
        let data = b"abcdefghabcdefghabcdefghabcdefgh";
        let params = btopt_params();
        let mut state = OptBlockState::new();

        load_prefix(&mut state, data, data.len(), params);

        assert_eq!(state.match_state.next_to_update, data.len());
        assert_eq!(state.match_state.next_to_update3, data.len());
        assert!(!state.match_state.chain_table.is_empty());
        assert!(state.match_state.hash_table.iter().any(|&index| index > 0));
    }
}
