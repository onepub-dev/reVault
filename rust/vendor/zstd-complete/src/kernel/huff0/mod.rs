use alloc::vec::Vec;
use core::convert::TryFrom;

mod table;

#[cfg(test)]
pub use table::build_huffman_table;
pub use table::{
    build_described_huffman_table, build_described_huffman_table_reusing,
    build_described_huffman_table_reusing_with_context, HuffmanBuildScratch,
};

pub type HuffmanCode = (u32, u8);

/// Appends C's four-stream Huffman representation for a complete byte table.
///
/// Keeping the table-log specializations in this small crate isolates their
/// generated code from the compressor strategies in `ruzstd`.
/// # Safety
///
/// If `bmi2` is true, the current CPU must support BMI2. `codes` must be a
/// valid canonical table for `max_num_bits`, and `data` must contain at least
/// four bytes.
pub unsafe fn encode_four_streams(
    output: &mut Vec<u8>,
    codes: &[HuffmanCode; 256],
    max_num_bits: u8,
    data: &[u8],
    bmi2: bool,
) {
    #[cfg(target_arch = "x86_64")]
    if bmi2 {
        // SAFETY: the caller obtains `bmi2` from a cached CPUID check.
        unsafe { encode_four_streams_bmi2(output, codes, max_num_bits, data) };
        return;
    }

    #[cfg(not(target_arch = "x86_64"))]
    let _ = bmi2;
    encode_four_streams_body(output, codes, max_num_bits, data);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "bmi2")]
#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_hue")]
#[cfg_attr(target_family = "windows", link_section = ".text$020.rz.hue")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.020.ruzstd.huffman.emit.bmi2"
)]
unsafe fn encode_four_streams_bmi2(
    output: &mut Vec<u8>,
    codes: &[HuffmanCode; 256],
    max_num_bits: u8,
    data: &[u8],
) {
    encode_four_streams_body(output, codes, max_num_bits, data);
}

#[inline(always)]
fn encode_four_streams_body(
    output: &mut Vec<u8>,
    codes: &[HuffmanCode; 256],
    max_num_bits: u8,
    data: &[u8],
) {
    assert!(data.len() >= 4);
    debug_assert!((1..=11).contains(&max_num_bits));

    let split_size = data.len().div_ceil(4);
    let streams = [
        &data[..split_size],
        &data[split_size..split_size * 2],
        &data[split_size * 2..split_size * 3],
        &data[split_size * 3..],
    ];

    let jump_table = output.len();
    let payload_bound = streams
        .iter()
        .map(|stream| tight_stream_bound(stream.len(), max_num_bits))
        .sum::<usize>();
    output.resize(jump_table + 6 + payload_bound + 7, 0);
    let mut sizes = [0usize; 3];
    let mut write_pos = jump_table + 6;

    for (index, stream) in streams.iter().copied().enumerate() {
        let start = write_pos;
        write_pos = match max_num_bits {
            11 => encode_stream::<5>(output.as_mut_slice(), write_pos, codes, stream),
            10 => encode_stream::<5>(output.as_mut_slice(), write_pos, codes, stream),
            9 => encode_stream::<6>(output.as_mut_slice(), write_pos, codes, stream),
            8 => encode_stream::<7>(output.as_mut_slice(), write_pos, codes, stream),
            7 => encode_stream::<8>(output.as_mut_slice(), write_pos, codes, stream),
            _ => encode_stream::<9>(output.as_mut_slice(), write_pos, codes, stream),
        };
        if index < sizes.len() {
            sizes[index] = write_pos - start;
        }
    }

    for (index, size) in sizes.iter().copied().enumerate() {
        let size = u16::try_from(size).expect("Huffman stream exceeds jump-table range");
        let offset = jump_table + index * 2;
        output[offset..offset + 2].copy_from_slice(&size.to_le_bytes());
    }
    output.truncate(write_pos);
}

fn tight_stream_bound(source_size: usize, max_num_bits: u8) -> usize {
    (source_size * usize::from(max_num_bits)).div_ceil(8) + 1
}

#[inline(always)]
fn encode_stream<const UNROLL: usize>(
    output: &mut [u8],
    mut write_pos: usize,
    codes: &[HuffmanCode; 256],
    data: &[u8],
) -> usize {
    debug_assert!(UNROLL > 0);
    let mut container = 0u64;
    let mut bit_count = 0usize;
    let mut remaining = data.len();

    let remainder = remaining % UNROLL;
    if remainder != 0 {
        let (batch, batch_bits) = pack_remainder(data, remaining - remainder, remaining, codes);
        append_batch(
            output,
            &mut write_pos,
            &mut container,
            &mut bit_count,
            batch,
            batch_bits,
        );
        remaining -= remainder;
    }

    if !remaining.is_multiple_of(2 * UNROLL) {
        let (batch, batch_bits) = pack_full::<UNROLL>(data, remaining - UNROLL, codes);
        append_batch(
            output,
            &mut write_pos,
            &mut container,
            &mut bit_count,
            batch,
            batch_bits,
        );
        remaining -= UNROLL;
    }

    while remaining != 0 {
        let upper_start = remaining - UNROLL;
        let lower_start = upper_start - UNROLL;
        let (upper, upper_bits) = pack_full::<UNROLL>(data, upper_start, codes);
        let (lower, lower_bits) = pack_full::<UNROLL>(data, lower_start, codes);

        append_batch(
            output,
            &mut write_pos,
            &mut container,
            &mut bit_count,
            upper,
            upper_bits,
        );
        append_batch(
            output,
            &mut write_pos,
            &mut container,
            &mut bit_count,
            lower,
            lower_bits,
        );
        remaining -= 2 * UNROLL;
    }

    container |= 1 << bit_count;
    bit_count += 1;
    store_word(output, write_pos, container);
    write_pos += bit_count.div_ceil(8);
    write_pos
}

#[inline(always)]
fn pack_remainder(
    data: &[u8],
    start: usize,
    end: usize,
    codes: &[HuffmanCode; 256],
) -> (u64, usize) {
    debug_assert!(start <= end);
    debug_assert!(end <= data.len());
    let mut container = 0u64;
    let mut bit_count = 0usize;
    for index in (start..end).rev() {
        // SAFETY: Every caller derives `start` and `end` from the current
        // remaining input length; the debug assertions state that invariant.
        let symbol = unsafe { *data.get_unchecked(index) };
        // SAFETY: A byte indexes the complete 256-entry code table.
        let (code, num_bits) = unsafe { *codes.get_unchecked(usize::from(symbol)) };
        debug_assert!(num_bits > 0);
        container |= u64::from(code) << bit_count;
        bit_count += usize::from(num_bits);
    }
    (container, bit_count)
}

#[inline(always)]
fn pack_full<const UNROLL: usize>(
    data: &[u8],
    start: usize,
    codes: &[HuffmanCode; 256],
) -> (u64, usize) {
    match UNROLL {
        5 => pack_5(data, start, codes),
        6 => pack_6(data, start, codes),
        7 => pack_7(data, start, codes),
        8 => pack_8(data, start, codes),
        9 => pack_9(data, start, codes),
        _ => unreachable!("Huffman unroll is selected from 5 through 9"),
    }
}

macro_rules! define_full_pack {
    ($name:ident, $width:literal, $($offset:literal),+ $(,)?) => {
        #[inline(always)]
        fn $name(
            data: &[u8],
            start: usize,
            codes: &[HuffmanCode; 256],
        ) -> (u64, usize) {
            debug_assert!(start + $width <= data.len());
            let mut container = 0u64;
            let mut bit_count = 0usize;
            $(
                // SAFETY: the caller supplies a complete `$width`-symbol
                // batch and every listed offset is inside that batch. A byte
                // indexes the complete 256-entry code table.
                let symbol = unsafe { *data.get_unchecked(start + $offset) };
                let (code, num_bits) =
                    unsafe { *codes.get_unchecked(usize::from(symbol)) };
                debug_assert!(num_bits > 0);
                container |= u64::from(code) << bit_count;
                bit_count += usize::from(num_bits);
            )+
            (container, bit_count)
        }
    };
}

// C's HUF_FLUSHBITS_n() templates spell out every HUF_ADD_BITS() operation.
// Keep the same reverse-symbol order without a runtime loop in full batches.
define_full_pack!(pack_5, 5, 4, 3, 2, 1, 0);
define_full_pack!(pack_6, 6, 5, 4, 3, 2, 1, 0);
define_full_pack!(pack_7, 7, 6, 5, 4, 3, 2, 1, 0);
define_full_pack!(pack_8, 8, 7, 6, 5, 4, 3, 2, 1, 0);
define_full_pack!(pack_9, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);

#[inline(always)]
fn append_batch(
    output: &mut [u8],
    write_pos: &mut usize,
    container: &mut u64,
    bit_count: &mut usize,
    batch: u64,
    batch_bits: usize,
) {
    debug_assert!(*bit_count + batch_bits < 64);
    *container |= batch << *bit_count;
    *bit_count += batch_bits;

    let full_bytes = *bit_count / 8;
    store_word(output, *write_pos, *container);
    *write_pos += full_bytes;
    *container >>= full_bytes * 8;
    *bit_count -= full_bytes * 8;
}

#[inline(always)]
fn store_word(output: &mut [u8], write_pos: usize, value: u64) {
    debug_assert!(write_pos + 8 <= output.len());
    // SAFETY: `encode_four_streams()` reserves the sum of every tight stream
    // bound plus seven initialized padding bytes before emission. Each stream
    // advances by only its emitted whole bytes, so every overlapping word
    // store, including the final marker, remains inside that reservation.
    unsafe {
        core::ptr::write_unaligned(
            output.as_mut_ptr().add(write_pos).cast::<u64>(),
            value.to_le(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{encode_four_streams, HuffmanCode};
    use alloc::vec::Vec;
    use core::convert::TryFrom;

    fn encode_reference_stream(output: &mut Vec<u8>, codes: &[HuffmanCode; 256], data: &[u8]) {
        let mut container = 0u64;
        let mut bit_count = 0usize;
        for &symbol in data.iter().rev() {
            let (code, num_bits) = codes[usize::from(symbol)];
            container |= u64::from(code) << bit_count;
            bit_count += usize::from(num_bits);
            while bit_count >= 8 {
                output.push(container as u8);
                container >>= 8;
                bit_count -= 8;
            }
        }
        container |= 1 << bit_count;
        bit_count += 1;
        while bit_count != 0 {
            output.push(container as u8);
            container >>= 8;
            bit_count = bit_count.saturating_sub(8);
        }
    }

    #[test]
    fn specialized_four_streams_match_symbol_reference() {
        let data = (0usize..4099)
            .map(|index| ((index * 37 + index / 13) & 0xff) as u8)
            .collect::<Vec<_>>();

        for max_num_bits in 1u8..=11 {
            let codes = core::array::from_fn(|symbol| {
                let num_bits = 1 + (symbol as u8 % max_num_bits);
                let code = symbol as u32 & ((1u32 << num_bits) - 1);
                (code, num_bits)
            });

            let split_size = data.len().div_ceil(4);
            let streams = [
                &data[..split_size],
                &data[split_size..split_size * 2],
                &data[split_size * 2..split_size * 3],
                &data[split_size * 3..],
            ];
            let mut expected = Vec::from([0u8; 6]);
            for (index, stream) in streams.iter().copied().enumerate() {
                let start = expected.len();
                encode_reference_stream(&mut expected, &codes, stream);
                if index < 3 {
                    let size = u16::try_from(expected.len() - start).unwrap();
                    expected[index * 2..index * 2 + 2].copy_from_slice(&size.to_le_bytes());
                }
            }

            let mut actual = Vec::new();
            // SAFETY: the generated table and input satisfy the encoder contract.
            unsafe { encode_four_streams(&mut actual, &codes, max_num_bits, &data, false) };
            assert_eq!(actual, expected, "table log {max_num_bits}");
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn bmi2_and_default_four_streams_match() {
        if !crate::cpu::bmi2_supported() {
            return;
        }

        let data = (0usize..4099)
            .map(|index| ((index * 37 + index / 13) & 0xff) as u8)
            .collect::<Vec<_>>();
        for max_num_bits in 1u8..=11 {
            let codes = core::array::from_fn(|symbol| {
                let num_bits = 1 + (symbol as u8 % max_num_bits);
                let code = symbol as u32 & ((1u32 << num_bits) - 1);
                (code, num_bits)
            });
            let mut default = Vec::new();
            let mut bmi2 = Vec::new();
            // SAFETY: the generated table is valid and this test is BMI2-gated.
            unsafe {
                encode_four_streams(&mut default, &codes, max_num_bits, &data, false);
                encode_four_streams(&mut bmi2, &codes, max_num_bits, &data, true);
            }
            assert_eq!(bmi2, default);
        }
    }
}
