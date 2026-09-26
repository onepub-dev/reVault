use alloc::vec::Vec;

#[cfg(test)]
use super::diagnostics::CandidateSource;
use super::suffix_store::SuffixStore;
use super::REPEAT_MATCH_LEN_MARGIN;
#[cfg(test)]
use super::REPEAT_SEARCH_EARLY_EXIT_LEN;

/// We keep a window of a few of these entries
/// All of these are valid targets for a match to be generated for
pub(super) struct WindowEntry {
    pub(super) data: Vec<u8>,
    /// Stores indexes into data
    pub(super) suffixes: SuffixStore,
    /// Makes offset calculations efficient
    pub(super) base_offset: usize,
}

pub(super) struct MatchCandidateContext<'data> {
    pub(super) suffix_idx: usize,
    pub(super) anchor_idx: usize,
    pub(super) min_non_repeat_match_len: usize,
    pub(super) data_slice: &'data [u8],
    #[cfg(debug_assertions)]
    pub(super) last_entry_len: usize,
    #[cfg(debug_assertions)]
    pub(super) concat_window: &'data [u8],
}

#[derive(Clone, Copy)]
pub(super) enum WindowCandidateKind {
    Oldest,
    Newest,
    SecondNewest,
}

#[derive(Clone, Copy)]
#[cfg_attr(not(test), allow(dead_code))]
pub(super) struct WindowCandidateMeta {
    pub(super) entry_distance: usize,
    pub(super) kind: WindowCandidateKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RepeatCandidateKind {
    First,
    Second,
    Third,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct MatchCandidate {
    pub(super) start_idx: usize,
    pub(super) offset: usize,
    pub(super) match_len: usize,
    pub(super) repeat_offset: bool,
    #[cfg(test)]
    pub(super) source: CandidateSource,
}

impl MatchCandidate {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn is_better_than(self, other: Self) -> bool {
        if self.repeat_offset != other.repeat_offset {
            if self.repeat_offset {
                return self.match_len + REPEAT_MATCH_LEN_MARGIN >= other.match_len;
            }
            return self.match_len > other.match_len + REPEAT_MATCH_LEN_MARGIN;
        }

        self.match_len > other.match_len
            || (self.match_len == other.match_len
                && (self.start_idx < other.start_idx
                    || self.start_idx == other.start_idx && self.offset < other.offset))
    }

    pub(super) fn worth_emitting(self, min_non_repeat_match_len: usize) -> bool {
        self.repeat_offset || self.match_len >= min_non_repeat_match_len
    }

    #[cfg(test)]
    pub(super) fn can_skip_window_search(self, block_len: usize) -> bool {
        self.repeat_offset
            && (self.start_idx + self.match_len == block_len
                || self.match_len >= REPEAT_SEARCH_EARLY_EXIT_LEN)
    }

    #[cfg(test)]
    pub(super) fn source_repeat_kind(self) -> RepeatCandidateKind {
        match self.source {
            CandidateSource::RepeatCurrent(kind) | CandidateSource::RepeatNextPosition(kind) => {
                kind
            }
            other => panic!("expected repeat candidate source, got {:?}", other),
        }
    }
}
