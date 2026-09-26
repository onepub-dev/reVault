use super::*;
use crate::blocks::{
    literals_section::{LiteralsSection, LiteralsSectionType},
    sequence_section::{ModeType, SequencesHeader},
};
use crate::encoding::{
    block_header::BlockHeader,
    frame_compressor::{FseTables, OffsetHistory},
};
use crate::fse::fse_encoder::SharedFSETable;
use alloc::vec::Vec;

fn sequence(ll: u32, ml: u32) -> PreparedSequence {
    PreparedSequence {
        ll,
        ml,
        raw_offset: 1,
        encoded_offset_value: 0,
    }
}

fn decode_literals(encoded: &[u8]) -> Vec<u8> {
    let mut section = crate::blocks::literals_section::LiteralsSection::new();
    let header_size = section
        .parse_from_header(encoded)
        .expect("literal header should parse");
    let mut scratch = crate::decoding::scratch::HuffmanScratch::new();
    let mut decoded = Vec::new();
    let bytes_read = crate::decoding::literals_section_decoder::decode_literals(
        &section,
        &mut scratch,
        &encoded[header_size as usize..],
        &mut decoded,
    )
    .expect("literal payload should decode");

    assert_eq!(header_size as usize + bytes_read as usize, encoded.len());
    decoded
}

fn decode_compressed_block(encoded: &[u8]) -> Vec<u8> {
    let mut block_decoder = crate::decoding::block_decoder::new();
    let (header, header_size) = block_decoder
        .read_block_header(encoded)
        .expect("block header should parse");
    assert_eq!(
        header.block_type,
        crate::blocks::block::BlockType::Compressed
    );
    assert_eq!(
        header.content_size as usize,
        encoded.len() - header_size as usize
    );

    let mut scratch = crate::decoding::scratch::DecoderScratch::new(128 * 1024);
    block_decoder
        .decode_block_content(&header, &mut scratch, &encoded[header_size as usize..])
        .expect("block content should decode");
    scratch.buffer.drain()
}

fn sequence_section_bytes(encoded: &[u8]) -> &[u8] {
    let mut block_decoder = crate::decoding::block_decoder::new();
    let (header, header_size) = block_decoder
        .read_block_header(encoded)
        .expect("block header should parse");
    assert_eq!(
        header.block_type,
        crate::blocks::block::BlockType::Compressed
    );

    let content = &encoded[header_size as usize..];
    let mut literals = LiteralsSection::new();
    let literals_header_size = literals
        .parse_from_header(content)
        .expect("literal header should parse") as usize;
    let literals_payload_size = match literals.ls_type {
        LiteralsSectionType::Raw => literals.regenerated_size as usize,
        LiteralsSectionType::RLE => usize::from(literals.regenerated_size > 0),
        LiteralsSectionType::Compressed | LiteralsSectionType::Treeless => literals
            .compressed_size
            .expect("compressed literal payload size")
            as usize,
    };
    &content[literals_header_size + literals_payload_size..]
}

#[test]
fn target_sub_block_count_clamps_target_and_rounds_like_c() {
    assert_eq!(target_sub_block_count(0, 0), 1);
    assert_eq!(target_sub_block_count(1_000, 0), 1);
    assert_eq!(target_sub_block_count(2_010, 0), 2);
    assert_eq!(target_sub_block_count(6_700, 1_340), 5);
    assert_eq!(target_sub_block_count(6_701, 1_340), 5);
    assert_eq!(target_sub_block_count(7_371, 1_340), 6);
}

#[test]
fn sub_block_budget_plan_matches_c_quick_estimation_formula() {
    let plan = sub_block_budget_plan(
        EstimatedSubBlockSize {
            literal_size: 300,
            block_size: 2_010,
        },
        100,
        30,
        1_340,
        8_000,
    )
    .expect("estimated superblock is compressible");

    assert_eq!(
        plan,
        SubBlockBudgetPlan {
            avg_lit_cost: 768,
            avg_seq_cost: 14_592,
            nb_sub_blocks: 2,
            avg_block_budget: 257_280,
        }
    );
}

#[test]
fn sub_block_budget_plan_uses_one_byte_per_literal_when_no_literals() {
    let plan = sub_block_budget_plan(
        EstimatedSubBlockSize {
            literal_size: 0,
            block_size: 1_340,
        },
        0,
        10,
        1_340,
        4_096,
    )
    .expect("estimated superblock is compressible");

    assert_eq!(plan.avg_lit_cost, BYTESCALE);
    assert_eq!(plan.avg_seq_cost, 34_304);
}

#[test]
fn sub_block_budget_plan_clamps_target_size_like_c() {
    let clamped = sub_block_budget_plan(
        EstimatedSubBlockSize {
            literal_size: 100,
            block_size: 2_010,
        },
        50,
        10,
        1,
        4_096,
    )
    .expect("estimated superblock is compressible");
    let explicit = sub_block_budget_plan(
        EstimatedSubBlockSize {
            literal_size: 100,
            block_size: 2_010,
        },
        50,
        10,
        TARGET_CBLOCK_SIZE_MIN,
        4_096,
    )
    .expect("estimated superblock is compressible");

    assert_eq!(clamped, explicit);
}

#[test]
fn sub_block_budget_plan_bails_out_when_estimate_exceeds_source_size() {
    assert_eq!(
        sub_block_budget_plan(
            EstimatedSubBlockSize {
                literal_size: 500,
                block_size: 4_097,
            },
            100,
            10,
            1_340,
            4_096,
        ),
        None
    );
}

#[test]
fn should_commit_sub_block_matches_c_compressibility_gate() {
    assert!(should_commit_sub_block(9, 10));
    assert!(!should_commit_sub_block(0, 10));
    assert!(!should_commit_sub_block(10, 10));
    assert!(!should_commit_sub_block(11, 10));
}

#[test]
fn target_superblock_acceptance_matches_c_minimum_gain_gate() {
    let src_size = 128 * 1024;
    let max_c_size = src_size - min_compression_gain(src_size, Strategy::Fast);

    assert!(should_accept_target_superblock(
        max_c_size + BLOCK_HEADER_SIZE - 1,
        src_size,
        Strategy::Fast
    ));
    assert!(!should_accept_target_superblock(
        max_c_size + BLOCK_HEADER_SIZE,
        src_size,
        Strategy::Fast
    ));
}

#[test]
fn target_superblock_acceptance_rejects_zero_like_c() {
    assert!(!should_accept_target_superblock(
        0,
        128 * 1024,
        Strategy::BtUltra2
    ));
}

#[test]
fn sub_block_literal_header_size_matches_c_thresholds_without_entropy() {
    assert_eq!(sub_block_literal_header_size(1023, false), 3);
    assert_eq!(sub_block_literal_header_size(1024, false), 4);
    assert_eq!(sub_block_literal_header_size((16 * 1024) - 1, false), 4);
    assert_eq!(sub_block_literal_header_size(16 * 1024, false), 5);
}

#[test]
fn sub_block_literal_header_size_reserves_entropy_header_guess_like_c() {
    assert_eq!(
        sub_block_literal_header_size(1024 - LITERAL_HEADER_ENTROPY_GUESS - 1, true),
        3
    );
    assert_eq!(
        sub_block_literal_header_size(1024 - LITERAL_HEADER_ENTROPY_GUESS, true),
        4
    );
    assert_eq!(
        sub_block_literal_header_size(16 * 1024 - LITERAL_HEADER_ENTROPY_GUESS - 1, true),
        4
    );
    assert_eq!(
        sub_block_literal_header_size(16 * 1024 - LITERAL_HEADER_ENTROPY_GUESS, true),
        5
    );
}

#[test]
fn append_sub_block_literals_basic_emits_raw_literals() {
    let literals = [7; 31];
    let mut encoded = Vec::new();

    let emission =
        append_sub_block_literals(&literals, EntropyTableMode::Basic, true, &mut encoded)
            .expect("basic literal mode is supported");

    assert_eq!(
        emission,
        SubBlockLiteralEmission {
            byte_size: encoded.len(),
            entropy_written: false,
        }
    );
    assert_eq!(encoded[0] & 0b11, 0);
    assert_eq!(decode_literals(&encoded), literals);
}

#[test]
fn append_sub_block_literals_reports_appended_byte_size() {
    let prefix_len = 2;
    let mut encoded = alloc::vec![0xAA, 0xBB];

    let emission = append_sub_block_literals(b"abc", EntropyTableMode::Basic, false, &mut encoded)
        .expect("basic literal mode is supported");

    assert_eq!(emission.byte_size, encoded.len() - prefix_len);
    assert_eq!(&encoded[..prefix_len], &[0xAA, 0xBB]);
    assert_eq!(decode_literals(&encoded[prefix_len..]), b"abc");
}

#[test]
fn append_sub_block_literals_rle_emits_rle_literals() {
    let literals = [9; 44];
    let mut encoded = Vec::new();

    let emission = append_sub_block_literals(&literals, EntropyTableMode::Rle, false, &mut encoded)
        .expect("rle literal mode is supported");

    assert_eq!(emission.byte_size, encoded.len());
    assert!(!emission.entropy_written);
    assert_eq!(encoded[0] & 0b11, 1);
    assert_eq!(decode_literals(&encoded), literals);
}

#[test]
fn append_sub_block_literals_empty_rle_falls_back_to_raw_like_c() {
    let mut encoded = Vec::new();

    let emission = append_sub_block_literals(&[], EntropyTableMode::Rle, true, &mut encoded)
        .expect("empty rle literal mode is supported");

    assert_eq!(emission.byte_size, encoded.len());
    assert!(!emission.entropy_written);
    assert_eq!(encoded[0] & 0b11, 0);
    assert!(decode_literals(&encoded).is_empty());
}

#[test]
fn append_sub_block_literals_defers_huffman_modes_until_tables_are_ported() {
    let mut encoded = Vec::new();

    assert_eq!(
        append_sub_block_literals(
            b"literals",
            EntropyTableMode::Compressed,
            true,
            &mut encoded
        ),
        None
    );
    assert!(encoded.is_empty());
    assert_eq!(
        append_sub_block_literals(b"literals", EntropyTableMode::Repeat, false, &mut encoded),
        None
    );
    assert!(encoded.is_empty());
}

#[test]
fn append_sub_block_sequences_empty_emits_zero_sequence_header() {
    let mut encoded = Vec::new();

    let emission = append_sub_block_sequences(&[], basic_sequence_modes(), true, &mut encoded)
        .expect("zero-sequence section is supported");

    assert_eq!(
        emission,
        SubBlockSequenceEmission {
            byte_size: 1,
            entropy_written: false,
        }
    );
    assert_eq!(encoded, [0]);
}

#[test]
fn append_supported_sub_block_sequences_with_tables_preserves_empty_entropy_flag() {
    let mut encoded = Vec::new();
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let emission = append_supported_sub_block_sequences_with_tables(
        &[],
        compressed_sequence_modes(),
        None,
        true,
        &mut fse_tables,
        &mut offset_history,
        &mut encoded,
    )
    .expect("zero-sequence section is supported");

    assert_eq!(
        emission,
        SubBlockSequenceEmission {
            byte_size: 1,
            entropy_written: false,
        }
    );
    assert_eq!(encoded, [0]);
    assert!(fse_tables.ll_previous.is_none());
    assert!(fse_tables.ml_previous.is_none());
    assert!(fse_tables.of_previous.is_none());
    assert_eq!(offset_history.as_offsets(), (1, 4, 8));
}

#[test]
fn append_sub_block_sequences_defers_non_empty_sequences_until_fse_tables_are_ported() {
    let mut encoded = alloc::vec![0xAA];

    assert_eq!(
        append_sub_block_sequences(
            &[sequence(1, 3)],
            basic_sequence_modes(),
            true,
            &mut encoded
        ),
        None
    );
    assert_eq!(encoded, [0xAA]);
}

#[test]
fn append_supported_sub_block_sequences_emits_decodable_predefined_sequences() {
    let literals = b"abc";
    let sequences = [
        PreparedSequence {
            ll: 3,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 0,
        },
        PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 0,
        },
        PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 0,
        },
        PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 0,
        },
    ];
    let mut encoded = alloc::vec![0; BLOCK_HEADER_SIZE];
    append_sub_block_literals(literals, EntropyTableMode::Basic, true, &mut encoded)
        .expect("basic literals");
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();
    let emission = append_supported_sub_block_sequences(
        &sequences,
        basic_sequence_modes(),
        true,
        &mut fse_tables,
        &mut offset_history,
        &mut encoded,
    )
    .expect("basic sequence modes should encode");
    let content_size = encoded.len() - BLOCK_HEADER_SIZE;
    let header = BlockHeader {
        last_block: true,
        block_type: crate::blocks::block::BlockType::Compressed,
        block_size: content_size as u32,
    };
    encoded[..BLOCK_HEADER_SIZE].copy_from_slice(&header.serialize_to_bytes());

    assert!(emission.byte_size >= 4);
    assert!(emission.entropy_written);
    assert_eq!(offset_history.as_offsets(), (3, 3, 1));
    assert_eq!(decode_compressed_block(&encoded), b"abcabcabcabcabc");
}

#[test]
fn append_supported_sub_block_sequences_defers_repeat_when_entropy_write_is_disabled() {
    let mut encoded = alloc::vec![0xAA];
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    assert_eq!(
        append_supported_sub_block_sequences(
            &[sequence(1, 3)],
            SequenceEntropyModes {
                ll: EntropyTableMode::Repeat,
                ml: EntropyTableMode::Repeat,
                of: EntropyTableMode::Repeat,
            },
            false,
            &mut fse_tables,
            &mut offset_history,
            &mut encoded,
        ),
        None
    );
    assert_eq!(encoded, [0xAA]);
    assert_eq!(offset_history.as_offsets(), (1, 4, 8));
}

#[test]
fn append_supported_sub_block_sequences_defers_repeat_without_previous_tables() {
    let mut encoded = alloc::vec![0xAA];
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    assert_eq!(
        append_supported_sub_block_sequences(
            &[sequence(1, 3)],
            repeat_sequence_modes(),
            true,
            &mut fse_tables,
            &mut offset_history,
            &mut encoded,
        ),
        None
    );
    assert_eq!(encoded, [0xAA]);
    assert_eq!(offset_history.as_offsets(), (1, 4, 8));
}

#[test]
fn append_literal_only_sub_block_builds_decodable_raw_literal_block() {
    let literals = b"literal-only-superblock";
    let mut encoded = Vec::new();

    let emission = append_literal_only_sub_block(
        literals,
        true,
        EntropyTableMode::Basic,
        basic_sequence_modes(),
        true,
        true,
        &mut encoded,
    )
    .expect("literal-only basic sub-block is supported");

    assert_eq!(
        emission,
        SubBlockEmission {
            byte_size: encoded.len(),
            literal_entropy_written: false,
            sequence_entropy_written: false,
        }
    );
    let mut block_decoder = crate::decoding::block_decoder::new();
    let (header, _) = block_decoder
        .read_block_header(encoded.as_slice())
        .expect("block header should parse");
    assert!(header.last_block);
    assert_eq!(decode_compressed_block(&encoded), literals);
}

#[test]
fn append_literal_only_sub_block_builds_decodable_rle_literal_block() {
    let literals = [0x5A; 64];
    let mut encoded = Vec::new();

    let emission = append_literal_only_sub_block(
        &literals,
        false,
        EntropyTableMode::Rle,
        basic_sequence_modes(),
        false,
        false,
        &mut encoded,
    )
    .expect("literal-only rle sub-block is supported");

    assert_eq!(emission.byte_size, encoded.len());
    let mut block_decoder = crate::decoding::block_decoder::new();
    let (header, _) = block_decoder
        .read_block_header(encoded.as_slice())
        .expect("block header should parse");
    assert!(!header.last_block);
    assert_eq!(decode_compressed_block(&encoded), literals);
}

#[test]
fn append_literal_only_sub_block_defers_unsupported_literal_modes() {
    let mut encoded = alloc::vec![0xAA];

    assert_eq!(
        append_literal_only_sub_block(
            b"abc",
            true,
            EntropyTableMode::Compressed,
            basic_sequence_modes(),
            true,
            true,
            &mut encoded,
        ),
        None
    );
    assert_eq!(encoded, [0xAA]);
}

#[test]
fn append_sequence_sub_block_builds_decodable_basic_sequence_block() {
    let literals = b"abc";
    let sequences = [
        PreparedSequence {
            ll: 3,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 0,
        },
        PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 0,
        },
        PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 0,
        },
        PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 0,
        },
    ];
    let mut encoded = Vec::new();
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let emission = append_sequence_sub_block(
        literals,
        &sequences,
        true,
        EntropyTableMode::Basic,
        basic_sequence_modes(),
        true,
        true,
        &mut fse_tables,
        &mut offset_history,
        &mut encoded,
    )
    .expect("basic sequence sub-block should encode");

    assert_eq!(emission.byte_size, encoded.len());
    assert!(!emission.literal_entropy_written);
    assert!(emission.sequence_entropy_written);
    assert_eq!(offset_history.as_offsets(), (3, 3, 1));
    assert_eq!(decode_compressed_block(&encoded), b"abcabcabcabcabc");
}

#[test]
fn append_sequence_sub_block_builds_decodable_rle_sequence_block() {
    let mut literals = Vec::new();
    let mut sequences = Vec::new();
    let mut expected = Vec::new();
    for idx in 0..24 {
        let chunk = [
            b'a' + (idx % 20) as u8,
            b'A' + (idx % 20) as u8,
            b'0' + (idx % 10) as u8,
        ];
        literals.extend_from_slice(&chunk);
        expected.extend_from_slice(&chunk);
        expected.extend_from_slice(&chunk);
        sequences.push(PreparedSequence {
            ll: 3,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 6,
        });
    }
    let mut encoded = Vec::new();
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let emission = append_sequence_sub_block(
        &literals,
        &sequences,
        true,
        EntropyTableMode::Basic,
        rle_sequence_modes(),
        true,
        true,
        &mut fse_tables,
        &mut offset_history,
        &mut encoded,
    )
    .expect("uniform sequence codes should encode with RLE metadata");

    assert_eq!(emission.byte_size, encoded.len());
    assert!(!emission.literal_entropy_written);
    assert!(emission.sequence_entropy_written);
    assert_eq!(offset_history.as_offsets(), (3, 3, 3));
    assert_eq!(decode_compressed_block(&encoded), expected);
}

#[test]
fn append_sequence_sub_block_emits_repeat_sequence_metadata() {
    let literals = b"abc";
    let sequences = [
        PreparedSequence {
            ll: 3,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 0,
        },
        PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 0,
        },
        PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 0,
        },
        PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 3,
            encoded_offset_value: 0,
        },
    ];
    let mut encoded = Vec::new();
    let mut fse_tables = FseTables::new();
    fse_tables.ll_previous = Some(SharedFSETable::new(fse_tables.ll_default.clone()));
    fse_tables.ml_previous = Some(SharedFSETable::new(fse_tables.ml_default.clone()));
    fse_tables.of_previous = Some(SharedFSETable::new(fse_tables.of_default.clone()));
    let mut offset_history = OffsetHistory::new();

    let emission = append_sequence_sub_block(
        literals,
        &sequences,
        true,
        EntropyTableMode::Basic,
        repeat_sequence_modes(),
        true,
        true,
        &mut fse_tables,
        &mut offset_history,
        &mut encoded,
    )
    .expect("repeat sequence metadata should encode when previous tables exist");

    assert_eq!(emission.byte_size, encoded.len());
    assert!(!emission.literal_entropy_written);
    assert!(emission.sequence_entropy_written);
    assert_eq!(offset_history.as_offsets(), (3, 3, 1));
    let sequence_section = sequence_section_bytes(&encoded);
    let mut sequence_header = SequencesHeader::new();
    sequence_header
        .parse_from_header(sequence_section)
        .expect("sequence header should parse");
    let modes = sequence_header.modes.expect("repeat section has sequences");
    assert!(matches!(modes.ll_mode(), ModeType::Repeat));
    assert!(matches!(modes.ml_mode(), ModeType::Repeat));
    assert!(matches!(modes.of_mode(), ModeType::Repeat));
}

#[test]
fn append_sequence_sub_block_builds_decodable_compressed_sequence_block() {
    let mut literals = Vec::new();
    let mut sequences = Vec::new();
    let mut expected = Vec::new();
    for idx in 0..36 {
        let lit_len = if idx == 0 { 5 } else { idx % 4 + 1 };
        let match_len = idx % 5 + 3;
        let raw_offset = idx % 3 + 3;
        let start = literals.len();
        for lit_idx in 0..lit_len {
            literals.push(b'a' + ((idx + lit_idx) % 26) as u8);
        }
        expected.extend_from_slice(&literals[start..]);
        for _ in 0..match_len {
            let byte = expected[expected.len() - raw_offset];
            expected.push(byte);
        }
        sequences.push(PreparedSequence {
            ll: lit_len as u32,
            ml: match_len as u32,
            raw_offset: raw_offset as u32,
            encoded_offset_value: raw_offset as u32 + 3,
        });
    }
    let mut encoded = Vec::new();
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let emission = append_sequence_sub_block(
        &literals,
        &sequences,
        true,
        EntropyTableMode::Basic,
        compressed_sequence_modes(),
        true,
        true,
        &mut fse_tables,
        &mut offset_history,
        &mut encoded,
    )
    .expect("varied sequence codes should encode with compressed FSE metadata");

    assert_eq!(emission.byte_size, encoded.len());
    assert!(!emission.literal_entropy_written);
    assert!(emission.sequence_entropy_written);
    assert!(fse_tables.ll_previous.is_some());
    assert!(fse_tables.ml_previous.is_some());
    assert!(fse_tables.of_previous.is_some());
    assert_eq!(decode_compressed_block(&encoded), expected);
}

#[test]
fn append_sequence_sub_block_builds_decodable_mixed_sequence_modes() {
    let mut literals = Vec::new();
    let mut sequences = Vec::new();
    let mut expected = Vec::new();
    for idx in 0..36 {
        let lit_len = 3usize;
        let match_len = idx % 5 + 3;
        let raw_offset = idx % 4 + 3;
        let start = literals.len();
        for lit_idx in 0..lit_len {
            literals.push(b'a' + ((idx + lit_idx) % 26) as u8);
        }
        expected.extend_from_slice(&literals[start..]);
        for _ in 0..match_len {
            let byte = expected[expected.len() - raw_offset];
            expected.push(byte);
        }
        sequences.push(PreparedSequence {
            ll: lit_len as u32,
            ml: match_len as u32,
            raw_offset: raw_offset as u32,
            encoded_offset_value: raw_offset as u32 + 3,
        });
    }
    let mut encoded = Vec::new();
    let mut fse_tables = FseTables::new();
    let mut offset_history = OffsetHistory::new();

    let emission = append_sequence_sub_block(
        &literals,
        &sequences,
        true,
        EntropyTableMode::Basic,
        SequenceEntropyModes {
            ll: EntropyTableMode::Rle,
            ml: EntropyTableMode::Basic,
            of: EntropyTableMode::Compressed,
        },
        true,
        true,
        &mut fse_tables,
        &mut offset_history,
        &mut encoded,
    )
    .expect("mixed sequence modes should encode");

    assert_eq!(emission.byte_size, encoded.len());
    assert!(!emission.literal_entropy_written);
    assert!(emission.sequence_entropy_written);
    assert!(fse_tables.ll_previous.is_none());
    assert!(fse_tables.ml_previous.is_none());
    assert!(fse_tables.of_previous.is_some());
    assert_eq!(decode_compressed_block(&encoded), expected);

    let sequence_section = sequence_section_bytes(&encoded);
    let mut sequence_header = SequencesHeader::new();
    sequence_header
        .parse_from_header(sequence_section)
        .expect("sequence header should parse");
    let modes = sequence_header.modes.expect("mixed section has sequences");
    assert!(matches!(modes.ll_mode(), ModeType::RLE));
    assert!(matches!(modes.ml_mode(), ModeType::Predefined));
    assert!(matches!(modes.of_mode(), ModeType::FSECompressed));
}

#[test]
fn need_sequence_entropy_tables_matches_c_metadata_gate() {
    let no_tables = SequenceEntropyModes {
        ll: EntropyTableMode::Basic,
        ml: EntropyTableMode::Repeat,
        of: EntropyTableMode::Basic,
    };
    let rle_tables = SequenceEntropyModes {
        ll: EntropyTableMode::Basic,
        ml: EntropyTableMode::Rle,
        of: EntropyTableMode::Basic,
    };
    let compressed_tables = SequenceEntropyModes {
        ll: EntropyTableMode::Repeat,
        ml: EntropyTableMode::Basic,
        of: EntropyTableMode::Compressed,
    };

    assert!(!need_sequence_entropy_tables(no_tables));
    assert!(need_sequence_entropy_tables(rle_tables));
    assert!(need_sequence_entropy_tables(compressed_tables));
}

#[test]
fn sequence_entropy_estimate_uses_c_rle_header_cost() {
    let sequences = [
        PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 4,
            encoded_offset_value: 1,
        },
        PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 1,
            encoded_offset_value: 1,
        },
    ];
    let modes = rle_sequence_modes();
    let fse_tables = FseTables::new();

    assert_eq!(
        estimate_sequence_entropy_section_size(
            &sequences,
            modes,
            None,
            &fse_tables,
            OffsetHistory::new(),
            true,
        ),
        Some(6)
    );
}

#[test]
fn target_sequence_entropy_selection_uses_c_strategy_policy() {
    let sequences = [
        PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 4,
            encoded_offset_value: 1,
        },
        PreparedSequence {
            ll: 1,
            ml: 4,
            raw_offset: 1,
            encoded_offset_value: 2,
        },
        PreparedSequence {
            ll: 0,
            ml: 3,
            raw_offset: 4,
            encoded_offset_value: 1,
        },
        PreparedSequence {
            ll: 1,
            ml: 4,
            raw_offset: 1,
            encoded_offset_value: 2,
        },
    ];
    let mut fse_tables = FseTables::new();
    fse_tables.ll_previous = Some(SharedFSETable::new(fse_tables.ll_default.clone()));
    fse_tables.ll_repeat_valid = true;
    fse_tables.ml_previous = Some(SharedFSETable::new(fse_tables.ml_default.clone()));
    fse_tables.ml_repeat_valid = true;
    fse_tables.of_previous = Some(SharedFSETable::new(fse_tables.of_default.clone()));
    fse_tables.of_repeat_valid = true;

    let fast_modes = select_sequence_entropy_modes(
        &sequences,
        &fse_tables,
        OffsetHistory::new(),
        Strategy::Fast,
    );
    let optimal_modes = select_sequence_entropy_modes(
        &sequences,
        &fse_tables,
        OffsetHistory::new(),
        Strategy::BtOpt,
    );

    assert_eq!(
        fast_modes,
        SequenceEntropyModes {
            ll: EntropyTableMode::Repeat,
            ml: EntropyTableMode::Repeat,
            of: EntropyTableMode::Repeat,
        }
    );
    assert_eq!(
        optimal_modes,
        SequenceEntropyModes {
            ll: EntropyTableMode::Basic,
            ml: EntropyTableMode::Basic,
            of: EntropyTableMode::Basic,
        }
    );
}

fn basic_sequence_modes() -> SequenceEntropyModes {
    SequenceEntropyModes {
        ll: EntropyTableMode::Basic,
        ml: EntropyTableMode::Basic,
        of: EntropyTableMode::Basic,
    }
}

fn rle_sequence_modes() -> SequenceEntropyModes {
    SequenceEntropyModes {
        ll: EntropyTableMode::Rle,
        ml: EntropyTableMode::Rle,
        of: EntropyTableMode::Rle,
    }
}

fn repeat_sequence_modes() -> SequenceEntropyModes {
    SequenceEntropyModes {
        ll: EntropyTableMode::Repeat,
        ml: EntropyTableMode::Repeat,
        of: EntropyTableMode::Repeat,
    }
}

fn compressed_sequence_modes() -> SequenceEntropyModes {
    SequenceEntropyModes {
        ll: EntropyTableMode::Compressed,
        ml: EntropyTableMode::Compressed,
        of: EntropyTableMode::Compressed,
    }
}

#[test]
fn plan_sub_blocks_commits_compressible_tentative_block() {
    let sequences = [sequence(1, 200), sequence(2, 200), sequence(3, 200)];
    let plan = SubBlockBudgetPlan {
        avg_lit_cost: 0,
        avg_seq_cost: 100,
        nb_sub_blocks: 2,
        avg_block_budget: ENTROPY_HEADER_BUDGET + 150,
    };

    let mut outcomes = [3].iter().copied();
    let blocks = plan_sub_blocks(&sequences, 9, plan, |_| {
        outcomes.next().expect("one tentative block")
    });

    assert_eq!(
        blocks,
        alloc::vec![
            PlannedSubBlock {
                start_sequence: 0,
                end_sequence: 1,
                literal_size: 1,
                decompressed_size: 201,
                last: false,
            },
            PlannedSubBlock {
                start_sequence: 1,
                end_sequence: 3,
                literal_size: 8,
                decompressed_size: 408,
                last: true,
            },
        ]
    );
}

#[test]
fn plan_sub_blocks_coalesces_failed_tentative_block_like_c() {
    let sequences = [sequence(0, 10), sequence(0, 10), sequence(0, 10)];
    let plan = SubBlockBudgetPlan {
        avg_lit_cost: 0,
        avg_seq_cost: 100,
        nb_sub_blocks: 3,
        avg_block_budget: 250,
    };

    let mut outcomes = [0, 10].iter().copied();
    let blocks = plan_sub_blocks(&sequences, 0, plan, |_| {
        outcomes.next().expect("two tentative blocks")
    });

    assert_eq!(
        blocks,
        alloc::vec![
            PlannedSubBlock {
                start_sequence: 0,
                end_sequence: 2,
                literal_size: 0,
                decompressed_size: 20,
                last: false,
            },
            PlannedSubBlock {
                start_sequence: 2,
                end_sequence: 3,
                literal_size: 0,
                decompressed_size: 10,
                last: true,
            },
        ]
    );
}

#[test]
fn plan_sub_blocks_breaks_to_last_block_when_candidate_reaches_end() {
    let sequences = [sequence(1, 5), sequence(2, 7)];
    let plan = SubBlockBudgetPlan {
        avg_lit_cost: 0,
        avg_seq_cost: 1,
        nb_sub_blocks: 3,
        avg_block_budget: ENTROPY_HEADER_BUDGET + 100,
    };

    let blocks = plan_sub_blocks(&sequences, 5, plan, |_| {
        panic!("candidate reaches end and should not be compressed")
    });

    assert_eq!(
        blocks,
        alloc::vec![PlannedSubBlock {
            start_sequence: 0,
            end_sequence: 2,
            literal_size: 5,
            decompressed_size: 17,
            last: true,
        }]
    );
}

#[test]
fn count_literals_sums_sequence_literal_lengths_like_c() {
    let sequences = [sequence(2, 5), sequence(0, 3), sequence(7, 11)];

    assert_eq!(count_literals(&sequences), 9);
    assert_eq!(count_literals(&[]), 0);
}

#[test]
fn decompressed_size_adds_literal_size_and_match_lengths_like_c() {
    let sequences = [sequence(2, 5), sequence(0, 3), sequence(7, 11)];

    assert_eq!(decompressed_size(&sequences, 9, false), 28);
}

#[test]
fn decompressed_size_allows_last_sub_block_to_include_last_literals() {
    let sequences = [sequence(2, 5), sequence(0, 3), sequence(7, 11)];

    assert_eq!(decompressed_size(&sequences, 13, true), 32);
}

#[test]
fn size_block_sequences_returns_one_when_first_sequence_exceeds_budget() {
    let sequences = [sequence(1, 3), sequence(1, 3)];

    assert_eq!(size_block_sequences(&sequences, 100, 256, 256, false), 1);
}

#[test]
fn size_block_sequences_charges_first_sub_block_entropy_budget() {
    let sequences = [sequence(0, 3), sequence(0, 3)];

    assert_eq!(
        size_block_sequences(&sequences, ENTROPY_HEADER_BUDGET - 1, 0, 0, true),
        1
    );
}

#[test]
fn size_block_sequences_returns_previous_count_when_next_sequence_trips_budget() {
    let sequences = [sequence(1, 3), sequence(1, 3), sequence(1, 3)];

    assert_eq!(size_block_sequences(&sequences, 800, 256, 256, false), 1);
}

#[test]
fn size_block_sequences_keeps_expanding_until_sub_block_is_compressible() {
    let sequences = [sequence(1, 3), sequence(1, 3), sequence(1, 3)];

    assert_eq!(
        size_block_sequences(&sequences, 2_500, 1_024, 1_024, false),
        sequences.len()
    );
}

#[test]
fn size_block_sequences_returns_all_when_budget_is_not_reached() {
    let sequences = [sequence(2, 5), sequence(3, 8), sequence(1, 4)];

    assert_eq!(
        size_block_sequences(&sequences, 10_000, 128, 192, false),
        sequences.len()
    );
}
