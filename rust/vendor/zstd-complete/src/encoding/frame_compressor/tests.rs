use alloc::vec;
use alloc::vec::Vec;

use super::{best_block_segment_lengths, FrameCompressor, OffsetHistory};
use crate::common::MAGIC_NUM;
use crate::decoding::FrameDecoder;

#[test]
fn frame_starts_with_magic_num() {
    let mock_data = [1_u8, 2, 3].as_slice();
    let mut output: Vec<u8> = Vec::new();
    let mut compressor = FrameCompressor::new(super::CompressionLevel::Uncompressed);
    compressor.set_source(mock_data);
    compressor.set_drain(&mut output);

    compressor.compress();
    assert!(output.starts_with(&MAGIC_NUM.to_le_bytes()));
}

#[test]
fn direct_repeat_history_update_matches_encoded_update_with_literals() {
    for offset in [1, 4, 8, 9] {
        let mut encoded = OffsetHistory::new();
        let mut direct = OffsetHistory::new();

        encoded.encode_offset_value(offset, 3);
        direct.update_after_match(offset, true);

        assert_eq!(direct, encoded);
    }
}

#[test]
fn direct_repeat_history_update_matches_encoded_update_without_literals() {
    for offset in [4, 8, 0, 9] {
        let mut encoded = OffsetHistory::new();
        let mut direct = OffsetHistory::new();

        encoded.encode_offset_value(offset, 0);
        direct.update_after_match(offset, false);

        assert_eq!(direct, encoded);
    }
}

#[test]
fn fused_c_offset_update_matches_resolved_history_transition() {
    for initial in [
        OffsetHistory::new(),
        OffsetHistory::from_offsets(11, 22, 33),
        OffsetHistory::from_offsets(101, 7, 53),
    ] {
        for offset_value in 1..=20 {
            for lit_len in [0, 1, 17] {
                let expected_offset = if offset_value > 3 {
                    offset_value - 3
                } else {
                    let rep_code = offset_value - 1 + u32::from(lit_len == 0);
                    match rep_code {
                        0 => initial.newest,
                        1 => initial.second,
                        2 => initial.third,
                        3 => initial.newest - 1,
                        _ => unreachable!(),
                    }
                };

                let mut separate = initial;
                separate.update_from_offset_value(offset_value, lit_len, expected_offset);
                let mut fused = initial;
                let actual_offset = fused.update_from_c_offset_value(offset_value, lit_len);

                assert_eq!(actual_offset, expected_offset);
                assert_eq!(fused, separate);
            }
        }
    }
}

#[test]
fn very_simple_raw_compress() {
    let mock_data = [1_u8, 2, 3].as_slice();
    let mut output: Vec<u8> = Vec::new();
    let mut compressor = FrameCompressor::new(super::CompressionLevel::Uncompressed);
    compressor.set_source(mock_data);
    compressor.set_drain(&mut output);

    compressor.compress();
}

#[test]
fn very_simple_compress() {
    let mut mock_data = vec![0; 1 << 17];
    mock_data.extend(vec![1; (1 << 17) - 1]);
    mock_data.extend(vec![2; (1 << 18) - 1]);
    mock_data.extend(vec![2; 1 << 17]);
    mock_data.extend(vec![3; (1 << 17) - 1]);
    let mut output: Vec<u8> = Vec::new();
    let mut compressor = FrameCompressor::new(super::CompressionLevel::Uncompressed);
    compressor.set_source(mock_data.as_slice());
    compressor.set_drain(&mut output);

    compressor.compress();

    let mut decoder = FrameDecoder::new();
    let mut decoded = Vec::with_capacity(mock_data.len());
    decoder.decode_all_to_vec(&output, &mut decoded).unwrap();
    assert_eq!(mock_data, decoded);

    let mut decoded = Vec::new();
    zstd::stream::copy_decode(output.as_slice(), &mut decoded).unwrap();
    assert_eq!(mock_data, decoded);
}

#[test]
fn rle_compress() {
    let mock_data = vec![0; 1 << 19];
    let mut output: Vec<u8> = Vec::new();
    let mut compressor = FrameCompressor::new(super::CompressionLevel::Uncompressed);
    compressor.set_source(mock_data.as_slice());
    compressor.set_drain(&mut output);

    compressor.compress();

    let mut decoder = FrameDecoder::new();
    let mut decoded = Vec::with_capacity(mock_data.len());
    decoder.decode_all_to_vec(&output, &mut decoded).unwrap();
    assert_eq!(mock_data, decoded);
}

#[test]
fn fastest_reused_compressor_handles_tiny_then_compressible_frame() {
    let mut compressor = FrameCompressor::new(super::CompressionLevel::Fastest);

    let tiny = b"a";
    let mut tiny_output = Vec::new();
    compressor.set_source(tiny.as_slice());
    compressor.set_drain(&mut tiny_output);
    compressor.compress();
    let tiny_frame = compressor.take_drain().expect("tiny frame drain is set");
    assert_decodes_with_rust_and_c(tiny_frame, tiny);

    let mut compressible = Vec::with_capacity(64 * 1024);
    while compressible.len() < 64 * 1024 {
        compressible.extend_from_slice(b"abcde-record-payload\n");
    }
    compressible.truncate(64 * 1024);

    let mut output = Vec::new();
    compressor.set_source(compressible.as_slice());
    compressor.set_drain(&mut output);
    compressor.compress();
    let output_frame = compressor.take_drain().expect("output drain is set");
    assert_decodes_with_rust_and_c(output_frame, &compressible);
    assert!(
        output_frame.len() < compressible.len() / 4,
        "reused fastest compressor should still compress repetitive data: {} bytes",
        output_frame.len()
    );
}

#[test]
fn exact_full_block_is_marked_last_without_empty_block() {
    let mock_data = vec![7; 128 * 1024];
    let mut output: Vec<u8> = Vec::new();
    let mut compressor = FrameCompressor::new(super::CompressionLevel::Uncompressed);
    compressor.set_source(mock_data.as_slice());
    compressor.set_drain(&mut output);

    compressor.compress();

    let (_, frame_header_size) = crate::decoding::frame::read_frame_header(output.as_slice())
        .expect("frame header should parse");
    let mut block_decoder = crate::decoding::block_decoder::new();
    let (block_header, _) = block_decoder
        .read_block_header(&output[frame_header_size as usize..])
        .expect("block header should parse");

    assert!(block_header.last_block);
    assert_eq!(
        block_header.block_type,
        crate::blocks::block::BlockType::Raw
    );
    assert_eq!(block_header.content_size, mock_data.len() as u32);

    let mut decoder = FrameDecoder::new();
    let mut decoded = Vec::with_capacity(mock_data.len());
    decoder.decode_all_to_vec(&output, &mut decoded).unwrap();
    assert_eq!(mock_data, decoded);
}

#[test]
fn full_block_lookahead_preserves_next_block_first_byte() {
    let mut mock_data = vec![3; 128 * 1024];
    mock_data.extend_from_slice(&[4, 5, 6]);
    let mut output: Vec<u8> = Vec::new();
    let mut compressor = FrameCompressor::new(super::CompressionLevel::Uncompressed);
    compressor.set_source(mock_data.as_slice());
    compressor.set_drain(&mut output);

    compressor.compress();

    let (_, frame_header_size) = crate::decoding::frame::read_frame_header(output.as_slice())
        .expect("frame header should parse");
    let first_block_start = frame_header_size as usize;
    let mut block_decoder = crate::decoding::block_decoder::new();
    let (first_header, first_header_size) = block_decoder
        .read_block_header(&output[first_block_start..])
        .expect("first block header should parse");
    let second_block_start =
        first_block_start + first_header_size as usize + first_header.content_size as usize;
    let mut block_decoder = crate::decoding::block_decoder::new();
    let (second_header, second_header_size) = block_decoder
        .read_block_header(&output[second_block_start..])
        .expect("second block header should parse");
    let second_body_start = second_block_start + second_header_size as usize;

    assert!(!first_header.last_block);
    assert!(second_header.last_block);
    assert_eq!(second_header.content_size, 3);
    assert_eq!(
        &output[second_body_start..second_body_start + 3],
        &[4, 5, 6]
    );

    let mut decoder = FrameDecoder::new();
    let mut decoded = Vec::with_capacity(mock_data.len());
    decoder.decode_all_to_vec(&output, &mut decoded).unwrap();
    assert_eq!(mock_data, decoded);
}

#[test]
fn best_keeps_fully_compressible_block_whole() {
    let mut data = Vec::with_capacity(128 * 1024);
    while data.len() < 128 * 1024 {
        data.extend_from_slice(b"tenant=alpha path=/v1/archive status=200 bytes=4812\n");
    }
    data.truncate(128 * 1024);

    let compressed =
        crate::encoding::compress_to_vec(data.as_slice(), super::CompressionLevel::Best);
    assert_decodes_with_rust_and_c(&compressed, &data);

    let block_headers = collect_block_headers(&compressed);
    assert_eq!(block_headers.len(), 1);
    assert!(block_headers[0].last_block);
}

#[test]
fn best_splits_mixed_block_and_round_trips() {
    let mut data = Vec::with_capacity(64 * 1024);
    while data.len() < 24 * 1024 {
        data.extend_from_slice(b"repeated-best-split-fixture-line\n");
    }
    data.truncate(24 * 1024);
    data.extend_from_slice(&xorshift(24 * 1024));
    while data.len() < 64 * 1024 {
        data.extend_from_slice(b"repeated-best-split-fixture-line\n");
    }
    data.truncate(64 * 1024);

    let compressed =
        crate::encoding::compress_to_vec(data.as_slice(), super::CompressionLevel::Best);
    assert_decodes_with_rust_and_c(&compressed, &data);

    let block_headers = collect_block_headers(&compressed);
    assert!(
        block_headers.len() > 1,
        "best level should split mixed blocks: got {} block(s)",
        block_headers.len()
    );
    assert!(block_headers
        .iter()
        .take(block_headers.len() - 1)
        .all(|header| !header.last_block));
    assert!(block_headers.last().expect("at least one block").last_block);
}

#[test]
fn best_presplit_marks_single_chunk_incompressible_runs() {
    let mut data = Vec::with_capacity(40 * 1024);
    while data.len() < 16 * 1024 {
        data.extend_from_slice(b"repeated-best-split-fixture-line\n");
    }
    data.truncate(16 * 1024);
    data.extend_from_slice(&xorshift(8 * 1024));
    while data.len() < 40 * 1024 {
        data.extend_from_slice(b"repeated-best-split-fixture-line\n");
    }
    data.truncate(40 * 1024);

    let segments = best_block_segment_lengths(&data);
    assert!(
        segments.len() > 1,
        "a single incompressible 8KiB chunk should be split"
    );
}

#[test]
fn best_keeps_fully_incompressible_block_unsplit() {
    let data = xorshift(128 * 1024);

    let compressed =
        crate::encoding::compress_to_vec(data.as_slice(), super::CompressionLevel::Best);
    assert_decodes_with_rust_and_c(&compressed, &data);

    let block_headers = collect_block_headers(&compressed);
    assert_eq!(block_headers.len(), 1);
    assert!(block_headers[0].last_block);
}

#[test]
fn aaa_compress() {
    let mock_data = vec![0, 1, 3, 4, 5];
    let mut output: Vec<u8> = Vec::new();
    let mut compressor = FrameCompressor::new(super::CompressionLevel::Uncompressed);
    compressor.set_source(mock_data.as_slice());
    compressor.set_drain(&mut output);

    compressor.compress();

    let mut decoder = FrameDecoder::new();
    let mut decoded = Vec::with_capacity(mock_data.len());
    decoder.decode_all_to_vec(&output, &mut decoded).unwrap();
    assert_eq!(mock_data, decoded);

    let mut decoded = Vec::new();
    zstd::stream::copy_decode(output.as_slice(), &mut decoded).unwrap();
    assert_eq!(mock_data, decoded);
}
#[cfg(feature = "hash")]
#[test]
fn checksum_two_frames_reused_compressor() {
    // Compress the same data twice using the same compressor and verify that:
    // 1. The checksum written in each frame matches what the decoder calculates.
    // 2. The hasher is correctly reset between frames (no cross-contamination).
    //    If the hasher were NOT reset, the second frame's calculated checksum
    //    would differ from the one stored in the frame data, causing assert_eq to fail.
    let data: Vec<u8> = (0u8..=255).cycle().take(1024).collect();

    let mut compressor = FrameCompressor::new(super::CompressionLevel::Uncompressed);

    // --- Frame 1 ---
    let mut compressed1 = Vec::new();
    compressor.set_source(data.as_slice());
    compressor.set_drain(&mut compressed1);
    compressor.compress();

    // --- Frame 2 (reuse the same compressor) ---
    let mut compressed2 = Vec::new();
    compressor.set_source(data.as_slice());
    compressor.set_drain(&mut compressed2);
    compressor.compress();

    fn decode_and_collect(compressed: &[u8]) -> (Vec<u8>, Option<u32>, Option<u32>) {
        let mut decoder = FrameDecoder::new();
        let mut source = compressed;
        decoder.reset(&mut source).unwrap();
        while !decoder.is_finished() {
            decoder
                .decode_blocks(&mut source, crate::decoding::BlockDecodingStrategy::All)
                .unwrap();
        }
        let mut decoded = Vec::new();
        decoder.collect_to_writer(&mut decoded).unwrap();
        (
            decoded,
            decoder.get_checksum_from_data(),
            decoder.get_calculated_checksum(),
        )
    }

    let (decoded1, chksum_from_data1, chksum_calculated1) = decode_and_collect(&compressed1);
    assert_eq!(decoded1, data, "frame 1: decoded data mismatch");
    assert_eq!(
        chksum_from_data1, chksum_calculated1,
        "frame 1: checksum mismatch"
    );

    let (decoded2, chksum_from_data2, chksum_calculated2) = decode_and_collect(&compressed2);
    assert_eq!(decoded2, data, "frame 2: decoded data mismatch");
    assert_eq!(
        chksum_from_data2, chksum_calculated2,
        "frame 2: checksum mismatch"
    );

    // Same data compressed twice must produce the same checksum.
    // If state leaked across frames, the second calculated checksum would differ.
    assert_eq!(
        chksum_from_data1, chksum_from_data2,
        "frame 1 and frame 2 should have the same checksum (same data, hash must reset per frame)"
    );
}

fn assert_decodes_with_rust_and_c(compressed: &[u8], expected: &[u8]) {
    let mut rust_decoded = Vec::with_capacity(expected.len());
    FrameDecoder::new()
        .decode_all_to_vec(compressed, &mut rust_decoded)
        .unwrap();
    assert_eq!(rust_decoded, expected);

    let mut c_decoded = Vec::new();
    zstd::stream::copy_decode(compressed, &mut c_decoded).unwrap();
    assert_eq!(c_decoded, expected);
}

fn collect_block_headers(frame: &[u8]) -> Vec<crate::blocks::block::BlockHeader> {
    let (_, frame_header_size) =
        crate::decoding::frame::read_frame_header(frame).expect("frame header should parse");
    let mut headers = Vec::new();
    let mut offset = frame_header_size as usize;

    loop {
        let mut block_decoder = crate::decoding::block_decoder::new();
        let (header, header_size) = block_decoder
            .read_block_header(&frame[offset..])
            .expect("block header should parse");
        offset += header_size as usize + header.content_size as usize;
        headers.push(header);
        if headers.last().expect("pushed header").last_block {
            break;
        }
    }

    headers
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

#[cfg(feature = "std")]
#[test]
fn fuzz_targets() {
    use std::io::Read;
    fn decode_ruzstd(data: &mut dyn std::io::Read) -> Vec<u8> {
        let mut decoder = crate::decoding::StreamingDecoder::new(data).unwrap();
        let mut result: Vec<u8> = Vec::new();
        decoder.read_to_end(&mut result).expect("Decoding failed");
        result
    }

    fn decode_ruzstd_writer(mut data: impl Read) -> Vec<u8> {
        let mut decoder = crate::decoding::FrameDecoder::new();
        decoder.reset(&mut data).unwrap();
        let mut result = vec![];
        while !decoder.is_finished() || decoder.can_collect() > 0 {
            decoder
                .decode_blocks(
                    &mut data,
                    crate::decoding::BlockDecodingStrategy::UptoBytes(1024 * 1024),
                )
                .unwrap();
            decoder.collect_to_writer(&mut result).unwrap();
        }
        result
    }

    fn encode_zstd(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        zstd::stream::encode_all(std::io::Cursor::new(data), 3)
    }

    fn encode_ruzstd_uncompressed(data: &mut dyn std::io::Read) -> Vec<u8> {
        let mut input = Vec::new();
        data.read_to_end(&mut input).unwrap();

        crate::encoding::compress_to_vec(
            input.as_slice(),
            crate::encoding::CompressionLevel::Uncompressed,
        )
    }

    fn encode_ruzstd_compressed(data: &mut dyn std::io::Read) -> Vec<u8> {
        let mut input = Vec::new();
        data.read_to_end(&mut input).unwrap();

        crate::encoding::compress_to_vec(
            input.as_slice(),
            crate::encoding::CompressionLevel::Fastest,
        )
    }

    fn decode_zstd(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let mut output = Vec::new();
        zstd::stream::copy_decode(data, &mut output)?;
        Ok(output)
    }
    if std::fs::exists("fuzz/artifacts/interop").unwrap_or(false) {
        for file in std::fs::read_dir("fuzz/artifacts/interop").unwrap() {
            if file.as_ref().unwrap().file_type().unwrap().is_file() {
                let data = std::fs::read(file.unwrap().path()).unwrap();
                let data = data.as_slice();
                // Decoding
                let compressed = encode_zstd(data).unwrap();
                let decoded = decode_ruzstd(&mut compressed.as_slice());
                let decoded2 = decode_ruzstd_writer(&mut compressed.as_slice());
                assert!(
                    decoded == data,
                    "Decoded data did not match the original input during decompression"
                );
                assert_eq!(
                    decoded2, data,
                    "Decoded data did not match the original input during decompression"
                );

                // Encoding
                // Uncompressed encoding
                let mut input = data;
                let compressed = encode_ruzstd_uncompressed(&mut input);
                let decoded = decode_zstd(&compressed).unwrap();
                assert_eq!(
                    decoded, data,
                    "Decoded data did not match the original input during compression"
                );
                // Compressed encoding
                let mut input = data;
                let compressed = encode_ruzstd_compressed(&mut input);
                let decoded = decode_zstd(&compressed).unwrap();
                assert_eq!(
                    decoded, data,
                    "Decoded data did not match the original input during compression"
                );
            }
        }
    }
}
