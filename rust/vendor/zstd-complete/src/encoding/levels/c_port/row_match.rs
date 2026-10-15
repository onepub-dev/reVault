//! Scalar row-based match finder ported from the no-dictionary row path in
//! `zstd_lazy.c`.

use super::{
    greedy::GreedyMatchState,
    hash_chain_match::MatchSearchConfig,
    params::CompressionParameters,
    sequence_store::OffBase,
    unaligned::{read32, read64},
};

const TAG_BITS: u32 = 8;
const TAG_MASK: u32 = (1 << TAG_BITS) - 1;
const SKIP_THRESHOLD: usize = 384;
const MAX_MATCH_START_POSITIONS_TO_UPDATE: usize = 96;
const MAX_MATCH_END_POSITIONS_TO_UPDATE: usize = 32;
const ROW_HASH_CACHE_SIZE: usize = 8;
const ROW_HASH_CACHE_MASK: usize = ROW_HASH_CACHE_SIZE - 1;

pub(super) fn row_find_best_match<const ATTACHED_DICT: bool>(
    src: &[u8],
    ip: usize,
    block_end: usize,
    off_base: &mut u32,
    state: &mut GreedyMatchState,
    config: MatchSearchConfig<'_>,
    no_dict_search: Option<crate::kernel::row::NoDictSearchFn>,
) -> usize {
    let params = config.params;
    debug_assert_eq!(config.attached_dictionary.is_some(), ATTACHED_DICT);
    if !ATTACHED_DICT {
        // SAFETY: the caller and frame state establish the source bounds and
        // exact row-table/cache relationship required by the selected kernel.
        return unsafe {
            no_dict_search.expect("normal row search is selected once per block")(
                src,
                ip,
                block_end,
                config.lowest_prefix_index(ip),
                params.hash_log,
                params.search_log,
                state.hash_salt,
                state.lazy_skipping,
                &mut state.hash_table,
                &mut state.tag_table,
                &mut state.row_hash_cache,
                &mut state.next_to_update,
                &mut state.hash_salt_entropy,
                off_base,
            )
        };
    }
    debug_assert!(no_dict_search.is_none());
    match row_log(params) {
        4 => row_find_best_match_impl::<4, ATTACHED_DICT>(
            src, ip, block_end, off_base, state, config,
        ),
        5 => row_find_best_match_impl::<5, ATTACHED_DICT>(
            src, ip, block_end, off_base, state, config,
        ),
        6 => row_find_best_match_impl::<6, ATTACHED_DICT>(
            src, ip, block_end, off_base, state, config,
        ),
        _ => unreachable!("row_log is clamped to 4..=6"),
    }
}

fn row_find_best_match_impl<const ROW_LOG: u32, const ATTACHED_DICT: bool>(
    src: &[u8],
    ip: usize,
    block_end: usize,
    off_base: &mut u32,
    state: &mut GreedyMatchState,
    config: MatchSearchConfig<'_>,
) -> usize {
    let params = config.params;
    let min_match = config.min_match;
    let row_entries = 1usize << ROW_LOG;
    let row_mask = row_entries - 1;
    let row_hash_log = params.hash_log - ROW_LOG;
    let max_attempts = 1usize << params.search_log.min(ROW_LOG);
    let curr = ip;
    let low_limit = config.lowest_prefix_index(curr);

    let hash = if state.lazy_skipping {
        state.next_to_update = curr;
        hash_ptr_salted(
            src,
            curr,
            row_hash_log + TAG_BITS,
            min_match,
            state.hash_salt,
        )
    } else {
        update_rows::<ROW_LOG>(src, curr, params, min_match, state);
        next_cached_hash(src, curr, row_hash_log, ROW_LOG, min_match, state)
    };

    state.hash_salt_entropy = state.hash_salt_entropy.wrapping_add(hash);

    let row_start = ((hash >> TAG_BITS) as usize) << ROW_LOG;
    let tag = (hash & TAG_MASK) as u8;
    let tag_row = super::row_table::tags(&state.tag_table, row_start, row_entries);
    let head = usize::from(tag_row[0] & row_mask as u8);
    let mut matches = row_match_mask(tag_row, tag, head);
    let mut best_len = 3usize;
    let mut attempts = 0usize;

    while matches != 0 && attempts < max_attempts {
        let step = matches.trailing_zeros() as usize;
        matches &= matches - 1;
        let pos = (head + step) & row_mask;
        if pos == 0 {
            continue;
        }

        let match_index =
            super::row_table::entry(&state.hash_table, row_start, pos, row_mask) as usize;
        if match_index < low_limit {
            break;
        }
        if match_index >= curr {
            continue;
        }
        attempts += 1;

        let mut current_len = 0usize;
        if read32(src, match_index + best_len - 3) == read32(src, ip + best_len - 3) {
            current_len = super::hash_chain_match::count_match(src, ip, match_index, block_end);
        }

        if current_len > best_len {
            best_len = current_len;
            *off_base = OffBase::offset_to_c_value((curr - match_index) as u32);
            if ip + current_len == block_end {
                break;
            }
        }
    }

    let insert_pos = super::row_table::next_index(&mut state.tag_table, row_start, row_mask);
    super::row_table::insert(
        &mut state.hash_table,
        &mut state.tag_table,
        row_start,
        insert_pos,
        row_mask,
        tag,
        state.next_to_update as u32,
    );
    state.next_to_update += 1;

    debug_assert_eq!(config.attached_dictionary.is_some(), ATTACHED_DICT);
    if ATTACHED_DICT {
        let attached = config
            .attached_dictionary
            .expect("attached row specialization carries dictionary state");
        best_len = row_find_best_attached_dictionary::<ROW_LOG>(
            src,
            ip,
            block_end,
            off_base,
            best_len,
            max_attempts - attempts,
            attached,
        );
    }

    best_len
}

#[allow(clippy::too_many_arguments)]
fn row_find_best_attached_dictionary<const ROW_LOG: u32>(
    src: &[u8],
    ip: usize,
    block_end: usize,
    off_base: &mut u32,
    mut best_len: usize,
    mut attempts: usize,
    attached: super::hash_chain_match::AttachedDictionarySearch<'_>,
) -> usize {
    let row_entries = 1usize << ROW_LOG;
    let row_mask = row_entries - 1;
    let row_hash_log = attached.params.hash_log - ROW_LOG;
    let dictionary_size = attached.src.len();
    let dictionary_index_end = match attached.dictionary_index_start.checked_add(dictionary_size) {
        Some(end) => end,
        None => return best_len,
    };
    if attempts == 0
        || dictionary_size == 0
        || attached.active_dict_limit < dictionary_index_end
        || attached.active_dict_limit < attached.active_prefix_start
    {
        return best_len;
    }

    let hash = hash_ptr_unsalted(src, ip, row_hash_log + TAG_BITS, attached.params.min_match);
    let row_start = ((hash >> TAG_BITS) as usize) << ROW_LOG;
    let tag = (hash & TAG_MASK) as u8;
    if row_start + row_entries > attached.state.tag_table.len()
        || row_start + row_entries > attached.state.hash_table.len()
    {
        return best_len;
    }

    let tag_row = super::row_table::tags(&attached.state.tag_table, row_start, row_entries);
    let head = usize::from(tag_row[0] & row_mask as u8);
    let mut matches = row_match_mask(tag_row, tag, head);
    let dms_index_delta = attached.active_dict_limit - dictionary_index_end;
    let active_index_delta = attached.active_dict_limit - attached.active_prefix_start;

    while matches != 0 && attempts > 0 {
        let step = matches.trailing_zeros() as usize;
        matches &= matches - 1;
        let pos = (head + step) & row_mask;
        if pos == 0 {
            continue;
        }

        let match_index =
            super::row_table::entry(&attached.state.hash_table, row_start, pos, row_mask) as usize;
        if match_index < attached.dictionary_index_start {
            break;
        }
        let dictionary_offset = match_index - attached.dictionary_index_start;
        if dictionary_offset + 4 > dictionary_size {
            continue;
        }
        attempts -= 1;

        let mut current_len = 0usize;
        if read32(attached.src, dictionary_offset) == read32(src, ip) {
            current_len = count_match_2segments(
                src,
                ip + 4,
                attached.src,
                dictionary_offset + 4,
                block_end,
                attached.active_prefix_start,
            ) + 4;
        }

        if current_len > best_len {
            best_len = current_len;
            let translated_match_index = match_index + dms_index_delta - active_index_delta;
            debug_assert!(ip > translated_match_index);
            *off_base = OffBase::offset_to_c_value((ip - translated_match_index) as u32);
            if ip + current_len == block_end {
                break;
            }
        }
    }

    best_len
}

fn count_match_2segments(
    src: &[u8],
    mut ip: usize,
    dict: &[u8],
    mut match_index: usize,
    block_end: usize,
    prefix_start: usize,
) -> usize {
    let start = ip;
    while ip < block_end {
        let match_byte = if match_index < dict.len() {
            dict[match_index]
        } else {
            let prefix_index = prefix_start + (match_index - dict.len());
            if prefix_index >= src.len() {
                break;
            }
            src[prefix_index]
        };
        if src[ip] != match_byte {
            break;
        }
        ip += 1;
        match_index += 1;
    }
    ip - start
}

fn update_rows<const ROW_LOG: u32>(
    src: &[u8],
    target: usize,
    params: CompressionParameters,
    min_match: u32,
    state: &mut GreedyMatchState,
) {
    update_rows_internal::<ROW_LOG>(src, target, 0, params, min_match, state, true);
}

pub(super) fn fill_hash_cache(
    src: &[u8],
    idx: usize,
    limit: usize,
    params: CompressionParameters,
    min_match: u32,
    state: &mut GreedyMatchState,
) {
    match row_log(params) {
        4 => fill_hash_cache_impl::<4>(src, idx, limit, params, min_match, state),
        5 => fill_hash_cache_impl::<5>(src, idx, limit, params, min_match, state),
        6 => fill_hash_cache_impl::<6>(src, idx, limit, params, min_match, state),
        _ => unreachable!("row_log is clamped to 4..=6"),
    }
}

fn fill_hash_cache_impl<const ROW_LOG: u32>(
    src: &[u8],
    mut idx: usize,
    limit: usize,
    params: CompressionParameters,
    min_match: u32,
    state: &mut GreedyMatchState,
) {
    let row_hash_log = params.hash_log - ROW_LOG;
    let end = idx
        .saturating_add(ROW_HASH_CACHE_SIZE)
        .min(limit.saturating_add(1));
    while idx < end {
        let hash = hash_ptr_salted(
            src,
            idx,
            row_hash_log + TAG_BITS,
            min_match,
            state.hash_salt,
        );
        #[cfg(target_arch = "x86_64")]
        prefetch_row(state, ((hash >> TAG_BITS) as usize) << ROW_LOG, ROW_LOG);
        state.row_hash_cache[idx & ROW_HASH_CACHE_MASK] = hash;
        idx += 1;
    }
}

pub(super) fn load_dictionary_rows(
    src: &[u8],
    target: usize,
    params: CompressionParameters,
    min_match: u32,
    state: &mut GreedyMatchState,
) {
    load_dictionary_rows_at_index_base(src, target, 0, params, min_match, state);
}

pub(super) fn load_dictionary_rows_at_index_base(
    src: &[u8],
    target: usize,
    index_base: usize,
    params: CompressionParameters,
    min_match: u32,
    state: &mut GreedyMatchState,
) {
    match row_log(params) {
        4 => update_rows_internal::<4>(src, target, index_base, params, min_match, state, false),
        5 => update_rows_internal::<5>(src, target, index_base, params, min_match, state, false),
        6 => update_rows_internal::<6>(src, target, index_base, params, min_match, state, false),
        _ => unreachable!("row_log is clamped to 4..=6"),
    }
}

fn update_rows_internal<const ROW_LOG: u32>(
    src: &[u8],
    target: usize,
    index_base: usize,
    params: CompressionParameters,
    min_match: u32,
    state: &mut GreedyMatchState,
    use_cache_skip: bool,
) {
    let row_mask = (1usize << ROW_LOG) - 1;
    let row_hash_log = params.hash_log - ROW_LOG;
    let target_index = index_base + target;
    let mut idx = state.next_to_update;

    debug_assert!(idx >= index_base);
    debug_assert!(target <= src.len());
    debug_assert!(!use_cache_skip || index_base == 0);

    if use_cache_skip && target_index.saturating_sub(idx) > SKIP_THRESHOLD {
        let start_bound = idx + MAX_MATCH_START_POSITIONS_TO_UPDATE;
        update_rows_range(
            src,
            idx,
            start_bound,
            index_base,
            row_hash_log,
            ROW_LOG,
            row_mask,
            min_match,
            state,
            use_cache_skip,
        );
        idx = target_index - MAX_MATCH_END_POSITIONS_TO_UPDATE;
        fill_hash_cache(src, idx, target, params, min_match, state);
    }

    update_rows_range(
        src,
        idx,
        target_index,
        index_base,
        row_hash_log,
        ROW_LOG,
        row_mask,
        min_match,
        state,
        use_cache_skip,
    );
    state.next_to_update = target_index;
}

#[allow(clippy::too_many_arguments)]
fn update_rows_range(
    src: &[u8],
    mut idx: usize,
    target: usize,
    index_base: usize,
    row_hash_log: u32,
    row_log: u32,
    row_mask: usize,
    min_match: u32,
    state: &mut GreedyMatchState,
    use_cache: bool,
) {
    while idx < target {
        let source_pos = idx - index_base;
        let hash = if use_cache {
            debug_assert_eq!(index_base, 0);
            next_cached_hash(src, source_pos, row_hash_log, row_log, min_match, state)
        } else {
            hash_ptr_salted(
                src,
                source_pos,
                row_hash_log + TAG_BITS,
                min_match,
                state.hash_salt,
            )
        };
        let row_start = ((hash >> TAG_BITS) as usize) << row_log;
        let pos = super::row_table::next_index(&mut state.tag_table, row_start, row_mask);
        super::row_table::insert(
            &mut state.hash_table,
            &mut state.tag_table,
            row_start,
            pos,
            row_mask,
            (hash & TAG_MASK) as u8,
            idx as u32,
        );
        idx += 1;
    }
}

fn next_cached_hash(
    src: &[u8],
    idx: usize,
    row_hash_log: u32,
    row_log: u32,
    min_match: u32,
    state: &mut GreedyMatchState,
) -> u32 {
    #[cfg(not(target_arch = "x86_64"))]
    let _ = row_log;
    let new_hash = hash_ptr_salted(
        src,
        idx + ROW_HASH_CACHE_SIZE,
        row_hash_log + TAG_BITS,
        min_match,
        state.hash_salt,
    );
    #[cfg(target_arch = "x86_64")]
    prefetch_row(state, ((new_hash >> TAG_BITS) as usize) << row_log, row_log);
    let cached = state.row_hash_cache[idx & ROW_HASH_CACHE_MASK];
    state.row_hash_cache[idx & ROW_HASH_CACHE_MASK] = new_hash;
    cached
}

#[cfg(target_arch = "x86_64")]
fn prefetch_row(state: &GreedyMatchState, row_start: usize, row_log: u32) {
    prefetch_read(state.hash_table.as_ptr().wrapping_add(row_start));
    if row_log >= 5 {
        prefetch_read(state.hash_table.as_ptr().wrapping_add(row_start + 16));
    }
    prefetch_read(state.tag_table.as_ptr().wrapping_add(row_start));
    if row_log == 6 {
        prefetch_read(state.tag_table.as_ptr().wrapping_add(row_start + 32));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn prefetch_read<T>(ptr: *const T) {
    super::x86::prefetch_read(ptr);
}

#[inline(always)]
fn row_match_mask(tag_row: &[u8], tag: u8, head: usize) -> u64 {
    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    {
        row_match_mask_sse2(tag_row, tag, head)
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
    {
        row_match_mask_scalar(tag_row, tag, head)
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[inline(always)]
fn row_match_mask_sse2(tag_row: &[u8], tag: u8, head: usize) -> u64 {
    debug_assert!(matches!(tag_row.len(), 16 | 32 | 64));
    debug_assert!(head < tag_row.len());

    rotate_right_within(
        super::x86::row_tag_match_mask(tag_row, tag),
        head,
        tag_row.len(),
    )
}

#[cfg(any(test, not(all(target_arch = "x86_64", target_feature = "sse2"))))]
#[inline(always)]
fn row_match_mask_scalar(tag_row: &[u8], tag: u8, head: usize) -> u64 {
    debug_assert!(matches!(tag_row.len(), 16 | 32 | 64));
    debug_assert!(head < tag_row.len());

    let mut matches = 0u64;
    let splat = u64::from_le_bytes([tag; 8]);
    let mut chunk_start = 0usize;
    while chunk_start < tag_row.len() {
        let chunk = read64(tag_row, chunk_start) ^ splat;
        let zero_bytes = chunk.wrapping_sub(0x0101_0101_0101_0101) & !chunk & 0x8080_8080_8080_8080;
        matches |= byte_high_bits_to_mask(zero_bytes) << chunk_start;
        chunk_start += 8;
    }

    rotate_right_within(matches, head, tag_row.len())
}

#[inline(always)]
fn byte_high_bits_to_mask(high_bits: u64) -> u64 {
    ((high_bits >> 7).wrapping_mul(0x0102_0408_1020_4080) >> 56) & 0xff
}

#[inline(always)]
fn rotate_right_within(value: u64, shift: usize, width: usize) -> u64 {
    debug_assert!((1..=64).contains(&width));
    debug_assert!(shift < width);
    let mask = if width == 64 {
        u64::MAX
    } else {
        (1u64 << width) - 1
    };
    if shift == 0 {
        value & mask
    } else {
        ((value >> shift) | (value << (width - shift))) & mask
    }
}

pub(super) fn row_log(params: CompressionParameters) -> u32 {
    params.search_log.clamp(4, 6)
}

pub(super) fn row_match_finder_enabled(params: CompressionParameters) -> bool {
    matches!(
        params.strategy,
        super::params::Strategy::Greedy
            | super::params::Strategy::Lazy
            | super::params::Strategy::Lazy2
    ) && params.window_log > 14
}

#[inline(always)]
fn hash_ptr_salted(src: &[u8], pos: usize, h_bits: u32, min_match: u32, salt: u64) -> u32 {
    match min_match {
        5 => hash5(read64(src, pos), h_bits, salt),
        6 => hash6(read64(src, pos), h_bits, salt),
        _ => hash4(read32(src, pos), h_bits, salt as u32),
    }
}

#[inline(always)]
fn hash_ptr_unsalted(src: &[u8], pos: usize, h_bits: u32, min_match: u32) -> u32 {
    hash_ptr_salted(src, pos, h_bits, min_match, 0)
}

#[inline(always)]
fn hash4(value: u32, h_bits: u32, salt: u32) -> u32 {
    const PRIME_4_BYTES: u32 = 2_654_435_761;
    (value.wrapping_mul(PRIME_4_BYTES) ^ salt).wrapping_shr(32 - h_bits)
}

#[inline(always)]
fn hash5(value: u64, h_bits: u32, salt: u64) -> u32 {
    const PRIME_5_BYTES: u64 = 889_523_592_379;
    (((value << (64 - 40)).wrapping_mul(PRIME_5_BYTES) ^ salt) >> (64 - h_bits)) as u32
}

#[inline(always)]
fn hash6(value: u64, h_bits: u32, salt: u64) -> u32 {
    const PRIME_6_BYTES: u64 = 227_718_039_650_203;
    (((value << (64 - 48)).wrapping_mul(PRIME_6_BYTES) ^ salt) >> (64 - h_bits)) as u32
}

#[cfg(test)]
mod tests;
