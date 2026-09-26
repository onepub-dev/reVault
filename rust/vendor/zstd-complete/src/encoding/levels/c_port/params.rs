//! Compression parameter selection ported from `lib/compress/clevels.h` and
//! `ZSTD_getCParams_internal()` in the upstream C implementation.

const KIB: u64 = 1024;

pub(crate) const DEFAULT_COMPRESSION_LEVEL: i32 = 3;
pub(crate) const MAX_COMPRESSION_LEVEL: i32 = 22;
pub(crate) const MIN_COMPRESSION_LEVEL: i32 = -(ZSTD_BLOCKSIZE_MAX as i32);

const ZSTD_BLOCKSIZE_MAX: u32 = 128 * KIB as u32;
pub(crate) const ZSTD_CONTENTSIZE_UNKNOWN: u64 = u64::MAX;
const ZSTD_WINDOWLOG_ABSOLUTE_MIN: u32 = 10;
const ZSTD_HASHLOG_MIN: u32 = 6;
const ZSTD_ROW_HASH_TAG_BITS: u32 = 8;
const ZSTD_SHORT_CACHE_TAG_BITS: u32 = 8;
const KIB_USIZE: usize = 1024;

#[cfg(target_pointer_width = "64")]
const ZSTD_WINDOWLOG_MAX: u32 = 31;
#[cfg(not(target_pointer_width = "64"))]
const ZSTD_WINDOWLOG_MAX: u32 = 30;

/// Compression strategies in the same order as `ZSTD_strategy`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub(crate) enum Strategy {
    Fast = 1,
    DFast = 2,
    Greedy = 3,
    Lazy = 4,
    Lazy2 = 5,
    BtLazy2 = 6,
    BtOpt = 7,
    BtUltra = 8,
    BtUltra2 = 9,
}

/// Rust equivalent of `ZSTD_CParamMode_e`.
///
/// The mode controls how dictionary size participates in C parameter table
/// selection and post-selection adjustment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CParamMode {
    Unknown,
    NoAttachDict,
    CreateCDict,
    AttachDict,
}

/// Direct Rust equivalent of `ZSTD_compressionParameters`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CompressionParameters {
    pub(crate) window_log: u32,
    pub(crate) chain_log: u32,
    pub(crate) hash_log: u32,
    pub(crate) search_log: u32,
    pub(crate) min_match: u32,
    pub(crate) target_length: u32,
    pub(crate) strategy: Strategy,
}

impl CompressionParameters {
    const fn new(
        window_log: u32,
        chain_log: u32,
        hash_log: u32,
        search_log: u32,
        min_match: u32,
        target_length: u32,
        strategy: Strategy,
    ) -> Self {
        Self {
            window_log,
            chain_log,
            hash_log,
            search_log,
            min_match,
            target_length,
            strategy,
        }
    }

    /// Port of `ZSTD_getCParams()`.
    pub(crate) fn for_level(compression_level: i32, src_size_hint: u64, dict_size: usize) -> Self {
        let src_size_hint = if src_size_hint == 0 {
            ZSTD_CONTENTSIZE_UNKNOWN
        } else {
            src_size_hint
        };
        Self::for_level_with_mode(
            compression_level,
            src_size_hint,
            dict_size,
            CParamMode::Unknown,
        )
    }

    /// Port of `ZSTD_getCParams_internal()`.
    ///
    /// Unlike `for_level()`, this follows the internal C contract where
    /// `src_size_hint == 0` means an empty source rather than unknown size.
    pub(crate) fn for_level_with_mode(
        compression_level: i32,
        src_size_hint: u64,
        dict_size: usize,
        mode: CParamMode,
    ) -> Self {
        let row_size = c_param_row_size(src_size_hint, dict_size, mode);
        let table_id = usize::from(row_size <= 256 * KIB)
            + usize::from(row_size <= 128 * KIB)
            + usize::from(row_size <= 16 * KIB);
        let row = if compression_level == 0 {
            DEFAULT_COMPRESSION_LEVEL as usize
        } else if compression_level < 0 {
            0
        } else {
            compression_level.min(MAX_COMPRESSION_LEVEL) as usize
        };

        let mut params = DEFAULT_C_PARAMETERS[table_id][row];
        if compression_level < 0 {
            params.target_length = compression_level.max(MIN_COMPRESSION_LEVEL).unsigned_abs();
        }
        params.adjust(src_size_hint, dict_size, mode)
    }

    /// Port of applying `ZSTD_adjustCParams_internal()` to an already selected
    /// C parameter row.
    pub(crate) fn adjusted_for_mode(
        self,
        src_size_hint: u64,
        dict_size: usize,
        mode: CParamMode,
    ) -> Self {
        self.adjust(src_size_hint, dict_size, mode)
    }

    fn adjust(mut self, mut src_size: u64, mut dict_size: usize, mode: CParamMode) -> Self {
        match mode {
            CParamMode::Unknown | CParamMode::NoAttachDict => {}
            CParamMode::CreateCDict => {
                if dict_size > 0 && src_size == ZSTD_CONTENTSIZE_UNKNOWN {
                    src_size = 513;
                }
            }
            CParamMode::AttachDict => {
                dict_size = 0;
            }
        }

        let dict_size = dict_size as u64;
        let max_window_resize = 1_u64 << (ZSTD_WINDOWLOG_MAX - 1);

        if src_size <= max_window_resize && dict_size <= max_window_resize {
            let total_size = src_size + dict_size;
            let src_log = if total_size < (1 << ZSTD_HASHLOG_MIN) {
                ZSTD_HASHLOG_MIN
            } else {
                highbit32((total_size - 1) as u32) + 1
            };
            self.window_log = self.window_log.min(src_log);
        }

        if src_size != ZSTD_CONTENTSIZE_UNKNOWN {
            let dict_and_window_log = dict_and_window_log(self.window_log, src_size, dict_size);
            let cycle_log = cycle_log(self.chain_log, self.strategy);

            self.hash_log = self.hash_log.min(dict_and_window_log + 1);
            if cycle_log > dict_and_window_log {
                self.chain_log -= cycle_log - dict_and_window_log;
            }
        }

        self.window_log = self.window_log.max(ZSTD_WINDOWLOG_ABSOLUTE_MIN);

        if mode == CParamMode::CreateCDict && cdict_indices_are_tagged(self.strategy) {
            let max_short_cache_hash_log = 32 - ZSTD_SHORT_CACHE_TAG_BITS;
            self.hash_log = self.hash_log.min(max_short_cache_hash_log);
            self.chain_log = self.chain_log.min(max_short_cache_hash_log);
        }

        if row_match_finder_used(self.strategy) {
            let row_log = self.search_log.clamp(4, 6);
            let max_row_hash_log = 32 - ZSTD_ROW_HASH_TAG_BITS;
            let max_hash_log = max_row_hash_log + row_log;
            self.hash_log = self.hash_log.min(max_hash_log);
        }

        self
    }
}

fn c_param_row_size(src_size_hint: u64, mut dict_size: usize, mode: CParamMode) -> u64 {
    if mode == CParamMode::AttachDict {
        dict_size = 0;
    }

    let unknown = src_size_hint == ZSTD_CONTENTSIZE_UNKNOWN;
    let added_size = if unknown && dict_size > 0 { 500 } else { 0 };
    if unknown && dict_size == 0 {
        ZSTD_CONTENTSIZE_UNKNOWN
    } else {
        src_size_hint
            .wrapping_add(dict_size as u64)
            .wrapping_add(added_size)
    }
}

fn cycle_log(hash_log: u32, strategy: Strategy) -> u32 {
    hash_log - u32::from(strategy >= Strategy::BtLazy2)
}

fn dict_and_window_log(window_log: u32, src_size: u64, dict_size: u64) -> u32 {
    if dict_size == 0 {
        return window_log;
    }

    let max_window_size = 1_u64 << ZSTD_WINDOWLOG_MAX;
    let window_size = 1_u64 << window_log;
    let dict_and_window_size = dict_size + window_size;

    if window_size >= dict_size + src_size {
        window_log
    } else if dict_and_window_size >= max_window_size {
        ZSTD_WINDOWLOG_MAX
    } else {
        highbit32((dict_and_window_size - 1) as u32) + 1
    }
}

fn highbit32(value: u32) -> u32 {
    u32::BITS - 1 - value.leading_zeros()
}

fn row_match_finder_used(strategy: Strategy) -> bool {
    (Strategy::Greedy..=Strategy::Lazy2).contains(&strategy)
}

fn cdict_indices_are_tagged(strategy: Strategy) -> bool {
    matches!(strategy, Strategy::Fast | Strategy::DFast)
}

/// Default `ZSTD_shouldAttachDict()` decision, excluding force modes and
/// dedicated dictionary search which are not currently exposed by this port.
pub(crate) fn should_attach_dict_by_default(
    cdict_strategy: Strategy,
    pledged_src_size: u64,
) -> bool {
    pledged_src_size == ZSTD_CONTENTSIZE_UNKNOWN
        || pledged_src_size <= attach_dict_size_cutoff(cdict_strategy) as u64
}

fn attach_dict_size_cutoff(strategy: Strategy) -> usize {
    match strategy {
        Strategy::Fast => 8 * KIB_USIZE,
        Strategy::DFast => 16 * KIB_USIZE,
        Strategy::Greedy
        | Strategy::Lazy
        | Strategy::Lazy2
        | Strategy::BtLazy2
        | Strategy::BtOpt => 32 * KIB_USIZE,
        Strategy::BtUltra | Strategy::BtUltra2 => 8 * KIB_USIZE,
    }
}

const fn p(
    window_log: u32,
    chain_log: u32,
    hash_log: u32,
    search_log: u32,
    min_match: u32,
    target_length: u32,
    strategy: Strategy,
) -> CompressionParameters {
    CompressionParameters::new(
        window_log,
        chain_log,
        hash_log,
        search_log,
        min_match,
        target_length,
        strategy,
    )
}

const DEFAULT_C_PARAMETERS: [[CompressionParameters; 23]; 4] = [
    [
        p(19, 12, 13, 1, 6, 1, Strategy::Fast),
        p(19, 13, 14, 1, 7, 0, Strategy::Fast),
        p(20, 15, 16, 1, 6, 0, Strategy::Fast),
        p(21, 16, 17, 1, 5, 0, Strategy::DFast),
        p(21, 18, 18, 1, 5, 0, Strategy::DFast),
        p(21, 18, 19, 3, 5, 2, Strategy::Greedy),
        p(21, 18, 19, 3, 5, 4, Strategy::Lazy),
        p(21, 19, 20, 4, 5, 8, Strategy::Lazy),
        p(21, 19, 20, 4, 5, 16, Strategy::Lazy2),
        p(22, 20, 21, 4, 5, 16, Strategy::Lazy2),
        p(22, 21, 22, 5, 5, 16, Strategy::Lazy2),
        p(22, 21, 22, 6, 5, 16, Strategy::Lazy2),
        p(22, 22, 23, 6, 5, 32, Strategy::Lazy2),
        p(22, 22, 22, 4, 5, 32, Strategy::BtLazy2),
        p(22, 22, 23, 5, 5, 32, Strategy::BtLazy2),
        p(22, 23, 23, 6, 5, 32, Strategy::BtLazy2),
        p(22, 22, 22, 5, 5, 48, Strategy::BtOpt),
        p(23, 23, 22, 5, 4, 64, Strategy::BtOpt),
        p(23, 23, 22, 6, 3, 64, Strategy::BtUltra),
        p(23, 24, 22, 7, 3, 256, Strategy::BtUltra2),
        p(25, 25, 23, 7, 3, 256, Strategy::BtUltra2),
        p(26, 26, 24, 7, 3, 512, Strategy::BtUltra2),
        p(27, 27, 25, 9, 3, 999, Strategy::BtUltra2),
    ],
    [
        p(18, 12, 13, 1, 5, 1, Strategy::Fast),
        p(18, 13, 14, 1, 6, 0, Strategy::Fast),
        p(18, 14, 14, 1, 5, 0, Strategy::DFast),
        p(18, 16, 16, 1, 4, 0, Strategy::DFast),
        p(18, 16, 17, 3, 5, 2, Strategy::Greedy),
        p(18, 17, 18, 5, 5, 2, Strategy::Greedy),
        p(18, 18, 19, 3, 5, 4, Strategy::Lazy),
        p(18, 18, 19, 4, 4, 4, Strategy::Lazy),
        p(18, 18, 19, 4, 4, 8, Strategy::Lazy2),
        p(18, 18, 19, 5, 4, 8, Strategy::Lazy2),
        p(18, 18, 19, 6, 4, 8, Strategy::Lazy2),
        p(18, 18, 19, 5, 4, 12, Strategy::BtLazy2),
        p(18, 19, 19, 7, 4, 12, Strategy::BtLazy2),
        p(18, 18, 19, 4, 4, 16, Strategy::BtOpt),
        p(18, 18, 19, 4, 3, 32, Strategy::BtOpt),
        p(18, 18, 19, 6, 3, 128, Strategy::BtOpt),
        p(18, 19, 19, 6, 3, 128, Strategy::BtUltra),
        p(18, 19, 19, 8, 3, 256, Strategy::BtUltra),
        p(18, 19, 19, 6, 3, 128, Strategy::BtUltra2),
        p(18, 19, 19, 8, 3, 256, Strategy::BtUltra2),
        p(18, 19, 19, 10, 3, 512, Strategy::BtUltra2),
        p(18, 19, 19, 12, 3, 512, Strategy::BtUltra2),
        p(18, 19, 19, 13, 3, 999, Strategy::BtUltra2),
    ],
    [
        p(17, 12, 12, 1, 5, 1, Strategy::Fast),
        p(17, 12, 13, 1, 6, 0, Strategy::Fast),
        p(17, 13, 15, 1, 5, 0, Strategy::Fast),
        p(17, 15, 16, 2, 5, 0, Strategy::DFast),
        p(17, 17, 17, 2, 4, 0, Strategy::DFast),
        p(17, 16, 17, 3, 4, 2, Strategy::Greedy),
        p(17, 16, 17, 3, 4, 4, Strategy::Lazy),
        p(17, 16, 17, 3, 4, 8, Strategy::Lazy2),
        p(17, 16, 17, 4, 4, 8, Strategy::Lazy2),
        p(17, 16, 17, 5, 4, 8, Strategy::Lazy2),
        p(17, 16, 17, 6, 4, 8, Strategy::Lazy2),
        p(17, 17, 17, 5, 4, 8, Strategy::BtLazy2),
        p(17, 18, 17, 7, 4, 12, Strategy::BtLazy2),
        p(17, 18, 17, 3, 4, 12, Strategy::BtOpt),
        p(17, 18, 17, 4, 3, 32, Strategy::BtOpt),
        p(17, 18, 17, 6, 3, 256, Strategy::BtOpt),
        p(17, 18, 17, 6, 3, 128, Strategy::BtUltra),
        p(17, 18, 17, 8, 3, 256, Strategy::BtUltra),
        p(17, 18, 17, 10, 3, 512, Strategy::BtUltra),
        p(17, 18, 17, 5, 3, 256, Strategy::BtUltra2),
        p(17, 18, 17, 7, 3, 512, Strategy::BtUltra2),
        p(17, 18, 17, 9, 3, 512, Strategy::BtUltra2),
        p(17, 18, 17, 11, 3, 999, Strategy::BtUltra2),
    ],
    [
        p(14, 12, 13, 1, 5, 1, Strategy::Fast),
        p(14, 14, 15, 1, 5, 0, Strategy::Fast),
        p(14, 14, 15, 1, 4, 0, Strategy::Fast),
        p(14, 14, 15, 2, 4, 0, Strategy::DFast),
        p(14, 14, 14, 4, 4, 2, Strategy::Greedy),
        p(14, 14, 14, 3, 4, 4, Strategy::Lazy),
        p(14, 14, 14, 4, 4, 8, Strategy::Lazy2),
        p(14, 14, 14, 6, 4, 8, Strategy::Lazy2),
        p(14, 14, 14, 8, 4, 8, Strategy::Lazy2),
        p(14, 15, 14, 5, 4, 8, Strategy::BtLazy2),
        p(14, 15, 14, 9, 4, 8, Strategy::BtLazy2),
        p(14, 15, 14, 3, 4, 12, Strategy::BtOpt),
        p(14, 15, 14, 4, 3, 24, Strategy::BtOpt),
        p(14, 15, 14, 5, 3, 32, Strategy::BtUltra),
        p(14, 15, 15, 6, 3, 64, Strategy::BtUltra),
        p(14, 15, 15, 7, 3, 256, Strategy::BtUltra),
        p(14, 15, 15, 5, 3, 48, Strategy::BtUltra2),
        p(14, 15, 15, 6, 3, 128, Strategy::BtUltra2),
        p(14, 15, 15, 7, 3, 256, Strategy::BtUltra2),
        p(14, 15, 15, 8, 3, 256, Strategy::BtUltra2),
        p(14, 15, 15, 8, 3, 512, Strategy::BtUltra2),
        p(14, 15, 15, 9, 3, 512, Strategy::BtUltra2),
        p(14, 15, 15, 10, 3, 999, Strategy::BtUltra2),
    ],
];
