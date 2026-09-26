//! Optimal-parser state shared by the no-dictionary C optimal strategies.

use alloc::vec::Vec;

use super::{
    greedy::GreedyMatchState,
    opt_match::OptMatchTable,
    opt_price::{OptPriceState, ZSTD_MAX_PRICE},
    params::CompressionParameters,
    post_split::PostSplitScratch,
    sequence_store::{
        lease_prepared_words, recover_prepared_words, PreparedBlockLease, PreparedStoreWords,
        RepeatOffsets, StoredSequence,
    },
};
use crate::{
    encoding::blocks::PreparedBlock,
    fse::fse_encoder::FSETableBuildScratch,
    huff0::huff0_encoder::HuffmanBuildScratch,
    workspace::{Arena, ArenaError, ArenaSize, ReusableVec, VecLease},
};

pub(super) const HASH_READ_SIZE: usize = 8;
pub(super) const ZSTD_OPT_NUM: usize = 1 << 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Optimal {
    pub(super) price: i32,
    pub(super) off: u32,
    pub(super) mlen: u32,
    pub(super) litlen: u32,
    pub(super) rep: [u32; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ForwardResult {
    pub(super) last_pos: usize,
    pub(super) last_stretch: Option<Optimal>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OptParserStrategy {
    BtOpt,
    BtUltra,
}

pub(crate) struct OptBlockState {
    pub(super) match_state: GreedyMatchState,
    pub(super) attached_match_state: Option<GreedyMatchState>,
    pub(super) price_state: OptPriceState,
    pub(super) literal_price_cache: LiteralPriceCache,
    pub(super) matches: OptMatchTable,
    pub(super) opt: ReusableVec<Optimal>,
    pub(super) path: ReusableVec<Optimal>,
    pub(super) post_split_scratch: PostSplitScratch,
    sequences: ReusableVec<StoredSequence>,
    prepared_store: PreparedStoreWords,
    prepared_lease: Option<PreparedBlockLease>,
    block_bytes: ReusableVec<u8>,
    block_bytes_lease: Option<VecLease<u8>>,
    pub(super) entropy_huffman_scratch: HuffmanBuildScratch,
    pub(super) entropy_fse_scratch: FSETableBuildScratch,
}

impl Default for Optimal {
    fn default() -> Self {
        Self {
            price: ZSTD_MAX_PRICE,
            off: 0,
            mlen: 0,
            litlen: 0,
            rep: RepeatOffsets::new().as_offsets(),
        }
    }
}

impl OptBlockState {
    pub(crate) fn is_workspace_backed(&self) -> bool {
        !self.opt.is_owned()
    }

    pub(crate) fn new() -> Self {
        let mut opt = ReusableVec::with_capacity(ZSTD_OPT_NUM + 4);
        opt.resize(ZSTD_OPT_NUM + 4, Optimal::default());
        Self {
            match_state: GreedyMatchState::new(),
            attached_match_state: None,
            price_state: OptPriceState::new(),
            literal_price_cache: LiteralPriceCache::new(),
            matches: OptMatchTable::new(),
            opt,
            path: ReusableVec::with_capacity(ZSTD_OPT_NUM),
            post_split_scratch: PostSplitScratch::new(),
            sequences: ReusableVec::new(),
            prepared_store: PreparedStoreWords::default(),
            prepared_lease: None,
            block_bytes: ReusableVec::new(),
            block_bytes_lease: None,
            entropy_huffman_scratch: HuffmanBuildScratch::new(),
            entropy_fse_scratch: FSETableBuildScratch::new(),
        }
    }

    pub(crate) fn add_workspace_size(
        size: &mut ArenaSize,
        params: CompressionParameters,
        block_size: usize,
    ) -> Result<(), ArenaError> {
        GreedyMatchState::add_workspace_size(size, params, block_size)?;
        OptMatchTable::add_workspace_size(size)?;
        size.add::<Optimal>(ZSTD_OPT_NUM + 4)?;
        size.add::<Optimal>(ZSTD_OPT_NUM)?;
        PostSplitScratch::add_workspace_size(size, block_size)?;
        size.add::<StoredSequence>(block_size / 3 + 1)?;
        PreparedStoreWords::add_workspace_size(
            size,
            block_size.saturating_add(64),
            block_size / 3 + 1,
        )?;
        size.add::<u8>(super::compress_bound::compress_bound(block_size))?;
        HuffmanBuildScratch::add_workspace_size(size)?;
        FSETableBuildScratch::add_workspace_size(size)
    }

    pub(crate) fn new_in(
        arena: &mut Arena<'_>,
        params: CompressionParameters,
        block_size: usize,
    ) -> Result<Self, ArenaError> {
        let mut opt = arena.allocate_reusable_vec(ZSTD_OPT_NUM + 4)?;
        opt.resize(ZSTD_OPT_NUM + 4, Optimal::default());
        Ok(Self {
            match_state: GreedyMatchState::new_in(arena, params, block_size)?,
            attached_match_state: None,
            price_state: OptPriceState::new(),
            literal_price_cache: LiteralPriceCache::new(),
            matches: OptMatchTable::new_in(arena)?,
            opt,
            path: arena.allocate_reusable_vec(ZSTD_OPT_NUM)?,
            post_split_scratch: PostSplitScratch::new_in(arena, block_size)?,
            sequences: arena.allocate_reusable_vec(block_size / 3 + 1)?,
            prepared_store: PreparedStoreWords::new_in(
                arena,
                block_size.saturating_add(64),
                block_size / 3 + 1,
            )?,
            prepared_lease: None,
            block_bytes: arena
                .allocate_reusable_vec(super::compress_bound::compress_bound(block_size))?,
            block_bytes_lease: None,
            entropy_huffman_scratch: HuffmanBuildScratch::new_in(arena)?,
            entropy_fse_scratch: FSETableBuildScratch::new_in(arena)?,
        })
    }

    pub(crate) fn reset_for_frame(&mut self, params: CompressionParameters) {
        self.match_state.reset_for_frame(params);
        self.attached_match_state = None;
        self.price_state.reset_for_frame();
    }

    pub(crate) fn take_sequences(&mut self, min_capacity: usize) -> ReusableVec<StoredSequence> {
        let mut sequences = core::mem::take(&mut self.sequences);
        sequences.clear();
        let capacity = sequences.capacity();
        if capacity < min_capacity {
            sequences.reserve(min_capacity - capacity);
        }
        sequences
    }

    pub(crate) fn recycle_sequences(&mut self, mut sequences: ReusableVec<StoredSequence>) {
        sequences.clear();
        if self.sequences.capacity() < sequences.capacity() {
            self.sequences = sequences;
        }
    }

    pub(crate) fn take_prepared_block(&mut self) -> PreparedBlock {
        debug_assert!(self.prepared_lease.is_none());
        let store = core::mem::take(&mut self.prepared_store);
        let (prepared, lease) = lease_prepared_words(store);
        self.prepared_lease = Some(lease);
        prepared
    }

    pub(crate) fn recycle_prepared_block(&mut self, mut prepared: PreparedBlock) {
        prepared.literals.clear();
        prepared.sequences.clear();
        let lease = self
            .prepared_lease
            .take()
            .expect("prepared store lease must be returned");
        self.prepared_store = recover_prepared_words(prepared, lease);
    }

    pub(crate) fn take_literal_store(&mut self) -> PreparedStoreWords {
        debug_assert!(self.prepared_lease.is_none());
        core::mem::take(&mut self.prepared_store)
    }

    pub(crate) fn recycle_literal_store(&mut self, mut prepared: PreparedStoreWords) {
        prepared.clear();
        self.prepared_store = prepared;
    }

    pub(crate) fn take_block_bytes(&mut self, min_capacity: usize) -> Vec<u8> {
        let mut bytes = core::mem::take(&mut self.block_bytes);
        bytes.clear();
        let capacity = bytes.capacity();
        if capacity < min_capacity {
            bytes.reserve(min_capacity - capacity);
        }
        let (bytes, lease) = bytes.lease_vec();
        debug_assert!(self.block_bytes_lease.is_none());
        self.block_bytes_lease = Some(lease);
        bytes
    }

    pub(crate) fn recycle_block_bytes(&mut self, bytes: Vec<u8>) {
        let lease = self
            .block_bytes_lease
            .take()
            .expect("block byte lease must be returned");
        let mut bytes = ReusableVec::recover_vec(bytes, lease);
        bytes.clear();
        if self.block_bytes.capacity() < bytes.capacity() {
            self.block_bytes = bytes;
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct LiteralPriceCache {
    prices: [u32; 256],
    generations: [u16; 256],
    generation: u16,
}

impl LiteralPriceCache {
    fn new() -> Self {
        Self {
            prices: [0; 256],
            generations: [0; 256],
            generation: 1,
        }
    }

    pub(super) fn begin_pass(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.generations = [0; 256];
            self.generation = 1;
        }
    }

    pub(super) fn lookup(&self, literal: u8) -> Option<u32> {
        let idx = usize::from(literal);
        (self.generations[idx] == self.generation).then_some(self.prices[idx])
    }

    pub(super) fn insert(&mut self, literal: u8, price: u32) {
        let idx = usize::from(literal);
        self.prices[idx] = price;
        self.generations[idx] = self.generation;
    }
}
