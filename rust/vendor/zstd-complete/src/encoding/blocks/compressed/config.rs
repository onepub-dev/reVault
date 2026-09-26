#[cfg(feature = "std")]
use core::convert::TryFrom;
#[cfg(feature = "std")]
use std::sync::OnceLock;

use crate::encoding::{CompressionFileProfile, CompressionFileType, CompressionLevel};

pub(super) const FILE_TYPE_SMALL_SEQUENCE_PREDEFINED_LLML_MAX_SEQUENCES: usize = 64;
pub(super) const FILE_TYPE_SINGLE_STREAM_HUFFMAN_MAX_LITERALS: usize = 1024;

#[derive(Clone, Copy)]
pub(crate) struct BlockCompressionConfig {
    pub(super) huffman_table_search: HuffmanTableSearch,
    pub(super) literal_compression_disabled: bool,
    pub(super) literal_compression_min_size: usize,
    pub(super) repeat_table_max_sequences: usize,
    pub(super) offset_table_max_log: u8,
    pub(super) offset_predefined_max_sequences: usize,
    pub(super) exact_sequence_mode_search: bool,
    pub(super) file_type_small_sequence_predefined_llml_max_sequences: Option<usize>,
    pub(super) file_type_single_stream_huffman_max_literals: Option<usize>,
    pub(super) c_fast_sequence_table_heuristics: bool,
    pub(super) c_fast_sequence_emission: bool,
    pub(super) c_dfast_compact_sequence_statistics: bool,
    pub(super) c_cost_sequence_table_selection: bool,
    pub(super) c_literal_cost_model: bool,
    pub(super) prefer_valid_repeat_huffman: bool,
    allocation_free_workspace: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HuffmanTableSearch {
    Heuristic,
    FileTypeSmall,
    AllSections,
}

#[cfg(feature = "std")]
#[derive(Clone, Copy, Debug, Default)]
struct BlockCompressionTuningOverrides {
    huffman_table_search: Option<HuffmanTableSearch>,
    repeat_table_max_sequences: Option<usize>,
    offset_table_max_log: Option<u8>,
    offset_predefined_max_sequences: Option<usize>,
    exact_sequence_mode_search: Option<bool>,
    file_type_small_sequence_predefined_llml_max_sequences: Option<Option<usize>>,
    file_type_single_stream_huffman_max_literals: Option<Option<usize>>,
}

#[cfg(feature = "std")]
static BLOCK_COMPRESSION_TUNING_OVERRIDES: OnceLock<BlockCompressionTuningOverrides> =
    OnceLock::new();

#[cfg(feature = "std")]
static C_NATIVE_SEQUENCE_STORE: OnceLock<bool> = OnceLock::new();

#[cfg(feature = "std")]
static C_GREEDY_NATIVE_SEQUENCE_STORE: OnceLock<bool> = OnceLock::new();

#[cfg(feature = "std")]
static C_OPT_NATIVE_SEQUENCE_STORE: OnceLock<bool> = OnceLock::new();

#[cfg(feature = "std")]
fn block_compression_tuning_overrides() -> &'static BlockCompressionTuningOverrides {
    BLOCK_COMPRESSION_TUNING_OVERRIDES.get_or_init(BlockCompressionTuningOverrides::from_env)
}

#[cfg(feature = "std")]
impl BlockCompressionTuningOverrides {
    fn from_env() -> Self {
        Self {
            huffman_table_search: std::env::var("RUZSTD_TUNE_HUFFMAN_TABLE_SEARCH")
                .ok()
                .and_then(|value| match value.as_str() {
                    "heuristic" => Some(HuffmanTableSearch::Heuristic),
                    "filetype" => Some(HuffmanTableSearch::FileTypeSmall),
                    "allsections" => Some(HuffmanTableSearch::AllSections),
                    _ => None,
                }),
            repeat_table_max_sequences: Self::parse_usize("RUZSTD_TUNE_REPEAT_TABLE_MAX_SEQUENCES"),
            offset_table_max_log: Self::parse_usize("RUZSTD_TUNE_OFFSET_TABLE_MAX_LOG")
                .and_then(|value| u8::try_from(value).ok()),
            offset_predefined_max_sequences: Self::parse_usize(
                "RUZSTD_TUNE_OFFSET_PREDEFINED_MAX_SEQUENCES",
            ),
            exact_sequence_mode_search: std::env::var("RUZSTD_TUNE_EXACT_SEQUENCE_MODE_SEARCH")
                .ok()
                .and_then(|value| Self::parse_bool_value(&value))
                .or_else(|| {
                    std::env::var("RUZSTD_TUNE_EXACT_OFFSET_MODE_SEARCH")
                        .ok()
                        .and_then(|value| Self::parse_bool_value(&value))
                }),
            file_type_small_sequence_predefined_llml_max_sequences: Self::parse_option_usize(
                "RUZSTD_TUNE_FILE_TYPE_SMALL_SEQUENCE_PREDEFINED_LLML_MAX_SEQUENCES",
            ),
            file_type_single_stream_huffman_max_literals: Self::parse_option_usize(
                "RUZSTD_TUNE_FILE_TYPE_SINGLE_STREAM_HUFFMAN_MAX_LITERALS",
            ),
        }
    }

    fn parse_usize(name: &str) -> Option<usize> {
        std::env::var(name).ok()?.parse().ok()
    }

    fn parse_option_usize(name: &str) -> Option<Option<usize>> {
        let value = std::env::var(name).ok()?;
        if value == "none" {
            Some(None)
        } else {
            value.parse().ok().map(Some)
        }
    }

    fn parse_bool_value(value: &str) -> Option<bool> {
        match value {
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON" => Some(true),
            "0" | "false" | "FALSE" | "no" | "NO" | "off" | "OFF" => Some(false),
            _ => None,
        }
    }
}

impl BlockCompressionConfig {
    pub(crate) fn prepare_for_allocation_free_workspace(&mut self) {
        self.huffman_table_search = HuffmanTableSearch::Heuristic;
        self.exact_sequence_mode_search = false;
        self.allocation_free_workspace = true;
    }

    pub(crate) const fn is_allocation_free_workspace(self) -> bool {
        self.allocation_free_workspace
    }

    pub(crate) fn uses_c_fast_entropy_path(self) -> bool {
        self.c_fast_sequence_emission
    }

    /// Enables Fast/DFast's direct C `SeqStore` entropy transaction. The
    /// environment override exists solely for same-binary A/B measurement;
    /// normal builds retain the native path.
    pub(crate) fn uses_c_native_sequence_store(self) -> bool {
        if !self.c_fast_sequence_emission {
            return false;
        }
        if self.allocation_free_workspace {
            return true;
        }

        #[cfg(feature = "std")]
        {
            *C_NATIVE_SEQUENCE_STORE.get_or_init(|| {
                std::env::var("RUZSTD_TUNE_C_NATIVE_SEQUENCE_STORE")
                    .ok()
                    .and_then(|value| BlockCompressionTuningOverrides::parse_bool_value(&value))
                    .unwrap_or(true)
            })
        }
        #[cfg(not(feature = "std"))]
        {
            true
        }
    }

    /// Enables Greedy/Lazy's direct C `SeqStore` entropy transaction. Keep
    /// this control separate from Fast/DFast so same-binary measurements can
    /// attribute the higher-strategy handoff without removing the retained
    /// low-level path.
    pub(crate) fn uses_c_greedy_native_sequence_store(self) -> bool {
        if self.c_fast_sequence_emission || !self.c_literal_cost_model {
            return false;
        }
        if self.allocation_free_workspace {
            return true;
        }

        #[cfg(feature = "std")]
        {
            *C_GREEDY_NATIVE_SEQUENCE_STORE.get_or_init(|| {
                std::env::var("RUZSTD_TUNE_C_GREEDY_NATIVE_SEQUENCE_STORE")
                    .ok()
                    .and_then(|value| BlockCompressionTuningOverrides::parse_bool_value(&value))
                    .unwrap_or(true)
            })
        }
        #[cfg(not(feature = "std"))]
        {
            true
        }
    }

    /// Enables the optimal parsers' direct C `SeqStore` entropy transaction.
    /// Target-size and post-split callers retain their prepared representation
    /// and therefore do not consult this gate.
    pub(crate) fn uses_c_opt_native_sequence_store(self) -> bool {
        if self.c_fast_sequence_emission || !self.c_literal_cost_model {
            return false;
        }
        if self.allocation_free_workspace {
            return true;
        }

        #[cfg(feature = "std")]
        {
            *C_OPT_NATIVE_SEQUENCE_STORE.get_or_init(|| {
                std::env::var("RUZSTD_TUNE_C_OPT_NATIVE_SEQUENCE_STORE")
                    .ok()
                    .and_then(|value| BlockCompressionTuningOverrides::parse_bool_value(&value))
                    .unwrap_or(true)
            })
        }
        #[cfg(not(feature = "std"))]
        {
            true
        }
    }

    pub(super) fn search_smallest_huffman_table(
        self,
        literal_count: usize,
        sequence_count: usize,
    ) -> bool {
        match self.huffman_table_search {
            HuffmanTableSearch::Heuristic => {
                !self.c_literal_cost_model
                    && (sequence_count == 0
                        || (sequence_count <= super::SMALL_HUFFMAN_TABLE_SEARCH_MAX_SEQUENCES
                            && literal_count <= super::SMALL_HUFFMAN_TABLE_SEARCH_MAX_LITERALS))
            }
            HuffmanTableSearch::FileTypeSmall => {
                literal_count <= super::FILE_TYPE_SMALL_HUFFMAN_TABLE_SEARCH_MAX_LITERALS
                    || sequence_count == 0
                    || (sequence_count <= super::SMALL_HUFFMAN_TABLE_SEARCH_MAX_SEQUENCES
                        && literal_count <= super::SMALL_HUFFMAN_TABLE_SEARCH_MAX_LITERALS)
            }
            HuffmanTableSearch::AllSections => true,
        }
    }

    pub(crate) fn for_c_strategy(strategy: u8) -> Self {
        let literal_compression_min_size = c_min_literals_to_compress(strategy);
        let fastish = strategy < 4;
        if !fastish {
            return Self {
                huffman_table_search: if strategy >= 8 {
                    HuffmanTableSearch::AllSections
                } else {
                    HuffmanTableSearch::Heuristic
                },
                literal_compression_disabled: false,
                literal_compression_min_size,
                repeat_table_max_sequences: 64,
                offset_table_max_log: 8,
                offset_predefined_max_sequences: 16,
                exact_sequence_mode_search: false,
                file_type_small_sequence_predefined_llml_max_sequences: None,
                file_type_single_stream_huffman_max_literals: None,
                c_fast_sequence_table_heuristics: false,
                c_fast_sequence_emission: false,
                c_dfast_compact_sequence_statistics: false,
                c_cost_sequence_table_selection: true,
                c_literal_cost_model: true,
                prefer_valid_repeat_huffman: false,
                allocation_free_workspace: false,
            };
        }

        let multiplier = 10usize.saturating_sub(strategy as usize);
        let llml_predefined_max_sequences = ((1usize << 6) * multiplier) >> 3;
        let offset_predefined_max_sequences = ((1usize << 5) * multiplier) >> 3;

        Self {
            huffman_table_search: HuffmanTableSearch::Heuristic,
            literal_compression_disabled: false,
            literal_compression_min_size,
            repeat_table_max_sequences: 1000,
            offset_table_max_log: 8,
            offset_predefined_max_sequences,
            exact_sequence_mode_search: false,
            file_type_small_sequence_predefined_llml_max_sequences: Some(
                llml_predefined_max_sequences,
            ),
            file_type_single_stream_huffman_max_literals: None,
            c_fast_sequence_table_heuristics: true,
            c_fast_sequence_emission: strategy <= 2,
            c_dfast_compact_sequence_statistics: strategy == 2,
            c_cost_sequence_table_selection: false,
            c_literal_cost_model: true,
            prefer_valid_repeat_huffman: false,
            allocation_free_workspace: false,
        }
    }

    pub(crate) fn for_level(level: CompressionLevel) -> Self {
        Self::for_level_and_file_type(level, CompressionFileType::Unknown)
    }

    pub(crate) fn for_level_and_file_type(
        level: CompressionLevel,
        file_type: CompressionFileType,
    ) -> Self {
        Self::for_level_and_hints(level, file_type, CompressionFileProfile::None)
    }

    pub(crate) fn for_level_and_hints(
        level: CompressionLevel,
        file_type: CompressionFileType,
        file_profile: CompressionFileProfile,
    ) -> Self {
        let huffman_table_search = if level.uses_best_legacy_profile() {
            HuffmanTableSearch::Heuristic
        } else {
            if matches!(file_type, CompressionFileType::DictionaryText) {
                HuffmanTableSearch::AllSections
            } else if matches!(
                file_type,
                CompressionFileType::CodeText
                    | CompressionFileType::ConfigText
                    | CompressionFileType::Unknown
            ) {
                HuffmanTableSearch::FileTypeSmall
            } else {
                HuffmanTableSearch::Heuristic
            }
        };
        let repeat_table_max_sequences = if level.uses_best_legacy_profile() {
            256
        } else {
            64
        };
        let mut config = Self {
            huffman_table_search,
            literal_compression_disabled: false,
            literal_compression_min_size: super::literals::COMPRESS_LITERALS_SIZE_MIN,
            repeat_table_max_sequences,
            offset_table_max_log: if matches!(file_type, CompressionFileType::DictionaryText)
                || (matches!(file_type, CompressionFileType::Unknown)
                    && level.uses_fastest_legacy_profile())
            {
                7
            } else {
                8
            },
            offset_predefined_max_sequences: 16,
            exact_sequence_mode_search: level.uses_fastest_legacy_profile()
                && matches!(file_type, CompressionFileType::DictionaryText),
            file_type_small_sequence_predefined_llml_max_sequences: if matches!(
                file_type,
                CompressionFileType::Unknown | CompressionFileType::ConfigText
            ) && level
                .uses_fastest_legacy_profile()
            {
                Some(FILE_TYPE_SMALL_SEQUENCE_PREDEFINED_LLML_MAX_SEQUENCES)
            } else {
                None
            },
            file_type_single_stream_huffman_max_literals: if level.uses_fastest_legacy_profile()
                && matches!(file_type, CompressionFileType::ConfigText)
            {
                Some(FILE_TYPE_SINGLE_STREAM_HUFFMAN_MAX_LITERALS)
            } else {
                None
            },
            c_fast_sequence_table_heuristics: false,
            c_fast_sequence_emission: false,
            c_dfast_compact_sequence_statistics: false,
            c_cost_sequence_table_selection: false,
            c_literal_cost_model: false,
            prefer_valid_repeat_huffman: false,
            allocation_free_workspace: false,
        };
        #[cfg(feature = "std")]
        config.apply_tuning_overrides();
        if matches!(file_profile, CompressionFileProfile::SmallTextLockfile) {
            config.apply_small_text_lockfile_tuning();
        } else if matches!(file_profile, CompressionFileProfile::DependencyJsonLockfile) {
            config.apply_dependency_json_lockfile_tuning();
        }
        config
    }

    #[cfg(feature = "std")]
    fn apply_tuning_overrides(&mut self) {
        let overrides = block_compression_tuning_overrides();
        if let Some(value) = overrides.huffman_table_search {
            self.huffman_table_search = value;
        }
        if let Some(value) = overrides.repeat_table_max_sequences {
            self.repeat_table_max_sequences = value;
        }
        if let Some(value) = overrides.offset_table_max_log {
            self.offset_table_max_log = value;
        }
        if let Some(value) = overrides.offset_predefined_max_sequences {
            self.offset_predefined_max_sequences = value;
        }
        if let Some(value) = overrides.exact_sequence_mode_search {
            self.exact_sequence_mode_search = value;
        }
        if let Some(value) = overrides.file_type_small_sequence_predefined_llml_max_sequences {
            self.file_type_small_sequence_predefined_llml_max_sequences = value;
        }
        if let Some(value) = overrides.file_type_single_stream_huffman_max_literals {
            self.file_type_single_stream_huffman_max_literals = value;
        }
    }

    pub(super) fn apply_dependency_json_lockfile_tuning(&mut self) {
        self.huffman_table_search = HuffmanTableSearch::AllSections;
        self.repeat_table_max_sequences = 256;
        self.offset_table_max_log = 8;
        self.exact_sequence_mode_search = true;
    }

    pub(crate) fn disable_literal_compression(&mut self) {
        self.literal_compression_disabled = true;
    }

    pub(crate) fn set_prefer_valid_repeat_huffman(&mut self, enabled: bool) {
        self.prefer_valid_repeat_huffman = enabled;
    }

    pub(crate) fn for_c_block_split_estimate(mut self) -> Self {
        if !matches!(self.huffman_table_search, HuffmanTableSearch::AllSections) {
            self.huffman_table_search = HuffmanTableSearch::Heuristic;
        }
        self
    }

    #[cfg(test)]
    pub(crate) fn literal_compression_disabled(&self) -> bool {
        self.literal_compression_disabled
    }

    fn apply_small_text_lockfile_tuning(&mut self) {
        self.huffman_table_search = HuffmanTableSearch::AllSections;
        self.repeat_table_max_sequences = 256;
        self.offset_table_max_log = 7;
        self.offset_predefined_max_sequences = 64;
    }
}

fn c_min_literals_to_compress(strategy: u8) -> usize {
    let shift = usize::from(9u8.saturating_sub(strategy)).min(3);
    8usize << shift
}
