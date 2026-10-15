use core::mem::MaybeUninit;

const HASH_READ_SIZE: usize = 8;
const SEARCH_STRENGTH: usize = 8;
const REPCODE_COUNT: u32 = 3;

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
    chain_log: u32,
    repeat_offsets: [u32; REPCODE_COUNT as usize],
    hash_long: &mut [u32],
    hash_small: &mut [u32],
    sequences: &mut [MaybeUninit<[u32; 3]>],
) -> BlockResult;

/// Selects a specialized matcher.
///
/// # Safety
///
/// The returned function requires valid source block bounds, hash tables of
/// exactly `1 << hash_log` and `1 << chain_log` entries, and enough sequence
/// slots for every match produced from the block. Repeat offsets and window
/// parameters must describe the same prefix represented by `src`.
pub fn select_block(min_match: u32) -> BlockFn {
    match min_match {
        4 => compress_block::<4>,
        5 => compress_block::<5>,
        6 => compress_block::<6>,
        7 => compress_block::<7>,
        _ => compress_block::<4>,
    }
}

/// Complete no-dictionary `ZSTD_compressBlock_doubleFast_noDict_generic()`
/// transaction behind a primitive ABI.
#[allow(clippy::too_many_arguments)]
#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_dfast")]
#[cfg_attr(target_family = "windows", link_section = ".text$011.rz.dfast")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.011.ruzstd.dfast.matcher"
)]
fn compress_block<const MIN_MATCH: u32>(
    src: &[u8],
    block_start: usize,
    block_end: usize,
    loaded_dict_end: usize,
    window_log: u32,
    hash_log: u32,
    chain_log: u32,
    repeat_offsets: [u32; REPCODE_COUNT as usize],
    hash_long: &mut [u32],
    hash_small: &mut [u32],
    sequences: &mut [MaybeUninit<[u32; 3]>],
) -> BlockResult {
    const { assert!(MIN_MATCH >= 4 && MIN_MATCH <= 7) };
    debug_assert!(block_start <= block_end);
    debug_assert!(block_end <= src.len());
    debug_assert_eq!(hash_long.len(), 1usize << hash_log);
    debug_assert_eq!(hash_small.len(), 1usize << chain_log);

    let block_len = block_end - block_start;
    if block_len <= HASH_READ_SIZE {
        return BlockResult {
            sequence_count: 0,
            last_literals: block_len as u32,
            repeat_offsets,
        };
    }

    let mut rep = repeat_offsets;
    let prefix_lowest_index = lowest_prefix_index(block_end, window_log, loaded_dict_end);
    let ilimit = block_end - HASH_READ_SIZE;
    let mut anchor = block_start;
    let mut ip = block_start + usize::from(block_start == prefix_lowest_index);
    let mut sequence_count = 0usize;

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

    'outer: loop {
        let mut step = 1usize;
        let mut next_step = ip + (1 << SEARCH_STRENGTH);
        let mut ip1 = ip + step;
        if ip1 > ilimit {
            break;
        }

        let mut long_hash0 = hash8_ptr(src, ip, hash_log);
        let mut long_index0 = entry(hash_long, long_hash0) as usize;
        let mut long_match0 = long_index0;

        loop {
            let short_hash0 = hash_small_ptr::<MIN_MATCH>(src, ip, chain_log);
            let short_index0 = entry(hash_small, short_hash0) as usize;
            let current = ip;
            let mut short_match0 = short_index0;

            set_entry(hash_long, long_hash0, current as u32);
            set_entry(hash_small, short_hash0, current as u32);

            if offset_1 > 0 && read32(src, ip + 1 - offset_1) == read32(src, ip + 1) {
                let match_length = count_match(src, ip + 5, ip + 5 - offset_1, block_end) + 4;
                ip += 1;
                store_match(
                    sequences,
                    &mut sequence_count,
                    &mut anchor,
                    &mut ip,
                    1,
                    match_length,
                );
                complementary_insert::<MIN_MATCH>(
                    src, hash_long, hash_small, hash_log, chain_log, current, ip, ilimit,
                );
                consume_immediate_repcodes::<MIN_MATCH>(
                    src,
                    hash_long,
                    hash_small,
                    hash_log,
                    chain_log,
                    sequences,
                    &mut sequence_count,
                    &mut anchor,
                    &mut ip,
                    ilimit,
                    &mut offset_1,
                    &mut offset_2,
                    block_end,
                );
                continue 'outer;
            }

            let long_hash1 = hash8_ptr(src, ip1, hash_log);
            if long_index0 > prefix_lowest_index && read64(src, long_match0) == read64(src, ip) {
                let mut match_length = count_match(src, ip + 8, long_match0 + 8, block_end) + 8;
                let mut offset = ip - long_match0;
                while ip > anchor
                    && long_match0 > prefix_lowest_index
                    && byte(src, ip - 1) == byte(src, long_match0 - 1)
                {
                    ip -= 1;
                    long_match0 -= 1;
                    offset = ip - long_match0;
                    match_length += 1;
                }
                if step < 4 {
                    set_entry(hash_long, long_hash1, ip1 as u32);
                }
                store_offset_match(
                    sequences,
                    &mut sequence_count,
                    &mut anchor,
                    &mut ip,
                    &mut offset_1,
                    &mut offset_2,
                    offset,
                    match_length,
                );
                complementary_insert::<MIN_MATCH>(
                    src, hash_long, hash_small, hash_log, chain_log, current, ip, ilimit,
                );
                consume_immediate_repcodes::<MIN_MATCH>(
                    src,
                    hash_long,
                    hash_small,
                    hash_log,
                    chain_log,
                    sequences,
                    &mut sequence_count,
                    &mut anchor,
                    &mut ip,
                    ilimit,
                    &mut offset_1,
                    &mut offset_2,
                    block_end,
                );
                continue 'outer;
            }

            let long_index1 = entry(hash_long, long_hash1) as usize;
            let long_match1 = long_index1;
            if short_index0 > prefix_lowest_index && read32(src, short_match0) == read32(src, ip) {
                let mut match_length = count_match(src, ip + 4, short_match0 + 4, block_end) + 4;
                let mut offset = ip - short_match0;

                if long_index1 > prefix_lowest_index && read64(src, long_match1) == read64(src, ip1)
                {
                    let next_length = count_match(src, ip1 + 8, long_match1 + 8, block_end) + 8;
                    if next_length > match_length {
                        ip = ip1;
                        match_length = next_length;
                        offset = ip - long_match1;
                        short_match0 = long_match1;
                    }
                }

                while ip > anchor
                    && short_match0 > prefix_lowest_index
                    && byte(src, ip - 1) == byte(src, short_match0 - 1)
                {
                    ip -= 1;
                    short_match0 -= 1;
                    offset = ip - short_match0;
                    match_length += 1;
                }
                if step < 4 {
                    set_entry(hash_long, long_hash1, ip1 as u32);
                }
                store_offset_match(
                    sequences,
                    &mut sequence_count,
                    &mut anchor,
                    &mut ip,
                    &mut offset_1,
                    &mut offset_2,
                    offset,
                    match_length,
                );
                complementary_insert::<MIN_MATCH>(
                    src, hash_long, hash_small, hash_log, chain_log, current, ip, ilimit,
                );
                consume_immediate_repcodes::<MIN_MATCH>(
                    src,
                    hash_long,
                    hash_small,
                    hash_log,
                    chain_log,
                    sequences,
                    &mut sequence_count,
                    &mut anchor,
                    &mut ip,
                    ilimit,
                    &mut offset_1,
                    &mut offset_2,
                    block_end,
                );
                continue 'outer;
            }

            if ip1 >= next_step {
                step += 1;
                next_step += 1 << SEARCH_STRENGTH;
            }
            ip = ip1;
            ip1 += step;
            long_hash0 = long_hash1;
            long_index0 = long_index1;
            long_match0 = long_match1;
            if ip1 > ilimit {
                break 'outer;
            }
        }
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

    let last_literals = block_end - anchor;
    BlockResult {
        sequence_count,
        last_literals: last_literals as u32,
        repeat_offsets: rep,
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn complementary_insert<const MIN_MATCH: u32>(
    src: &[u8],
    hash_long: &mut [u32],
    hash_small: &mut [u32],
    hash_log: u32,
    chain_log: u32,
    current: usize,
    ip: usize,
    ilimit: usize,
) {
    if ip > ilimit {
        return;
    }
    let index_to_insert = current + 2;
    if index_to_insert <= ilimit {
        set_entry(
            hash_long,
            hash8_ptr(src, index_to_insert, hash_log),
            index_to_insert as u32,
        );
        set_entry(
            hash_small,
            hash_small_ptr::<MIN_MATCH>(src, index_to_insert, chain_log),
            index_to_insert as u32,
        );
    }
    if let Some(index) = ip.checked_sub(2).filter(|index| *index <= ilimit) {
        set_entry(hash_long, hash8_ptr(src, index, hash_log), index as u32);
    }
    if let Some(index) = ip.checked_sub(1).filter(|index| *index <= ilimit) {
        set_entry(
            hash_small,
            hash_small_ptr::<MIN_MATCH>(src, index, chain_log),
            index as u32,
        );
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn consume_immediate_repcodes<const MIN_MATCH: u32>(
    src: &[u8],
    hash_long: &mut [u32],
    hash_small: &mut [u32],
    hash_log: u32,
    chain_log: u32,
    sequences: &mut [MaybeUninit<[u32; 3]>],
    sequence_count: &mut usize,
    anchor: &mut usize,
    ip: &mut usize,
    ilimit: usize,
    offset_1: &mut usize,
    offset_2: &mut usize,
    block_end: usize,
) {
    while *ip <= ilimit && *offset_2 > 0 && read32(src, *ip) == read32(src, *ip - *offset_2) {
        let repeat_length = count_match(src, *ip + 4, *ip + 4 - *offset_2, block_end) + 4;
        core::mem::swap(offset_1, offset_2);
        set_entry(
            hash_small,
            hash_small_ptr::<MIN_MATCH>(src, *ip, chain_log),
            *ip as u32,
        );
        set_entry(hash_long, hash8_ptr(src, *ip, hash_log), *ip as u32);
        store_match(sequences, sequence_count, anchor, ip, 1, repeat_length);
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn store_offset_match(
    sequences: &mut [MaybeUninit<[u32; 3]>],
    sequence_count: &mut usize,
    anchor: &mut usize,
    ip: &mut usize,
    offset_1: &mut usize,
    offset_2: &mut usize,
    offset: usize,
    match_length: usize,
) {
    *offset_2 = *offset_1;
    *offset_1 = offset;
    store_match(
        sequences,
        sequence_count,
        anchor,
        ip,
        offset as u32 + REPCODE_COUNT,
        match_length,
    );
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn store_match(
    sequences: &mut [MaybeUninit<[u32; 3]>],
    sequence_count: &mut usize,
    anchor: &mut usize,
    ip: &mut usize,
    off_base: u32,
    match_length: usize,
) {
    debug_assert!(*sequence_count < sequences.len());
    // SAFETY: every record consumes a disjoint match of at least four bytes;
    // the caller reserves a block-length-derived prefix.
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
fn lowest_prefix_index(position: usize, window_log: u32, loaded_dict_end: usize) -> usize {
    let window_size = 1usize << window_log;
    if loaded_dict_end != 0 && position <= loaded_dict_end.saturating_add(window_size) {
        0
    } else {
        position.saturating_sub(window_size)
    }
}

#[inline(always)]
fn hash_small_ptr<const MIN_MATCH: u32>(src: &[u8], position: usize, hash_bits: u32) -> usize {
    match MIN_MATCH {
        5 => hash5(read64(src, position), hash_bits),
        6 => hash6(read64(src, position), hash_bits),
        7 => hash7(read64(src, position), hash_bits),
        _ => hash4(read32(src, position), hash_bits),
    }
}

#[inline(always)]
fn hash8_ptr(src: &[u8], position: usize, hash_bits: u32) -> usize {
    hash8(read64(src, position), hash_bits)
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
fn hash8(value: u64, hash_bits: u32) -> usize {
    const PRIME: u64 = 0xCF1B_BCDC_B7A5_6463;
    value.wrapping_mul(PRIME).wrapping_shr(64 - hash_bits) as usize
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
    // SAFETY: table lengths are exactly `1 << hash_log`, and hashes are
    // reduced to the corresponding log.
    unsafe { *table.get_unchecked(slot) }
}

#[inline(always)]
fn set_entry(table: &mut [u32], slot: usize, value: u32) {
    debug_assert!(slot < table.len());
    // SAFETY: the same table-size/hash-range invariant as `entry()`.
    unsafe { *table.get_unchecked_mut(slot) = value }
}

#[inline(always)]
fn byte(src: &[u8], position: usize) -> u8 {
    debug_assert!(position < src.len());
    // SAFETY: callers prove the byte is within the source window.
    unsafe { *src.get_unchecked(position) }
}

#[inline(always)]
fn read16(src: &[u8], position: usize) -> u16 {
    debug_assert!(position + 2 <= src.len());
    // SAFETY: parser and match-limit invariants bound every read.
    unsafe { u16::from_le(core::ptr::read_unaligned(src.as_ptr().add(position).cast())) }
}

#[inline(always)]
fn read32(src: &[u8], position: usize) -> u32 {
    debug_assert!(position + 4 <= src.len());
    // SAFETY: parser and match-limit invariants bound every read.
    unsafe { u32::from_le(core::ptr::read_unaligned(src.as_ptr().add(position).cast())) }
}

#[inline(always)]
fn read64(src: &[u8], position: usize) -> u64 {
    debug_assert!(position + 8 <= src.len());
    // SAFETY: parser and match-limit invariants bound every read.
    unsafe { u64::from_le(core::ptr::read_unaligned(src.as_ptr().add(position).cast())) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_match_and_updates_both_tables() {
        let source = b"abcdefghabcdefghabcdefgh-tail";
        let mut long = [0u32; 1 << 10];
        let mut short = [0u32; 1 << 10];
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
                10,
                [1, 4, 8],
                &mut long,
                &mut short,
                &mut sequences,
            )
        };
        assert!(result.sequence_count > 0);
        assert!(long.iter().any(|entry| *entry != 0));
        assert!(short.iter().any(|entry| *entry != 0));
    }
}
