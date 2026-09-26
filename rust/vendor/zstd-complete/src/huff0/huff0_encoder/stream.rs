use alloc::vec::Vec;
use core::convert::TryFrom;

use crate::bit_io::BitWriter;

/// Encodes one Huffman stream with C's table-log-sized batching strategy.
pub(super) fn encode_stream<V: AsMut<Vec<u8>>>(
    writer: &mut BitWriter<V>,
    codes: &[(u32, u8)],
    max_num_bits: u8,
    data: &[u8],
) {
    if let Ok(codes) = <&[(u32, u8); 256]>::try_from(codes) {
        encode_full_table_stream(writer, codes, max_num_bits, data);
    } else {
        encode_compact_table_stream(writer, codes, max_num_bits, data);
    }
}

/// Normal emitted tables are built from the complete byte alphabet. Keeping
/// this loop separate gives safe Rust the same statically bounded code-table
/// lookup as C's fixed `HUF_CElt CTable[256]`.
fn encode_full_table_stream<V: AsMut<Vec<u8>>>(
    writer: &mut BitWriter<V>,
    codes: &[(u32, u8); 256],
    max_num_bits: u8,
    data: &[u8],
) {
    debug_assert!(max_num_bits > 0);
    let symbols_per_batch = 56 / usize::from(max_num_bits);
    debug_assert!(symbols_per_batch > 0);

    writer.append_aligned_with(|output| {
        let max_stream_bytes = (data.len() * usize::from(max_num_bits)).div_ceil(8) + 1;
        let mut write_pos = output.len();
        output.resize(write_pos + max_stream_bytes + 7, 0);
        let mut container = 0u64;
        let mut bit_count = 0usize;

        for batch in data.rchunks(symbols_per_batch) {
            for &symbol in batch.iter().rev() {
                let (code, num_bits) = codes[usize::from(symbol)];
                debug_assert!(num_bits > 0);
                container |= u64::from(code) << bit_count;
                bit_count += usize::from(num_bits);
            }

            let full_bytes = bit_count / 8;
            output[write_pos..write_pos + 8].copy_from_slice(&container.to_le_bytes());
            write_pos += full_bytes;
            container >>= full_bytes * 8;
            bit_count -= full_bytes * 8;
        }

        container |= 1 << bit_count;
        bit_count += 1;
        output[write_pos..write_pos + 8].copy_from_slice(&container.to_le_bytes());
        write_pos += bit_count.div_ceil(8);
        output.truncate(write_pos);
    });
}

/// Full dictionaries and small direct construction tests may retain only the
/// represented symbol prefix, so preserve a separately generated checked
/// fallback rather than padding or moving the owning code vector.
fn encode_compact_table_stream<V: AsMut<Vec<u8>>>(
    writer: &mut BitWriter<V>,
    codes: &[(u32, u8)],
    max_num_bits: u8,
    data: &[u8],
) {
    debug_assert!(max_num_bits > 0);
    let symbols_per_batch = 56 / usize::from(max_num_bits);
    debug_assert!(symbols_per_batch > 0);

    writer.append_aligned_with(|output| {
        let max_stream_bytes = (data.len() * usize::from(max_num_bits)).div_ceil(8) + 1;
        let mut write_pos = output.len();
        output.resize(write_pos + max_stream_bytes + 7, 0);
        let mut container = 0u64;
        let mut bit_count = 0usize;

        for batch in data.rchunks(symbols_per_batch) {
            for &symbol in batch.iter().rev() {
                let (code, num_bits) = codes[usize::from(symbol)];
                debug_assert!(num_bits > 0);
                container |= u64::from(code) << bit_count;
                bit_count += usize::from(num_bits);
            }

            let full_bytes = bit_count / 8;
            output[write_pos..write_pos + 8].copy_from_slice(&container.to_le_bytes());
            write_pos += full_bytes;
            container >>= full_bytes * 8;
            bit_count -= full_bytes * 8;
        }

        container |= 1 << bit_count;
        bit_count += 1;
        output[write_pos..write_pos + 8].copy_from_slice(&container.to_le_bytes());
        write_pos += bit_count.div_ceil(8);
        output.truncate(write_pos);
    });
}

#[cfg(test)]
mod tests {
    use super::encode_stream;
    use crate::bit_io::BitWriter;
    use alloc::vec::Vec;

    #[test]
    fn batched_stream_matches_per_symbol_bit_writer() {
        let data = (0usize..1027)
            .map(|index| ((index * 37 + index / 13) & 0xff) as u8)
            .collect::<Vec<_>>();

        for max_num_bits in 1u8..=11 {
            let codes = (0u32..=255)
                .map(|symbol| {
                    let num_bits = 1 + (symbol as u8 % max_num_bits);
                    let code = symbol & ((1u32 << num_bits) - 1);
                    (code, num_bits)
                })
                .collect::<Vec<_>>();
            let mut expected = Vec::new();
            let mut expected_writer = BitWriter::from(&mut expected);
            for &symbol in data.iter().rev() {
                let (code, num_bits) = codes[usize::from(symbol)];
                expected_writer.write_bits(code, usize::from(num_bits));
            }
            let bits_to_fill = expected_writer.misaligned();
            expected_writer.write_bits(1u32, if bits_to_fill == 0 { 8 } else { bits_to_fill });
            expected_writer.flush();

            let mut actual = Vec::new();
            let mut actual_writer = BitWriter::from(&mut actual);
            encode_stream(&mut actual_writer, &codes, max_num_bits, &data);
            actual_writer.flush();

            assert_eq!(actual, expected, "table log {max_num_bits}");
        }
    }

    #[test]
    fn compact_table_fallback_matches_per_symbol_bit_writer() {
        let codes = [(0, 1), (1, 2), (3, 3), (2, 3), (6, 3)];
        let data = (0usize..103)
            .map(|index| (index % 5) as u8)
            .collect::<Vec<_>>();

        let mut expected = Vec::new();
        let mut expected_writer = BitWriter::from(&mut expected);
        for &symbol in data.iter().rev() {
            let (code, num_bits) = codes[usize::from(symbol)];
            expected_writer.write_bits(code, usize::from(num_bits));
        }
        let bits_to_fill = expected_writer.misaligned();
        expected_writer.write_bits(1u32, if bits_to_fill == 0 { 8 } else { bits_to_fill });
        expected_writer.flush();

        let mut actual = Vec::new();
        let mut actual_writer = BitWriter::from(&mut actual);
        encode_stream(&mut actual_writer, &codes, 3, &data);
        actual_writer.flush();

        assert_eq!(actual, expected);
    }
}
