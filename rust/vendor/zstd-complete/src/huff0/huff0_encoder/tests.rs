use alloc::{vec, vec::Vec};

use crate::{bit_io::BitWriter, fse::fse_encoder::FSETableBuildScratch};

use super::{
    lengths::{
        distribute_weights, length_units, rank_limited_weights, rank_limited_weights_into,
        redistribute_weights,
    },
    tree::{
        c_sorted_huffman_nodes, length_limited_code_lengths,
        length_limited_code_lengths_with_nodes, HuffmanNode,
    },
    weights::{
        c_huff_weight_fse_is_unusable, encode_weight_table_fse_bytes, encoded_weight_table_bytes,
        raw_weight_table_bytes, raw_weight_table_is_supported,
        table_description_bytes_from_weights, table_description_bytes_from_weights_reusing,
        table_description_bytes_from_weights_reusing_with_fse_scratch, weights_from_codes,
    },
    HuffmanBuildScratch, HuffmanEncoder, HuffmanTable, MAX_HUFFMAN_BITS,
};

pub(crate) fn four_stream_counts(data: &[u8]) -> [[usize; 256]; 4] {
    let mut counts = [[0usize; 256]; 4];
    let split_size = data.len().div_ceil(4);
    for (stream_idx, stream_counts) in counts.iter_mut().enumerate() {
        let start = split_size * stream_idx;
        if start >= data.len() {
            break;
        }
        let end = (start + split_size).min(data.len());
        for symbol in &data[start..end] {
            stream_counts[usize::from(*symbol)] += 1;
        }
    }
    counts
}

#[test]
fn code_lengths_use_c_bits_to_weight_mapping() {
    let codes = [(0, 0), (0, 1), (0, 6), (0, 11)];
    assert_eq!(weights_from_codes(&codes, 11), [0, 11, 6, 1]);
}

#[test]
fn huffman() {
    let table = HuffmanTable::build_from_weights(&[2, 2, 2, 1, 1]);
    assert_eq!(table.codes[0], (1, 2));
    assert_eq!(table.codes[1], (2, 2));
    assert_eq!(table.codes[2], (3, 2));
    assert_eq!(table.codes[3], (0, 3));
    assert_eq!(table.codes[4], (1, 3));

    let table = HuffmanTable::build_from_weights(&[4, 3, 2, 0, 1, 1]);
    assert_eq!(table.codes[0], (1, 1));
    assert_eq!(table.codes[1], (1, 2));
    assert_eq!(table.codes[2], (1, 3));
    assert_eq!(table.codes[3], (0, 0));
    assert_eq!(table.codes[4], (0, 4));
    assert_eq!(table.codes[5], (1, 4));
}

/// Distributes weights that add up to a clean power of two

#[test]
fn weights() {
    // assert_eq!(distribute_weights(5).as_slice(), &[1, 1, 2, 3, 4]);
    for amount in 2..=256 {
        let mut weights = distribute_weights(amount);
        assert_eq!(weights.len(), amount);
        let sum = weights
            .iter()
            .copied()
            .map(|weight| 1 << weight)
            .sum::<usize>();
        assert!(sum.is_power_of_two());

        for num_bit_limit in (amount.ilog2() as usize + 1)..=11 {
            redistribute_weights(&mut weights, num_bit_limit);
            let sum = weights
                .iter()
                .copied()
                .map(|weight| 1 << weight)
                .sum::<usize>();
            assert!(sum.is_power_of_two());
            assert!(
                sum.ilog2() <= 11,
                "Max bits too big: sum: {} {weights:?}",
                sum
            );

            let code_weights = weights
                .iter()
                .map(|weight| usize::from(*weight))
                .collect::<Vec<_>>();
            let codes = HuffmanTable::build_from_weights(&code_weights).codes;
            for (code, num_bits) in codes.iter().copied() {
                for (code2, num_bits2) in codes.iter().copied() {
                    if num_bits == 0 || num_bits2 == 0 || (code, num_bits) == (code2, num_bits2) {
                        continue;
                    }
                    if num_bits <= num_bits2 {
                        let code2_shifted = code2 >> (num_bits2 - num_bits);
                        assert_ne!(
                            code, code2_shifted,
                            "{code:b},{num_bits:} is prefix of {code2:b},{num_bits2:}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn counts() {
    let counts = &[3, 0, 4, 1, 5];
    let table = HuffmanTable::build_from_counts(counts).codes;

    assert_eq!(table[1].1, 0);
    assert!(table[3].1 >= table[0].1);
    assert!(table[0].1 >= table[2].1);
    assert!(table[2].1 >= table[4].1);

    let counts = &[3, 0, 4, 0, 7, 2, 2, 2, 0, 2, 2, 1, 5];
    let table = HuffmanTable::build_from_counts(counts).codes;

    assert_eq!(table[1].1, 0);
    assert_eq!(table[3].1, 0);
    assert_eq!(table[8].1, 0);
    assert!(table[11].1 >= table[5].1);
    assert!(table[5].1 >= table[6].1);
    assert!(table[6].1 >= table[7].1);
    assert!(table[7].1 >= table[9].1);
    assert!(table[9].1 >= table[10].1);
    assert!(table[10].1 >= table[0].1);
    assert!(table[0].1 >= table[2].1);
    assert!(table[2].1 >= table[12].1);
    assert!(table[12].1 >= table[4].1);
}

#[test]
fn cached_max_num_bits_matches_codes() {
    let cases: &[&[usize]] = &[
        &[3, 0, 4, 1, 5],
        &[16, 16, 16, 16, 16, 16, 16, 16],
        &[1, 1, 2, 3, 5, 8, 13, 21],
        &[0, 7, 7, 7, 7, 7, 0],
    ];

    for counts in cases {
        let table = HuffmanTable::build_from_counts(counts);
        let max_num_bits = table.codes.iter().map(|(_, bits)| *bits).max().unwrap_or(0);
        assert_eq!(table.max_num_bits, max_num_bits);
    }
}

#[test]
fn build_from_counts_produces_bounded_prefix_free_codes() {
    let flat_counts = [1usize; 256];
    let sparse_skewed_counts = [
        3, 0, 4, 0, 7, 2, 2, 2, 0, 2, 2, 1, 5, 144, 89, 55, 34, 21, 13, 8,
    ];
    let tied_counts = [8, 8, 4, 4, 2, 2, 1, 1, 0, 16, 16, 32, 32];
    let cases: &[&[usize]] = &[&flat_counts, &sparse_skewed_counts, &tied_counts];

    for counts in cases {
        let table = HuffmanTable::build_from_counts(counts);

        assert_eq!(table.codes.len(), counts.len());
        assert!(table.max_num_bits <= MAX_HUFFMAN_BITS as u8);
        for (symbol, (code, num_bits)) in table.codes.iter().copied().enumerate() {
            assert_eq!(
                num_bits == 0,
                counts[symbol] == 0,
                "symbol {symbol} has count {} and code ({code:b}, {num_bits})",
                counts[symbol]
            );
            assert!(num_bits <= MAX_HUFFMAN_BITS as u8);
        }
        assert_prefix_free(&table.codes);
    }
}

#[test]
fn build_smallest_from_counts_reduces_small_repeated_text_literals() {
    let data = b"the quick brown fox jumps over the lazy dog\n\
zstd-rs fastest encoder repeated text fixture\n\
0123456789 abcdefghijklmnopqrstuvwxyz\n\
the quick brown fox";
    let mut counts = [0usize; 256];
    let mut max_symbol = 0usize;
    for symbol in data {
        let symbol = *symbol as usize;
        counts[symbol] += 1;
        max_symbol = max_symbol.max(symbol);
    }
    let counts = &counts[..=max_symbol];

    let baseline = HuffmanTable::build_from_counts(counts);
    let smallest = HuffmanTable::build_smallest_from_counts(counts, data, false);

    assert!(smallest.encoded_len(data, true, false) < baseline.encoded_len(data, true, false));
    assert!(smallest.max_num_bits <= MAX_HUFFMAN_BITS as u8);
    assert_prefix_free(&smallest.codes);
}

#[test]
fn build_smallest_from_counts_keeps_flat_distribution_table() {
    let data = (0..=255).chain(0..=255).collect::<Vec<_>>();
    let counts = [2usize; 256];

    let baseline = HuffmanTable::build_from_counts(&counts);
    let smallest = HuffmanTable::build_smallest_from_counts(&counts, &data, false);

    assert_eq!(smallest.codes, baseline.codes);
    assert_eq!(
        smallest.table_description_len(),
        baseline.table_description_len()
    );
}

#[test]
fn can_encode_counts_checks_symbols_without_building_table() {
    let table = HuffmanTable::build_from_counts(&[4, 3, 0, 1]);

    assert!(table.can_encode_counts(&[1, 2, 0, 1]));
    assert!(!table.can_encode_counts(&[1, 2, 1, 1]));
    assert!(!table.can_encode_counts(&[1, 2, 0, 1, 1]));
}

#[test]
fn rank_limited_weights_preserve_symbol_order_for_equal_counts() {
    let mut expected_nonzero = distribute_weights(5);
    let limit = expected_nonzero.len().ilog2() as usize + 2;
    redistribute_weights(&mut expected_nonzero, limit);

    let weights = rank_limited_weights(&[0, 7, 7, 7, 7, 7, 0]);

    assert_eq!(weights[0], 0);
    assert_eq!(weights[6], 0);
    assert_eq!(
        &weights[1..6],
        expected_nonzero
            .iter()
            .map(|weight| usize::from(*weight))
            .collect::<Vec<_>>()
            .as_slice()
    );
}

#[test]
fn fixed_rank_limited_workspace_matches_owned_builder() {
    for counts in [
        vec![1, 1],
        vec![0, 7, 7, 7, 7, 7, 0],
        (0..256)
            .map(|symbol| usize::from(symbol % 11 != 0) * (symbol * 17 % 101 + 1))
            .collect(),
    ] {
        let expected = rank_limited_weights(&counts);
        let mut storage = [0usize; 256];
        assert_eq!(
            rank_limited_weights_into(&counts, &mut storage),
            expected.as_slice()
        );
    }
}

#[test]
fn length_limited_code_lengths_are_stable_for_tied_counts() {
    let counts = &[8, 8, 4, 4, 2, 2, 1, 1, 0, 16, 16, 32, 32];
    let lengths = length_limited_code_lengths(counts, MAX_HUFFMAN_BITS)
        .expect("length-limited table should be valid");

    for _ in 0..8 {
        assert_eq!(
            length_limited_code_lengths(counts, MAX_HUFFMAN_BITS),
            Some(lengths.clone())
        );
    }

    assert_eq!(
        length_units(&lengths, MAX_HUFFMAN_BITS),
        1 << MAX_HUFFMAN_BITS
    );
}

#[test]
fn from_data() {
    let data = &[0, 2, 4, 4, 0, 3, 2, 2, 0, 2];
    let table = HuffmanTable::build_from_data(data).codes;

    assert_eq!(table[1].1, 0);
    for symbol in [0, 2, 3, 4] {
        assert!(table[symbol].1 > 0);
        assert!(table[symbol].1 <= MAX_HUFFMAN_BITS as u8);
    }
}

#[test]
fn c_style_weight_fse_handles_sparse_short_weight_tables() {
    let counts = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 27, 0, 15, 0, 8, 0, 2, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 26, 0, 10, 0, 4, 0, 2, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34, 0, 168, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        90, 0, 15, 0, 9, 0, 1, 0, 2, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 15, 0, 4, 0, 2, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 36, 0, 13, 0, 3, 0, 1,
    ];
    let table = HuffmanTable::build_from_counts(&counts);

    assert_eq!(table.table_description_len(), 33);
}

#[test]
fn encoded_len_matches_single_stream_encoder() {
    let data = b"abbcccddddeeeee";
    let table = HuffmanTable::build_from_data(data);
    let mut counts = [0usize; 256];
    for byte in data {
        counts[usize::from(*byte)] += 1;
    }

    assert_eq!(
        table.encoded_len(data, true, false),
        actual_encoded_len(&table, data, true, false)
    );
    assert_eq!(
        table.encoded_len(data, false, false),
        actual_encoded_len(&table, data, false, false)
    );
    assert_eq!(
        table.encoded_len_from_counts(&counts, true),
        table.encoded_len(data, true, false)
    );
    assert_eq!(
        table.encoded_len_from_counts(&counts, false),
        table.encoded_len(data, false, false)
    );
}

#[test]
fn estimated_compressed_size_matches_c_unpadded_bit_count() {
    let data = b"abbcccddddeeeee";
    let table = HuffmanTable::build_from_data(data);
    let mut counts = [0usize; 256];
    for byte in data {
        counts[usize::from(*byte)] += 1;
    }

    assert_eq!(
        table.estimated_compressed_size_from_counts(&counts),
        table.encoded_len_from_counts(&counts, false) - 1
    );
}

#[test]
fn weight_table_uses_c_fse_threshold() {
    let raw_fallback_weights = (0u8..24).map(|idx| idx % 12).collect::<Vec<_>>();
    assert_eq!(
        encoded_weight_table_bytes(&raw_fallback_weights),
        raw_weight_table_bytes(&raw_fallback_weights)
    );

    let too_many_raw_weights = (0u8..129).map(|idx| idx % 12).collect::<Vec<_>>();
    assert!(!raw_weight_table_is_supported(&too_many_raw_weights));
    assert_ne!(
        encoded_weight_table_bytes(&too_many_raw_weights)[0],
        (too_many_raw_weights.len() as u8).wrapping_add(127)
    );

    for seed in 1u32..=64 {
        let mut state = seed;
        let len = 17 + (seed as usize % 24);
        let mut weights = Vec::with_capacity(len);
        for _ in 0..len {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            weights.push((state % 12) as u8);
        }

        let encoded = encoded_weight_table_bytes(&weights);
        let fse = encode_weight_table_fse_bytes(&weights, c_huff_weight_fse_is_unusable(&weights));
        let raw = raw_weight_table_bytes(&weights);
        let fse_payload_len = fse.len().saturating_sub(1);

        if fse_payload_len > 1 && fse_payload_len < weights.len() / 2 {
            assert_eq!(encoded, fse);
        } else {
            assert_eq!(encoded, raw);
        }
    }
}

#[test]
fn encoded_len_matches_four_stream_encoder() {
    let data = b"tenant=alpha path=/v1/archive status=200 tenant=beta path=/v1/search status=404";
    let table = HuffmanTable::build_from_data(data);
    let counts = four_stream_counts(data);

    assert_eq!(
        table.encoded_len(data, true, true),
        actual_encoded_len(&table, data, true, true)
    );
    assert_eq!(
        table.encoded_len(data, false, true),
        actual_encoded_len(&table, data, false, true)
    );
    assert_eq!(
        table.encoded_len_from_stream_counts(&counts, true),
        table.encoded_len(data, true, true)
    );
    assert_eq!(
        table.encoded_len_from_stream_counts(&counts, false),
        table.encoded_len(data, false, true)
    );
}

#[cfg(test)]
fn actual_encoded_len(
    table: &HuffmanTable,
    data: &[u8],
    with_table: bool,
    four_streams: bool,
) -> usize {
    let mut encoded = Vec::new();
    let mut writer = BitWriter::from(&mut encoded);
    let mut encoder = HuffmanEncoder::new(table, &mut writer);
    if four_streams {
        encoder.encode4x(data, with_table);
    } else {
        encoder.encode(data, with_table);
    }
    writer.flush();
    encoded.len()
}

#[cfg(test)]
fn assert_prefix_free(codes: &[(u32, u8)]) {
    for (idx, (code, num_bits)) in codes.iter().copied().enumerate() {
        if num_bits == 0 {
            continue;
        }
        for (other_idx, (other_code, other_num_bits)) in codes.iter().copied().enumerate() {
            if idx == other_idx || other_num_bits == 0 {
                continue;
            }
            if num_bits <= other_num_bits {
                let other_code_prefix = other_code >> (other_num_bits - num_bits);
                assert_ne!(
                    code, other_code_prefix,
                    "symbol {idx}'s {num_bits}-bit code is a prefix of symbol {other_idx}'s {other_num_bits}-bit code"
                );
            }
        }
    }
}

#[test]
fn compact_tree_builder_matches_vector_code_length_builder() {
    let counts = [10_000usize, 2_000, 800, 300, 120, 50, 20, 8, 3, 1, 1, 1];
    for max_bits in 4..=MAX_HUFFMAN_BITS {
        let mut direct_nodes = Vec::new();
        let direct = HuffmanTable::build_c_table_with_nodes(&counts, max_bits, &mut direct_nodes)
            .expect("the selected table log can represent this alphabet");
        let mut vector_nodes = Vec::new();
        let lengths = length_limited_code_lengths_with_nodes(&counts, max_bits, &mut vector_nodes)
            .expect("the vector builder can represent this alphabet");
        let vector = HuffmanTable::build_from_code_lengths(&lengths, max_bits);
        assert_eq!(direct.codes, vector.codes);
        assert_eq!(direct.table_description, vector.table_description);
    }
}

#[test]
fn generated_tree_to_weight_transaction_matches_local_builder() {
    let cases = [
        vec![10_000usize, 2_000, 800, 300, 120, 50, 20, 8, 3, 1, 1, 1],
        (0..64).map(|symbol| symbol * 37 % 113 + 1).collect(),
        (0..256)
            .map(|symbol| usize::from(symbol % 7 != 0) * (symbol * 13 % 29 + 1))
            .collect(),
    ];

    for counts in cases {
        for max_bits in 4..=MAX_HUFFMAN_BITS {
            let mut local_nodes = Vec::new();
            let local = HuffmanTable::build_c_table_with_nodes(&counts, max_bits, &mut local_nodes);
            let mut generated_scratch = crate::kernel::huff0::HuffmanBuildScratch::default();
            let generated = crate::kernel::huff0::build_huffman_table(
                &counts,
                max_bits,
                &mut generated_scratch,
            );

            match (local, generated) {
                (None, None) => {}
                (Some(local), Some(generated)) => {
                    assert_eq!(generated.codes.as_slice(), local.codes.as_slice());
                    assert_eq!(generated.max_num_bits, local.max_num_bits);
                    assert_eq!(
                        generated.weights,
                        weights_from_codes(&local.codes, local.max_num_bits)
                    );
                    let mut described_scratch =
                        crate::kernel::huff0::HuffmanBuildScratch::default();
                    let described = crate::kernel::huff0::build_described_huffman_table(
                        &counts,
                        max_bits,
                        &mut described_scratch,
                        super::weights::table_description_bytes_from_weights,
                    )
                    .expect("the equivalent generated table is representable");
                    assert_eq!(described.codes.as_slice(), local.codes.as_slice());
                    assert_eq!(described.max_num_bits, local.max_num_bits);
                    assert_eq!(
                        described.table_description.as_slice(),
                        local.table_description.as_slice()
                    );
                }
                _ => panic!(
                    "generated and local Huffman builders disagreed at log {}",
                    max_bits
                ),
            }
        }
    }
}

#[test]
fn huffman_sort_matches_c_log_bucket_tie_order() {
    assert_eq!(core::mem::size_of::<HuffmanNode>(), 8);
    let nodes = c_sorted_huffman_nodes(&[200usize; 9]);
    let symbols = nodes
        .into_iter()
        .skip(1)
        .map(|node| node.symbol)
        .collect::<Vec<_>>();
    assert_eq!(symbols, [8, 0, 1, 2, 3, 4, 5, 6, 7]);
}

#[test]
fn c_fast_table_log_matches_huf_non_optimal_depth_shape() {
    assert_eq!(HuffmanTable::c_fast_table_log(1493, 255), 9);
    assert_eq!(HuffmanTable::c_fast_table_log(3554, 255), 10);
    assert_eq!(HuffmanTable::c_fast_table_log(40_960, 255), 11);
}

#[test]
fn c_fast_table_log_reduces_normal_literal_table_description() {
    let counts = [1usize; 256];
    let table_log = HuffmanTable::c_fast_table_log(1420, counts.len() - 1);
    let shaped = HuffmanTable::build_from_counts_with_max_bits(&counts, table_log);
    assert_eq!(table_log, 9);
    assert!(shaped.table_description_len() > 0);
}

#[test]
fn reusable_huffman_output_table_preserves_allocations_and_bytes() {
    let first_counts = (0..256)
        .map(|symbol| 1 + (symbol * 17 % 31))
        .collect::<Vec<_>>();
    let second_counts = (0..256)
        .map(|symbol| 1 + (symbol * 29 % 23))
        .collect::<Vec<_>>();
    let mut scratch = HuffmanBuildScratch::new();
    let first = HuffmanTable::build_from_counts_with_max_bits_and_scratch(
        &first_counts,
        MAX_HUFFMAN_BITS,
        &mut scratch,
    );
    let codes_ptr = first.codes.as_ptr();
    let description_ptr = first.table_description.as_ptr();
    scratch.recycle_table(first);
    assert_eq!(scratch.recycled_table_count(), 1);

    let reused = HuffmanTable::build_from_counts_with_max_bits_and_scratch(
        &second_counts,
        MAX_HUFFMAN_BITS,
        &mut scratch,
    );
    let mut local_nodes = Vec::new();
    let reference =
        HuffmanTable::build_c_table_with_nodes(&second_counts, MAX_HUFFMAN_BITS, &mut local_nodes)
            .expect("dense histogram builds a C table");

    assert_eq!(scratch.recycled_table_count(), 0);
    assert_eq!(reused.codes, reference.codes);
    assert_eq!(reused.max_num_bits, reference.max_num_bits);
    assert_eq!(reused.table_description, reference.table_description);
    assert_eq!(reused.codes.as_ptr(), codes_ptr);
    assert_eq!(reused.table_description.as_ptr(), description_ptr);
}

#[test]
fn reusable_table_description_matches_allocating_serializer() {
    let fixtures = [
        vec![1, 1],
        vec![1, 2, 3, 4, 5, 6, 7],
        (0..31).map(|index| 1 + (index % 6) as u8).collect(),
        (0..127).map(|index| 1 + (index % 10) as u8).collect(),
        (0..255).map(|index| 1 + (index % 11) as u8).collect(),
    ];
    let mut reused = Vec::with_capacity(256);
    for mut weights in fixtures {
        weights.push(1);
        let expected = table_description_bytes_from_weights(&weights);
        table_description_bytes_from_weights_reusing(&weights, &mut reused);
        assert_eq!(reused, expected);
    }
}

#[test]
fn reusable_weight_fse_workspace_preserves_description_and_table_allocations() {
    let fixtures = [
        (0..255).map(|index| 1 + (index % 11) as u8).collect(),
        vec![1; 256],
        (0..255).map(|index| 1 + (index * 7 % 10) as u8).collect(),
    ];
    let mut output = Vec::with_capacity(256);
    let mut scratch = FSETableBuildScratch::new();
    let mut retained_allocations = None;

    for mut weights in fixtures {
        weights.push(1);
        let expected = table_description_bytes_from_weights(&weights);
        table_description_bytes_from_weights_reusing_with_fse_scratch(
            &weights,
            &mut output,
            &mut scratch,
        );
        assert_eq!(output, expected);
        assert_eq!(scratch.recycled_table_count(), 1);
        let allocations = scratch.recycled_table_allocation_addresses();
        if let Some(previous) = &retained_allocations {
            assert_eq!(&allocations, previous);
        }
        retained_allocations = Some(allocations);
    }
}

#[test]
fn nested_fse_workspace_preserves_complete_huffman_table() {
    let counts = (0..256)
        .map(|symbol| 1 + (symbol * 37 % 41))
        .collect::<Vec<_>>();
    let mut huffman_scratch = HuffmanBuildScratch::new();
    let mut fse_scratch = FSETableBuildScratch::new();
    let actual = HuffmanTable::build_from_counts_with_max_bits_and_workspaces(
        &counts,
        MAX_HUFFMAN_BITS,
        &mut huffman_scratch,
        &mut fse_scratch,
    );
    let expected = HuffmanTable::build_from_counts_with_max_bits(&counts, MAX_HUFFMAN_BITS);

    assert_eq!(actual.codes, expected.codes);
    assert_eq!(actual.max_num_bits, expected.max_num_bits);
    assert_eq!(actual.table_description, expected.table_description);
    assert_eq!(fse_scratch.recycled_table_count(), 1);
}

#[test]
fn c_optimal_depth_reuses_base_tree_without_changing_tables() {
    let counts = [
        76usize, 70, 55, 52, 29, 64, 69, 59, 43, 58, 50, 71, 47, 56, 71, 48,
    ];
    let optimized = HuffmanTable::build_c_optimal_depth_from_counts(&counts);
    let baseline = HuffmanTable::build_from_counts(&counts);
    assert!(optimized.max_num_bits <= MAX_HUFFMAN_BITS as u8);
    assert!(baseline.max_num_bits <= MAX_HUFFMAN_BITS as u8);
    assert_prefix_free(&optimized.codes);
}
