//! External-dictionary double-fast block compressor ported from `zstd_double_fast.c`.

use core::ops::Range;

use super::{
    dfast::{
        compress_block_double_fast_no_dict_with_state_and_loaded_dict, DFastBlockOutput,
        DFastMatchState,
    },
    dfast_helpers::{
        count_match, hash8_ptr, hash_small_ptr, lowest_prefix_index_with_loaded_dict, read32,
        read64, store_match, HASH_READ_SIZE,
    },
    dfast_table::{entry, set_entry},
    params::CompressionParameters,
    sequence_store::{OffBase, RepeatCode, RepeatOffsets, StoredSequence},
};

pub(crate) fn compress_block_double_fast_ext_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    dict_limit: usize,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut DFastMatchState,
    loaded_dict_end: usize,
) -> DFastBlockOutput {
    match params.min_match {
        4 => compress_block_double_fast_ext_dict_with_state_mls::<4>(
            src,
            block_range,
            dict_limit,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        5 => compress_block_double_fast_ext_dict_with_state_mls::<5>(
            src,
            block_range,
            dict_limit,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        6 => compress_block_double_fast_ext_dict_with_state_mls::<6>(
            src,
            block_range,
            dict_limit,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        7 => compress_block_double_fast_ext_dict_with_state_mls::<7>(
            src,
            block_range,
            dict_limit,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        _ => compress_block_double_fast_ext_dict_with_state_mls::<4>(
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

fn compress_block_double_fast_ext_dict_with_state_mls<const MIN_MATCH: u32>(
    src: &[u8],
    block_range: Range<usize>,
    dict_limit: usize,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut DFastMatchState,
    loaded_dict_end: usize,
) -> DFastBlockOutput {
    debug_assert!(dict_limit <= block_range.start);
    debug_assert!(block_range.start <= block_range.end);
    debug_assert!(block_range.end <= src.len());

    let mut rep = repeat_offsets.as_offsets();
    let block_start = block_range.start;
    let block_end = block_range.end;
    let block_len = block_end - block_start;

    if block_len <= HASH_READ_SIZE {
        return DFastBlockOutput {
            sequences: state.take_sequence_store(),
            last_literals: block_len as u32,
            repeat_offsets,
        };
    }

    state.ensure_tables(params);
    let h_bits_l = params.hash_log;
    let h_bits_s = params.chain_log;
    let dict_start_index =
        lowest_prefix_index_with_loaded_dict(block_end, params.window_log, loaded_dict_end);
    let prefix_start_index = dict_limit.max(dict_start_index);
    if prefix_start_index == dict_start_index {
        return compress_block_double_fast_no_dict_with_state_and_loaded_dict(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            0,
        );
    }
    let mut sequences = state.take_sequence_store();
    let hash_long = &mut state.hash_long;
    let hash_small = &mut state.hash_small;
    let ilimit = block_end - HASH_READ_SIZE;

    let mut anchor = block_start;
    let mut ip = block_start;
    let mut offset_1 = rep[0] as usize;
    let mut offset_2 = rep[1] as usize;

    while ip < ilimit {
        let h_small = hash_small_ptr::<MIN_MATCH>(src, ip, h_bits_s);
        let match_index = entry(hash_small, h_small) as usize;
        let mut match_pos = match_index;

        let h_long = hash8_ptr(src, ip, h_bits_l);
        let match_long_index = entry(hash_long, h_long) as usize;
        let mut match_long = match_long_index;

        let curr = ip;
        set_entry(hash_small, h_small, curr as u32);
        set_entry(hash_long, h_long, curr as u32);

        let mut match_length;
        if rep_match_at_next(src, ip, offset_1, dict_start_index, prefix_start_index) {
            let rep_index = ip + 1 - offset_1;
            match_length = count_match(src, ip + 1 + 4, rep_index + 4, block_end) + 4;
            ip += 1;
            store_match(
                &mut sequences,
                &mut anchor,
                &mut ip,
                OffBase::Repeat(RepeatCode::First),
                match_length,
            );
        } else if match_long_index > dict_start_index && read64(src, match_long) == read64(src, ip)
        {
            match_length = count_match(src, ip + 8, match_long + 8, block_end) + 8;
            let low_match = if match_long_index < prefix_start_index {
                dict_start_index
            } else {
                prefix_start_index
            };
            while ip > anchor && match_long > low_match && src[ip - 1] == src[match_long - 1] {
                ip -= 1;
                match_long -= 1;
                match_length += 1;
            }
            let offset = ip - match_long;
            offset_2 = offset_1;
            offset_1 = offset;
            store_match(
                &mut sequences,
                &mut anchor,
                &mut ip,
                OffBase::Offset(offset as u32),
                match_length,
            );
        } else if match_index > dict_start_index && read32(src, match_pos) == read32(src, ip) {
            let h3 = hash8_ptr(src, ip + 1, h_bits_l);
            let match_index3 = entry(hash_long, h3) as usize;
            let mut match3 = match_index3;
            set_entry(hash_long, h3, (curr + 1) as u32);

            let offset;
            if match_index3 > dict_start_index && read64(src, match3) == read64(src, ip + 1) {
                match_length = count_match(src, ip + 9, match3 + 8, block_end) + 8;
                ip += 1;
                offset = ip - match3;
                let low_match = if match_index3 < prefix_start_index {
                    dict_start_index
                } else {
                    prefix_start_index
                };
                while ip > anchor && match3 > low_match && src[ip - 1] == src[match3 - 1] {
                    ip -= 1;
                    match3 -= 1;
                    match_length += 1;
                }
            } else {
                match_length = count_match(src, ip + 4, match_pos + 4, block_end) + 4;
                offset = ip - match_pos;
                let low_match = if match_index < prefix_start_index {
                    dict_start_index
                } else {
                    prefix_start_index
                };
                while ip > anchor && match_pos > low_match && src[ip - 1] == src[match_pos - 1] {
                    ip -= 1;
                    match_pos -= 1;
                    match_length += 1;
                }
            }
            offset_2 = offset_1;
            offset_1 = offset;
            store_match(
                &mut sequences,
                &mut anchor,
                &mut ip,
                OffBase::Offset(offset as u32),
                match_length,
            );
        } else {
            ip += ((ip - anchor) >> 8) + 1;
            continue;
        }

        if ip <= ilimit {
            complementary_insert::<MIN_MATCH>(
                src, hash_long, hash_small, h_bits_l, h_bits_s, curr, ip, ilimit,
            );
            consume_immediate_repcodes::<MIN_MATCH>(
                src,
                hash_long,
                hash_small,
                h_bits_l,
                h_bits_s,
                &mut sequences,
                &mut anchor,
                &mut ip,
                ilimit,
                &mut offset_1,
                &mut offset_2,
                block_end,
                dict_start_index,
                prefix_start_index,
            );
        }
    }

    rep[0] = offset_1 as u32;
    rep[1] = offset_2 as u32;

    DFastBlockOutput {
        sequences,
        last_literals: (block_end - anchor) as u32,
        repeat_offsets: RepeatOffsets::from_offsets(rep[0], rep[1], rep[2]),
    }
}

#[allow(clippy::too_many_arguments)]
fn complementary_insert<const MIN_MATCH: u32>(
    src: &[u8],
    hash_long: &mut [u32],
    hash_small: &mut [u32],
    h_bits_l: u32,
    h_bits_s: u32,
    curr: usize,
    ip: usize,
    ilimit: usize,
) {
    let index_to_insert = curr + 2;
    if index_to_insert <= ilimit {
        set_entry(
            hash_long,
            hash8_ptr(src, index_to_insert, h_bits_l),
            index_to_insert as u32,
        );
        set_entry(
            hash_small,
            hash_small_ptr::<MIN_MATCH>(src, index_to_insert, h_bits_s),
            index_to_insert as u32,
        );
    }
    if let Some(index) = ip.checked_sub(2).filter(|index| *index <= ilimit) {
        set_entry(hash_long, hash8_ptr(src, index, h_bits_l), index as u32);
    }
    if let Some(index) = ip.checked_sub(1).filter(|index| *index <= ilimit) {
        set_entry(
            hash_small,
            hash_small_ptr::<MIN_MATCH>(src, index, h_bits_s),
            index as u32,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn consume_immediate_repcodes<const MIN_MATCH: u32>(
    src: &[u8],
    hash_long: &mut [u32],
    hash_small: &mut [u32],
    h_bits_l: u32,
    h_bits_s: u32,
    sequences: &mut crate::workspace::ReusableVec<StoredSequence>,
    anchor: &mut usize,
    ip: &mut usize,
    ilimit: usize,
    offset_1: &mut usize,
    offset_2: &mut usize,
    block_end: usize,
    dict_start_index: usize,
    prefix_start_index: usize,
) {
    while *ip <= ilimit && rep_match(src, *ip, *offset_2, dict_start_index, prefix_start_index) {
        let rep_index = *ip - *offset_2;
        let repeat_length = count_match(src, *ip + 4, rep_index + 4, block_end) + 4;
        core::mem::swap(offset_1, offset_2);
        set_entry(
            hash_small,
            hash_small_ptr::<MIN_MATCH>(src, *ip, h_bits_s),
            *ip as u32,
        );
        set_entry(hash_long, hash8_ptr(src, *ip, h_bits_l), *ip as u32);
        store_match(
            sequences,
            anchor,
            ip,
            OffBase::Repeat(RepeatCode::First),
            repeat_length,
        );
    }
}

fn rep_match_at_next(
    src: &[u8],
    current: usize,
    offset: usize,
    dict_start_index: usize,
    prefix_start_index: usize,
) -> bool {
    offset <= current + 1 - dict_start_index
        && rep_match(
            src,
            current + 1,
            offset,
            dict_start_index,
            prefix_start_index,
        )
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
        && read32(src, rep_index) == read32(src, current)
}

fn index_overlap_check(prefix_lowest_index: usize, rep_index: usize) -> bool {
    prefix_lowest_index.wrapping_sub(1).wrapping_sub(rep_index) >= 3
}
