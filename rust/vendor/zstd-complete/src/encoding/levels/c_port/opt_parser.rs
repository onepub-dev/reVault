//! No-dictionary optimal parser ported from `zstd_opt.c`.

use super::{
    greedy::GreedyBlockOutput,
    ldm::opt::LdmOptCursor,
    opt_match::{OptAttachedDictionary, OptMatchBounds},
    opt_path::{select_path, update_reps},
    opt_state::{OptBlockState, OptParserStrategy, Optimal, HASH_READ_SIZE, ZSTD_OPT_NUM},
    params::CompressionParameters,
    sequence_store::{OffBase, RepeatOffsets, StoredSequence},
};

mod forward;
mod matches;

use forward::{forward_pass, seed_match_prices, seed_parser_root};
#[cfg(test)]
pub(super) use matches::collect_matches;
use matches::{collect_matches_no_ldm_mls, collect_matches_with_ldm_mls};

pub(crate) fn compress_block_opt_no_dict_with_state(
    src: &[u8],
    block_range: core::ops::Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut OptBlockState,
    strategy: OptParserStrategy,
) -> GreedyBlockOutput {
    let bounds = OptMatchBounds::no_dict(block_range.end, params, 0);
    compress_block_opt_with_state_and_ldm::<false, false, false, false>(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        strategy,
        None,
        bounds,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compress_block_opt_no_dict_with_state_and_ldm(
    src: &[u8],
    block_range: core::ops::Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut OptBlockState,
    strategy: OptParserStrategy,
    ldm_cursor: Option<&mut LdmOptCursor<'_>>,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    let bounds = OptMatchBounds::no_dict(block_range.end, params, loaded_dict_end);
    match (ldm_cursor, loaded_dict_end != 0) {
        (Some(cursor), false) => {
            compress_block_opt_with_state_and_ldm::<true, false, false, false>(
                src,
                block_range,
                params,
                repeat_offsets,
                state,
                strategy,
                Some(cursor),
                bounds,
            )
        }
        (Some(cursor), true) => compress_block_opt_with_state_and_ldm::<true, false, true, false>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            strategy,
            Some(cursor),
            bounds,
        ),
        (None, false) => compress_block_opt_with_state_and_ldm::<false, false, false, false>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            strategy,
            None,
            bounds,
        ),
        (None, true) => compress_block_opt_with_state_and_ldm::<false, false, true, false>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            strategy,
            None,
            bounds,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compress_block_opt_ext_dict_with_state_and_ldm(
    src: &[u8],
    block_range: core::ops::Range<usize>,
    dict_limit: usize,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut OptBlockState,
    strategy: OptParserStrategy,
    ldm_cursor: Option<&mut LdmOptCursor<'_>>,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    let bounds = OptMatchBounds::ext_dict(block_range.end, dict_limit, params, loaded_dict_end);
    match (ldm_cursor, loaded_dict_end != 0) {
        (Some(cursor), false) => compress_block_opt_with_state_and_ldm::<true, true, false, false>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            strategy,
            Some(cursor),
            bounds,
        ),
        (Some(cursor), true) => compress_block_opt_with_state_and_ldm::<true, true, true, false>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            strategy,
            Some(cursor),
            bounds,
        ),
        (None, false) => compress_block_opt_with_state_and_ldm::<false, true, false, false>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            strategy,
            None,
            bounds,
        ),
        (None, true) => compress_block_opt_with_state_and_ldm::<false, true, true, false>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            strategy,
            None,
            bounds,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compress_block_opt_attached_dict_with_state_and_ldm(
    src: &[u8],
    block_range: core::ops::Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut OptBlockState,
    strategy: OptParserStrategy,
    ldm_cursor: Option<&mut LdmOptCursor<'_>>,
    attached_dictionary: OptAttachedDictionary,
) -> GreedyBlockOutput {
    let bounds = OptMatchBounds::attached(block_range.end, attached_dictionary);
    match ldm_cursor {
        Some(cursor) => compress_block_opt_with_state_and_ldm::<true, false, false, true>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            strategy,
            Some(cursor),
            bounds,
        ),
        None => compress_block_opt_with_state_and_ldm::<false, false, false, true>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            strategy,
            None,
            bounds,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_optp")]
#[cfg_attr(target_family = "windows", link_section = ".text$030.rz.optp")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.030.ruzstd.opt.parser"
)]
fn compress_block_opt_with_state_and_ldm<
    const WITH_LDM: bool,
    const EXT_DICT: bool,
    const LOADED_DICT: bool,
    const ATTACHED_DICT: bool,
>(
    src: &[u8],
    block_range: core::ops::Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut OptBlockState,
    strategy: OptParserStrategy,
    ldm_cursor: Option<&mut LdmOptCursor<'_>>,
    bounds: OptMatchBounds,
) -> GreedyBlockOutput {
    match params.min_match.clamp(3, 6) {
        3 => compress_block_opt_with_state_and_ldm_mls::<
            3,
            WITH_LDM,
            EXT_DICT,
            LOADED_DICT,
            ATTACHED_DICT,
        >(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            strategy,
            ldm_cursor,
            bounds,
        ),
        4 => compress_block_opt_with_state_and_ldm_mls::<
            4,
            WITH_LDM,
            EXT_DICT,
            LOADED_DICT,
            ATTACHED_DICT,
        >(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            strategy,
            ldm_cursor,
            bounds,
        ),
        5 => compress_block_opt_with_state_and_ldm_mls::<
            5,
            WITH_LDM,
            EXT_DICT,
            LOADED_DICT,
            ATTACHED_DICT,
        >(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            strategy,
            ldm_cursor,
            bounds,
        ),
        6 => compress_block_opt_with_state_and_ldm_mls::<
            6,
            WITH_LDM,
            EXT_DICT,
            LOADED_DICT,
            ATTACHED_DICT,
        >(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            strategy,
            ldm_cursor,
            bounds,
        ),
        _ => unreachable!("mls is clamped to 3..=6"),
    }
}

#[allow(clippy::too_many_arguments)]
fn compress_block_opt_with_state_and_ldm_mls<
    const MLS: u32,
    const WITH_LDM: bool,
    const EXT_DICT: bool,
    const LOADED_DICT: bool,
    const ATTACHED_DICT: bool,
>(
    src: &[u8],
    block_range: core::ops::Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut OptBlockState,
    strategy: OptParserStrategy,
    ldm_cursor: Option<&mut LdmOptCursor<'_>>,
    bounds: OptMatchBounds,
) -> GreedyBlockOutput {
    match strategy {
        OptParserStrategy::BtOpt => compress_block_opt_with_state_and_ldm_mls_level::<
            MLS,
            false,
            WITH_LDM,
            EXT_DICT,
            LOADED_DICT,
            ATTACHED_DICT,
        >(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            ldm_cursor,
            bounds,
        ),
        OptParserStrategy::BtUltra => compress_block_opt_with_state_and_ldm_mls_level::<
            MLS,
            true,
            WITH_LDM,
            EXT_DICT,
            LOADED_DICT,
            ATTACHED_DICT,
        >(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            ldm_cursor,
            bounds,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn compress_block_opt_with_state_and_ldm_mls_level<
    const MLS: u32,
    const ULTRA: bool,
    const WITH_LDM: bool,
    const EXT_DICT: bool,
    const LOADED_DICT: bool,
    const ATTACHED_DICT: bool,
>(
    src: &[u8],
    block_range: core::ops::Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut OptBlockState,
    mut ldm_cursor: Option<&mut LdmOptCursor<'_>>,
    bounds: OptMatchBounds,
) -> GreedyBlockOutput {
    debug_assert!(block_range.start <= block_range.end);
    debug_assert!(block_range.end <= src.len());

    let block_start = block_range.start;
    let block_end = block_range.end;
    let block_len = block_end - block_start;
    if block_len <= HASH_READ_SIZE {
        return GreedyBlockOutput {
            sequences: crate::workspace::ReusableVec::new(),
            last_literals: block_len as u32,
            repeat_offsets,
        };
    }

    // C writes into a reused seqStore; reserve conservatively to avoid repeated
    // growth without allocating for the theoretical maximum sequence density.
    let sequence_capacity = (block_len / min_match_len::<MLS>()).min(block_len);
    let mut sequences = state.take_sequences(sequence_capacity);
    let mut path = core::mem::take(&mut state.path);
    path.clear();

    state.match_state.ensure_tables(params);
    state.match_state.correct_after_long_match_gap(block_start);
    state.match_state.reset_hash3_cursor_to_primary();
    state.match_state.lazy_skipping = false;
    let opt_level = if ULTRA {
        super::opt_price::OptLevel::BtUltra
    } else {
        super::opt_price::OptLevel::BtOpt
    };
    state
        .price_state
        .rescale_freqs(&src[block_range], opt_level);

    let ilimit = block_end - HASH_READ_SIZE;
    let sufficient_len = params.target_length.min((ZSTD_OPT_NUM - 1) as u32);
    let min_match = if MLS == 3 { 3 } else { 4 };
    let mut rep = repeat_offsets.as_offsets();
    let mut ip = block_start + usize::from(block_start == bounds.prefix_start_index());
    let mut anchor = block_start;

    while ip < ilimit {
        let litlen = ip - anchor;
        let match_count = if WITH_LDM {
            collect_matches_with_ldm_mls::<MLS, EXT_DICT, LOADED_DICT, ATTACHED_DICT>(
                src,
                ip,
                block_end,
                rep,
                litlen == 0,
                min_match,
                params,
                state,
                block_start,
                ldm_cursor
                    .as_deref_mut()
                    .expect("LDM specialization requires a cursor"),
                bounds,
            )
        } else {
            collect_matches_no_ldm_mls::<MLS, EXT_DICT, LOADED_DICT, ATTACHED_DICT>(
                src,
                ip,
                block_end,
                rep,
                litlen == 0,
                min_match,
                params,
                state,
                bounds,
            )
        };

        if match_count == 0 {
            ip += 1;
            continue;
        }

        let longest = state.matches.get(match_count - 1);
        seed_parser_root::<ULTRA>(ip, anchor, rep, state);
        path.clear();
        if longest.len > sufficient_len {
            let litlen = (ip - anchor) as u32;
            rep = update_reps(rep, longest.off_base, litlen == 0);
            path.push(Optimal {
                price: 0,
                off: longest.off_base,
                mlen: longest.len,
                litlen,
                rep,
            });
        } else {
            let (seeded_last_pos, zero_literal_length_price) =
                seed_match_prices::<ULTRA>(min_match, match_count, state);
            let forward_ldm_cursor = if WITH_LDM {
                ldm_cursor.as_deref_mut()
            } else {
                None
            };
            let result = forward_pass::<MLS, ULTRA, WITH_LDM, EXT_DICT, LOADED_DICT, ATTACHED_DICT>(
                src,
                ip,
                block_end,
                ilimit,
                seeded_last_pos,
                min_match,
                sufficient_len,
                zero_literal_length_price,
                params,
                state,
                block_start,
                forward_ldm_cursor,
                bounds,
            );

            let empty_stretch = match result.last_stretch {
                Some(stretch) => stretch.mlen == 0,
                None => state.opt[result.last_pos].mlen == 0,
            };
            if empty_stretch {
                ip += result.last_pos;
                continue;
            }

            select_path(
                result.last_pos,
                result.last_stretch,
                &mut rep,
                state,
                &mut path,
            );
        }

        for &step in &path {
            if step.mlen == 0 {
                ip = anchor + step.litlen as usize;
                continue;
            }

            let lit_length = step.litlen;
            let match_length = step.mlen;
            let off_base = step.off;
            let literals = &src[anchor..];
            state
                .price_state
                .update_stats(lit_length, literals, off_base, match_length);
            sequences.push(StoredSequence::new(
                lit_length,
                OffBase::from_c_value(off_base).expect("optimal parser offBase"),
                match_length,
            ));
            anchor += lit_length as usize + match_length as usize;
            ip = anchor;
        }

        state.price_state.refresh_base_prices(opt_level);
    }

    let repeat_offsets = RepeatOffsets::from_offsets(rep[0], rep[1], rep[2]);
    path.clear();
    state.path = path;

    GreedyBlockOutput {
        sequences,
        last_literals: (block_end - anchor) as u32,
        repeat_offsets,
    }
}

const fn min_match_len<const MLS: u32>() -> usize {
    if MLS == 3 {
        3
    } else {
        4
    }
}
