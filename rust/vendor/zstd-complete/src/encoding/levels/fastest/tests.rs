use super::{best_split_mid_by_decompressed_bytes, derive_best_partitions, PreparedRange};
use crate::encoding::{
    blocks::{PreparedBlock, PreparedSequence},
    frame_compressor::OffsetHistory,
};
use alloc::{vec, vec::Vec};

#[test]
fn best_split_mid_biases_toward_balanced_decompressed_bytes() {
    let prepared = PreparedBlock {
        literals: vec![0; 103],
        sequences: vec![
            PreparedSequence {
                ll: 100,
                ml: 100,
                raw_offset: 4,
                encoded_offset_value: 0,
            },
            PreparedSequence {
                ll: 1,
                ml: 1,
                raw_offset: 4,
                encoded_offset_value: 0,
            },
            PreparedSequence {
                ll: 1,
                ml: 1,
                raw_offset: 4,
                encoded_offset_value: 0,
            },
            PreparedSequence {
                ll: 1,
                ml: 1,
                raw_offset: 4,
                encoded_offset_value: 0,
            },
        ],
    };

    assert_eq!(best_split_mid_by_decompressed_bytes(prepared.as_ref()), 1);
}

#[test]
fn best_split_mid_keeps_even_sequences_centered() {
    let prepared = PreparedBlock {
        literals: vec![0; 8],
        sequences: vec![
            PreparedSequence {
                ll: 2,
                ml: 2,
                raw_offset: 4,
                encoded_offset_value: 0,
            },
            PreparedSequence {
                ll: 2,
                ml: 2,
                raw_offset: 4,
                encoded_offset_value: 0,
            },
            PreparedSequence {
                ll: 2,
                ml: 2,
                raw_offset: 4,
                encoded_offset_value: 0,
            },
            PreparedSequence {
                ll: 2,
                ml: 2,
                raw_offset: 4,
                encoded_offset_value: 0,
            },
        ],
    };

    assert_eq!(best_split_mid_by_decompressed_bytes(prepared.as_ref()), 2);
}

#[test]
fn derive_best_partitions_respects_partition_budget() {
    let quarter_len = 300;
    let sequence_count = quarter_len * 4;
    let mut literals = Vec::with_capacity(sequence_count);
    literals.extend(core::iter::repeat_n(0u8, quarter_len));
    literals.extend(core::iter::repeat_n(1u8, quarter_len));
    literals.extend(core::iter::repeat_n(2u8, quarter_len));
    literals.extend(core::iter::repeat_n(3u8, quarter_len));
    let sequences = vec![
        PreparedSequence {
            ll: 1,
            ml: 3,
            raw_offset: 4,
            encoded_offset_value: 0,
        };
        sequence_count
    ];
    let prepared = PreparedBlock {
        literals,
        sequences,
    };
    let data = vec![0u8; sequence_count * 4];
    let whole_range = PreparedRange {
        prepared: prepared.as_ref(),
        data: &data,
    };
    let mut partitions = Vec::new();

    let used = derive_best_partitions(whole_range, OffsetHistory::new(), 2, &mut partitions);

    assert_eq!(used, 2);
    assert_eq!(partitions.len(), 2);
}
