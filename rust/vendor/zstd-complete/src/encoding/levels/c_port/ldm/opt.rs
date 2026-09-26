use super::sequence::LdmRawSequence;
use crate::encoding::levels::c_port::{
    opt_match::{OptMatch, OptMatchTable},
    sequence_store::OffBase,
};

const NO_LDM_POSITION: u32 = u32::MAX;
const ZSTD_OPT_NUM: usize = 1 << 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LdmRawSeqStore<'a> {
    sequences: &'a [LdmRawSequence],
    pos: usize,
    pos_in_sequence: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LdmOptCursor<'a> {
    seq_store: LdmRawSeqStore<'a>,
    start_pos_in_block: u32,
    end_pos_in_block: u32,
    offset: u32,
}

impl<'a> LdmRawSeqStore<'a> {
    pub(crate) const fn new(sequences: &'a [LdmRawSequence]) -> Self {
        Self {
            sequences,
            pos: 0,
            pos_in_sequence: 0,
        }
    }

    pub(crate) fn skip_bytes(&mut self, bytes: u32) {
        let mut curr_pos = self.pos_in_sequence + bytes;
        while curr_pos != 0 && self.pos < self.sequences.len() {
            let sequence = self.sequences[self.pos];
            let sequence_len = sequence.lit_length + sequence.match_length;
            if curr_pos >= sequence_len {
                curr_pos -= sequence_len;
                self.pos += 1;
            } else {
                self.pos_in_sequence = curr_pos;
                break;
            }
        }

        if curr_pos == 0 || self.pos == self.sequences.len() {
            self.pos_in_sequence = 0;
        }
    }

    pub(crate) fn position(self) -> (usize, u32) {
        (self.pos, self.pos_in_sequence)
    }
}

impl<'a> LdmOptCursor<'a> {
    pub(crate) fn new(sequences: &'a [LdmRawSequence], block_size: u32) -> Self {
        Self::from_store_for_block(LdmRawSeqStore::new(sequences), block_size)
    }

    pub(crate) fn from_store_for_block(seq_store: LdmRawSeqStore<'a>, block_size: u32) -> Self {
        let mut cursor = Self {
            seq_store,
            start_pos_in_block: 0,
            end_pos_in_block: 0,
            offset: 0,
        };
        cursor.get_next_match_and_update_seq_store(0, block_size);
        cursor
    }

    pub(crate) fn process_match_candidate(
        &mut self,
        matches: &mut OptMatchTable,
        curr_pos_in_block: u32,
        remaining_bytes: u32,
        min_match: u32,
    ) {
        if self.start_pos_in_block == NO_LDM_POSITION
            && self.seq_store.pos >= self.seq_store.sequences.len()
        {
            return;
        }

        if curr_pos_in_block >= self.end_pos_in_block {
            if curr_pos_in_block > self.end_pos_in_block {
                self.seq_store
                    .skip_bytes(curr_pos_in_block - self.end_pos_in_block);
            }
            self.get_next_match_and_update_seq_store(curr_pos_in_block, remaining_bytes);
        }
        self.maybe_add_match(matches, curr_pos_in_block, min_match);
    }

    pub(crate) fn current_match(self) -> Option<(u32, u32, u32)> {
        (self.start_pos_in_block != NO_LDM_POSITION).then_some((
            self.start_pos_in_block,
            self.end_pos_in_block,
            self.offset,
        ))
    }

    pub(crate) fn seq_store_position(self) -> (usize, u32) {
        self.seq_store.position()
    }

    fn get_next_match_and_update_seq_store(
        &mut self,
        curr_pos_in_block: u32,
        block_bytes_remaining: u32,
    ) {
        if self.seq_store.pos >= self.seq_store.sequences.len() {
            self.disable_for_block();
            return;
        }

        let sequence = self.seq_store.sequences[self.seq_store.pos];
        debug_assert!(
            self.seq_store.pos_in_sequence <= sequence.lit_length + sequence.match_length
        );
        let curr_block_end_pos = curr_pos_in_block + block_bytes_remaining;
        let literals_bytes_remaining = sequence
            .lit_length
            .saturating_sub(self.seq_store.pos_in_sequence);
        let match_bytes_remaining = if literals_bytes_remaining == 0 {
            sequence.match_length - (self.seq_store.pos_in_sequence - sequence.lit_length)
        } else {
            sequence.match_length
        };

        if literals_bytes_remaining >= block_bytes_remaining {
            self.disable_for_block();
            self.seq_store.skip_bytes(block_bytes_remaining);
            return;
        }

        self.start_pos_in_block = curr_pos_in_block + literals_bytes_remaining;
        self.end_pos_in_block = self.start_pos_in_block + match_bytes_remaining;
        self.offset = sequence.offset;

        if self.end_pos_in_block > curr_block_end_pos {
            self.end_pos_in_block = curr_block_end_pos;
            self.seq_store
                .skip_bytes(curr_block_end_pos - curr_pos_in_block);
        } else {
            self.seq_store
                .skip_bytes(literals_bytes_remaining + match_bytes_remaining);
        }
    }

    fn maybe_add_match(&self, matches: &mut OptMatchTable, curr_pos_in_block: u32, min_match: u32) {
        if curr_pos_in_block < self.start_pos_in_block || curr_pos_in_block >= self.end_pos_in_block
        {
            return;
        }

        let pos_diff = curr_pos_in_block - self.start_pos_in_block;
        let candidate_match_length = self.end_pos_in_block - self.start_pos_in_block - pos_diff;
        if candidate_match_length < min_match {
            return;
        }

        if matches
            .last()
            .is_none_or(|last| candidate_match_length > last.len && matches.len() < ZSTD_OPT_NUM)
        {
            matches.push(OptMatch {
                off_base: OffBase::offset_to_c_value(self.offset),
                len: candidate_match_length,
            });
        }
    }

    fn disable_for_block(&mut self) {
        self.start_pos_in_block = NO_LDM_POSITION;
        self.end_pos_in_block = NO_LDM_POSITION;
    }
}
