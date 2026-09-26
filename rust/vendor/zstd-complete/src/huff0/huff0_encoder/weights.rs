use alloc::vec::Vec;

use crate::{
    bit_io::BitWriter,
    fse::fse_encoder::{self, FSEEncoder, FSETableBuildScratch},
};

use super::MAX_HUFFMAN_BITS;

pub(super) fn encoded_weight_table_bytes(weights: &[u8]) -> Vec<u8> {
    let fse_unusable = c_huff_weight_fse_is_unusable(weights);
    if raw_weight_table_is_supported(weights) && fse_unusable {
        return raw_weight_table_bytes(weights);
    }

    let encoded = encode_weight_table_fse_bytes(weights, fse_unusable);
    let compressed_size = encoded.len().saturating_sub(1);
    if compressed_size > 1 && compressed_size < weights.len() / 2 {
        encoded
    } else if raw_weight_table_is_supported(weights) {
        raw_weight_table_bytes(weights)
    } else {
        encoded
    }
}

pub(super) fn raw_weight_table_is_supported(weights: &[u8]) -> bool {
    weights.len() <= 128
}

pub(super) fn c_huff_weight_fse_is_unusable(weights: &[u8]) -> bool {
    if weights.len() <= 1 {
        return true;
    }

    let mut counts = [0usize; MAX_HUFFMAN_BITS + 1];
    let mut max_count = 0usize;
    for &weight in weights {
        let count = &mut counts[usize::from(weight)];
        *count += 1;
        max_count = max_count.max(*count);
    }

    max_count == weights.len() || max_count == 1
}

pub(super) fn table_description_bytes_from_weights(weights: &[u8]) -> Vec<u8> {
    let weights = &weights[..weights.len() - 1];
    if weights.len() > 16 {
        encoded_weight_table_bytes(weights)
    } else {
        small_weight_table_description_bytes(weights)
    }
}

pub(super) fn table_description_bytes_from_weights_reusing(weights: &[u8], output: &mut Vec<u8>) {
    let weights = &weights[..weights.len() - 1];
    if weights.len() <= 16 {
        output.clear();
        output.reserve(1 + weights.len().div_ceil(2));
        output.push(weights.len() as u8 + 127);
        let pairs = weights.chunks_exact(2);
        let remainder = pairs.remainder();
        for pair in pairs {
            output.push((pair[0] << 4) | pair[1]);
        }
        if let Some(&weight) = remainder.first() {
            output.push(weight << 4);
        }
        return;
    }

    let fse_unusable = c_huff_weight_fse_is_unusable(weights);
    if raw_weight_table_is_supported(weights) && fse_unusable {
        write_raw_weight_table(weights, output);
        return;
    }

    encode_weight_table_fse_reusing(weights, fse_unusable, output);
    let compressed_size = output.len().saturating_sub(1);
    if (compressed_size <= 1 || compressed_size >= weights.len() / 2)
        && raw_weight_table_is_supported(weights)
    {
        write_raw_weight_table(weights, output);
    }
}

pub(super) fn table_description_bytes_from_weights_reusing_with_fse_scratch(
    weights: &[u8],
    output: &mut Vec<u8>,
    fse_scratch: &mut FSETableBuildScratch,
) {
    let weights = &weights[..weights.len() - 1];
    if weights.len() <= 16 {
        output.clear();
        output.reserve(1 + weights.len().div_ceil(2));
        output.push(weights.len() as u8 + 127);
        let pairs = weights.chunks_exact(2);
        let remainder = pairs.remainder();
        for pair in pairs {
            output.push((pair[0] << 4) | pair[1]);
        }
        if let Some(&weight) = remainder.first() {
            output.push(weight << 4);
        }
        return;
    }

    let fse_unusable = c_huff_weight_fse_is_unusable(weights);
    if raw_weight_table_is_supported(weights) && fse_unusable {
        write_raw_weight_table(weights, output);
        return;
    }

    encode_weight_table_fse_reusing_with_scratch(weights, fse_unusable, output, fse_scratch);
    let compressed_size = output.len().saturating_sub(1);
    if (compressed_size <= 1 || compressed_size >= weights.len() / 2)
        && raw_weight_table_is_supported(weights)
    {
        write_raw_weight_table(weights, output);
    }
}

fn write_raw_weight_table(weights: &[u8], output: &mut Vec<u8>) {
    output.clear();
    output.reserve(1 + weights.len().div_ceil(2));
    output.push(weights.len() as u8 + 127);
    let pairs = weights.chunks_exact(2);
    let remainder = pairs.remainder();
    for pair in pairs {
        output.push((pair[0] << 4) | pair[1]);
    }
    if let Some(&weight) = remainder.first() {
        output.push(weight << 4);
    }
}

fn encode_weight_table_fse_reusing(weights: &[u8], fse_unusable: bool, output: &mut Vec<u8>) {
    output.clear();
    let mut writer = BitWriter::from(output);
    writer.write_bits(0u8, 8);
    let size_idx = writer.index() - 8;
    let idx_before = writer.index();
    let max_symbol = weights.iter().copied().max().unwrap_or(0) as usize;
    let table_log = fse_encoder::optimal_table_log(6, weights.len(), max_symbol);
    let table = if !fse_unusable {
        fse_encoder::build_huffman_weight_table_from_data(weights, 6)
    } else {
        fse_encoder::build_table_from_data(weights.iter().copied(), table_log, true)
    };
    let mut encoder = FSEEncoder::new(table, &mut writer);
    encoder.encode_interleaved(weights);
    let encoded_len = (writer.index() - idx_before) / 8;
    assert!(encoded_len < 128);
    writer.change_bits(size_idx, encoded_len as u8, 8);
    writer.flush();
}

fn encode_weight_table_fse_reusing_with_scratch(
    weights: &[u8],
    fse_unusable: bool,
    output: &mut Vec<u8>,
    scratch: &mut FSETableBuildScratch,
) {
    output.clear();
    let mut writer = BitWriter::from(output);
    writer.write_bits(0u8, 8);
    let size_idx = writer.index() - 8;
    let idx_before = writer.index();
    let max_symbol = weights.iter().copied().max().unwrap_or(0) as usize;
    let table_log = fse_encoder::optimal_table_log(6, weights.len(), max_symbol);
    let table = if !fse_unusable {
        fse_encoder::build_huffman_weight_table_from_data_with_scratch(weights, 6, scratch)
    } else {
        fse_encoder::build_table_from_data_with_scratch(
            weights.iter().copied(),
            table_log,
            true,
            scratch,
        )
    };
    let mut encoder = FSEEncoder::new(table, &mut writer);
    encoder.encode_interleaved(weights);
    let table = encoder.into_table();
    scratch.recycle_table(table);
    let encoded_len = (writer.index() - idx_before) / 8;
    assert!(encoded_len < 128);
    writer.change_bits(size_idx, encoded_len as u8, 8);
    writer.flush();
}

fn small_weight_table_description_bytes(weights: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(1 + weights.len().div_ceil(2));
    encoded.push(weights.len() as u8 + 127);
    let pairs = weights.chunks_exact(2);
    let remainder = pairs.remainder();
    for pair in pairs {
        let weight1 = pair[0];
        let weight2 = pair[1];
        debug_assert!(weight1 < 16);
        debug_assert!(weight2 < 16);
        encoded.push((weight1 << 4) | weight2);
    }
    if let Some(&weight) = remainder.first() {
        debug_assert!(weight < 16);
        encoded.push(weight << 4);
    }
    encoded
}

pub(super) fn raw_weight_table_bytes(weights: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(1 + weights.len().div_ceil(2));
    encoded.push(weights.len() as u8 + 127);
    let pairs = weights.chunks_exact(2);
    let remainder = pairs.remainder();
    for pair in pairs {
        let weight1 = pair[0];
        let weight2 = pair[1];
        debug_assert!(weight1 < 16);
        debug_assert!(weight2 < 16);
        encoded.push((weight1 << 4) | weight2);
    }
    if let Some(&weight) = remainder.first() {
        debug_assert!(weight < 16);
        encoded.push(weight << 4);
    }
    encoded
}

pub(super) fn weights_from_codes(codes: &[(u32, u8)], max_num_bits: u8) -> Vec<u8> {
    // C uses a `HUF_TABLELOG_MAX + 1` workspace table. Covering every `u8`
    // value lets safe Rust prove the symbol-provided index is always in bounds.
    let mut bits_to_weight = [0u8; 256];
    for num_bits in 1..=max_num_bits {
        bits_to_weight[usize::from(num_bits)] = max_num_bits + 1 - num_bits;
    }
    codes
        .iter()
        .copied()
        .map(|(_, num_bits)| bits_to_weight[usize::from(num_bits)])
        .collect()
}

pub(super) fn encode_weight_table_fse_bytes(weights: &[u8], fse_unusable: bool) -> Vec<u8> {
    let mut encoded = Vec::new();
    let mut writer = BitWriter::from(&mut encoded);
    writer.write_bits(0u8, 8);
    let size_idx = writer.index() - 8;
    let idx_before = writer.index();
    let max_symbol = weights.iter().copied().max().unwrap_or(0) as usize;
    let table_log = fse_encoder::optimal_table_log(6, weights.len(), max_symbol);
    let table = if !fse_unusable {
        fse_encoder::build_huffman_weight_table_from_data(weights, 6)
    } else {
        fse_encoder::build_table_from_data(weights.iter().copied(), table_log, true)
    };
    let mut encoder = FSEEncoder::new(table, &mut writer);
    encoder.encode_interleaved(weights);
    let encoded_len = (writer.index() - idx_before) / 8;
    assert!(encoded_len < 128);
    writer.change_bits(size_idx, encoded_len as u8, 8);
    writer.flush();
    encoded
}

#[cfg(test)]
mod raw_weight_tests {
    use super::{raw_weight_table_bytes, write_raw_weight_table};
    use alloc::{vec, vec::Vec};

    #[test]
    fn raw_huffman_weights_preserve_symbol_order() {
        for count in 1..=128 {
            let weights: Vec<u8> = (0..count).map(|symbol| (symbol % 12) as u8).collect();
            let mut reused = vec![0xff; 80];
            write_raw_weight_table(&weights, &mut reused);
            for encoded in [raw_weight_table_bytes(&weights), reused] {
                assert_eq!(encoded[0] as usize, count + 127);
                assert_eq!(encoded.len(), 1 + count.div_ceil(2));
                for (symbol, &weight) in weights.iter().enumerate() {
                    let byte = encoded[1 + symbol / 2];
                    let actual = if symbol % 2 == 0 {
                        byte >> 4
                    } else {
                        byte & 15
                    };
                    assert_eq!(actual, weight, "count={count} symbol={symbol}");
                }
                if count % 2 == 1 {
                    assert_eq!(encoded[encoded.len() - 1] & 15, 0);
                }
            }
        }
    }

    #[test]
    fn raw_huffman_weights_round_trip_asymmetric_valid_tree() {
        // Two weight-four symbols (8 units each) and sixteen weight-one
        // symbols form a complete 32-unit tree. Unequal adjacent weights
        // expose nibble reversal; the last weight is implicit on the wire.
        let mut weights = [1u8; 18];
        weights[0] = 4;
        weights[2] = 4;
        let encoded = raw_weight_table_bytes(&weights[..weights.len() - 1]);
        let mut decoder = crate::huff0::HuffmanTable::new();
        assert_eq!(
            decoder.build_decoder(&encoded).unwrap() as usize,
            encoded.len()
        );
        assert_eq!(
            decoder.encoder_weights(),
            weights
                .iter()
                .map(|&weight| usize::from(weight))
                .collect::<Vec<_>>()
        );
    }
}
