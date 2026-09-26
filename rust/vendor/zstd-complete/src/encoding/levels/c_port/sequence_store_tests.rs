use alloc::{vec, vec::Vec};

use super::sequence_store::{
    append_sequence_literals, prepared_block_into_words, prepared_words_into_block, OffBase,
    RepeatCode, RepeatOffsets, StoredSequence,
};
use crate::encoding::blocks::{PreparedBlock, PreparedSequence};

#[test]
fn offbase_uses_c_numeric_representation() {
    assert_eq!(OffBase::from_c_value(0), None);
    assert_eq!(
        OffBase::from_c_value(1),
        Some(OffBase::Repeat(RepeatCode::First))
    );
    assert_eq!(
        OffBase::from_c_value(2),
        Some(OffBase::Repeat(RepeatCode::Second))
    );
    assert_eq!(
        OffBase::from_c_value(3),
        Some(OffBase::Repeat(RepeatCode::Third))
    );
    assert_eq!(OffBase::from_c_value(4), Some(OffBase::Offset(1)));
    assert_eq!(OffBase::from_offset(17).unwrap().to_c_value(), 20);
}

#[test]
fn stored_sequence_preserves_c_fields() {
    let sequence = StoredSequence::new(3, OffBase::Offset(9), 12);

    assert_eq!(sequence.lit_len, 3);
    assert_eq!(sequence.match_len, 12);
    assert_eq!(sequence.off_base(), OffBase::Offset(9));
    assert_eq!(sequence.off_base_value(), 12);
    assert_eq!(core::mem::size_of::<StoredSequence>(), 12);
}

#[test]
fn prepared_block_word_round_trip_preserves_ownership_and_records() {
    let prepared = PreparedBlock {
        literals: vec![1, 2, 3, 4],
        sequences: vec![PreparedSequence {
            ll: 4,
            ml: 7,
            raw_offset: 11,
            encoded_offset_value: 14,
        }],
    };
    let literal_allocation = prepared.literals.as_ptr();
    let sequence_allocation = prepared.sequences.as_ptr();

    let words = prepared_block_into_words(prepared);
    assert_eq!(words.literals.as_ptr(), literal_allocation);
    assert_eq!(words.sequences.as_ptr().cast(), sequence_allocation);
    assert_eq!(words.sequences, [[4, 7, 11, 14]]);

    let prepared = prepared_words_into_block(words);
    assert_eq!(prepared.literals.as_ptr(), literal_allocation);
    assert_eq!(prepared.sequences.as_ptr(), sequence_allocation);
    assert_eq!(prepared.literals, [1, 2, 3, 4]);
    assert_eq!(prepared.sequences[0].ll, 4);
    assert_eq!(prepared.sequences[0].ml, 7);
    assert_eq!(prepared.sequences[0].raw_offset, 11);
    assert_eq!(prepared.sequences[0].encoded_offset_value, 14);
}

#[test]
fn offset_match_pushes_repeat_history() {
    let mut repeats = RepeatOffsets::new();

    repeats.update(OffBase::Offset(9), 4);

    assert_eq!(repeats.as_offsets(), [9, 1, 4]);
    assert_eq!(repeats.resolve(OffBase::Offset(9), 4), 9);
}

#[test]
fn repeat_codes_with_literals_match_c_update_rules() {
    let mut repeats = RepeatOffsets::from_offsets(11, 22, 33);

    repeats.update(OffBase::Repeat(RepeatCode::First), 5);
    assert_eq!(repeats.as_offsets(), [11, 22, 33]);
    assert_eq!(repeats.resolve(OffBase::Repeat(RepeatCode::First), 5), 11);

    repeats.update(OffBase::Repeat(RepeatCode::Second), 5);
    assert_eq!(repeats.as_offsets(), [22, 11, 33]);

    repeats.update(OffBase::Repeat(RepeatCode::Third), 5);
    assert_eq!(repeats.as_offsets(), [33, 22, 11]);
}

#[test]
fn repeat_codes_without_literals_shift_like_c() {
    let mut repeats = RepeatOffsets::from_offsets(11, 22, 33);

    repeats.update(OffBase::Repeat(RepeatCode::First), 0);
    assert_eq!(repeats.as_offsets(), [22, 11, 33]);

    repeats.update(OffBase::Repeat(RepeatCode::Second), 0);
    assert_eq!(repeats.as_offsets(), [33, 22, 11]);

    repeats.update(OffBase::Repeat(RepeatCode::Third), 0);
    assert_eq!(repeats.as_offsets(), [32, 33, 22]);
}

#[test]
fn fused_numeric_resolve_and_update_matches_separate_c_operations() {
    for initial in [
        RepeatOffsets::new(),
        RepeatOffsets::from_offsets(11, 22, 33),
        RepeatOffsets::from_offsets(101, 7, 53),
    ] {
        for off_base in 1..=20 {
            for lit_len in [0, 1, 17] {
                let mut separate = initial;
                let expected = separate.resolve_c_value(off_base, lit_len);
                separate.update_c_value(off_base, lit_len);

                let mut fused = initial;
                let actual = fused.resolve_and_update_c_value(off_base, lit_len);

                assert_eq!(actual, expected);
                assert_eq!(fused, separate);
            }
        }
    }
}

#[test]
fn safe_sequence_literal_wildcopy_matches_exact_slices_at_every_boundary() {
    let src = (0usize..160).map(|value| value as u8).collect::<Vec<_>>();

    for len in [0, 1, 16, 17, 32, 33, 48, 49, 64, 65, 96] {
        let mut output = vec![0xAA, 0xBB];
        append_sequence_literals(&mut output, &src, 7, len);
        assert_eq!(&output[..2], &[0xAA, 0xBB]);
        assert_eq!(&output[2..], &src[7..7 + len]);
    }
}

#[test]
fn safe_sequence_literal_wildcopy_falls_back_near_source_end() {
    let src = (0usize..80).map(|value| value as u8).collect::<Vec<_>>();

    for len in [1, 15, 31, 47, 63] {
        let start = src.len() - len;
        let mut output = Vec::new();
        append_sequence_literals(&mut output, &src, start, len);
        assert_eq!(output, src[start..]);
    }
}
