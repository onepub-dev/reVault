#[cfg(test)]
use alloc::vec::Vec;
use core::ops::Range;

use crate::encoding::levels::c_port::{
    bt_match::bt_find_best_match,
    greedy::{
        GreedyBlockOutput, GreedyMatchState, SEARCH_BINARY_TREE, SEARCH_HASH_CHAIN, SEARCH_ROW_HASH,
    },
    greedy_bounds::{rep_match_length_no_dict, LazyDictionaryBounds},
    hash_chain_match::{
        hc_find_best_match, highbit32, lowest_prefix_index_with_loaded_dict,
        AttachedDictionarySearch, MatchSearchConfig,
    },
    params::CompressionParameters,
    row_match::{fill_hash_cache, row_find_best_match},
    sequence_store::{OffBase, RepeatCode, RepeatOffsets, StoredSequence},
};

const HASH_READ_SIZE: usize = 8;
const ROW_HASH_CACHE_SIZE: usize = 8;
const SEARCH_STRENGTH: usize = 8;
const LAZY_SKIPPING_STEP: usize = 8;

struct LazySearchContext<'a> {
    src: &'a [u8],
    block_end: usize,
    config: MatchSearchConfig<'a>,
    no_dict_row_search: Option<crate::kernel::row::NoDictSearchFn>,
}

pub(super) fn compress_block_lazy_generic_no_dict_with_state<const SEARCH: u8, const DEPTH: u32>(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    if SEARCH == SEARCH_ROW_HASH {
        return compress_block_row_no_dict_codegen::<DEPTH>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        );
    }
    let bounds = LazyDictionaryBounds::no_dict(block_range.end, params, loaded_dict_end);
    compress_block_lazy_generic_impl::<SEARCH, DEPTH, false, false>(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        bounds,
        None,
    )
}

fn compress_block_row_no_dict_codegen<const DEPTH: u32>(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    loaded_dict_end: usize,
) -> GreedyBlockOutput {
    debug_assert!(DEPTH <= 2);
    debug_assert!(block_range.start <= block_range.end);
    debug_assert!(block_range.end <= src.len());

    let mut sequences = state.take_sequence_store();
    let block_len = block_range.end - block_range.start;
    if block_len <= HASH_READ_SIZE + ROW_HASH_CACHE_SIZE {
        return GreedyBlockOutput {
            sequences,
            last_literals: block_len as u32,
            repeat_offsets,
        };
    }

    state.ensure_tables(params);
    state.correct_after_long_match_gap(block_range.start);
    state.lazy_skipping = false;

    // Every stored sequence consumes a disjoint match of at least four source
    // bytes. Reserve that proven maximum once, then let the isolated parser
    // initialize only the returned prefix, mirroring C's bounded SeqStore.
    let maximum_sequences = block_len / 4 + 1;
    sequences.reserve(maximum_sequences);
    let spare = sequences.spare_capacity_mut();
    debug_assert!(spare.len() >= maximum_sequences);
    // SAFETY: `StoredSequence` is compile-time asserted to be layout-identical
    // to `[u32; 3]`; `MaybeUninit` preserves that layout and no elements are
    // considered initialized until the returned prefix length is committed.
    let sequence_words = unsafe {
        core::slice::from_raw_parts_mut(
            spare
                .as_mut_ptr()
                .cast::<core::mem::MaybeUninit<[u32; 3]>>(),
            spare.len(),
        )
    };
    // SAFETY: the frame state owns tables/cache for these parameters and the
    // block bounds plus reserved sequence prefix satisfy the kernel contract.
    let result = unsafe {
        crate::kernel::row::select_block_no_dict(
            DEPTH,
            params.min_match.clamp(4, 6),
            params.search_log,
        )(
            src,
            block_range.start,
            block_range.end,
            loaded_dict_end,
            params.window_log,
            params.hash_log,
            params.search_log,
            state.hash_salt,
            repeat_offsets.as_offsets(),
            &mut state.hash_table,
            &mut state.tag_table,
            &mut state.row_hash_cache,
            &mut state.next_to_update,
            &mut state.hash_salt_entropy,
            sequence_words,
        )
    };
    debug_assert!(result.sequence_count <= maximum_sequences);
    // SAFETY: the isolated parser initialized exactly the returned prefix, and
    // that prefix is bounded by the spare-capacity slice passed above.
    unsafe { sequences.set_len(result.sequence_count) };
    state.lazy_skipping = result.lazy_skipping;

    GreedyBlockOutput {
        sequences,
        last_literals: result.last_literals,
        repeat_offsets: RepeatOffsets::from_offsets(
            result.repeat_offsets[0],
            result.repeat_offsets[1],
            result.repeat_offsets[2],
        ),
    }
}

pub(in crate::encoding::levels::c_port) fn compress_block_lazy_generic_with_state<
    const SEARCH: u8,
    const DEPTH: u32,
>(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    bounds: LazyDictionaryBounds,
) -> GreedyBlockOutput {
    compress_block_lazy_generic_impl::<SEARCH, DEPTH, true, false>(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        bounds,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::encoding::levels::c_port) fn compress_block_lazy_generic_with_state_and_attached<
    'a,
    const SEARCH: u8,
    const DEPTH: u32,
>(
    src: &'a [u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    bounds: LazyDictionaryBounds,
    attached_dictionary: AttachedDictionarySearch<'a>,
) -> GreedyBlockOutput {
    compress_block_lazy_generic_impl::<SEARCH, DEPTH, false, true>(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        bounds,
        Some(attached_dictionary),
    )
}

#[allow(clippy::too_many_arguments)]
fn compress_block_lazy_generic_impl<
    'a,
    const SEARCH: u8,
    const DEPTH: u32,
    const EXT_DICT: bool,
    const ATTACHED_DICT: bool,
>(
    src: &'a [u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut GreedyMatchState,
    bounds: LazyDictionaryBounds,
    attached_dictionary: Option<AttachedDictionarySearch<'a>>,
) -> GreedyBlockOutput {
    debug_assert!(block_range.start <= block_range.end);
    debug_assert!(block_range.end <= src.len());
    debug_assert_eq!(bounds.ext_dict, EXT_DICT);
    debug_assert_eq!(attached_dictionary.is_some(), ATTACHED_DICT);
    debug_assert!(DEPTH <= 2);
    debug_assert!(!EXT_DICT || bounds.dict_limit <= block_range.start);

    let mut rep = repeat_offsets.as_offsets();
    let mut sequences = state.take_sequence_store();
    let block_start = block_range.start;
    let block_end = block_range.end;
    let block_len = block_end - block_start;

    let search_read_size = search_read_size::<SEARCH>();
    if block_len <= search_read_size {
        return GreedyBlockOutput {
            sequences,
            last_literals: block_len as u32,
            repeat_offsets,
        };
    }

    state.ensure_tables(params);
    state.correct_after_long_match_gap(block_start);
    state.lazy_skipping = false;

    let min_match = params.min_match.clamp(4, 6);
    let mut search_config = MatchSearchConfig::new(params, min_match, bounds.loaded_dict_end);
    if ATTACHED_DICT {
        search_config = search_config.with_attached_dictionary(
            attached_dictionary.expect("attached specialization carries dictionary state"),
        );
    }
    let ilimit = block_end - search_read_size;
    let search_context = LazySearchContext {
        src,
        block_end,
        config: search_config,
        no_dict_row_search: if SEARCH == SEARCH_ROW_HASH && !ATTACHED_DICT {
            Some(crate::kernel::row::select_best_match_no_dict(
                min_match,
                params.search_log,
            ))
        } else {
            None
        },
    };
    let mut ip = block_start + usize::from(block_start == bounds.prefix_start_index);
    if SEARCH == SEARCH_ROW_HASH {
        fill_hash_cache(src, state.next_to_update, ilimit, params, min_match, state);
    }
    let mut anchor = block_start;

    let mut offset_1 = rep[0] as usize;
    let mut offset_2 = rep[1] as usize;
    let mut offset_saved1 = 0_usize;
    let mut offset_saved2 = 0_usize;

    if !EXT_DICT {
        let window_low =
            lowest_prefix_index_with_loaded_dict(ip, params.window_log, bounds.loaded_dict_end);
        let max_rep = ip - window_low;
        if offset_2 > max_rep {
            offset_saved2 = offset_2;
            offset_2 = 0;
        }
        if offset_1 > max_rep {
            offset_saved1 = offset_1;
            offset_1 = 0;
        }
    }

    while ip < ilimit {
        let mut match_length = 0_usize;
        let mut off_base = OffBase::Repeat(RepeatCode::First).to_c_value();
        let mut start = ip + 1;

        let rep_length =
            rep_match_length::<EXT_DICT>(bounds, src, ip + 1, offset_1, params, block_end);
        if rep_length >= 4 {
            match_length = rep_length;
            if DEPTH == 0 {
                store_sequence(
                    &mut sequences,
                    &mut anchor,
                    &mut ip,
                    start,
                    OffBase::from_c_value(off_base).expect("repcode offBase"),
                    match_length,
                );
                continue_immediate_repcodes::<EXT_DICT>(
                    src,
                    &mut sequences,
                    &mut anchor,
                    &mut ip,
                    ilimit,
                    block_end,
                    &mut offset_1,
                    &mut offset_2,
                    params,
                    bounds,
                );
                continue;
            }
        }

        let mut offbase_found = 999_999_999_u32;
        let ml2 =
            search_max::<SEARCH, ATTACHED_DICT>(&search_context, ip, &mut offbase_found, state);
        if ml2 > match_length {
            match_length = ml2;
            start = ip;
            off_base = offbase_found;
        }

        if match_length < 4 {
            let (step, lazy_skipping) = lazy_miss_step(ip - anchor, EXT_DICT);
            ip += step;
            state.lazy_skipping = lazy_skipping;
            continue;
        }

        if DEPTH >= 1 {
            loop {
                ip += 1;
                if off_base != 0 {
                    let ml_rep =
                        rep_match_length::<EXT_DICT>(bounds, src, ip, offset_1, params, block_end);
                    let gain2 = (ml_rep * 3) as i32;
                    let gain1 = (match_length * 3) as i32 - highbit32(off_base) as i32 + 1;
                    if ml_rep >= 4 && gain2 > gain1 {
                        match_length = ml_rep;
                        off_base = OffBase::Repeat(RepeatCode::First).to_c_value();
                        start = ip;
                    }
                }

                let mut ofb_candidate = 999_999_999_u32;
                let ml2 = search_max::<SEARCH, ATTACHED_DICT>(
                    &search_context,
                    ip,
                    &mut ofb_candidate,
                    state,
                );
                let gain2 = (ml2 * 4) as i32 - highbit32(ofb_candidate) as i32;
                let gain1 = (match_length * 4) as i32 - highbit32(off_base) as i32 + 4;
                if ml2 >= 4 && gain2 > gain1 {
                    match_length = ml2;
                    off_base = ofb_candidate;
                    start = ip;
                    if ip < ilimit {
                        continue;
                    }
                }

                if DEPTH == 2 && ip < ilimit {
                    ip += 1;
                    if off_base != 0 {
                        let ml_rep = rep_match_length::<EXT_DICT>(
                            bounds, src, ip, offset_1, params, block_end,
                        );
                        let gain2 = (ml_rep * 4) as i32;
                        let gain1 = (match_length * 4) as i32 - highbit32(off_base) as i32 + 1;
                        if ml_rep >= 4 && gain2 > gain1 {
                            match_length = ml_rep;
                            off_base = OffBase::Repeat(RepeatCode::First).to_c_value();
                            start = ip;
                        }
                    }

                    let mut ofb_candidate = 999_999_999_u32;
                    let ml2 = search_max::<SEARCH, ATTACHED_DICT>(
                        &search_context,
                        ip,
                        &mut ofb_candidate,
                        state,
                    );
                    let gain2 = (ml2 * 4) as i32 - highbit32(ofb_candidate) as i32;
                    let gain1 = (match_length * 4) as i32 - highbit32(off_base) as i32 + 7;
                    if ml2 >= 4 && gain2 > gain1 {
                        match_length = ml2;
                        off_base = ofb_candidate;
                        start = ip;
                        if ip < ilimit {
                            continue;
                        }
                    }
                }

                break;
            }
        }

        let off_base = OffBase::from_c_value(off_base).expect("stored match has an offBase");
        if let OffBase::Offset(offset) = off_base {
            let offset = offset as usize;
            let mut match_pos = start - offset;
            let low_match = bounds.low_match_index::<EXT_DICT>(match_pos);
            while start > anchor && match_pos > low_match && src[start - 1] == src[match_pos - 1] {
                start -= 1;
                match_pos -= 1;
                match_length += 1;
            }
            offset_2 = offset_1;
            offset_1 = offset;
        }

        store_sequence(
            &mut sequences,
            &mut anchor,
            &mut ip,
            start,
            off_base,
            match_length,
        );

        if state.lazy_skipping {
            if SEARCH == SEARCH_ROW_HASH {
                fill_hash_cache(src, state.next_to_update, ilimit, params, min_match, state);
            }
            state.lazy_skipping = false;
        }

        continue_immediate_repcodes::<EXT_DICT>(
            src,
            &mut sequences,
            &mut anchor,
            &mut ip,
            ilimit,
            block_end,
            &mut offset_1,
            &mut offset_2,
            params,
            bounds,
        );
    }

    if EXT_DICT {
        rep[0] = offset_1 as u32;
        rep[1] = offset_2 as u32;
    } else {
        if offset_saved1 != 0 && offset_1 != 0 {
            offset_saved2 = offset_saved1;
        }

        rep[0] = if offset_1 != 0 {
            offset_1
        } else {
            offset_saved1
        } as u32;
        rep[1] = if offset_2 != 0 {
            offset_2
        } else {
            offset_saved2
        } as u32;
    }

    GreedyBlockOutput {
        sequences,
        last_literals: (block_end - anchor) as u32,
        repeat_offsets: RepeatOffsets::from_offsets(rep[0], rep[1], rep[2]),
    }
}

fn search_max<const SEARCH: u8, const ATTACHED_DICT: bool>(
    context: &LazySearchContext<'_>,
    ip: usize,
    off_base: &mut u32,
    state: &mut GreedyMatchState,
) -> usize {
    match SEARCH {
        SEARCH_HASH_CHAIN => hc_find_best_match(
            context.src,
            ip,
            context.block_end,
            off_base,
            state,
            context.config,
        ),
        SEARCH_ROW_HASH => row_find_best_match::<ATTACHED_DICT>(
            context.src,
            ip,
            context.block_end,
            off_base,
            state,
            context.config,
            context.no_dict_row_search,
        ),
        SEARCH_BINARY_TREE => bt_find_best_match::<ATTACHED_DICT>(
            context.src,
            ip,
            context.block_end,
            off_base,
            state,
            context.config,
        ),
        _ => unreachable!("unknown lazy search kind"),
    }
}

fn search_read_size<const SEARCH: u8>() -> usize {
    match SEARCH {
        SEARCH_ROW_HASH => HASH_READ_SIZE + ROW_HASH_CACHE_SIZE,
        SEARCH_HASH_CHAIN | SEARCH_BINARY_TREE => HASH_READ_SIZE,
        _ => unreachable!("unknown lazy search kind"),
    }
}

#[inline(always)]
fn rep_match_length<const EXT_DICT: bool>(
    bounds: LazyDictionaryBounds,
    src: &[u8],
    current: usize,
    offset: usize,
    params: CompressionParameters,
    block_end: usize,
) -> usize {
    if EXT_DICT {
        bounds
            .rep_match_length::<true>(src, current, offset, params, block_end)
            .unwrap_or(0)
    } else {
        rep_match_length_no_dict(src, current, offset, block_end)
    }
}

#[allow(clippy::too_many_arguments)]
fn continue_immediate_repcodes<const EXT_DICT: bool>(
    src: &[u8],
    sequences: &mut crate::workspace::ReusableVec<StoredSequence>,
    anchor: &mut usize,
    ip: &mut usize,
    ilimit: usize,
    block_end: usize,
    offset_1: &mut usize,
    offset_2: &mut usize,
    params: CompressionParameters,
    bounds: LazyDictionaryBounds,
) {
    while *ip <= ilimit {
        let repeat_length =
            rep_match_length::<EXT_DICT>(bounds, src, *ip, *offset_2, params, block_end);
        if repeat_length < 4 {
            break;
        }
        core::mem::swap(offset_2, offset_1);
        let repeat_start = *anchor;
        store_sequence(
            sequences,
            anchor,
            ip,
            repeat_start,
            OffBase::Repeat(RepeatCode::First),
            repeat_length,
        );
    }
}

fn store_sequence(
    sequences: &mut crate::workspace::ReusableVec<StoredSequence>,
    anchor: &mut usize,
    ip: &mut usize,
    start: usize,
    off_base: OffBase,
    match_length: usize,
) {
    sequences.push(StoredSequence::new(
        (start - *anchor) as u32,
        off_base,
        match_length as u32,
    ));
    *ip = start + match_length;
    *anchor = *ip;
}

fn lazy_miss_step(distance_from_anchor: usize, ext_dict: bool) -> (usize, bool) {
    let raw_step = distance_from_anchor >> SEARCH_STRENGTH;
    let step = raw_step + 1;
    // `zstd_lazy.c` uses the incremented step for no-dict lazy skipping, but
    // the raw pre-increment step for ext-dict lazy skipping.
    let lazy_skipping = if ext_dict {
        raw_step > LAZY_SKIPPING_STEP
    } else {
        step > LAZY_SKIPPING_STEP
    };
    (step, lazy_skipping)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::levels::c_port::params::Strategy;

    fn row_params() -> CompressionParameters {
        CompressionParameters {
            window_log: 18,
            chain_log: 16,
            hash_log: 16,
            search_log: 5,
            min_match: 4,
            target_length: 0,
            strategy: Strategy::Lazy2,
        }
    }

    fn assert_codegen_parser_matches_local<const DEPTH: u32>() {
        let mut source = Vec::new();
        for index in 0..4096u32 {
            source.extend_from_slice(&(index % 37).to_le_bytes());
            source.extend_from_slice(b"row-parser-reference-pattern");
            if index.is_multiple_of(11) {
                source.extend_from_slice(b"row-parser-reference-pattern");
            }
        }
        let params = row_params();
        let block_range = 0..source.len();
        let bounds = LazyDictionaryBounds::no_dict(block_range.end, params, 0);
        let mut local_state = GreedyMatchState::new();
        local_state.reset_for_frame(params);
        let mut codegen_state = local_state.clone();

        let local = compress_block_lazy_generic_impl::<SEARCH_ROW_HASH, DEPTH, false, false>(
            &source,
            block_range.clone(),
            params,
            RepeatOffsets::new(),
            &mut local_state,
            bounds,
            None,
        );
        let codegen = compress_block_row_no_dict_codegen::<DEPTH>(
            &source,
            block_range,
            params,
            RepeatOffsets::new(),
            &mut codegen_state,
            0,
        );

        assert_eq!(codegen, local);
        assert_eq!(codegen_state, local_state);
    }

    #[test]
    fn no_dict_lazy_skipping_uses_incremented_step_like_c() {
        assert_eq!(lazy_miss_step(7 << SEARCH_STRENGTH, false), (8, false));
        assert_eq!(lazy_miss_step(8 << SEARCH_STRENGTH, false), (9, true));
    }

    #[test]
    fn ext_dict_lazy_skipping_uses_raw_step_like_c() {
        assert_eq!(lazy_miss_step(8 << SEARCH_STRENGTH, true), (9, false));
        assert_eq!(lazy_miss_step(9 << SEARCH_STRENGTH, true), (10, true));
    }

    #[test]
    fn generated_row_parser_matches_local_greedy_lazy_and_lazy2() {
        assert_codegen_parser_matches_local::<0>();
        assert_codegen_parser_matches_local::<1>();
        assert_codegen_parser_matches_local::<2>();
    }
}
