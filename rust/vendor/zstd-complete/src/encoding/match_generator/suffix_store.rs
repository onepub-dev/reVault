use alloc::vec::Vec;
use core::convert::TryFrom;
use core::num::NonZeroU32;

pub(super) const INITIAL_TOUCHED_SLOT_CAPACITY: usize = 1024;
pub(super) const TOUCHED_SLOT_CLEAR_LIMIT: usize = 32 * 1024;

/// This stores the index of a suffix of a string by hashing the first few bytes of that suffix
/// This means that collisions just overwrite and that you need to check validity after a get
pub(super) struct SuffixStore {
    pub(super) slots: Vec<Option<Candidates>>,
    pub(super) touched_slots: Vec<u32>,
    pub(super) clear_all_slots: bool,
    pub(super) len_log: u32,
}

#[derive(Copy, Clone)]
pub(super) struct Candidates {
    // We need 17 bits per index to store the maximum block size of 128kb.
    // We store indexes using one-based so Option can use a NonZeroU32 niche.
    pub(super) oldest: NonZeroU32,
    pub(super) newest: NonZeroU32,
}

pub(super) struct CandidateIndexes {
    pub(super) oldest: usize,
    pub(super) newest: Option<usize>,
}

impl SuffixStore {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        let capacity = Self::normalized_capacity(capacity);
        Self {
            slots: alloc::vec![None; capacity],
            touched_slots: Vec::with_capacity(INITIAL_TOUCHED_SLOT_CAPACITY),
            clear_all_slots: false,
            len_log: capacity.ilog2(),
        }
    }

    pub(super) fn normalized_capacity(capacity: usize) -> usize {
        capacity.max(2)
    }

    pub(super) fn capacity(&self) -> usize {
        self.slots.len()
    }

    #[inline(always)]
    pub(super) fn insert(&mut self, suffix: &[u8], idx: usize) {
        let key = self.slot_key(Self::key_value(suffix)).index;
        let idx = Self::stored_index(idx);
        if let Some(slot) = self.slots[key] {
            self.slots[key] = Some(Candidates {
                oldest: slot.oldest,
                newest: idx,
            });
        } else {
            self.record_touched_slot(key);
            self.slots[key] = Some(Candidates {
                oldest: idx,
                newest: idx,
            });
        }
    }

    pub(super) fn clear(&mut self) {
        if self.clear_all_slots {
            self.slots.fill(None);
            self.touched_slots.clear();
            self.clear_all_slots = false;
            return;
        }

        for key in self.touched_slots.drain(..) {
            self.slots[key as usize] = None;
        }
    }

    #[inline(always)]
    pub(super) fn record_touched_slot(&mut self, key: usize) {
        if self.clear_all_slots {
            return;
        }
        if self.touched_slots.len() == TOUCHED_SLOT_CLEAR_LIMIT {
            self.touched_slots.clear();
            self.clear_all_slots = true;
            return;
        }
        self.touched_slots.push(Self::stored_slot_key(key));
    }

    #[inline(always)]
    fn stored_slot_key(key: usize) -> u32 {
        match u32::try_from(key) {
            Ok(key) => key,
            Err(_) => Self::invalid_stored_index(),
        }
    }

    #[inline(always)]
    pub(super) fn stored_index(idx: usize) -> NonZeroU32 {
        let Some(idx) = idx.checked_add(1) else {
            Self::invalid_stored_index()
        };
        let Ok(idx) = u32::try_from(idx) else {
            Self::invalid_stored_index()
        };
        let Some(idx) = NonZeroU32::new(idx) else {
            Self::invalid_stored_index()
        };
        idx
    }

    #[cold]
    #[inline(never)]
    fn invalid_stored_index() -> ! {
        panic!("suffix index must fit in non-zero u32")
    }

    #[cfg(test)]
    #[inline(always)]
    pub(super) fn candidates(&self, suffix: &[u8]) -> Option<CandidateIndexes> {
        self.candidates_for_key_value(Self::key_value(suffix))
    }

    #[inline(always)]
    pub(super) fn candidates_for_key_value(&self, value: u64) -> Option<CandidateIndexes> {
        self.candidates_for_slot_key(self.slot_key(value))
    }

    #[inline(always)]
    pub(super) fn candidates_for_slot_key(&self, slot_key: SlotKey) -> Option<CandidateIndexes> {
        let key = slot_key.index;
        let slot = self.slots[key]?;
        let oldest = slot.oldest.get() as usize - 1;
        let newest = slot.newest.get() as usize - 1;
        let newest = if oldest == newest { None } else { Some(newest) };
        Some(CandidateIndexes { oldest, newest })
    }

    #[cfg(test)]
    #[inline(always)]
    pub(super) fn key(&self, suffix: &[u8]) -> usize {
        self.slot_key(Self::key_value(suffix)).index
    }

    #[inline(always)]
    pub(super) fn key_value(suffix: &[u8]) -> u64 {
        u64::from(suffix[0])
            | (u64::from(suffix[1]) << 8)
            | (u64::from(suffix[2]) << 16)
            | (u64::from(suffix[3]) << 24)
            | (u64::from(suffix[4]) << 32)
    }

    #[cfg(test)]
    #[inline(always)]
    pub(super) fn key_from_value(&self, value: u64) -> usize {
        self.slot_key(value).index
    }

    #[inline(always)]
    pub(super) fn slot_key(&self, value: u64) -> SlotKey {
        let hash = value.wrapping_mul(0x9E37_79B1_85EB_CA87);
        let index = (hash >> (64 - self.len_log)) as usize;
        SlotKey { index }
    }
}

#[derive(Copy, Clone)]
pub(super) struct SlotKey {
    pub(super) index: usize,
}
