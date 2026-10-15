//! High-performance Rust compressor derived from the zstd 1.5.7 compressor.
//!
//! C remains the disclosed provenance and a useful compatibility/performance
//! oracle, not an ongoing structural or byte-parity contract. Prefer Rust-owned
//! invariants and measured improvements while preserving Zstandard format
//! interoperability.

#![deny(unsafe_op_in_unsafe_fn)]

mod block_compressor;
mod block_emit;
mod block_policy;
mod bt_match;
mod c_frame_header;
mod cctx_params;
mod compress_bound;
mod dfast;
mod dfast_block;
mod dfast_dict;
mod dfast_ext;
mod dfast_frame;
mod dfast_helpers;
mod dfast_table;
mod dictionary;
mod dictionary_frame;
mod fast;
mod fast_block;
mod fast_ext;
mod fast_frame;
mod fast_helpers;
mod frame_state;
mod greedy;
mod greedy_block;
mod greedy_bounds;
mod greedy_dict;
mod greedy_ext;
mod greedy_ext_block;
mod greedy_frame;
mod greedy_state;
mod hash_chain_match;
mod ldm;
mod match_count;
#[cfg(feature = "std")]
mod memory;
mod opt_block;
mod opt_dict;
mod opt_encode;
mod opt_frame;
mod opt_match;
mod opt_parser;
mod opt_path;
mod opt_price;
mod opt_state;
mod params;
mod post_split;
mod pre_split;
mod row_match;
mod row_table;
pub(crate) mod sequence_store;
mod strategy_frame;
mod superblock;
mod superblock_sequences;
mod target_acceptance;
mod target_block;
mod target_modes;
mod target_multi;
mod target_multi_basic;
mod target_single;
mod unaligned;
#[cfg(target_arch = "x86_64")]
mod x86;

pub(crate) use cctx_params::{
    CctxParameters as WorkspaceCctxParameters, ParamSwitch as WorkspaceParamSwitch,
};
pub(crate) use compress_bound::compress_bound as workspace_compress_bound;
pub(crate) use dfast::DFastMatchState as WorkspaceDFastMatchState;
pub(crate) use dfast_frame::encode_frame_double_fast_no_dict_with_cctx_in as workspace_encode_dfast;
pub(crate) use fast::FastMatchState as WorkspaceFastMatchState;
pub(crate) use fast_frame::encode_frame_fast_no_dict_with_cctx_in as workspace_encode_fast;
pub(crate) use frame_state::FrameBlockState as WorkspaceFrameBlockState;
pub(crate) use greedy::GreedyMatchState as WorkspaceGreedyMatchState;
pub(crate) use greedy_block::LazyBlockStrategy as WorkspaceLazyBlockStrategy;
pub(crate) use greedy_frame::encode_frame_hash_chain_no_dict_with_cctx_in as workspace_encode_greedy;
pub(crate) use ldm::LdmWorkspace as WorkspaceLdmWorkspace;
#[cfg(feature = "std")]
pub(crate) use memory::estimated_frame_memory;
pub(crate) use opt_frame::{
    encode_frame_opt_no_dict_with_cctx_in as workspace_encode_opt,
    OptFrameStrategy as WorkspaceOptFrameStrategy,
};
pub(crate) use opt_state::OptBlockState as WorkspaceOptBlockState;
pub(crate) use params::Strategy as WorkspaceStrategy;

#[cfg(any(feature = "std", feature = "c-port-validation", test))]
pub(crate) use strategy_frame::encode_frame_no_dict;
#[cfg(any(feature = "c-port-validation", test))]
pub(crate) use strategy_frame::encode_frame_no_dict_with_target_c_block_size;
#[cfg(any(feature = "c-port-validation", test))]
pub(crate) use strategy_frame::encode_frame_with_dictionary;
#[cfg(any(feature = "c-port-validation", test))]
pub(crate) use strategy_frame::encode_frame_with_dictionary_and_target_c_block_size;
#[cfg(any(feature = "std", feature = "c-port-validation", test))]
pub(crate) use strategy_frame::encode_frame_with_prepared_dictionary;
#[cfg(feature = "std")]
pub(crate) use strategy_frame::{
    encode_frame_no_dict_with_tuning, encode_frame_with_prepared_dictionary_and_tuning,
};

#[cfg(any(feature = "std", feature = "c-port-validation", test))]
pub(crate) use dictionary::{DictionaryParseError, PreparedDictionary};

#[cfg(test)]
mod cctx_params_tests;
#[cfg(test)]
mod dfast_ext_tests;
#[cfg(test)]
mod dfast_tests;
#[cfg(test)]
mod fast_ext_tests;
#[cfg(test)]
mod fast_tests;
#[cfg(test)]
mod greedy_ext_tests;
#[cfg(test)]
mod greedy_frame_tests;
#[cfg(test)]
mod greedy_tests;
#[cfg(test)]
mod ldm_tests;
#[cfg(test)]
mod ldm_window_tests;
#[cfg(test)]
mod opt_frame_tests;
#[cfg(test)]
mod opt_match_tests;
#[cfg(test)]
mod opt_parser_tests;
#[cfg(test)]
mod opt_price_tests;
#[cfg(test)]
mod params_tests;
#[cfg(test)]
mod sequence_store_tests;
#[cfg(test)]
mod strategy_frame_tests;
#[cfg(test)]
mod target_block_fixtures;
#[cfg(test)]
mod target_block_tests;
#[cfg(test)]
mod test_dictionary;

#[cfg(test)]
pub(crate) use params::{
    CompressionParameters, Strategy, MAX_COMPRESSION_LEVEL, MIN_COMPRESSION_LEVEL,
};
