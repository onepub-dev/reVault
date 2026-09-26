//! Target-block sequence preference adjustments.

use crate::encoding::{
    blocks::PreparedSequence,
    levels::c_port::{
        greedy::GreedyBlockOutput,
        greedy_block::GreedyPreparedBlock,
        sequence_store::{OffBase, RepeatCode, RepeatOffsets, StoredSequence},
    },
};

pub(super) fn prefer_stored_leading_repcode_literal(
    src: &[u8],
    block_start: usize,
    block: &[u8],
    last_block: bool,
    initial_repeat_offsets: RepeatOffsets,
    output: &mut GreedyBlockOutput,
) {
    if !last_block || output.last_literals != 0 {
        return;
    }
    let [sequence] = output.sequences.as_mut_slice() else {
        return;
    };
    if sequence.lit_len != 0
        || sequence.match_len as usize != block.len()
        || sequence.match_len <= 1
    {
        return;
    }
    let rep1 = initial_repeat_offsets.as_offsets()[0] as usize;
    let Some(rep_start) = block_start
        .checked_add(1)
        .and_then(|pos| pos.checked_sub(rep1))
    else {
        return;
    };
    let match_len = sequence.match_len as usize - 1;
    let Some(rep_end) = rep_start.checked_add(match_len) else {
        return;
    };
    if rep_end > src.len() || src.get(rep_start..rep_end) != block.get(1..) {
        return;
    }

    *sequence = StoredSequence::new(
        1,
        OffBase::Repeat(RepeatCode::First),
        sequence.match_len - 1,
    );
    output.repeat_offsets = initial_repeat_offsets;
}

pub(super) fn prefer_target_leading_repcode_literal(
    src: &[u8],
    block_start: usize,
    block: &[u8],
    last_block: bool,
    initial_repeat_offsets: RepeatOffsets,
    prepared: &mut GreedyPreparedBlock,
) {
    if !last_block || !prepared.prepared.literals.is_empty() {
        return;
    }
    let [sequence] = prepared.prepared.sequences.as_slice() else {
        return;
    };
    if sequence.ll != 0 || sequence.ml as usize != block.len() || sequence.ml <= 1 {
        return;
    }
    let rep1 = initial_repeat_offsets.as_offsets()[0] as usize;
    let Some(rep_start) = block_start
        .checked_add(1)
        .and_then(|pos| pos.checked_sub(rep1))
    else {
        return;
    };
    let match_len = sequence.ml as usize - 1;
    let Some(rep_end) = rep_start.checked_add(match_len) else {
        return;
    };
    if rep_end > src.len() || src.get(rep_start..rep_end) != block.get(1..) {
        return;
    }

    let replacement = PreparedSequence {
        ll: 1,
        ml: sequence.ml - 1,
        raw_offset: rep1 as u32,
        encoded_offset_value: 1,
    };
    prepared.prepared.literals.push(block[0]);
    prepared.prepared.sequences[0] = replacement;
    prepared.repeat_offsets = initial_repeat_offsets;
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::*;
    use crate::encoding::{
        blocks::PreparedBlock, levels::c_port::sequence_store::prepare_stored_sequences,
    };

    fn full_match_prepared() -> GreedyPreparedBlock {
        GreedyPreparedBlock {
            prepared: PreparedBlock {
                literals: Vec::new(),
                sequences: vec![PreparedSequence {
                    ll: 0,
                    ml: 5,
                    raw_offset: 9,
                    encoded_offset_value: 12,
                }],
            },
            repeat_offsets: RepeatOffsets::from_offsets(9, 4, 8),
        }
    }

    #[test]
    fn target_leading_repcode_literal_rewrites_verified_full_match() {
        let src = b"zabcdXabcd";
        let block = &src[5..];
        let mut prepared = full_match_prepared();
        let repeats = RepeatOffsets::from_offsets(5, 4, 8);

        prefer_target_leading_repcode_literal(src, 5, block, true, repeats, &mut prepared);

        assert_eq!(prepared.prepared.literals, b"X");
        assert_eq!(prepared.prepared.sequences.len(), 1);
        let sequence = prepared.prepared.sequences[0];
        assert_eq!(sequence.ll, 1);
        assert_eq!(sequence.ml, 4);
        assert_eq!(sequence.raw_offset, 5);
        assert_eq!(sequence.encoded_offset_value, 1);
        assert_eq!(prepared.repeat_offsets, repeats);
    }

    #[test]
    fn stored_leading_repcode_rewrite_matches_prepared_transaction() {
        let src = b"zabcdXabcd";
        let block = &src[5..];
        let repeats = RepeatOffsets::from_offsets(5, 4, 8);
        let mut prepared = full_match_prepared();
        let mut stored = GreedyBlockOutput {
            sequences: crate::workspace::ReusableVec::from_owned(vec![StoredSequence::new(
                0,
                OffBase::Offset(9),
                5,
            )]),
            last_literals: 0,
            repeat_offsets: RepeatOffsets::from_offsets(9, 5, 4),
        };

        prefer_target_leading_repcode_literal(src, 5, block, true, repeats, &mut prepared);
        prefer_stored_leading_repcode_literal(src, 5, block, true, repeats, &mut stored);
        let materialized =
            prepare_stored_sequences(block, repeats, &stored.sequences, stored.last_literals);

        assert_eq!(materialized.literals, prepared.prepared.literals);
        assert_eq!(materialized.sequences.len(), 1);
        let materialized_sequence = materialized.sequences[0];
        let prepared_sequence = prepared.prepared.sequences[0];
        assert_eq!(materialized_sequence.ll, prepared_sequence.ll);
        assert_eq!(materialized_sequence.ml, prepared_sequence.ml);
        assert_eq!(
            materialized_sequence.raw_offset,
            prepared_sequence.raw_offset
        );
        assert_eq!(
            materialized_sequence.encoded_offset_value,
            prepared_sequence.encoded_offset_value
        );
        assert_eq!(stored.repeat_offsets, prepared.repeat_offsets);
    }

    #[test]
    fn target_leading_repcode_literal_requires_matching_rep1_bytes() {
        let src = b"zabceXabcd";
        let block = &src[5..];
        let mut prepared = full_match_prepared();
        let original_sequence = prepared.prepared.sequences[0];

        prefer_target_leading_repcode_literal(
            src,
            5,
            block,
            true,
            RepeatOffsets::from_offsets(5, 4, 8),
            &mut prepared,
        );

        assert!(prepared.prepared.literals.is_empty());
        assert_eq!(prepared.prepared.sequences[0].ll, original_sequence.ll);
        assert_eq!(prepared.prepared.sequences[0].ml, original_sequence.ml);
        assert_eq!(
            prepared.prepared.sequences[0].encoded_offset_value,
            original_sequence.encoded_offset_value
        );
    }
}
