use alloc::vec::Vec;

use super::{
    cctx_params::{CctxParameters, LdmParameters, ParamSwitch},
    ldm::{
        opt::{LdmOptCursor, LdmRawSeqStore},
        sequence::{
            fill_prefix_hash_table, generate_sequences_no_dict, generate_sequences_with_prefix,
            LdmRawSequence,
        },
        LdmEntry, LdmHashTable, LdmRollingHashState, LDM_BATCH_SIZE,
    },
    opt_match::{OptMatch, OptMatchTable},
};

fn small_ldm_params() -> LdmParameters {
    LdmParameters {
        enable_ldm: ParamSwitch::Enable,
        window_log: 10,
        hash_log: 6,
        min_match_length: 16,
        bucket_size_log: 3,
        hash_rate_log: 4,
    }
}

#[test]
fn ldm_hash_table_sizes_like_c() {
    let table = LdmHashTable::new(small_ldm_params());

    assert_eq!(table.table_len(), 64);
    assert_eq!(table.bucket_count(), 8);
    assert_eq!(table.bucket(5).len(), 8);
}

#[test]
fn ldm_hash_table_inserts_into_bucket_slots_like_c() {
    let mut table = LdmHashTable::new(small_ldm_params());

    table.insert_entry(
        5,
        LdmEntry {
            offset: 11,
            checksum: 101,
        },
    );
    table.insert_entry(
        5,
        LdmEntry {
            offset: 12,
            checksum: 102,
        },
    );
    table.insert_entry(
        2,
        LdmEntry {
            offset: 21,
            checksum: 201,
        },
    );

    assert_eq!(table.bucket_offset(5), 2);
    assert_eq!(table.bucket_offset(2), 1);
    assert_eq!(
        &table.bucket(5)[..3],
        &[
            LdmEntry {
                offset: 11,
                checksum: 101,
            },
            LdmEntry {
                offset: 12,
                checksum: 102,
            },
            LdmEntry::default(),
        ]
    );
    assert_eq!(
        table.bucket(2)[0],
        LdmEntry {
            offset: 21,
            checksum: 201,
        }
    );
}

#[test]
fn ldm_hash_table_wraps_bucket_offsets_like_c() {
    let mut table = LdmHashTable::new(small_ldm_params());

    for i in 0..10 {
        table.insert_entry(
            3,
            LdmEntry {
                offset: i,
                checksum: 100 + i,
            },
        );
    }

    assert_eq!(table.bucket_offset(3), 2);
    assert_eq!(
        &table.bucket(3)[..4],
        &[
            LdmEntry {
                offset: 8,
                checksum: 108,
            },
            LdmEntry {
                offset: 9,
                checksum: 109,
            },
            LdmEntry {
                offset: 2,
                checksum: 102,
            },
            LdmEntry {
                offset: 3,
                checksum: 103,
            },
        ]
    );
}

#[test]
fn ldm_gear_initializes_stop_mask_like_c() {
    let params = CctxParameters::for_level(22, 256 * 1024 * 1024, 0).ldm;
    let state = LdmRollingHashState::new(params);

    assert_eq!(state.rolling(), 0xffff_ffff);
    assert_eq!(state.stop_mask(), 0xf000_0000);
}

#[test]
fn ldm_gear_reset_preserves_rolling_like_c_1_5_7() {
    let params = CctxParameters::for_level(22, 256 * 1024 * 1024, 0).ldm;
    let mut state = LdmRollingHashState::new(params);
    let data: Vec<u8> = (0_u16..64).map(|i| (i * 13 + 5) as u8).collect();

    state.reset(&data, params.min_match_length as usize);

    assert_eq!(state.rolling(), 0xffff_ffff);
}

#[test]
fn ldm_gear_feed_finds_c_split_points() {
    let params = CctxParameters::for_level(22, 256 * 1024 * 1024, 0).ldm;
    let mut state = LdmRollingHashState::new(params);
    let data: Vec<u8> = (0_u16..512).map(|i| ((i * 37 + 11) & 0xff) as u8).collect();
    let mut splits = [0_usize; LDM_BATCH_SIZE];
    let mut num_splits = 0;

    let processed = state.feed(&data, &mut splits, &mut num_splits);

    assert_eq!(processed, 512);
    assert_eq!(num_splits, 25);
    assert_eq!(state.rolling(), 0x9e16_056e_bb3e_cce6);
    assert_eq!(
        &splits[..num_splits],
        &[
            7, 10, 69, 95, 120, 126, 130, 146, 176, 178, 192, 229, 252, 266, 325, 351, 376, 382,
            386, 402, 432, 434, 448, 485, 508,
        ]
    );
}

#[test]
fn ldm_no_dict_sequence_generation_matches_c_vector() {
    let params = LdmParameters {
        enable_ldm: ParamSwitch::Enable,
        window_log: 12,
        hash_log: 8,
        min_match_length: 16,
        bucket_size_log: 3,
        hash_rate_log: 3,
    };
    let mut data: Vec<u8> = (0_usize..2048)
        .map(|i| ((i * 37 + 11) & 0xff) as u8)
        .collect();
    for i in 0..512 {
        data[768 + i] = data[128 + i];
    }
    for i in 0..400 {
        data[1400 + i] = data[128 + i];
    }
    let mut table = LdmHashTable::new(params);

    let result = generate_sequences_no_dict(&data, params, &mut table);

    assert_eq!(result.last_literals, 0);
    assert_eq!(
        result.sequences,
        [
            LdmRawSequence {
                offset: 256,
                lit_length: 256,
                match_length: 512,
            },
            LdmRawSequence {
                offset: 640,
                lit_length: 0,
                match_length: 512,
            },
            LdmRawSequence {
                offset: 1024,
                lit_length: 0,
                match_length: 120,
            },
            LdmRawSequence {
                offset: 1272,
                lit_length: 0,
                match_length: 400,
            },
            LdmRawSequence {
                offset: 264,
                lit_length: 0,
                match_length: 248,
            },
        ]
    );
}

#[test]
fn ldm_prefix_sequence_generation_emits_source_relative_literals() {
    let params = LdmParameters {
        enable_ldm: ParamSwitch::Enable,
        window_log: 12,
        hash_log: 8,
        min_match_length: 16,
        bucket_size_log: 3,
        hash_rate_log: 0,
    };
    let dictionary: Vec<u8> = (0_usize..128)
        .map(|i| ((i * 37 + 11) & 0xff) as u8)
        .collect();
    let source = dictionary[32..96].to_vec();
    let mut combined = dictionary.clone();
    combined.extend_from_slice(&source);
    let mut table = LdmHashTable::new(params);

    fill_prefix_hash_table(&combined, 0..dictionary.len(), params, &mut table);
    let result = generate_sequences_with_prefix(
        &combined,
        dictionary.len()..combined.len(),
        params,
        &mut table,
    );

    assert!(!result.sequences.is_empty());
    assert_eq!(result.sequences[0].lit_length, 0);
    assert_eq!(result.sequences[0].offset, 96);
    assert_eq!(result.sequences[0].match_length, 64);
    assert_eq!(result.last_literals, 0);
}

#[test]
fn ldm_source_prefix_match_does_not_extend_back_into_dictionary() {
    let params = LdmParameters {
        enable_ldm: ParamSwitch::Enable,
        window_log: 12,
        hash_log: 8,
        min_match_length: 4,
        bucket_size_log: 3,
        hash_rate_log: 0,
    };
    let dictionary = b"dictionary-tail-D".to_vec();
    let source = b"YZABCDYZABCD-padding".to_vec();
    let mut combined = dictionary.clone();
    combined.extend_from_slice(&source);
    let source_start = dictionary.len();
    let mut table = LdmHashTable::new(params);

    fill_prefix_hash_table(&combined, 0..source_start, params, &mut table);
    let result =
        generate_sequences_with_prefix(&combined, source_start..combined.len(), params, &mut table);

    assert!(
        result.sequences.iter().any(|sequence| {
            sequence.lit_length == 6 && sequence.match_length == 6 && sequence.offset == 6
        }),
        "{:?}",
        result.sequences
    );
}

#[test]
fn ldm_chunking_prepends_previous_leftover_literals_like_c() {
    let params = LdmParameters {
        enable_ldm: ParamSwitch::Enable,
        window_log: 22,
        hash_log: 10,
        min_match_length: 16,
        bucket_size_log: 3,
        hash_rate_log: 0,
    };
    let chunk_size = 1 << 20;
    let mut data = Vec::with_capacity(chunk_size + 96);
    let mut value = 0x1234_5678_u32;
    for _ in 0..chunk_size {
        value ^= value << 13;
        value ^= value >> 17;
        value ^= value << 5;
        data.push(value as u8);
    }
    let repeat = data[chunk_size - 128..chunk_size - 32].to_vec();
    data.extend_from_slice(&repeat);
    let mut table = LdmHashTable::new(params);

    let result = generate_sequences_no_dict(&data, params, &mut table);

    assert!(!result.sequences.is_empty());
    assert!(
        result.sequences[0].lit_length >= chunk_size as u32,
        "{:?}",
        result.sequences.first()
    );
}

#[test]
fn ldm_raw_seq_store_skips_bytes_like_c_opt_ldm() {
    let sequences = [
        LdmRawSequence {
            offset: 10,
            lit_length: 3,
            match_length: 5,
        },
        LdmRawSequence {
            offset: 20,
            lit_length: 2,
            match_length: 4,
        },
    ];
    let mut store = LdmRawSeqStore::new(&sequences);

    store.skip_bytes(2);
    assert_eq!(store.position(), (0, 2));

    store.skip_bytes(1);
    assert_eq!(store.position(), (0, 3));

    store.skip_bytes(5);
    assert_eq!(store.position(), (1, 0));

    store.skip_bytes(3);
    assert_eq!(store.position(), (1, 3));

    store.skip_bytes(3);
    assert_eq!(store.position(), (2, 0));
}

#[test]
fn ldm_opt_cursor_exposes_current_match_like_c() {
    let sequences = [LdmRawSequence {
        offset: 100,
        lit_length: 4,
        match_length: 10,
    }];
    let cursor = LdmOptCursor::new(&sequences, 20);

    assert_eq!(cursor.current_match(), Some((4, 14, 100)));
    assert_eq!(cursor.seq_store_position(), (1, 0));
}

#[test]
fn ldm_opt_cursor_truncates_match_at_block_end_like_c() {
    let sequences = [LdmRawSequence {
        offset: 100,
        lit_length: 2,
        match_length: 10,
    }];
    let mut cursor = LdmOptCursor::new(&sequences, 6);
    let mut matches = OptMatchTable::new();

    assert_eq!(cursor.current_match(), Some((2, 6, 100)));
    assert_eq!(cursor.seq_store_position(), (0, 6));

    cursor.process_match_candidate(&mut matches, 2, 4, 4);

    assert_eq!(
        matches.as_slice(),
        [OptMatch {
            off_base: 103,
            len: 4,
        }]
    );
}

#[test]
fn ldm_raw_seq_store_skips_block_after_opt_parser_like_c() {
    let sequences = [LdmRawSequence {
        offset: 100,
        lit_length: 2,
        match_length: 10,
    }];
    let mut store = LdmRawSeqStore::new(&sequences);

    let first = LdmOptCursor::from_store_for_block(store, 6);
    assert_eq!(first.current_match(), Some((2, 6, 100)));
    assert_eq!(store.position(), (0, 0));

    store.skip_bytes(6);
    assert_eq!(store.position(), (0, 6));

    let second = LdmOptCursor::from_store_for_block(store, 6);
    assert_eq!(second.current_match(), Some((0, 6, 100)));
    assert_eq!(second.seq_store_position(), (1, 0));
}

#[test]
fn ldm_opt_cursor_adds_candidates_only_when_ordered_like_c() {
    let sequences = [LdmRawSequence {
        offset: 100,
        lit_length: 4,
        match_length: 10,
    }];
    let mut cursor = LdmOptCursor::new(&sequences, 20);
    let mut matches = OptMatchTable::new();

    cursor.process_match_candidate(&mut matches, 4, 16, 4);
    cursor.process_match_candidate(&mut matches, 8, 12, 4);

    assert_eq!(
        matches.as_slice(),
        [OptMatch {
            off_base: 103,
            len: 10,
        }]
    );

    let mut shorter_existing = OptMatchTable::new();
    shorter_existing.push(OptMatch {
        off_base: 7,
        len: 5,
    });
    cursor.process_match_candidate(&mut shorter_existing, 8, 12, 4);

    assert_eq!(
        shorter_existing.as_slice(),
        [
            OptMatch {
                off_base: 7,
                len: 5,
            },
            OptMatch {
                off_base: 103,
                len: 6,
            },
        ]
    );
}
