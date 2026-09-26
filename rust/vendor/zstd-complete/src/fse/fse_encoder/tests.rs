use super::{
    build_huffman_weight_table_from_data, build_table_from_data, build_table_from_probabilities,
    default_ll_table, default_ml_table, default_of_table, ncount_cost_from_probabilities,
    normalize_counts, normalize_counts_with_table_log, normalize_u32_counts_with_table_log,
    optimal_table_log, optimal_table_log_u32, write_normalized_probabilities_bytes,
};

use crate::bit_io::BitWriter;
use alloc::{vec, vec::Vec};

#[test]
fn ncount_cost_matches_written_table_size() {
    fn assert_ncount_cost_matches_writer(probs: &[i32], acc_log: u8) {
        let table = build_table_from_probabilities(probs, acc_log);
        let mut bytes = Vec::new();
        let mut writer = BitWriter::from(&mut bytes);
        table.write_table(&mut writer);
        writer.flush();

        assert_eq!(
            ncount_cost_from_probabilities(probs, acc_log),
            bytes.len() * 8
        );
    }

    assert_ncount_cost_matches_writer(super::LL_DIST, 6);
    assert_ncount_cost_matches_writer(super::ML_DIST, 6);
    assert_ncount_cost_matches_writer(super::OF_DIST, 5);

    let cases = [
        &[128, 64, 32, 16, 8, 4, 2, 1][..],
        &[9, 0, 7, 0, 5, 3, 0, 2, 1][..],
        &[4000, 31, 29, 23, 17, 11, 7, 5, 3, 1][..],
    ];
    for counts in cases {
        let (probs, acc_log) = super::normalized_probabilities_from_counts(counts, 9, true);
        assert_ncount_cost_matches_writer(&probs, acc_log);
    }
}

#[test]
fn c_style_ncount_writer_matches_bit_writer_reference() {
    fn reference(probs: &[i32], acc_log: u8) -> Vec<u8> {
        let mut output = Vec::new();
        let mut writer = BitWriter::from(&mut output);
        writer.write_bits(acc_log - 5, 4);
        let mut probabilities = probs.iter().copied();
        let mut probability_counter = 0usize;
        let probability_sum = 1usize << acc_log;
        let mut pending_zero_probability = None;

        while probability_counter < probability_sum {
            let prob = pending_zero_probability.take().unwrap_or_else(|| {
                probabilities
                    .next()
                    .expect("normalized probabilities must fill the table")
            });
            let max_remaining_value = probability_sum - probability_counter + 1;
            let bits_to_write = max_remaining_value.ilog2() + 1;
            let low_threshold = ((1 << bits_to_write) - 1) - max_remaining_value;
            let mask = (1 << (bits_to_write - 1)) - 1;
            let value = (prob + 1) as u32;

            if value < low_threshold as u32 {
                writer.write_bits(value, bits_to_write as usize - 1);
            } else if value > mask {
                writer.write_bits(value + low_threshold as u32, bits_to_write as usize);
            } else {
                writer.write_bits(value, bits_to_write as usize);
            }

            probability_counter += prob.unsigned_abs() as usize;
            if prob == 0 {
                let mut zeros = 0u8;
                loop {
                    let next = probabilities
                        .next()
                        .expect("normalized probabilities must end non-zero");
                    if next != 0 {
                        pending_zero_probability = Some(next);
                        break;
                    }
                    zeros += 1;
                    if zeros == 3 {
                        writer.write_bits(3u8, 2);
                        zeros = 0;
                    }
                }
                writer.write_bits(zeros, 2);
            }
        }
        writer.write_bits(0u8, writer.misaligned());
        writer.dump();
        output
    }

    let mut cases = vec![
        (super::LL_DIST.to_vec(), 6),
        (super::ML_DIST.to_vec(), 6),
        (super::OF_DIST.to_vec(), 5),
    ];
    for counts in [
        &[128, 64, 32, 16, 8, 4, 2, 1][..],
        &[9, 0, 7, 0, 5, 3, 0, 2, 1][..],
        &[4000, 31, 29, 23, 17, 11, 7, 5, 3, 1][..],
    ] {
        cases.push(super::normalized_probabilities_from_counts(counts, 9, true));
    }

    for (probs, acc_log) in cases {
        let mut output = Vec::new();
        write_normalized_probabilities_bytes(probs.iter().copied(), acc_log, &mut output);
        assert_eq!(output, reference(&probs, acc_log));
    }
}

#[test]
fn default_tables_cache_their_accuracy_log() {
    for (table, expected_acc_log) in [
        (default_ll_table(), 6),
        (default_ml_table(), 6),
        (default_of_table(), 5),
    ] {
        assert_eq!(table.acc_log(), expected_acc_log);
        assert_eq!(table.state_table_len(), 1 << table.acc_log());
    }
}

#[test]
fn optimal_table_log_matches_c_fast_sequence_shape() {
    assert_eq!(optimal_table_log(9, 3000, 35), 9);
    assert_eq!(optimal_table_log(8, 3000, 24), 8);
    assert_eq!(optimal_table_log(9, 12, 35), 5);
}

#[test]
fn c_unsigned_sequence_normalization_matches_machine_word_reference() {
    for max_symbol in [0usize, 1, 5, 15, 31, 35, 52, 63] {
        for seed in 1u32..=97 {
            let mut state = seed;
            let mut counts_u32 = vec![0u32; max_symbol + 1];
            for (symbol, count) in counts_u32.iter_mut().enumerate() {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                *count = if symbol == max_symbol {
                    state % 257 + 1
                } else {
                    state % 257
                };
            }
            let counts_usize = counts_u32
                .iter()
                .copied()
                .map(|count| count as usize)
                .collect::<Vec<_>>();
            let total_u32 = counts_u32.iter().sum::<u32>();
            let total_usize = total_u32 as usize;

            for max_log in [5u8, 6, 8, 9] {
                let table_log_u32 = optimal_table_log_u32(max_log, total_u32, max_symbol as u32);
                let table_log_usize = optimal_table_log(max_log, total_usize, max_symbol);
                assert_eq!(table_log_u32, table_log_usize);

                for low_prob_count in [-1, 1] {
                    let actual = normalize_u32_counts_with_table_log(
                        &counts_u32,
                        total_u32,
                        table_log_u32,
                        max_log,
                        low_prob_count,
                        true,
                    );
                    let expected = normalize_counts_with_table_log(
                        &counts_usize,
                        total_usize,
                        table_log_usize,
                        max_log,
                        low_prob_count,
                        true,
                    );
                    assert_eq!(actual, expected, "max={max_symbol} seed={seed}");
                }
            }
        }
    }
}

#[test]
fn c_style_normalization_sums_to_table_size() {
    let counts = [0, 57, 104, 88, 42, 17, 9, 3, 1, 0, 23, 61];
    let table_log = optimal_table_log(9, counts.iter().sum(), counts.len() - 1);
    let normalized = normalize_counts(&counts, table_log, 1)
        .expect("normalization should represent the distribution");
    let total = normalized
        .iter()
        .map(|probability| probability.unsigned_abs() as usize)
        .sum::<usize>();

    assert_eq!(total, 1usize << table_log);
    for (count, probability) in counts.iter().zip(normalized) {
        assert_eq!(*count == 0, probability == 0);
    }
}

#[test]
fn huffman_weight_table_uses_c_low_probability_policy() {
    let mut weights = Vec::new();
    weights.extend(alloc::vec![1; 70]);
    weights.extend(alloc::vec![2; 96]);
    weights.extend(alloc::vec![3; 42]);
    weights.extend(alloc::vec![4; 26]);
    weights.extend(alloc::vec![5; 18]);
    weights.extend(alloc::vec![6; 3]);

    let table = build_huffman_weight_table_from_data(&weights, 6);
    let probabilities = (0..=6)
        .map(|symbol| table.normalized_probability(symbol))
        .collect::<Vec<_>>();

    assert_eq!(table.acc_log(), 5);
    assert_eq!(probabilities, [0, 8, 13, 5, 3, 2, 1]);
}

#[test]
fn sequence_table_builder_keeps_large_balanced_tables_precise() {
    let mut data = alloc::vec::Vec::new();
    for _ in 0..100 {
        data.extend(0u8..30);
    }

    let table = build_table_from_data(data.iter().copied(), 9, true);

    assert_eq!(table.acc_log(), 9);
}
