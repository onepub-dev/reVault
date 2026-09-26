//! Sequence and repeat-offset primitives ported from `ZSTD_storeSeqOnly()` and
//! `ZSTD_updateRep()`.

use alloc::vec::Vec;
use core::{mem::ManuallyDrop, slice};

use crate::encoding::blocks::{PreparedBlock, PreparedBlockRef, PreparedSequence};

pub(crate) use crate::kernel::seqstore::PreparedStoreWords;

const REPCODE_COUNT: u32 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OffBase {
    Repeat(RepeatCode),
    Offset(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RepeatCode {
    First,
    Second,
    Third,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StoredSequence {
    pub(crate) lit_len: u32,
    pub(crate) match_len: u32,
    off_base_value: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RepeatOffsets {
    offsets: [u32; REPCODE_COUNT as usize],
}

const _: () = {
    assert!(core::mem::size_of::<StoredSequence>() == core::mem::size_of::<[u32; 3]>());
    assert!(core::mem::align_of::<StoredSequence>() == core::mem::align_of::<[u32; 3]>());
    assert!(core::mem::offset_of!(StoredSequence, lit_len) == 0);
    assert!(core::mem::offset_of!(StoredSequence, match_len) == 4);
    assert!(core::mem::offset_of!(StoredSequence, off_base_value) == 8);
    assert!(core::mem::size_of::<PreparedSequence>() == core::mem::size_of::<[u32; 4]>());
    assert!(core::mem::align_of::<PreparedSequence>() == core::mem::align_of::<[u32; 4]>());
    assert!(core::mem::offset_of!(PreparedSequence, ll) == 0);
    assert!(core::mem::offset_of!(PreparedSequence, ml) == 4);
    assert!(core::mem::offset_of!(PreparedSequence, raw_offset) == 8);
    assert!(core::mem::offset_of!(PreparedSequence, encoded_offset_value) == 12);
};

impl OffBase {
    pub(crate) fn from_c_value(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::Repeat(RepeatCode::First)),
            2 => Some(Self::Repeat(RepeatCode::Second)),
            3 => Some(Self::Repeat(RepeatCode::Third)),
            4.. => Some(Self::Offset(value - REPCODE_COUNT)),
            0 => None,
        }
    }

    pub(crate) fn from_offset(offset: u32) -> Option<Self> {
        (offset > 0).then_some(Self::Offset(offset))
    }

    pub(crate) fn offset_to_c_value(offset: u32) -> u32 {
        debug_assert!(offset > 0);
        offset + REPCODE_COUNT
    }

    pub(crate) fn to_c_value(self) -> u32 {
        match self {
            Self::Repeat(repeat) => repeat.to_c_value(),
            Self::Offset(offset) => Self::offset_to_c_value(offset),
        }
    }
}

impl RepeatCode {
    fn to_c_value(self) -> u32 {
        match self {
            Self::First => 1,
            Self::Second => 2,
            Self::Third => 3,
        }
    }
}

impl StoredSequence {
    pub(crate) fn new(lit_len: u32, off_base: OffBase, match_len: u32) -> Self {
        Self {
            lit_len,
            match_len,
            off_base_value: off_base.to_c_value(),
        }
    }

    pub(crate) fn off_base(self) -> OffBase {
        OffBase::from_c_value(self.off_base_value)
            .expect("stored C offBase values are always nonzero")
    }

    pub(crate) const fn off_base_value(self) -> u32 {
        self.off_base_value
    }
}

impl RepeatOffsets {
    pub(crate) const fn new() -> Self {
        Self { offsets: [1, 4, 8] }
    }

    pub(crate) const fn from_offsets(newest: u32, second: u32, third: u32) -> Self {
        Self {
            offsets: [newest, second, third],
        }
    }

    pub(crate) const fn as_offsets(self) -> [u32; REPCODE_COUNT as usize] {
        self.offsets
    }

    pub(crate) fn resolve(self, off_base: OffBase, lit_len: u32) -> u32 {
        self.resolve_c_value(off_base.to_c_value(), lit_len)
    }

    pub(crate) fn resolve_c_value(self, off_base: u32, lit_len: u32) -> u32 {
        debug_assert!(off_base > 0);
        if off_base > REPCODE_COUNT {
            off_base - REPCODE_COUNT
        } else {
            let rep_code = off_base - 1 + u32::from(lit_len == 0);
            if rep_code == REPCODE_COUNT {
                self.offsets[0] - 1
            } else {
                self.offsets[rep_code as usize]
            }
        }
    }

    pub(crate) fn update(&mut self, off_base: OffBase, lit_len: u32) {
        self.update_c_value(off_base.to_c_value(), lit_len);
    }

    pub(crate) fn update_c_value(&mut self, off_base: u32, lit_len: u32) {
        debug_assert!(off_base > 0);
        if off_base > REPCODE_COUNT {
            self.offsets[2] = self.offsets[1];
            self.offsets[1] = self.offsets[0];
            self.offsets[0] = off_base - REPCODE_COUNT;
            return;
        }

        let rep_code = off_base - 1 + u32::from(lit_len == 0);
        if rep_code == 0 {
            return;
        }

        let current_offset = if rep_code == REPCODE_COUNT {
            self.offsets[0] - 1
        } else {
            self.offsets[rep_code as usize]
        };
        if rep_code >= 2 {
            self.offsets[2] = self.offsets[1];
        }
        self.offsets[1] = self.offsets[0];
        self.offsets[0] = current_offset;
    }

    /// Resolve C's numeric `offBase` and apply `ZSTD_updateRep()` in one walk.
    pub(crate) fn resolve_and_update_c_value(&mut self, off_base: u32, lit_len: u32) -> u32 {
        debug_assert!(off_base > 0);
        if off_base > REPCODE_COUNT {
            let raw_offset = off_base - REPCODE_COUNT;
            self.offsets[2] = self.offsets[1];
            self.offsets[1] = self.offsets[0];
            self.offsets[0] = raw_offset;
            return raw_offset;
        }

        let rep_code = off_base - 1 + u32::from(lit_len == 0);
        if rep_code == 0 {
            return self.offsets[0];
        }

        let raw_offset = if rep_code == REPCODE_COUNT {
            self.offsets[0] - 1
        } else {
            self.offsets[rep_code as usize]
        };
        if rep_code >= 2 {
            self.offsets[2] = self.offsets[1];
        }
        self.offsets[1] = self.offsets[0];
        self.offsets[0] = raw_offset;
        raw_offset
    }
}

pub(crate) fn prepare_stored_sequences(
    src: &[u8],
    initial_repeat_offsets: RepeatOffsets,
    sequences: &[StoredSequence],
    last_literals: u32,
) -> PreparedBlock {
    prepared_words_into_block(prepare_stored_sequence_words_in(
        src,
        initial_repeat_offsets,
        sequences,
        last_literals,
        PreparedStoreWords::default(),
    ))
}

pub(crate) fn prepare_stored_sequence_words_in(
    src: &[u8],
    initial_repeat_offsets: RepeatOffsets,
    sequences: &[StoredSequence],
    last_literals: u32,
    reuse: PreparedStoreWords,
) -> PreparedStoreWords {
    // SAFETY: the compile-time layout assertions above prove that the local C
    // record has exactly the primitive word-array ABI expected by the leaf
    // codegen crate.
    let sequence_words = unsafe {
        slice::from_raw_parts(
            sequences
                .as_ptr()
                .cast::<crate::kernel::seqstore::StoredSequenceWords>(),
            sequences.len(),
        )
    };
    // SAFETY: matcher-produced sequences and `last_literals` partition `src`,
    // and the repeat history belongs to that same matcher transaction.
    unsafe {
        crate::kernel::seqstore::prepare_stored_sequences_in(
            src,
            initial_repeat_offsets.as_offsets(),
            sequence_words,
            last_literals,
            reuse,
        )
    }
}

pub(crate) fn prepare_stored_literal_words_in(
    src: &[u8],
    sequences: &[StoredSequence],
    last_literals: u32,
    reuse: PreparedStoreWords,
) -> PreparedStoreWords {
    // SAFETY: the compile-time layout assertions above prove that the local C
    // record has exactly the primitive word-array ABI expected by the leaf
    // codegen crate.
    let sequence_words = unsafe {
        slice::from_raw_parts(
            sequences
                .as_ptr()
                .cast::<crate::kernel::seqstore::StoredSequenceWords>(),
            sequences.len(),
        )
    };
    // SAFETY: matcher-produced lengths and `last_literals` partition `src`.
    unsafe {
        crate::kernel::seqstore::prepare_stored_literals_in(
            src,
            sequence_words,
            last_literals,
            reuse,
        )
    }
}

pub(crate) fn prepared_words_as_ref(prepared: &PreparedStoreWords) -> PreparedBlockRef<'_> {
    // SAFETY: the compile-time layout assertions prove that every initialized
    // primitive word record is a valid local `PreparedSequence`.
    let sequences = unsafe {
        slice::from_raw_parts(
            prepared.sequences.as_ptr().cast::<PreparedSequence>(),
            prepared.sequences.len(),
        )
    };
    PreparedBlockRef {
        literals: &prepared.literals,
        sequences,
    }
}

pub(crate) fn prepared_words_into_block(prepared: PreparedStoreWords) -> PreparedBlock {
    PreparedBlock {
        literals: prepared.literals.into_owned_vec(),
        // SAFETY: the compile-time layout assertions prove the returned word
        // arrays and local prepared records have identical size and alignment,
        // and every returned element is initialized.
        sequences: unsafe { prepared_words_into_sequences(prepared.sequences.into_owned_vec()) },
    }
}

pub(crate) type PreparedBlockLease = (
    crate::workspace::VecLease<u8>,
    crate::workspace::VecLease<crate::kernel::seqstore::PreparedSequenceWords>,
);

pub(crate) fn lease_prepared_words(
    prepared: PreparedStoreWords,
) -> (PreparedBlock, PreparedBlockLease) {
    let (literals, literal_lease) = prepared.literals.lease_vec();
    let (sequences, sequence_lease) = prepared.sequences.lease_vec();
    let prepared = PreparedBlock {
        literals,
        // SAFETY: the compile-time layout assertions prove that the word
        // arrays and prepared records have identical layout.
        sequences: unsafe { prepared_words_into_sequences(sequences) },
    };
    (prepared, (literal_lease, sequence_lease))
}

pub(crate) fn recover_prepared_words(
    prepared: PreparedBlock,
    leases: PreparedBlockLease,
) -> PreparedStoreWords {
    let sequences = unsafe { prepared_sequences_into_words(prepared.sequences) };
    PreparedStoreWords {
        literals: crate::workspace::ReusableVec::recover_vec(prepared.literals, leases.0),
        sequences: crate::workspace::ReusableVec::recover_vec(sequences, leases.1),
    }
}

pub(crate) fn prepared_block_into_words(prepared: PreparedBlock) -> PreparedStoreWords {
    PreparedStoreWords {
        literals: crate::workspace::ReusableVec::from_owned(prepared.literals),
        // SAFETY: the compile-time layout assertions prove that the local
        // prepared records and primitive word arrays have identical layout.
        // Ownership moves into the returned vector without duplication.
        sequences: crate::workspace::ReusableVec::from_owned(unsafe {
            prepared_sequences_into_words(prepared.sequences)
        }),
    }
}

unsafe fn prepared_words_into_sequences(
    sequences: Vec<crate::kernel::seqstore::PreparedSequenceWords>,
) -> Vec<PreparedSequence> {
    let mut sequences = ManuallyDrop::new(sequences);
    // SAFETY: the caller establishes identical element layout, and ownership
    // of the allocation transfers to the returned vector without duplication.
    unsafe {
        Vec::from_raw_parts(
            sequences.as_mut_ptr().cast::<PreparedSequence>(),
            sequences.len(),
            sequences.capacity(),
        )
    }
}

unsafe fn prepared_sequences_into_words(
    sequences: Vec<PreparedSequence>,
) -> Vec<crate::kernel::seqstore::PreparedSequenceWords> {
    let mut sequences = ManuallyDrop::new(sequences);
    // SAFETY: the caller establishes identical element layout, and ownership
    // of the allocation transfers to the returned vector without duplication.
    unsafe {
        Vec::from_raw_parts(
            sequences.as_mut_ptr().cast(),
            sequences.len(),
            sequences.capacity(),
        )
    }
}

/// Safe short-literal analogue of C's `ZSTD_storeSeq()` wild copy.
///
/// C writes at least 16 bytes and advances by only the logical literal length.
/// The sequence store reserves padding for that overcopy. Rust keeps the same
/// cadence for the common short lengths when the source has enough bytes, then
/// truncates the initialized overcopy before the next sequence is appended.
pub(super) fn append_sequence_literals(output: &mut Vec<u8>, src: &[u8], start: usize, len: usize) {
    if len == 0 {
        return;
    }

    let logical_end = start + len;
    debug_assert!(logical_end <= src.len());
    let output_start = output.len();
    let available = src.len() - start;

    match len {
        1..=16 if available >= 16 => output.extend_from_slice(&src[start..start + 16]),
        17..=32 if available >= 32 => output.extend_from_slice(&src[start..start + 32]),
        33..=48 if available >= 48 => output.extend_from_slice(&src[start..start + 48]),
        49..=64 if available >= 64 => output.extend_from_slice(&src[start..start + 64]),
        _ => output.extend_from_slice(&src[start..logical_end]),
    }
    output.truncate(output_start + len);
}
