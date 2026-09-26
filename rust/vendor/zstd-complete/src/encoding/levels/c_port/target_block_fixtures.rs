use alloc::vec::Vec;

use super::greedy_block::GreedyPreparedBlock;
use super::sequence_store::RepeatOffsets;
use crate::{
    blocks::{
        block::BlockType,
        literals_section::{LiteralsSection, LiteralsSectionType},
        sequence_section::{CompressionModes, ModeType, SequencesHeader},
    },
    encoding::blocks::{PreparedBlock, PreparedSequence},
};

pub(super) fn huffman_friendly_sequence_block() -> (Vec<u8>, GreedyPreparedBlock) {
    let mut data = Vec::new();
    let mut literals = Vec::new();
    let mut sequences = Vec::new();

    for idx in 0..160 {
        let chunk = [
            b'a' + (idx % 4) as u8,
            b'a' + ((idx + 1) % 4) as u8,
            b'0' + (idx % 2) as u8,
            b'\n',
        ];
        literals.extend_from_slice(&chunk);
        data.extend_from_slice(&chunk);
        data.extend_from_slice(&chunk);
        sequences.push(PreparedSequence {
            ll: chunk.len() as u32,
            ml: chunk.len() as u32,
            raw_offset: chunk.len() as u32,
            encoded_offset_value: 0,
        });
    }

    (
        data,
        GreedyPreparedBlock {
            prepared: PreparedBlock {
                literals,
                sequences,
            },
            repeat_offsets: RepeatOffsets::new(),
        },
    )
}

pub(super) fn split_friendly_sequence_block() -> (Vec<u8>, GreedyPreparedBlock) {
    let mut data = Vec::new();
    let mut literals = Vec::new();
    let mut sequences = Vec::new();

    for idx in 0..240 {
        let mut chunk = [0u8; 16];
        for (byte_idx, byte) in chunk.iter_mut().enumerate() {
            *byte = b'A' + ((idx + byte_idx) % 26) as u8;
        }
        literals.extend_from_slice(&chunk);
        data.extend_from_slice(&chunk);
        data.extend_from_slice(&chunk);
        sequences.push(PreparedSequence {
            ll: chunk.len() as u32,
            ml: chunk.len() as u32,
            raw_offset: chunk.len() as u32,
            encoded_offset_value: 0,
        });
    }

    (
        data,
        GreedyPreparedBlock {
            prepared: PreparedBlock {
                literals,
                sequences,
            },
            repeat_offsets: RepeatOffsets::new(),
        },
    )
}

pub(super) fn high_entropy_literal_sequence_block() -> (Vec<u8>, GreedyPreparedBlock) {
    let mut data = Vec::new();
    let mut literals = Vec::new();
    let mut sequences = Vec::new();
    let mut state = 0xA511_E9B3_u32;

    for _ in 0..240 {
        let mut chunk = [0u8; 16];
        for byte in &mut chunk {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            *byte = state as u8;
        }
        literals.extend_from_slice(&chunk);
        data.extend_from_slice(&chunk);
        data.extend_from_slice(&chunk);
        sequences.push(PreparedSequence {
            ll: chunk.len() as u32,
            ml: chunk.len() as u32,
            raw_offset: chunk.len() as u32,
            encoded_offset_value: 0,
        });
    }

    (
        data,
        GreedyPreparedBlock {
            prepared: PreparedBlock {
                literals,
                sequences,
            },
            repeat_offsets: RepeatOffsets::new(),
        },
    )
}

pub(super) fn zero_literal_tail_sequence_block() -> (Vec<u8>, GreedyPreparedBlock) {
    let mut base = [0u8; 64];
    for (idx, byte) in base.iter_mut().enumerate() {
        *byte = b'a' + (idx % 4) as u8;
    }

    let mut data = Vec::from(base);
    let mut sequences = Vec::new();
    sequences.push(PreparedSequence {
        ll: base.len() as u32,
        ml: base.len() as u32,
        raw_offset: base.len() as u32,
        encoded_offset_value: 0,
    });
    data.extend_from_slice(&base);

    for idx in 0..7000 {
        let match_len = (idx % 8 + 3) as u32;
        sequences.push(PreparedSequence {
            ll: 0,
            ml: match_len,
            raw_offset: 1,
            encoded_offset_value: 0,
        });
        for _ in 0..match_len {
            data.push(data[data.len() - 1]);
        }
    }

    (
        data,
        GreedyPreparedBlock {
            prepared: PreparedBlock {
                literals: base.to_vec(),
                sequences,
            },
            repeat_offsets: RepeatOffsets::new(),
        },
    )
}

pub(super) fn raw_tail_sequence_block() -> (Vec<u8>, GreedyPreparedBlock) {
    let mut data = Vec::new();
    let mut literals = Vec::new();
    let mut sequences = Vec::new();

    for idx in 0..80 {
        let mut chunk = [0u8; 16];
        for (byte_idx, byte) in chunk.iter_mut().enumerate() {
            *byte = b'a' + ((idx + byte_idx) % 6) as u8;
        }
        literals.extend_from_slice(&chunk);
        data.extend_from_slice(&chunk);
        data.extend_from_slice(&chunk);
        sequences.push(PreparedSequence {
            ll: chunk.len() as u32,
            ml: chunk.len() as u32,
            raw_offset: chunk.len() as u32,
            encoded_offset_value: 0,
        });
    }

    let mut state = 0xC0FF_EE31_u32;
    for _ in 0..50_000 {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let byte = state as u8;
        literals.push(byte);
        data.push(byte);
    }

    (
        data,
        GreedyPreparedBlock {
            prepared: PreparedBlock {
                literals,
                sequences,
            },
            repeat_offsets: RepeatOffsets::new(),
        },
    )
}

pub(super) fn parse_block_header(bytes: &[u8]) -> (bool, BlockType, u32) {
    assert!(bytes.len() >= 3);
    let raw = u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16);
    let block_type = match (raw >> 1) & 0b11 {
        0 => BlockType::Raw,
        1 => BlockType::RLE,
        2 => BlockType::Compressed,
        _ => BlockType::Reserved,
    };
    (raw & 1 != 0, block_type, raw >> 3)
}

pub(super) fn parse_block_headers(bytes: &[u8]) -> Vec<(BlockType, usize, bool)> {
    let mut headers = Vec::new();
    let mut pos = 0usize;
    while pos < bytes.len() {
        let (last_block, block_type, block_size) = parse_block_header(&bytes[pos..]);
        headers.push((block_type, block_size as usize, last_block));
        pos += 3 + block_size as usize;
    }
    assert_eq!(pos, bytes.len());
    headers
}

pub(super) fn assert_repeats_rle_sequence_metadata_after_first_sub_block(encoded: &[u8]) {
    let sequence_modes = compressed_block_sequence_modes(encoded);
    assert!(sequence_modes.len() > 1);
    let first = sequence_modes[0];
    assert!(matches!(first.ll_mode(), ModeType::RLE));
    assert!(matches!(first.ml_mode(), ModeType::RLE));
    assert!(sequence_modes.iter().skip(1).any(|modes| {
        matches!(modes.ll_mode(), ModeType::Repeat)
            && matches!(modes.ml_mode(), ModeType::Repeat)
            && matches!(modes.of_mode(), ModeType::Repeat)
    }));
}

fn compressed_block_sequence_modes(encoded: &[u8]) -> Vec<CompressionModes> {
    let mut modes = Vec::new();
    let mut pos = 0usize;
    while pos < encoded.len() {
        let (last_block, block_type, block_size) = parse_block_header(&encoded[pos..]);
        let content_start = pos + 3;
        let content_end = content_start + block_size as usize;
        if matches!(block_type, BlockType::Compressed) {
            modes.push(sequence_modes(&encoded[content_start..content_end]));
        }
        pos = content_end;
        if last_block {
            break;
        }
    }
    assert_eq!(pos, encoded.len());
    modes
}

fn sequence_modes(content: &[u8]) -> CompressionModes {
    let mut literals = LiteralsSection::new();
    let literal_header_size = literals
        .parse_from_header(content)
        .expect("literal header should parse") as usize;
    let literal_payload_size = match literals.ls_type {
        LiteralsSectionType::Raw => literals.regenerated_size as usize,
        LiteralsSectionType::RLE => usize::from(literals.regenerated_size > 0),
        LiteralsSectionType::Compressed | LiteralsSectionType::Treeless => literals
            .compressed_size
            .expect("compressed literal payload size")
            as usize,
    };
    let sequence_start = literal_header_size + literal_payload_size;
    let mut sequence_header = SequencesHeader::new();
    sequence_header
        .parse_from_header(&content[sequence_start..])
        .expect("sequence header should parse");
    sequence_header
        .modes
        .expect("target sequence blocks should contain sequences")
}

pub(super) fn decode_blocks(encoded: &[u8]) -> Vec<u8> {
    let mut block_decoder = crate::decoding::block_decoder::new();
    let mut scratch = crate::decoding::scratch::DecoderScratch::new(128 * 1024);
    let mut decoded = Vec::new();
    let mut pos = 0usize;

    while pos < encoded.len() {
        let (header, header_size) = block_decoder
            .read_block_header(&encoded[pos..])
            .expect("block header should parse");
        let content_start = pos + header_size as usize;
        let content_end = content_start + header.content_size as usize;
        match header.block_type {
            BlockType::Compressed => {
                block_decoder
                    .decode_block_content(
                        &header,
                        &mut scratch,
                        &encoded[content_start..content_end],
                    )
                    .expect("compressed block should decode");
                decoded.extend(scratch.buffer.drain());
            }
            BlockType::Raw => decoded.extend_from_slice(&encoded[content_start..content_end]),
            other => panic!("unexpected block type {:?}", other),
        }
        pos = content_end;
    }

    decoded
}

pub(super) fn decode_compressed_block(encoded: &[u8]) -> Vec<u8> {
    let mut block_decoder = crate::decoding::block_decoder::new();
    let (header, header_size) = block_decoder
        .read_block_header(encoded)
        .expect("block header should parse");
    assert_eq!(header.block_type, BlockType::Compressed);
    assert_eq!(
        header.content_size as usize,
        encoded.len() - header_size as usize
    );

    let mut scratch = crate::decoding::scratch::DecoderScratch::new(128 * 1024);
    block_decoder
        .decode_block_content(&header, &mut scratch, &encoded[header_size as usize..])
        .expect("compressed block should decode");
    scratch.buffer.drain()
}
