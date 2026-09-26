use core::ops::Range;

use super::{LdmEntry, LdmHashTable, LdmRollingHashState, LDM_BATCH_SIZE};
use crate::encoding::levels::c_port::{cctx_params::LdmParameters, match_count::count_match};
use crate::workspace::ReusableVec;

const HASH_READ_SIZE: usize = 8;
const LDM_MAX_CHUNK_SIZE: usize = 1 << 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LdmRawSequence {
    pub(crate) offset: u32,
    pub(crate) lit_length: u32,
    pub(crate) match_length: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LdmSequenceResult {
    pub(crate) sequences: ReusableVec<LdmRawSequence>,
    pub(crate) last_literals: usize,
}

pub(crate) fn generate_sequences_no_dict(
    src: &[u8],
    params: LdmParameters,
    table: &mut LdmHashTable,
) -> LdmSequenceResult {
    generate_sequences_no_dict_in(src, params, table, ReusableVec::new())
}

pub(crate) fn generate_sequences_no_dict_in(
    src: &[u8],
    params: LdmParameters,
    table: &mut LdmHashTable,
    sequences: ReusableVec<LdmRawSequence>,
) -> LdmSequenceResult {
    generate_sequences_in_range(src, 0..src.len(), 0, 0, 0, params, table, sequences)
}

pub(crate) fn fill_prefix_hash_table(
    src: &[u8],
    prefix_range: Range<usize>,
    params: LdmParameters,
    table: &mut LdmHashTable,
) {
    debug_assert!(prefix_range.start <= prefix_range.end);
    debug_assert!(prefix_range.end <= src.len());

    let min_match_length = params.min_match_length as usize;
    if prefix_range.len() < min_match_length {
        return;
    }

    let hash_bits = (params.hash_log - params.bucket_size_log) as usize;
    let hash_mask = (1_usize << hash_bits) - 1;
    let mut ip = prefix_range.start;
    let mut hash_state = LdmRollingHashState::new(params);
    let mut splits = [0_usize; LDM_BATCH_SIZE];

    while ip < prefix_range.end {
        let mut num_splits = 0;
        let hashed = hash_state.feed(&src[ip..prefix_range.end], &mut splits, &mut num_splits);

        for &split_index in &splits[..num_splits] {
            if ip + split_index < prefix_range.start + min_match_length {
                continue;
            }

            let split = ip + split_index - min_match_length;
            let xxhash = xxh64(&src[split..split + min_match_length], 0);
            let hash = (xxhash as usize) & hash_mask;
            table.insert_entry(
                hash,
                LdmEntry {
                    offset: to_u32(split),
                    checksum: (xxhash >> 32) as u32,
                },
            );
        }

        ip += hashed;
    }
}

pub(crate) fn generate_sequences_with_prefix(
    src: &[u8],
    source_range: Range<usize>,
    params: LdmParameters,
    table: &mut LdmHashTable,
) -> LdmSequenceResult {
    let dict_limit = source_range.start;
    generate_sequences_in_range(
        src,
        source_range,
        0,
        dict_limit,
        dict_limit,
        params,
        table,
        ReusableVec::new(),
    )
}

#[allow(clippy::too_many_arguments)]
fn generate_sequences_in_range(
    src: &[u8],
    source_range: Range<usize>,
    dict_low_limit: usize,
    dict_limit: usize,
    loaded_dict_end: usize,
    params: LdmParameters,
    table: &mut LdmHashTable,
    mut sequences: ReusableVec<LdmRawSequence>,
) -> LdmSequenceResult {
    debug_assert!(source_range.start <= source_range.end);
    debug_assert!(source_range.end <= src.len());
    debug_assert!(dict_low_limit <= dict_limit);
    debug_assert!(dict_limit <= source_range.start);

    sequences.clear();
    let mut last_literals = 0;
    let mut chunk_start = source_range.start;
    while chunk_start < source_range.end {
        let chunk_end = (chunk_start + LDM_MAX_CHUNK_SIZE).min(source_range.end);
        let (chunk_dict_low_limit, chunk_dict_limit) = enforce_max_distance(
            dict_low_limit,
            dict_limit,
            loaded_dict_end,
            params,
            chunk_end,
        );
        let sequence_start = sequences.len();
        let chunk_last_literals = generate_sequences_in_chunk_into(
            src,
            chunk_start..chunk_end,
            chunk_dict_low_limit,
            chunk_dict_limit,
            params,
            table,
            &mut sequences,
        );

        if sequences.len() == sequence_start {
            last_literals += chunk_last_literals;
        } else {
            sequences[sequence_start].lit_length += to_u32(last_literals);
            last_literals = chunk_last_literals;
        }

        chunk_start = chunk_end;
    }

    LdmSequenceResult {
        sequences,
        last_literals,
    }
}

fn enforce_max_distance(
    dict_low_limit: usize,
    dict_limit: usize,
    loaded_dict_end: usize,
    params: LdmParameters,
    chunk_end: usize,
) -> (usize, usize) {
    let max_distance = 1_usize << params.window_log;
    if chunk_end <= max_distance.saturating_add(loaded_dict_end) {
        return (dict_low_limit, dict_limit);
    }

    let low_limit = dict_low_limit.max(chunk_end - max_distance);
    let dict_limit = dict_limit.max(low_limit);
    (low_limit, dict_limit)
}

fn generate_sequences_in_chunk_into(
    src: &[u8],
    source_range: Range<usize>,
    dict_low_limit: usize,
    dict_limit: usize,
    params: LdmParameters,
    table: &mut LdmHashTable,
    sequences: &mut ReusableVec<LdmRawSequence>,
) -> usize {
    let min_match_length = params.min_match_length as usize;
    let source_len = source_range.len();
    if source_len < min_match_length || source_len <= HASH_READ_SIZE {
        return source_len;
    }

    let hash_bits = (params.hash_log - params.bucket_size_log) as usize;
    let hash_mask = (1_usize << hash_bits) - 1;
    let ilimit = source_range.end - HASH_READ_SIZE;
    let mut anchor = source_range.start;
    let mut ip = source_range.start;
    let mut hash_state = LdmRollingHashState::new(params);
    let mut splits = [0_usize; LDM_BATCH_SIZE];

    hash_state.reset(&src[source_range.start..], min_match_length);
    ip += min_match_length;

    while ip < ilimit {
        let mut num_splits = 0;
        let hashed = hash_state.feed(&src[ip..ilimit], &mut splits, &mut num_splits);

        for &split_index in &splits[..num_splits] {
            let split = ip + split_index - min_match_length;
            let xxhash = xxh64(&src[split..split + min_match_length], 0);
            let hash = (xxhash as usize) & hash_mask;
            let checksum = (xxhash >> 32) as u32;
            let new_entry = LdmEntry {
                offset: to_u32(split),
                checksum,
            };

            if split < anchor {
                table.insert_entry(hash, new_entry);
                continue;
            }

            let mut forward_match_length = 0;
            let mut backward_match_length = 0;
            let mut best_match_length = 0;
            let mut best_entry_offset = None;

            for entry in table.bucket(hash) {
                let match_index = entry.offset as usize;
                if entry.checksum != checksum || match_index <= dict_low_limit {
                    continue;
                }

                let current_forward = count_match(src, split, match_index, source_range.end);
                if current_forward < min_match_length {
                    continue;
                }

                let match_low_prefix = if match_index < dict_limit {
                    dict_low_limit
                } else {
                    dict_limit
                };
                let current_backward =
                    count_backwards_match(src, split, anchor, match_index, match_low_prefix);
                let current_total = current_forward + current_backward;
                if current_total > best_match_length {
                    best_match_length = current_total;
                    forward_match_length = current_forward;
                    backward_match_length = current_backward;
                    best_entry_offset = Some(entry.offset);
                }
            }

            let Some(best_entry_offset) = best_entry_offset else {
                table.insert_entry(hash, new_entry);
                continue;
            };

            let match_length = forward_match_length + backward_match_length;
            sequences.push(LdmRawSequence {
                offset: to_u32(split - best_entry_offset as usize),
                lit_length: to_u32(split - backward_match_length - anchor),
                match_length: to_u32(match_length),
            });

            table.insert_entry(hash, new_entry);
            anchor = split + forward_match_length;

            if anchor > ip + hashed {
                hash_state.reset(&src[anchor - min_match_length..], min_match_length);
                ip = anchor - hashed;
                break;
            }
        }

        ip += hashed;
    }

    source_range.end - anchor
}

fn count_backwards_match(
    src: &[u8],
    mut pos: usize,
    anchor: usize,
    mut match_pos: usize,
    low_prefix: usize,
) -> usize {
    let start = pos;
    while pos > anchor && match_pos > low_prefix && src[pos - 1] == src[match_pos - 1] {
        pos -= 1;
        match_pos -= 1;
    }
    start - pos
}

fn to_u32(value: usize) -> u32 {
    debug_assert!(value <= u32::MAX as usize);
    value as u32
}

fn xxh64(data: &[u8], seed: u64) -> u64 {
    let mut input = data;
    let mut hash;

    if input.len() >= 32 {
        let mut v1 = seed.wrapping_add(PRIME64_1).wrapping_add(PRIME64_2);
        let mut v2 = seed.wrapping_add(PRIME64_2);
        let mut v3 = seed;
        let mut v4 = seed.wrapping_sub(PRIME64_1);

        while input.len() >= 32 {
            v1 = xxh64_round(v1, read64(input, 0));
            v2 = xxh64_round(v2, read64(input, 8));
            v3 = xxh64_round(v3, read64(input, 16));
            v4 = xxh64_round(v4, read64(input, 24));
            input = &input[32..];
        }

        hash = v1
            .rotate_left(1)
            .wrapping_add(v2.rotate_left(7))
            .wrapping_add(v3.rotate_left(12))
            .wrapping_add(v4.rotate_left(18));
        hash = xxh64_merge_round(hash, v1);
        hash = xxh64_merge_round(hash, v2);
        hash = xxh64_merge_round(hash, v3);
        hash = xxh64_merge_round(hash, v4);
    } else {
        hash = seed.wrapping_add(PRIME64_5);
    }

    hash = hash.wrapping_add(data.len() as u64);

    while input.len() >= 8 {
        let k1 = xxh64_round(0, read64(input, 0));
        hash ^= k1;
        hash = hash
            .rotate_left(27)
            .wrapping_mul(PRIME64_1)
            .wrapping_add(PRIME64_4);
        input = &input[8..];
    }

    if input.len() >= 4 {
        hash ^= (read32(input, 0) as u64).wrapping_mul(PRIME64_1);
        hash = hash
            .rotate_left(23)
            .wrapping_mul(PRIME64_2)
            .wrapping_add(PRIME64_3);
        input = &input[4..];
    }

    for &byte in input {
        hash ^= u64::from(byte).wrapping_mul(PRIME64_5);
        hash = hash.rotate_left(11).wrapping_mul(PRIME64_1);
    }

    xxh64_avalanche(hash)
}

#[inline(always)]
fn xxh64_round(acc: u64, input: u64) -> u64 {
    acc.wrapping_add(input.wrapping_mul(PRIME64_2))
        .rotate_left(31)
        .wrapping_mul(PRIME64_1)
}

#[inline(always)]
fn xxh64_merge_round(acc: u64, value: u64) -> u64 {
    let mixed = acc ^ xxh64_round(0, value);
    mixed.wrapping_mul(PRIME64_1).wrapping_add(PRIME64_4)
}

#[inline(always)]
fn xxh64_avalanche(mut hash: u64) -> u64 {
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(PRIME64_2);
    hash ^= hash >> 29;
    hash = hash.wrapping_mul(PRIME64_3);
    hash ^= hash >> 32;
    hash
}

#[inline(always)]
fn read32(input: &[u8], pos: usize) -> u32 {
    debug_assert!(pos + 4 <= input.len());
    let bytes = [input[pos], input[pos + 1], input[pos + 2], input[pos + 3]];
    u32::from_le_bytes(bytes)
}

#[inline(always)]
fn read64(input: &[u8], pos: usize) -> u64 {
    debug_assert!(pos + 8 <= input.len());
    let bytes = [
        input[pos],
        input[pos + 1],
        input[pos + 2],
        input[pos + 3],
        input[pos + 4],
        input[pos + 5],
        input[pos + 6],
        input[pos + 7],
    ];
    u64::from_le_bytes(bytes)
}

const PRIME64_1: u64 = 0x9e37_79b1_85eb_ca87;
const PRIME64_2: u64 = 0xc2b2_ae3d_27d4_eb4f;
const PRIME64_3: u64 = 0x1656_67b1_9e37_79f9;
const PRIME64_4: u64 = 0x85eb_ca77_c2b2_ae63;
const PRIME64_5: u64 = 0x27d4_eb2f_1656_67c5;
