use super::params::CompressionParameters;
use super::row_match::{row_log, row_match_finder_enabled};
use super::sequence_store::{PreparedStoreWords, StoredSequence};
use crate::workspace::{Arena, ArenaError, ArenaSize, ReusableVec};
use crate::{fse::fse_encoder::FSETableBuildScratch, huff0::huff0_encoder::HuffmanBuildScratch};

const LONG_MATCH_UPDATE_GAP: usize = 384;
const LONG_MATCH_UPDATE_LIMIT: usize = 192;

pub(crate) struct GreedyMatchState {
    pub(super) hash_table: ReusableVec<u32>,
    pub(super) hash_table3: ReusableVec<u32>,
    pub(super) chain_table: ReusableVec<u32>,
    pub(super) hash_log: u32,
    pub(super) hash_log3: u32,
    pub(super) chain_log: u32,
    pub(super) row_log: u32,
    pub(super) next_to_update: usize,
    pub(super) next_to_update3: usize,
    pub(super) lazy_skipping: bool,
    pub(super) tag_table: ReusableVec<u8>,
    pub(super) hash_salt: u64,
    pub(super) hash_salt_entropy: u32,
    pub(super) row_hash_cache: [u32; 8],
    sequence_store: ReusableVec<StoredSequence>,
    prepared_store: PreparedStoreWords,
    block_bytes: ReusableVec<u8>,
    pub(super) entropy_huffman_scratch: HuffmanBuildScratch,
    pub(super) entropy_fse_scratch: FSETableBuildScratch,
}

impl Clone for GreedyMatchState {
    fn clone(&self) -> Self {
        Self {
            hash_table: self.hash_table.clone(),
            hash_table3: self.hash_table3.clone(),
            chain_table: self.chain_table.clone(),
            hash_log: self.hash_log,
            hash_log3: self.hash_log3,
            chain_log: self.chain_log,
            row_log: self.row_log,
            next_to_update: self.next_to_update,
            next_to_update3: self.next_to_update3,
            lazy_skipping: self.lazy_skipping,
            tag_table: self.tag_table.clone(),
            hash_salt: self.hash_salt,
            hash_salt_entropy: self.hash_salt_entropy,
            row_hash_cache: self.row_hash_cache,
            sequence_store: self.sequence_store.clone(),
            prepared_store: self.prepared_store.clone(),
            block_bytes: self.block_bytes.clone(),
            entropy_huffman_scratch: HuffmanBuildScratch::new(),
            entropy_fse_scratch: FSETableBuildScratch::new(),
        }
    }
}

impl core::fmt::Debug for GreedyMatchState {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("GreedyMatchState")
            .field("hash_log", &self.hash_log)
            .field("hash_log3", &self.hash_log3)
            .field("chain_log", &self.chain_log)
            .field("next_to_update", &self.next_to_update)
            .field("next_to_update3", &self.next_to_update3)
            .finish_non_exhaustive()
    }
}

impl PartialEq for GreedyMatchState {
    fn eq(&self, other: &Self) -> bool {
        self.hash_table == other.hash_table
            && self.hash_table3 == other.hash_table3
            && self.chain_table == other.chain_table
            && self.hash_log == other.hash_log
            && self.hash_log3 == other.hash_log3
            && self.chain_log == other.chain_log
            && self.row_log == other.row_log
            && self.next_to_update == other.next_to_update
            && self.next_to_update3 == other.next_to_update3
            && self.lazy_skipping == other.lazy_skipping
            && self.tag_table == other.tag_table
            && self.hash_salt == other.hash_salt
            && self.hash_salt_entropy == other.hash_salt_entropy
            && self.row_hash_cache == other.row_hash_cache
            && self.sequence_store == other.sequence_store
            && self.prepared_store == other.prepared_store
            && self.block_bytes == other.block_bytes
    }
}

impl Eq for GreedyMatchState {}

impl GreedyMatchState {
    pub(crate) fn new() -> Self {
        Self {
            hash_table: ReusableVec::new(),
            hash_table3: ReusableVec::new(),
            chain_table: ReusableVec::new(),
            hash_log: 0,
            hash_log3: 0,
            chain_log: 0,
            row_log: 0,
            next_to_update: 0,
            next_to_update3: 0,
            lazy_skipping: false,
            tag_table: ReusableVec::new(),
            hash_salt: 0,
            hash_salt_entropy: 0,
            row_hash_cache: [0; 8],
            sequence_store: ReusableVec::new(),
            prepared_store: PreparedStoreWords::default(),
            block_bytes: ReusableVec::new(),
            entropy_huffman_scratch: HuffmanBuildScratch::new(),
            entropy_fse_scratch: FSETableBuildScratch::new(),
        }
    }

    pub(crate) fn add_workspace_size(
        size: &mut ArenaSize,
        params: CompressionParameters,
        block_size: usize,
    ) -> Result<(), ArenaError> {
        let hash_size = 1_usize << params.hash_log;
        let hash3_size = if params.min_match == 3 {
            1_usize << params.window_log.min(17)
        } else {
            0
        };
        let row_enabled = row_match_finder_enabled(params);
        let chain_size = if row_enabled {
            0
        } else {
            (1_usize << params.chain_log) + 1
        };
        let sequence_capacity = block_size / 3 + 1;
        size.add::<u32>(hash_size)?;
        size.add::<u32>(hash3_size)?;
        size.add::<u32>(chain_size)?;
        size.add::<u8>(if row_enabled { hash_size } else { 0 })?;
        size.add::<StoredSequence>(sequence_capacity)?;
        PreparedStoreWords::add_workspace_size(size, block_size.saturating_add(64), 0)?;
        size.add::<u8>(super::compress_bound::compress_bound(block_size))?;
        HuffmanBuildScratch::add_workspace_size(size)?;
        FSETableBuildScratch::add_workspace_size(size)
    }

    pub(crate) fn new_in(
        arena: &mut Arena<'_>,
        params: CompressionParameters,
        block_size: usize,
    ) -> Result<Self, ArenaError> {
        let hash_size = 1_usize << params.hash_log;
        let hash_log3 = if params.min_match == 3 {
            params.window_log.min(17)
        } else {
            0
        };
        let hash3_size = if hash_log3 > 0 {
            1_usize << hash_log3
        } else {
            0
        };
        let row_enabled = row_match_finder_enabled(params);
        let chain_size = if row_enabled {
            0
        } else {
            (1_usize << params.chain_log) + 1
        };
        let sequence_capacity = block_size / 3 + 1;
        let mut state = Self {
            hash_table: arena.allocate_reusable_vec(hash_size)?,
            hash_table3: arena.allocate_reusable_vec(hash3_size)?,
            chain_table: arena.allocate_reusable_vec(chain_size)?,
            hash_log: params.hash_log,
            hash_log3,
            chain_log: params.chain_log,
            row_log: row_log(params),
            next_to_update: 0,
            next_to_update3: 0,
            lazy_skipping: false,
            tag_table: arena.allocate_reusable_vec(if row_enabled { hash_size } else { 0 })?,
            hash_salt: 0,
            hash_salt_entropy: 0,
            row_hash_cache: [0; 8],
            sequence_store: arena.allocate_reusable_vec(sequence_capacity)?,
            prepared_store: PreparedStoreWords::new_in(arena, block_size.saturating_add(64), 0)?,
            block_bytes: arena
                .allocate_reusable_vec(super::compress_bound::compress_bound(block_size))?,
            entropy_huffman_scratch: HuffmanBuildScratch::new_in(arena)?,
            entropy_fse_scratch: FSETableBuildScratch::new_in(arena)?,
        };
        state.hash_table.resize(hash_size, 0);
        state.hash_table3.resize(hash3_size, 0);
        state.chain_table.resize(chain_size, 0);
        state
            .tag_table
            .resize(if row_enabled { hash_size } else { 0 }, 0);
        Ok(state)
    }

    pub(super) fn take_prepared_store(&mut self) -> PreparedStoreWords {
        core::mem::take(&mut self.prepared_store)
    }

    pub(super) fn recycle_prepared_store(&mut self, mut prepared: PreparedStoreWords) {
        prepared.clear();
        self.prepared_store = prepared;
    }

    pub(super) fn take_block_bytes(&mut self) -> ReusableVec<u8> {
        let mut bytes = core::mem::take(&mut self.block_bytes);
        bytes.clear();
        bytes
    }

    pub(super) fn recycle_block_bytes(&mut self, mut bytes: ReusableVec<u8>) {
        bytes.clear();
        self.block_bytes = bytes;
    }

    pub(super) fn take_sequence_store(&mut self) -> ReusableVec<StoredSequence> {
        let mut sequences = core::mem::take(&mut self.sequence_store);
        sequences.clear();
        sequences
    }

    pub(super) fn recycle_sequence_store(&mut self, mut sequences: ReusableVec<StoredSequence>) {
        sequences.clear();
        if sequences.capacity() > self.sequence_store.capacity() {
            self.sequence_store = sequences;
        }
    }

    #[cfg(test)]
    pub(super) fn sequence_store_allocation(&self) -> (*const StoredSequence, usize) {
        (self.sequence_store.as_ptr(), self.sequence_store.capacity())
    }

    pub(crate) fn reset_for_frame(&mut self, params: CompressionParameters) {
        self.next_to_update = 0;
        self.next_to_update3 = 0;
        self.lazy_skipping = false;
        self.row_hash_cache = [0; 8];
        self.hash_table.fill(0);
        self.hash_table3.fill(0);
        self.chain_table.fill(0);
        if row_match_finder_enabled(params) {
            self.advance_hash_salt();
        } else {
            self.hash_salt = 0;
        }
    }

    pub(super) fn ensure_tables(&mut self, params: CompressionParameters) {
        if self.hash_log != params.hash_log {
            self.hash_log = params.hash_log;
            self.hash_table.clear();
            self.next_to_update = 0;
            self.row_hash_cache = [0; 8];
        }
        let hash_log3 = if params.min_match == 3 {
            params.window_log.min(17)
        } else {
            0
        };
        if self.hash_log3 != hash_log3 {
            self.hash_log3 = hash_log3;
            self.hash_table3.clear();
            self.next_to_update3 = 0;
        }
        if self.chain_log != params.chain_log {
            self.chain_log = params.chain_log;
            self.chain_table.clear();
            self.next_to_update = 0;
        }

        let hash_size = 1_usize << params.hash_log;
        if self.hash_table.len() != hash_size {
            self.hash_table.resize(hash_size, 0);
        }

        let hash3_size = if self.hash_log3 > 0 {
            1_usize << self.hash_log3
        } else {
            0
        };
        if self.hash_table3.len() != hash3_size {
            self.hash_table3.resize(hash3_size, 0);
        }

        let row_match_enabled = row_match_finder_enabled(params);
        let chain_size = if row_match_enabled {
            0
        } else {
            (1_usize << params.chain_log) + 1
        };
        if self.chain_table.len() != chain_size {
            self.chain_table.resize(chain_size, 0);
        }

        let row_log = row_log(params);
        if self.row_log != row_log {
            self.row_log = row_log;
            self.tag_table.clear();
            self.next_to_update = 0;
            self.row_hash_cache = [0; 8];
        }
        if row_match_enabled && self.tag_table.len() != hash_size {
            self.tag_table.resize(hash_size, 0);
        }
    }

    pub(super) fn correct_after_long_match_gap(&mut self, block_start: usize) {
        if block_start > self.next_to_update + LONG_MATCH_UPDATE_GAP {
            let gap = block_start - self.next_to_update - LONG_MATCH_UPDATE_GAP;
            self.next_to_update = block_start - gap.min(LONG_MATCH_UPDATE_LIMIT);
        }
    }

    pub(super) fn reset_hash3_cursor_to_primary(&mut self) {
        self.next_to_update3 = self.next_to_update;
    }

    fn advance_hash_salt(&mut self) {
        self.hash_salt = bitmix(self.hash_salt, 8) ^ bitmix(u64::from(self.hash_salt_entropy), 4);
    }
}

fn bitmix(mut value: u64, len: u64) -> u64 {
    const PRIME: u64 = 0x9FB2_1C65_1E98_DF25;
    value ^= value.rotate_right(49) ^ value.rotate_right(24);
    value = value.wrapping_mul(PRIME);
    value ^= (value >> 35).wrapping_add(len);
    value = value.wrapping_mul(PRIME);
    value ^ (value >> 28)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::levels::c_port::params::Strategy;

    fn row_params() -> CompressionParameters {
        CompressionParameters {
            window_log: 18,
            chain_log: 16,
            hash_log: 16,
            search_log: 5,
            min_match: 4,
            target_length: 0,
            strategy: Strategy::Greedy,
        }
    }

    #[test]
    fn reset_for_frame_advances_row_hash_salt() {
        let mut state = GreedyMatchState::new();
        let params = row_params();

        state.reset_for_frame(params);
        let first_salt = state.hash_salt;
        state.hash_salt_entropy = 12345;
        state.reset_for_frame(params);

        assert_ne!(first_salt, 0);
        assert_ne!(state.hash_salt, first_salt);
    }

    #[test]
    fn reset_for_frame_clears_indexes_but_keeps_allocations() {
        let params = row_params();
        let mut state = GreedyMatchState::new();
        state.ensure_tables(params);
        state.hash_table[3] = 99;
        state.hash_table3.resize(8, 7);
        state.next_to_update = 42;
        state.next_to_update3 = 24;
        let hash_capacity = state.hash_table.capacity();

        state.reset_for_frame(params);

        assert_eq!(state.next_to_update, 0);
        assert_eq!(state.next_to_update3, 0);
        assert!(state.hash_table.iter().all(|&index| index == 0));
        assert!(state.hash_table3.iter().all(|&index| index == 0));
        assert!(state.chain_table.iter().all(|&index| index == 0));
        assert_eq!(state.hash_table.capacity(), hash_capacity);
    }

    #[test]
    fn long_match_gap_correction_leaves_nearby_cursor_unchanged() {
        let mut state = GreedyMatchState::new();
        state.next_to_update = 100;

        state.correct_after_long_match_gap(484);

        assert_eq!(state.next_to_update, 100);
    }

    #[test]
    fn long_match_gap_correction_updates_short_gap_like_c() {
        let mut state = GreedyMatchState::new();
        state.next_to_update = 100;

        state.correct_after_long_match_gap(500);

        assert_eq!(state.next_to_update, 484);
    }

    #[test]
    fn long_match_gap_correction_caps_update_distance_like_c() {
        let mut state = GreedyMatchState::new();
        state.next_to_update = 100;

        state.correct_after_long_match_gap(800);

        assert_eq!(state.next_to_update, 608);
    }

    #[test]
    fn hash3_cursor_can_be_reset_to_primary_for_opt_parser() {
        let mut state = GreedyMatchState::new();
        state.next_to_update = 608;
        state.next_to_update3 = 1200;

        state.reset_hash3_cursor_to_primary();

        assert_eq!(state.next_to_update3, 608);
    }

    #[test]
    fn row_match_finder_does_not_allocate_chain_table() {
        let mut state = GreedyMatchState::new();

        state.ensure_tables(row_params());

        assert!(state.chain_table.is_empty());
        assert!(!state.hash_table.is_empty());
        assert!(!state.tag_table.is_empty());
    }

    #[test]
    fn hash_chain_match_finder_allocates_chain_table() {
        let mut params = row_params();
        params.window_log = 14;
        let mut state = GreedyMatchState::new();

        state.ensure_tables(params);

        assert_eq!(state.chain_table.len(), (1_usize << params.chain_log) + 1);
        assert!(state.tag_table.is_empty());
    }
}
