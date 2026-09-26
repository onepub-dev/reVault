#[cfg(test)]
use alloc::vec::Vec;
use core::{mem::MaybeUninit, ptr};

use crate::workspace::{Arena, ArenaError, ArenaSize, ReusableVec};

pub type StoredSequenceWords = [u32; 3];
pub type PreparedSequenceWords = [u32; 4];

const REPCODE_COUNT: u32 = 3;
const LITERAL_WILDCOPY_MAX: usize = 64;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PreparedStoreWords {
    pub literals: ReusableVec<u8>,
    pub sequences: ReusableVec<PreparedSequenceWords>,
}

impl PreparedStoreWords {
    pub(crate) fn add_workspace_size(
        size: &mut ArenaSize,
        literal_capacity: usize,
        sequence_capacity: usize,
    ) -> Result<(), ArenaError> {
        size.add::<u8>(literal_capacity)?;
        size.add::<PreparedSequenceWords>(sequence_capacity)
    }

    pub(crate) fn new_in(
        arena: &mut Arena<'_>,
        literal_capacity: usize,
        sequence_capacity: usize,
    ) -> Result<Self, ArenaError> {
        Ok(Self {
            literals: arena.allocate_reusable_vec(literal_capacity)?,
            sequences: arena.allocate_reusable_vec(sequence_capacity)?,
        })
    }

    pub fn clear(&mut self) {
        self.literals.clear();
        self.sequences.clear();
    }

    #[cfg(test)]
    pub fn allocation(&self) -> ((*const u8, usize), (*const PreparedSequenceWords, usize)) {
        (
            (self.literals.as_ptr(), self.literals.capacity()),
            (self.sequences.as_ptr(), self.sequences.capacity()),
        )
    }
}

/// Ports the C `SeqStore_t` post-match preparation transaction behind a
/// primitive ABI. Word order is `{litLength, matchLength, offBase}` on input
/// and `{litLength, matchLength, rawOffset, offBase}` on output.
/// # Safety
///
/// The sequence lengths must partition `src` together with `last_literals`,
/// and every offset code must be valid for the supplied repeat history.
#[cfg(test)]
pub unsafe fn prepare_stored_sequences(
    src: &[u8],
    initial_repeat_offsets: [u32; 3],
    sequences: &[StoredSequenceWords],
    last_literals: u32,
) -> PreparedStoreWords {
    unsafe {
        prepare_stored_sequences_in(
            src,
            initial_repeat_offsets,
            sequences,
            last_literals,
            PreparedStoreWords::default(),
        )
    }
}

/// Reuses both owning allocations from `reuse` while rebuilding the complete
/// prepared store.
/// # Safety
///
/// The sequence lengths must partition `src` together with `last_literals`,
/// and every offset code must be valid for the supplied repeat history.
pub unsafe fn prepare_stored_sequences_in(
    src: &[u8],
    initial_repeat_offsets: [u32; 3],
    sequences: &[StoredSequenceWords],
    last_literals: u32,
    reuse: PreparedStoreWords,
) -> PreparedStoreWords {
    let mut output = PreparedStoreWriter::new(
        src.len().saturating_add(LITERAL_WILDCOPY_MAX),
        sequences.len(),
        reuse,
    );
    let mut repeat_offsets = initial_repeat_offsets;
    let mut anchor = 0_usize;

    for &[lit_len, match_len, off_base] in sequences {
        let literal_length = lit_len as usize;
        let match_length = match_len as usize;
        let lit_end = anchor + literal_length;
        debug_assert!(lit_end <= src.len());
        output.append_wildcopy_literals(src, anchor, literal_length);

        let raw_offset = resolve_and_update(&mut repeat_offsets, off_base, lit_len);
        output.push_sequence([lit_len, match_len, raw_offset, off_base]);
        anchor = lit_end + match_length;
        debug_assert!(anchor <= src.len());
    }

    let tail_end = anchor + last_literals as usize;
    debug_assert_eq!(tail_end, src.len());
    output.append_exact_literals(&src[anchor..tail_end]);
    output.finish()
}

/// Gathers the literal stream from C's native sequence store without
/// materializing entropy-ready sequence records.
///
/// This is the direct Fast/DFast path: match collection already produced LL,
/// ML, and numeric `offBase`, so only C's interleaved literal buffer remains
/// to be assembled before entropy encoding.
/// # Safety
///
/// The sequence lengths must partition `src` together with `last_literals`.
pub unsafe fn prepare_stored_literals_in(
    src: &[u8],
    sequences: &[StoredSequenceWords],
    last_literals: u32,
    reuse: PreparedStoreWords,
) -> PreparedStoreWords {
    let mut output =
        PreparedStoreWriter::new(src.len().saturating_add(LITERAL_WILDCOPY_MAX), 0, reuse);
    let mut anchor = 0_usize;

    for &[lit_len, match_len, _off_base] in sequences {
        let literal_length = lit_len as usize;
        let lit_end = anchor + literal_length;
        debug_assert!(lit_end <= src.len());
        output.append_wildcopy_literals(src, anchor, literal_length);
        anchor = lit_end + match_len as usize;
        debug_assert!(anchor <= src.len());
    }

    let tail_end = anchor + last_literals as usize;
    debug_assert_eq!(tail_end, src.len());
    output.append_exact_literals(&src[anchor..tail_end]);
    output.finish()
}

#[inline(always)]
fn resolve_and_update(offsets: &mut [u32; 3], off_base: u32, lit_len: u32) -> u32 {
    debug_assert!(off_base > 0);
    if off_base > REPCODE_COUNT {
        let raw_offset = off_base - REPCODE_COUNT;
        offsets[2] = offsets[1];
        offsets[1] = offsets[0];
        offsets[0] = raw_offset;
        return raw_offset;
    }

    let rep_code = off_base - 1 + u32::from(lit_len == 0);
    if rep_code == 0 {
        return offsets[0];
    }

    let raw_offset = if rep_code == REPCODE_COUNT {
        offsets[0] - 1
    } else {
        offsets[rep_code as usize]
    };
    if rep_code >= 2 {
        offsets[2] = offsets[1];
    }
    offsets[1] = offsets[0];
    offsets[0] = raw_offset;
    raw_offset
}

struct PreparedStoreWriter {
    literals: ReusableVec<MaybeUninit<u8>>,
    literal_len: usize,
    sequences: ReusableVec<MaybeUninit<PreparedSequenceWords>>,
    sequence_len: usize,
}

impl PreparedStoreWriter {
    fn new(literal_capacity: usize, sequence_capacity: usize, reuse: PreparedStoreWords) -> Self {
        let mut literals = into_uninit(reuse.literals);
        if literals.capacity() < literal_capacity {
            literals.reserve(literal_capacity);
        }
        let mut sequences = into_uninit(reuse.sequences);
        if sequences.capacity() < sequence_capacity {
            sequences.reserve(sequence_capacity);
        }
        Self {
            literals,
            literal_len: 0,
            sequences,
            sequence_len: 0,
        }
    }

    fn append_wildcopy_literals(&mut self, src: &[u8], start: usize, len: usize) {
        if len == 0 {
            return;
        }

        let available = src.len() - start;
        let stored_len = match len {
            1..=16 if available >= 16 => 16,
            17..=32 if available >= 32 => 32,
            33..=48 if available >= 48 => 48,
            49..=64 if available >= 64 => 64,
            _ => len,
        };
        assert!(stored_len <= available);
        assert!(self.literal_len + stored_len <= self.literals.capacity());

        // SAFETY: both ranges were proved in bounds above, the destination is
        // byte-aligned, and source/output allocations cannot overlap.
        unsafe {
            ptr::copy_nonoverlapping(
                src.as_ptr().add(start),
                self.literals
                    .as_mut_ptr()
                    .add(self.literal_len)
                    .cast::<u8>(),
                stored_len,
            );
        }
        self.literal_len += len;
    }

    fn append_exact_literals(&mut self, src: &[u8]) {
        assert!(self.literal_len + src.len() <= self.literals.capacity());

        // SAFETY: the assertion proves the destination range, and the
        // source/output allocations cannot overlap.
        unsafe {
            ptr::copy_nonoverlapping(
                src.as_ptr(),
                self.literals
                    .as_mut_ptr()
                    .add(self.literal_len)
                    .cast::<u8>(),
                src.len(),
            );
        }
        self.literal_len += src.len();
    }

    fn push_sequence(&mut self, sequence: PreparedSequenceWords) {
        assert!(self.sequence_len < self.sequences.capacity());

        // SAFETY: the assertion proves the aligned slot is allocated. Each
        // sequence slot is written exactly once.
        unsafe {
            self.sequences
                .as_mut_ptr()
                .add(self.sequence_len)
                .write(MaybeUninit::new(sequence));
        }
        self.sequence_len += 1;
    }

    fn finish(self) -> PreparedStoreWords {
        PreparedStoreWords {
            // SAFETY: literal writes cover the complete logical prefix.
            literals: unsafe { assume_init_prefix(self.literals, self.literal_len) },
            // SAFETY: `sequence_len` tracks the contiguous initialized prefix.
            sequences: unsafe { assume_init_prefix(self.sequences, self.sequence_len) },
        }
    }
}

fn into_uninit<T>(mut values: ReusableVec<T>) -> ReusableVec<MaybeUninit<T>> {
    values.clear();
    values.into_uninit()
}

unsafe fn assume_init_prefix<T>(
    mut values: ReusableVec<MaybeUninit<T>>,
    initialized_len: usize,
) -> ReusableVec<T> {
    debug_assert!(initialized_len <= values.capacity());
    // SAFETY: the caller proves every exposed prefix element is initialized.
    unsafe { values.set_len(initialized_len) };
    // SAFETY: the initialized prefix invariant was established above.
    unsafe { values.assume_init() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepared_store_wildcopy_matches_exact_literals_at_every_boundary() {
        let src = (0usize..160).map(|value| value as u8).collect::<Vec<_>>();

        for len in [0, 1, 16, 17, 32, 33, 48, 49, 64, 65, 96] {
            let sequences = [[len, 4, 4]];
            // SAFETY: the fixture sequence exactly partitions this source.
            let prepared = unsafe {
                prepare_stored_sequences(&src[..len as usize + 4], [1, 4, 8], &sequences, 0)
            };
            assert_eq!(prepared.literals.as_slice(), &src[..len as usize]);
            assert_eq!(prepared.sequences, [[len, 4, 1, 4]]);
        }
    }

    #[test]
    fn prepared_store_wildcopy_falls_back_near_source_end() {
        let src = (0usize..80).map(|value| value as u8).collect::<Vec<_>>();

        for len in [1, 15, 31, 47, 63] {
            // SAFETY: the literal-only fixture exactly partitions this source.
            let prepared = unsafe {
                prepare_stored_sequences(&src[..len as usize], [1, 4, 8], &[[len, 0, 4]], 0)
            };
            assert_eq!(prepared.literals.as_slice(), &src[..len as usize]);
        }
    }

    #[test]
    fn numeric_repeat_transition_matches_c_rules() {
        assert_eq!(
            unsafe { prepare_stored_sequences(&[0; 12], [11, 22, 33], &[[0, 4, 1], [0, 4, 2]], 4) }
                .sequences,
            [[0, 4, 22, 1], [0, 4, 33, 2]]
        );
        assert_eq!(
            unsafe { prepare_stored_sequences(&[0; 4], [11, 22, 33], &[[0, 4, 3]], 0) }.sequences,
            [[0, 4, 10, 3]]
        );
    }

    #[test]
    fn prepared_store_reuses_both_allocations() {
        let src = [0_u8; 128];
        // SAFETY: the fixture sequence plus tail exactly partitions `src`.
        let first = unsafe { prepare_stored_sequences(&src, [1, 4, 8], &[[32, 32, 4]], 64) };
        let allocation = first.allocation();
        assert!(allocation.0 .1 >= src.len());
        assert!(allocation.1 .1 >= 1);

        // SAFETY: the same validated fixture is reused.
        let second =
            unsafe { prepare_stored_sequences_in(&src, [1, 4, 8], &[[32, 32, 4]], 64, first) };

        assert_eq!(second.allocation(), allocation);
        assert_eq!(second.literals.len(), 96);
        assert_eq!(second.sequences, [[32, 32, 1, 4]]);
    }

    #[test]
    fn literal_only_store_matches_full_preparation_and_preserves_sequence_allocation() {
        let src = (0usize..128).map(|value| value as u8).collect::<Vec<_>>();
        let sequences = [[12, 8, 4], [17, 16, 7], [0, 4, 1]];
        // SAFETY: the fixture sequences plus tail exactly partition `src`.
        let full = unsafe { prepare_stored_sequences(&src, [1, 4, 8], &sequences, 71) };
        let expected_literals = full.literals.clone();
        let allocation = full.allocation();

        // SAFETY: the same validated fixture is reused.
        let literals = unsafe { prepare_stored_literals_in(&src, &sequences, 71, full) };

        assert_eq!(literals.literals, expected_literals);
        assert!(literals.sequences.is_empty());
        assert_eq!(literals.allocation(), allocation);
    }
}
