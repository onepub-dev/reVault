use super::super::hash_chain_match::{AttachedDictionarySearch, MatchSearchConfig};
use super::*;
use crate::encoding::levels::c_port::params::Strategy;
use alloc::{vec, vec::Vec};

const C_WINDOW_START_INDEX: usize = 2;

fn params() -> CompressionParameters {
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
fn row_next_index_cycles_backwards_and_skips_zero() {
    let mut tags = [0u8; 16];

    assert_eq!(super::super::row_table::next_index(&mut tags, 0, 15), 15);
    assert_eq!(tags[0], 15);
    assert_eq!(super::super::row_table::next_index(&mut tags, 0, 15), 14);
}

#[test]
fn row_match_mask_preserves_circular_scan_order() {
    let mut row = [0u8; 16];
    row[0] = 5;
    row[2] = 7;
    row[5] = 7;
    row[9] = 7;
    row[15] = 7;

    let mut matches = row_match_mask(&row, 7, usize::from(row[0]));
    let mut positions = vec![];
    while matches != 0 {
        let step = matches.trailing_zeros() as usize;
        matches &= matches - 1;
        positions.push((usize::from(row[0]) + step) & 15);
    }

    assert_eq!(positions, vec![5, 9, 15, 2]);
}

#[test]
fn row_match_mask_matches_scalar_for_all_row_widths() {
    for width in [16usize, 32, 64] {
        let mut row = vec![0u8; width];
        for (idx, byte) in row.iter_mut().enumerate() {
            *byte = (idx.wrapping_mul(37).wrapping_add(width) & 0xff) as u8;
        }
        for idx in (3..width).step_by(7) {
            row[idx] = 0x5a;
        }

        for head in [0usize, 1, width / 3, width - 1] {
            assert_eq!(
                row_match_mask(&row, 0x5a, head),
                row_match_mask_scalar(&row, 0x5a, head)
            );
        }
    }
}

#[test]
fn row_finder_reports_previous_match() {
    let data = b"abcdefghabcdefgh-tail";
    let mut state = GreedyMatchState::new();
    let params = params();
    state.ensure_tables(params);
    let mut off_base = 0;
    fill_hash_cache(
        data,
        state.next_to_update,
        data.len() - 16,
        params,
        4,
        &mut state,
    );

    let match_len = row_find_best_match::<false>(
        data,
        8,
        data.len(),
        &mut off_base,
        &mut state,
        MatchSearchConfig::new(params, 4, 0),
        Some(crate::kernel::row::select_best_match_no_dict(
            4,
            params.search_log,
        )),
    );

    assert!(match_len >= 8);
    assert_eq!(off_base, 11);
}

#[test]
fn row_finder_searches_attached_dictionary_rows_after_active_rows() {
    let dict = b"prefix:shared-payload";
    let source = b"zzzz:shared-payload-tail";
    let mut combined = Vec::new();
    combined.extend_from_slice(dict);
    combined.extend_from_slice(source);

    let params = params();
    let mut dict_state = GreedyMatchState::new();
    dict_state.ensure_tables(params);
    dict_state.next_to_update = C_WINDOW_START_INDEX;
    load_dictionary_rows_at_index_base(
        dict,
        dict.len() - 8,
        C_WINDOW_START_INDEX,
        params,
        4,
        &mut dict_state,
    );

    let mut active_state = GreedyMatchState::new();
    active_state.ensure_tables(params);
    active_state.next_to_update = dict.len();
    let mut off_base = 0;
    fill_hash_cache(
        &combined,
        active_state.next_to_update,
        combined.len() - 16,
        params,
        4,
        &mut active_state,
    );

    let ip = dict.len() + 5;
    let match_len = row_find_best_match::<true>(
        &combined,
        ip,
        combined.len(),
        &mut off_base,
        &mut active_state,
        MatchSearchConfig::new(params, 4, 0).with_attached_dictionary(AttachedDictionarySearch {
            src: dict,
            state: &dict_state,
            params,
            dictionary_index_start: C_WINDOW_START_INDEX,
            active_dict_limit: dict.len() + C_WINDOW_START_INDEX,
            active_prefix_start: dict.len(),
        }),
        None,
    );

    assert!(match_len >= b"shared-payload".len());
    assert_eq!(off_base, OffBase::offset_to_c_value((ip - 7) as u32));
}

#[test]
fn row_finder_uses_c_userspace_window_gate() {
    let mut params = params();
    assert!(row_match_finder_enabled(params));
    params.window_log = 15;
    assert!(row_match_finder_enabled(params));
    params.window_log = 14;
    assert!(!row_match_finder_enabled(params));
}

#[test]
fn row_update_skips_middle_of_large_gaps_like_c() {
    let mut data = vec![0u8; 540];
    for (idx, byte) in data.iter_mut().enumerate() {
        *byte = (idx.wrapping_mul(37) & 0xFF) as u8;
    }
    let pattern = b"abcdefghijklmnopqrstuvwxyz";
    data[200..200 + pattern.len()].copy_from_slice(pattern);
    data[500..500 + pattern.len()].copy_from_slice(pattern);

    let mut state = GreedyMatchState::new();
    let params = params();
    state.ensure_tables(params);
    let mut off_base = 0;
    fill_hash_cache(
        &data,
        state.next_to_update,
        data.len() - 16,
        params,
        4,
        &mut state,
    );

    let match_len = row_find_best_match::<false>(
        &data,
        500,
        data.len(),
        &mut off_base,
        &mut state,
        MatchSearchConfig::new(params, 4, 0),
        Some(crate::kernel::row::select_best_match_no_dict(
            4,
            params.search_log,
        )),
    );

    assert_eq!(match_len, 3);
    assert_eq!(off_base, 0);
}
