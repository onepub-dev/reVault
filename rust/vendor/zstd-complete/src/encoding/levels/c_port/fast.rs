//! No-dictionary fast block compressor ported from `zstd_fast.c`.

use alloc::vec;
#[cfg(test)]
use alloc::vec::Vec;
use core::ops::Range;

use super::fast_helpers::{hash_ptr, hash_small_ptr, lowest_prefix_index_with_loaded_dict, read32};
use super::match_count::count_match_behind as count_match;
use super::params::CompressionParameters;
use super::sequence_store::{
    OffBase, PreparedStoreWords, RepeatCode, RepeatOffsets, StoredSequence,
};
use crate::workspace::{Arena, ArenaError, ArenaSize, ReusableVec};

const HASH_READ_SIZE: usize = 8;
const SEARCH_STRENGTH: usize = 8;
const INVALID_INDEX: u32 = u32::MAX;
const SHORT_CACHE_TAG_BITS: u32 = 8;
const SHORT_CACHE_TAG_MASK: usize = (1 << SHORT_CACHE_TAG_BITS) - 1;
const FAST_HASH_FILL_STEP: usize = 3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FastBlockOutput {
    pub(crate) sequences: ReusableVec<StoredSequence>,
    pub(crate) last_literals: u32,
    pub(crate) repeat_offsets: RepeatOffsets,
}

#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FastMatchState {
    hash_table: ReusableVec<u32>,
    hash_log: u32,
    prepared_store: PreparedStoreWords,
    sequence_store: ReusableVec<StoredSequence>,
}

impl FastMatchState {
    pub(crate) fn new() -> Self {
        Self {
            hash_table: ReusableVec::new(),
            hash_log: 0,
            prepared_store: PreparedStoreWords::default(),
            sequence_store: ReusableVec::new(),
        }
    }

    pub(crate) fn add_workspace_size(
        size: &mut ArenaSize,
        params: CompressionParameters,
        block_size: usize,
    ) -> Result<(), ArenaError> {
        let sequence_capacity = block_size / 3 + 1;
        size.add::<u32>(1_usize << params.hash_log)?;
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
        let mut hash_table = arena.allocate_reusable_vec(1_usize << params.hash_log)?;
        hash_table.resize(1_usize << params.hash_log, INVALID_INDEX);
        Ok(Self {
            hash_table,
            hash_log: params.hash_log,
            prepared_store: PreparedStoreWords::new_in(
                arena,
                block_size.saturating_add(64),
                sequence_capacity,
            )?,
            sequence_store: arena.allocate_reusable_vec(sequence_capacity)?,
        })
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

    pub(super) fn table_for(&mut self, hash_log: u32) -> &mut [u32] {
        if self.hash_log != hash_log {
            self.hash_log = hash_log;
            self.hash_table.clear();
        }

        let table_size = 1_usize << hash_log;
        if self.hash_table.len() != table_size {
            self.hash_table.resize(table_size, INVALID_INDEX);
        }

        &mut self.hash_table
    }

    pub(crate) fn reset_for_frame(&mut self, params: CompressionParameters) {
        self.table_for(params.hash_log).fill(INVALID_INDEX);
        self.prepared_store.clear();
        self.sequence_store.clear();
    }

    pub(crate) fn load_prefix(
        &mut self,
        src: &[u8],
        prefix_len: usize,
        params: CompressionParameters,
    ) {
        debug_assert!(prefix_len <= src.len());
        if prefix_len <= HASH_READ_SIZE {
            return;
        }

        let hlog = params.hash_log;
        let min_match = params.min_match;
        let hash_table = self.table_for(hlog);
        let iend = prefix_len - HASH_READ_SIZE;
        let mut ip = 0_usize;

        while ip + 3 < iend + 2 {
            let hash = hash_ptr(src, ip, hlog, min_match);
            hash_table[hash] = ip as u32;
            ip += 3;
        }
    }

    pub(crate) fn load_cdict_copy_prefix(
        &mut self,
        src: &[u8],
        prefix_len: usize,
        params: CompressionParameters,
    ) {
        debug_assert!(prefix_len <= src.len());
        if prefix_len <= HASH_READ_SIZE {
            return;
        }

        let hash_table = self.table_for(params.hash_log);
        hash_table.fill(0);

        let mut tagged_table = vec![0_u32; hash_table.len()];
        let iend = prefix_len - HASH_READ_SIZE;
        let mut ip = 0_usize;

        while ip + FAST_HASH_FILL_STEP - 1 <= iend {
            for step in 0..FAST_HASH_FILL_STEP {
                let pos = ip + step;
                let hash_and_tag = hash_ptr(
                    src,
                    pos,
                    params.hash_log + SHORT_CACHE_TAG_BITS,
                    params.min_match,
                );
                let slot = table_index(hash_and_tag);
                if step == 0 || tagged_table[slot] == 0 {
                    tagged_table[slot] = tagged_index(hash_and_tag, pos);
                }
            }
            ip += FAST_HASH_FILL_STEP;
        }

        for (dst, tagged) in hash_table.iter_mut().zip(tagged_table) {
            *dst = tagged >> SHORT_CACHE_TAG_BITS;
        }
    }
}

fn table_index(hash_and_tag: usize) -> usize {
    hash_and_tag >> SHORT_CACHE_TAG_BITS
}

fn tagged_index(hash_and_tag: usize, index: usize) -> u32 {
    debug_assert!(index <= (u32::MAX >> SHORT_CACHE_TAG_BITS) as usize);
    let tag = hash_and_tag & SHORT_CACHE_TAG_MASK;
    ((index as u32) << SHORT_CACHE_TAG_BITS) | tag as u32
}

pub(crate) fn compress_block_fast_no_dict(
    src: &[u8],
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
) -> FastBlockOutput {
    let mut state = FastMatchState::new();
    compress_block_fast_no_dict_with_state(src, 0..src.len(), params, repeat_offsets, &mut state)
}

pub(crate) fn compress_block_fast_no_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut FastMatchState,
) -> FastBlockOutput {
    compress_block_fast_no_dict_with_state_and_loaded_dict(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        0,
    )
}

pub(crate) fn compress_block_fast_no_dict_with_state_and_loaded_dict(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut FastMatchState,
    loaded_dict_end: usize,
) -> FastBlockOutput {
    compress_block_fast_no_dict_codegen(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        loaded_dict_end,
    )
}

fn compress_block_fast_no_dict_codegen(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut FastMatchState,
    loaded_dict_end: usize,
) -> FastBlockOutput {
    debug_assert!(block_range.start <= block_range.end);
    debug_assert!(block_range.end <= src.len());
    let block_len = block_range.end - block_range.start;
    if block_len <= HASH_READ_SIZE {
        return FastBlockOutput {
            sequences: state.take_sequence_store(),
            last_literals: block_len as u32,
            repeat_offsets,
        };
    }

    let maximum_sequences = block_len / 4 + 1;
    let mut sequences = state.take_sequence_store();
    sequences.reserve(maximum_sequences);
    let spare = sequences.spare_capacity_mut();
    // SAFETY: `StoredSequence` has a compile-time-proven `[u32; 3]` layout;
    // the leaf transaction initializes only the returned spare prefix.
    let sequence_words = unsafe {
        core::slice::from_raw_parts_mut(
            spare
                .as_mut_ptr()
                .cast::<core::mem::MaybeUninit<[u32; 3]>>(),
            spare.len(),
        )
    };
    // SAFETY: the block bounds were checked above, the state owns the exact
    // hash table selected by `hash_log`, and `sequence_words` is the complete
    // maximum-size spare sequence prefix for this block.
    let result = unsafe {
        crate::kernel::fast::select_block(params.min_match)(
            src,
            block_range.start,
            block_range.end,
            loaded_dict_end,
            params.window_log,
            params.hash_log,
            params.target_length,
            repeat_offsets.as_offsets(),
            state.table_for(params.hash_log),
            sequence_words,
        )
    };
    debug_assert!(result.sequence_count <= maximum_sequences);
    // SAFETY: the returned count is exactly the initialized spare prefix.
    unsafe { sequences.set_len(result.sequence_count) };

    FastBlockOutput {
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
fn compress_block_fast_no_dict_local(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut FastMatchState,
    loaded_dict_end: usize,
) -> FastBlockOutput {
    match params.min_match {
        4 => compress_block_fast_no_dict_with_state_mls::<4>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        5 => compress_block_fast_no_dict_with_state_mls::<5>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        6 => compress_block_fast_no_dict_with_state_mls::<6>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        7 => compress_block_fast_no_dict_with_state_mls::<7>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
        _ => compress_block_fast_no_dict_with_state_mls::<4>(
            src,
            block_range,
            params,
            repeat_offsets,
            state,
            loaded_dict_end,
        ),
    }
}

fn compress_block_fast_no_dict_with_state_mls<const MIN_MATCH: u32>(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut FastMatchState,
    loaded_dict_end: usize,
) -> FastBlockOutput {
    debug_assert!(block_range.start <= block_range.end);
    debug_assert!(block_range.end <= src.len());

    let mut rep = repeat_offsets.as_offsets();
    let mut sequences = state.take_sequence_store();
    let block_start = block_range.start;
    let block_end = block_range.end;
    let block_len = block_end - block_start;
    if block_len <= HASH_READ_SIZE {
        return FastBlockOutput {
            sequences,
            last_literals: block_len as u32,
            repeat_offsets,
        };
    }

    let hlog = params.hash_log;
    let step_size = params.target_length as usize + usize::from(params.target_length == 0) + 1;
    let prefix_start_index =
        lowest_prefix_index_with_loaded_dict(block_end, params.window_log, loaded_dict_end);
    let ilimit = block_end - HASH_READ_SIZE;

    let hash_table = state.table_for(hlog);
    let mut anchor = block_start;
    let mut ip0 = if block_start == 0 && prefix_start_index == 0 {
        1
    } else {
        block_start
    };

    let mut rep_offset1 = rep[0] as usize;
    let mut rep_offset2 = rep[1] as usize;
    let mut offset_saved1 = 0_usize;
    let mut offset_saved2 = 0_usize;

    let curr = ip0;
    let window_low = lowest_prefix_index_with_loaded_dict(curr, params.window_log, loaded_dict_end);
    let max_rep = curr - window_low;
    if rep_offset2 > max_rep {
        offset_saved2 = rep_offset2;
        rep_offset2 = 0;
    }
    if rep_offset1 > max_rep {
        offset_saved1 = rep_offset1;
        rep_offset1 = 0;
    }

    'restart: loop {
        let mut step = step_size;
        let mut next_step = ip0 + (1 << (SEARCH_STRENGTH - 1));
        let mut ip1 = ip0 + 1;
        let mut ip2 = ip0 + step;
        let mut ip3 = ip2 + 1;

        if ip3 >= ilimit {
            break;
        }

        let mut hash0 = hash_small_ptr::<MIN_MATCH>(src, ip0, hlog);
        let mut hash1 = hash_small_ptr::<MIN_MATCH>(src, ip1, hlog);
        let mut match_idx = hash_table[hash0] as usize;

        while ip3 < ilimit {
            let current0 = ip0;
            hash_table[hash0] = current0 as u32;

            if rep_offset1 > 0 && {
                debug_assert!(ip2 >= rep_offset1);
                read32(src, ip2) == read32(src, ip2 - rep_offset1)
            } {
                ip0 = ip2;
                let mut match0 = ip0 - rep_offset1;
                let backward = usize::from(ip0 > 0 && src[ip0 - 1] == src[match0 - 1]);
                ip0 -= backward;
                match0 -= backward;
                hash_table[hash1] = ip1 as u32;
                store_match(
                    src,
                    &mut sequences,
                    &mut anchor,
                    &mut ip0,
                    match0,
                    OffBase::Repeat(RepeatCode::First),
                    4 + backward,
                    block_end,
                );
                fill_after_match::<MIN_MATCH>(src, hash_table, hlog, current0, ip0, ilimit);
                consume_immediate_repcodes::<MIN_MATCH>(
                    src,
                    hash_table,
                    &mut sequences,
                    hlog,
                    &mut anchor,
                    &mut ip0,
                    ilimit,
                    &mut rep_offset1,
                    &mut rep_offset2,
                    block_end,
                );
                continue 'restart;
            }

            if match4_found(src, ip0, match_idx, prefix_start_index) {
                hash_table[hash1] = ip1 as u32;
                let mut match0 = match_idx;
                rep_offset2 = rep_offset1;
                rep_offset1 = ip0 - match0;
                let mut match_length = 4;
                while ip0 > anchor && match0 > prefix_start_index && src[ip0 - 1] == src[match0 - 1]
                {
                    ip0 -= 1;
                    match0 -= 1;
                    match_length += 1;
                }
                store_match(
                    src,
                    &mut sequences,
                    &mut anchor,
                    &mut ip0,
                    match0,
                    OffBase::Offset(rep_offset1 as u32),
                    match_length,
                    block_end,
                );
                fill_after_match::<MIN_MATCH>(src, hash_table, hlog, current0, ip0, ilimit);
                consume_immediate_repcodes::<MIN_MATCH>(
                    src,
                    hash_table,
                    &mut sequences,
                    hlog,
                    &mut anchor,
                    &mut ip0,
                    ilimit,
                    &mut rep_offset1,
                    &mut rep_offset2,
                    block_end,
                );
                continue 'restart;
            }

            match_idx = hash_table[hash1] as usize;
            hash0 = hash1;
            hash1 = hash_small_ptr::<MIN_MATCH>(src, ip2, hlog);
            ip0 = ip1;
            ip1 = ip2;
            ip2 = ip3;

            let current0 = ip0;
            hash_table[hash0] = current0 as u32;

            if match4_found(src, ip0, match_idx, prefix_start_index) {
                if step <= 4 {
                    hash_table[hash1] = ip1 as u32;
                }
                let mut match0 = match_idx;
                rep_offset2 = rep_offset1;
                rep_offset1 = ip0 - match0;
                let mut match_length = 4;
                while ip0 > anchor && match0 > prefix_start_index && src[ip0 - 1] == src[match0 - 1]
                {
                    ip0 -= 1;
                    match0 -= 1;
                    match_length += 1;
                }
                store_match(
                    src,
                    &mut sequences,
                    &mut anchor,
                    &mut ip0,
                    match0,
                    OffBase::Offset(rep_offset1 as u32),
                    match_length,
                    block_end,
                );
                fill_after_match::<MIN_MATCH>(src, hash_table, hlog, current0, ip0, ilimit);
                consume_immediate_repcodes::<MIN_MATCH>(
                    src,
                    hash_table,
                    &mut sequences,
                    hlog,
                    &mut anchor,
                    &mut ip0,
                    ilimit,
                    &mut rep_offset1,
                    &mut rep_offset2,
                    block_end,
                );
                continue 'restart;
            }

            match_idx = hash_table[hash1] as usize;
            hash0 = hash1;
            hash1 = hash_small_ptr::<MIN_MATCH>(src, ip2, hlog);
            ip0 = ip1;
            ip1 = ip2;
            ip2 = ip0 + step;
            ip3 = ip1 + step;

            if ip2 >= next_step {
                step += 1;
                next_step += 1 << (SEARCH_STRENGTH - 1);
            }
        }

        break;
    }

    if offset_saved1 != 0 && rep_offset1 != 0 {
        offset_saved2 = offset_saved1;
    }
    rep[0] = (if rep_offset1 != 0 {
        rep_offset1
    } else {
        offset_saved1
    }) as u32;
    rep[1] = (if rep_offset2 != 0 {
        rep_offset2
    } else {
        offset_saved2
    }) as u32;

    FastBlockOutput {
        sequences,
        last_literals: (block_end - anchor) as u32,
        repeat_offsets: RepeatOffsets::from_offsets(rep[0], rep[1], rep[2]),
    }
}

#[allow(clippy::too_many_arguments)]
fn store_match(
    src: &[u8],
    sequences: &mut ReusableVec<StoredSequence>,
    anchor: &mut usize,
    ip: &mut usize,
    match_pos: usize,
    off_base: OffBase,
    match_length: usize,
    match_limit: usize,
) {
    let match_length = match_length
        + count_match(
            src,
            *ip + match_length,
            match_pos + match_length,
            match_limit,
        );
    sequences.push(StoredSequence::new(
        (*ip - *anchor) as u32,
        off_base,
        match_length as u32,
    ));
    *ip += match_length;
    *anchor = *ip;
}

fn fill_after_match<const MIN_MATCH: u32>(
    src: &[u8],
    hash_table: &mut [u32],
    hlog: u32,
    current0: usize,
    ip: usize,
    ilimit: usize,
) {
    if ip > ilimit {
        return;
    }
    if current0 + 2 <= ilimit {
        hash_table[hash_small_ptr::<MIN_MATCH>(src, current0 + 2, hlog)] = (current0 + 2) as u32;
    }
    if ip >= 2 && ip - 2 <= ilimit {
        hash_table[hash_small_ptr::<MIN_MATCH>(src, ip - 2, hlog)] = (ip - 2) as u32;
    }
}

#[allow(clippy::too_many_arguments)]
fn consume_immediate_repcodes<const MIN_MATCH: u32>(
    src: &[u8],
    hash_table: &mut [u32],
    sequences: &mut ReusableVec<StoredSequence>,
    hlog: u32,
    anchor: &mut usize,
    ip: &mut usize,
    ilimit: usize,
    rep_offset1: &mut usize,
    rep_offset2: &mut usize,
    match_limit: usize,
) {
    if *rep_offset2 == 0 {
        return;
    }

    while *ip <= ilimit && {
        debug_assert!(*ip >= *rep_offset2);
        read32(src, *ip) == read32(src, *ip - *rep_offset2)
    } {
        let repeat_length = count_match(src, *ip + 4, *ip + 4 - *rep_offset2, match_limit) + 4;
        core::mem::swap(rep_offset1, rep_offset2);
        hash_table[hash_small_ptr::<MIN_MATCH>(src, *ip, hlog)] = *ip as u32;
        *ip += repeat_length;
        sequences.push(StoredSequence::new(
            0,
            OffBase::Repeat(RepeatCode::First),
            repeat_length as u32,
        ));
        *anchor = *ip;
    }
}

fn match4_found(src: &[u8], current: usize, match_idx: usize, prefix_start_index: usize) -> bool {
    if match_idx == INVALID_INDEX as usize || match_idx < prefix_start_index {
        return false;
    }
    debug_assert!(current + 4 <= src.len());
    debug_assert!(match_idx + 4 <= src.len());
    read32(src, current) == read32(src, match_idx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::levels::c_port::params::Strategy;

    #[test]
    fn fast_cdict_copy_loader_uses_full_tagged_hash_like_c() {
        let data = b"abcdefghijklmnopqrstuvwxyabcdefghijklmnopqrstuvwxy";
        let params = CompressionParameters {
            window_log: 17,
            chain_log: 12,
            hash_log: 13,
            search_log: 1,
            min_match: 5,
            target_length: 0,
            strategy: Strategy::Fast,
        };
        let mut state = FastMatchState::new();

        state.load_cdict_copy_prefix(data, data.len(), params);

        let hash0 = hash_ptr(
            data,
            0,
            params.hash_log + SHORT_CACHE_TAG_BITS,
            params.min_match,
        ) >> SHORT_CACHE_TAG_BITS;
        let hash1 = hash_ptr(
            data,
            1,
            params.hash_log + SHORT_CACHE_TAG_BITS,
            params.min_match,
        ) >> SHORT_CACHE_TAG_BITS;

        assert_eq!(state.hash_table[hash0], 0);
        assert_eq!(state.hash_table[hash1], 1);
    }

    #[test]
    fn generated_transaction_matches_local_for_every_min_match() {
        let mut source = Vec::new();
        for index in 0..4096u32 {
            source.extend_from_slice(&(index % 41).to_le_bytes());
            source.extend_from_slice(b"fast-transaction-reference-pattern");
            if index.is_multiple_of(9) {
                source.extend_from_slice(b"fast-transaction-reference-pattern");
            }
        }

        for min_match in 4..=7 {
            let params = CompressionParameters {
                window_log: 18,
                chain_log: 12,
                hash_log: 17,
                search_log: 1,
                min_match,
                target_length: 1,
                strategy: Strategy::Fast,
            };
            let block_range = 0..source.len();
            let mut local_state = FastMatchState::new();
            let mut codegen_state = local_state.clone();
            let local = compress_block_fast_no_dict_local(
                &source,
                block_range.clone(),
                params,
                RepeatOffsets::new(),
                &mut local_state,
                0,
            );
            let codegen = compress_block_fast_no_dict_codegen(
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
