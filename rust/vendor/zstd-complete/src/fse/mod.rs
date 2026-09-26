//! FSE, short for Finite State Entropy, is an encoding technique
//! that assigns shorter codes to symbols that appear more frequently in data,
//! and longer codes to less frequent symbols.
//!
//! FSE works by mutating a state and using that state to index into a table.
//!
//! Zstandard uses two different kinds of entropy encoding: FSE, and Huffman coding.
//! Huffman is used to compress literals,
//! while FSE is used for all other symbols (literal length code, match length code, offset code).
//!
//! <https://github.com/facebook/zstd/blob/dev/doc/zstd_compression_format.md#fse>
//!
//! <https://arxiv.org/pdf/1311.2540>

mod fse_decoder;

pub use fse_decoder::*;

pub mod fse_encoder;

#[test]
fn tables_equal() {
    let probs = &[0, 0, -1, 3, 2, 2, (1 << 6) - 8];
    let mut dec_table = FSETable::new(255);
    dec_table.build_from_probabilities(6, probs).unwrap();
    let enc_table = fse_encoder::build_table_from_probabilities(probs, 6);

    check_tables(&dec_table, &enc_table);

    let huff_weight_probs = &[0, 8, 13, 5, 3, 2, 1];
    let mut dec_table = FSETable::new(255);
    dec_table
        .build_from_probabilities(5, huff_weight_probs)
        .unwrap();
    let enc_table = fse_encoder::build_table_from_probabilities(huff_weight_probs, 5);

    check_tables(&dec_table, &enc_table);

    for enc_table in [
        fse_encoder::default_ll_table(),
        fse_encoder::default_ml_table(),
        fse_encoder::default_of_table(),
    ] {
        let mut dec_table = FSETable::new(255);
        dec_table
            .build_from_probabilities(enc_table.acc_log(), &enc_table.probabilities())
            .unwrap();
        check_tables(&dec_table, &enc_table);
    }
}

#[cfg(any(test, feature = "fuzz_exports"))]
fn check_tables(dec_table: &fse_decoder::FSETable, enc_table: &fse_encoder::FSETable) {
    let table_size = 1usize << enc_table.acc_log();
    let table_mask = table_size - 1;
    for symbol in 0..=u8::MAX {
        if !enc_table.can_encode_symbol(symbol) {
            continue;
        }
        for state in table_size..2 * table_size {
            let (bits, num_bits, next_state) = enc_table.encode_symbol(symbol, state as u32);
            let dec_state = dec_table.decode[next_state as usize & table_mask];
            let bit_mask = (1u32 << num_bits).wrapping_sub(1);
            assert_eq!(dec_state.symbol, symbol);
            assert_eq!(dec_state.num_bits, num_bits);
            assert_eq!(
                dec_state.base_line + (bits & bit_mask),
                state as u32 & table_mask as u32
            );
        }
        let start_state = enc_table.c_start_state_index(symbol) as usize & table_mask;
        assert_eq!(dec_table.decode[start_state].symbol, symbol);
    }
}

#[test]
fn roundtrip() {
    round_trip(&(0..64).collect::<alloc::vec::Vec<_>>());
    let mut data = alloc::vec![];
    data.extend(0..32);
    data.extend(0..32);
    data.extend(0..32);
    data.extend(0..32);
    data.extend(0..32);
    data.extend(20..32);
    data.extend(20..32);
    data.extend(0..32);
    data.extend(20..32);
    data.extend(100..255);
    data.extend(20..32);
    data.extend(20..32);
    round_trip(&data);

    #[cfg(feature = "std")]
    if std::fs::exists("fuzz/artifacts/fse").unwrap_or(false) {
        for file in std::fs::read_dir("fuzz/artifacts/fse").unwrap() {
            if file.as_ref().unwrap().file_type().unwrap().is_file() {
                let data = std::fs::read(file.unwrap().path()).unwrap();
                round_trip(&data);
            }
        }
    }
}

/// Only needed for testing.
///
/// Encodes the data with a table built from that data
/// Decodes the result again by first decoding the table and then the data
/// Asserts that the decoded data equals the input
#[cfg(any(test, feature = "fuzz_exports"))]
pub fn round_trip(data: &[u8]) {
    use crate::bit_io::{BitReaderReversed, BitWriter};
    use fse_encoder::FSEEncoder;

    if data.len() < 2 {
        return;
    }
    if data.iter().all(|x| *x == data[0]) {
        return;
    }
    if data.len() < 64 {
        return;
    }

    let mut writer = BitWriter::new();
    let mut encoder = FSEEncoder::new(
        fse_encoder::build_table_from_data(data.iter().copied(), 22, false),
        &mut writer,
    );
    let mut dec_table = FSETable::new(255);
    encoder.encode(data);
    let acc_log = encoder.acc_log();
    let enc_table = encoder.into_table();
    let encoded = writer.dump();

    let table_bytes = dec_table.build_decoder(&encoded, acc_log).unwrap();
    let encoded = &encoded[table_bytes..];
    let mut decoder = FSEDecoder::new(&dec_table);

    check_tables(&dec_table, &enc_table);

    let mut br = BitReaderReversed::new(encoded);
    let mut skipped_bits = 0;
    loop {
        let val = br.get_bits(1);
        skipped_bits += 1;
        if val == 1 || skipped_bits > 8 {
            break;
        }
    }
    if skipped_bits > 8 {
        //if more than 7 bits are 0, this is not the correct end of the bitstream. Either a bug or corrupted data
        panic!("Corrupted end marker");
    }
    decoder.init_state(&mut br).unwrap();
    let mut decoded = alloc::vec::Vec::new();

    for x in data {
        let w = decoder.decode_symbol();
        assert_eq!(w, *x);
        decoded.push(w);
        if decoded.len() < data.len() {
            decoder.update_state(&mut br);
        }
    }

    assert_eq!(&decoded, data);

    assert_eq!(br.bits_remaining(), 0);
}
