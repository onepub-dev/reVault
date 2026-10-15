use alloc::vec::Vec;

use crate::decoding::FrameDecoder;
use crate::encoding::frame_compressor::{CompressState, FseTables, OffsetHistory};
use crate::encoding::{compress_to_vec, CompressionLevel, MatchGeneratorDriver};

#[test]
fn fastest_does_not_expand_incompressible_blocks_past_raw_size() {
    assert_fastest_does_not_exceed_raw(8 * 1024);
}

#[test]
fn fastest_does_not_expand_incompressible_max_size_blocks() {
    assert_fastest_does_not_exceed_raw(128 * 1024);
}

#[test]
fn fastest_reuses_history_across_blocks() {
    let phrase = b"the quick brown fox jumps over the lazy dog\n";
    let mut data = Vec::with_capacity(512 * 1024);
    while data.len() < 512 * 1024 {
        data.extend_from_slice(phrase);
    }
    data.truncate(512 * 1024);

    let fastest = compress_to_vec(data.as_slice(), CompressionLevel::Fastest);
    let mut decoded = Vec::with_capacity(data.len());
    FrameDecoder::new()
        .decode_all_to_vec(fastest.as_slice(), &mut decoded)
        .unwrap();

    assert_eq!(decoded, data);

    let mut decoded_by_c = Vec::new();
    zstd::stream::copy_decode(fastest.as_slice(), &mut decoded_by_c).unwrap();
    assert_eq!(decoded_by_c, data);

    assert!(
        fastest.len() < 1024,
        "fastest should reuse previous blocks for repetitive data: got {} bytes",
        fastest.len()
    );
}

#[test]
fn incompressible_gate_distinguishes_random_from_repetitive_data() {
    let random = xorshift(128 * 1024);
    assert!(crate::encoding::util::likely_incompressible(&random));

    let mut repetitive = Vec::with_capacity(128 * 1024);
    while repetitive.len() < 128 * 1024 {
        repetitive.extend_from_slice(b"tenant=alpha path=/v1/archive status=200\n");
    }
    repetitive.truncate(128 * 1024);
    assert!(!crate::encoding::util::likely_incompressible(&repetitive));
}

#[test]
fn fastest_emits_whole_block_rle_and_round_trips() {
    let data = alloc::vec![0x5A; 128 * 1024];
    let fastest = assert_fastest_round_trips_with_rust_and_c(&data);
    let (_, frame_header_size) = crate::decoding::frame::read_frame_header(fastest.as_slice())
        .expect("fastest frame header should parse");
    let mut block_decoder = crate::decoding::block_decoder::new();
    let (block_header, block_header_size) = block_decoder
        .read_block_header(&fastest[frame_header_size as usize..])
        .expect("fastest block header should parse");

    assert!(block_header.last_block);
    assert_eq!(
        block_header.block_type,
        crate::blocks::block::BlockType::RLE
    );
    assert_eq!(block_header.decompressed_size, data.len() as u32);
    assert_eq!(block_header.content_size, 1);
    assert_eq!(
        fastest[frame_header_size as usize + block_header_size as usize],
        data[0]
    );
}

#[test]
fn fastest_empty_block_emits_raw_without_panic() {
    let mut state = CompressState {
        matcher: MatchGeneratorDriver::new(128 * 1024, 4),
        last_huff_table: None,
        fse_tables: FseTables::new(),
        offset_history: OffsetHistory::new(),
        file_type_hint: crate::encoding::CompressionFileType::Unknown,
        file_profile_hint: crate::encoding::CompressionFileProfile::None,
    };
    let mut output = Vec::new();

    super::fastest::compress_fastest(&mut state, true, Vec::new(), &mut output);

    let mut block_decoder = crate::decoding::block_decoder::new();
    let (block_header, block_header_size) = block_decoder
        .read_block_header(output.as_slice())
        .expect("empty block header should parse");

    assert_eq!(usize::from(block_header_size), output.len());
    assert!(block_header.last_block);
    assert_eq!(
        block_header.block_type,
        crate::blocks::block::BlockType::Raw
    );
    assert_eq!(block_header.content_size, 0);
}

#[test]
fn fastest_round_trips_mixed_text_binary_and_random_blocks() {
    let mut data = Vec::new();
    extend_repeated_to_len(
        &mut data,
        b"tenant=alpha path=/v1/archive status=200 bytes=4812\n",
        128 * 1024,
    );
    data.extend_from_slice(&xorshift(128 * 1024));
    extend_repeated_to_len(
        &mut data,
        b"\x00\x01\x02\x03\x04binary-record\x00\x01\x02\x03\x04payload\n",
        128 * 1024,
    );

    let fastest = assert_fastest_round_trips_with_rust_and_c(&data);
    assert!(
        fastest.len() < data.len(),
        "mixed frame should still be smaller than raw data: {} >= {}",
        fastest.len(),
        data.len()
    );
}

#[test]
fn fastest_reuses_repetitive_history_after_incompressible_block() {
    let mut repeated = Vec::new();
    extend_repeated_to_len(
        &mut repeated,
        b"user=123 action=checkout region=apac total=19.95\n",
        128 * 1024,
    );

    let mut data = repeated.clone();
    data.extend_from_slice(&xorshift(128 * 1024));
    data.extend_from_slice(&repeated);

    let fastest = assert_fastest_round_trips_with_rust_and_c(&data);
    assert!(
        fastest.len() < data.len() / 2,
        "repeated blocks around an incompressible block should compress well: {} bytes",
        fastest.len()
    );
}

#[test]
fn implemented_compression_levels_round_trip_with_rust_and_c_decoders() {
    let mut data = Vec::new();
    extend_repeated_to_len(
        &mut data,
        b"tenant=alpha path=/v1/archive status=200 bytes=4812\n",
        64 * 1024,
    );
    data.extend_from_slice(&xorshift(64 * 1024));
    extend_repeated_to_len(
        &mut data,
        b"zstd-rs higher level round trip fixture\n",
        64 * 1024,
    );

    for level in [
        CompressionLevel::Fastest,
        CompressionLevel::Default,
        CompressionLevel::Better,
        CompressionLevel::Best,
    ] {
        let compressed = compress_to_vec(data.as_slice(), level);

        let mut decoded = Vec::with_capacity(data.len());
        FrameDecoder::new()
            .decode_all_to_vec(compressed.as_slice(), &mut decoded)
            .unwrap();
        assert_eq!(
            decoded, data,
            "{level:?} should round-trip with Rust decoder"
        );

        let mut decoded_by_c = Vec::new();
        zstd::stream::copy_decode(compressed.as_slice(), &mut decoded_by_c).unwrap();
        assert_eq!(
            decoded_by_c, data,
            "{level:?} should round-trip with C zstd decoder"
        );
    }
}

#[cfg(feature = "std")]
#[test]
#[ignore]
fn best_level_external_fixture_round_trips_with_rust_and_c_decoders_from_env() {
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Write;

    let fixture =
        std::env::var("RUZSTD_BEST_FIXTURE").expect("set RUZSTD_BEST_FIXTURE to a fixture path");
    let data = fs::read(&fixture).expect("fixture must be readable");
    let by_slice = compress_to_vec(data.as_slice(), CompressionLevel::Best);
    assert_round_trips_with_rust_and_c(&by_slice, &data, &fixture, "slice source");

    let mut by_reader = Vec::new();
    crate::encoding::compress(
        BufReader::new(File::open(&fixture).expect("fixture must reopen")),
        &mut by_reader,
        CompressionLevel::Best,
    );
    assert_round_trips_with_rust_and_c(&by_reader, &data, &fixture, "reader source");

    let temp_output = std::env::temp_dir().join("ruzstd-best-external-fixture.zst");
    let mut output_file = File::create(&temp_output).expect("temp output must be creatable");
    crate::encoding::compress(
        BufReader::new(File::open(&fixture).expect("fixture must reopen")),
        &mut output_file,
        CompressionLevel::Best,
    );
    output_file.flush().expect("temp output must flush");
    drop(output_file);
    let by_file = fs::read(&temp_output).expect("temp output must be readable");
    let _ = fs::remove_file(&temp_output);
    assert_round_trips_with_rust_and_c(&by_file, &data, &fixture, "file drain");
}

#[test]
fn raw_fallback_restores_matcher_repeat_offsets() {
    let previous_offsets = OffsetHistory::from_offsets(7, 11, 13);
    let mut state = CompressState {
        matcher: MatchGeneratorDriver::new(128 * 1024, 4),
        last_huff_table: None,
        fse_tables: FseTables::new(),
        offset_history: previous_offsets,
        file_type_hint: crate::encoding::CompressionFileType::Unknown,
        file_profile_hint: crate::encoding::CompressionFileProfile::None,
    };

    let mut output = Vec::new();
    super::fastest::compress_fastest(&mut state, true, b"abcdeabcde".to_vec(), &mut output);

    assert_eq!(state.offset_history, previous_offsets);
    assert_eq!(
        state.matcher.repeat_offsets(),
        previous_offsets.as_offsets()
    );
}

fn assert_fastest_round_trips_with_rust_and_c(data: &[u8]) -> Vec<u8> {
    let fastest = compress_to_vec(data, CompressionLevel::Fastest);

    let mut decoded = Vec::with_capacity(data.len());
    FrameDecoder::new()
        .decode_all_to_vec(fastest.as_slice(), &mut decoded)
        .unwrap();
    assert_eq!(decoded, data);

    let mut decoded_by_c = Vec::new();
    zstd::stream::copy_decode(fastest.as_slice(), &mut decoded_by_c).unwrap();
    assert_eq!(decoded_by_c, data);

    fastest
}

#[cfg(feature = "std")]
fn assert_round_trips_with_rust_and_c(
    compressed: &[u8],
    expected: &[u8],
    fixture: &str,
    mode: &str,
) {
    let mut decoded = Vec::with_capacity(expected.len());
    FrameDecoder::new()
        .decode_all_to_vec(compressed, &mut decoded)
        .unwrap_or_else(|err| panic!("{} Rust decode failed for {}: {:?}", mode, fixture, err));
    assert_eq!(
        decoded, expected,
        "{mode} Rust decoder mismatch for {fixture}"
    );

    let mut decoded_by_c = Vec::new();
    zstd::stream::copy_decode(compressed, &mut decoded_by_c)
        .unwrap_or_else(|err| panic!("{} C decode failed for {}: {:?}", mode, fixture, err));
    assert_eq!(
        decoded_by_c, expected,
        "{mode} C decoder mismatch for {fixture}"
    );
}

fn assert_fastest_does_not_exceed_raw(len: usize) {
    let data = xorshift(len);
    let raw = compress_to_vec(data.as_slice(), CompressionLevel::Uncompressed);
    let fastest = compress_to_vec(data.as_slice(), CompressionLevel::Fastest);

    assert!(
        fastest.len() <= raw.len(),
        "fastest output should not exceed raw frame size for {len} bytes: {} > {}",
        fastest.len(),
        raw.len()
    );
}

fn extend_repeated_to_len(data: &mut Vec<u8>, phrase: &[u8], len: usize) {
    let target_len = data.len() + len;
    while data.len() < target_len {
        let remaining = target_len - data.len();
        let take = remaining.min(phrase.len());
        data.extend_from_slice(&phrase[..take]);
    }
}

fn xorshift(len: usize) -> Vec<u8> {
    let mut state = 0x1234_5678_9ABC_DEF0u64;
    let mut data = Vec::with_capacity(len);
    while data.len() < len {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        data.extend_from_slice(&state.to_le_bytes());
    }
    data.truncate(len);
    data
}
