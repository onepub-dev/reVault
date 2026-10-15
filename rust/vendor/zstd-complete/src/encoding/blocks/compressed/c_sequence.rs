use crate::encoding::levels::c_port::sequence_store::StoredSequence;

use super::PreparedSequence;

/// Numeric fields shared by entropy-ready records and C's native SeqStore.
pub(super) trait CSequenceValues: Copy {
    fn literal_length(self) -> u32;
    fn match_length(self) -> u32;
    fn offset_value(self) -> u32;
    fn expected_raw_offset(self) -> Option<u32>;
}

impl CSequenceValues for PreparedSequence {
    #[inline(always)]
    fn literal_length(self) -> u32 {
        self.ll
    }

    #[inline(always)]
    fn match_length(self) -> u32 {
        self.ml
    }

    #[inline(always)]
    fn offset_value(self) -> u32 {
        self.encoded_offset_value
    }

    #[inline(always)]
    fn expected_raw_offset(self) -> Option<u32> {
        Some(self.raw_offset)
    }
}

impl CSequenceValues for StoredSequence {
    #[inline(always)]
    fn literal_length(self) -> u32 {
        self.lit_len
    }

    #[inline(always)]
    fn match_length(self) -> u32 {
        self.match_len
    }

    #[inline(always)]
    fn offset_value(self) -> u32 {
        self.off_base_value()
    }

    #[inline(always)]
    fn expected_raw_offset(self) -> Option<u32> {
        None
    }
}
