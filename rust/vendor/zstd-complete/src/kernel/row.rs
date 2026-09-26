use core::mem::MaybeUninit;

const TAG_BITS: u32 = 8;
const TAG_MASK: u32 = (1 << TAG_BITS) - 1;
const HASH_CACHE_SIZE: usize = 8;
const HASH_CACHE_MASK: usize = HASH_CACHE_SIZE - 1;
const SKIP_THRESHOLD: usize = 384;
const MAX_MATCH_START_POSITIONS_TO_UPDATE: usize = 96;
const MAX_MATCH_END_POSITIONS_TO_UPDATE: usize = 32;
const MAX_ROW_ENTRIES: usize = 64;
const C_REPCODE_COUNT: u32 = 3;

/// Primitive result returned by one complete no-dictionary row parser.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoDictBlockResult {
    pub sequence_count: usize,
    pub last_literals: u32,
    pub repeat_offsets: [u32; C_REPCODE_COUNT as usize],
    pub lazy_skipping: bool,
}

/// Complete primitive ABI for one generated Greedy/Lazy row parser.
pub type NoDictBlockFn = unsafe fn(
    src: &[u8],
    block_start: usize,
    block_end: usize,
    loaded_dict_end: usize,
    window_log: u32,
    hash_log: u32,
    search_log: u32,
    hash_salt: u64,
    repeat_offsets: [u32; C_REPCODE_COUNT as usize],
    hash_table: &mut [u32],
    tag_table: &mut [u8],
    hash_cache: &mut [u32; HASH_CACHE_SIZE],
    next_to_update: &mut usize,
    hash_salt_entropy: &mut u32,
    sequences: &mut [MaybeUninit<[u32; 3]>],
) -> NoDictBlockResult;

/// Primitive ABI of one generated no-dictionary row-search specialization.
pub type NoDictSearchFn = unsafe fn(
    src: &[u8],
    ip: usize,
    block_end: usize,
    low_limit: usize,
    hash_log: u32,
    search_log: u32,
    hash_salt: u64,
    lazy_skipping: bool,
    hash_table: &mut [u32],
    tag_table: &mut [u8],
    hash_cache: &mut [u32; HASH_CACHE_SIZE],
    next_to_update: &mut usize,
    hash_salt_entropy: &mut u32,
    off_base: &mut u32,
) -> usize;

/// Selects C's generated row-search function once per source block.
pub fn select_best_match_no_dict(min_match: u32, search_log: u32) -> NoDictSearchFn {
    match (min_match, search_log.clamp(4, 6)) {
        (4, 4) => find_best_match_impl::<4, 4>,
        (4, 5) => find_best_match_impl::<4, 5>,
        (4, 6) => find_best_match_impl::<4, 6>,
        (5, 4) => find_best_match_impl::<5, 4>,
        (5, 5) => find_best_match_impl::<5, 5>,
        (5, 6) => find_best_match_impl::<5, 6>,
        (6, 4) => find_best_match_impl::<6, 4>,
        (6, 5) => find_best_match_impl::<6, 5>,
        (6, 6) => find_best_match_impl::<6, 6>,
        _ => unreachable!("minMatch and rowLog are clamped to 4..=6"),
    }
}

/// Selects the complete C-shaped no-dictionary row parser once per block.
pub fn select_block_no_dict(depth: u32, min_match: u32, search_log: u32) -> NoDictBlockFn {
    match (depth, min_match, search_log.clamp(4, 6)) {
        (0, 4, 4) => compress_block_no_dict_impl::<0, 4, 4>,
        (0, 4, 5) => compress_block_no_dict_impl::<0, 4, 5>,
        (0, 4, 6) => compress_block_no_dict_impl::<0, 4, 6>,
        (0, 5, 4) => compress_block_no_dict_impl::<0, 5, 4>,
        (0, 5, 5) => compress_block_no_dict_impl::<0, 5, 5>,
        (0, 5, 6) => compress_block_no_dict_impl::<0, 5, 6>,
        (0, 6, 4) => compress_block_no_dict_impl::<0, 6, 4>,
        (0, 6, 5) => compress_block_no_dict_impl::<0, 6, 5>,
        (0, 6, 6) => compress_block_no_dict_impl::<0, 6, 6>,
        (1, 4, 4) => compress_block_no_dict_impl::<1, 4, 4>,
        (1, 4, 5) => compress_block_no_dict_impl::<1, 4, 5>,
        (1, 4, 6) => compress_block_no_dict_impl::<1, 4, 6>,
        (1, 5, 4) => compress_block_no_dict_impl::<1, 5, 4>,
        (1, 5, 5) => compress_block_no_dict_impl::<1, 5, 5>,
        (1, 5, 6) => compress_block_no_dict_impl::<1, 5, 6>,
        (1, 6, 4) => compress_block_no_dict_impl::<1, 6, 4>,
        (1, 6, 5) => compress_block_no_dict_impl::<1, 6, 5>,
        (1, 6, 6) => compress_block_no_dict_impl::<1, 6, 6>,
        (2, 4, 4) => compress_block_no_dict_impl::<2, 4, 4>,
        (2, 4, 5) => compress_block_no_dict_impl::<2, 4, 5>,
        (2, 4, 6) => compress_block_no_dict_impl::<2, 4, 6>,
        (2, 5, 4) => compress_block_no_dict_impl::<2, 5, 4>,
        (2, 5, 5) => compress_block_no_dict_impl::<2, 5, 5>,
        (2, 5, 6) => compress_block_no_dict_impl::<2, 5, 6>,
        (2, 6, 4) => compress_block_no_dict_impl::<2, 6, 4>,
        (2, 6, 5) => compress_block_no_dict_impl::<2, 6, 5>,
        (2, 6, 6) => compress_block_no_dict_impl::<2, 6, 6>,
        _ => unreachable!("depth is 0..=2 and row parameters are clamped to 4..=6"),
    }
}

#[allow(clippy::too_many_arguments)]
#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_rowp")]
#[cfg_attr(target_family = "windows", link_section = ".text$012.rz.rowp")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.012.ruzstd.row.parser"
)]
fn compress_block_no_dict_impl<const DEPTH: u32, const MIN_MATCH: u32, const ROW_LOG: u32>(
    src: &[u8],
    block_start: usize,
    block_end: usize,
    loaded_dict_end: usize,
    window_log: u32,
    hash_log: u32,
    search_log: u32,
    hash_salt: u64,
    repeat_offsets: [u32; C_REPCODE_COUNT as usize],
    hash_table: &mut [u32],
    tag_table: &mut [u8],
    hash_cache: &mut [u32; HASH_CACHE_SIZE],
    next_to_update: &mut usize,
    hash_salt_entropy: &mut u32,
    sequences: &mut [MaybeUninit<[u32; 3]>],
) -> NoDictBlockResult {
    const {
        assert!(DEPTH <= 2);
        assert!(MIN_MATCH >= 4 && MIN_MATCH <= 6);
        assert!(ROW_LOG >= 4 && ROW_LOG <= 6);
    }
    debug_assert!(block_start <= block_end);
    debug_assert!(block_end <= src.len());
    debug_assert_eq!(hash_table.len(), 1usize << hash_log);
    debug_assert_eq!(hash_table.len(), tag_table.len());

    let block_len = block_end - block_start;
    if block_len <= HASH_CACHE_SIZE + 8 {
        return NoDictBlockResult {
            sequence_count: 0,
            last_literals: block_len as u32,
            repeat_offsets,
            lazy_skipping: false,
        };
    }

    let prefix_start = lowest_prefix_index(block_end, window_log, loaded_dict_end);
    let ilimit = block_end - HASH_CACHE_SIZE - 8;
    let row_hash_log = hash_log - ROW_LOG;
    fill_hash_cache::<MIN_MATCH, ROW_LOG>(
        src,
        *next_to_update,
        ilimit,
        row_hash_log,
        hash_salt,
        hash_table,
        tag_table,
        hash_cache,
    );

    let mut rep = repeat_offsets;
    let mut ip = block_start + usize::from(block_start == prefix_start);
    let mut anchor = block_start;
    let mut sequence_count = 0usize;
    let mut lazy_skipping = false;
    let mut offset_1 = rep[0] as usize;
    let mut offset_2 = rep[1] as usize;
    let mut offset_saved1 = 0usize;
    let mut offset_saved2 = 0usize;

    let window_low = lowest_prefix_index(ip, window_log, loaded_dict_end);
    let max_rep = ip - window_low;
    if offset_2 > max_rep {
        offset_saved2 = offset_2;
        offset_2 = 0;
    }
    if offset_1 > max_rep {
        offset_saved1 = offset_1;
        offset_1 = 0;
    }

    while ip < ilimit {
        let mut match_length = 0usize;
        let mut off_base = 1u32;
        let mut start = ip + 1;

        let rep_length = rep_match_length(src, ip + 1, offset_1, block_end);
        if rep_length >= 4 {
            match_length = rep_length;
            if DEPTH == 0 {
                store_sequence(
                    sequences,
                    &mut sequence_count,
                    &mut anchor,
                    &mut ip,
                    start,
                    off_base,
                    match_length,
                );
                continue_immediate_repcodes(
                    src,
                    sequences,
                    &mut sequence_count,
                    &mut anchor,
                    &mut ip,
                    ilimit,
                    block_end,
                    &mut offset_1,
                    &mut offset_2,
                );
                continue;
            }
        }

        let mut candidate_off_base = u32::MAX;
        let candidate_length = find_best_match_impl::<MIN_MATCH, ROW_LOG>(
            src,
            ip,
            block_end,
            lowest_prefix_index(ip, window_log, loaded_dict_end),
            hash_log,
            search_log,
            hash_salt,
            lazy_skipping,
            hash_table,
            tag_table,
            hash_cache,
            next_to_update,
            hash_salt_entropy,
            &mut candidate_off_base,
        );
        if candidate_length > match_length {
            match_length = candidate_length;
            start = ip;
            off_base = candidate_off_base;
        }

        if match_length < 4 {
            let step = ((ip - anchor) >> 8) + 1;
            ip += step;
            lazy_skipping = step > 8;
            continue;
        }

        if DEPTH >= 1 {
            loop {
                ip += 1;
                if off_base != 0 {
                    let ml_rep = rep_match_length(src, ip, offset_1, block_end);
                    let gain2 = (ml_rep * 3) as i32;
                    let gain1 = (match_length * 3) as i32 - highbit32(off_base) as i32 + 1;
                    if ml_rep >= 4 && gain2 > gain1 {
                        match_length = ml_rep;
                        off_base = 1;
                        start = ip;
                    }
                }

                let mut next_off_base = u32::MAX;
                let next_length = find_best_match_impl::<MIN_MATCH, ROW_LOG>(
                    src,
                    ip,
                    block_end,
                    lowest_prefix_index(ip, window_log, loaded_dict_end),
                    hash_log,
                    search_log,
                    hash_salt,
                    lazy_skipping,
                    hash_table,
                    tag_table,
                    hash_cache,
                    next_to_update,
                    hash_salt_entropy,
                    &mut next_off_base,
                );
                let gain2 = (next_length * 4) as i32 - highbit32(next_off_base) as i32;
                let gain1 = (match_length * 4) as i32 - highbit32(off_base) as i32 + 4;
                if next_length >= 4 && gain2 > gain1 {
                    match_length = next_length;
                    off_base = next_off_base;
                    start = ip;
                    if ip < ilimit {
                        continue;
                    }
                }

                if DEPTH == 2 && ip < ilimit {
                    ip += 1;
                    if off_base != 0 {
                        let ml_rep = rep_match_length(src, ip, offset_1, block_end);
                        let gain2 = (ml_rep * 4) as i32;
                        let gain1 = (match_length * 4) as i32 - highbit32(off_base) as i32 + 1;
                        if ml_rep >= 4 && gain2 > gain1 {
                            match_length = ml_rep;
                            off_base = 1;
                            start = ip;
                        }
                    }

                    let mut next_off_base = u32::MAX;
                    let next_length = find_best_match_impl::<MIN_MATCH, ROW_LOG>(
                        src,
                        ip,
                        block_end,
                        lowest_prefix_index(ip, window_log, loaded_dict_end),
                        hash_log,
                        search_log,
                        hash_salt,
                        lazy_skipping,
                        hash_table,
                        tag_table,
                        hash_cache,
                        next_to_update,
                        hash_salt_entropy,
                        &mut next_off_base,
                    );
                    let gain2 = (next_length * 4) as i32 - highbit32(next_off_base) as i32;
                    let gain1 = (match_length * 4) as i32 - highbit32(off_base) as i32 + 7;
                    if next_length >= 4 && gain2 > gain1 {
                        match_length = next_length;
                        off_base = next_off_base;
                        start = ip;
                        if ip < ilimit {
                            continue;
                        }
                    }
                }
                break;
            }
        }

        if off_base > C_REPCODE_COUNT {
            let offset = (off_base - C_REPCODE_COUNT) as usize;
            let mut match_pos = start - offset;
            while start > anchor
                && match_pos > prefix_start
                && byte(src, start - 1) == byte(src, match_pos - 1)
            {
                start -= 1;
                match_pos -= 1;
                match_length += 1;
            }
            offset_2 = offset_1;
            offset_1 = offset;
        }

        store_sequence(
            sequences,
            &mut sequence_count,
            &mut anchor,
            &mut ip,
            start,
            off_base,
            match_length,
        );

        if lazy_skipping {
            fill_hash_cache::<MIN_MATCH, ROW_LOG>(
                src,
                *next_to_update,
                ilimit,
                row_hash_log,
                hash_salt,
                hash_table,
                tag_table,
                hash_cache,
            );
            lazy_skipping = false;
        }
        continue_immediate_repcodes(
            src,
            sequences,
            &mut sequence_count,
            &mut anchor,
            &mut ip,
            ilimit,
            block_end,
            &mut offset_1,
            &mut offset_2,
        );
    }

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

    NoDictBlockResult {
        sequence_count,
        last_literals: (block_end - anchor) as u32,
        repeat_offsets: rep,
        lazy_skipping,
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn continue_immediate_repcodes(
    src: &[u8],
    sequences: &mut [MaybeUninit<[u32; 3]>],
    sequence_count: &mut usize,
    anchor: &mut usize,
    ip: &mut usize,
    ilimit: usize,
    block_end: usize,
    offset_1: &mut usize,
    offset_2: &mut usize,
) {
    while *ip <= ilimit {
        let repeat_length = rep_match_length(src, *ip, *offset_2, block_end);
        if repeat_length < 4 {
            break;
        }
        core::mem::swap(offset_2, offset_1);
        store_sequence(
            sequences,
            sequence_count,
            anchor,
            ip,
            *anchor,
            1,
            repeat_length,
        );
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn store_sequence(
    sequences: &mut [MaybeUninit<[u32; 3]>],
    sequence_count: &mut usize,
    anchor: &mut usize,
    ip: &mut usize,
    start: usize,
    off_base: u32,
    match_length: usize,
) {
    debug_assert!(*sequence_count < sequences.len());
    // SAFETY: every stored match consumes at least four new source bytes, so
    // the caller's block-length-derived capacity bounds the written prefix.
    unsafe {
        sequences.get_unchecked_mut(*sequence_count).write([
            (start - *anchor) as u32,
            match_length as u32,
            off_base,
        ]);
    }
    *sequence_count += 1;
    *ip = start + match_length;
    *anchor = *ip;
}

#[inline(always)]
fn rep_match_length(src: &[u8], current: usize, offset: usize, block_end: usize) -> usize {
    if offset == 0 {
        return 0;
    }
    debug_assert!(current >= offset);
    let rep_index = current - offset;
    if read32(src, rep_index) != read32(src, current) {
        return 0;
    }
    count_match(src, current + 4, rep_index + 4, block_end) + 4
}

#[inline(always)]
fn lowest_prefix_index(position: usize, window_log: u32, loaded_dict_end: usize) -> usize {
    let window_size = 1usize << window_log;
    if loaded_dict_end != 0 && position <= loaded_dict_end.saturating_add(window_size) {
        0
    } else {
        position.saturating_sub(window_size)
    }
}

#[inline(always)]
fn highbit32(value: u32) -> u32 {
    debug_assert!(value > 0);
    u32::BITS - 1 - value.leading_zeros()
}

/// Ports C's complete no-dictionary `ZSTD_RowFindBestMatch()` transaction.
///
/// The public boundary deliberately uses only slices and primitive values.
/// Its internal dispatch produces the same nine `(mls, rowLog)` generated
/// functions as C, isolated from the Greedy/Lazy parser's code layout.
#[allow(clippy::too_many_arguments)]
/// Executes a selected row search through the primitive kernel ABI.
///
/// # Safety
///
/// `ip`, `block_end`, and `low_limit` must describe valid positions in `src`;
/// the row tables and cache must have the sizes implied by the hash/search
/// parameters and represent the same prefix. All mutable state must belong to
/// that parser instance.
#[cfg(test)]
pub unsafe fn find_best_match_no_dict(
    src: &[u8],
    ip: usize,
    block_end: usize,
    low_limit: usize,
    hash_log: u32,
    search_log: u32,
    min_match: u32,
    hash_salt: u64,
    lazy_skipping: bool,
    hash_table: &mut [u32],
    tag_table: &mut [u8],
    hash_cache: &mut [u32; HASH_CACHE_SIZE],
    next_to_update: &mut usize,
    hash_salt_entropy: &mut u32,
    off_base: &mut u32,
) -> usize {
    debug_assert!(ip < block_end);
    debug_assert!(block_end <= src.len());
    debug_assert!(low_limit <= ip);
    debug_assert_eq!(hash_table.len(), tag_table.len());
    debug_assert_eq!(hash_table.len(), 1usize << hash_log);
    debug_assert!(matches!(min_match, 4..=6));

    unsafe {
        select_best_match_no_dict(min_match, search_log)(
            src,
            ip,
            block_end,
            low_limit,
            hash_log,
            search_log,
            hash_salt,
            lazy_skipping,
            hash_table,
            tag_table,
            hash_cache,
            next_to_update,
            hash_salt_entropy,
            off_base,
        )
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_rows")]
#[cfg_attr(target_family = "windows", link_section = ".text$013.rz.rows")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.013.ruzstd.row.search"
)]
fn find_best_match_impl<const MIN_MATCH: u32, const ROW_LOG: u32>(
    src: &[u8],
    ip: usize,
    block_end: usize,
    low_limit: usize,
    hash_log: u32,
    search_log: u32,
    hash_salt: u64,
    lazy_skipping: bool,
    hash_table: &mut [u32],
    tag_table: &mut [u8],
    hash_cache: &mut [u32; HASH_CACHE_SIZE],
    next_to_update: &mut usize,
    hash_salt_entropy: &mut u32,
    off_base: &mut u32,
) -> usize {
    const {
        assert!(MIN_MATCH >= 4 && MIN_MATCH <= 6);
        assert!(ROW_LOG >= 4 && ROW_LOG <= 6);
    }
    let row_entries = 1usize << ROW_LOG;
    let row_mask = row_entries - 1;
    let row_hash_log = hash_log - ROW_LOG;
    let mut attempts = 1usize << search_log.min(ROW_LOG);

    let hash = if lazy_skipping {
        *next_to_update = ip;
        hash_ptr_salted::<MIN_MATCH>(src, ip, row_hash_log + TAG_BITS, hash_salt)
    } else {
        update_rows_for_search::<MIN_MATCH, ROW_LOG>(
            src,
            ip,
            block_end,
            row_hash_log,
            hash_salt,
            hash_table,
            tag_table,
            hash_cache,
            next_to_update,
        );
        next_cached_hash::<MIN_MATCH, ROW_LOG>(
            src,
            ip,
            row_hash_log,
            hash_salt,
            hash_table,
            tag_table,
            hash_cache,
        )
    };
    *hash_salt_entropy = hash_salt_entropy.wrapping_add(hash);

    let row_start = ((hash >> TAG_BITS) as usize) << ROW_LOG;
    let tag = (hash & TAG_MASK) as u8;
    let head = usize::from(tag_at(tag_table, row_start) & row_mask as u8);
    let mut matches = row_match_mask::<ROW_LOG>(tag_table, row_start, tag, head);
    // C leaves this bounded scratch uninitialized and only reads the prefix
    // written by the tag scan. `MaybeUninit` preserves that exact contract
    // without paying to clear 256 stack bytes for every match search.
    let mut match_buffer = [MaybeUninit::<u32>::uninit(); MAX_ROW_ENTRIES];
    let mut num_matches = 0usize;

    // C first materializes and prefetches every viable tagged candidate, then
    // inserts the current position before following the candidate pointers.
    while matches != 0 && attempts != 0 {
        let step = matches.trailing_zeros() as usize;
        matches &= matches - 1;
        let position = (head + step) & row_mask;
        if position == 0 {
            continue;
        }

        let match_index = entry(hash_table, row_start + position);
        if match_index < low_limit as u32 {
            break;
        }
        debug_assert!((match_index as usize) < ip);
        prefetch_read(src.as_ptr().wrapping_add(match_index as usize));
        // SAFETY: `num_matches < row_entries <= MAX_ROW_ENTRIES`, and this
        // slot is read only after `num_matches` is incremented.
        unsafe {
            match_buffer
                .get_unchecked_mut(num_matches)
                .write(match_index)
        };
        num_matches += 1;
        attempts -= 1;
    }

    let insert_position = next_index(tag_table, row_start, row_mask);
    insert(
        hash_table,
        tag_table,
        row_start + insert_position,
        tag,
        *next_to_update as u32,
    );
    *next_to_update += 1;

    let mut best_len = 3usize;
    for current_match in 0..num_matches {
        // SAFETY: the collection loop initialized exactly `0..num_matches`.
        let match_index =
            unsafe { match_buffer.get_unchecked(current_match).assume_init() } as usize;
        let probe = best_len - 3;
        let current_len = if read32(src, match_index + probe) == read32(src, ip + probe) {
            count_match(src, ip, match_index, block_end)
        } else {
            0
        };

        if current_len > best_len {
            best_len = current_len;
            *off_base = (ip - match_index) as u32 + C_REPCODE_COUNT;
            if ip + current_len == block_end {
                break;
            }
        }
    }

    best_len
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn update_rows_for_search<const MIN_MATCH: u32, const ROW_LOG: u32>(
    src: &[u8],
    target: usize,
    block_end: usize,
    row_hash_log: u32,
    hash_salt: u64,
    hash_table: &mut [u32],
    tag_table: &mut [u8],
    hash_cache: &mut [u32; HASH_CACHE_SIZE],
    next_to_update: &mut usize,
) {
    let mut index = *next_to_update;
    if target.saturating_sub(index) > SKIP_THRESHOLD {
        let start_bound = index + MAX_MATCH_START_POSITIONS_TO_UPDATE;
        update_rows_range::<MIN_MATCH, ROW_LOG>(
            src,
            index,
            start_bound,
            row_hash_log,
            hash_salt,
            hash_table,
            tag_table,
            hash_cache,
        );
        index = target - MAX_MATCH_END_POSITIONS_TO_UPDATE;
        fill_hash_cache::<MIN_MATCH, ROW_LOG>(
            src,
            index,
            target.min(block_end),
            row_hash_log,
            hash_salt,
            hash_table,
            tag_table,
            hash_cache,
        );
    }

    update_rows_range::<MIN_MATCH, ROW_LOG>(
        src,
        index,
        target,
        row_hash_log,
        hash_salt,
        hash_table,
        tag_table,
        hash_cache,
    );
    *next_to_update = target;
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn update_rows_range<const MIN_MATCH: u32, const ROW_LOG: u32>(
    src: &[u8],
    mut index: usize,
    target: usize,
    row_hash_log: u32,
    hash_salt: u64,
    hash_table: &mut [u32],
    tag_table: &mut [u8],
    hash_cache: &mut [u32; HASH_CACHE_SIZE],
) {
    let row_mask = (1usize << ROW_LOG) - 1;
    while index < target {
        let hash = next_cached_hash::<MIN_MATCH, ROW_LOG>(
            src,
            index,
            row_hash_log,
            hash_salt,
            hash_table,
            tag_table,
            hash_cache,
        );
        let row_start = ((hash >> TAG_BITS) as usize) << ROW_LOG;
        let position = next_index(tag_table, row_start, row_mask);
        insert(
            hash_table,
            tag_table,
            row_start + position,
            (hash & TAG_MASK) as u8,
            index as u32,
        );
        index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn fill_hash_cache<const MIN_MATCH: u32, const ROW_LOG: u32>(
    src: &[u8],
    mut index: usize,
    limit: usize,
    row_hash_log: u32,
    hash_salt: u64,
    hash_table: &[u32],
    tag_table: &[u8],
    hash_cache: &mut [u32; HASH_CACHE_SIZE],
) {
    let end = index
        .saturating_add(HASH_CACHE_SIZE)
        .min(limit.saturating_add(1));
    while index < end {
        let hash = hash_ptr_salted::<MIN_MATCH>(src, index, row_hash_log + TAG_BITS, hash_salt);
        let row_start = ((hash >> TAG_BITS) as usize) << ROW_LOG;
        prefetch_row::<ROW_LOG>(hash_table, tag_table, row_start);
        hash_cache[index & HASH_CACHE_MASK] = hash;
        index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn next_cached_hash<const MIN_MATCH: u32, const ROW_LOG: u32>(
    src: &[u8],
    index: usize,
    row_hash_log: u32,
    hash_salt: u64,
    hash_table: &[u32],
    tag_table: &[u8],
    hash_cache: &mut [u32; HASH_CACHE_SIZE],
) -> u32 {
    let new_hash = hash_ptr_salted::<MIN_MATCH>(
        src,
        index + HASH_CACHE_SIZE,
        row_hash_log + TAG_BITS,
        hash_salt,
    );
    let row_start = ((new_hash >> TAG_BITS) as usize) << ROW_LOG;
    prefetch_row::<ROW_LOG>(hash_table, tag_table, row_start);
    let slot = index & HASH_CACHE_MASK;
    let cached = hash_cache[slot];
    hash_cache[slot] = new_hash;
    cached
}

#[inline(always)]
fn hash_ptr_salted<const MIN_MATCH: u32>(
    src: &[u8],
    position: usize,
    hash_bits: u32,
    salt: u64,
) -> u32 {
    match MIN_MATCH {
        5 => {
            const PRIME: u64 = 889_523_592_379;
            (((read64(src, position) << 24).wrapping_mul(PRIME) ^ salt) >> (64 - hash_bits)) as u32
        }
        6 => {
            const PRIME: u64 = 227_718_039_650_203;
            (((read64(src, position) << 16).wrapping_mul(PRIME) ^ salt) >> (64 - hash_bits)) as u32
        }
        _ => {
            const PRIME: u32 = 2_654_435_761;
            (read32(src, position).wrapping_mul(PRIME) ^ salt as u32) >> (32 - hash_bits)
        }
    }
}

#[inline(always)]
fn count_match(
    src: &[u8],
    mut position: usize,
    mut match_position: usize,
    match_limit: usize,
) -> usize {
    let start = position;
    let loop_limit = match_limit.saturating_sub(7);

    if position < loop_limit {
        let diff = read64(src, position) ^ read64(src, match_position);
        if diff != 0 {
            return common_prefix_bytes(diff);
        }
        position += 8;
        match_position += 8;

        while position < loop_limit {
            let diff = read64(src, position) ^ read64(src, match_position);
            if diff != 0 {
                return position - start + common_prefix_bytes(diff);
            }
            position += 8;
            match_position += 8;
        }
    }

    if position + 4 <= match_limit && read32(src, position) == read32(src, match_position) {
        position += 4;
        match_position += 4;
    }
    if position + 2 <= match_limit && read16(src, position) == read16(src, match_position) {
        position += 2;
        match_position += 2;
    }
    if position < match_limit && byte(src, position) == byte(src, match_position) {
        position += 1;
    }
    position - start
}

#[inline(always)]
fn common_prefix_bytes(diff: u64) -> usize {
    (diff.trailing_zeros() >> 3) as usize
}

#[inline(always)]
fn next_index(tag_table: &mut [u8], row_start: usize, row_mask: usize) -> usize {
    debug_assert!(row_start + row_mask < tag_table.len());
    // SAFETY: the row hash and mask identify a complete table row.
    let head = unsafe { tag_table.get_unchecked_mut(row_start) };
    let mut next = usize::from(head.wrapping_sub(1)) & row_mask;
    if next == 0 {
        next = row_mask;
    }
    *head = next as u8;
    next
}

#[inline(always)]
fn insert(hash_table: &mut [u32], tag_table: &mut [u8], slot: usize, tag: u8, index: u32) {
    debug_assert!(slot < hash_table.len());
    debug_assert!(slot < tag_table.len());
    // SAFETY: callers combine an in-table row start and masked nonzero slot.
    unsafe {
        *tag_table.get_unchecked_mut(slot) = tag;
        *hash_table.get_unchecked_mut(slot) = index;
    }
}

#[inline(always)]
fn entry(hash_table: &[u32], slot: usize) -> u32 {
    debug_assert!(slot < hash_table.len());
    // SAFETY: callers combine an in-table row start and masked slot.
    unsafe { *hash_table.get_unchecked(slot) }
}

#[inline(always)]
fn tag_at(tag_table: &[u8], slot: usize) -> u8 {
    debug_assert!(slot < tag_table.len());
    // SAFETY: row starts are derived from the configured hash width.
    unsafe { *tag_table.get_unchecked(slot) }
}

#[inline(always)]
fn byte(src: &[u8], position: usize) -> u8 {
    debug_assert!(position < src.len());
    // SAFETY: callers prove the byte is before the match limit or behind the
    // current source position.
    unsafe { *src.get_unchecked(position) }
}

#[inline(always)]
fn read16(src: &[u8], position: usize) -> u16 {
    debug_assert!(position + 2 <= src.len());
    // SAFETY: every read is bounded by the source or match limit.
    unsafe { u16::from_le(core::ptr::read_unaligned(src.as_ptr().add(position).cast())) }
}

#[inline(always)]
fn read32(src: &[u8], position: usize) -> u32 {
    debug_assert!(position + 4 <= src.len());
    // SAFETY: every read is bounded by the source or match limit.
    unsafe { u32::from_le(core::ptr::read_unaligned(src.as_ptr().add(position).cast())) }
}

#[inline(always)]
fn read64(src: &[u8], position: usize) -> u64 {
    debug_assert!(position + 8 <= src.len());
    // SAFETY: every read is bounded by the source or match limit.
    unsafe { u64::from_le(core::ptr::read_unaligned(src.as_ptr().add(position).cast())) }
}

#[inline(always)]
fn row_match_mask<const ROW_LOG: u32>(
    tag_table: &[u8],
    row_start: usize,
    tag: u8,
    head: usize,
) -> u64 {
    let row_entries = 1usize << ROW_LOG;
    debug_assert!(row_start + row_entries <= tag_table.len());
    debug_assert!(head < row_entries);

    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    let matches = row_match_mask_sse2::<ROW_LOG>(tag_table, row_start, tag);
    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
    let matches = row_match_mask_scalar::<ROW_LOG>(tag_table, row_start, tag);

    rotate_right_within(matches, head, row_entries)
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[inline(always)]
fn row_match_mask_sse2<const ROW_LOG: u32>(tag_table: &[u8], row_start: usize, tag: u8) -> u64 {
    use core::arch::x86_64::{
        __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
    };

    let comparison = unsafe { _mm_set1_epi8(tag as i8) };
    let load = |offset: usize| -> u64 {
        debug_assert!(row_start + offset + 16 <= tag_table.len());
        // SAFETY: each load is a complete 16-byte chunk of the selected row.
        unsafe {
            let chunk =
                _mm_loadu_si128(tag_table.as_ptr().add(row_start + offset).cast::<__m128i>());
            _mm_movemask_epi8(_mm_cmpeq_epi8(chunk, comparison)) as u64
        }
    };

    match ROW_LOG {
        4 => load(0),
        5 => load(0) | load(16) << 16,
        6 => load(0) | load(16) << 16 | load(32) << 32 | load(48) << 48,
        _ => unreachable!(),
    }
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
#[inline(always)]
fn row_match_mask_scalar<const ROW_LOG: u32>(tag_table: &[u8], row_start: usize, tag: u8) -> u64 {
    let row_entries = 1usize << ROW_LOG;
    let splat = u64::from_le_bytes([tag; 8]);
    let mut matches = 0u64;
    let mut offset = 0usize;
    while offset < row_entries {
        let chunk = read64(tag_table, row_start + offset) ^ splat;
        let zero_bytes = chunk.wrapping_sub(0x0101_0101_0101_0101) & !chunk & 0x8080_8080_8080_8080;
        matches |= byte_high_bits_to_mask(zero_bytes) << offset;
        offset += 8;
    }
    matches
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
#[inline(always)]
fn byte_high_bits_to_mask(high_bits: u64) -> u64 {
    ((high_bits >> 7).wrapping_mul(0x0102_0408_1020_4080) >> 56) & 0xff
}

#[inline(always)]
fn rotate_right_within(value: u64, shift: usize, width: usize) -> u64 {
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

#[inline(always)]
fn prefetch_row<const ROW_LOG: u32>(hash_table: &[u32], tag_table: &[u8], row_start: usize) {
    prefetch_read(hash_table.as_ptr().wrapping_add(row_start));
    if ROW_LOG >= 5 {
        prefetch_read(hash_table.as_ptr().wrapping_add(row_start + 16));
    }
    prefetch_read(tag_table.as_ptr().wrapping_add(row_start));
    if ROW_LOG == 6 {
        prefetch_read(tag_table.as_ptr().wrapping_add(row_start + 32));
    }
}

#[inline(always)]
fn prefetch_read<T>(pointer: *const T) {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: `_mm_prefetch` is a non-faulting cache hint, matching C.
    unsafe {
        core::arch::x86_64::_mm_prefetch(pointer.cast::<i8>(), core::arch::x86_64::_MM_HINT_T0);
    }
    #[cfg(not(target_arch = "x86_64"))]
    let _ = pointer;
}

#[cfg(test)]
mod tests {
    use super::find_best_match_no_dict;

    #[test]
    fn finds_previous_row_match_and_updates_state() {
        let data = b"abcdefghabcdefgh-tail-padding";
        let mut hashes = [0u32; 1 << 10];
        let mut tags = [0u8; 1 << 10];
        let mut cache = [0u32; 8];
        let mut next = 0usize;
        let mut entropy = 0u32;
        let mut off_base = 0u32;

        // The production caller primes the same eight-entry hash cache.
        for (index, slot) in cache.iter_mut().enumerate() {
            *slot = super::hash_ptr_salted::<4>(data, index, 14, 1);
        }
        // SAFETY: this fixture primes exact row tables/cache for valid bounds.
        let length = unsafe {
            find_best_match_no_dict(
                data,
                8,
                data.len(),
                0,
                10,
                4,
                4,
                1,
                false,
                &mut hashes,
                &mut tags,
                &mut cache,
                &mut next,
                &mut entropy,
                &mut off_base,
            )
        };

        assert!(length >= 8);
        assert_eq!(off_base, 11);
        assert_eq!(next, 9);
        assert_ne!(entropy, 0);
    }
}
