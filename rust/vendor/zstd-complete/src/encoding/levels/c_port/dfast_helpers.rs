//! Low-level primitives for the C double-fast block compressor.

#![forbid(unsafe_code)]

use super::sequence_store::{OffBase, StoredSequence};
pub(super) use super::unaligned::{read32, read64};
pub(super) const HASH_READ_SIZE: usize = 8;

pub(super) fn lowest_prefix_index_with_loaded_dict(
    end_index: usize,
    window_log: u32,
    loaded_dict_end: usize,
) -> usize {
    let window_size = 1_usize << window_log;
    if loaded_dict_end != 0 && end_index <= loaded_dict_end.saturating_add(window_size) {
        0
    } else {
        end_index.saturating_sub(window_size)
    }
}

pub(super) fn hash8_ptr(src: &[u8], pos: usize, h_bits: u32) -> usize {
    debug_assert!(h_bits <= 32);
    debug_assert!(pos + HASH_READ_SIZE <= src.len());
    hash8(read64(src, pos), h_bits)
}

pub(super) fn hash_small_ptr<const MIN_MATCH: u32>(src: &[u8], pos: usize, h_bits: u32) -> usize {
    debug_assert!(h_bits <= 32);
    debug_assert!(pos + HASH_READ_SIZE <= src.len());

    match MIN_MATCH {
        5 => hash5(read64(src, pos), h_bits),
        6 => hash6(read64(src, pos), h_bits),
        7 => hash7(read64(src, pos), h_bits),
        8 => hash8(read64(src, pos), h_bits),
        _ => hash4(read32(src, pos), h_bits),
    }
}

pub(super) fn hash_ptr(src: &[u8], pos: usize, h_bits: u32, min_match: u32) -> usize {
    debug_assert!(h_bits <= 32);
    debug_assert!(pos + HASH_READ_SIZE <= src.len());

    match min_match {
        5 => hash5(read64(src, pos), h_bits),
        6 => hash6(read64(src, pos), h_bits),
        7 => hash7(read64(src, pos), h_bits),
        8 => hash8(read64(src, pos), h_bits),
        _ => hash4(read32(src, pos), h_bits),
    }
}

pub(super) use super::match_count::count_match_behind as count_match;

#[inline(always)]
pub(super) fn store_match(
    sequences: &mut crate::workspace::ReusableVec<StoredSequence>,
    anchor: &mut usize,
    ip: &mut usize,
    off_base: OffBase,
    match_length: usize,
) {
    sequences.push(StoredSequence::new(
        (*ip - *anchor) as u32,
        off_base,
        match_length as u32,
    ));
    *ip += match_length;
    *anchor = *ip;
}

fn hash4(value: u32, h_bits: u32) -> usize {
    const PRIME_4_BYTES: u32 = 2_654_435_761;
    value.wrapping_mul(PRIME_4_BYTES).wrapping_shr(32 - h_bits) as usize
}

fn hash5(value: u64, h_bits: u32) -> usize {
    const PRIME_5_BYTES: u64 = 889_523_592_379;
    ((value << (64 - 40)).wrapping_mul(PRIME_5_BYTES) >> (64 - h_bits)) as usize
}

fn hash6(value: u64, h_bits: u32) -> usize {
    const PRIME_6_BYTES: u64 = 227_718_039_650_203;
    ((value << (64 - 48)).wrapping_mul(PRIME_6_BYTES) >> (64 - h_bits)) as usize
}

fn hash7(value: u64, h_bits: u32) -> usize {
    const PRIME_7_BYTES: u64 = 58_295_818_150_454_627;
    ((value << (64 - 56)).wrapping_mul(PRIME_7_BYTES) >> (64 - h_bits)) as usize
}

fn hash8(value: u64, h_bits: u32) -> usize {
    const PRIME_8_BYTES: u64 = 0xCF1B_BCDC_B7A5_6463;
    value.wrapping_mul(PRIME_8_BYTES).wrapping_shr(64 - h_bits) as usize
}
