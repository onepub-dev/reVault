//! External-dictionary fast block compressor ported from `zstd_fast.c`.

use core::ops::Range;

use super::{
    fast::{
        compress_block_fast_no_dict_with_state_and_loaded_dict, FastBlockOutput, FastMatchState,
    },
    fast_helpers::{hash_small_ptr, lowest_prefix_index_with_loaded_dict, read32},
    match_count::count_match_behind as count_match,
    params::CompressionParameters,
    sequence_store::{OffBase, RepeatCode, RepeatOffsets, StoredSequence},
};

const HASH_READ_SIZE: usize = 8;
const SEARCH_STRENGTH: usize = 8;
const INVALID_INDEX: usize = u32::MAX as usize;

pub(crate) fn compress_block_fast_ext_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    dict_limit: usize,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut FastMatchState,
    loaded_dict_end: usize,
) -> FastBlockOutput {
    match params.min_match {
        4 => compress_block_fast_ext_dict_with_state_mls::<4>(
            src,
            block_range,
            dict_limit,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        5 => compress_block_fast_ext_dict_with_state_mls::<5>(
            src,
            block_range,
            dict_limit,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        6 => compress_block_fast_ext_dict_with_state_mls::<6>(
            src,
            block_range,
            dict_limit,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        7 => compress_block_fast_ext_dict_with_state_mls::<7>(
            src,
            block_range,
            dict_limit,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        _ => compress_block_fast_ext_dict_with_state_mls::<4>(
            src,
            block_range,
            dict_limit,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
    }
}

fn compress_block_fast_ext_dict_with_state_mls<const MIN_MATCH: u32>(
    src: &[u8],
    block_range: Range<usize>,
    dict_limit: usize,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut FastMatchState,
    loaded_dict_end: usize,
) -> FastBlockOutput {
    debug_assert!(dict_limit <= block_range.start);
    debug_assert!(block_range.start <= block_range.end);
    debug_assert!(block_range.end <= src.len());

    let mut rep = repeat_offsets.as_offsets();
    let block_start = block_range.start;
    let block_end = block_range.end;
    let block_len = block_end - block_start;

    if block_len <= HASH_READ_SIZE {
        return FastBlockOutput {
            sequences: state.take_sequence_store(),
            last_literals: block_len as u32,
            repeat_offsets,
        };
    }

    let hlog = params.hash_log;
    let step_size = params.target_length as usize + usize::from(params.target_length == 0) + 1;
    let dict_start_index =
        lowest_prefix_index_with_loaded_dict(block_end, params.window_log, loaded_dict_end);
    let prefix_start_index = dict_limit.max(dict_start_index);
    if prefix_start_index == dict_start_index {
        return compress_block_fast_no_dict_with_state_and_loaded_dict(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            0,
        );
    }
    let mut sequences = state.take_sequence_store();
    let ilimit = block_end - HASH_READ_SIZE;

    let hash_table = state.table_for(hlog);
    let mut anchor = block_start;
    let mut ip0 = block_start;

    let mut rep_offset1 = rep[0] as usize;
    let mut rep_offset2 = rep[1] as usize;
    let mut offset_saved1 = 0_usize;
    let mut offset_saved2 = 0_usize;

    let max_rep = ip0 - dict_start_index;
    if rep_offset2 >= max_rep {
        offset_saved2 = rep_offset2;
        rep_offset2 = 0;
    }
    if rep_offset1 >= max_rep {
        offset_saved1 = rep_offset1;
        rep_offset1 = 0;
    }

    'restart: loop {
        let mut step = step_size;
        let mut next_step = ip0 + (1 << (SEARCH_STRENGTH - 1));
        let mut ip1 = ip0 + 1;
        let mut ip2 = ip0 + step;
        let mut ip3 = ip2 + 1;

        if ip3 >= ilimit {
            break;
        }

        let mut hash0 = hash_small_ptr::<MIN_MATCH>(src, ip0, hlog);
        let mut hash1 = hash_small_ptr::<MIN_MATCH>(src, ip1, hlog);
        let mut match_idx = hash_table[hash0] as usize;

        while ip3 < ilimit {
            let current0 = ip0;
            hash_table[hash0] = current0 as u32;

            if rep_match(src, ip2, rep_offset1, dict_start_index, prefix_start_index) {
                ip0 = ip2;
                let mut match0 = ip0 - rep_offset1;
                let backward = usize::from(ip0 > anchor && src[ip0 - 1] == src[match0 - 1]);
                ip0 -= backward;
                match0 -= backward;
                hash_table[hash1] = ip1 as u32;
                store_match(
                    src,
                    &mut sequences,
                    &mut anchor,
                    &mut ip0,
                    match0,
                    OffBase::Repeat(RepeatCode::First),
                    4 + backward,
                    block_end,
                );
                fill_after_match::<MIN_MATCH>(src, hash_table, hlog, current0, ip0, ilimit);
                consume_immediate_repcodes::<MIN_MATCH>(
                    src,
                    hash_table,
                    &mut sequences,
                    hlog,
                    &mut anchor,
                    &mut ip0,
                    ilimit,
                    &mut rep_offset1,
                    &mut rep_offset2,
                    block_end,
                    dict_start_index,
                    prefix_start_index,
                );
                continue 'restart;
            }

            if match4_found(src, ip0, match_idx, dict_start_index, block_end) {
                hash_table[hash1] = ip1 as u32;
                store_offset_match::<MIN_MATCH>(
                    src,
                    hash_table,
                    hlog,
                    &mut sequences,
                    &mut anchor,
                    &mut ip0,
                    match_idx,
                    current0,
                    ilimit,
                    block_end,
                    &mut rep_offset1,
                    &mut rep_offset2,
                    dict_start_index,
                    prefix_start_index,
                );
                continue 'restart;
            }

            match_idx = hash_table[hash1] as usize;
            hash0 = hash1;
            hash1 = hash_small_ptr::<MIN_MATCH>(src, ip2, hlog);
            ip0 = ip1;
            ip1 = ip2;
            ip2 = ip3;

            let current0 = ip0;
            hash_table[hash0] = current0 as u32;

            if match4_found(src, ip0, match_idx, dict_start_index, block_end) {
                if step <= 4 {
                    hash_table[hash1] = ip1 as u32;
                }
                store_offset_match::<MIN_MATCH>(
                    src,
                    hash_table,
                    hlog,
                    &mut sequences,
                    &mut anchor,
                    &mut ip0,
                    match_idx,
                    current0,
                    ilimit,
                    block_end,
                    &mut rep_offset1,
                    &mut rep_offset2,
                    dict_start_index,
                    prefix_start_index,
                );
                continue 'restart;
            }

            match_idx = hash_table[hash1] as usize;
            hash0 = hash1;
            hash1 = hash_small_ptr::<MIN_MATCH>(src, ip2, hlog);
            ip0 = ip1;
            ip1 = ip2;
            ip2 = ip0 + step;
            ip3 = ip1 + step;

            if ip2 >= next_step {
                step += 1;
                next_step += 1 << (SEARCH_STRENGTH - 1);
            }
        }

        break;
    }

    if offset_saved1 != 0 && rep_offset1 != 0 {
        offset_saved2 = offset_saved1;
    }
    rep[0] = (if rep_offset1 != 0 {
        rep_offset1
    } else {
        offset_saved1
    }) as u32;
    rep[1] = (if rep_offset2 != 0 {
        rep_offset2
    } else {
        offset_saved2
    }) as u32;

    FastBlockOutput {
        sequences,
        last_literals: (block_end - anchor) as u32,
        repeat_offsets: RepeatOffsets::from_offsets(rep[0], rep[1], rep[2]),
    }
}

#[allow(clippy::too_many_arguments)]
fn store_offset_match<const MIN_MATCH: u32>(
    src: &[u8],
    hash_table: &mut [u32],
    hlog: u32,
    sequences: &mut crate::workspace::ReusableVec<StoredSequence>,
    anchor: &mut usize,
    ip: &mut usize,
    mut match_pos: usize,
    current0: usize,
    ilimit: usize,
    block_end: usize,
    rep_offset1: &mut usize,
    rep_offset2: &mut usize,
    dict_start_index: usize,
    prefix_start_index: usize,
) {
    let low_match = if match_pos < prefix_start_index {
        dict_start_index
    } else {
        prefix_start_index
    };
    *rep_offset2 = *rep_offset1;
    *rep_offset1 = *ip - match_pos;
    let mut match_length = 4;
    while *ip > *anchor && match_pos > low_match && src[*ip - 1] == src[match_pos - 1] {
        *ip -= 1;
        match_pos -= 1;
        match_length += 1;
    }
    store_match(
        src,
        sequences,
        anchor,
        ip,
        match_pos,
        OffBase::Offset(*rep_offset1 as u32),
        match_length,
        block_end,
    );
    fill_after_match::<MIN_MATCH>(src, hash_table, hlog, current0, *ip, ilimit);
    consume_immediate_repcodes::<MIN_MATCH>(
        src,
        hash_table,
        sequences,
        hlog,
        anchor,
        ip,
        ilimit,
        rep_offset1,
        rep_offset2,
        block_end,
        dict_start_index,
        prefix_start_index,
    );
}

#[allow(clippy::too_many_arguments)]
fn store_match(
    src: &[u8],
    sequences: &mut crate::workspace::ReusableVec<StoredSequence>,
    anchor: &mut usize,
    ip: &mut usize,
    match_pos: usize,
    off_base: OffBase,
    match_length: usize,
    match_limit: usize,
) {
    let match_length = match_length
        + count_match(
            src,
            *ip + match_length,
            match_pos + match_length,
            match_limit,
        );
    sequences.push(StoredSequence::new(
        (*ip - *anchor) as u32,
        off_base,
        match_length as u32,
    ));
    *ip += match_length;
    *anchor = *ip;
}

fn fill_after_match<const MIN_MATCH: u32>(
    src: &[u8],
    hash_table: &mut [u32],
    hlog: u32,
    current0: usize,
    ip: usize,
    ilimit: usize,
) {
    if ip > ilimit {
        return;
    }
    if current0 + 2 <= ilimit {
        hash_table[hash_small_ptr::<MIN_MATCH>(src, current0 + 2, hlog)] = (current0 + 2) as u32;
    }
    if ip >= 2 && ip - 2 <= ilimit {
        hash_table[hash_small_ptr::<MIN_MATCH>(src, ip - 2, hlog)] = (ip - 2) as u32;
    }
}

#[allow(clippy::too_many_arguments)]
fn consume_immediate_repcodes<const MIN_MATCH: u32>(
    src: &[u8],
    hash_table: &mut [u32],
    sequences: &mut crate::workspace::ReusableVec<StoredSequence>,
    hlog: u32,
    anchor: &mut usize,
    ip: &mut usize,
    ilimit: usize,
    rep_offset1: &mut usize,
    rep_offset2: &mut usize,
    match_limit: usize,
    dict_start_index: usize,
    prefix_start_index: usize,
) {
    if *rep_offset2 == 0 {
        return;
    }

    while *ip <= ilimit && rep_match(src, *ip, *rep_offset2, dict_start_index, prefix_start_index) {
        let rep_index = *ip - *rep_offset2;
        let repeat_length = count_match(src, *ip + 4, rep_index + 4, match_limit) + 4;
        core::mem::swap(rep_offset1, rep_offset2);
        hash_table[hash_small_ptr::<MIN_MATCH>(src, *ip, hlog)] = *ip as u32;
        *ip += repeat_length;
        sequences.push(StoredSequence::new(
            0,
            OffBase::Repeat(RepeatCode::First),
            repeat_length as u32,
        ));
        *anchor = *ip;
    }
}

fn rep_match(
    src: &[u8],
    current: usize,
    offset: usize,
    dict_start_index: usize,
    prefix_start_index: usize,
) -> bool {
    if offset == 0 || current < offset {
        return false;
    }
    let rep_index = current - offset;
    rep_index >= dict_start_index
        && index_overlap_check(prefix_start_index, rep_index)
        && rep_index + 4 <= src.len()
        && current + 4 <= src.len()
        && read32(src, rep_index) == read32(src, current)
}

fn match4_found(
    src: &[u8],
    current: usize,
    match_idx: usize,
    dict_start_index: usize,
    match_limit: usize,
) -> bool {
    match_idx != INVALID_INDEX
        && match_idx >= dict_start_index
        && current + 4 <= match_limit
        && match_idx.checked_add(4).is_some_and(|end| end <= src.len())
        && read32(src, current) == read32(src, match_idx)
}

fn index_overlap_check(prefix_lowest_index: usize, rep_index: usize) -> bool {
    prefix_lowest_index.wrapping_sub(1).wrapping_sub(rep_index) >= 3
}
