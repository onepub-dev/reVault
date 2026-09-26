//! Audited table access for the C double-fast match finder.
//!
//! `DFastMatchState::ensure_tables()` sizes each table to exactly `1 << log`,
//! while every slot passed here is produced by a hash using that same log.
//! Keeping the unchecked boundary here mirrors C's raw table access without
//! spreading unsafe indexing through the generated matcher bodies.

#[inline(always)]
pub(super) fn entry(table: &[u32], slot: usize) -> u32 {
    debug_assert!(slot < table.len());
    // SAFETY: DFast tables have `1 << hash_log` entries and callers pass a
    // hash reduced to that log. `ensure_tables()` establishes the invariant
    // before the matcher borrows either table.
    unsafe { *table.get_unchecked(slot) }
}

#[inline(always)]
pub(super) fn set_entry(table: &mut [u32], slot: usize, value: u32) {
    debug_assert!(slot < table.len());
    // SAFETY: Same table-size/hash-range invariant as `entry()`.
    unsafe { *table.get_unchecked_mut(slot) = value }
}

#[cfg(test)]
mod tests {
    use super::{entry, set_entry};

    #[test]
    fn accesses_first_and_last_valid_slots() {
        let mut table = [0_u32; 8];
        set_entry(&mut table, 0, 11);
        set_entry(&mut table, 7, 29);
        assert_eq!(entry(&table, 0), 11);
        assert_eq!(entry(&table, 7), 29);
    }
}
