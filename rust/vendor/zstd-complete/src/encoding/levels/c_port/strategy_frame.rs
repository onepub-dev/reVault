//! Strategy-dispatching frame adapter for the C frame paths.

#[cfg(feature = "std")]
use crate::encoding::CompressionTuning;
use alloc::vec::Vec;

use super::{
    cctx_params::CctxParameters,
    dfast_frame::{
        encode_frame_double_fast_no_dict, encode_frame_double_fast_no_dict_with_cctx,
        encode_frame_double_fast_with_dictionary,
        encode_frame_double_fast_with_dictionary_and_cctx,
    },
    dictionary::{
        parse_dictionary, DictionaryContentType, DictionaryParseError, ParsedDictionary,
        PreparedDictionary,
    },
    fast_frame::{
        encode_frame_fast_no_dict, encode_frame_fast_no_dict_with_cctx,
        encode_frame_fast_with_dictionary, encode_frame_fast_with_dictionary_and_cctx,
    },
    greedy_block::LazyBlockStrategy,
    greedy_frame::{
        encode_frame_btlazy2_no_dict, encode_frame_btlazy2_with_dictionary,
        encode_frame_greedy_no_dict, encode_frame_greedy_with_dictionary,
        encode_frame_hash_chain_no_dict_with_cctx,
        encode_frame_hash_chain_with_dictionary_and_cctx,
        encode_frame_hash_chain_with_prepared_dictionary_and_cctx, encode_frame_lazy2_no_dict,
        encode_frame_lazy2_with_dictionary, encode_frame_lazy_no_dict,
        encode_frame_lazy_with_dictionary,
    },
    opt_frame::{
        encode_frame_btopt_with_dictionary, encode_frame_btopt_with_dictionary_and_cctx,
        encode_frame_btultra2_with_dictionary, encode_frame_btultra2_with_dictionary_and_cctx,
        encode_frame_btultra_with_dictionary, encode_frame_btultra_with_dictionary_and_cctx,
        encode_frame_opt_no_dict_with_cctx, encode_frame_opt_with_prepared_dictionary_and_cctx,
    },
    params::{CParamMode, CompressionParameters, Strategy, ZSTD_CONTENTSIZE_UNKNOWN},
};

const USE_CDICT_PARAMS_SOURCE_SIZE_CUTOFF: usize = 128 * 1024;
const USE_CDICT_PARAMS_DICTIONARY_SIZE_MULTIPLIER: usize = 6;
const CDICT_SOURCE_WINDOW_LOG_LIMIT: u32 = 19;

pub(crate) fn strategy_for_level(level: i32, src_size: usize) -> Strategy {
    let cctx = CctxParameters::for_level(level, src_size as u64, 0);
    cctx.assert_resolved();
    cctx.compression.strategy
}

pub(crate) fn strategy_for_level_with_dictionary(
    level: i32,
    src_size: usize,
    dict_size: usize,
) -> Strategy {
    let cctx = CctxParameters::for_level_with_mode(
        level,
        src_size as u64,
        dict_size,
        CParamMode::NoAttachDict,
    );
    cctx.assert_resolved();
    cctx.compression.strategy
}

pub(crate) fn encode_frame_no_dict(src: &[u8], level: i32) -> Vec<u8> {
    let cctx = CctxParameters::for_level(level, src.len() as u64, 0);
    cctx.assert_resolved();
    match cctx.compression.strategy {
        Strategy::Fast => encode_frame_fast_no_dict(src, level),
        Strategy::DFast => encode_frame_double_fast_no_dict(src, level),
        Strategy::Greedy => encode_frame_greedy_no_dict(src, level),
        Strategy::Lazy => encode_frame_lazy_no_dict(src, level),
        Strategy::Lazy2 => encode_frame_lazy2_no_dict(src, level),
        Strategy::BtLazy2 => encode_frame_btlazy2_no_dict(src, level),
        Strategy::BtOpt | Strategy::BtUltra | Strategy::BtUltra2 => {
            encode_frame_opt_no_dict_with_cctx(src, cctx)
        }
    }
}

#[cfg(feature = "std")]
pub(crate) fn encode_frame_no_dict_with_tuning(
    src: &[u8],
    level: i32,
    tuning: CompressionTuning,
) -> Vec<u8> {
    let cctx = CctxParameters::for_level(level, src.len() as u64, 0).apply_tuning(
        level,
        src.len() as u64,
        tuning,
    );
    encode_frame_no_dict_from_cctx(src, cctx)
}

#[cfg(feature = "std")]
fn encode_frame_no_dict_from_cctx(src: &[u8], cctx: CctxParameters) -> Vec<u8> {
    cctx.assert_resolved();
    match cctx.compression.strategy {
        Strategy::Fast => encode_frame_fast_no_dict_with_cctx(src, cctx),
        Strategy::DFast => encode_frame_double_fast_no_dict_with_cctx(src, cctx),
        Strategy::Greedy => {
            encode_frame_hash_chain_no_dict_with_cctx(src, cctx, LazyBlockStrategy::Greedy)
        }
        Strategy::Lazy => {
            encode_frame_hash_chain_no_dict_with_cctx(src, cctx, LazyBlockStrategy::Lazy)
        }
        Strategy::Lazy2 => {
            encode_frame_hash_chain_no_dict_with_cctx(src, cctx, LazyBlockStrategy::Lazy2)
        }
        Strategy::BtLazy2 => {
            encode_frame_hash_chain_no_dict_with_cctx(src, cctx, LazyBlockStrategy::BtLazy2)
        }
        Strategy::BtOpt | Strategy::BtUltra | Strategy::BtUltra2 => {
            encode_frame_opt_no_dict_with_cctx(src, cctx)
        }
    }
}

pub(crate) fn encode_frame_no_dict_with_target_c_block_size(
    src: &[u8],
    level: i32,
    target_c_block_size: usize,
) -> Option<Vec<u8>> {
    let mut cctx = CctxParameters::for_level(level, src.len() as u64, 0);
    if !cctx.set_target_c_block_size(target_c_block_size) {
        return None;
    }
    cctx.assert_resolved();

    match cctx.compression.strategy {
        Strategy::Fast => Some(encode_frame_fast_no_dict_with_cctx(src, cctx)),
        Strategy::DFast => Some(encode_frame_double_fast_no_dict_with_cctx(src, cctx)),
        Strategy::Greedy => Some(encode_frame_hash_chain_no_dict_with_cctx(
            src,
            cctx,
            LazyBlockStrategy::Greedy,
        )),
        Strategy::Lazy => Some(encode_frame_hash_chain_no_dict_with_cctx(
            src,
            cctx,
            LazyBlockStrategy::Lazy,
        )),
        Strategy::Lazy2 => Some(encode_frame_hash_chain_no_dict_with_cctx(
            src,
            cctx,
            LazyBlockStrategy::Lazy2,
        )),
        Strategy::BtLazy2 => Some(encode_frame_hash_chain_no_dict_with_cctx(
            src,
            cctx,
            LazyBlockStrategy::BtLazy2,
        )),
        Strategy::BtOpt | Strategy::BtUltra | Strategy::BtUltra2 => {
            Some(encode_frame_opt_no_dict_with_cctx(src, cctx))
        }
    }
}

pub(crate) fn encode_frame_with_dictionary(
    src: &[u8],
    level: i32,
    dictionary: &[u8],
) -> Result<Vec<u8>, DictionaryParseError> {
    let Some(parsed) = parse_dictionary(dictionary, DictionaryContentType::Auto, false)? else {
        return Ok(encode_frame_no_dict(src, level));
    };

    Ok(encode_frame_with_parsed_dictionary(src, level, parsed))
}

pub(crate) fn encode_frame_with_dictionary_and_target_c_block_size(
    src: &[u8],
    level: i32,
    dictionary: &[u8],
    target_c_block_size: usize,
) -> Result<Option<Vec<u8>>, DictionaryParseError> {
    let Some(parsed) = parse_dictionary(dictionary, DictionaryContentType::Auto, false)? else {
        return Ok(encode_frame_no_dict_with_target_c_block_size(
            src,
            level,
            target_c_block_size,
        ));
    };

    let mut cctx = CctxParameters::for_level_with_mode(
        level,
        src.len() as u64,
        parsed.content.len(),
        CParamMode::NoAttachDict,
    );
    if !cctx.set_target_c_block_size(target_c_block_size) {
        return Ok(None);
    }
    cctx.assert_resolved();

    Ok(encode_frame_with_parsed_dictionary_and_cctx(
        src, level, parsed, cctx,
    ))
}

pub(crate) fn encode_frame_with_prepared_dictionary(
    src: &[u8],
    level: i32,
    dictionary: &PreparedDictionary,
) -> Vec<u8> {
    let cctx = prepared_dictionary_cctx(src.len(), level, dictionary.raw_size());
    let parsed = dictionary.as_parsed();

    match cctx.compression.strategy {
        Strategy::Fast => {
            encode_frame_fast_with_dictionary_and_cctx(src, level, parsed, cctx, true)
        }
        Strategy::DFast => {
            encode_frame_double_fast_with_dictionary_and_cctx(src, level, parsed, cctx, true)
        }
        Strategy::Greedy => encode_frame_hash_chain_with_prepared_dictionary_and_cctx(
            src,
            level,
            parsed,
            cctx,
            LazyBlockStrategy::Greedy,
        ),
        Strategy::Lazy => encode_frame_hash_chain_with_prepared_dictionary_and_cctx(
            src,
            level,
            parsed,
            cctx,
            LazyBlockStrategy::Lazy,
        ),
        Strategy::Lazy2 => encode_frame_hash_chain_with_prepared_dictionary_and_cctx(
            src,
            level,
            parsed,
            cctx,
            LazyBlockStrategy::Lazy2,
        ),
        Strategy::BtLazy2 => encode_frame_hash_chain_with_prepared_dictionary_and_cctx(
            src,
            level,
            parsed,
            cctx,
            LazyBlockStrategy::BtLazy2,
        ),
        Strategy::BtOpt | Strategy::BtUltra | Strategy::BtUltra2 => {
            encode_frame_opt_with_prepared_dictionary_and_cctx(src, level, parsed, cctx)
        }
    }
}

#[cfg(feature = "std")]
pub(crate) fn encode_frame_with_prepared_dictionary_and_tuning(
    src: &[u8],
    level: i32,
    dictionary: &PreparedDictionary,
    tuning: CompressionTuning,
) -> Vec<u8> {
    let cctx = prepared_dictionary_cctx(src.len(), level, dictionary.raw_size()).apply_tuning(
        level,
        src.len() as u64,
        tuning,
    );
    let parsed = dictionary.as_parsed();

    match cctx.compression.strategy {
        Strategy::Fast => {
            encode_frame_fast_with_dictionary_and_cctx(src, level, parsed, cctx, true)
        }
        Strategy::DFast => {
            encode_frame_double_fast_with_dictionary_and_cctx(src, level, parsed, cctx, true)
        }
        Strategy::Greedy => encode_frame_hash_chain_with_prepared_dictionary_and_cctx(
            src,
            level,
            parsed,
            cctx,
            LazyBlockStrategy::Greedy,
        ),
        Strategy::Lazy => encode_frame_hash_chain_with_prepared_dictionary_and_cctx(
            src,
            level,
            parsed,
            cctx,
            LazyBlockStrategy::Lazy,
        ),
        Strategy::Lazy2 => encode_frame_hash_chain_with_prepared_dictionary_and_cctx(
            src,
            level,
            parsed,
            cctx,
            LazyBlockStrategy::Lazy2,
        ),
        Strategy::BtLazy2 => encode_frame_hash_chain_with_prepared_dictionary_and_cctx(
            src,
            level,
            parsed,
            cctx,
            LazyBlockStrategy::BtLazy2,
        ),
        Strategy::BtOpt | Strategy::BtUltra | Strategy::BtUltra2 => {
            encode_frame_opt_with_prepared_dictionary_and_cctx(src, level, parsed, cctx)
        }
    }
}

fn prepared_dictionary_cctx(src_size: usize, level: i32, dictionary_size: usize) -> CctxParameters {
    let dictionary_params = CompressionParameters::for_level_with_mode(
        level,
        ZSTD_CONTENTSIZE_UNKNOWN,
        dictionary_size,
        CParamMode::CreateCDict,
    );
    let use_dictionary_params = src_size < USE_CDICT_PARAMS_SOURCE_SIZE_CUTOFF
        || src_size < dictionary_size.saturating_mul(USE_CDICT_PARAMS_DICTIONARY_SIZE_MULTIPLIER);
    let requested_params = if use_dictionary_params {
        dictionary_params
    } else {
        CompressionParameters::for_level_with_mode(
            level,
            src_size as u64,
            dictionary_size,
            CParamMode::NoAttachDict,
        )
    };

    let limited_src_size = src_size.min(1 << CDICT_SOURCE_WINDOW_LOG_LIMIT);
    let source_window_log = if limited_src_size > 1 {
        usize::BITS - (limited_src_size - 1).leading_zeros()
    } else {
        1
    };
    let mut active_params = dictionary_params;
    active_params.window_log = requested_params.window_log.max(source_window_log);
    CctxParameters::from_compression_parameters(level, active_params, src_size as u64)
}

fn encode_frame_with_parsed_dictionary(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
) -> Vec<u8> {
    match strategy_for_level_with_dictionary(level, src.len(), dictionary.content.len()) {
        Strategy::Fast => encode_frame_fast_with_dictionary(src, level, dictionary),
        Strategy::DFast => encode_frame_double_fast_with_dictionary(src, level, dictionary),
        Strategy::Greedy => encode_frame_greedy_with_dictionary(src, level, dictionary),
        Strategy::Lazy => encode_frame_lazy_with_dictionary(src, level, dictionary),
        Strategy::Lazy2 => encode_frame_lazy2_with_dictionary(src, level, dictionary),
        Strategy::BtLazy2 => encode_frame_btlazy2_with_dictionary(src, level, dictionary),
        Strategy::BtOpt => encode_frame_btopt_with_dictionary(src, level, dictionary),
        Strategy::BtUltra => encode_frame_btultra_with_dictionary(src, level, dictionary),
        Strategy::BtUltra2 => encode_frame_btultra2_with_dictionary(src, level, dictionary),
    }
}

fn encode_frame_with_parsed_dictionary_and_cctx(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
    cctx: CctxParameters,
) -> Option<Vec<u8>> {
    match cctx.compression.strategy {
        Strategy::Fast => Some(encode_frame_fast_with_dictionary_and_cctx(
            src, level, dictionary, cctx, false,
        )),
        Strategy::DFast => Some(encode_frame_double_fast_with_dictionary_and_cctx(
            src, level, dictionary, cctx, false,
        )),
        Strategy::Greedy => Some(encode_frame_hash_chain_with_dictionary_and_cctx(
            src,
            level,
            dictionary,
            cctx,
            LazyBlockStrategy::Greedy,
        )),
        Strategy::Lazy => Some(encode_frame_hash_chain_with_dictionary_and_cctx(
            src,
            level,
            dictionary,
            cctx,
            LazyBlockStrategy::Lazy,
        )),
        Strategy::Lazy2 => Some(encode_frame_hash_chain_with_dictionary_and_cctx(
            src,
            level,
            dictionary,
            cctx,
            LazyBlockStrategy::Lazy2,
        )),
        Strategy::BtLazy2 => Some(encode_frame_hash_chain_with_dictionary_and_cctx(
            src,
            level,
            dictionary,
            cctx,
            LazyBlockStrategy::BtLazy2,
        )),
        Strategy::BtOpt => Some(encode_frame_btopt_with_dictionary_and_cctx(
            src, level, dictionary, cctx,
        )),
        Strategy::BtUltra => Some(encode_frame_btultra_with_dictionary_and_cctx(
            src, level, dictionary, cctx,
        )),
        Strategy::BtUltra2 => Some(encode_frame_btultra2_with_dictionary_and_cctx(
            src, level, dictionary, cctx,
        )),
    }
}
