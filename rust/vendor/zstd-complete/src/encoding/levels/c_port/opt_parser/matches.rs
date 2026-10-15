use crate::encoding::levels::c_port::{
    ldm::opt::LdmOptCursor,
    opt_match::{bt_get_all_matches_mls, BtMatchRequest, OptMatchBounds},
    opt_state::OptBlockState,
    params::CompressionParameters,
};

#[allow(clippy::too_many_arguments)]
pub(in crate::encoding::levels::c_port) fn collect_matches(
    src: &[u8],
    ip: usize,
    block_end: usize,
    rep: [u32; 3],
    ll0: bool,
    length_to_beat: u32,
    params: CompressionParameters,
    state: &mut OptBlockState,
    block_start: usize,
    ldm_cursor: Option<&mut LdmOptCursor<'_>>,
    bounds: OptMatchBounds,
) -> usize {
    let ext_dict = bounds.is_ext_dict();
    let loaded_dict = bounds.has_loaded_dict();
    let attached_dict = bounds.attached_dictionary().is_some();
    let mls = params.min_match.clamp(3, 6);

    macro_rules! no_ldm {
        ($ext_dict:literal, $loaded_dict:literal, $attached_dict:literal) => {
            match mls {
                3 => collect_matches_no_ldm_mls::<3, $ext_dict, $loaded_dict, $attached_dict>(
                    src,
                    ip,
                    block_end,
                    rep,
                    ll0,
                    length_to_beat,
                    params,
                    state,
                    bounds,
                ),
                4 => collect_matches_no_ldm_mls::<4, $ext_dict, $loaded_dict, $attached_dict>(
                    src,
                    ip,
                    block_end,
                    rep,
                    ll0,
                    length_to_beat,
                    params,
                    state,
                    bounds,
                ),
                5 => collect_matches_no_ldm_mls::<5, $ext_dict, $loaded_dict, $attached_dict>(
                    src,
                    ip,
                    block_end,
                    rep,
                    ll0,
                    length_to_beat,
                    params,
                    state,
                    bounds,
                ),
                6 => collect_matches_no_ldm_mls::<6, $ext_dict, $loaded_dict, $attached_dict>(
                    src,
                    ip,
                    block_end,
                    rep,
                    ll0,
                    length_to_beat,
                    params,
                    state,
                    bounds,
                ),
                _ => unreachable!("mls is clamped to 3..=6"),
            }
        };
    }

    macro_rules! with_ldm {
        ($cursor:expr, $ext_dict:literal, $loaded_dict:literal, $attached_dict:literal) => {
            match mls {
                3 => collect_matches_with_ldm_mls::<3, $ext_dict, $loaded_dict, $attached_dict>(
                    src,
                    ip,
                    block_end,
                    rep,
                    ll0,
                    length_to_beat,
                    params,
                    state,
                    block_start,
                    $cursor,
                    bounds,
                ),
                4 => collect_matches_with_ldm_mls::<4, $ext_dict, $loaded_dict, $attached_dict>(
                    src,
                    ip,
                    block_end,
                    rep,
                    ll0,
                    length_to_beat,
                    params,
                    state,
                    block_start,
                    $cursor,
                    bounds,
                ),
                5 => collect_matches_with_ldm_mls::<5, $ext_dict, $loaded_dict, $attached_dict>(
                    src,
                    ip,
                    block_end,
                    rep,
                    ll0,
                    length_to_beat,
                    params,
                    state,
                    block_start,
                    $cursor,
                    bounds,
                ),
                6 => collect_matches_with_ldm_mls::<6, $ext_dict, $loaded_dict, $attached_dict>(
                    src,
                    ip,
                    block_end,
                    rep,
                    ll0,
                    length_to_beat,
                    params,
                    state,
                    block_start,
                    $cursor,
                    bounds,
                ),
                _ => unreachable!("mls is clamped to 3..=6"),
            }
        };
    }

    match (ldm_cursor, ext_dict, loaded_dict, attached_dict) {
        (Some(cursor), false, false, true) => with_ldm!(cursor, false, false, true),
        (None, false, false, true) => no_ldm!(false, false, true),
        (Some(cursor), false, false, false) => with_ldm!(cursor, false, false, false),
        (Some(cursor), false, true, false) => with_ldm!(cursor, false, true, false),
        (Some(cursor), true, false, false) => with_ldm!(cursor, true, false, false),
        (Some(cursor), true, true, false) => with_ldm!(cursor, true, true, false),
        (None, false, false, false) => no_ldm!(false, false, false),
        (None, false, true, false) => no_ldm!(false, true, false),
        (None, true, false, false) => no_ldm!(true, false, false),
        (None, true, true, false) => no_ldm!(true, true, false),
        (_, _, _, true) => unreachable!("attached dictionaries use the no-dictionary bounds mode"),
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
pub(super) fn collect_matches_no_ldm_mls<
    const MLS: u32,
    const EXT_DICT: bool,
    const LOADED_DICT: bool,
    const ATTACHED_DICT: bool,
>(
    src: &[u8],
    ip: usize,
    block_end: usize,
    rep: [u32; 3],
    ll0: bool,
    length_to_beat: u32,
    params: CompressionParameters,
    state: &mut OptBlockState,
    bounds: OptMatchBounds,
) -> usize {
    debug_assert_eq!(ATTACHED_DICT, bounds.attached_dictionary().is_some());
    let attached_dictionary = if ATTACHED_DICT {
        let metadata = bounds
            .attached_dictionary()
            .expect("attached dictionary specialization requires attached bounds");
        let attached_state = state
            .attached_match_state
            .as_ref()
            .expect("attached match bounds require an attached dictionary tree");
        Some(metadata.search(src, attached_state))
    } else {
        None
    };
    bt_get_all_matches_mls::<MLS, EXT_DICT, LOADED_DICT, ATTACHED_DICT>(
        &mut state.matches,
        BtMatchRequest {
            src,
            ip,
            block_end,
            rep,
            ll0,
            length_to_beat,
            params,
            bounds,
            attached_dictionary,
        },
        &mut state.match_state,
    );
    state.matches.len()
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
pub(super) fn collect_matches_with_ldm_mls<
    const MLS: u32,
    const EXT_DICT: bool,
    const LOADED_DICT: bool,
    const ATTACHED_DICT: bool,
>(
    src: &[u8],
    ip: usize,
    block_end: usize,
    rep: [u32; 3],
    ll0: bool,
    length_to_beat: u32,
    params: CompressionParameters,
    state: &mut OptBlockState,
    block_start: usize,
    ldm_cursor: &mut LdmOptCursor<'_>,
    bounds: OptMatchBounds,
) -> usize {
    collect_matches_no_ldm_mls::<MLS, EXT_DICT, LOADED_DICT, ATTACHED_DICT>(
        src,
        ip,
        block_end,
        rep,
        ll0,
        length_to_beat,
        params,
        state,
        bounds,
    );
    debug_assert!(ip >= block_start);
    ldm_cursor.process_match_candidate(
        &mut state.matches,
        (ip - block_start) as u32,
        (block_end - ip) as u32,
        length_to_beat,
    );
    state.matches.len()
}
