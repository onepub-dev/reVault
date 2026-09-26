use core::convert::TryInto;

use super::*;

#[test]
fn suffix_store_slot_stays_compact() {
    assert_eq!(core::mem::size_of::<Candidates>(), 8);
    assert_eq!(core::mem::size_of::<Option<Candidates>>(), 8);
}

#[test]
fn suffix_store_reports_single_candidate_once() {
    let mut suffixes = SuffixStore::with_capacity(64);

    suffixes.insert(b"abcde", 7);

    let candidates = suffixes
        .candidates(b"abcde")
        .expect("candidate should exist");
    assert_eq!(candidates.oldest, 7);
    assert_eq!(candidates.newest, None);
}

#[test]
fn suffix_store_round_trips_zero_index_candidate() {
    let mut suffixes = SuffixStore::with_capacity(64);

    suffixes.insert(b"abcde", 0);

    let candidates = suffixes
        .candidates(b"abcde")
        .expect("zero index candidate should exist");
    assert_eq!(candidates.oldest, 0);
    assert_eq!(candidates.newest, None);
}

#[test]
fn suffix_store_preserves_oldest_and_latest_candidates() {
    let mut suffixes = SuffixStore::with_capacity(64);

    suffixes.insert(b"abcde", 3);
    suffixes.insert(b"abcde", 8);
    suffixes.insert(b"abcde", 15);

    let candidates = suffixes
        .candidates(b"abcde")
        .expect("candidate should exist");
    assert_eq!(candidates.oldest, 3);
    assert_eq!(candidates.newest, Some(15));
}

#[test]
fn suffix_store_clear_removes_touched_candidates() {
    let mut suffixes = SuffixStore::with_capacity(64);

    suffixes.insert(b"abcde", 3);
    suffixes.insert(b"fghij", 8);
    suffixes.clear();

    assert!(suffixes.candidates(b"abcde").is_none());
    assert!(suffixes.candidates(b"fghij").is_none());
}

#[test]
fn suffix_store_reuses_slots_after_clear() {
    let mut suffixes = SuffixStore::with_capacity(64);

    suffixes.insert(b"abcde", 3);
    suffixes.clear();
    suffixes.insert(b"abcde", 9);

    let candidates = suffixes
        .candidates(b"abcde")
        .expect("candidate should exist after reinsertion");
    assert_eq!(candidates.oldest, 9);
    assert_eq!(candidates.newest, None);
}

#[test]
fn suffix_store_full_clear_mode_removes_untracked_candidates() {
    let mut suffixes = SuffixStore::with_capacity(64);
    let stored = NonZeroU32::new(1).expect("one is non-zero");
    suffixes.slots[1] = Some(Candidates {
        oldest: stored,
        newest: stored,
    });
    suffixes.clear_all_slots = true;

    suffixes.clear();

    assert!(suffixes.slots.iter().all(Option::is_none));
    assert!(!suffixes.clear_all_slots);
    assert!(suffixes.touched_slots.is_empty());
}

#[test]
fn suffix_store_switches_to_full_clear_after_many_touched_slots() {
    let mut suffixes = SuffixStore::with_capacity(TOUCHED_SLOT_CLEAR_LIMIT + 1);

    for key in 0..TOUCHED_SLOT_CLEAR_LIMIT {
        suffixes.record_touched_slot(key);
    }
    assert!(!suffixes.clear_all_slots);

    suffixes.record_touched_slot(TOUCHED_SLOT_CLEAR_LIMIT);

    assert!(suffixes.clear_all_slots);
    assert!(suffixes.touched_slots.is_empty());
}

#[test]
fn suffix_store_preallocates_touched_slots_modestly() {
    let suffixes = SuffixStore::with_capacity(64);

    assert_eq!(
        suffixes.touched_slots.capacity(),
        INITIAL_TOUCHED_SLOT_CAPACITY
    );
    assert!(suffixes.touched_slots.capacity() < TOUCHED_SLOT_CLEAR_LIMIT);
}

#[test]
fn suffix_store_handles_zero_capacity_request() {
    let mut suffixes = SuffixStore::with_capacity(0);

    suffixes.insert(b"abcde", 0);

    let candidates = suffixes
        .candidates(b"abcde")
        .expect("candidate should exist with minimum backing storage");
    assert_eq!(candidates.oldest, 0);
    assert_eq!(candidates.newest, None);
}

#[test]
fn suffix_store_key_is_bounded_without_modulo() {
    let suffixes = SuffixStore::with_capacity(100);

    for suffix in [
        b"abcde".as_slice(),
        b"vwxyz".as_slice(),
        b"12345".as_slice(),
        b"\xff\xff\xff\xff\xff".as_slice(),
    ] {
        assert!(suffixes.key(suffix) < suffixes.slots.len());
    }
}

#[test]
fn suffix_store_reuses_precomputed_key_values_for_lookup() {
    let mut suffixes = SuffixStore::with_capacity(64);

    suffixes.insert(b"abcde", 11);
    let key_value = SuffixStore::key_value(b"abcde");
    let candidates = suffixes
        .candidates_for_key_value(key_value)
        .expect("candidate should exist for precomputed key");

    assert_eq!(suffixes.key(b"abcde"), suffixes.key_from_value(key_value));
    assert_eq!(candidates.oldest, 11);
    assert_eq!(candidates.newest, None);
    assert!(suffixes
        .candidates_for_key_value(SuffixStore::key_value(b"vwxyz"))
        .is_none());
}

#[test]
fn suffix_store_reuses_precomputed_slot_keys_for_lookup() {
    let mut suffixes = SuffixStore::with_capacity(64);

    suffixes.insert(b"abcde", 11);
    let slot_key = suffixes.slot_key(SuffixStore::key_value(b"abcde"));
    let candidates = suffixes
        .candidates_for_slot_key(slot_key)
        .expect("candidate should exist for precomputed slot key");

    assert_eq!(candidates.oldest, 11);
    assert_eq!(candidates.newest, None);
}

#[test]
fn suffix_store_slot_keys_are_not_reused_across_different_capacities() {
    let mut small = SuffixStore::with_capacity(64);
    let mut large = SuffixStore::with_capacity(128);

    small.insert(b"abcde", 11);
    large.insert(b"abcde", 29);

    assert_ne!(small.len_log, large.len_log);

    let large_candidates = large
        .candidates_for_key_value(SuffixStore::key_value(b"abcde"))
        .expect("capacity-aware lookup should still find candidate");

    assert_eq!(large_candidates.oldest, 29);
    assert_eq!(large_candidates.newest, None);
}

#[test]
fn match_len_extends_overlapping_same_block() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        alloc::vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    let last_entry = matcher.last_entry();
    let context = MatchCandidateContext {
        suffix_idx: 1,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data[1..],
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };

    assert_eq!(matcher.match_len_at_offset(1, &context), 9);
}

#[test]
fn match_len_stops_at_chunk_boundary_mismatch() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        b"abcdefghabcdefghZ".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    let last_entry = matcher.last_entry();
    let context = MatchCandidateContext {
        suffix_idx: 8,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data[8..],
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };

    assert_eq!(matcher.match_len_at_offset(8, &context), 8);
}

#[test]
fn match_len_reads_from_previous_window_entry() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        b"prefix_MATCHTAIL".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    matcher.skip_matching();
    matcher.add_data(
        b"MATCHTAILx".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    let last_entry = matcher.last_entry();
    let context = MatchCandidateContext {
        suffix_idx: 0,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data,
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };

    assert_eq!(matcher.match_len_at_offset(b"MATCHTAIL".len(), &context), 9);
}

#[test]
fn verified_min_match_prefix_skips_rechecked_bytes() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        b"abcdefghabcdefghZ".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    let last_entry = matcher.last_entry();
    let context = MatchCandidateContext {
        suffix_idx: 8,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data[8..],
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };

    assert_eq!(
        matcher.verified_min_match_prefix_len(8, &context),
        Some(MIN_MATCH_LEN)
    );
    assert_eq!(
        matcher.match_len_at_offset_with_prefix(8, &context, MIN_MATCH_LEN),
        matcher.match_len_at_offset(8, &context)
    );
}

#[test]
fn same_block_min_match_precheck_handles_hits_and_misses() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        b"abcdeXabcdeYabcdnZ".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    let last_entry = matcher.last_entry();
    let matching_context = MatchCandidateContext {
        suffix_idx: 6,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data[6..],
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };
    let mismatching_context = MatchCandidateContext {
        suffix_idx: 12,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data[12..],
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };

    assert_eq!(
        matcher.verified_min_match_prefix_len(6, &matching_context),
        Some(MIN_MATCH_LEN)
    );
    assert!(matcher.has_min_match_at_index_offset(6, 6));
    assert_eq!(
        matcher.verified_min_match_prefix_len(6, &mismatching_context),
        None
    );
    assert!(!matcher.has_min_match_at_index_offset(12, 6));
}

#[test]
fn same_block_prefix_fast_path_handles_overlap() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        alloc::vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    let last_entry = matcher.last_entry();
    let context = MatchCandidateContext {
        suffix_idx: 1,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data[1..],
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };

    assert_eq!(
        matcher.match_len_at_offset_with_prefix(1, &context, MIN_MATCH_LEN),
        matcher.match_len_at_offset(1, &context)
    );
}

#[test]
fn short_previous_entry_prefix_falls_back_to_full_match_scan() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(b"ATCH".to_vec(), SuffixStore::with_capacity(100), |_, _| {});
    matcher.skip_matching();
    matcher.add_data(
        b"MATCHTAIL".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    let last_entry = matcher.last_entry();
    let context = MatchCandidateContext {
        suffix_idx: 0,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data,
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };

    assert_eq!(matcher.verified_min_match_prefix_len(1, &context), Some(0));
    assert_eq!(
        matcher.match_len_at_offset_with_prefix(1, &context, 0),
        matcher.match_len_at_offset(1, &context)
    );
}

#[test]
fn match_len_reads_from_most_recent_previous_window_entry() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        b"older_MATCH".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    matcher.skip_matching();
    matcher.add_data(
        b"recent_MATCH".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    matcher.skip_matching();
    matcher.add_data(
        b"MATCH!".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    let last_entry = matcher.last_entry();
    let context = MatchCandidateContext {
        suffix_idx: 0,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data,
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };

    assert_eq!(matcher.match_len_at_offset(b"MATCH".len(), &context), 5);
}

#[test]
fn repeat_offset_precheck_rejects_obvious_miss() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        b"abcde-----vwxyz".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    let last_entry = matcher.last_entry();
    let context = MatchCandidateContext {
        suffix_idx: 10,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data[10..],
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };

    assert!(!matcher.has_min_match_at_offset(10, &context));
}

#[test]
fn repeat_offset_precheck_accepts_candidate_match() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        b"abcde-----abcde".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    let last_entry = matcher.last_entry();
    let context = MatchCandidateContext {
        suffix_idx: 10,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data[10..],
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };

    assert!(matcher.has_min_match_at_offset(10, &context));
}

#[test]
fn hash_candidate_precheck_rejects_collision() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        b"abcde-----vwxyz".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    let last_entry = matcher.last_entry();
    let context = MatchCandidateContext {
        suffix_idx: 10,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data[10..],
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };

    assert!(!MatchGenerator::has_min_match_at_index(
        last_entry, 0, &context
    ));
}

#[test]
fn hash_candidate_precheck_accepts_candidate_match() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        b"abcde-----abcde".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    let last_entry = matcher.last_entry();
    let context = MatchCandidateContext {
        suffix_idx: 10,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data[10..],
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };

    assert!(MatchGenerator::has_min_match_at_index(
        last_entry, 0, &context
    ));
}

#[test]
fn window_candidate_helper_updates_best_candidate() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        b"abcdefghabcdefghZ".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    let last_entry = matcher.last_entry();
    let context = MatchCandidateContext {
        suffix_idx: 8,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data[8..],
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };
    let mut candidate = Some(MatchCandidate {
        start_idx: 8,
        offset: 16,
        match_len: MIN_MATCH_LEN,
        repeat_offset: false,
        #[cfg(test)]
        source: CandidateSource::WindowCurrentNewest { entry_distance: 0 },
    });

    assert!(!matcher.consider_window_candidate(
        last_entry,
        0,
        &context,
        WindowCandidateMeta {
            entry_distance: 0,
            kind: WindowCandidateKind::Newest,
        },
        &mut candidate,
        last_entry.data.len(),
    ));
    assert_eq!(candidate.map(|candidate| candidate.match_len), Some(8));
}

#[test]
fn window_candidate_helper_stops_on_block_end_match() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        alloc::vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    let last_entry = matcher.last_entry();
    let context = MatchCandidateContext {
        suffix_idx: 2,
        anchor_idx: 0,
        min_non_repeat_match_len: MIN_MATCH_LEN,
        data_slice: &last_entry.data[2..],
        #[cfg(debug_assertions)]
        last_entry_len: last_entry.data.len(),
        #[cfg(debug_assertions)]
        concat_window: &matcher.concat_window,
    };
    let mut candidate = None;

    assert!(matcher.consider_window_candidate(
        last_entry,
        0,
        &context,
        WindowCandidateMeta {
            entry_distance: 0,
            kind: WindowCandidateKind::Newest,
        },
        &mut candidate,
        last_entry.data.len(),
    ));
    assert_eq!(candidate.map(|candidate| candidate.offset), Some(2));
}

#[test]
fn window_search_uses_newest_block_end_candidate() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        b"abcde?".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    matcher.skip_matching();
    matcher.add_data(
        b"abcde!".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    matcher.skip_matching();
    matcher.add_data(
        b"abcde!".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    matcher.next_sequence(|seq| {
        assert_eq!(
            seq,
            Sequence::Triple {
                literals: &[],
                offset: 6,
                match_len: 6,
            },
        );
    });
    assert!(!matcher.next_sequence(|_| {}));
}

#[test]
fn repeat_offset_probe_finds_match_without_suffix_index() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        b"MATCHTAIL".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    matcher.skip_matching_for_incompressible();
    matcher.offset_history = OffsetHistory::from_offsets(10, 4, 8);
    matcher.add_data(
        b"xMATCHTAIL".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    matcher.next_sequence(|seq| {
        assert_eq!(
            seq,
            Sequence::Triple {
                literals: b"x",
                offset: 10,
                match_len: 9,
            },
        );
    });
    assert!(!matcher.next_sequence(|_| {}));
    assert_eq!(matcher.offset_history.as_offsets(), (10, 4, 8));
}

#[test]
fn repeat_offset_candidates_keep_history_order_after_literals() {
    let mut matcher = MatchGenerator::new(100);
    matcher.offset_history = OffsetHistory::from_offsets(7, 11, 13);

    assert_eq!(
        matcher.repeat_offset_candidates(3),
        [
            (RepeatCandidateKind::First, 7),
            (RepeatCandidateKind::Second, 11),
            (RepeatCandidateKind::Third, 13),
        ]
    );
}

#[test]
fn repeat_offset_candidates_shift_for_zero_literals() {
    let mut matcher = MatchGenerator::new(100);
    matcher.offset_history = OffsetHistory::from_offsets(7, 11, 13);

    assert_eq!(
        matcher.repeat_offset_candidates(0),
        [
            (RepeatCandidateKind::Second, 11),
            (RepeatCandidateKind::Third, 13),
            (RepeatCandidateKind::First, 6),
        ]
    );
}

#[test]
fn match_candidate_extends_backwards_to_anchor() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        b"XabcdeXabcde".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    matcher.next_sequence(|seq| {
        assert_eq!(
            seq,
            Sequence::Triple {
                literals: b"Xabcde",
                offset: 6,
                match_len: 6,
            },
        );
    });
    assert!(!matcher.next_sequence(|_| {}));
}

#[test]
fn small_text_blocks_use_shorter_non_repeat_match_minimum() {
    let mut matcher = MatchGenerator::new(2048);
    matcher.add_data(
        b"tenant=alpha path=/v1/archive status=200\n"
            .repeat(32)
            .to_vec(),
        SuffixStore::with_capacity(2048),
        |_, _| {},
    );

    assert_eq!(
        matcher.min_non_repeat_match_len,
        SMALL_TEXT_MIN_NON_REPEAT_MATCH_LEN
    );
}

#[test]
fn large_short_line_text_blocks_use_shorter_non_repeat_match_minimum() {
    let mut matcher = MatchGenerator::new(512 * 1024);
    matcher.add_data(
        b"let alpha = archive_status(path, tenant);\n"
            .repeat((128 * 1024 / 40) + 512)
            .to_vec(),
        SuffixStore::with_capacity(512 * 1024),
        |_, _| {},
    );

    assert_eq!(
        matcher.min_non_repeat_match_len,
        CODE_LIKE_SHORT_TEXT_MIN_NON_REPEAT_MATCH_LEN
    );
}

#[test]
fn large_long_line_text_blocks_keep_retained_non_repeat_match_minimum() {
    let mut matcher = MatchGenerator::new(512 * 1024);
    matcher.add_data(
        b"{\"tenant\":\"alpha\",\"path\":\"/v1/archive\",\"status\":200,\"request_id\":\"0123456789abcdef0123456789abcdef\",\"message\":\"structured-log-line\"}\n"
            .repeat((128 * 1024 / 120) + 512)
            .to_vec(),
        SuffixStore::with_capacity(512 * 1024),
        |_, _| {},
    );

    assert_eq!(
        matcher.min_non_repeat_match_len,
        TEXT_MIN_NON_REPEAT_MATCH_LEN
    );
}

#[test]
fn large_short_line_config_text_keeps_smaller_non_repeat_match_minimum() {
    let mut matcher = MatchGenerator::new(512 * 1024);
    matcher.add_data(
        b"Description=Login Service\nProtectSystem=strict\nReadWritePaths=/etc /run\n"
            .repeat((128 * 1024 / 72) + 512)
            .to_vec(),
        SuffixStore::with_capacity(512 * 1024),
        |_, _| {},
    );

    assert_eq!(
        matcher.min_non_repeat_match_len,
        SMALL_TEXT_MIN_NON_REPEAT_MATCH_LEN
    );
}

#[test]
fn small_short_line_config_text_keeps_smaller_non_repeat_match_minimum() {
    let mut matcher = MatchGenerator::new(8 * 1024);
    matcher.add_data(
        b"Description=Login Service\nProtectSystem=strict\nReadWritePaths=/etc /run\n"
            .repeat(8)
            .to_vec(),
        SuffixStore::with_capacity(8 * 1024),
        |_, _| {},
    );

    assert_eq!(
        matcher.min_non_repeat_match_len,
        SMALL_TEXT_MIN_NON_REPEAT_MATCH_LEN
    );
}

#[test]
fn binary_blocks_keep_short_non_repeat_matches() {
    let mut matcher = MatchGenerator::new(2048);
    matcher.add_data(xorshift(2048), SuffixStore::with_capacity(2048), |_, _| {});

    assert_eq!(matcher.min_non_repeat_match_len, MIN_MATCH_LEN);
}

#[test]
fn short_line_text_blocks_use_denser_no_match_probe_step() {
    let mut matcher = MatchGenerator::new(512 * 1024);
    matcher.add_data(
        b"Description=Login Service\nProtectSystem=strict\nReadWritePaths=/etc /run\n"
            .repeat((128 * 1024 / 72) + 512)
            .to_vec(),
        SuffixStore::with_capacity(512 * 1024),
        |_, _| {},
    );

    assert_eq!(
        matcher.no_match_probe_step(0),
        SHORT_LINE_TEXT_NO_MATCH_PROBE_STEP
    );
}

#[test]
fn long_line_text_blocks_keep_wider_no_match_probe_step() {
    let mut matcher = MatchGenerator::new(512 * 1024);
    matcher.add_data(
        b"{\"tenant\":\"alpha\",\"path\":\"/v1/archive\",\"status\":200,\"request_id\":\"0123456789abcdef0123456789abcdef\",\"message\":\"structured-log-line\"}\n"
            .repeat((128 * 1024 / 120) + 512)
            .to_vec(),
        SuffixStore::with_capacity(512 * 1024),
        |_, _| {},
    );

    assert_eq!(matcher.no_match_probe_step(0), TEXT_NO_MATCH_PROBE_STEP);
}

#[test]
fn binary_blocks_keep_default_no_match_probe_step() {
    let mut matcher = MatchGenerator::new(2048);
    matcher.add_data(xorshift(2048), SuffixStore::with_capacity(2048), |_, _| {});

    assert_eq!(matcher.no_match_probe_step(0), NO_MATCH_PROBE_STEP);
}

#[test]
fn empty_committed_entry_has_no_sequences() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(Vec::new(), SuffixStore::with_capacity(100), |_, _| {});

    assert!(!matcher.next_sequence(|_| panic!("empty entry should not emit sequences")));

    matcher.add_data(
        b"abcdeabcde".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    assert!(matcher.next_sequence(|_| {}));
}

#[test]
fn driver_reuses_short_frame_suffix_store_for_larger_frame() {
    let mut matcher = MatchGeneratorDriver::new(32, 2);
    matcher.commit_space(b"a".to_vec());
    matcher.skip_matching_for_rle();
    matcher.reset(CompressionLevel::Fastest);

    matcher.commit_space(b"abcdeabcdeabcde".to_vec());

    let mut emitted_sequence = false;
    matcher.start_matching(|_| emitted_sequence = true);
    assert!(emitted_sequence);
}

#[test]
fn driver_uses_c_fast_sized_suffix_store() {
    let mut matcher = MatchGeneratorDriver::new(128, 2);

    matcher.commit_space(b"abcdeabcde".to_vec());

    assert_eq!(
        matcher.match_generator.last_entry().suffixes.slots.len(),
        128 / SUFFIX_STORE_CAPACITY_DIVISOR
    );
}

#[test]
fn driver_uses_larger_suffix_store_for_best_level() {
    let mut matcher = MatchGeneratorDriver::new(128, 2);

    for (level, expected_capacity) in [
        (
            CompressionLevel::Fastest,
            128 / SUFFIX_STORE_CAPACITY_DIVISOR,
        ),
        (
            CompressionLevel::Default,
            128 / SUFFIX_STORE_CAPACITY_DIVISOR,
        ),
        (
            CompressionLevel::Better,
            128 / SUFFIX_STORE_CAPACITY_DIVISOR,
        ),
        (
            CompressionLevel::Best,
            128 * BEST_SUFFIX_STORE_CAPACITY_MULTIPLIER,
        ),
    ] {
        matcher.reset(level);
        matcher.commit_space(b"abcdeabcde".to_vec());

        assert_eq!(
            matcher.match_generator.last_entry().suffixes.slots.len(),
            expected_capacity,
            "{level:?} should use its configured suffix table size"
        );
        matcher.skip_matching();
    }
}

#[test]
fn driver_uses_larger_window_for_best_level() {
    let mut matcher = MatchGeneratorDriver::new(128, 2);

    matcher.reset(CompressionLevel::Fastest);
    assert_eq!(matcher.window_size(), (128 * FASTEST_WINDOW_BLOCKS) as u64);

    matcher.reset(CompressionLevel::Default);
    assert_eq!(matcher.window_size(), (128 * FASTEST_WINDOW_BLOCKS) as u64);

    matcher.reset(CompressionLevel::Better);
    assert_eq!(matcher.window_size(), (128 * FASTEST_WINDOW_BLOCKS) as u64);

    matcher.reset(CompressionLevel::Best);
    assert_eq!(matcher.window_size(), (128 * BEST_WINDOW_BLOCKS) as u64);
}

#[test]
fn driver_enables_adaptive_binary_no_match_probe_for_best_level() {
    let mut matcher = MatchGeneratorDriver::new(128, 2);

    matcher.reset(CompressionLevel::Fastest);
    assert!(!matcher.match_generator.adaptive_binary_no_match_probe);

    matcher.reset(CompressionLevel::Best);
    assert!(matcher.match_generator.adaptive_binary_no_match_probe);
}

#[test]
fn driver_enables_fast_small_dense_binary_probe_for_fastest_level() {
    let mut matcher = MatchGeneratorDriver::new(128, 2);

    matcher.reset(CompressionLevel::Fastest);
    assert!(matcher.match_generator.use_fast_small_dense_binary_probe);
    assert!(!matcher.match_generator.adaptive_binary_no_match_probe);

    matcher.reset(CompressionLevel::Best);
    assert!(!matcher.match_generator.use_fast_small_dense_binary_probe);
}

#[test]
fn driver_enables_next_position_repeat_lookahead_for_best_level() {
    let mut matcher = MatchGeneratorDriver::new(128, 2);

    matcher.reset(CompressionLevel::Fastest);
    assert!(
        !matcher
            .match_generator
            .prefer_binary_next_position_repeat_lookahead
    );

    matcher.reset(CompressionLevel::Best);
    assert!(
        matcher
            .match_generator
            .prefer_binary_next_position_repeat_lookahead
    );
}

#[test]
fn driver_enables_fast_binary_next_position_repeat_lookahead_for_fastest_level() {
    let mut matcher = MatchGeneratorDriver::new(128, 2);

    matcher.reset(CompressionLevel::Fastest);
    assert!(
        matcher
            .match_generator
            .prefer_fast_binary_next_position_repeat_lookahead
    );

    matcher.reset(CompressionLevel::Best);
    assert!(
        !matcher
            .match_generator
            .prefer_fast_binary_next_position_repeat_lookahead
    );
}

#[test]
fn driver_enables_next_position_lookahead_for_best_level() {
    let mut matcher = MatchGeneratorDriver::new(128, 2);

    matcher.reset(CompressionLevel::Fastest);
    assert!(
        !matcher
            .match_generator
            .prefer_binary_next_position_lookahead
    );

    matcher.reset(CompressionLevel::Best);
    assert!(
        matcher
            .match_generator
            .prefer_binary_next_position_lookahead
    );
}

#[test]
fn driver_enables_oldest_first_window_probe_for_best_level() {
    let mut matcher = MatchGeneratorDriver::new(128, 2);

    matcher.reset(CompressionLevel::Fastest);
    assert!(!matcher.match_generator.prefer_oldest_first_window_probe);

    matcher.reset(CompressionLevel::Best);
    assert!(matcher.match_generator.prefer_oldest_first_window_probe);
}

#[test]
fn driver_enables_complementary_end_insertion_for_best_level() {
    let mut matcher = MatchGeneratorDriver::new(128, 2);

    matcher.reset(CompressionLevel::Fastest);
    assert!(!matcher.match_generator.use_complementary_end_insertion);

    matcher.reset(CompressionLevel::Best);
    assert!(matcher.match_generator.use_complementary_end_insertion);
}

#[test]
fn driver_enables_second_newest_probe_for_best_level() {
    let mut matcher = MatchGeneratorDriver::new(128, 2);

    matcher.reset(CompressionLevel::Fastest);
    assert!(!matcher.match_generator.use_second_newest_probe);

    matcher.reset(CompressionLevel::Best);
    assert!(matcher.match_generator.use_second_newest_probe);
}

#[test]
fn driver_enables_fast_binary_small_second_newest_for_fastest_level() {
    let mut matcher = MatchGeneratorDriver::new(128, 2);

    matcher.reset(CompressionLevel::Fastest);
    assert!(matcher.match_generator.use_fast_binary_small_second_newest);
    assert!(!matcher.match_generator.use_second_newest_probe);

    matcher.reset(CompressionLevel::Best);
    assert!(!matcher.match_generator.use_fast_binary_small_second_newest);
}

#[test]
fn driver_enables_text_repeat_pipeline_for_best_level() {
    let mut matcher = MatchGeneratorDriver::new(128, 2);

    matcher.reset(CompressionLevel::Fastest);
    assert!(!matcher.match_generator.use_text_repeat_pipeline);

    matcher.reset(CompressionLevel::Best);
    assert!(matcher.match_generator.use_text_repeat_pipeline);
}

#[test]
fn matcher_tracks_uniform_suffix_store_capacity() {
    let mut matcher = MatchGenerator::new(1024);

    matcher.add_data(b"abcde".to_vec(), SuffixStore::with_capacity(64), |_, _| {});
    assert_eq!(matcher.uniform_suffix_len_log, Some(6));

    matcher.skip_matching();
    matcher.add_data(b"vwxyz".to_vec(), SuffixStore::with_capacity(64), |_, _| {});
    assert_eq!(matcher.uniform_suffix_len_log, Some(6));
}

#[test]
fn matcher_disables_uniform_suffix_store_fast_path_for_mixed_capacities() {
    let mut matcher = MatchGenerator::new(1024);

    matcher.add_data(b"abcde".to_vec(), SuffixStore::with_capacity(64), |_, _| {});
    matcher.skip_matching();
    matcher.add_data(
        b"vwxyz".to_vec(),
        SuffixStore::with_capacity(128),
        |_, _| {},
    );

    assert_eq!(matcher.uniform_suffix_len_log, None);
}

#[test]
fn best_level_keeps_large_active_window_for_text_blocks_only() {
    let mut matcher = MatchGenerator::new(4 * 1024);
    matcher.set_window_sizes(4 * 1024, 2 * 1024);

    for _ in 0..3 {
        matcher.add_data(
            alloc::vec![b'a'; 1024],
            SuffixStore::with_capacity(128),
            |_, _| {},
        );
        matcher.skip_matching();
    }
    assert_eq!(matcher.window_size, 3 * 1024);

    matcher.reset(|_, _| {});
    matcher.set_window_sizes(4 * 1024, 2 * 1024);
    for _ in 0..3 {
        matcher.add_data(xorshift(1024), SuffixStore::with_capacity(128), |_, _| {});
        matcher.skip_matching_for_incompressible();
    }
    assert_eq!(matcher.window_size, 2 * 1024);
}

#[test]
fn repeat_offset_candidate_can_win_with_small_length_deficit() {
    let repeat = MatchCandidate {
        start_idx: 10,
        offset: 16,
        match_len: 8,
        repeat_offset: true,
        #[cfg(test)]
        source: CandidateSource::RepeatCurrent(RepeatCandidateKind::First),
    };
    let normal = MatchCandidate {
        start_idx: 10,
        offset: 1024,
        match_len: 8 + REPEAT_MATCH_LEN_MARGIN,
        repeat_offset: false,
        #[cfg(test)]
        source: CandidateSource::WindowCurrentNewest { entry_distance: 0 },
    };

    assert!(repeat.is_better_than(normal));
}

#[test]
fn dictionary_text_can_prefer_smaller_offset_at_same_start() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;

    let smaller_offset = MatchCandidate {
        start_idx: 10,
        offset: 29,
        match_len: 11,
        repeat_offset: false,
        #[cfg(test)]
        source: CandidateSource::WindowCurrentNewest { entry_distance: 0 },
    };
    let longer_farther = MatchCandidate {
        start_idx: 10,
        offset: 125,
        match_len: 12,
        repeat_offset: false,
        #[cfg(test)]
        source: CandidateSource::WindowCurrentOldest { entry_distance: 0 },
    };

    assert!(matcher.candidate_is_better_than(smaller_offset, longer_farther));
    assert!(!matcher.candidate_is_better_than(longer_farther, smaller_offset));
}

#[test]
fn composer_dictionary_text_can_prefer_repeat_kind_at_same_start() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.current_block_is_composer_dictionary_text = true;
    matcher.offset_history = OffsetHistory::from_offsets(20, 40, 60);
    matcher.suffix_idx = 100;
    matcher.last_idx_in_sequence = 100;

    let preferred = MatchCandidate {
        start_idx: 100,
        offset: 40,
        match_len: 10,
        repeat_offset: true,
        #[cfg(test)]
        source: CandidateSource::RepeatCurrent(RepeatCandidateKind::Second),
    };
    let other = MatchCandidate {
        start_idx: 100,
        offset: 19,
        match_len: 11,
        repeat_offset: true,
        #[cfg(test)]
        source: CandidateSource::RepeatCurrent(RepeatCandidateKind::First),
    };

    assert!(matcher.candidate_is_better_than(preferred, other));
    assert!(!matcher.candidate_is_better_than(other, preferred));
}

#[test]
fn lockfile_dictionary_text_can_prefer_repeat_kind_at_same_start() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.current_block_is_dictionary_lockfile_text = true;
    matcher.offset_history = OffsetHistory::from_offsets(20, 40, 60);
    matcher.suffix_idx = 100;
    matcher.last_idx_in_sequence = 100;

    let preferred = MatchCandidate {
        start_idx: 100,
        offset: 40,
        match_len: 10,
        repeat_offset: true,
        #[cfg(test)]
        source: CandidateSource::RepeatCurrent(RepeatCandidateKind::Second),
    };
    let other = MatchCandidate {
        start_idx: 100,
        offset: 19,
        match_len: 11,
        repeat_offset: true,
        #[cfg(test)]
        source: CandidateSource::RepeatCurrent(RepeatCandidateKind::First),
    };

    assert!(matcher.candidate_is_better_than(preferred, other));
    assert!(!matcher.candidate_is_better_than(other, preferred));
}

#[test]
fn lockfile_dictionary_text_can_prefer_smaller_offset_at_same_end() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.current_block_is_dictionary_lockfile_text = true;

    let smaller_offset = MatchCandidate {
        start_idx: 11,
        offset: 29,
        match_len: 11,
        repeat_offset: false,
        #[cfg(test)]
        source: CandidateSource::WindowCurrentSecondNewest { entry_distance: 0 },
    };
    let longer_farther = MatchCandidate {
        start_idx: 10,
        offset: 125,
        match_len: 12,
        repeat_offset: false,
        #[cfg(test)]
        source: CandidateSource::WindowCurrentOldest { entry_distance: 0 },
    };

    assert!(matcher.candidate_is_better_than(smaller_offset, longer_farther));
    assert!(!matcher.candidate_is_better_than(longer_farther, smaller_offset));
}

#[test]
fn longer_normal_candidate_wins_beyond_repeat_offset_margin() {
    let repeat = MatchCandidate {
        start_idx: 10,
        offset: 16,
        match_len: 8,
        repeat_offset: true,
        #[cfg(test)]
        source: CandidateSource::RepeatCurrent(RepeatCandidateKind::First),
    };
    let normal = MatchCandidate {
        start_idx: 10,
        offset: 1024,
        match_len: 8 + REPEAT_MATCH_LEN_MARGIN + 1,
        repeat_offset: false,
        #[cfg(test)]
        source: CandidateSource::WindowCurrentNewest { entry_distance: 0 },
    };

    assert!(normal.is_better_than(repeat));
}

#[test]
fn long_repeat_offset_candidate_skips_window_search() {
    let repeat = MatchCandidate {
        start_idx: 10,
        offset: 16,
        match_len: REPEAT_SEARCH_EARLY_EXIT_LEN,
        repeat_offset: true,
        #[cfg(test)]
        source: CandidateSource::RepeatCurrent(RepeatCandidateKind::First),
    };
    let normal = MatchCandidate {
        start_idx: 10,
        offset: 16,
        match_len: REPEAT_SEARCH_EARLY_EXIT_LEN,
        repeat_offset: false,
        #[cfg(test)]
        source: CandidateSource::WindowCurrentNewest { entry_distance: 0 },
    };

    assert!(repeat.can_skip_window_search(128));
    assert!(!normal.can_skip_window_search(128));
}

#[test]
fn repeat_offset_candidate_skips_window_search_at_block_end() {
    let repeat = MatchCandidate {
        start_idx: 10,
        offset: 16,
        match_len: MIN_MATCH_LEN,
        repeat_offset: true,
        #[cfg(test)]
        source: CandidateSource::RepeatCurrent(RepeatCandidateKind::First),
    };

    assert!(repeat.can_skip_window_search(10 + MIN_MATCH_LEN));
}

#[test]
fn no_match_step_does_not_skip_next_repeat_offset_match() {
    let mut matcher = MatchGenerator::new(100);
    matcher.add_data(
        b"MATCHTAIL".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    matcher.skip_matching_for_incompressible();
    matcher.offset_history = OffsetHistory::from_offsets(10, 4, 8);
    matcher.add_data(
        b"xMATCHTAIL".to_vec(),
        SuffixStore::with_capacity(100),
        |_, _| {},
    );

    assert!(matcher.repeat_offset_can_match_at(1, "MATCHTAIL".len()));
}

#[cfg(feature = "std")]
#[test]
#[ignore]
fn inspect_best_matcher_from_env() {
    use std::fs;
    use std::println;

    let fixture = std::env::var("RUZSTD_MATCHER_FIXTURE")
        .expect("set RUZSTD_MATCHER_FIXTURE to a fixture path");
    let input = fs::read(&fixture).expect("fixture must be readable");

    let mut matcher = MatchGeneratorDriver::new(128 * 1024, 4);
    matcher.reset(CompressionLevel::Best);
    if let Ok(file_type) = std::env::var("RUZSTD_MATCHER_FILE_TYPE") {
        let file_type = match file_type.as_str() {
            "unknown" => CompressionFileType::Unknown,
            "archive" => CompressionFileType::ArchiveLike,
            "binary" => CompressionFileType::BinaryLike,
            "code" => CompressionFileType::CodeText,
            "config" => CompressionFileType::ConfigText,
            "json" => CompressionFileType::JsonText,
            "dictionary" => CompressionFileType::DictionaryText,
            other => panic!("unsupported RUZSTD_MATCHER_FILE_TYPE={}", other),
        };
        matcher.set_file_type_hint(file_type);
    }

    let mut offset = 0usize;
    while offset < input.len() {
        let end = (offset + 128 * 1024).min(input.len());
        let chunk = &input[offset..end];
        let mut space = matcher.get_next_space();
        space[..chunk.len()].copy_from_slice(chunk);
        space.truncate(chunk.len());
        matcher.commit_space(space);
        matcher.start_matching(|_| {});
        offset = end;
    }

    println!("fixture: {fixture}");
    println!("{:#?}", matcher.diagnostics());
}

#[cfg(feature = "std")]
#[test]
#[ignore]
fn inspect_fastest_matcher_from_env() {
    use std::fs;
    use std::println;

    let fixture = std::env::var("RUZSTD_MATCHER_FIXTURE")
        .expect("set RUZSTD_MATCHER_FIXTURE to a fixture path");
    let input = fs::read(&fixture).expect("fixture must be readable");

    let mut matcher = MatchGeneratorDriver::new(128 * 1024, 4);
    matcher.reset(CompressionLevel::Fastest);
    if let Ok(file_type) = std::env::var("RUZSTD_MATCHER_FILE_TYPE") {
        let file_type = match file_type.as_str() {
            "unknown" => CompressionFileType::Unknown,
            "archive" => CompressionFileType::ArchiveLike,
            "binary" => CompressionFileType::BinaryLike,
            "code" => CompressionFileType::CodeText,
            "config" => CompressionFileType::ConfigText,
            "json" => CompressionFileType::JsonText,
            "dictionary" => CompressionFileType::DictionaryText,
            other => panic!("unsupported RUZSTD_MATCHER_FILE_TYPE={}", other),
        };
        matcher.set_file_type_hint(file_type);
    }

    let mut offset = 0usize;
    while offset < input.len() {
        let end = (offset + 128 * 1024).min(input.len());
        let chunk = &input[offset..end];
        let mut space = matcher.get_next_space();
        space[..chunk.len()].copy_from_slice(chunk);
        space.truncate(chunk.len());
        matcher.commit_space(space);
        matcher.start_matching(|_| {});
        offset = end;
    }

    println!("fixture: {fixture}");
    println!("{:#?}", matcher.diagnostics());
}

#[test]
fn adaptive_binary_no_match_probe_grows_with_literal_run() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.adaptive_binary_no_match_probe = true;

    assert_eq!(matcher.no_match_probe_step(0), NO_MATCH_PROBE_STEP);
    assert_eq!(matcher.no_match_probe_step(255), NO_MATCH_PROBE_STEP);
    assert_eq!(matcher.no_match_probe_step(256), NO_MATCH_PROBE_STEP + 1);
    assert_eq!(matcher.no_match_probe_step(512), NO_MATCH_PROBE_STEP + 2);
}

#[test]
fn adaptive_binary_no_match_probe_does_not_change_text_probe_step() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.adaptive_binary_no_match_probe = true;
    matcher.is_text_block = true;
    matcher.min_non_repeat_match_len = TEXT_MIN_NON_REPEAT_MATCH_LEN;

    assert_eq!(matcher.no_match_probe_step(1024), TEXT_NO_MATCH_PROBE_STEP);
}

#[test]
fn fast_dense_binary_probe_uses_dense_step_for_small_non_text() {
    let mut matcher = MatchGenerator::new(FASTEST_DENSE_BINARY_PROBE_MAX_BLOCK_LEN + 1);
    matcher.use_fast_small_dense_binary_probe = true;
    matcher.add_data(
        alloc::vec![0u8; FASTEST_DENSE_BINARY_PROBE_MAX_BLOCK_LEN],
        SuffixStore::with_capacity(64),
        |_, _| {},
    );

    assert_eq!(matcher.no_match_probe_step(0), 1);
}

#[test]
fn fast_dense_binary_probe_keeps_default_step_for_large_non_text() {
    let mut matcher = MatchGenerator::new(FASTEST_DENSE_BINARY_PROBE_MAX_BLOCK_LEN + 1);
    matcher.use_fast_small_dense_binary_probe = true;
    matcher.add_data(
        alloc::vec![0u8; FASTEST_DENSE_BINARY_PROBE_MAX_BLOCK_LEN + 1],
        SuffixStore::with_capacity(64),
        |_, _| {},
    );

    assert_eq!(matcher.no_match_probe_step(0), NO_MATCH_PROBE_STEP);
}

#[test]
fn medium_code_text_blocks_keep_dense_probe_step() {
    let mut matcher = MatchGenerator::new(CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN + 1);
    matcher.file_type_hint = CompressionFileType::CodeText;
    matcher.add_data(
        b"fn progress_step(value: usize) { println!(\"{}\", value); }\n"
            .repeat((9 * 1024 / 58) + 8)
            .to_vec(),
        SuffixStore::with_capacity(CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN + 1),
        |_, _| {},
    );

    assert!(matcher.is_short_line_text);
    assert!(matcher.last_entry().data.len() > CONFIG_TEXT_DENSE_PROBE_MAX_BLOCK_LEN);
    assert!(matcher.last_entry().data.len() <= CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN);
    assert_eq!(matcher.no_match_probe_step(0), 1);
}

#[test]
fn medium_config_text_blocks_keep_short_line_probe_step() {
    let mut matcher = MatchGenerator::new(CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN + 1);
    matcher.file_type_hint = CompressionFileType::ConfigText;
    matcher.add_data(
        b"Description=Progress Service\nProtectSystem=strict\nReadWritePaths=/run/progress\n"
            .repeat((9 * 1024 / 78) + 8)
            .to_vec(),
        SuffixStore::with_capacity(CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN + 1),
        |_, _| {},
    );

    assert!(matcher.is_short_line_text);
    assert!(matcher.last_entry().data.len() > CONFIG_TEXT_DENSE_PROBE_MAX_BLOCK_LEN);
    assert!(matcher.last_entry().data.len() <= CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN);
    assert_eq!(
        matcher.no_match_probe_step(0),
        SHORT_LINE_TEXT_NO_MATCH_PROBE_STEP
    );
}

#[test]
fn structured_json_config_text_is_detected() {
    let data = br#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext"
  },
  "include": [
    "src/**/*.ts"
  ],
  "exclude": [
    "dist"
  ]
}"#;

    assert!(MatchGenerator::likely_structured_json_config_text(data));
}

#[test]
fn medium_structured_json_config_text_blocks_keep_dense_probe_step() {
    let mut matcher = MatchGenerator::new(STRUCTURED_JSON_CONFIG_DENSE_PROBE_MAX_BLOCK_LEN + 1);
    matcher.file_type_hint = CompressionFileType::ConfigText;
    matcher.add_data(
        b"{\n  \"dependencies\": {\n    \"dep-0001\": \"^1.0.0\",\n    \"dep-0002\": \"^2.0.0\"\n  },\n  \"devDependencies\": {\n    \"dev-dep-0001\": \"~1.0.0\",\n    \"dev-dep-0002\": \"~2.0.0\"\n  },\n  \"scripts\": {\n    \"build\": \"vite build\",\n    \"test\": \"vitest run\"\n  }\n}\n"
            .repeat((96 * 1024 / 304) + 64)
            .to_vec(),
        SuffixStore::with_capacity(STRUCTURED_JSON_CONFIG_DENSE_PROBE_MAX_BLOCK_LEN + 1),
        |_, _| {},
    );

    assert!(matcher.is_short_line_text);
    assert!(matcher.current_block_is_structured_json_config_text);
    assert!(!matcher.current_block_is_tsconfig_json_config_text);
    assert!(matcher.last_entry().data.len() > CONFIG_TEXT_DENSE_PROBE_MAX_BLOCK_LEN);
    assert!(matcher.last_entry().data.len() <= STRUCTURED_JSON_CONFIG_DENSE_PROBE_MAX_BLOCK_LEN);
    assert_eq!(matcher.no_match_probe_step(0), 1);
}

#[test]
fn tsconfig_json_config_text_is_detected() {
    let data = br#"{
  "compilerOptions": {
    "module": "NodeNext",
    "paths": {
      "@feature/0000/*": [
        "./src/feature_0000/*"
      ]
    }
  },
  "include": [
    "src/**/*.ts"
  ],
  "exclude": [
    "dist"
  ]
}"#;

    assert!(MatchGenerator::likely_tsconfig_json_config_text(data));
}

#[test]
fn tsconfig_json_config_text_keeps_wider_probe_step() {
    let mut matcher = MatchGenerator::new(STRUCTURED_JSON_CONFIG_DENSE_PROBE_MAX_BLOCK_LEN + 1);
    matcher.file_type_hint = CompressionFileType::ConfigText;
    matcher.add_data(
        b"{\n  \"compilerOptions\": {\n    \"module\": \"NodeNext\",\n    \"paths\": {\n      \"@feature/0000/*\": [\"./src/feature_0000/*\"],\n      \"@feature/0001/*\": [\"./src/feature_0001/*\"]\n    }\n  },\n  \"include\": [\"src/**/*.ts\"],\n  \"exclude\": [\"dist\"]\n}\n"
            .repeat((96 * 1024 / 268) + 64)
            .to_vec(),
        SuffixStore::with_capacity(STRUCTURED_JSON_CONFIG_DENSE_PROBE_MAX_BLOCK_LEN + 1),
        |_, _| {},
    );

    assert!(matcher.is_short_line_text);
    assert!(matcher.current_block_is_structured_json_config_text);
    assert!(matcher.current_block_is_tsconfig_json_config_text);
    assert_eq!(
        matcher.no_match_probe_step(0),
        TSCONFIG_JSON_TEXT_NO_MATCH_PROBE_STEP
    );
}

#[test]
fn composer_dictionary_text_is_detected() {
    let chunk = br#"{
  "packages": [
    {
      "name": "vendor/package-0001",
      "require": {
        "php": ">=8.2"
      },
      "source": {
        "reference": "0000000000000000000000000000000000000001",
        "type": "git",
        "url": "https://example.com/vendor/package-0001.git"
      },
      "version": "1.0.0"
    },
    {
      "name": "vendor/package-0002",
      "require": {
        "php": ">=8.2"
      },
      "source": {
        "reference": "0000000000000000000000000000000000000002",
        "type": "git",
        "url": "https://example.com/vendor/package-0002.git"
      },
      "version": "1.0.1"
    }
  ]
}
"#;
    let data = chunk.repeat((16 * 1024 / chunk.len()) + 4);

    assert!(MatchGenerator::likely_composer_dictionary_text(&data));
}

#[test]
fn composer_dictionary_text_keeps_wider_probe_step() {
    let mut matcher = MatchGenerator::new(STRUCTURED_JSON_CONFIG_DENSE_PROBE_MAX_BLOCK_LEN + 1);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.add_data(
        b"{\n  \"packages\": [\n    {\n      \"name\": \"vendor/package-0001\",\n      \"require\": {\n        \"php\": \">=8.2\",\n        \"vendor/dependency-0002\": \"^2.0\"\n      },\n      \"source\": {\n        \"reference\": \"0000000000000000000000000000000000000001\",\n        \"type\": \"git\",\n        \"url\": \"https://example.com/vendor/package-0001.git\"\n      },\n      \"version\": \"1.0.0\"\n    }\n  ]\n}\n"
            .repeat((96 * 1024 / 420) + 96)
            .to_vec(),
        SuffixStore::with_capacity(STRUCTURED_JSON_CONFIG_DENSE_PROBE_MAX_BLOCK_LEN + 1),
        |_, _| {},
    );

    assert!(matcher.is_short_line_text);
    assert!(matcher.current_block_is_composer_dictionary_text);
    assert!(!matcher.current_block_is_dictionary_lockfile_text);
    assert_eq!(
        matcher.no_match_probe_step(0),
        COMPOSER_JSON_LOCKFILE_NO_MATCH_PROBE_STEP
    );
}

#[test]
fn large_code_text_blocks_keep_dense_probe_step() {
    let mut matcher = MatchGenerator::new(CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN + 1);
    matcher.file_type_hint = CompressionFileType::CodeText;
    matcher.add_data(
        b"pub(crate) fn emit_symbol(code: u8) { table.push(code); }\n"
            .repeat((48 * 1024 / 57) + 16)
            .to_vec(),
        SuffixStore::with_capacity(CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN + 1),
        |_, _| {},
    );

    assert!(matcher.is_short_line_text);
    assert!(matcher.last_entry().data.len() > 32 * 1024);
    assert!(matcher.last_entry().data.len() <= CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN);
    assert_eq!(matcher.no_match_probe_step(0), 1);
}

#[test]
fn code_text_blocks_just_above_64k_keep_dense_probe_step() {
    let mut matcher = MatchGenerator::new(CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN + 1);
    matcher.file_type_hint = CompressionFileType::CodeText;
    matcher.add_data(
        b"pub(crate) fn write_block_header(bits: &mut BitWriter, compressed: bool) {\n"
            .repeat((66 * 1024 / 73) + 24)
            .to_vec(),
        SuffixStore::with_capacity(CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN + 1),
        |_, _| {},
    );

    assert!(matcher.is_short_line_text);
    assert!(matcher.last_entry().data.len() > 64 * 1024);
    assert!(matcher.last_entry().data.len() <= CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN);
    assert_eq!(matcher.no_match_probe_step(0), 1);
}

#[test]
fn small_code_text_blocks_use_lower_non_repeat_floor() {
    let mut matcher = MatchGenerator::new(SMALL_CODE_TEXT_MIN_NON_REPEAT_MAX_BLOCK_LEN + 1);
    matcher.file_type_hint = CompressionFileType::CodeText;
    matcher.add_data(
        b"pub(crate) fn update_progress(done: usize, total: usize) { println!(\"{done}/{total}\"); }\n"
            .repeat((12 * 1024 / 91) + 12)
            .to_vec(),
        SuffixStore::with_capacity(SMALL_CODE_TEXT_MIN_NON_REPEAT_MAX_BLOCK_LEN + 1),
        |_, _| {},
    );

    assert!(matcher.is_short_line_text);
    assert!(matcher.last_entry().data.len() <= SMALL_CODE_TEXT_MIN_NON_REPEAT_MAX_BLOCK_LEN);
    assert_eq!(
        matcher.min_non_repeat_match_len,
        SMALL_TEXT_MIN_NON_REPEAT_MATCH_LEN
    );
}

#[test]
fn large_code_text_blocks_keep_code_non_repeat_floor() {
    let mut matcher = MatchGenerator::new(CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN + 1);
    matcher.file_type_hint = CompressionFileType::CodeText;
    matcher.add_data(
        b"pub(crate) fn emit_symbol(code: u8) { table.push(code); }\n"
            .repeat((48 * 1024 / 57) + 16)
            .to_vec(),
        SuffixStore::with_capacity(CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN + 1),
        |_, _| {},
    );

    assert!(matcher.is_short_line_text);
    assert!(matcher.last_entry().data.len() > SMALL_CODE_TEXT_MIN_NON_REPEAT_MAX_BLOCK_LEN);
    assert_eq!(
        matcher.min_non_repeat_match_len,
        CODE_LIKE_SHORT_TEXT_MIN_NON_REPEAT_MATCH_LEN
    );
}

#[test]
fn best_level_can_prefer_longer_next_position_window_match() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.prefer_binary_next_position_lookahead = true;
    matcher.add_data(
        b"ABCDEBCDE1234567890".to_vec(),
        SuffixStore::with_capacity(1024),
        |_, _| {},
    );
    matcher.skip_matching();
    matcher.add_data(
        b"ABCDE1234567890".to_vec(),
        SuffixStore::with_capacity(1024),
        |_, _| {},
    );

    matcher.next_sequence(|seq| {
        assert_eq!(
            seq,
            Sequence::Triple {
                literals: b"A",
                offset: 15,
                match_len: 14,
            }
        );
    });
    assert!(!matcher.next_sequence(|_| {}));
}

#[test]
fn best_level_can_find_next_position_window_match_without_current_candidate() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.prefer_binary_next_position_lookahead = true;
    matcher.add_data(
        b"BCDE1234567890".to_vec(),
        SuffixStore::with_capacity(1024),
        |_, _| {},
    );
    matcher.skip_matching();
    matcher.add_data(
        b"ABCDE1234567890".to_vec(),
        SuffixStore::with_capacity(1024),
        |_, _| {},
    );

    matcher.next_sequence(|seq| match seq {
        Sequence::Triple {
            literals,
            offset,
            match_len,
        } => {
            assert_eq!(literals, b"A");
            assert_eq!(match_len, 14);
            assert!(offset > 1);
        }
        other => panic!("expected triple, got {:?}", other),
    });
    assert!(!matcher.next_sequence(|_| {}));
}

#[test]
fn best_level_can_prefer_next_position_repeat_match() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.prefer_binary_next_position_repeat_lookahead = true;
    matcher.add_data(
        b"MATCHTAIL".to_vec(),
        SuffixStore::with_capacity(1024),
        |_, _| {},
    );
    matcher.skip_matching_for_incompressible();
    matcher.offset_history = OffsetHistory::from_offsets(10, 4, 8);
    matcher.add_data(
        b"xMATCHTAIL".to_vec(),
        SuffixStore::with_capacity(1024),
        |_, _| {},
    );

    matcher.next_sequence(|seq| {
        assert_eq!(
            seq,
            Sequence::Triple {
                literals: b"x",
                offset: 10,
                match_len: 9,
            }
        );
    });
    assert!(!matcher.next_sequence(|_| {}));
}

#[test]
fn best_text_repeat_helper_can_prefer_next_position_repeat_match() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.prefer_binary_next_position_repeat_lookahead = true;
    matcher.use_second_newest_probe = true;
    matcher.use_text_repeat_pipeline = true;
    matcher.is_text_block = true;
    matcher.min_non_repeat_match_len = TEXT_MIN_NON_REPEAT_MATCH_LEN;
    matcher.add_data(
        b"MATCHTAIL".to_vec(),
        SuffixStore::with_capacity(1024),
        |_, _| {},
    );
    matcher.skip_matching_for_incompressible();
    matcher.offset_history = OffsetHistory::from_offsets(10, 4, 8);
    matcher.add_data(
        b"xMATCHTAIL".to_vec(),
        SuffixStore::with_capacity(1024),
        |_, _| {},
    );

    matcher.next_sequence(|seq| {
        assert_eq!(
            seq,
            Sequence::Triple {
                literals: b"x",
                offset: 10,
                match_len: 9,
            }
        );
    });
    assert!(!matcher.next_sequence(|_| {}));
}

#[test]
fn best_text_repeat_helper_keeps_current_repeat_match() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.use_text_repeat_pipeline = true;
    matcher.is_text_block = true;
    matcher.min_non_repeat_match_len = TEXT_MIN_NON_REPEAT_MATCH_LEN;
    matcher.add_data(
        b"MATCHTAIL".to_vec(),
        SuffixStore::with_capacity(1024),
        |_, _| {},
    );
    matcher.skip_matching_for_incompressible();
    matcher.offset_history = OffsetHistory::from_offsets(12, 9, 4);
    matcher.add_data(
        b"MATCHTAIL".to_vec(),
        SuffixStore::with_capacity(1024),
        |_, _| {},
    );

    matcher.next_sequence(|seq| {
        assert_eq!(
            seq,
            Sequence::Triple {
                literals: b"",
                offset: 9,
                match_len: 9,
            }
        );
    });
    assert!(!matcher.next_sequence(|_| {}));
}

#[test]
fn unavailable_repeat_offsets_are_rejected_before_lookup() {
    assert!(!MatchGenerator::repeat_offset_is_available(0, 8, 4));
    assert!(!MatchGenerator::repeat_offset_is_available(13, 8, 4));
    assert!(MatchGenerator::repeat_offset_is_available(12, 8, 4));
}

#[test]
fn rle_history_indexes_only_extreme_suffixes() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.add_data(
        alloc::vec![0; 512],
        SuffixStore::with_capacity(1024),
        |_, _| {},
    );

    matcher.skip_matching_for_rle();

    let suffixes = &matcher.last_entry().suffixes;
    let indexed = suffixes.slots.iter().filter(|slot| slot.is_some()).count();
    assert_eq!(indexed, 1, "RLE block should only need one suffix key");

    let candidates = suffixes
        .candidates(&[0; MIN_MATCH_LEN])
        .expect("RLE suffix key should exist");
    assert_eq!(candidates.oldest, 0);
    assert_eq!(candidates.newest, Some(512 - MIN_MATCH_LEN));
}

#[test]
fn sparse_rle_history_still_matches_following_repeated_block() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.add_data(
        alloc::vec![0; 512],
        SuffixStore::with_capacity(1024),
        |_, _| {},
    );
    matcher.skip_matching_for_rle();
    matcher.offset_history = OffsetHistory::from_offsets(2048, 2049, 2050);
    matcher.add_data(
        alloc::vec![0; 32],
        SuffixStore::with_capacity(1024),
        |_, _| {},
    );

    matcher.next_sequence(|seq| {
        assert_eq!(
            seq,
            Sequence::Triple {
                literals: &[],
                offset: MIN_MATCH_LEN,
                match_len: 32,
            },
        );
    });
    assert!(!matcher.next_sequence(|_| {}));
}

#[test]
fn short_match_ranges_are_indexed_densely() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.add_data(xorshift(512), SuffixStore::with_capacity(1024), |_, _| {});
    matcher.suffix_idx = 10;

    matcher.add_suffixes_for_match(10 + DENSE_MATCH_INDEX_LIMIT);

    let indexed = matcher
        .last_entry()
        .suffixes
        .slots
        .iter()
        .filter(|slot| slot.is_some())
        .count();
    assert!(
        indexed > 16,
        "short match should index densely: {}",
        indexed
    );
}

#[test]
fn long_match_ranges_are_indexed_sparsely() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.add_data(xorshift(512), SuffixStore::with_capacity(1024), |_, _| {});
    matcher.suffix_idx = 10;

    let match_end = 10 + DENSE_MATCH_INDEX_LIMIT + 1;
    matcher.add_suffixes_for_match(match_end);

    let last_entry = matcher.last_entry();
    let suffixes = &last_entry.suffixes;
    let indexed = suffixes.slots.iter().filter(|slot| slot.is_some()).count();
    assert!(
        indexed <= 3,
        "long match should index sparsely: {}",
        indexed
    );

    for idx in [10, 12, match_end - SPARSE_MATCH_END_INDEX_BACKOFF] {
        let key = &last_entry.data[idx..idx + MIN_MATCH_LEN];
        let candidates = suffixes.candidates(key).expect("sparse index must exist");
        assert_eq!(candidates.oldest, idx);
        assert_eq!(candidates.newest, None);
    }
}

#[test]
fn best_level_long_match_ranges_use_complementary_end_insertion() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.use_complementary_end_insertion = true;
    matcher.add_data(xorshift(512), SuffixStore::with_capacity(1024), |_, _| {});
    matcher.suffix_idx = 10;

    let match_end = 10 + DENSE_MATCH_INDEX_LIMIT + 1;
    matcher.add_suffixes_for_match(match_end);

    let last_entry = matcher.last_entry();
    let suffixes = &last_entry.suffixes;

    for idx in [
        10,
        12,
        match_end - 1,
        match_end - SPARSE_MATCH_END_INDEX_BACKOFF,
    ] {
        let key = &last_entry.data[idx..idx + MIN_MATCH_LEN];
        let candidates = suffixes
            .candidates(key)
            .expect("best-level sparse index must exist");
        assert_eq!(candidates.oldest, idx);
        assert_eq!(candidates.newest, None);
    }
}

#[test]
fn best_level_zero_literal_repeat_matches_use_sparse_indexing() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.use_complementary_end_insertion = true;
    matcher.add_data(xorshift(512), SuffixStore::with_capacity(1024), |_, _| {});
    matcher.suffix_idx = 10;
    matcher.last_idx_in_sequence = 10;

    let match_end = 10 + 32;
    matcher.emit_candidate(
        MatchCandidate {
            start_idx: 10,
            offset: 32,
            match_len: 32,
            repeat_offset: true,
            #[cfg(test)]
            source: CandidateSource::RepeatCurrent(RepeatCandidateKind::First),
        },
        &mut |_| {},
    );

    let last_entry = matcher.last_entry();
    let suffixes = &last_entry.suffixes;
    for idx in [
        10,
        12,
        match_end - 1,
        match_end - SPARSE_MATCH_END_INDEX_BACKOFF,
    ] {
        let key = &last_entry.data[idx..idx + MIN_MATCH_LEN];
        let candidates = suffixes
            .candidates(key)
            .expect("best-level zero-literal repeat sparse index must exist");
        assert_eq!(candidates.oldest, idx);
        assert_eq!(candidates.newest, None);
    }
}

#[test]
fn best_sidecar_tracks_second_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(64 * 1024);
    matcher.use_second_newest_probe = true;
    let data = [0u8, 1, 2, 3, 4].repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / 5 + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    let key_value = SuffixStore::key_value(&matcher.last_entry().data[0..MIN_MATCH_LEN]);
    matcher.add_suffix_at(0);
    matcher.add_suffix_at(5);
    matcher.add_suffix_at(10);

    let slot_index = matcher.last_entry().suffixes.slot_key(key_value).index;
    let stored = matcher.current_second_newest_sidecar[slot_index];
    assert!(stored.is_some(), "expected second-newest sidecar entry");
}

#[test]
fn lockfile_sidecar_tracks_third_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.file_profile_hint = CompressionFileProfile::CargoLock;
    let package = b"[[package]]\nname = \"crate\"\nversion = \"1.0.0\"\nchecksum = \"0123456789abcdef\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\ndependencies = [\n \"dep\",\n]\n\n";
    let data = package.repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / package.len() + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    let key_value = SuffixStore::key_value(&matcher.last_entry().data[0..MIN_MATCH_LEN]);
    matcher.add_suffix_at(0);
    matcher.add_suffix_at(package.len());
    matcher.add_suffix_at(package.len() * 2);
    matcher.add_suffix_at(package.len() * 3);

    let slot_index = matcher.last_entry().suffixes.slot_key(key_value).index;
    let stored = matcher.current_third_newest_sidecar[slot_index];
    assert!(stored.is_some(), "expected third-newest sidecar entry");
}

#[test]
fn lockfile_sidecar_tracks_fourth_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.file_profile_hint = CompressionFileProfile::CargoLock;
    let package = b"[[package]]\nname = \"crate\"\nversion = \"1.0.0\"\nchecksum = \"0123456789abcdef\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\ndependencies = [\n \"dep\",\n]\n\n";
    let data = package.repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / package.len() + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    let key_value = SuffixStore::key_value(&matcher.last_entry().data[0..MIN_MATCH_LEN]);
    matcher.add_suffix_at(0);
    matcher.add_suffix_at(package.len());
    matcher.add_suffix_at(package.len() * 2);
    matcher.add_suffix_at(package.len() * 3);
    matcher.add_suffix_at(package.len() * 4);

    let slot_index = matcher.last_entry().suffixes.slot_key(key_value).index;
    let stored = matcher.current_fourth_newest_sidecar[slot_index];
    assert!(stored.is_some(), "expected fourth-newest sidecar entry");
}

#[test]
fn lockfile_sidecar_tracks_fifth_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.file_profile_hint = CompressionFileProfile::CargoLock;
    let package = b"[[package]]\nname = \"crate\"\nversion = \"1.0.0\"\nchecksum = \"0123456789abcdef\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\ndependencies = [\n \"dep\",\n]\n\n";
    let data = package.repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / package.len() + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    let key_value = SuffixStore::key_value(&matcher.last_entry().data[0..MIN_MATCH_LEN]);
    matcher.add_suffix_at(0);
    matcher.add_suffix_at(package.len());
    matcher.add_suffix_at(package.len() * 2);
    matcher.add_suffix_at(package.len() * 3);
    matcher.add_suffix_at(package.len() * 4);
    matcher.add_suffix_at(package.len() * 5);

    let slot_index = matcher.last_entry().suffixes.slot_key(key_value).index;
    let stored = matcher.current_fifth_newest_sidecar[slot_index];
    assert!(stored.is_some(), "expected fifth-newest sidecar entry");
}

#[test]
fn lockfile_sidecar_tracks_sixth_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.file_profile_hint = CompressionFileProfile::CargoLock;
    let package = b"[[package]]\nname = \"crate\"\nversion = \"1.0.0\"\nchecksum = \"0123456789abcdef\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\ndependencies = [\n \"dep\",\n]\n\n";
    let data = package.repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / package.len() + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    let key_value = SuffixStore::key_value(&matcher.last_entry().data[0..MIN_MATCH_LEN]);
    matcher.add_suffix_at(0);
    matcher.add_suffix_at(package.len());
    matcher.add_suffix_at(package.len() * 2);
    matcher.add_suffix_at(package.len() * 3);
    matcher.add_suffix_at(package.len() * 4);
    matcher.add_suffix_at(package.len() * 5);
    matcher.add_suffix_at(package.len() * 6);

    let slot_index = matcher.last_entry().suffixes.slot_key(key_value).index;
    let stored = matcher.current_sixth_newest_sidecar[slot_index];
    assert!(stored.is_some(), "expected sixth-newest sidecar entry");
}

#[test]
fn lockfile_sidecar_tracks_seventh_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.file_profile_hint = CompressionFileProfile::CargoLock;
    let package = b"[[package]]\nname = \"crate\"\nversion = \"1.0.0\"\nchecksum = \"0123456789abcdef\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\ndependencies = [\n \"dep\",\n]\n\n";
    let data = package.repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / package.len() + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    let key_value = SuffixStore::key_value(&matcher.last_entry().data[0..MIN_MATCH_LEN]);
    matcher.add_suffix_at(0);
    matcher.add_suffix_at(package.len());
    matcher.add_suffix_at(package.len() * 2);
    matcher.add_suffix_at(package.len() * 3);
    matcher.add_suffix_at(package.len() * 4);
    matcher.add_suffix_at(package.len() * 5);
    matcher.add_suffix_at(package.len() * 6);
    matcher.add_suffix_at(package.len() * 7);

    let slot_index = matcher.last_entry().suffixes.slot_key(key_value).index;
    let stored = matcher.current_seventh_newest_sidecar[slot_index];
    assert!(stored.is_some(), "expected seventh-newest sidecar entry");
}

#[test]
fn lockfile_sidecar_tracks_eighth_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.file_profile_hint = CompressionFileProfile::CargoLock;
    let package = b"[[package]]\nname = \"crate\"\nversion = \"1.0.0\"\nchecksum = \"0123456789abcdef\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\ndependencies = [\n \"dep\",\n]\n\n";
    let data = package.repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / package.len() + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    let key_value = SuffixStore::key_value(&matcher.last_entry().data[0..MIN_MATCH_LEN]);
    matcher.add_suffix_at(0);
    matcher.add_suffix_at(package.len());
    matcher.add_suffix_at(package.len() * 2);
    matcher.add_suffix_at(package.len() * 3);
    matcher.add_suffix_at(package.len() * 4);
    matcher.add_suffix_at(package.len() * 5);
    matcher.add_suffix_at(package.len() * 6);
    matcher.add_suffix_at(package.len() * 7);
    matcher.add_suffix_at(package.len() * 8);

    let slot_index = matcher.last_entry().suffixes.slot_key(key_value).index;
    let stored = matcher.current_eighth_newest_sidecar[slot_index];
    assert!(stored.is_some(), "expected eighth-newest sidecar entry");
}

#[test]
fn lockfile_sidecar_tracks_ninth_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.file_profile_hint = CompressionFileProfile::CargoLock;
    let package = b"[[package]]\nname = \"crate\"\nversion = \"1.0.0\"\nchecksum = \"0123456789abcdef\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\ndependencies = [\n \"dep\",\n]\n\n";
    let data = package.repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / package.len() + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    let key_value = SuffixStore::key_value(&matcher.last_entry().data[0..MIN_MATCH_LEN]);
    matcher.add_suffix_at(0);
    matcher.add_suffix_at(package.len());
    matcher.add_suffix_at(package.len() * 2);
    matcher.add_suffix_at(package.len() * 3);
    matcher.add_suffix_at(package.len() * 4);
    matcher.add_suffix_at(package.len() * 5);
    matcher.add_suffix_at(package.len() * 6);
    matcher.add_suffix_at(package.len() * 7);
    matcher.add_suffix_at(package.len() * 8);
    matcher.add_suffix_at(package.len() * 9);

    let slot_index = matcher.last_entry().suffixes.slot_key(key_value).index;
    let stored = matcher.current_ninth_newest_sidecar[slot_index];
    assert!(stored.is_some(), "expected ninth-newest sidecar entry");
}

#[test]
fn lockfile_sidecar_tracks_tenth_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.file_profile_hint = CompressionFileProfile::CargoLock;
    let package = b"[[package]]\nname = \"crate\"\nversion = \"1.0.0\"\nchecksum = \"0123456789abcdef\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\ndependencies = [\n \"dep\",\n]\n\n";
    let data = package.repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / package.len() + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    let key_value = SuffixStore::key_value(&matcher.last_entry().data[0..MIN_MATCH_LEN]);
    matcher.add_suffix_at(0);
    matcher.add_suffix_at(package.len());
    matcher.add_suffix_at(package.len() * 2);
    matcher.add_suffix_at(package.len() * 3);
    matcher.add_suffix_at(package.len() * 4);
    matcher.add_suffix_at(package.len() * 5);
    matcher.add_suffix_at(package.len() * 6);
    matcher.add_suffix_at(package.len() * 7);
    matcher.add_suffix_at(package.len() * 8);
    matcher.add_suffix_at(package.len() * 9);
    matcher.add_suffix_at(package.len() * 10);

    let slot_index = matcher.last_entry().suffixes.slot_key(key_value).index;
    let stored = matcher.current_tenth_newest_sidecar[slot_index];
    assert!(stored.is_some(), "expected tenth-newest sidecar entry");
}

#[test]
fn lockfile_sidecar_tracks_eleventh_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.file_profile_hint = CompressionFileProfile::CargoLock;
    let package = b"[[package]]\nname = \"crate\"\nversion = \"1.0.0\"\nchecksum = \"0123456789abcdef\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\ndependencies = [\n \"dep\",\n]\n\n";
    let data = package.repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / package.len() + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    let key_value = SuffixStore::key_value(&matcher.last_entry().data[0..MIN_MATCH_LEN]);
    matcher.add_suffix_at(0);
    matcher.add_suffix_at(package.len());
    matcher.add_suffix_at(package.len() * 2);
    matcher.add_suffix_at(package.len() * 3);
    matcher.add_suffix_at(package.len() * 4);
    matcher.add_suffix_at(package.len() * 5);
    matcher.add_suffix_at(package.len() * 6);
    matcher.add_suffix_at(package.len() * 7);
    matcher.add_suffix_at(package.len() * 8);
    matcher.add_suffix_at(package.len() * 9);
    matcher.add_suffix_at(package.len() * 10);
    matcher.add_suffix_at(package.len() * 11);

    let slot_index = matcher.last_entry().suffixes.slot_key(key_value).index;
    let stored = matcher.current_eleventh_newest_sidecar[slot_index];
    assert!(stored.is_some(), "expected eleventh-newest sidecar entry");
}

#[test]
fn lockfile_sidecar_tracks_twelfth_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.file_profile_hint = CompressionFileProfile::CargoLock;
    let package = b"[[package]]\nname = \"crate\"\nversion = \"1.0.0\"\nchecksum = \"0123456789abcdef\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\ndependencies = [\n \"dep\",\n]\n\n";
    let data = package.repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / package.len() + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    let key_value = SuffixStore::key_value(&matcher.last_entry().data[0..MIN_MATCH_LEN]);
    matcher.add_suffix_at(0);
    matcher.add_suffix_at(package.len());
    matcher.add_suffix_at(package.len() * 2);
    matcher.add_suffix_at(package.len() * 3);
    matcher.add_suffix_at(package.len() * 4);
    matcher.add_suffix_at(package.len() * 5);
    matcher.add_suffix_at(package.len() * 6);
    matcher.add_suffix_at(package.len() * 7);
    matcher.add_suffix_at(package.len() * 8);
    matcher.add_suffix_at(package.len() * 9);
    matcher.add_suffix_at(package.len() * 10);
    matcher.add_suffix_at(package.len() * 11);
    matcher.add_suffix_at(package.len() * 12);

    let slot_index = matcher.last_entry().suffixes.slot_key(key_value).index;
    let stored = matcher.current_twelfth_newest_sidecar[slot_index];
    assert!(stored.is_some(), "expected twelfth-newest sidecar entry");
}

#[test]
fn lockfile_sidecar_tracks_thirteenth_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.file_profile_hint = CompressionFileProfile::CargoLock;
    let package = b"[[package]]\nname = \"crate\"\nversion = \"1.0.0\"\nchecksum = \"0123456789abcdef\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\ndependencies = [\n \"dep\",\n]\n\n";
    let data = package.repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / package.len() + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    let key_value = SuffixStore::key_value(&matcher.last_entry().data[0..MIN_MATCH_LEN]);
    matcher.add_suffix_at(0);
    matcher.add_suffix_at(package.len());
    matcher.add_suffix_at(package.len() * 2);
    matcher.add_suffix_at(package.len() * 3);
    matcher.add_suffix_at(package.len() * 4);
    matcher.add_suffix_at(package.len() * 5);
    matcher.add_suffix_at(package.len() * 6);
    matcher.add_suffix_at(package.len() * 7);
    matcher.add_suffix_at(package.len() * 8);
    matcher.add_suffix_at(package.len() * 9);
    matcher.add_suffix_at(package.len() * 10);
    matcher.add_suffix_at(package.len() * 11);
    matcher.add_suffix_at(package.len() * 12);
    matcher.add_suffix_at(package.len() * 13);

    let slot_index = matcher.last_entry().suffixes.slot_key(key_value).index;
    let stored = matcher.current_thirteenth_newest_sidecar[slot_index];
    assert!(stored.is_some(), "expected thirteenth-newest sidecar entry");
}

#[test]
fn best_sidecar_is_disabled_for_small_blocks() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.use_second_newest_probe = true;
    matcher.add_data(
        b"abcdeabcdeabcdeabcde".to_vec(),
        SuffixStore::with_capacity(64),
        |_, _| {},
    );

    assert!(matcher.current_second_newest_sidecar.is_empty());
    assert!(!matcher.should_track_second_newest_for_current_entry());
}

#[test]
fn dictionary_lockfile_text_tracks_second_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    let package = b"[[package]]\nname = \"crate\"\nversion = \"1.0.0\"\nchecksum = \"0123456789abcdef\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\ndependencies = [\n \"dep\",\n]\n\n";
    let data = package.repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / package.len() + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    assert!(!matcher.current_second_newest_sidecar.is_empty());
    assert!(matcher.should_track_second_newest_for_current_entry());
    assert!(matcher.prefer_lockfile_second_newest_before_newest());
    assert_eq!(
        matcher.no_match_probe_step(0),
        SHORT_LINE_TEXT_NO_MATCH_PROBE_STEP
    );
}

#[test]
fn cargo_lock_profile_marks_dictionary_lockfile_block_without_content_heuristic() {
    let mut matcher = MatchGenerator::new(32 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.file_profile_hint = CompressionFileProfile::CargoLock;
    let data = b"name = \"crate\"\nversion = \"1.0.0\"\nsource = \"registry\"\n"
        .repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / 44 + 8);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    assert!(matcher.current_block_is_dictionary_lockfile_text);
}

#[test]
fn composer_lock_profile_marks_composer_block_without_content_heuristic() {
    let mut matcher = MatchGenerator::new(32 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.file_profile_hint = CompressionFileProfile::ComposerLock;
    let data = b"name = \"vendor/package\"\nversion = \"1.0.0\"\nsource = \"git\"\n"
        .repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / 46 + 8);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    assert!(matcher.current_block_is_composer_dictionary_text);
    assert!(!matcher.current_block_is_dictionary_lockfile_text);
}

#[test]
fn dictionary_text_tracks_second_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    let data = b"dictionary-entry\n".repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / 17 + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    assert!(!matcher.current_second_newest_sidecar.is_empty());
    assert!(matcher.should_track_second_newest_for_current_entry());
    assert!(!matcher.prefer_lockfile_second_newest_before_newest());
}

#[test]
fn code_text_tracks_second_newest_for_current_entry() {
    let mut matcher = MatchGenerator::new(CODE_TEXT_SECOND_NEWEST_MAX_BLOCK_LEN + 1);
    matcher.file_type_hint = CompressionFileType::CodeText;
    let data = b"def build_fixture_manifest(root: Path, entries: list[str]) -> None:\n"
        .repeat(BEST_SECOND_NEWEST_MIN_BLOCK_LEN / 67 + 64);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    assert!(matcher.is_short_line_text);
    assert!(!matcher.current_second_newest_sidecar.is_empty());
    assert!(matcher.should_track_second_newest_for_current_entry());
    assert!(!matcher.prefer_lockfile_second_newest_before_newest());
}

#[test]
fn large_code_text_disables_second_newest_sidecar() {
    let mut matcher = MatchGenerator::new(CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN + 1);
    matcher.file_type_hint = CompressionFileType::CodeText;
    let data = b"def build_fixture_manifest(root: Path, entries: list[str]) -> None:\n"
        .repeat(CODE_TEXT_SECOND_NEWEST_MAX_BLOCK_LEN / 67 + 256);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    assert!(matcher.is_short_line_text);
    assert!(matcher.last_entry().data.len() > CODE_TEXT_SECOND_NEWEST_MAX_BLOCK_LEN);
    assert!(matcher.current_second_newest_sidecar.is_empty());
    assert!(!matcher.should_track_second_newest_for_current_entry());
}

#[test]
fn dictionary_binary_does_not_track_second_newest_without_best_probe() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.add_data(
        alloc::vec![0u8; BEST_SECOND_NEWEST_MIN_BLOCK_LEN],
        SuffixStore::with_capacity(64),
        |_, _| {},
    );

    assert!(matcher.current_second_newest_sidecar.is_empty());
    assert!(!matcher.should_track_second_newest_for_current_entry());
    assert!(!matcher.prefer_lockfile_second_newest_before_newest());
}

#[test]
fn lockfile_text_can_keep_current_candidate_over_farther_oldest() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.file_type_hint = CompressionFileType::DictionaryText;
    matcher.current_block_is_dictionary_lockfile_text = true;

    let current = MatchCandidate {
        start_idx: 100,
        offset: 24,
        match_len: 10,
        repeat_offset: false,
        #[cfg(test)]
        source: CandidateSource::WindowCurrentSecondNewest { entry_distance: 0 },
    };
    let found = MatchCandidate {
        start_idx: 100,
        offset: 64,
        match_len: 11,
        repeat_offset: false,
        #[cfg(test)]
        source: CandidateSource::WindowCurrentOldest { entry_distance: 0 },
    };

    assert!(matcher.keep_lockfile_current_candidate_over_oldest(
        found,
        current,
        WindowCandidateMeta {
            entry_distance: 0,
            kind: WindowCandidateKind::Oldest,
        },
    ));
}

#[test]
fn best_current_long_hash_tracks_latest_current_entry_index() {
    let mut matcher = MatchGenerator::new(128 * 1024);
    matcher.use_second_newest_probe = true;
    let data = [0u8, 1, 2, 3, 4, 5, 6, 7].repeat(BEST_CURRENT_LONG_HASH_MIN_BLOCK_LEN / 8 + 1);
    matcher.add_data(data, SuffixStore::with_capacity(64), |_, _| {});

    matcher.add_suffix_at(0);
    matcher.add_suffix_at(8);
    matcher.add_suffix_at(16);

    let long_key = u64::from_le_bytes(
        matcher.last_entry().data[0..8]
            .try_into()
            .expect("need 8 bytes"),
    );
    let slot_index = matcher.last_entry().suffixes.slot_key(long_key).index;
    let stored = matcher.current_long_hash[slot_index];
    assert_eq!(stored.map(|idx| idx.get() as usize - 1), Some(16));
    assert!(matcher.should_track_current_long_hash());
}

#[test]
fn best_current_long_hash_is_disabled_below_threshold() {
    let mut matcher = MatchGenerator::new(BEST_CURRENT_LONG_HASH_MIN_BLOCK_LEN);
    matcher.use_second_newest_probe = true;
    matcher.add_data(
        alloc::vec![0u8; BEST_CURRENT_LONG_HASH_MIN_BLOCK_LEN - 1],
        SuffixStore::with_capacity(64),
        |_, _| {},
    );

    assert!(matcher.current_long_hash.is_empty());
    assert!(!matcher.should_track_current_long_hash());
}

#[test]
fn sparse_next_position_match_preserves_start_index() {
    let mut matcher = MatchGenerator::new(1024);
    matcher.add_data(xorshift(512), SuffixStore::with_capacity(1024), |_, _| {});
    matcher.suffix_idx = 10;

    matcher.emit_candidate(
        MatchCandidate {
            start_idx: 11,
            offset: 32,
            match_len: DENSE_MATCH_INDEX_LIMIT + 1,
            repeat_offset: true,
            #[cfg(test)]
            source: CandidateSource::RepeatNextPosition(RepeatCandidateKind::First),
        },
        &mut |_| {},
    );

    let last_entry = matcher.last_entry();
    let key = &last_entry.data[11..11 + MIN_MATCH_LEN];
    let candidates = last_entry
        .suffixes
        .candidates(key)
        .expect("next-position sparse match should keep start index");
    assert_eq!(candidates.oldest, 11);
    assert_eq!(candidates.newest, None);
}

#[test]
fn matches() {
    let mut matcher = MatchGenerator::new(1000);
    let mut original_data = Vec::new();
    let mut reconstructed = Vec::new();

    let reconstruct = |seq: Sequence<'_>, reconstructed: &mut Vec<u8>| match seq {
        Sequence::Literals { literals } => reconstructed.extend_from_slice(literals),
        Sequence::Triple {
            literals,
            offset,
            match_len,
        } => {
            reconstructed.extend_from_slice(literals);
            let start = reconstructed.len() - offset;
            for idx in 0..match_len {
                let byte = reconstructed[start + idx];
                reconstructed.push(byte);
            }
        }
    };
    let assert_seq_equal =
        |seq: Sequence<'_>, expected: Sequence<'_>, reconstructed: &mut Vec<u8>| {
            assert_eq!(seq, expected);
            reconstruct(seq, reconstructed);
        };

    matcher.add_data(
        alloc::vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    original_data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    matcher.next_sequence(|seq| {
        assert_eq!(
            seq,
            Sequence::Triple {
                literals: &[0],
                offset: 1,
                match_len: 9,
            },
        );
        reconstruct(seq, &mut reconstructed);
    });

    assert!(!matcher.next_sequence(|_| {}));

    matcher.add_data(
        alloc::vec![1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 0, 0, 0, 0, 0,],
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    original_data.extend_from_slice(&[
        1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 0, 0, 0, 0, 0,
    ]);

    matcher.next_sequence(|seq| {
        assert_seq_equal(
            seq,
            Sequence::Triple {
                literals: &[1, 2, 3, 4, 5, 6],
                offset: 6,
                match_len: 12,
            },
            &mut reconstructed,
        );
    });
    matcher.next_sequence(|seq| {
        assert_seq_equal(
            seq,
            Sequence::Triple {
                literals: &[],
                offset: 23,
                match_len: 5,
            },
            &mut reconstructed,
        );
    });
    assert!(!matcher.next_sequence(|_| {}));

    matcher.add_data(
        alloc::vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 0, 0, 0, 0],
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    original_data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 0, 0, 0, 0]);

    matcher.next_sequence(|seq| {
        assert_seq_equal(
            seq,
            Sequence::Triple {
                literals: &[],
                offset: 11,
                match_len: 6,
            },
            &mut reconstructed,
        );
    });
    matcher.next_sequence(|seq| {
        assert_seq_equal(
            seq,
            Sequence::Triple {
                literals: &[7, 8, 9, 10, 11],
                offset: 16,
                match_len: 5,
            },
            &mut reconstructed,
        );
    });
    assert!(!matcher.next_sequence(|_| {}));

    matcher.add_data(
        alloc::vec![0, 0, 0, 0, 0],
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    original_data.extend_from_slice(&[0, 0, 0, 0, 0]);

    matcher.next_sequence(|seq| {
        assert_seq_equal(
            seq,
            Sequence::Triple {
                literals: &[],
                offset: 5,
                match_len: 5,
            },
            &mut reconstructed,
        );
    });
    assert!(!matcher.next_sequence(|_| {}));

    matcher.add_data(
        alloc::vec![7, 8, 9, 10, 11],
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    original_data.extend_from_slice(&[7, 8, 9, 10, 11]);

    matcher.next_sequence(|seq| {
        assert_seq_equal(
            seq,
            Sequence::Triple {
                literals: &[],
                offset: 15,
                match_len: 5,
            },
            &mut reconstructed,
        );
    });
    assert!(!matcher.next_sequence(|_| {}));

    matcher.add_data(
        alloc::vec![1, 3, 5, 7, 9],
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    matcher.skip_matching();
    original_data.extend_from_slice(&[1, 3, 5, 7, 9]);
    reconstructed.extend_from_slice(&[1, 3, 5, 7, 9]);
    assert!(!matcher.next_sequence(|_| {}));

    matcher.add_data(
        alloc::vec![1, 3, 5, 7, 9],
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    original_data.extend_from_slice(&[1, 3, 5, 7, 9]);

    matcher.next_sequence(|seq| {
        assert_seq_equal(
            seq,
            Sequence::Triple {
                literals: &[],
                offset: 5,
                match_len: 5,
            },
            &mut reconstructed,
        );
    });
    assert!(!matcher.next_sequence(|_| {}));

    matcher.add_data(
        alloc::vec![31, 32, 33, 34, 35],
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    matcher.skip_matching_for_incompressible();
    original_data.extend_from_slice(&[31, 32, 33, 34, 35]);
    reconstructed.extend_from_slice(&[31, 32, 33, 34, 35]);
    assert!(!matcher.next_sequence(|_| {}));

    matcher.add_data(
        alloc::vec![31, 32, 33, 34, 35],
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    original_data.extend_from_slice(&[31, 32, 33, 34, 35]);

    matcher.next_sequence(|seq| {
        assert_seq_equal(
            seq,
            Sequence::Literals {
                literals: &[31, 32, 33, 34, 35],
            },
            &mut reconstructed,
        );
    });
    assert!(!matcher.next_sequence(|_| {}));

    matcher.add_data(
        alloc::vec![0, 0, 11, 13, 15, 17, 20, 11, 13, 15, 17, 20, 21, 23],
        SuffixStore::with_capacity(100),
        |_, _| {},
    );
    original_data.extend_from_slice(&[0, 0, 11, 13, 15, 17, 20, 11, 13, 15, 17, 20, 21, 23]);

    matcher.next_sequence(|seq| {
        assert_seq_equal(
            seq,
            Sequence::Triple {
                literals: &[0, 0, 11, 13, 15, 17, 20],
                offset: 5,
                match_len: 5,
            },
            &mut reconstructed,
        );
    });
    matcher.next_sequence(|seq| {
        assert_seq_equal(
            seq,
            Sequence::Literals {
                literals: &[21, 23],
            },
            &mut reconstructed,
        );
    });
    assert!(!matcher.next_sequence(|_| {}));

    assert_eq!(reconstructed, original_data);
}

#[cfg(test)]
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
