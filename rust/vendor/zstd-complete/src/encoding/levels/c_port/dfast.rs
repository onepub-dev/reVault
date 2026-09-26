//! No-dictionary double-fast block compressor ported from `zstd_double_fast.c`.

#[cfg(test)]
use alloc::vec::Vec;
use core::ops::Range;

use super::dfast_helpers::{
    count_match, hash8_ptr, hash_small_ptr, lowest_prefix_index_with_loaded_dict, read32, read64,
    store_match, HASH_READ_SIZE,
};
use super::dfast_table::{entry, set_entry};
use super::params::CompressionParameters;
use super::sequence_store::{
    OffBase, PreparedStoreWords, RepeatCode, RepeatOffsets, StoredSequence,
};
use crate::workspace::{Arena, ArenaError, ArenaSize, ReusableVec};

const SEARCH_STRENGTH: usize = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DFastBlockOutput {
    pub(crate) sequences: ReusableVec<StoredSequence>,
    pub(crate) last_literals: u32,
    pub(crate) repeat_offsets: RepeatOffsets,
}

#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DFastMatchState {
    pub(super) hash_long: ReusableVec<u32>,
    pub(super) hash_small: ReusableVec<u32>,
    pub(super) hash_log: u32,
    pub(super) chain_log: u32,
    sequence_store: ReusableVec<StoredSequence>,
    prepared_store: PreparedStoreWords,
}

impl DFastMatchState {
    pub(crate) fn new() -> Self {
        Self {
            hash_long: ReusableVec::new(),
            hash_small: ReusableVec::new(),
            hash_log: 0,
            chain_log: 0,
            sequence_store: ReusableVec::new(),
            prepared_store: PreparedStoreWords::default(),
        }
    }

    pub(crate) fn add_workspace_size(
        size: &mut ArenaSize,
        params: CompressionParameters,
        block_size: usize,
    ) -> Result<(), ArenaError> {
        let sequence_capacity = block_size / 3 + 1;
        size.add::<u32>(1_usize << params.hash_log)?;
        size.add::<u32>(1_usize << params.chain_log)?;
        size.add::<StoredSequence>(sequence_capacity)?;
        PreparedStoreWords::add_workspace_size(
            size,
            block_size.saturating_add(64),
            sequence_capacity,
        )
    }

    pub(crate) fn new_in(
        arena: &mut Arena<'_>,
        params: CompressionParameters,
        block_size: usize,
    ) -> Result<Self, ArenaError> {
        let sequence_capacity = block_size / 3 + 1;
        let mut hash_long = arena.allocate_reusable_vec(1_usize << params.hash_log)?;
        hash_long.resize(1_usize << params.hash_log, 0);
        let mut hash_small = arena.allocate_reusable_vec(1_usize << params.chain_log)?;
        hash_small.resize(1_usize << params.chain_log, 0);
        Ok(Self {
            hash_long,
            hash_small,
            hash_log: params.hash_log,
            chain_log: params.chain_log,
            sequence_store: arena.allocate_reusable_vec(sequence_capacity)?,
            prepared_store: PreparedStoreWords::new_in(
                arena,
                block_size.saturating_add(64),
                sequence_capacity,
            )?,
        })
    }

    pub(super) fn take_prepared_store(&mut self) -> PreparedStoreWords {
        core::mem::take(&mut self.prepared_store)
    }

    pub(super) fn recycle_prepared_store(&mut self, mut prepared: PreparedStoreWords) {
        prepared.clear();
        self.prepared_store = prepared;
    }

    #[cfg(test)]
    pub(super) fn prepared_store_allocation(
        &self,
    ) -> ((*const u8, usize), (*const [u32; 4], usize)) {
        self.prepared_store.allocation()
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

    pub(super) fn ensure_tables(&mut self, params: CompressionParameters) {
        if self.hash_log != params.hash_log {
            self.hash_log = params.hash_log;
            self.hash_long.clear();
        }
        if self.chain_log != params.chain_log {
            self.chain_log = params.chain_log;
            self.hash_small.clear();
        }

        let long_size = 1_usize << params.hash_log;
        if self.hash_long.len() != long_size {
            self.hash_long.resize(long_size, 0);
        }

        let small_size = 1_usize << params.chain_log;
        if self.hash_small.len() != small_size {
            self.hash_small.resize(small_size, 0);
        }
    }

    pub(crate) fn reset_for_frame(&mut self, params: CompressionParameters) {
        self.ensure_tables(params);
        self.hash_long.fill(0);
        self.hash_small.fill(0);
        self.prepared_store.clear();
        self.sequence_store.clear();
    }
}

pub(crate) fn compress_block_double_fast_no_dict(
    src: &[u8],
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
) -> DFastBlockOutput {
    let mut state = DFastMatchState::new();
    compress_block_double_fast_no_dict_with_state(
        src,
        0..src.len(),
        params,
        repeat_offsets,
        &mut state,
    )
}

pub(crate) fn compress_block_double_fast_no_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut DFastMatchState,
) -> DFastBlockOutput {
    compress_block_double_fast_no_dict_with_state_and_loaded_dict(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        0,
    )
}

pub(crate) fn compress_block_double_fast_no_dict_with_state_and_loaded_dict(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut DFastMatchState,
    loaded_dict_end: usize,
) -> DFastBlockOutput {
    compress_block_double_fast_no_dict_codegen(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        loaded_dict_end,
    )
}

fn compress_block_double_fast_no_dict_codegen(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut DFastMatchState,
    loaded_dict_end: usize,
) -> DFastBlockOutput {
    debug_assert!(block_range.start <= block_range.end);
    debug_assert!(block_range.end <= src.len());

    let mut sequences = state.take_sequence_store();
    let block_len = block_range.end - block_range.start;
    if block_len <= HASH_READ_SIZE {
        return DFastBlockOutput {
            sequences,
            last_literals: block_len as u32,
            repeat_offsets,
        };
    }

    state.ensure_tables(params);
    let maximum_sequences = block_len / 4 + 1;
    sequences.reserve(maximum_sequences);
    let spare = sequences.spare_capacity_mut();
    debug_assert!(spare.len() >= maximum_sequences);
    // SAFETY: `StoredSequence` has a compile-time-proven `[u32; 3]` layout;
    // the leaf transaction receives only spare capacity and initializes the
    // returned bounded prefix.
    let sequence_words = unsafe {
        core::slice::from_raw_parts_mut(
            spare
                .as_mut_ptr()
                .cast::<core::mem::MaybeUninit<[u32; 3]>>(),
            spare.len(),
        )
    };
    // SAFETY: the checked block and context parameters own the exact two hash
    // tables selected by the logs, and `sequence_words` spans the reserved
    // maximum number of sequences for this block.
    let result = unsafe {
        crate::kernel::dfast::select_block(params.min_match)(
            src,
            block_range.start,
            block_range.end,
            loaded_dict_end,
            params.window_log,
            params.hash_log,
            params.chain_log,
            repeat_offsets.as_offsets(),
            &mut state.hash_long,
            &mut state.hash_small,
            sequence_words,
        )
    };
    debug_assert!(result.sequence_count <= maximum_sequences);
    // SAFETY: the returned count covers exactly the initialized spare prefix.
    unsafe { sequences.set_len(result.sequence_count) };

    DFastBlockOutput {
        sequences,
        last_literals: result.last_literals,
        repeat_offsets: RepeatOffsets::from_offsets(
            result.repeat_offsets[0],
            result.repeat_offsets[1],
            result.repeat_offsets[2],
        ),
    }
}

#[cfg(test)]
fn compress_block_double_fast_no_dict_local(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut DFastMatchState,
    loaded_dict_end: usize,
) -> DFastBlockOutput {
    match params.min_match {
        4 => compress_block_double_fast_no_dict_with_state_mls::<4>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        5 => compress_block_double_fast_no_dict_with_state_mls::<5>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        6 => compress_block_double_fast_no_dict_with_state_mls::<6>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        7 => compress_block_double_fast_no_dict_with_state_mls::<7>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        _ => compress_block_double_fast_no_dict_with_state_mls::<4>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
    }
}

fn compress_block_double_fast_no_dict_with_state_mls<const MIN_MATCH: u32>(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut DFastMatchState,
    loaded_dict_end: usize,
) -> DFastBlockOutput {
    debug_assert!(block_range.start <= block_range.end);
    debug_assert!(block_range.end <= src.len());

    let mut rep = repeat_offsets.as_offsets();
    let mut sequences = state.take_sequence_store();
    let block_start = block_range.start;
    let block_end = block_range.end;
    let block_len = block_end - block_start;

    if block_len <= HASH_READ_SIZE {
        return DFastBlockOutput {
            sequences,
            last_literals: block_len as u32,
            repeat_offsets,
        };
    }

    state.ensure_tables(params);
    let hash_long = &mut state.hash_long;
    let hash_small = &mut state.hash_small;
    let h_bits_l = params.hash_log;
    let h_bits_s = params.chain_log;
    let prefix_lowest_index =
        lowest_prefix_index_with_loaded_dict(block_end, params.window_log, loaded_dict_end);
    let ilimit = block_end - HASH_READ_SIZE;

    let mut anchor = block_start;
    let mut ip = if block_start == prefix_lowest_index {
        block_start + 1
    } else {
        block_start
    };

    let mut offset_1 = rep[0] as usize;
    let mut offset_2 = rep[1] as usize;
    let mut offset_saved1 = 0_usize;
    let mut offset_saved2 = 0_usize;

    let current = ip;
    let window_low =
        lowest_prefix_index_with_loaded_dict(current, params.window_log, loaded_dict_end);
    let max_rep = current - window_low;
    if offset_2 > max_rep {
        offset_saved2 = offset_2;
        offset_2 = 0;
    }
    if offset_1 > max_rep {
        offset_saved1 = offset_1;
        offset_1 = 0;
    }

    'outer: loop {
        let mut step = 1_usize;
        let mut next_step = ip + (1 << SEARCH_STRENGTH);
        let mut ip1 = ip + step;

        if ip1 > ilimit {
            break;
        }

        let mut hl0 = hash8_ptr(src, ip, h_bits_l);
        let mut idxl0 = entry(hash_long, hl0) as usize;
        let mut matchl0 = idxl0;

        loop {
            let hs0 = hash_small_ptr::<MIN_MATCH>(src, ip, h_bits_s);
            let idxs0 = entry(hash_small, hs0) as usize;
            let curr = ip;
            let mut matchs0 = idxs0;

            set_entry(hash_long, hl0, curr as u32);
            set_entry(hash_small, hs0, curr as u32);

            if offset_1 > 0 && {
                debug_assert!(ip + 1 >= offset_1);
                read32(src, ip + 1 - offset_1) == read32(src, ip + 1)
            } {
                let match_length =
                    count_match(src, ip + 1 + 4, ip + 1 + 4 - offset_1, block_end) + 4;
                ip += 1;
                store_match(
                    &mut sequences,
                    &mut anchor,
                    &mut ip,
                    OffBase::Repeat(RepeatCode::First),
                    match_length,
                );
                complementary_insert::<MIN_MATCH>(
                    src, hash_long, hash_small, h_bits_l, h_bits_s, curr, ip, ilimit,
                );
                consume_immediate_repcodes::<MIN_MATCH>(
                    src,
                    hash_long,
                    hash_small,
                    h_bits_l,
                    h_bits_s,
                    &mut sequences,
                    &mut anchor,
                    &mut ip,
                    ilimit,
                    &mut offset_1,
                    &mut offset_2,
                    block_end,
                );
                continue 'outer;
            }

            let hl1 = hash8_ptr(src, ip1, h_bits_l);

            if idxl0 > prefix_lowest_index && read64(src, matchl0) == read64(src, ip) {
                let mut match_length = count_match(src, ip + 8, matchl0 + 8, block_end) + 8;
                let mut offset = ip - matchl0;
                while ip > anchor
                    && matchl0 > prefix_lowest_index
                    && src[ip - 1] == src[matchl0 - 1]
                {
                    ip -= 1;
                    matchl0 -= 1;
                    offset = ip - matchl0;
                    match_length += 1;
                }
                if step < 4 {
                    set_entry(hash_long, hl1, ip1 as u32);
                }
                store_offset_match(
                    &mut sequences,
                    &mut anchor,
                    &mut ip,
                    &mut offset_1,
                    &mut offset_2,
                    offset,
                    match_length,
                );
                complementary_insert::<MIN_MATCH>(
                    src, hash_long, hash_small, h_bits_l, h_bits_s, curr, ip, ilimit,
                );
                consume_immediate_repcodes::<MIN_MATCH>(
                    src,
                    hash_long,
                    hash_small,
                    h_bits_l,
                    h_bits_s,
                    &mut sequences,
                    &mut anchor,
                    &mut ip,
                    ilimit,
                    &mut offset_1,
                    &mut offset_2,
                    block_end,
                );
                continue 'outer;
            }

            let idxl1 = entry(hash_long, hl1) as usize;
            let matchl1 = idxl1;

            if idxs0 > prefix_lowest_index && read32(src, matchs0) == read32(src, ip) {
                let mut match_length = count_match(src, ip + 4, matchs0 + 4, block_end) + 4;
                let mut offset = ip - matchs0;

                if idxl1 > prefix_lowest_index && read64(src, matchl1) == read64(src, ip1) {
                    let l1_len = count_match(src, ip1 + 8, matchl1 + 8, block_end) + 8;
                    if l1_len > match_length {
                        ip = ip1;
                        match_length = l1_len;
                        offset = ip - matchl1;
                        matchs0 = matchl1;
                    }
                }

                while ip > anchor
                    && matchs0 > prefix_lowest_index
                    && src[ip - 1] == src[matchs0 - 1]
                {
                    ip -= 1;
                    matchs0 -= 1;
                    offset = ip - matchs0;
                    match_length += 1;
                }

                if step < 4 {
                    set_entry(hash_long, hl1, ip1 as u32);
                }
                store_offset_match(
                    &mut sequences,
                    &mut anchor,
                    &mut ip,
                    &mut offset_1,
                    &mut offset_2,
                    offset,
                    match_length,
                );
                complementary_insert::<MIN_MATCH>(
                    src, hash_long, hash_small, h_bits_l, h_bits_s, curr, ip, ilimit,
                );
                consume_immediate_repcodes::<MIN_MATCH>(
                    src,
                    hash_long,
                    hash_small,
                    h_bits_l,
                    h_bits_s,
                    &mut sequences,
                    &mut anchor,
                    &mut ip,
                    ilimit,
                    &mut offset_1,
                    &mut offset_2,
                    block_end,
                );
                continue 'outer;
            }

            if ip1 >= next_step {
                step += 1;
                next_step += 1 << SEARCH_STRENGTH;
            }
            ip = ip1;
            ip1 += step;

            hl0 = hl1;
            idxl0 = idxl1;
            matchl0 = matchl1;

            if ip1 > ilimit {
                break 'outer;
            }
        }
    }

    if offset_saved1 != 0 && offset_1 != 0 {
        offset_saved2 = offset_saved1;
    }
    rep[0] = (if offset_1 != 0 {
        offset_1
    } else {
        offset_saved1
    }) as u32;
    rep[1] = (if offset_2 != 0 {
        offset_2
    } else {
        offset_saved2
    }) as u32;

    DFastBlockOutput {
        sequences,
        last_literals: (block_end - anchor) as u32,
        repeat_offsets: RepeatOffsets::from_offsets(rep[0], rep[1], rep[2]),
    }
}

#[allow(clippy::too_many_arguments)]
fn complementary_insert<const MIN_MATCH: u32>(
    src: &[u8],
    hash_long: &mut [u32],
    hash_small: &mut [u32],
    h_bits_l: u32,
    h_bits_s: u32,
    curr: usize,
    ip: usize,
    ilimit: usize,
) {
    if ip > ilimit {
        return;
    }

    let index_to_insert = curr + 2;
    if index_to_insert <= ilimit {
        set_entry(
            hash_long,
            hash8_ptr(src, index_to_insert, h_bits_l),
            index_to_insert as u32,
        );
        set_entry(
            hash_small,
            hash_small_ptr::<MIN_MATCH>(src, index_to_insert, h_bits_s),
            index_to_insert as u32,
        );
    }
    if let Some(index) = ip.checked_sub(2).filter(|index| *index <= ilimit) {
        set_entry(hash_long, hash8_ptr(src, index, h_bits_l), index as u32);
    }
    if let Some(index) = ip.checked_sub(1).filter(|index| *index <= ilimit) {
        set_entry(
            hash_small,
            hash_small_ptr::<MIN_MATCH>(src, index, h_bits_s),
            index as u32,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn consume_immediate_repcodes<const MIN_MATCH: u32>(
    src: &[u8],
    hash_long: &mut [u32],
    hash_small: &mut [u32],
    h_bits_l: u32,
    h_bits_s: u32,
    sequences: &mut ReusableVec<StoredSequence>,
    anchor: &mut usize,
    ip: &mut usize,
    ilimit: usize,
    offset_1: &mut usize,
    offset_2: &mut usize,
    block_end: usize,
) {
    while *ip <= ilimit && *offset_2 > 0 && {
        debug_assert!(*ip >= *offset_2);
        read32(src, *ip) == read32(src, *ip - *offset_2)
    } {
        let repeat_length = count_match(src, *ip + 4, *ip + 4 - *offset_2, block_end) + 4;
        core::mem::swap(offset_1, offset_2);
        set_entry(
            hash_small,
            hash_small_ptr::<MIN_MATCH>(src, *ip, h_bits_s),
            *ip as u32,
        );
        set_entry(hash_long, hash8_ptr(src, *ip, h_bits_l), *ip as u32);
        store_match(
            sequences,
            anchor,
            ip,
            OffBase::Repeat(RepeatCode::First),
            repeat_length,
        );
    }
}

fn store_offset_match(
    sequences: &mut ReusableVec<StoredSequence>,
    anchor: &mut usize,
    ip: &mut usize,
    offset_1: &mut usize,
    offset_2: &mut usize,
    offset: usize,
    match_length: usize,
) {
    *offset_2 = *offset_1;
    *offset_1 = offset;
    store_match(
        sequences,
        anchor,
        ip,
        OffBase::Offset(offset as u32),
        match_length,
    );
}

#[cfg(test)]
mod transaction_tests {
    use super::*;
    use crate::encoding::levels::c_port::params::Strategy;

    #[test]
    fn generated_transaction_matches_local_for_every_min_match() {
        let mut source = Vec::new();
        for index in 0..4096u32 {
            source.extend_from_slice(&(index % 53).to_le_bytes());
            source.extend_from_slice(b"double-fast-transaction-reference");
            if index.is_multiple_of(7) {
                source.extend_from_slice(b"double-fast-transaction-reference");
            }
        }

        for min_match in 4..=7 {
            let params = CompressionParameters {
                window_log: 18,
                chain_log: 16,
                hash_log: 17,
                search_log: 1,
                min_match,
                target_length: 1,
                strategy: Strategy::DFast,
            };
            let block_range = 0..source.len();
            let mut local_state = DFastMatchState::new();
            let mut codegen_state = local_state.clone();
            let local = compress_block_double_fast_no_dict_local(
                &source,
                block_range.clone(),
                params,
                RepeatOffsets::new(),
                &mut local_state,
                0,
            );
            let codegen = compress_block_double_fast_no_dict_codegen(
                &source,
                block_range,
                params,
                RepeatOffsets::new(),
                &mut codegen_state,
                0,
            );

            assert_eq!(codegen, local, "min_match={min_match}");
            assert_eq!(codegen_state, local_state, "min_match={min_match}");
        }
    }
}
