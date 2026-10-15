//! Audited raw table access for C's row-based Greedy/Lazy match finder.
//!
//! `GreedyMatchState::ensure_tables()` allocates both row tables to exactly
//! `1 << hash_log`. Every `row_start` is produced from the high `hash_log -
//! row_log` hash bits and shifted by the same `row_log`; row positions are
//! masked to `0..(1 << row_log)`. Keeping this boundary here mirrors C's raw
//! row pointers without spreading unchecked indexing through parser code.

#[inline(always)]
pub(super) fn tags(table: &[u8], row_start: usize, row_entries: usize) -> &[u8] {
    debug_assert!(row_entries.is_power_of_two());
    debug_assert!(row_start <= table.len());
    debug_assert!(row_entries <= table.len() - row_start);
    // SAFETY: Row hashes select a complete aligned row inside the table sized
    // by `ensure_tables()`. The assertions state the full range invariant.
    unsafe { core::slice::from_raw_parts(table.as_ptr().add(row_start), row_entries) }
}

#[inline(always)]
pub(super) fn entry(table: &[u32], row_start: usize, position: usize, row_mask: usize) -> u32 {
    debug_assert!(position <= row_mask);
    debug_assert!(row_start <= table.len());
    debug_assert!(row_mask < table.len() - row_start);
    // SAFETY: `position` is masked to this complete row and the row lies in
    // the exactly-sized hash table.
    unsafe { *table.get_unchecked(row_start + position) }
}

#[inline(always)]
pub(super) fn next_index(tag_table: &mut [u8], row_start: usize, row_mask: usize) -> usize {
    debug_assert!(row_start < tag_table.len());
    debug_assert!(row_mask < tag_table.len() - row_start);
    // SAFETY: `row_start` identifies the first byte of a complete row.
    let head = unsafe { tag_table.get_unchecked_mut(row_start) };
    let mut next = usize::from(head.wrapping_sub(1)) & row_mask;
    if next == 0 {
        next = row_mask;
    }
    *head = next as u8;
    next
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
pub(super) fn insert(
    hash_table: &mut [u32],
    tag_table: &mut [u8],
    row_start: usize,
    position: usize,
    row_mask: usize,
    tag: u8,
    index: u32,
) {
    debug_assert!(position > 0);
    debug_assert!(position <= row_mask);
    debug_assert!(row_start <= hash_table.len());
    debug_assert!(row_mask < hash_table.len() - row_start);
    debug_assert!(row_start <= tag_table.len());
    debug_assert!(row_mask < tag_table.len() - row_start);
    let slot = row_start + position;
    // SAFETY: `position` is in the complete row range established above.
    unsafe {
        *tag_table.get_unchecked_mut(slot) = tag;
        *hash_table.get_unchecked_mut(slot) = index;
    }
}

#[cfg(test)]
mod tests {
    use super::{entry, insert, next_index, tags};

    #[test]
    fn accesses_complete_first_and_last_rows() {
        let mut hashes = [0u32; 32];
        let mut tags_table = [0u8; 32];

        let first = next_index(&mut tags_table, 0, 15);
        insert(&mut hashes, &mut tags_table, 0, first, 15, 7, 11);
        assert_eq!(entry(&hashes, 0, first, 15), 11);
        assert_eq!(tags(&tags_table, 0, 16)[first], 7);

        let last = next_index(&mut tags_table, 16, 15);
        insert(&mut hashes, &mut tags_table, 16, last, 15, 9, 29);
        assert_eq!(entry(&hashes, 16, last, 15), 29);
        assert_eq!(tags(&tags_table, 16, 16)[last], 9);
    }
}
