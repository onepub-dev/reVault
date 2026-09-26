use core::mem::MaybeUninit;

const HASH_READ_SIZE: usize = 8;
const SEARCH_STRENGTH: usize = 8;
const REPCODE_COUNT: u32 = 3;
const INVALID_INDEX: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockResult {
    pub sequence_count: usize,
    pub last_literals: u32,
    pub repeat_offsets: [u32; REPCODE_COUNT as usize],
}

/// Generated matcher entry point. Callers must uphold the slice and table
/// invariants documented by [`select_block`].
pub type BlockFn = unsafe fn(
    src: &[u8],
    block_start: usize,
    block_end: usize,
    loaded_dict_end: usize,
    window_log: u32,
    hash_log: u32,
    target_length: u32,
    repeat_offsets: [u32; REPCODE_COUNT as usize],
    hash_table: &mut [u32],
    sequences: &mut [MaybeUninit<[u32; 3]>],
) -> BlockResult;

/// Selects a specialized matcher.
///
/// # Safety
///
/// The returned function requires valid source block bounds, a hash table of
/// exactly `1 << hash_log` entries, and enough sequence slots for every match
/// produced from the block. Repeat offsets and window parameters must describe
/// the same prefix represented by `src`.
pub fn select_block(min_match: u32) -> BlockFn {
    match min_match {
        4 => compress_block::<4>,
        5 => compress_block::<5>,
        6 => compress_block::<6>,
        7 => compress_block::<7>,
        _ => compress_block::<4>,
    }
}

/// Complete no-dictionary `ZSTD_compressBlock_fast_noDict_generic()`
/// transaction behind a primitive ABI.
#[allow(clippy::too_many_arguments)]
#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_fast")]
#[cfg_attr(target_family = "windows", link_section = ".text$010.rz.fast")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.010.ruzstd.fast.matcher"
)]
fn compress_block<const MIN_MATCH: u32>(
    src: &[u8],
    block_start: usize,
    block_end: usize,
    loaded_dict_end: usize,
    window_log: u32,
    hash_log: u32,
    target_length: u32,
    repeat_offsets: [u32; REPCODE_COUNT as usize],
    hash_table: &mut [u32],
    sequences: &mut [MaybeUninit<[u32; 3]>],
) -> BlockResult {
    const { assert!(MIN_MATCH >= 4 && MIN_MATCH <= 7) };
    debug_assert!(block_start <= block_end);
    debug_assert!(block_end <= src.len());
    debug_assert_eq!(hash_table.len(), 1usize << hash_log);

    let block_len = block_end - block_start;
    if block_len <= HASH_READ_SIZE {
        return BlockResult {
            sequence_count: 0,
            last_literals: block_len as u32,
            repeat_offsets,
        };
    }

    let mut rep = repeat_offsets;
    let step_size = target_length as usize + usize::from(target_length == 0) + 1;
    let prefix_start = lowest_prefix_index(block_end, window_log, loaded_dict_end);
    let ilimit = block_end - HASH_READ_SIZE;
    let mut anchor = block_start;
    let mut ip0 = block_start + usize::from(block_start == 0 && prefix_start == 0);
    let mut sequence_count = 0usize;

    let mut offset1 = rep[0] as usize;
    let mut offset2 = rep[1] as usize;
    let mut saved1 = 0usize;
    let mut saved2 = 0usize;
    let window_low = lowest_prefix_index(ip0, window_log, loaded_dict_end);
    let max_rep = ip0 - window_low;
    if offset2 > max_rep {
        saved2 = offset2;
        offset2 = 0;
    }
    if offset1 > max_rep {
        saved1 = offset1;
        offset1 = 0;
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

        let mut hash0 = hash_ptr::<MIN_MATCH>(src, ip0, hash_log);
        let mut hash1 = hash_ptr::<MIN_MATCH>(src, ip1, hash_log);
        let mut match_index = entry(hash_table, hash0) as usize;

        while ip3 < ilimit {
            let current0 = ip0;
            set_entry(hash_table, hash0, current0 as u32);

            if offset1 > 0 && read32(src, ip2) == read32(src, ip2 - offset1) {
                ip0 = ip2;
                let mut match0 = ip0 - offset1;
                let backward = usize::from(ip0 > 0 && byte(src, ip0 - 1) == byte(src, match0 - 1));
                ip0 -= backward;
                match0 -= backward;
                set_entry(hash_table, hash1, ip1 as u32);
                store_match(
                    src,
                    sequences,
                    &mut sequence_count,
                    &mut anchor,
                    &mut ip0,
                    match0,
                    1,
                    4 + backward,
                    block_end,
                );
                fill_after_match::<MIN_MATCH>(src, hash_table, hash_log, current0, ip0, ilimit);
                consume_immediate_repcodes::<MIN_MATCH>(
                    src,
                    hash_table,
                    hash_log,
                    sequences,
                    &mut sequence_count,
                    &mut anchor,
                    &mut ip0,
                    ilimit,
                    &mut offset1,
                    &mut offset2,
                    block_end,
                );
                continue 'restart;
            }

            if match4_found(src, ip0, match_index, prefix_start) {
                set_entry(hash_table, hash1, ip1 as u32);
                let mut match0 = match_index;
                offset2 = offset1;
                offset1 = ip0 - match0;
                let mut match_length = 4;
                while ip0 > anchor
                    && match0 > prefix_start
                    && byte(src, ip0 - 1) == byte(src, match0 - 1)
                {
                    ip0 -= 1;
                    match0 -= 1;
                    match_length += 1;
                }
                store_match(
                    src,
                    sequences,
                    &mut sequence_count,
                    &mut anchor,
                    &mut ip0,
                    match0,
                    offset1 as u32 + REPCODE_COUNT,
                    match_length,
                    block_end,
                );
                fill_after_match::<MIN_MATCH>(src, hash_table, hash_log, current0, ip0, ilimit);
                consume_immediate_repcodes::<MIN_MATCH>(
                    src,
                    hash_table,
                    hash_log,
                    sequences,
                    &mut sequence_count,
                    &mut anchor,
                    &mut ip0,
                    ilimit,
                    &mut offset1,
                    &mut offset2,
                    block_end,
                );
                continue 'restart;
            }

            match_index = entry(hash_table, hash1) as usize;
            hash0 = hash1;
            hash1 = hash_ptr::<MIN_MATCH>(src, ip2, hash_log);
            ip0 = ip1;
            ip1 = ip2;
            ip2 = ip3;
            let current0 = ip0;
            set_entry(hash_table, hash0, current0 as u32);

            if match4_found(src, ip0, match_index, prefix_start) {
                if step <= 4 {
                    set_entry(hash_table, hash1, ip1 as u32);
                }
                let mut match0 = match_index;
                offset2 = offset1;
                offset1 = ip0 - match0;
                let mut match_length = 4;
                while ip0 > anchor
                    && match0 > prefix_start
                    && byte(src, ip0 - 1) == byte(src, match0 - 1)
                {
                    ip0 -= 1;
                    match0 -= 1;
                    match_length += 1;
                }
                store_match(
                    src,
                    sequences,
                    &mut sequence_count,
                    &mut anchor,
                    &mut ip0,
                    match0,
                    offset1 as u32 + REPCODE_COUNT,
                    match_length,
                    block_end,
                );
                fill_after_match::<MIN_MATCH>(src, hash_table, hash_log, current0, ip0, ilimit);
                consume_immediate_repcodes::<MIN_MATCH>(
                    src,
                    hash_table,
                    hash_log,
                    sequences,
                    &mut sequence_count,
                    &mut anchor,
                    &mut ip0,
                    ilimit,
                    &mut offset1,
                    &mut offset2,
                    block_end,
                );
                continue 'restart;
            }

            match_index = entry(hash_table, hash1) as usize;
            hash0 = hash1;
            hash1 = hash_ptr::<MIN_MATCH>(src, ip2, hash_log);
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

    if saved1 != 0 && offset1 != 0 {
        saved2 = saved1;
    }
    rep[0] = if offset1 != 0 { offset1 } else { saved1 } as u32;
    rep[1] = if offset2 != 0 { offset2 } else { saved2 } as u32;
    BlockResult {
        sequence_count,
        last_literals: (block_end - anchor) as u32,
        repeat_offsets: rep,
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn store_match(
    src: &[u8],
    sequences: &mut [MaybeUninit<[u32; 3]>],
    sequence_count: &mut usize,
    anchor: &mut usize,
    ip: &mut usize,
    match_position: usize,
    off_base: u32,
    initial_length: usize,
    match_limit: usize,
) {
    let match_length = initial_length
        + count_match(
            src,
            *ip + initial_length,
            match_position + initial_length,
            match_limit,
        );
    debug_assert!(*sequence_count < sequences.len());
    // SAFETY: each record consumes a disjoint match of at least four bytes;
    // the caller provides a block-length-derived spare prefix.
    unsafe {
        sequences.get_unchecked_mut(*sequence_count).write([
            (*ip - *anchor) as u32,
            match_length as u32,
            off_base,
        ]);
    }
    *sequence_count += 1;
    *ip += match_length;
    *anchor = *ip;
}

#[inline(always)]
fn fill_after_match<const MIN_MATCH: u32>(
    src: &[u8],
    hash_table: &mut [u32],
    hash_log: u32,
    current0: usize,
    ip: usize,
    ilimit: usize,
) {
    if ip > ilimit {
        return;
    }
    if current0 + 2 <= ilimit {
        let position = current0 + 2;
        set_entry(
            hash_table,
            hash_ptr::<MIN_MATCH>(src, position, hash_log),
            position as u32,
        );
    }
    if let Some(position) = ip.checked_sub(2).filter(|position| *position <= ilimit) {
        set_entry(
            hash_table,
            hash_ptr::<MIN_MATCH>(src, position, hash_log),
            position as u32,
        );
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn consume_immediate_repcodes<const MIN_MATCH: u32>(
    src: &[u8],
    hash_table: &mut [u32],
    hash_log: u32,
    sequences: &mut [MaybeUninit<[u32; 3]>],
    sequence_count: &mut usize,
    anchor: &mut usize,
    ip: &mut usize,
    ilimit: usize,
    offset1: &mut usize,
    offset2: &mut usize,
    match_limit: usize,
) {
    if *offset2 == 0 {
        return;
    }
    while *ip <= ilimit && read32(src, *ip) == read32(src, *ip - *offset2) {
        let repeat_length = count_match(src, *ip + 4, *ip + 4 - *offset2, match_limit) + 4;
        core::mem::swap(offset1, offset2);
        set_entry(
            hash_table,
            hash_ptr::<MIN_MATCH>(src, *ip, hash_log),
            *ip as u32,
        );
        debug_assert!(*sequence_count < sequences.len());
        // SAFETY: same bounded sequence-prefix invariant as `store_match()`.
        unsafe {
            sequences
                .get_unchecked_mut(*sequence_count)
                .write([0, repeat_length as u32, 1]);
        }
        *sequence_count += 1;
        *ip += repeat_length;
        *anchor = *ip;
    }
}

#[inline(always)]
fn match4_found(src: &[u8], current: usize, candidate: usize, prefix_start: usize) -> bool {
    candidate != INVALID_INDEX as usize
        && candidate >= prefix_start
        && read32(src, current) == read32(src, candidate)
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
fn hash_ptr<const MIN_MATCH: u32>(src: &[u8], position: usize, hash_bits: u32) -> usize {
    match MIN_MATCH {
        5 => hash5(read64(src, position), hash_bits),
        6 => hash6(read64(src, position), hash_bits),
        7 => hash7(read64(src, position), hash_bits),
        _ => hash4(read32(src, position), hash_bits),
    }
}

#[inline(always)]
fn hash4(value: u32, hash_bits: u32) -> usize {
    const PRIME: u32 = 2_654_435_761;
    value.wrapping_mul(PRIME).wrapping_shr(32 - hash_bits) as usize
}

#[inline(always)]
fn hash5(value: u64, hash_bits: u32) -> usize {
    const PRIME: u64 = 889_523_592_379;
    ((value << 24).wrapping_mul(PRIME) >> (64 - hash_bits)) as usize
}

#[inline(always)]
fn hash6(value: u64, hash_bits: u32) -> usize {
    const PRIME: u64 = 227_718_039_650_203;
    ((value << 16).wrapping_mul(PRIME) >> (64 - hash_bits)) as usize
}

#[inline(always)]
fn hash7(value: u64, hash_bits: u32) -> usize {
    const PRIME: u64 = 58_295_818_150_454_627;
    ((value << 8).wrapping_mul(PRIME) >> (64 - hash_bits)) as usize
}

#[inline(always)]
fn count_match(
    src: &[u8],
    mut position: usize,
    mut match_position: usize,
    match_limit: usize,
) -> usize {
    let start = position;
    while position + 8 <= match_limit {
        let diff = read64(src, position) ^ read64(src, match_position);
        if diff != 0 {
            return position - start + (diff.trailing_zeros() >> 3) as usize;
        }
        position += 8;
        match_position += 8;
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
fn entry(table: &[u32], slot: usize) -> u32 {
    debug_assert!(slot < table.len());
    // SAFETY: table length and hash log are validated at entry.
    unsafe { *table.get_unchecked(slot) }
}

#[inline(always)]
fn set_entry(table: &mut [u32], slot: usize, value: u32) {
    debug_assert!(slot < table.len());
    // SAFETY: table length and hash log are validated at entry.
    unsafe { *table.get_unchecked_mut(slot) = value }
}

#[inline(always)]
fn byte(src: &[u8], position: usize) -> u8 {
    debug_assert!(position < src.len());
    // SAFETY: every byte probe is inside the active source window.
    unsafe { *src.get_unchecked(position) }
}

#[inline(always)]
fn read16(src: &[u8], position: usize) -> u16 {
    debug_assert!(position + 2 <= src.len());
    // SAFETY: parser/match limits bound each unaligned read.
    unsafe { u16::from_le(core::ptr::read_unaligned(src.as_ptr().add(position).cast())) }
}

#[inline(always)]
fn read32(src: &[u8], position: usize) -> u32 {
    debug_assert!(position + 4 <= src.len());
    // SAFETY: parser/match limits bound each unaligned read.
    unsafe { u32::from_le(core::ptr::read_unaligned(src.as_ptr().add(position).cast())) }
}

#[inline(always)]
fn read64(src: &[u8], position: usize) -> u64 {
    debug_assert!(position + 8 <= src.len());
    // SAFETY: parser/match limits bound each unaligned read.
    unsafe { u64::from_le(core::ptr::read_unaligned(src.as_ptr().add(position).cast())) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_match_and_updates_table() {
        let source = b"abcdefghabcdefghabcdefgh-tail";
        let mut table = [INVALID_INDEX; 1 << 10];
        let mut sequences = [MaybeUninit::uninit(); 16];
        // SAFETY: this fixture supplies exact tables, valid bounds, and ample
        // sequence capacity.
        let result = unsafe {
            select_block(4)(
                source,
                0,
                source.len(),
                0,
                10,
                10,
                0,
                [1, 4, 8],
                &mut table,
                &mut sequences,
            )
        };
        assert!(result.sequence_count > 0);
        assert!(table.iter().any(|entry| *entry != INVALID_INDEX));
    }
}
