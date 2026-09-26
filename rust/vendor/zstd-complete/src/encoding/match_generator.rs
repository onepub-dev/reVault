//! Matching algorithm used find repeated parts in the original data
//!
//! The Zstd format relies on finden repeated sequences of data and compressing these sequences as instructions to the decoder.
//! A sequence basically tells the decoder "Go back X bytes and copy Y bytes to the end of your decode buffer".
//!
//! The task here is to efficiently find matches in the already encoded data for the current suffix of the not yet encoded data.

use alloc::vec::Vec;
#[cfg(all(test, feature = "std"))]
use core::cell::Ref;
#[cfg(test)]
use core::cell::RefCell;
use core::convert::TryFrom;
use core::num::NonZeroU32;
mod byte_match;
mod candidate_building;
#[cfg(test)]
mod diagnostics;
mod driver;
mod emit;
mod repeat_search;
mod selection_policy;
mod sequence;
mod sidecar;
mod suffix_store;
mod text;
#[cfg(feature = "std")]
mod tuning;
mod types;
mod window_search;

#[cfg(test)]
use diagnostics::{CandidateSource, MatcherDiagnostics, RepeatNextPositionSelectionReason};
pub use driver::MatchGeneratorDriver;
use suffix_store::SuffixStore;
#[cfg(test)]
use suffix_store::{Candidates, INITIAL_TOUCHED_SLOT_CAPACITY, TOUCHED_SLOT_CLEAR_LIMIT};
#[cfg(feature = "std")]
use tuning::matcher_tuning_overrides;
use types::{
    MatchCandidate, MatchCandidateContext, RepeatCandidateKind, WindowCandidateKind,
    WindowCandidateMeta, WindowEntry,
};

use super::frame_compressor::OffsetHistory;
use super::util::{likely_composer_lockfile_text, likely_lockfile_text};
use super::CompressionFileProfile;
use super::CompressionFileType;
use super::CompressionLevel;
use super::Matcher;
use super::Sequence;

const MIN_MATCH_LEN: usize = 5;
const SMALL_TEXT_MIN_NON_REPEAT_MATCH_LEN: usize = 5;
const CODE_LIKE_SHORT_TEXT_MIN_NON_REPEAT_MATCH_LEN: usize = 6;
const TEXT_MIN_NON_REPEAT_MATCH_LEN: usize = 8;
const SHORT_TEXT_LINE_LEN_LIMIT: usize = 96;
const SHORT_TEXT_LINE_FRACTION_PERCENT: usize = 95;
const CODE_LIKE_SEMI_PER_100_LINES: usize = 15;
const CODE_LIKE_BRACES_PER_100_LINES: usize = 15;
const SMALL_CODE_TEXT_MIN_NON_REPEAT_MAX_BLOCK_LEN: usize = 16 * 1024;
const CODE_TEXT_DENSE_PROBE_MAX_BLOCK_LEN: usize = 96 * 1024;
const CONFIG_TEXT_DENSE_PROBE_MAX_BLOCK_LEN: usize = 8 * 1024;
const STRUCTURED_JSON_CONFIG_DENSE_PROBE_MAX_BLOCK_LEN: usize = 128 * 1024;
const SHORT_LINE_TEXT_NO_MATCH_PROBE_STEP: usize = 2;
const TSCONFIG_JSON_TEXT_NO_MATCH_PROBE_STEP: usize = 6;
const COMPOSER_JSON_LOCKFILE_NO_MATCH_PROBE_STEP: usize = 5;
const REPEAT_MATCH_LEN_MARGIN: usize = 2;
const DICTIONARY_SMALLER_OFFSET_BITS_GAIN_MIN: usize = 2;
const DICTIONARY_SMALLER_OFFSET_MATCH_LOSS_MAX: usize = 1;
const LARGE_UNKNOWN_SMALLER_OFFSET_BITS_GAIN_MIN: usize = 2;
const LARGE_UNKNOWN_SMALLER_OFFSET_MATCH_LOSS_MAX: usize = 1;
const LARGE_UNKNOWN_NEWEST_DISPLACEMENT_MIN_GAIN: usize = 2;
const LARGE_UNKNOWN_OLDEST_DISPLACEMENT_MIN_GAIN: usize = 2;
const LOCKFILE_OLDEST_DISPLACEMENT_MIN_GAIN: usize = 2;
const LOCKFILE_SAME_END_SMALLER_OFFSET_MATCH_LOSS_MAX: usize = 1;
const LOCKFILE_REPEAT_KIND_MATCH_LOSS_MAX: usize = 1;
const LOCKFILE_NEXT_POSITION_MAX_CURRENT_MATCH_LEN: usize = 7;
const LOCKFILE_NEXT_POSITION_MAX_SKIP_LITERALS: usize = 2;
const LOCKFILE_NEXT_POSITION_MATCH_LOSS_MAX: usize = 0;
const LOCKFILE_NEXT_POSITION_LITERAL_WEIGHT: usize = 6;
const LOCKFILE_NEXT_POSITION_MATCH_REWARD: usize = 2;
const LOCKFILE_NEXT_POSITION_OFFSET_WEIGHT: usize = 3;
const LOCKFILE_NEXT_POSITION_MARGIN: usize = 1;
const COMPOSER_REPEAT_KIND_MATCH_LOSS_MAX: usize = 2;
const REPEAT_SEARCH_EARLY_EXIT_LEN: usize = 10;
const DENSE_MATCH_INDEX_LIMIT: usize = 128;
const NO_MATCH_PROBE_STEP: usize = 2;
const TEXT_NO_MATCH_PROBE_STEP: usize = 3;
const BEST_BINARY_NO_MATCH_SEARCH_STRENGTH: usize = 8;
const BEST_SECOND_NEWEST_RECENT_ENTRY_LIMIT: usize = 1;
const LOCKFILE_SECOND_NEWEST_RECENT_ENTRY_LIMIT: usize = 2;
const BEST_SECOND_NEWEST_MIN_BLOCK_LEN: usize = 16 * 1024;
const CODE_TEXT_SECOND_NEWEST_MAX_BLOCK_LEN: usize = 64 * 1024;
const FASTEST_SECOND_NEWEST_MAX_BLOCK_LEN: usize = 64 * 1024;
const FASTEST_UNKNOWN_SECOND_NEWEST_MAX_BLOCK_LEN: usize = 128 * 1024;
const FASTEST_DENSE_BINARY_PROBE_MAX_BLOCK_LEN: usize = 64 * 1024;
const BEST_CURRENT_LONG_HASH_MIN_BLOCK_LEN: usize = 32 * 1024;
const BEST_CURRENT_LONG_HASH_SKIP_OLDER_LEN: usize = 56;
const BEST_CURRENT_LONG_HASH_DISTANT_NEWEST_ENTRY_START: usize = 2;
const SPARSE_MATCH_END_INDEX_BACKOFF: usize = 2;
const SUFFIX_STORE_CAPACITY_DIVISOR: usize = 16;
const BEST_SUFFIX_STORE_CAPACITY_MULTIPLIER: usize = 2;
const FASTEST_WINDOW_BLOCKS: usize = 4;
const BEST_WINDOW_BLOCKS: usize = 16;

pub(crate) struct MatchGenerator {
    max_window_size: usize,
    fast_window_size: usize,
    /// Data window we are operating on to find matches
    /// The data we want to find matches for is in the last slice
    window: Vec<WindowEntry>,
    current_second_newest_sidecar: Vec<Option<NonZeroU32>>,
    current_third_newest_sidecar: Vec<Option<NonZeroU32>>,
    current_fourth_newest_sidecar: Vec<Option<NonZeroU32>>,
    current_fifth_newest_sidecar: Vec<Option<NonZeroU32>>,
    current_sixth_newest_sidecar: Vec<Option<NonZeroU32>>,
    current_seventh_newest_sidecar: Vec<Option<NonZeroU32>>,
    current_eighth_newest_sidecar: Vec<Option<NonZeroU32>>,
    current_ninth_newest_sidecar: Vec<Option<NonZeroU32>>,
    current_tenth_newest_sidecar: Vec<Option<NonZeroU32>>,
    current_eleventh_newest_sidecar: Vec<Option<NonZeroU32>>,
    current_twelfth_newest_sidecar: Vec<Option<NonZeroU32>>,
    current_thirteenth_newest_sidecar: Vec<Option<NonZeroU32>>,
    current_long_hash: Vec<Option<NonZeroU32>>,
    window_size: usize,
    #[cfg(debug_assertions)]
    concat_window: Vec<u8>,
    uniform_suffix_len_log: Option<u32>,
    /// Index in the last slice that we already processed
    suffix_idx: usize,
    /// Gets updated when a new sequence is returned to point right behind that sequence
    last_idx_in_sequence: usize,
    offset_history: OffsetHistory,
    is_text_block: bool,
    is_short_line_text: bool,
    min_non_repeat_match_len: usize,
    adaptive_binary_no_match_probe: bool,
    use_fast_small_dense_binary_probe: bool,
    prefer_binary_next_position_repeat_lookahead: bool,
    prefer_fast_binary_next_position_repeat_lookahead: bool,
    prefer_binary_next_position_lookahead: bool,
    prefer_oldest_first_window_probe: bool,
    use_complementary_end_insertion: bool,
    use_second_newest_probe: bool,
    use_fast_binary_small_second_newest: bool,
    use_text_repeat_pipeline: bool,
    current_block_is_dictionary_lockfile_text: bool,
    current_block_is_composer_dictionary_text: bool,
    current_block_is_structured_json_config_text: bool,
    current_block_is_tsconfig_json_config_text: bool,
    file_type_hint: CompressionFileType,
    file_profile_hint: CompressionFileProfile,
    #[cfg(test)]
    diagnostics: RefCell<MatcherDiagnostics>,
}

impl MatchGenerator {
    /// max_size defines how many bytes will be used at most in the window used for matching
    fn new(max_size: usize) -> Self {
        Self {
            max_window_size: max_size,
            fast_window_size: max_size,
            window: Vec::new(),
            current_second_newest_sidecar: Vec::new(),
            current_third_newest_sidecar: Vec::new(),
            current_fourth_newest_sidecar: Vec::new(),
            current_fifth_newest_sidecar: Vec::new(),
            current_sixth_newest_sidecar: Vec::new(),
            current_seventh_newest_sidecar: Vec::new(),
            current_eighth_newest_sidecar: Vec::new(),
            current_ninth_newest_sidecar: Vec::new(),
            current_tenth_newest_sidecar: Vec::new(),
            current_eleventh_newest_sidecar: Vec::new(),
            current_twelfth_newest_sidecar: Vec::new(),
            current_thirteenth_newest_sidecar: Vec::new(),
            current_long_hash: Vec::new(),
            window_size: 0,
            #[cfg(debug_assertions)]
            concat_window: Vec::new(),
            uniform_suffix_len_log: None,
            suffix_idx: 0,
            last_idx_in_sequence: 0,
            offset_history: OffsetHistory::new(),
            is_text_block: false,
            is_short_line_text: false,
            min_non_repeat_match_len: MIN_MATCH_LEN,
            adaptive_binary_no_match_probe: false,
            use_fast_small_dense_binary_probe: false,
            prefer_binary_next_position_repeat_lookahead: false,
            prefer_fast_binary_next_position_repeat_lookahead: false,
            prefer_binary_next_position_lookahead: false,
            prefer_oldest_first_window_probe: false,
            use_complementary_end_insertion: false,
            use_second_newest_probe: false,
            use_fast_binary_small_second_newest: false,
            use_text_repeat_pipeline: false,
            current_block_is_dictionary_lockfile_text: false,
            current_block_is_composer_dictionary_text: false,
            current_block_is_structured_json_config_text: false,
            current_block_is_tsconfig_json_config_text: false,
            file_type_hint: CompressionFileType::Unknown,
            file_profile_hint: CompressionFileProfile::None,
            #[cfg(test)]
            diagnostics: RefCell::new(MatcherDiagnostics::default()),
        }
    }

    fn reset(&mut self, mut reuse_space: impl FnMut(Vec<u8>, SuffixStore)) {
        self.window_size = 0;
        self.current_second_newest_sidecar.clear();
        self.current_third_newest_sidecar.clear();
        self.current_fourth_newest_sidecar.clear();
        self.current_fifth_newest_sidecar.clear();
        self.current_sixth_newest_sidecar.clear();
        self.current_seventh_newest_sidecar.clear();
        self.current_eighth_newest_sidecar.clear();
        self.current_ninth_newest_sidecar.clear();
        self.current_tenth_newest_sidecar.clear();
        self.current_eleventh_newest_sidecar.clear();
        self.current_twelfth_newest_sidecar.clear();
        self.current_thirteenth_newest_sidecar.clear();
        self.current_long_hash.clear();
        #[cfg(debug_assertions)]
        self.concat_window.clear();
        self.uniform_suffix_len_log = None;
        self.suffix_idx = 0;
        self.last_idx_in_sequence = 0;
        self.offset_history = OffsetHistory::new();
        self.is_text_block = false;
        self.is_short_line_text = false;
        self.min_non_repeat_match_len = MIN_MATCH_LEN;
        self.adaptive_binary_no_match_probe = false;
        self.use_fast_small_dense_binary_probe = false;
        self.prefer_binary_next_position_repeat_lookahead = false;
        self.prefer_fast_binary_next_position_repeat_lookahead = false;
        self.prefer_binary_next_position_lookahead = false;
        self.prefer_oldest_first_window_probe = false;
        self.use_complementary_end_insertion = false;
        self.use_second_newest_probe = false;
        self.use_fast_binary_small_second_newest = false;
        self.use_text_repeat_pipeline = false;
        self.current_block_is_dictionary_lockfile_text = false;
        self.current_block_is_composer_dictionary_text = false;
        self.current_block_is_structured_json_config_text = false;
        self.current_block_is_tsconfig_json_config_text = false;
        self.file_type_hint = CompressionFileType::Unknown;
        self.file_profile_hint = CompressionFileProfile::None;
        #[cfg(test)]
        {
            *self.diagnostics.borrow_mut() = MatcherDiagnostics::default();
        }
        self.window.drain(..).for_each(|entry| {
            reuse_space(entry.data, entry.suffixes);
        });
    }

    fn set_window_sizes(&mut self, max_size: usize, fast_size: usize) {
        debug_assert!(self.window.is_empty());
        self.max_window_size = max_size;
        self.fast_window_size = fast_size.min(max_size);
    }

    #[inline(always)]
    fn last_entry(&self) -> &WindowEntry {
        match self.window.last() {
            Some(entry) => entry,
            None => Self::missing_window_entry(),
        }
    }

    #[inline(always)]
    fn last_entry_mut(&mut self) -> &mut WindowEntry {
        match self.window.last_mut() {
            Some(entry) => entry,
            None => Self::missing_window_entry(),
        }
    }

    #[inline(always)]
    fn last_entry_index(&self) -> usize {
        match self.window.len().checked_sub(1) {
            Some(idx) => idx,
            None => Self::missing_window_entry(),
        }
    }

    #[cold]
    #[inline(never)]
    fn missing_window_entry() -> ! {
        panic!("match generator requires a committed window entry")
    }

    #[inline(always)]
    fn bounded_u32(value: usize) -> u32 {
        match u32::try_from(value) {
            Ok(value) => value,
            Err(_) => unreachable!("match generator indexes are bounded by the compressor window"),
        }
    }
}

#[cfg(test)]
mod tests;
