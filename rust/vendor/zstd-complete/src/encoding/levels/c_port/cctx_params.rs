//! Resolved C compression context parameters.
//!
//! This mirrors the C layer that turns `ZSTD_compressionParameters` plus
//! auto-mode switches into finalized `ZSTD_CCtx_params` behavior.

use super::params::{CParamMode, CompressionParameters, Strategy};
#[cfg(feature = "std")]
use crate::encoding::{CompressionStrategy, CompressionTuning};

const ZSTD_BLOCKSIZE_MAX: usize = 128 * 1024;
const ZSTD_TARGETCBLOCKSIZE_MIN: usize = 1340;
const ZSTD_TARGETCBLOCKSIZE_MAX: usize = ZSTD_BLOCKSIZE_MAX;
const ZSTD_HASHLOG_MIN: u32 = 6;
const ZSTD_HASHLOG_MAX: u32 = 30;
const ZSTD_LDM_BUCKETSIZELOG_MAX: u32 = 8;
const LDM_BUCKET_SIZE_LOG: u32 = 4;
const LDM_MIN_MATCH_LENGTH: u32 = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParamSwitch {
    Auto,
    Enable,
    Disable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LdmParameters {
    pub(crate) enable_ldm: ParamSwitch,
    pub(crate) window_log: u32,
    pub(crate) hash_log: u32,
    pub(crate) min_match_length: u32,
    pub(crate) bucket_size_log: u32,
    pub(crate) hash_rate_log: u32,
}

impl Default for LdmParameters {
    fn default() -> Self {
        Self {
            enable_ldm: ParamSwitch::Auto,
            window_log: 0,
            hash_log: 0,
            min_match_length: 0,
            bucket_size_log: 0,
            hash_rate_log: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CctxParameters {
    pub(crate) compression: CompressionParameters,
    pub(crate) use_row_match_finder: ParamSwitch,
    pub(crate) post_block_splitter: ParamSwitch,
    pub(crate) ldm: LdmParameters,
    pub(crate) max_block_size: usize,
    pub(crate) target_c_block_size: usize,
    pub(crate) search_for_external_repcodes: ParamSwitch,
}

impl CctxParameters {
    pub(crate) fn for_level(level: i32, src_size_hint: u64, dict_size: usize) -> Self {
        let compression = CompressionParameters::for_level(level, src_size_hint, dict_size);
        Self::from_compression_parameters(level, compression, src_size_hint)
    }

    pub(crate) fn for_level_with_mode(
        level: i32,
        src_size_hint: u64,
        dict_size: usize,
        mode: CParamMode,
    ) -> Self {
        let compression =
            CompressionParameters::for_level_with_mode(level, src_size_hint, dict_size, mode);
        Self::from_compression_parameters(level, compression, src_size_hint)
    }

    pub(crate) fn from_compression_parameters(
        level: i32,
        compression: CompressionParameters,
        pledged_src_size: u64,
    ) -> Self {
        let mut ldm = LdmParameters {
            enable_ldm: resolve_enable_ldm(ParamSwitch::Auto, compression),
            ..LdmParameters::default()
        };
        if ldm.enable_ldm == ParamSwitch::Enable {
            adjust_ldm_parameters(&mut ldm, compression);
        }

        Self {
            compression,
            use_row_match_finder: resolve_row_match_finder(ParamSwitch::Auto, compression),
            post_block_splitter: resolve_block_splitter(ParamSwitch::Auto, compression),
            ldm,
            max_block_size: resolve_max_block_size(compression, pledged_src_size),
            target_c_block_size: 0,
            search_for_external_repcodes: resolve_external_repcode_search(ParamSwitch::Auto, level),
        }
    }

    pub(crate) fn set_target_c_block_size(&mut self, value: usize) -> bool {
        let Some(target) = resolve_target_c_block_size(value) else {
            return false;
        };
        self.target_c_block_size = target;
        true
    }

    #[cfg(feature = "std")]
    pub(crate) fn apply_tuning(
        mut self,
        level: i32,
        pledged_src_size: u64,
        tuning: CompressionTuning,
    ) -> Self {
        let compression = &mut self.compression;
        if let Some(value) = tuning.window_log {
            compression.window_log = u32::from(value);
        }
        if let Some(value) = tuning.hash_log {
            compression.hash_log = u32::from(value);
        }
        if let Some(value) = tuning.chain_log {
            compression.chain_log = u32::from(value);
        }
        if let Some(value) = tuning.search_log {
            compression.search_log = u32::from(value);
        }
        if let Some(value) = tuning.min_match {
            compression.min_match = u32::from(value);
        }
        if let Some(value) = tuning.target_length {
            compression.target_length = value;
        }
        if let Some(value) = tuning.strategy {
            compression.strategy = strategy_from_public(value);
        }

        let mut tuned = Self::from_compression_parameters(level, *compression, pledged_src_size);
        if let Some(value) = tuning.target_compressed_block_size {
            let accepted = tuned.set_target_c_block_size(value);
            debug_assert!(accepted, "public tuning is validated before dispatch");
        }
        if let Some(ldm) = tuning.long_distance_matching {
            tuned.ldm.enable_ldm = ParamSwitch::Enable;
            tuned.ldm.window_log = ldm.window_log.map_or(0, u32::from);
            tuned.ldm.hash_log = ldm.hash_log.map_or(0, u32::from);
            tuned.ldm.min_match_length = ldm.min_match.map_or(0, u32::from);
            tuned.ldm.bucket_size_log = ldm.bucket_size_log.map_or(0, u32::from);
            tuned.ldm.hash_rate_log = ldm.hash_rate_log.map_or(0, u32::from);
            adjust_ldm_parameters(&mut tuned.ldm, tuned.compression);
        }
        tuned
    }

    pub(crate) fn use_target_c_block_size(&self) -> bool {
        self.target_c_block_size != 0
    }

    pub(crate) fn assert_resolved(&self) {
        debug_assert_ne!(self.use_row_match_finder, ParamSwitch::Auto);
        debug_assert_ne!(self.post_block_splitter, ParamSwitch::Auto);
        debug_assert_ne!(self.ldm.enable_ldm, ParamSwitch::Auto);
        debug_assert_ne!(self.search_for_external_repcodes, ParamSwitch::Auto);
        debug_assert!((1..=ZSTD_BLOCKSIZE_MAX).contains(&self.max_block_size));
        debug_assert!(
            self.target_c_block_size == 0
                || (ZSTD_TARGETCBLOCKSIZE_MIN..=ZSTD_TARGETCBLOCKSIZE_MAX)
                    .contains(&self.target_c_block_size)
        );
        if self.ldm.enable_ldm == ParamSwitch::Enable {
            debug_assert!(self.ldm.window_log > 0);
            debug_assert!(self.ldm.hash_log > 0);
            debug_assert!(self.ldm.min_match_length > 0);
            debug_assert!(self.ldm.bucket_size_log > 0);
            let gear = super::ldm::LdmRollingHashState::new(self.ldm);
            debug_assert_eq!(gear.rolling(), u32::MAX as u64);
            debug_assert!(gear.stop_mask() > 0 || self.ldm.hash_rate_log == 0);
        }
    }
}

#[cfg(feature = "std")]
const fn strategy_from_public(strategy: CompressionStrategy) -> Strategy {
    match strategy {
        CompressionStrategy::Fast => Strategy::Fast,
        CompressionStrategy::DoubleFast => Strategy::DFast,
        CompressionStrategy::Greedy => Strategy::Greedy,
        CompressionStrategy::Lazy => Strategy::Lazy,
        CompressionStrategy::Lazy2 => Strategy::Lazy2,
        CompressionStrategy::BinaryTreeLazy2 => Strategy::BtLazy2,
        CompressionStrategy::Optimal => Strategy::BtOpt,
        CompressionStrategy::Ultra => Strategy::BtUltra,
        CompressionStrategy::Ultra2 => Strategy::BtUltra2,
    }
}

fn resolve_target_c_block_size(value: usize) -> Option<usize> {
    if value == 0 {
        return Some(0);
    }
    let value = value.max(ZSTD_TARGETCBLOCKSIZE_MIN);
    (value <= ZSTD_TARGETCBLOCKSIZE_MAX).then_some(value)
}

fn resolve_max_block_size(params: CompressionParameters, pledged_src_size: u64) -> usize {
    let window_size = (1_u64 << params.window_log).min(pledged_src_size).max(1);
    window_size.min(ZSTD_BLOCKSIZE_MAX as u64) as usize
}

fn resolve_row_match_finder(mode: ParamSwitch, params: CompressionParameters) -> ParamSwitch {
    if mode != ParamSwitch::Auto {
        return mode;
    }
    if row_match_finder_supported(params.strategy) && params.window_log > 14 {
        ParamSwitch::Enable
    } else {
        ParamSwitch::Disable
    }
}

fn resolve_block_splitter(mode: ParamSwitch, params: CompressionParameters) -> ParamSwitch {
    if mode != ParamSwitch::Auto {
        return mode;
    }
    if params.strategy >= Strategy::BtOpt && params.window_log >= 17 {
        ParamSwitch::Enable
    } else {
        ParamSwitch::Disable
    }
}

fn resolve_enable_ldm(mode: ParamSwitch, params: CompressionParameters) -> ParamSwitch {
    if mode != ParamSwitch::Auto {
        return mode;
    }
    if params.strategy >= Strategy::BtOpt && params.window_log >= 27 {
        ParamSwitch::Enable
    } else {
        ParamSwitch::Disable
    }
}

fn resolve_external_repcode_search(mode: ParamSwitch, level: i32) -> ParamSwitch {
    if mode != ParamSwitch::Auto {
        return mode;
    }
    if level < 10 {
        ParamSwitch::Disable
    } else {
        ParamSwitch::Enable
    }
}

fn adjust_ldm_parameters(ldm: &mut LdmParameters, params: CompressionParameters) {
    ldm.window_log = params.window_log;
    if ldm.hash_rate_log == 0 {
        if ldm.hash_log > 0 {
            if ldm.window_log > ldm.hash_log {
                ldm.hash_rate_log = ldm.window_log - ldm.hash_log;
            }
        } else {
            ldm.hash_rate_log = 7 - (params.strategy as u32 / 3);
        }
    }
    if ldm.hash_log == 0 {
        ldm.hash_log =
            (ldm.window_log - ldm.hash_rate_log).clamp(ZSTD_HASHLOG_MIN, ZSTD_HASHLOG_MAX);
    }
    if ldm.min_match_length == 0 {
        ldm.min_match_length = LDM_MIN_MATCH_LENGTH;
        if params.strategy >= Strategy::BtUltra {
            ldm.min_match_length /= 2;
        }
    }
    if ldm.bucket_size_log == 0 {
        ldm.bucket_size_log =
            (params.strategy as u32).clamp(LDM_BUCKET_SIZE_LOG, ZSTD_LDM_BUCKETSIZELOG_MAX);
    }
    ldm.bucket_size_log = ldm.bucket_size_log.min(ldm.hash_log);
}

fn row_match_finder_supported(strategy: Strategy) -> bool {
    (Strategy::Greedy..=Strategy::Lazy2).contains(&strategy)
}
