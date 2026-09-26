use super::config::{HuffmanTableSearch, FILE_TYPE_SINGLE_STREAM_HUFFMAN_MAX_LITERALS};
use super::sequence_bitstream::FseTableUpdate;
use super::*;
use crate::encoding::frame_compressor::{CompressState, FseTables, OffsetHistory};
use crate::encoding::levels::c_port::sequence_store::OffBase;
use crate::encoding::{CompressionFileType, CompressionLevel};
use crate::fse::fse_encoder::SharedFSETable;
use crate::fse::fse_encoder::{
    build_table_from_data, default_ll_table, default_ml_table, default_of_table,
};
use crate::huff0::huff0_encoder::HuffmanTable;

fn offset_history(newest: u32, second: u32, third: u32) -> OffsetHistory {
    OffsetHistory {
        newest,
        second,
        third,
    }
}

fn encoded_sequence(ll: u32, ml: u32, of: u32) -> crate::blocks::sequence_section::Sequence {
    crate::blocks::sequence_section::Sequence { ll, ml, of }
}

fn literal_options(
    search_smallest_table: bool,
    suspect_uncompressible: bool,
) -> LiteralCompressionOptions {
    LiteralCompressionOptions {
        search_smallest_table,
        force_single_stream_max_literals: None,
        suspect_uncompressible,
        c_literal_cost_model: false,
        prefer_valid_repeat: false,
    }
}

#[test]
fn reusable_huffman_workspace_matches_fresh_construction_across_blocks() {
    let literals = b"tenant=alpha route=/archive status=200 bytes=4812\n".repeat(128);
    let options = LiteralCompressionOptions {
        search_smallest_table: false,
        force_single_stream_max_literals: None,
        suspect_uncompressible: false,
        c_literal_cost_model: true,
        prefer_valid_repeat: false,
    };
    let mut scratch = crate::huff0::huff0_encoder::HuffmanBuildScratch::new();

    for pass in 0..2 {
        let mut reused_output = Vec::new();
        let mut reused_writer = BitWriter::from(&mut reused_output);
        let reused_table = compress_literals_with_scratch(
            &literals,
            None,
            false,
            options,
            Some(&mut scratch),
            None,
            &mut reused_writer,
        );
        reused_writer.flush();

        let mut fresh_output = Vec::new();
        let mut fresh_writer = BitWriter::from(&mut fresh_output);
        let fresh_table = compress_literals(&literals, None, false, options, &mut fresh_writer);
        fresh_writer.flush();

        assert!(reused_table.is_some(), "pass {} should build a table", pass);
        assert!(fresh_table.is_some(), "pass {} should build a table", pass);
        assert_eq!(reused_output, fresh_output, "pass {pass}");
        assert!(scratch.retained_generated_node_capacity() > 0);
    }
}

#[test]
fn shared_huffman_weight_fse_workspace_matches_fresh_literal_transaction() {
    let literals = b"tenant=alpha route=/archive status=200 bytes=4812\n".repeat(128);
    let options = LiteralCompressionOptions {
        search_smallest_table: false,
        force_single_stream_max_literals: None,
        suspect_uncompressible: false,
        c_literal_cost_model: true,
        prefer_valid_repeat: false,
    };
    let mut huffman_scratch = crate::huff0::huff0_encoder::HuffmanBuildScratch::new();
    let mut fse_scratch = FSETableBuildScratch::new();
    let mut reused_output = Vec::new();
    let mut reused_writer = BitWriter::from(&mut reused_output);
    let reused_table = compress_literals_with_scratch(
        &literals,
        None,
        false,
        options,
        Some(&mut huffman_scratch),
        Some(&mut fse_scratch),
        &mut reused_writer,
    );
    reused_writer.flush();

    let mut fresh_output = Vec::new();
    let mut fresh_writer = BitWriter::from(&mut fresh_output);
    let fresh_table = compress_literals(&literals, None, false, options, &mut fresh_writer);
    fresh_writer.flush();

    assert_eq!(reused_output, fresh_output);
    assert_eq!(
        reused_table
            .as_ref()
            .map(HuffmanTable::table_description_len),
        fresh_table
            .as_ref()
            .map(HuffmanTable::table_description_len)
    );
    assert_eq!(fse_scratch.recycled_table_count(), 1);
}

#[test]
fn unselected_new_huffman_table_returns_to_fast_scratch() {
    let mut literals = alloc::vec![0; 4096];
    for idx in (15..literals.len()).step_by(16) {
        literals[idx] = 1;
    }
    let stats = LiteralStats::from_literals(&literals);
    let previous = HuffmanTable::build_from_counts(stats.counts());
    let options = LiteralCompressionOptions {
        search_smallest_table: false,
        force_single_stream_max_literals: None,
        suspect_uncompressible: false,
        c_literal_cost_model: true,
        prefer_valid_repeat: false,
    };
    let mut scratch = crate::huff0::huff0_encoder::HuffmanBuildScratch::new();
    let mut output = Vec::new();
    let mut writer = BitWriter::from(&mut output);

    let new_table = compress_literals_with_scratch(
        &literals,
        Some(&previous),
        true,
        options,
        Some(&mut scratch),
        None,
        &mut writer,
    );

    assert!(new_table.is_none());
    assert_eq!(output[0] & 0b11, 3);
    assert_eq!(scratch.recycled_table_count(), 1);
}

#[test]
fn legacy_decoder_guard_rejects_tiny_encoded_sequence_section() {
    assert!(should_emit_raw_for_legacy_decoder(2, 1));
    assert!(should_emit_raw_for_legacy_decoder(3, 0));
    assert!(!should_emit_raw_for_legacy_decoder(0, 1));
    assert!(!should_emit_raw_for_legacy_decoder(2, 2));
    assert!(!should_emit_raw_for_legacy_decoder(4, 0));
}

struct LiteralPayloadMatcher {
    literals: Vec<u8>,
    emitted: bool,
}

impl Matcher for LiteralPayloadMatcher {
    fn get_next_space(&mut self) -> Vec<u8> {
        Vec::new()
    }

    fn get_last_space(&self) -> &[u8] {
        &[]
    }

    fn commit_space(&mut self, _space: Vec<u8>) {}

    fn skip_matching(&mut self) {}

    fn start_matching(&mut self, mut handle_sequence: impl for<'a> FnMut(Sequence<'a>)) {
        if !self.emitted {
            self.emitted = true;
            handle_sequence(Sequence::Triple {
                literals: &self.literals,
                offset: 1,
                match_len: 16,
            });
        }
    }

    fn reset(&mut self, _level: crate::encoding::CompressionLevel) {
        self.emitted = false;
    }

    fn window_size(&self) -> u64 {
        128 * 1024
    }
}

fn compressed_frame_with_literal_payload(literals: Vec<u8>) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    compressed_frame_with_literal_payload_and_last_table(literals, None)
}

fn compressed_frame_with_literal_payload_and_last_table(
    literals: Vec<u8>,
    last_huff_table: Option<huff0_encoder::HuffmanTable>,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    compressed_frame_with_literal_payload_and_config(
        literals,
        last_huff_table,
        BlockCompressionConfig::for_level(CompressionLevel::Fastest),
    )
}

fn compressed_frame_with_literal_payload_and_config(
    literals: Vec<u8>,
    last_huff_table: Option<huff0_encoder::HuffmanTable>,
    config: BlockCompressionConfig,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    assert!(!literals.is_empty());

    let mut state = CompressState {
        matcher: LiteralPayloadMatcher {
            literals: literals.clone(),
            emitted: false,
        },
        last_huff_table,
        fse_tables: FseTables::new(),
        offset_history: OffsetHistory::new(),
        file_type_hint: CompressionFileType::Unknown,
        file_profile_hint: CompressionFileProfile::None,
    };
    let mut block_payload = Vec::new();

    compress_block_with_config(&mut state, &mut block_payload, config);

    let mut frame = Vec::new();
    crate::encoding::frame_header::FrameHeader {
        frame_content_size: None,
        single_segment: false,
        content_checksum: false,
        dictionary_id: None,
        window_size: Some(128 * 1024),
    }
    .serialize(&mut frame);
    crate::encoding::block_header::BlockHeader {
        last_block: true,
        block_type: crate::blocks::block::BlockType::Compressed,
        block_size: block_payload.len() as u32,
    }
    .serialize(&mut frame);
    frame.extend_from_slice(&block_payload);

    let last_literal = literals[literals.len() - 1];
    let mut expected = literals;
    expected.extend_from_slice(&[last_literal; 16]);

    (block_payload, frame, expected)
}

fn literal_length_code_from_spec(len: u32) -> (u8, u32, usize) {
    match len {
        0..=15 => (len as u8, 0, 0),
        16..=17 => (16, len - 16, 1),
        18..=19 => (17, len - 18, 1),
        20..=21 => (18, len - 20, 1),
        22..=23 => (19, len - 22, 1),
        24..=27 => (20, len - 24, 2),
        28..=31 => (21, len - 28, 2),
        32..=39 => (22, len - 32, 3),
        40..=47 => (23, len - 40, 3),
        48..=63 => (24, len - 48, 4),
        64..=127 => (25, len - 64, 6),
        _ => panic!("test helper only covers literal lengths through code 25"),
    }
}

fn match_length_code_from_spec(len: u32) -> (u8, u32, usize) {
    match len {
        0..=2 => panic!("match lengths below 3 are invalid"),
        3..=34 => (len as u8 - 3, 0, 0),
        35..=36 => (32, len - 35, 1),
        37..=38 => (33, len - 37, 1),
        39..=40 => (34, len - 39, 1),
        41..=42 => (35, len - 41, 1),
        43..=46 => (36, len - 43, 2),
        47..=50 => (37, len - 47, 2),
        51..=58 => (38, len - 51, 3),
        59..=66 => (39, len - 59, 3),
        67..=82 => (40, len - 67, 4),
        83..=98 => (41, len - 83, 4),
        99..=130 => (42, len - 99, 5),
        131..=258 => (43, len - 131, 7),
        _ => panic!("test helper only covers match lengths through code 43"),
    }
}

fn offset_code_from_spec(len: u32) -> (u8, u32, usize) {
    let code = len.ilog2();
    let additional = len - (1 << code);
    (code as u8, additional, code as usize)
}

#[test]
fn offset_history_uses_repeat_offsets_when_literals_are_present() {
    let mut history = OffsetHistory::new();

    assert_eq!(history.encode_offset_value(4, 3), 2);
    assert_eq!(history, offset_history(4, 1, 8));

    assert_eq!(history.encode_offset_value(4, 1), 1);
    assert_eq!(history, offset_history(4, 1, 8));

    assert_eq!(history.encode_offset_value(8, 2), 3);
    assert_eq!(history, offset_history(8, 4, 1));
}

#[test]
fn offset_history_uses_shifted_repeat_offsets_for_zero_literals() {
    let mut history = offset_history(5, 9, 13);

    assert_eq!(history.encode_offset_value(9, 0), 1);
    assert_eq!(history, offset_history(9, 5, 13));

    let mut history = offset_history(5, 9, 13);
    assert_eq!(history.encode_offset_value(13, 0), 2);
    assert_eq!(history, offset_history(13, 5, 9));

    let mut history = offset_history(5, 9, 13);
    assert_eq!(history.encode_offset_value(4, 0), 3);
    assert_eq!(history, offset_history(4, 5, 9));
}

#[test]
fn offset_history_encodes_new_offsets_and_updates_history() {
    let mut history = OffsetHistory::new();

    assert_eq!(history.encode_offset_value(10, 1), 13);
    assert_eq!(history, offset_history(10, 1, 4));
}

#[test]
fn preencoded_offset_value_preserves_c_port_offbase_choice() {
    assert_eq!(core::mem::size_of::<PreparedSequence>(), 16);
    let mut history = offset_history(10, 4, 2);
    let sequences = encode_sequences_for_history(
        &[PreparedSequence {
            ll: 0,
            ml: 13,
            raw_offset: 2,
            encoded_offset_value: 5,
        }],
        &mut history,
    );

    assert_eq!(sequences[0].of, 5);
    assert_eq!(history, offset_history(2, 10, 4));
}

#[test]
fn history_materialization_initializes_reserved_prefix_without_reallocating() {
    let mut encoded = Vec::with_capacity(4);
    let allocation = encoded.as_ptr();
    let mut history = OffsetHistory::new();
    let prepared = [
        PreparedSequence {
            ll: 1,
            ml: 7,
            raw_offset: 4,
            encoded_offset_value: 0,
        },
        PreparedSequence {
            ll: 0,
            ml: 9,
            raw_offset: 7,
            encoded_offset_value: 10,
        },
    ];

    sequence_bitstream::encode_sequences_for_history_into(&prepared, &mut history, &mut encoded);

    assert_eq!(encoded.as_ptr(), allocation);
    assert_eq!(encoded.len(), prepared.len());
    assert_eq!((encoded[0].ll, encoded[0].ml, encoded[0].of), (1, 7, 2));
    assert_eq!((encoded[1].ll, encoded[1].ml, encoded[1].of), (0, 9, 10));
    assert_eq!(history, offset_history(7, 4, 1));
}

#[test]
fn direct_c_prepared_entropy_path_matches_materialized_reference_across_blocks() {
    fn prepared_sequences(seed: u32) -> Vec<PreparedSequence> {
        (0..180u32)
            .map(|idx| {
                let ll = (idx * 7 + seed) % 80;
                let ml = 3 + (idx * 11 + seed) % 180;
                let raw_offset = 1 + (idx * 13 + seed) % 4096;
                PreparedSequence {
                    ll,
                    ml,
                    raw_offset,
                    encoded_offset_value: raw_offset + 3,
                }
            })
            .collect()
    }

    let first_sequences = prepared_sequences(3);
    let second_sequences = prepared_sequences(29);
    let first_literals = (0..4096)
        .map(|idx| ((idx * 17 + idx / 9) & 0xff) as u8)
        .collect::<Vec<_>>();
    let second_literals = (0..3072)
        .map(|idx| ((idx * 31 + idx / 5) & 0xff) as u8)
        .collect::<Vec<_>>();
    let config = BlockCompressionConfig::for_c_strategy(2);
    let mut reference_tables = FseTables::new();
    let mut direct_tables = reference_tables.clone();
    let mut reference_history = OffsetHistory::new();
    let mut direct_history = reference_history;
    let mut reference_output = Vec::new();
    let mut direct_output = Vec::new();

    let reference_first = compress_prepared_block_with_stats(
        &mut reference_output,
        config,
        PreparedBlockRef {
            literals: &first_literals,
            sequences: &first_sequences,
        },
        &mut reference_tables,
        &mut reference_history,
        None,
        false,
    );
    let direct_first = compress_c_prepared_block_with_stats(
        &mut direct_output,
        config,
        PreparedBlockRef {
            literals: &first_literals,
            sequences: &first_sequences,
        },
        &mut direct_tables,
        &mut direct_history,
        None,
        false,
    );
    assert_eq!(direct_output, reference_output);
    assert_eq!(direct_history, reference_history);
    assert_eq!(
        direct_first.should_emit_raw_block,
        reference_first.should_emit_raw_block
    );

    let reference_start = reference_output.len();
    let direct_start = direct_output.len();
    let reference_second = compress_prepared_block_with_stats(
        &mut reference_output,
        config,
        PreparedBlockRef {
            literals: &second_literals,
            sequences: &second_sequences,
        },
        &mut reference_tables,
        &mut reference_history,
        reference_first.new_huffman_table.as_ref(),
        reference_first.new_huffman_table.is_some(),
    );
    let direct_second = compress_c_prepared_block_with_stats(
        &mut direct_output,
        config,
        PreparedBlockRef {
            literals: &second_literals,
            sequences: &second_sequences,
        },
        &mut direct_tables,
        &mut direct_history,
        direct_first.new_huffman_table.as_ref(),
        direct_first.new_huffman_table.is_some(),
    );
    assert_eq!(direct_start, reference_start);
    assert_eq!(direct_output, reference_output);
    assert_eq!(direct_history, reference_history);
    assert_eq!(
        direct_second.should_emit_raw_block,
        reference_second.should_emit_raw_block
    );
}

#[test]
fn direct_stored_entropy_path_matches_prepared_path_across_blocks() {
    fn records(seed: u32) -> (Vec<PreparedSequence>, Vec<StoredSequence>) {
        (0..180u32)
            .map(|idx| {
                let ll = (idx * 7 + seed) % 80;
                let ml = 3 + (idx * 11 + seed) % 180;
                let raw_offset = 1 + (idx * 13 + seed) % 4096;
                (
                    PreparedSequence {
                        ll,
                        ml,
                        raw_offset,
                        encoded_offset_value: raw_offset + 3,
                    },
                    StoredSequence::new(ll, OffBase::Offset(raw_offset), ml),
                )
            })
            .unzip()
    }

    let (first_prepared, first_stored) = records(3);
    let (second_prepared, second_stored) = records(29);
    let first_literals = (0..4096)
        .map(|idx| ((idx * 17 + idx / 9) & 0xff) as u8)
        .collect::<Vec<_>>();
    let second_literals = (0..3072)
        .map(|idx| ((idx * 31 + idx / 5) & 0xff) as u8)
        .collect::<Vec<_>>();

    for strategy in 1..=9 {
        let config = BlockCompressionConfig::for_c_strategy(strategy);
        let mut prepared_tables = FseTables::new();
        let mut stored_tables = prepared_tables.clone();
        let mut deferred_tables = prepared_tables.clone();
        let mut prepared_history = OffsetHistory::new();
        let mut stored_history = prepared_history;
        let mut deferred_history = prepared_history;
        let mut prepared_output = Vec::new();
        let mut stored_output = Vec::new();
        let mut deferred_output = Vec::new();

        let prepared_first = compress_c_prepared_block_with_stats(
            &mut prepared_output,
            config,
            PreparedBlockRef {
                literals: &first_literals,
                sequences: &first_prepared,
            },
            &mut prepared_tables,
            &mut prepared_history,
            None,
            false,
        );
        let stored_first = compress_c_stored_block_with_stats(
            &mut stored_output,
            config,
            StoredBlockRef {
                literals: &first_literals,
                sequences: &first_stored,
            },
            &mut stored_tables,
            &mut stored_history,
            None,
            false,
            None,
            None,
        );
        let mut pending_first = PendingStoredEntropyState::new();
        let deferred_first = compress_c_stored_block_deferred_with_stats(
            &mut deferred_output,
            config,
            StoredBlockRef {
                literals: &first_literals,
                sequences: &first_stored,
            },
            &mut deferred_tables,
            &mut deferred_history,
            None,
            false,
            None,
            None,
            &mut pending_first,
        );
        assert_eq!(deferred_history, OffsetHistory::new());
        assert!(deferred_tables.ll_previous.is_none());
        assert!(deferred_tables.ml_previous.is_none());
        assert!(deferred_tables.of_previous.is_none());
        pending_first.commit(&mut deferred_tables, &mut deferred_history, None);
        assert_eq!(
            stored_output, prepared_output,
            "strategy {strategy}, block 1"
        );
        assert_eq!(
            deferred_output, prepared_output,
            "deferred strategy {strategy}, block 1"
        );
        assert_eq!(stored_history, prepared_history);
        assert_eq!(deferred_history, prepared_history);
        assert_eq!(
            stored_first.should_emit_raw_block,
            prepared_first.should_emit_raw_block
        );

        let prepared_second = compress_c_prepared_block_with_stats(
            &mut prepared_output,
            config,
            PreparedBlockRef {
                literals: &second_literals,
                sequences: &second_prepared,
            },
            &mut prepared_tables,
            &mut prepared_history,
            prepared_first.new_huffman_table.as_ref(),
            prepared_first.new_huffman_table.is_some(),
        );
        let stored_second = compress_c_stored_block_with_stats(
            &mut stored_output,
            config,
            StoredBlockRef {
                literals: &second_literals,
                sequences: &second_stored,
            },
            &mut stored_tables,
            &mut stored_history,
            stored_first.new_huffman_table.as_ref(),
            stored_first.new_huffman_table.is_some(),
            None,
            None,
        );
        let ll_before = deferred_tables.ll_previous.clone();
        let ml_before = deferred_tables.ml_previous.clone();
        let of_before = deferred_tables.of_previous.clone();
        let history_before = deferred_history;
        let mut pending_second = PendingStoredEntropyState::new();
        let _deferred_second = compress_c_stored_block_deferred_with_stats(
            &mut deferred_output,
            config,
            StoredBlockRef {
                literals: &second_literals,
                sequences: &second_stored,
            },
            &mut deferred_tables,
            &mut deferred_history,
            deferred_first.new_huffman_table.as_ref(),
            deferred_first.new_huffman_table.is_some(),
            None,
            None,
            &mut pending_second,
        );
        assert_eq!(deferred_history, history_before);
        assert_eq!(
            deferred_tables
                .ll_previous
                .as_ref()
                .map(SharedFSETable::as_ptr),
            ll_before.as_ref().map(SharedFSETable::as_ptr)
        );
        assert_eq!(
            deferred_tables
                .ml_previous
                .as_ref()
                .map(SharedFSETable::as_ptr),
            ml_before.as_ref().map(SharedFSETable::as_ptr)
        );
        assert_eq!(
            deferred_tables
                .of_previous
                .as_ref()
                .map(SharedFSETable::as_ptr),
            of_before.as_ref().map(SharedFSETable::as_ptr)
        );
        pending_second.commit(&mut deferred_tables, &mut deferred_history, None);
        assert_eq!(
            stored_output, prepared_output,
            "strategy {strategy}, block 2"
        );
        assert_eq!(
            deferred_output, prepared_output,
            "deferred strategy {strategy}, block 2"
        );
        assert_eq!(stored_history, prepared_history);
        assert_eq!(deferred_history, prepared_history);
        assert_eq!(
            stored_second.should_emit_raw_block,
            prepared_second.should_emit_raw_block
        );
    }
}

#[test]
fn matcher_offset_handoff_matches_replay_for_all_c_strategies() {
    let stored = (0..180u32)
        .map(|idx| {
            let lit_len = (idx * 7 + 3) % 80;
            let match_len = 3 + (idx * 11 + 5) % 180;
            let off_base = match idx % 7 {
                0 => OffBase::Repeat(
                    crate::encoding::levels::c_port::sequence_store::RepeatCode::First,
                ),
                1 => OffBase::Repeat(
                    crate::encoding::levels::c_port::sequence_store::RepeatCode::Second,
                ),
                2 => OffBase::Repeat(
                    crate::encoding::levels::c_port::sequence_store::RepeatCode::Third,
                ),
                _ => OffBase::Offset(1 + (idx * 13) % 4096),
            };
            StoredSequence::new(lit_len, off_base, match_len)
        })
        .collect::<Vec<_>>();
    let literals = (0..4096)
        .map(|idx| ((idx * 17 + idx / 9) & 0xff) as u8)
        .collect::<Vec<_>>();
    let initial_history = OffsetHistory::from_offsets(11, 7, 3);
    let mut final_history = initial_history;
    for sequence in &stored {
        final_history.update_from_c_offset_value(sequence.off_base_value(), sequence.lit_len);
    }

    for strategy in 1..=9 {
        let config = BlockCompressionConfig::for_c_strategy(strategy);
        let mut replay_tables = FseTables::new();
        let mut handoff_tables = replay_tables.clone();
        let mut replay_history = initial_history;
        let mut handoff_history = initial_history;
        let mut replay_output = Vec::new();
        let mut handoff_output = Vec::new();

        let replay = compress_c_stored_block_with_stats(
            &mut replay_output,
            config,
            StoredBlockRef {
                literals: &literals,
                sequences: &stored,
            },
            &mut replay_tables,
            &mut replay_history,
            None,
            false,
            None,
            None,
        );
        let handoff = compress_c_stored_block_with_matcher_history(
            &mut handoff_output,
            config,
            StoredBlockRef {
                literals: &literals,
                sequences: &stored,
            },
            &mut handoff_tables,
            &mut handoff_history,
            final_history,
            None,
            false,
            None,
            None,
        );

        assert_eq!(handoff_output, replay_output, "strategy {strategy}");
        assert_eq!(handoff_history, replay_history, "strategy {strategy}");
        assert_eq!(handoff_history, final_history, "strategy {strategy}");
        assert_eq!(
            handoff.should_emit_raw_block, replay.should_emit_raw_block,
            "strategy {strategy}"
        );

        let mut deferred_tables = FseTables::new();
        let mut deferred_history = initial_history;
        let mut deferred_output = Vec::new();
        let mut pending = PendingStoredEntropyState::new();
        let deferred = compress_c_stored_block_deferred_with_matcher_history(
            &mut deferred_output,
            config,
            StoredBlockRef {
                literals: &literals,
                sequences: &stored,
            },
            &mut deferred_tables,
            &mut deferred_history,
            final_history,
            None,
            false,
            None,
            None,
            &mut pending,
        );
        assert_eq!(deferred_history, initial_history, "strategy {strategy}");
        pending.commit(&mut deferred_tables, &mut deferred_history, None);
        assert_eq!(deferred_output, replay_output, "strategy {strategy}");
        assert_eq!(deferred_history, final_history, "strategy {strategy}");
        assert_eq!(
            deferred.should_emit_raw_block, replay.should_emit_raw_block,
            "strategy {strategy}"
        );
    }
}

#[test]
fn pending_entropy_transaction_recycles_committed_and_rejected_tables() {
    fn table(seed: u8) -> crate::fse::fse_encoder::FSETable {
        build_table_from_data(
            [seed, seed, seed.wrapping_add(1), seed.wrapping_add(2)]
                .iter()
                .copied(),
            9,
            true,
        )
    }

    let mut tables = FseTables::new();
    tables.ll_previous = Some(SharedFSETable::new(table(1)));
    tables.ml_previous = Some(SharedFSETable::new(table(4)));
    tables.of_previous = Some(SharedFSETable::new(table(7)));
    let mut history = OffsetHistory::new();
    let next_history = OffsetHistory::from_offsets(9, 5, 2);
    let mut pending = PendingStoredEntropyState {
        fse_updates: Some(PendingFseTableUpdates {
            ll: FseTableUpdate::Replace(table(10)),
            ml: FseTableUpdate::Clear,
            of: FseTableUpdate::Replace(table(13)),
        }),
        offset_history: Some(next_history),
    };
    let mut scratch = FSETableBuildScratch::new();

    pending.commit(&mut tables, &mut history, Some(&mut scratch));

    assert_eq!(history, next_history);
    assert!(tables.ll_previous.is_some());
    assert!(tables.ml_previous.is_none());
    assert!(tables.of_previous.is_some());
    assert_eq!(scratch.recycled_table_count(), 3);

    let mut rejected = PendingStoredEntropyState {
        fse_updates: Some(PendingFseTableUpdates {
            ll: FseTableUpdate::Replace(table(16)),
            ml: FseTableUpdate::Replace(table(19)),
            of: FseTableUpdate::Replace(table(22)),
        }),
        offset_history: Some(OffsetHistory::from_offsets(99, 98, 97)),
    };
    let mut rejected_scratch = FSETableBuildScratch::new();
    rejected.discard(Some(&mut rejected_scratch));

    assert_eq!(history, next_history);
    assert_eq!(rejected_scratch.recycled_table_count(), 3);
}

#[test]
fn repeat_sequence_section_rejects_tables_that_cannot_encode_symbols() {
    let mut fse_tables = FseTables::new();
    fse_tables.ll_previous = Some(SharedFSETable::new(build_table_from_data(
        [0u8, 1, 1].iter().copied(),
        9,
        true,
    )));
    fse_tables.ml_previous = Some(SharedFSETable::new(default_ml_table()));
    fse_tables.of_previous = Some(SharedFSETable::new(default_of_table()));
    let mut history = OffsetHistory::new();
    let original_history = history;
    let mut output = Vec::new();
    let sequences = [PreparedSequence {
        ll: 64,
        ml: 3,
        raw_offset: 1,
        encoded_offset_value: 1,
    }];

    let emitted =
        append_repeat_sequence_section(&sequences, &fse_tables, &mut history, &mut output);

    assert_eq!(emitted, None);
    assert!(output.is_empty());
    assert_eq!(history, original_history);
}

#[test]
fn choose_table_uses_predefined_tables_for_tiny_sequence_counts() {
    let default = default_ll_table();
    let sequences = [encoded_sequence(0, 3, 1)];

    assert!(matches!(
        choose_table(
            None,
            &default,
            &sequences,
            |seq| literal_length_code(seq.ll),
            9,
            64,
            16,
        ),
        FseTableMode::Predefined(_)
    ));
}

#[test]
fn choose_table_uses_predefined_tables_for_small_non_rle_blocks() {
    let default = default_ll_table();
    let sequences = [
        encoded_sequence(0, 3, 1),
        encoded_sequence(2, 3, 1),
        encoded_sequence(4, 3, 1),
    ];

    assert!(matches!(
        choose_table(
            None,
            &default,
            &sequences,
            |seq| literal_length_code(seq.ll),
            9,
            64,
            16,
        ),
        FseTableMode::Predefined(_)
    ));
}

#[test]
fn choose_table_uses_rle_for_repeated_codes() {
    let default = default_ll_table();
    let sequences = [encoded_sequence(5, 8, 1); 3];

    assert!(matches!(
        choose_table(
            None,
            &default,
            &sequences,
            |seq| literal_length_code(seq.ll),
            9,
            64,
            16,
        ),
        FseTableMode::Rle(5)
    ));
}

#[test]
fn valid_previous_huffman_table_lowers_literal_compression_threshold() {
    assert!(should_compress_literals(
        COMPRESS_LITERALS_SIZE_MIN,
        false,
        COMPRESS_LITERALS_SIZE_MIN
    ));
    assert!(!should_compress_literals(
        COMPRESS_LITERALS_SIZE_MIN - 1,
        false,
        COMPRESS_LITERALS_SIZE_MIN
    ));
    assert!(!should_compress_literals(
        REPEAT_LITERALS_SIZE_MIN,
        false,
        COMPRESS_LITERALS_SIZE_MIN
    ));

    assert!(should_compress_literals(
        REPEAT_LITERALS_SIZE_MIN + 1,
        true,
        COMPRESS_LITERALS_SIZE_MIN
    ));
    assert!(should_compress_literals(
        REPEAT_LITERALS_SIZE_MIN,
        true,
        COMPRESS_LITERALS_SIZE_MIN
    ));
    assert!(!should_compress_literals(
        REPEAT_LITERALS_SIZE_MIN - 1,
        true,
        COMPRESS_LITERALS_SIZE_MIN
    ));
}

#[test]
fn suspect_literal_ratio_matches_c_threshold() {
    assert!(suspect_uncompressible_literals(0, 0));
    assert!(suspect_uncompressible_literals(20, 1));
    assert!(suspect_uncompressible_literals(40, 2));
    assert!(!suspect_uncompressible_literals(39, 2));
}

#[test]
fn valid_repeat_table_bypasses_small_literal_histogram_for_fastish_c_strategy() {
    let literals = b"apowdex\n[r]\nOnCale=d\nAccuracy12h\ntimers.target";
    assert_eq!(literals.len(), 46);

    let mut training_counts = [1usize; 256];
    for &literal in literals {
        training_counts[usize::from(literal)] += 64;
    }
    let previous = HuffmanTable::build_from_counts(&training_counts);

    let mut output = Vec::new();
    let mut writer = BitWriter::from(&mut output);
    let mut options = literal_options(false, false);
    options.c_literal_cost_model = true;
    options.prefer_valid_repeat = true;
    let new_table = compress_literals(literals, Some(&previous), true, options, &mut writer);

    assert!(new_table.is_none());
    assert_eq!(
        output[0] & 0b11,
        3,
        "C's prefer-repeat path should emit treeless literals before the histogram rejection"
    );
}

#[test]
fn suspect_literal_sampling_uses_raw_literals_before_full_histogram() {
    let mut literals = Vec::with_capacity(4096 * 10);
    for value in 0_usize..4096 {
        literals.push(value as u8);
    }
    literals.resize(4096 * 9, 0);
    for value in 0_usize..4096 {
        literals.push(value as u8);
    }

    let mut sampled_output = Vec::new();
    let sampled_table = {
        let mut sampled_writer = BitWriter::from(&mut sampled_output);
        compress_literals(
            &literals,
            None,
            false,
            literal_options(true, true),
            &mut sampled_writer,
        )
    };

    assert!(sampled_table.is_none());
    assert_eq!(
        sampled_output[0] & 0b11,
        0,
        "suspect incompressible sampling should choose raw literals"
    );

    let mut full_output = Vec::new();
    let full_table = {
        let mut full_writer = BitWriter::from(&mut full_output);
        compress_literals(
            &literals,
            None,
            false,
            literal_options(true, false),
            &mut full_writer,
        )
    };

    assert!(full_table.is_some());
    assert_eq!(
        full_output[0] & 0b11,
        2,
        "without the sampling gate the compressible middle should use Huffman"
    );
}

#[test]
fn literal_stats_combined_scan_matches_four_stream_counts() {
    let literals = (0..4097)
        .map(|idx| ((idx * 37 + idx / 11) & 0xff) as u8)
        .collect::<Vec<_>>();

    let (stats, stream_counts) = LiteralStats::from_literals_with_stream_counts(&literals, true);
    let separate_stats = LiteralStats::from_literals(&literals);

    assert_eq!(stats.counts(), separate_stats.counts());
    assert_eq!(
        stream_counts.unwrap(),
        huff0_encoder::four_stream_counts(&literals)
    );
}

#[test]
fn rle_sequence_modes_round_trip_through_decoder() {
    let sequences = [
        encoded_sequence(5, 8, 1),
        encoded_sequence(5, 8, 1),
        encoded_sequence(5, 8, 1),
    ];
    let ll_mode = FseTableMode::Rle(literal_length_code(sequences[0].ll));
    let ml_mode = FseTableMode::Rle(match_length_code(sequences[0].ml));
    let of_mode = FseTableMode::Rle(offset_code(sequences[0].of));
    let mut encoded = Vec::new();
    let mut writer = BitWriter::from(&mut encoded);

    encode_seqnum(sequences.len(), &mut writer);
    writer.write_bits(encode_fse_table_modes(&ll_mode, &ml_mode, &of_mode), 8);
    encode_table(&ll_mode, &mut writer);
    encode_table(&of_mode, &mut writer);
    encode_table(&ml_mode, &mut writer);
    encode_sequences(&sequences, &mut writer, &ll_mode, &ml_mode, &of_mode);
    writer.flush();

    let mut header = crate::blocks::sequence_section::SequencesHeader::new();
    let header_size = header.parse_from_header(&encoded).unwrap();
    let mut scratch = crate::decoding::scratch::FSEScratch::new();
    let mut decoded = Vec::new();

    crate::decoding::sequence_section_decoder::decode_sequences(
        &header,
        &encoded[header_size as usize..],
        &mut scratch,
        &mut decoded,
    )
    .unwrap();

    assert_eq!(decoded.len(), sequences.len());
    for (actual, expected) in decoded.iter().zip(sequences) {
        assert_eq!(actual.ll, expected.ll);
        assert_eq!(actual.ml, expected.ml);
        assert_eq!(actual.of, expected.of);
    }
}

#[test]
fn all_rle_sequence_modes_preserve_additional_bits() {
    let sequences = [
        encoded_sequence(16, 35, 4),
        encoded_sequence(17, 36, 5),
        encoded_sequence(16, 35, 6),
    ];
    let ll_mode = FseTableMode::Rle(literal_length_code(sequences[0].ll));
    let ml_mode = FseTableMode::Rle(match_length_code(sequences[0].ml));
    let of_mode = FseTableMode::Rle(offset_code(sequences[0].of));
    let mut encoded = Vec::new();
    let mut writer = BitWriter::from(&mut encoded);

    encode_seqnum(sequences.len(), &mut writer);
    writer.write_bits(encode_fse_table_modes(&ll_mode, &ml_mode, &of_mode), 8);
    encode_table(&ll_mode, &mut writer);
    encode_table(&of_mode, &mut writer);
    encode_table(&ml_mode, &mut writer);
    encode_sequences(&sequences, &mut writer, &ll_mode, &ml_mode, &of_mode);
    writer.flush();

    let mut header = crate::blocks::sequence_section::SequencesHeader::new();
    let header_size = header.parse_from_header(&encoded).unwrap();
    let mut scratch = crate::decoding::scratch::FSEScratch::new();
    let mut decoded = Vec::new();

    crate::decoding::sequence_section_decoder::decode_sequences(
        &header,
        &encoded[header_size as usize..],
        &mut scratch,
        &mut decoded,
    )
    .unwrap();

    assert_eq!(decoded.len(), sequences.len());
    for (actual, expected) in decoded.iter().zip(sequences) {
        assert_eq!(actual.ll, expected.ll);
        assert_eq!(actual.ml, expected.ml);
        assert_eq!(actual.of, expected.of);
    }
}

#[test]
fn mixed_predefined_sequence_modes_round_trip_through_decoder() {
    let sequences = [
        encoded_sequence(0, 3, 1),
        encoded_sequence(1, 4, 2),
        encoded_sequence(4, 8, 4),
        encoded_sequence(12, 16, 8),
    ];
    let ll_default = default_ll_table();
    let ml_default = default_ml_table();
    let of_default = default_of_table();
    let ll_mode = FseTableMode::Predefined(&ll_default);
    let ml_mode = FseTableMode::Predefined(&ml_default);
    let of_mode = FseTableMode::Predefined(&of_default);
    let mut encoded = Vec::new();
    let mut writer = BitWriter::from(&mut encoded);

    encode_seqnum(sequences.len(), &mut writer);
    writer.write_bits(encode_fse_table_modes(&ll_mode, &ml_mode, &of_mode), 8);
    encode_table(&ll_mode, &mut writer);
    encode_table(&of_mode, &mut writer);
    encode_table(&ml_mode, &mut writer);
    encode_sequences(&sequences, &mut writer, &ll_mode, &ml_mode, &of_mode);
    writer.flush();

    let mut header = crate::blocks::sequence_section::SequencesHeader::new();
    let header_size = header.parse_from_header(&encoded).unwrap();
    let mut scratch = crate::decoding::scratch::FSEScratch::new();
    let mut decoded = Vec::new();

    crate::decoding::sequence_section_decoder::decode_sequences(
        &header,
        &encoded[header_size as usize..],
        &mut scratch,
        &mut decoded,
    )
    .unwrap();

    assert_eq!(decoded.len(), sequences.len());
    for (actual, expected) in decoded.iter().zip(sequences) {
        assert_eq!(actual.ll, expected.ll);
        assert_eq!(actual.ml, expected.ml);
        assert_eq!(actual.of, expected.of);
    }
}

#[test]
fn match_length_code_52_uses_65539_baseline() {
    assert_eq!(encode_match_len(65538), (51, 32767, 15));
    assert_eq!(encode_match_len(65539), (52, 0, 16));
    assert_eq!(encode_match_len(98264), (52, 32725, 16));
    assert_eq!(encode_match_len(131074), (52, 65535, 16));
}

#[test]
fn small_length_code_tables_match_spec_ranges() {
    for len in 0..=64 {
        assert_eq!(
            encode_literal_length(len),
            literal_length_code_from_spec(len)
        );
    }

    for len in 3..=131 {
        assert_eq!(encode_match_len(len), match_length_code_from_spec(len));
    }

    for len in 1..=129 {
        assert_eq!(encode_offset(len), offset_code_from_spec(len));
    }
}

#[test]
fn raw_literals_use_shortest_header_form() {
    let mut one_byte_header = Vec::new();
    let mut writer = BitWriter::from(&mut one_byte_header);
    raw_literals(&[7; 31], &mut writer);
    writer.flush();
    assert_eq!(one_byte_header[0], 31 << 3);
    assert_eq!(&one_byte_header[1..], &[7; 31]);

    let mut two_byte_header = Vec::new();
    let mut writer = BitWriter::from(&mut two_byte_header);
    raw_literals(&[9; 44], &mut writer);
    writer.flush();
    assert_eq!(&two_byte_header[..2], &[0xC4, 0x02]);
    assert_eq!(&two_byte_header[2..], &[9; 44]);

    let mut three_byte_header = Vec::new();
    let mut writer = BitWriter::from(&mut three_byte_header);
    raw_literals(&[11; 4096], &mut writer);
    writer.flush();
    assert_eq!(&three_byte_header[..3], &[0x0C, 0x00, 0x01]);
    assert_eq!(&three_byte_header[3..], &[11; 4096]);
}

#[test]
fn raw_literals_support_max_block_size_header() {
    const MAX_BLOCK_LITERALS: usize = 128 * 1024;

    let mut encoded = Vec::new();
    let mut writer = BitWriter::from(&mut encoded);
    raw_literals(&alloc::vec![0xA5; MAX_BLOCK_LITERALS], &mut writer);
    writer.flush();

    assert_eq!(&encoded[..3], &[0x0C, 0x00, 0x20]);
    assert_eq!(encoded.len(), 3 + MAX_BLOCK_LITERALS);
}

#[test]
fn sub_block_huffman_literals_fall_back_to_raw_when_repeat_expands() {
    let literals = [0xFF; 16];
    let mut counts = [0usize; 256];
    counts[0] = 4096;
    for count in counts.iter_mut().skip(1) {
        *count = 1;
    }
    let table = huff0_encoder::HuffmanTable::build_from_counts(&counts);
    let mut encoded = Vec::new();

    let emission = append_huffman_literal_section_with_table(
        &literals,
        &table,
        HuffmanLiteralMode::Repeat,
        &mut encoded,
    )
    .expect("C sub-block literal path falls back to raw literals");

    assert_eq!(emission.byte_size, encoded.len());
    assert!(emission.new_huffman_table.is_none());
    assert!(!emission.entropy_written);

    let mut section = crate::blocks::literals_section::LiteralsSection::new();
    let header_size = section.parse_from_header(&encoded).unwrap();
    assert!(matches!(
        section.ls_type,
        crate::blocks::literals_section::LiteralsSectionType::Raw
    ));

    let mut scratch = crate::decoding::scratch::HuffmanScratch::new();
    let mut decoded = Vec::new();
    crate::decoding::literals_section_decoder::decode_literals(
        &section,
        &mut scratch,
        &encoded[header_size as usize..],
        &mut decoded,
    )
    .unwrap();
    assert_eq!(decoded, literals);
}

#[test]
fn sub_block_huffman_table_rejects_no_gain_new_table_like_c() {
    let mut literals = Vec::with_capacity(3554);
    literals.extend(core::iter::repeat_n(0, 64));
    let mut state = 1u32;
    while literals.len() < 3554 {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        literals.push((state >> 24) as u8);
    }

    let (stats, _) = LiteralStats::from_literals_with_stream_counts(&literals, false);
    assert!(stats.largest() > (literals.len() >> 7) + 4);
    let table = huff0_encoder::HuffmanTable::build_from_counts(stats.counts());
    assert!(table.estimated_compressed_size_from_counts(stats.counts()) < literals.len());
    assert!(
        table.estimated_compressed_size_from_counts(stats.counts()) + table.table_description_len()
            >= literals.len()
    );

    assert!(
        build_huffman_literal_table_with_optimal_depth(&literals, false).is_none(),
        "C target-superblock literal entropy selection uses set_basic when a new Huffman table has no net gain"
    );
}

#[test]
fn sub_block_huffman_table_uses_optimal_depth_for_btultra_target_mode() {
    let counts = [
        (3u8, 1usize),
        (17, 4),
        (31, 37),
        (48, 2),
        (62, 3),
        (64, 8),
        (76, 51),
        (93, 1),
        (107, 11),
        (111, 50),
        (152, 11),
        (158, 59),
        (161, 1),
        (163, 3),
        (164, 3),
        (165, 3),
        (166, 1),
        (167, 5),
        (168, 4),
        (169, 7),
        (170, 10),
        (171, 5),
        (172, 6),
        (173, 8),
        (174, 13),
        (175, 30),
        (176, 23),
        (177, 26),
        (183, 1),
        (197, 18),
        (228, 4),
        (242, 27),
    ];
    let mut literals = Vec::new();
    for (symbol, count) in counts {
        literals.extend(core::iter::repeat_n(symbol, count));
    }

    let fast_table = build_huffman_literal_table_with_optimal_depth(&literals, false)
        .expect("fast table is worthwhile");
    let optimal_table = build_huffman_literal_table_with_optimal_depth(&literals, true)
        .expect("optimal-depth table is worthwhile");

    assert_eq!(literals.len(), 436);
    assert_eq!(fast_table.table_description_len(), 35);
    assert_eq!(
        optimal_table.table_description_len(),
        31,
        "C enables HUF optimal-depth table search for btultra/btultra2 target superblocks"
    );
}

#[test]
fn sub_block_huffman_optimal_depth_prefers_shallower_table_on_estimate_tie() {
    let counts = [
        (0u8, 32usize),
        (1, 69),
        (2, 33),
        (3, 140),
        (4, 258),
        (5, 1820),
        (6, 180),
        (7, 30),
        (8, 58),
        (9, 29),
        (10, 52),
        (11, 44),
        (12, 44),
        (13, 31),
        (14, 87),
        (15, 71),
        (29, 1),
        (39, 10),
        (49, 1),
        (51, 1),
        (52, 1),
        (65, 1),
        (66, 10),
        (73, 1),
        (74, 1),
        (79, 1),
        (86, 1),
        (93, 15),
        (94, 1),
        (106, 4),
        (117, 1),
        (121, 15),
        (134, 3),
        (139, 1),
        (145, 1),
        (152, 1),
        (159, 1),
        (161, 1),
        (173, 1),
        (180, 1),
        (187, 1),
        (191, 2),
        (192, 2),
        (198, 2),
        (220, 2),
        (229, 1),
        (232, 1),
        (246, 4),
        (253, 2),
    ];
    let mut literals = Vec::new();
    for (symbol, count) in counts {
        literals.extend(core::iter::repeat_n(symbol, count));
    }

    let optimal_table = build_huffman_literal_table_with_optimal_depth(&literals, true)
        .expect("optimal-depth table is worthwhile");

    assert_eq!(literals.len(), 3070);
    assert_eq!(
        optimal_table.table_description_len(),
        47,
        "C keeps the shallower HUF optimal-depth candidate when estimated size ties"
    );
}

#[test]
fn sub_block_huffman_optimal_depth_does_not_use_rank_limited_rust_search() {
    let counts = [
        (10u8, 3usize),
        (32, 14),
        (33, 1),
        (34, 2),
        (35, 1),
        (40, 1),
        (41, 1),
        (44, 2),
        (45, 1),
        (47, 2),
        (58, 4),
        (59, 1),
        (61, 1),
        (69, 1),
        (75, 1),
        (82, 2),
        (87, 1),
        (91, 1),
        (93, 1),
        (97, 8),
        (98, 2),
        (99, 2),
        (100, 3),
        (101, 11),
        (102, 3),
        (103, 1),
        (105, 9),
        (108, 5),
        (109, 2),
        (110, 4),
        (111, 7),
        (112, 3),
        (114, 9),
        (115, 6),
        (116, 9),
        (117, 3),
        (118, 1),
        (121, 1),
        (123, 1),
        (125, 1),
    ];
    let mut literals = Vec::new();
    for (symbol, count) in counts {
        literals.extend(core::iter::repeat_n(symbol, count));
    }

    let optimal_table = build_huffman_literal_table_with_optimal_depth(&literals, true)
        .expect("optimal-depth table is worthwhile");

    assert_eq!(literals.len(), 132);
    assert_eq!(
        optimal_table.table_description_len(),
        25,
        "C HUF optimal-depth search does not try Rust's rank-limited alternate table"
    );
}

#[test]
fn sub_block_huffman_estimate_excludes_four_stream_jump_table_like_c() {
    let mut literals = Vec::with_capacity(2048);
    for idx in 0..2048 {
        literals.push((idx % 17) as u8);
    }

    let (stats, _) = LiteralStats::from_literals_with_stream_counts(&literals, false);
    let table = huff0_encoder::HuffmanTable::build_from_counts(stats.counts());
    let estimate = estimate_huffman_literal_section_with_table(&literals, &table, true)
        .expect("table can encode literals");

    assert_eq!(
        estimate,
        table.estimated_compressed_size_from_counts(stats.counts())
            + table.table_description_len()
            + 3,
        "C target-superblock sizing uses HUF_estimateCompressedSize + hufDesSize + a 3-byte literal header"
    );
}

#[test]
fn sub_block_huffman_literals_emit_raw_when_empty() {
    let table = huff0_encoder::HuffmanTable::build_from_counts(&[1, 1]);
    let mut encoded = Vec::new();

    let emission = append_huffman_literal_section_with_table(
        &[],
        &table,
        HuffmanLiteralMode::Compressed,
        &mut encoded,
    )
    .expect("C sub-block literal path emits raw zero literals");

    assert_eq!(emission.byte_size, 1);
    assert!(emission.new_huffman_table.is_none());
    assert!(!emission.entropy_written);
    assert_eq!(encoded, [0]);
}

#[test]
fn rle_literals_use_shortest_header_form() {
    let mut one_byte_header = Vec::new();
    let mut writer = BitWriter::from(&mut one_byte_header);
    rle_literals(&[7; 31], &mut writer);
    writer.flush();
    assert_eq!(&one_byte_header, &[0xF9, 7]);

    let mut two_byte_header = Vec::new();
    let mut writer = BitWriter::from(&mut two_byte_header);
    rle_literals(&[9; 44], &mut writer);
    writer.flush();
    assert_eq!(&two_byte_header, &[0xC5, 0x02, 9]);

    let mut three_byte_header = Vec::new();
    let mut writer = BitWriter::from(&mut three_byte_header);
    rle_literals(&[11; 4096], &mut writer);
    writer.flush();
    assert_eq!(&three_byte_header, &[0x0D, 0x00, 0x01, 11]);
}

#[test]
fn rle_literals_support_max_block_size_header() {
    const MAX_BLOCK_LITERALS: usize = 128 * 1024;

    let mut encoded = Vec::new();
    let mut writer = BitWriter::from(&mut encoded);
    rle_literals(&alloc::vec![0x5A; MAX_BLOCK_LITERALS], &mut writer);
    writer.flush();

    assert_eq!(&encoded, &[0x0D, 0x00, 0x20, 0x5A]);
}

#[test]
fn rle_literals_round_trip_through_decoder() {
    let mut encoded = Vec::new();
    let mut writer = BitWriter::from(&mut encoded);
    rle_literals(&[42; 44], &mut writer);
    writer.flush();

    let mut section = crate::blocks::literals_section::LiteralsSection::new();
    let header_size = section.parse_from_header(&encoded).unwrap();
    assert!(matches!(
        section.ls_type,
        crate::blocks::literals_section::LiteralsSectionType::RLE
    ));

    let mut scratch = crate::decoding::scratch::HuffmanScratch::new();
    let mut decoded = Vec::new();
    let bytes_read = crate::decoding::literals_section_decoder::decode_literals(
        &section,
        &mut scratch,
        &encoded[header_size as usize..],
        &mut decoded,
    )
    .unwrap();

    assert_eq!(bytes_read, 1);
    assert_eq!(decoded, [42; 44]);
}

#[test]
fn rle_literals_frame_round_trips_through_rust_and_c_decoders() {
    let (block_payload, frame, expected) =
        compressed_frame_with_literal_payload(alloc::vec![42; 2048]);

    assert_eq!(block_payload[0] & 0b11, 1, "literal section should be RLE");

    let mut rust_decoded = Vec::with_capacity(expected.len());
    let mut decoder = crate::decoding::FrameDecoder::new();
    decoder
        .decode_all_to_vec(&frame, &mut rust_decoded)
        .unwrap();
    assert_eq!(rust_decoded, expected);

    let mut c_decoded = Vec::new();
    zstd::stream::copy_decode(frame.as_slice(), &mut c_decoded).unwrap();
    assert_eq!(c_decoded, expected);
}

#[test]
fn small_rle_literals_use_previous_table_threshold_and_round_trip() {
    let previous_table = huff0_encoder::HuffmanTable::build_from_counts(&[8, 1, 1, 1, 1, 1, 1, 1]);
    let (block_payload, frame, expected) = compressed_frame_with_literal_payload_and_last_table(
        alloc::vec![42; 7],
        Some(previous_table),
    );

    assert_eq!(
        block_payload[0] & 0b11,
        1,
        "small repeated literals should use RLE when a previous table lowers the threshold"
    );

    let mut rust_decoded = Vec::with_capacity(expected.len());
    let mut decoder = crate::decoding::FrameDecoder::new();
    decoder
        .decode_all_to_vec(&frame, &mut rust_decoded)
        .unwrap();
    assert_eq!(rust_decoded, expected);

    let mut c_decoded = Vec::new();
    zstd::stream::copy_decode(frame.as_slice(), &mut c_decoded).unwrap();
    assert_eq!(c_decoded, expected);
}

#[test]
fn small_compressible_literals_use_huffman_and_round_trip() {
    let mut literals = alloc::vec![b'a'; 512];
    for idx in (15..literals.len()).step_by(16) {
        literals[idx] = b'b';
    }

    let (block_payload, frame, expected) = compressed_frame_with_literal_payload(literals);

    assert_eq!(
        block_payload[0] & 0b11,
        2,
        "small skewed literal section should use Huffman compression"
    );

    let mut rust_decoded = Vec::with_capacity(expected.len());
    let mut decoder = crate::decoding::FrameDecoder::new();
    decoder
        .decode_all_to_vec(&frame, &mut rust_decoded)
        .unwrap();
    assert_eq!(rust_decoded, expected);

    let mut c_decoded = Vec::new();
    zstd::stream::copy_decode(frame.as_slice(), &mut c_decoded).unwrap();
    assert_eq!(c_decoded, expected);
}

#[test]
fn small_huffman_literals_use_single_stream_and_round_trip() {
    let mut literals = alloc::vec![b'a'; 128];
    for idx in (15..literals.len()).step_by(16) {
        literals[idx] = b'b';
    }

    let (block_payload, frame, expected) = compressed_frame_with_literal_payload(literals);

    assert_eq!(
        block_payload[0] & 0b11,
        2,
        "small skewed literal section should use Huffman compression"
    );
    assert_eq!(
        (block_payload[0] >> 2) & 0b11,
        0,
        "small Huffman literal payloads should use the single-stream header"
    );

    let mut rust_decoded = Vec::with_capacity(expected.len());
    let mut decoder = crate::decoding::FrameDecoder::new();
    decoder
        .decode_all_to_vec(&frame, &mut rust_decoded)
        .unwrap();
    assert_eq!(rust_decoded, expected);

    let mut c_decoded = Vec::new();
    zstd::stream::copy_decode(frame.as_slice(), &mut c_decoded).unwrap();
    assert_eq!(c_decoded, expected);
}

#[test]
fn small_literals_prefer_previous_huffman_table_and_single_stream() {
    let mut first_literals = alloc::vec![0; 512];
    for idx in (15..first_literals.len()).step_by(16) {
        first_literals[idx] = 1;
    }
    let second_literals = first_literals.clone();

    let mut state = CompressState {
        matcher: LiteralPayloadMatcher {
            literals: first_literals.clone(),
            emitted: false,
        },
        last_huff_table: None,
        fse_tables: FseTables::new(),
        offset_history: OffsetHistory::new(),
        file_type_hint: CompressionFileType::Unknown,
        file_profile_hint: CompressionFileProfile::None,
    };
    let mut first_payload = Vec::new();
    state.last_huff_table = compress_block_with_config(
        &mut state,
        &mut first_payload,
        BlockCompressionConfig::for_level(CompressionLevel::Fastest),
    );

    state.matcher = LiteralPayloadMatcher {
        literals: second_literals.clone(),
        emitted: false,
    };
    let mut second_payload = Vec::new();
    compress_block_with_config(
        &mut state,
        &mut second_payload,
        BlockCompressionConfig::for_level(CompressionLevel::Fastest),
    );

    assert_eq!(
        second_payload[0] & 0b11,
        3,
        "small literals encodable by previous table should use treeless Huffman"
    );
    assert_eq!(
        (second_payload[0] >> 2) & 0b11,
        0,
        "small repeated-table Huffman literals should use the single-stream header"
    );

    let mut frame = Vec::new();
    crate::encoding::frame_header::FrameHeader {
        frame_content_size: None,
        single_segment: false,
        content_checksum: false,
        dictionary_id: None,
        window_size: Some(128 * 1024),
    }
    .serialize(&mut frame);
    crate::encoding::block_header::BlockHeader {
        last_block: false,
        block_type: crate::blocks::block::BlockType::Compressed,
        block_size: first_payload.len() as u32,
    }
    .serialize(&mut frame);
    frame.extend_from_slice(&first_payload);
    crate::encoding::block_header::BlockHeader {
        last_block: true,
        block_type: crate::blocks::block::BlockType::Compressed,
        block_size: second_payload.len() as u32,
    }
    .serialize(&mut frame);
    frame.extend_from_slice(&second_payload);

    let mut expected = first_literals.clone();
    expected.extend_from_slice(&[1; 16]);
    expected.extend_from_slice(&second_literals);
    expected.extend_from_slice(&[1; 16]);

    let mut rust_decoded = Vec::with_capacity(expected.len());
    let mut decoder = crate::decoding::FrameDecoder::new();
    decoder
        .decode_all_to_vec(&frame, &mut rust_decoded)
        .unwrap();
    assert_eq!(rust_decoded, expected);

    let mut c_decoded = Vec::new();
    zstd::stream::copy_decode(frame.as_slice(), &mut c_decoded).unwrap();
    assert_eq!(c_decoded, expected);
}

#[test]
fn literal_estimate_without_gain_uses_raw_literals_and_round_trips() {
    let mut literals = alloc::vec![0; 5];
    for value in 1..=69u8 {
        literals.push(value);
    }

    let (block_payload, frame, expected) = compressed_frame_with_literal_payload(literals);

    assert_eq!(
        block_payload[0] & 0b11,
        0,
        "literal estimate should choose raw when Huffman cannot beat raw"
    );

    let mut rust_decoded = Vec::with_capacity(expected.len());
    let mut decoder = crate::decoding::FrameDecoder::new();
    decoder
        .decode_all_to_vec(&frame, &mut rust_decoded)
        .unwrap();
    assert_eq!(rust_decoded, expected);

    let mut c_decoded = Vec::new();
    zstd::stream::copy_decode(frame.as_slice(), &mut c_decoded).unwrap();
    assert_eq!(c_decoded, expected);
}

#[test]
fn disabled_literal_compression_forces_raw_even_with_previous_table() {
    let mut literals = Vec::with_capacity(96);
    for idx in 0..96 {
        literals.push((idx % 3) as u8);
    }
    let literal_stats = LiteralStats::from_literals(&literals);
    let previous_table = huff0_encoder::HuffmanTable::build_from_counts(literal_stats.counts());
    let mut config = BlockCompressionConfig::for_level(CompressionLevel::Fastest);
    config.disable_literal_compression();

    let (block_payload, frame, expected) =
        compressed_frame_with_literal_payload_and_config(literals, Some(previous_table), config);

    assert_eq!(
        block_payload[0] & 0b11,
        0,
        "C disabled literal compression maps directly to raw literals"
    );

    let mut rust_decoded = Vec::with_capacity(expected.len());
    let mut decoder = crate::decoding::FrameDecoder::new();
    decoder
        .decode_all_to_vec(&frame, &mut rust_decoded)
        .unwrap();
    assert_eq!(rust_decoded, expected);
}

#[test]
fn literal_min_gain_boundary_uses_exact_table_search_and_round_trips() {
    let len = 129usize;
    let period = 88u32;
    let mut state = (len as u32).wrapping_mul(1_664_525).wrapping_add(period);
    let mut literals = Vec::with_capacity(len);
    for _ in 0..len {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        literals.push((state % period) as u8);
    }

    let literal_stats = LiteralStats::from_literals(&literals);
    let table = huff0_encoder::HuffmanTable::build_from_counts(literal_stats.counts());
    let estimated_len = table.encoded_len(&literals, true, false);
    let exact_table = huff0_encoder::HuffmanTable::build_smallest_from_counts(
        literal_stats.counts(),
        &literals,
        false,
    );
    let exact_estimated_len = exact_table.encoded_len(&literals, true, false);
    let header_len = compressed_literals_header_len(0);

    assert_eq!(literal_min_gain(literals.len()), 4);
    assert!(
        estimated_len + header_len < literals.len(),
        "without the min-gain check this payload would be Huffman-compressed"
    );
    assert!(
        estimated_len
            >= literals
                .len()
                .saturating_sub(literal_min_gain(literals.len())),
        "C-style min-gain threshold should reject this narrow literal gain"
    );
    assert!(
        literal_estimate_has_enough_gain(exact_estimated_len, header_len, literals.len()),
        "exact table search should find enough gain for small all-literal payloads"
    );

    let (block_payload, frame, expected) = compressed_frame_with_literal_payload(literals);

    assert_eq!(
        block_payload[0] & 0b11,
        2,
        "small all-literal payloads should use the exact Huffman table when it has enough gain"
    );

    let mut rust_decoded = Vec::with_capacity(expected.len());
    let mut decoder = crate::decoding::FrameDecoder::new();
    decoder
        .decode_all_to_vec(&frame, &mut rust_decoded)
        .unwrap();
    assert_eq!(rust_decoded, expected);

    let mut c_decoded = Vec::new();
    zstd::stream::copy_decode(frame.as_slice(), &mut c_decoded).unwrap();
    assert_eq!(c_decoded, expected);
}

#[test]
fn best_level_searches_exact_huffman_tables_beyond_small_literal_sections() {
    let len = SMALL_HUFFMAN_TABLE_SEARCH_MAX_LITERALS + 256;
    let period = 86u32;
    let mut state = (len as u32).wrapping_mul(1_664_525).wrapping_add(period);
    let mut literals = Vec::with_capacity(len);
    for _ in 0..len {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        literals.push((state % period) as u8);
    }

    let literal_stats = LiteralStats::from_literals(&literals);
    let baseline_table = huff0_encoder::HuffmanTable::build_from_counts(literal_stats.counts());
    let exact_table = huff0_encoder::HuffmanTable::build_smallest_from_counts(
        literal_stats.counts(),
        &literals,
        true,
    );
    assert!(
        exact_table.encoded_len(&literals, true, true)
            <= baseline_table.encoded_len(&literals, true, true),
        "exact table search should not be worse on this higher-level fixture"
    );

    let (fast_block, _, _) = compressed_frame_with_literal_payload_and_config(
        literals.clone(),
        None,
        BlockCompressionConfig::for_level_and_file_type(
            CompressionLevel::Fastest,
            CompressionFileType::ArchiveLike,
        ),
    );
    let (best_block, best_frame, expected) = compressed_frame_with_literal_payload_and_config(
        literals,
        None,
        BlockCompressionConfig::for_level_and_file_type(
            CompressionLevel::Best,
            CompressionFileType::ArchiveLike,
        ),
    );

    assert!(
        best_block.len() <= fast_block.len(),
        "best-level exact Huffman search should not emit a larger block: {} > {}",
        best_block.len(),
        fast_block.len()
    );

    let mut rust_decoded = Vec::with_capacity(expected.len());
    let mut decoder = crate::decoding::FrameDecoder::new();
    decoder
        .decode_all_to_vec(&best_frame, &mut rust_decoded)
        .unwrap();
    assert_eq!(rust_decoded, expected);

    let mut c_decoded = Vec::new();
    zstd::stream::copy_decode(best_frame.as_slice(), &mut c_decoded).unwrap();
    assert_eq!(c_decoded, expected);
}

#[test]
fn fastest_code_and_config_text_enable_small_literal_exact_search() {
    assert!(matches!(
        BlockCompressionConfig::for_level_and_file_type(
            CompressionLevel::Fastest,
            CompressionFileType::CodeText,
        )
        .huffman_table_search,
        HuffmanTableSearch::FileTypeSmall
    ));
    assert!(matches!(
        BlockCompressionConfig::for_level_and_file_type(
            CompressionLevel::Fastest,
            CompressionFileType::ConfigText,
        )
        .huffman_table_search,
        HuffmanTableSearch::FileTypeSmall
    ));
    assert!(matches!(
        BlockCompressionConfig::for_level_and_file_type(
            CompressionLevel::Fastest,
            CompressionFileType::Unknown,
        )
        .huffman_table_search,
        HuffmanTableSearch::FileTypeSmall
    ));
    assert!(matches!(
        BlockCompressionConfig::for_level_and_file_type(
            CompressionLevel::Fastest,
            CompressionFileType::JsonText,
        )
        .huffman_table_search,
        HuffmanTableSearch::Heuristic
    ));
    assert!(matches!(
        BlockCompressionConfig::for_level_and_file_type(
            CompressionLevel::Fastest,
            CompressionFileType::DictionaryText,
        )
        .huffman_table_search,
        HuffmanTableSearch::AllSections
    ));
}

#[test]
fn fastest_config_text_enables_small_single_stream_huffman_override() {
    assert_eq!(
        BlockCompressionConfig::for_level_and_file_type(
            CompressionLevel::Fastest,
            CompressionFileType::ConfigText,
        )
        .file_type_single_stream_huffman_max_literals,
        Some(FILE_TYPE_SINGLE_STREAM_HUFFMAN_MAX_LITERALS)
    );
    assert_eq!(
        BlockCompressionConfig::for_level_and_file_type(
            CompressionLevel::Fastest,
            CompressionFileType::CodeText,
        )
        .file_type_single_stream_huffman_max_literals,
        None
    );
}

#[test]
fn c_strategy_literal_thresholds_match_zstd_min_literals_to_compress() {
    assert_eq!(
        BlockCompressionConfig::for_c_strategy(1).literal_compression_min_size,
        64
    );
    assert_eq!(
        BlockCompressionConfig::for_c_strategy(5).literal_compression_min_size,
        64
    );
    assert_eq!(
        BlockCompressionConfig::for_c_strategy(7).literal_compression_min_size,
        32
    );
    assert_eq!(
        BlockCompressionConfig::for_c_strategy(8).literal_compression_min_size,
        16
    );
    assert_eq!(
        BlockCompressionConfig::for_c_strategy(9).literal_compression_min_size,
        8
    );
}

#[test]
fn c_strategy_huffman_search_matches_strategy_threshold() {
    assert_eq!(
        BlockCompressionConfig::for_c_strategy(7).huffman_table_search,
        HuffmanTableSearch::Heuristic
    );
    assert_eq!(
        BlockCompressionConfig::for_c_strategy(8).huffman_table_search,
        HuffmanTableSearch::AllSections
    );
    assert_eq!(
        BlockCompressionConfig::for_c_strategy(9).huffman_table_search,
        HuffmanTableSearch::AllSections
    );
    assert!(!BlockCompressionConfig::for_c_strategy(7).search_smallest_huffman_table(256, 0));
    assert!(BlockCompressionConfig::for_c_strategy(8).search_smallest_huffman_table(256, 1));
    assert!(BlockCompressionConfig::for_level(CompressionLevel::Best)
        .search_smallest_huffman_table(256, 0));
}

#[test]
fn c_block_split_estimate_preserves_btultra_optimal_depth() {
    assert_eq!(
        BlockCompressionConfig::for_c_strategy(7)
            .for_c_block_split_estimate()
            .huffman_table_search,
        HuffmanTableSearch::Heuristic
    );
    assert_eq!(
        BlockCompressionConfig::for_c_strategy(8)
            .for_c_block_split_estimate()
            .huffman_table_search,
        HuffmanTableSearch::AllSections
    );
}

#[test]
fn c_strategy_literal_compression_stays_enabled_by_default() {
    assert!(!BlockCompressionConfig::for_c_strategy(1).literal_compression_disabled);
}

#[test]
fn c_strategy_uses_c_literal_cost_model() {
    assert!(BlockCompressionConfig::for_c_strategy(9).c_literal_cost_model);
    assert!(!BlockCompressionConfig::for_level(CompressionLevel::Best).c_literal_cost_model);
}

#[test]
fn c_fast_sequence_emission_is_limited_to_fast_and_dfast() {
    assert!(BlockCompressionConfig::for_c_strategy(1).c_fast_sequence_emission);
    assert!(BlockCompressionConfig::for_c_strategy(2).c_fast_sequence_emission);
    assert!(!BlockCompressionConfig::for_c_strategy(1).c_dfast_compact_sequence_statistics);
    assert!(BlockCompressionConfig::for_c_strategy(2).c_dfast_compact_sequence_statistics);
    for strategy in 3..=9 {
        assert!(!BlockCompressionConfig::for_c_strategy(strategy).c_fast_sequence_emission);
        assert!(
            !BlockCompressionConfig::for_c_strategy(strategy).c_dfast_compact_sequence_statistics
        );
    }
}

#[test]
fn fastest_dictionary_text_keeps_default_predefined_llml_window() {
    let config = BlockCompressionConfig::for_level_and_file_type(
        CompressionLevel::Fastest,
        CompressionFileType::DictionaryText,
    );
    assert_eq!(
        config.file_type_small_sequence_predefined_llml_max_sequences,
        None
    );

    let code_config = BlockCompressionConfig::for_level_and_file_type(
        CompressionLevel::Fastest,
        CompressionFileType::CodeText,
    );
    assert_eq!(
        code_config.file_type_small_sequence_predefined_llml_max_sequences,
        None
    );
}

#[test]
fn fastest_dictionary_text_enables_exact_sequence_mode_search() {
    let dictionary_config = BlockCompressionConfig::for_level_and_hints(
        CompressionLevel::Fastest,
        CompressionFileType::DictionaryText,
        CompressionFileProfile::None,
    );
    assert!(dictionary_config.exact_sequence_mode_search);

    let dependency_json_config = BlockCompressionConfig::for_level_and_hints(
        CompressionLevel::Fastest,
        CompressionFileType::JsonText,
        CompressionFileProfile::DependencyJsonLockfile,
    );
    assert!(dependency_json_config.exact_sequence_mode_search);

    let small_text_lockfile_config = BlockCompressionConfig::for_level_and_hints(
        CompressionLevel::Fastest,
        CompressionFileType::ConfigText,
        CompressionFileProfile::SmallTextLockfile,
    );
    assert!(!small_text_lockfile_config.exact_sequence_mode_search);
    assert_eq!(small_text_lockfile_config.offset_table_max_log, 7);
    assert_eq!(
        small_text_lockfile_config.offset_predefined_max_sequences,
        64
    );
    assert_eq!(small_text_lockfile_config.repeat_table_max_sequences, 256);

    let code_config = BlockCompressionConfig::for_level_and_hints(
        CompressionLevel::Fastest,
        CompressionFileType::CodeText,
        CompressionFileProfile::None,
    );
    assert!(!code_config.exact_sequence_mode_search);
}

#[test]
fn forced_single_stream_huffman_uses_single_stream_size_format() {
    assert_eq!(compressed_literals_size_format(821, false), (0b01, 10));
    assert_eq!(compressed_literals_size_format(821, true), (0b00, 10));
    assert_eq!(
        compressed_literals_size_format(128 * 1024, false),
        (0b11, 18)
    );
}

#[test]
fn choose_table_repeats_previous_table_for_small_blocks_when_valid() {
    let default = default_of_table();
    let previous = build_table_from_data([29u8, 30, 30].iter().copied(), 8, true);
    let sequences = [
        encoded_sequence(0, 3, 1 << 29),
        encoded_sequence(0, 3, 1 << 30),
        encoded_sequence(0, 3, 1 << 30),
    ];

    assert!(matches!(
        choose_table(
            Some(&previous),
            &default,
            &sequences,
            |seq| offset_code(seq.of),
            8,
            64,
            16,
        ),
        FseTableMode::RepeatLast(_)
    ));
}

#[test]
fn c_fast_sequence_heuristic_repeats_previous_table_before_default() {
    let ll_default = default_ll_table();
    let ml_default = default_ml_table();
    let of_default = default_of_table();
    let ll_previous = build_table_from_data([0u8, 2, 4].iter().copied(), 9, true);
    let sequences = [
        encoded_sequence(0, 3, 1),
        encoded_sequence(2, 3, 1),
        encoded_sequence(4, 3, 1),
    ];

    let (ll_mode, _, _) = choose_sequence_table_modes(
        &sequences,
        SequenceModeSearchConfig {
            ll_previous: Some(&ll_previous),
            ll_repeat_valid: true,
            ll_default: &ll_default,
            ml_previous: None,
            ml_repeat_valid: false,
            ml_default: &ml_default,
            of_previous: None,
            of_repeat_valid: false,
            of_default: &of_default,
            repeat_table_max_sequences: 1000,
            llml_predefined_max_sequences: 56,
            of_predefined_max_sequences: 28,
            of_max_log: 8,
            exact_sequence_mode_search: false,
            c_fast_heuristics: true,
            c_cost_model: false,
        },
    );

    assert!(matches!(ll_mode, FseTableMode::RepeatLast(_)));
}

#[test]
fn c_fast_sequence_heuristic_does_not_repeat_check_only_table() {
    let ll_default = default_ll_table();
    let ml_default = default_ml_table();
    let of_default = default_of_table();
    let ll_previous = build_table_from_data([0u8, 2, 4].iter().copied(), 9, true);
    let sequences = sequences_for_literal_lengths([0, 2, 4]);

    let (ll_mode, _, _) = choose_sequence_table_modes(
        &sequences,
        SequenceModeSearchConfig {
            ll_previous: Some(&ll_previous),
            ll_repeat_valid: false,
            ll_default: &ll_default,
            ml_previous: None,
            ml_repeat_valid: false,
            ml_default: &ml_default,
            of_previous: None,
            of_repeat_valid: false,
            of_default: &of_default,
            repeat_table_max_sequences: 1000,
            llml_predefined_max_sequences: 56,
            of_predefined_max_sequences: 28,
            of_max_log: 8,
            exact_sequence_mode_search: false,
            c_fast_heuristics: true,
            c_cost_model: false,
        },
    );

    assert!(matches!(ll_mode, FseTableMode::Predefined(_)));
}

#[test]
fn c_fast_sequence_heuristic_requires_default_table_before_repeat() {
    let ll_default = default_ll_table();
    let ml_default = default_ml_table();
    let of_default = default_of_table();
    let of_previous = build_table_from_data([29u8, 30, 30].iter().copied(), 8, true);
    let sequences = [
        encoded_sequence(0, 3, 1 << 29),
        encoded_sequence(0, 3, 1 << 30),
        encoded_sequence(0, 3, 1 << 30),
    ];

    let (_, _, of_mode) = choose_sequence_table_modes(
        &sequences,
        SequenceModeSearchConfig {
            ll_previous: None,
            ll_repeat_valid: false,
            ll_default: &ll_default,
            ml_previous: None,
            ml_repeat_valid: false,
            ml_default: &ml_default,
            of_previous: Some(&of_previous),
            of_repeat_valid: true,
            of_default: &of_default,
            repeat_table_max_sequences: 1000,
            llml_predefined_max_sequences: 56,
            of_predefined_max_sequences: 28,
            of_max_log: 8,
            exact_sequence_mode_search: false,
            c_fast_heuristics: true,
            c_cost_model: false,
        },
    );

    assert!(matches!(of_mode, FseTableMode::Encoded(_)));
}

#[test]
fn c_cost_sequence_model_uses_basic_when_default_is_cheapest() {
    let ll_default = default_ll_table();
    let ml_default = default_ml_table();
    let of_default = default_of_table();
    let sequences = sequences_for_literal_lengths([0, 2, 4, 6, 8, 10, 12, 14]);

    let (ll_mode, _, _) = choose_sequence_table_modes(
        &sequences,
        c_cost_config(&ll_default, None, &ml_default, &of_default, None),
    );

    assert!(matches!(ll_mode, FseTableMode::Predefined(_)));
}

#[test]
fn c_cost_sequence_model_repeats_previous_table_when_cheapest() {
    let ll_default = default_ll_table();
    let ml_default = default_ml_table();
    let of_default = default_of_table();
    let mut lengths = Vec::new();
    lengths.extend(core::iter::repeat_n(0, 160));
    lengths.extend(core::iter::repeat_n(15, 40));
    let sequences = sequences_for_literal_lengths(lengths.iter().copied());
    let previous = build_table_from_data(
        sequences
            .iter()
            .map(|sequence| literal_length_code(sequence.ll)),
        9,
        true,
    );

    let (ll_mode, _, _) = choose_sequence_table_modes(
        &sequences,
        c_cost_config(&ll_default, Some(&previous), &ml_default, &of_default, None),
    );

    assert!(matches!(ll_mode, FseTableMode::RepeatLast(_)));
}

#[test]
fn c_cost_sequence_model_encodes_new_table_when_cheapest() {
    let ll_default = default_ll_table();
    let ml_default = default_ml_table();
    let of_default = default_of_table();
    let mut lengths = Vec::new();
    lengths.extend(core::iter::repeat_n(60, 120));
    lengths.extend(core::iter::repeat_n(1024, 80));
    let sequences = sequences_for_literal_lengths(lengths.iter().copied());

    let (ll_mode, _, _) = choose_sequence_table_modes(
        &sequences,
        c_cost_config(&ll_default, None, &ml_default, &of_default, None),
    );

    assert!(matches!(ll_mode, FseTableMode::Encoded(_)));
}

#[test]
fn encoded_sequence_table_removes_repeated_last_code_from_counts() {
    let ll_default = default_ll_table();
    let ml_default = default_ml_table();
    let of_default = default_of_table();
    let mut lengths = Vec::new();
    lengths.extend(core::iter::repeat_n(0, 2));
    lengths.extend(core::iter::repeat_n(64, 2));
    lengths.push(0);
    let sequences = sequences_for_literal_lengths(lengths.iter().copied());

    let (ll_mode, _, _) = choose_sequence_table_modes(
        &sequences,
        SequenceModeSearchConfig {
            ll_previous: None,
            ll_repeat_valid: false,
            ll_default: &ll_default,
            ml_previous: None,
            ml_repeat_valid: false,
            ml_default: &ml_default,
            of_previous: None,
            of_repeat_valid: false,
            of_default: &of_default,
            repeat_table_max_sequences: 0,
            llml_predefined_max_sequences: 0,
            of_predefined_max_sequences: 0,
            of_max_log: 8,
            exact_sequence_mode_search: false,
            c_fast_heuristics: false,
            c_cost_model: false,
        },
    );

    let FseTableMode::Encoded(table) = ll_mode else {
        panic!("forced encoded table");
    };
    let code_0 = literal_length_code(encoded_sequence(0, 3, 1).ll);
    let code_64 = literal_length_code(encoded_sequence(64, 3, 1).ll);
    let unadjusted_table = build_table_from_data(
        sequences
            .iter()
            .map(|sequence| literal_length_code(sequence.ll)),
        9,
        true,
    );

    assert_eq!(
        table.normalized_probability(code_0),
        table.normalized_probability(code_64)
    );
    assert_ne!(
        unadjusted_table.normalized_probability(code_0),
        unadjusted_table.normalized_probability(code_64)
    );
}

#[test]
fn encoded_sequence_table_log_uses_original_sequence_count() {
    let ll_default = default_ll_table();
    let ml_default = default_ml_table();
    let of_default = default_of_table();
    let mut lengths = Vec::new();
    lengths.extend(core::iter::repeat_n(0, 300));
    lengths.extend(core::iter::repeat_n(1, 60));
    lengths.extend(core::iter::repeat_n(2, 40));
    lengths.extend(core::iter::repeat_n(3, 30));
    lengths.extend(core::iter::repeat_n(4, 20));
    lengths.extend(core::iter::repeat_n(8, 20));
    lengths.extend(core::iter::repeat_n(20, 20));
    lengths.extend(core::iter::repeat_n(40, 22));
    lengths.push(0);
    let sequences = sequences_for_literal_lengths(lengths.iter().copied());

    let (ll_mode, _, _) = choose_sequence_table_modes(
        &sequences,
        SequenceModeSearchConfig {
            ll_previous: None,
            ll_repeat_valid: false,
            ll_default: &ll_default,
            ml_previous: None,
            ml_repeat_valid: false,
            ml_default: &ml_default,
            of_previous: None,
            of_repeat_valid: false,
            of_default: &of_default,
            repeat_table_max_sequences: 0,
            llml_predefined_max_sequences: 0,
            of_predefined_max_sequences: 0,
            of_max_log: 8,
            exact_sequence_mode_search: false,
            c_fast_heuristics: false,
            c_cost_model: false,
        },
    );

    let FseTableMode::Encoded(table) = ll_mode else {
        panic!("forced encoded table");
    };
    let max_code = sequences
        .iter()
        .map(|sequence| usize::from(literal_length_code(sequence.ll)))
        .max()
        .unwrap();
    let expected_log = crate::fse::fse_encoder::optimal_table_log(9, sequences.len(), max_code);
    let adjusted_log = crate::fse::fse_encoder::optimal_table_log(9, sequences.len() - 1, max_code);

    assert_ne!(expected_log, adjusted_log);
    assert_eq!(table.acc_log(), expected_log);
}

fn c_cost_config<'a>(
    ll_default: &'a crate::fse::fse_encoder::FSETable,
    ll_previous: Option<&'a crate::fse::fse_encoder::FSETable>,
    ml_default: &'a crate::fse::fse_encoder::FSETable,
    of_default: &'a crate::fse::fse_encoder::FSETable,
    of_previous: Option<&'a crate::fse::fse_encoder::FSETable>,
) -> SequenceModeSearchConfig<'a> {
    SequenceModeSearchConfig {
        ll_previous,
        ll_repeat_valid: false,
        ll_default,
        ml_previous: None,
        ml_repeat_valid: false,
        ml_default,
        of_previous,
        of_repeat_valid: false,
        of_default,
        repeat_table_max_sequences: 1000,
        llml_predefined_max_sequences: 56,
        of_predefined_max_sequences: 28,
        of_max_log: 8,
        exact_sequence_mode_search: false,
        c_fast_heuristics: false,
        c_cost_model: true,
    }
}

fn sequences_for_literal_lengths(
    lengths: impl IntoIterator<Item = u32>,
) -> Vec<crate::blocks::sequence_section::Sequence> {
    lengths
        .into_iter()
        .map(|ll| encoded_sequence(ll, 3, 1))
        .collect()
}

#[test]
fn exact_sequence_mode_search_never_worsens_threshold_choice() {
    let ll_default = default_ll_table();
    let ml_default = default_ml_table();
    let of_default = default_of_table();

    for a in 0..=8u32 {
        for b in 0..=8u32 {
            for c in 0..=8u32 {
                for d in 0..=8u32 {
                    let sequences = [
                        encoded_sequence(0, 3, 1u32 << a),
                        encoded_sequence(0, 3, 1u32 << b),
                        encoded_sequence(0, 3, 1u32 << c),
                        encoded_sequence(0, 3, 1u32 << d),
                    ];
                    let heuristic_ll = choose_table(
                        None,
                        &ll_default,
                        &sequences,
                        |seq| literal_length_code(seq.ll),
                        9,
                        64,
                        16,
                    );
                    let heuristic_ml = choose_table(
                        None,
                        &ml_default,
                        &sequences,
                        |seq| match_length_code(seq.ml),
                        9,
                        64,
                        16,
                    );
                    let heuristic_of = choose_table(
                        None,
                        &of_default,
                        &sequences,
                        |seq| offset_code(seq.of),
                        8,
                        64,
                        16,
                    );
                    let (ll_mode, ml_mode, of_mode) = choose_sequence_table_modes(
                        &sequences,
                        SequenceModeSearchConfig {
                            ll_previous: None,
                            ll_repeat_valid: false,
                            ll_default: &ll_default,
                            ml_previous: None,
                            ml_repeat_valid: false,
                            ml_default: &ml_default,
                            of_previous: None,
                            of_repeat_valid: false,
                            of_default: &of_default,
                            repeat_table_max_sequences: 64,
                            llml_predefined_max_sequences: 16,
                            of_predefined_max_sequences: 16,
                            of_max_log: 8,
                            exact_sequence_mode_search: true,
                            c_fast_heuristics: false,
                            c_cost_model: false,
                        },
                    );
                    let heuristic_size = exact_sequence_section_size(
                        &sequences,
                        &heuristic_ll,
                        &heuristic_ml,
                        &heuristic_of,
                    );
                    let exact_size =
                        exact_sequence_section_size(&sequences, &ll_mode, &ml_mode, &of_mode);
                    assert!(exact_size <= heuristic_size);
                }
            }
        }
    }
}
