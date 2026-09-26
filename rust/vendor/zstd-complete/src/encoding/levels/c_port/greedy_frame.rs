//! Frame-level adapter for the C greedy no-dictionary path.

use alloc::vec::Vec;

use super::{
    bt_match::load_attached_dictionary_binary_tree,
    c_frame_header::write_frame_header_no_dict,
    cctx_params::CctxParameters,
    compress_bound::compress_bound,
    dictionary::ParsedDictionary,
    dictionary_frame::DictionaryFrameContext,
    frame_state::{streaming_dict_limit, BlockEncodeMode, FrameBlockState},
    greedy::GreedyMatchState,
    greedy_block::{
        encode_block_hash_chain_no_dict,
        encode_block_hash_chain_no_dict_with_state_and_policy_in_mode,
        encode_block_hash_chain_no_dict_with_state_and_policy_in_mode_into,
        GreedyBlockEncodeContext, GreedyBlockSource, LazyBlockStrategy,
    },
    greedy_dict::{load_binary_tree_prefix, load_prefix},
    greedy_ext_block::{
        encode_block_attached_dict_with_state_and_policy_in_mode,
        encode_block_hash_chain_ext_dict_with_state_and_policy_in_mode,
        encode_block_hash_chain_ext_dict_with_state_and_policy_in_mode_into,
        GreedyAttachedDictBlockSource, GreedyExtDictBlockSource,
    },
    params::{should_attach_dict_by_default, CParamMode, CompressionParameters},
    row_match::{load_dictionary_rows_at_index_base, row_log, row_match_finder_enabled},
};
use crate::common::MAX_BLOCK_SIZE;

const C_WINDOW_START_INDEX: usize = 2;

pub(crate) fn encode_single_block_frame_greedy_no_dict(src: &[u8], level: i32) -> Vec<u8> {
    debug_assert!(src.len() <= MAX_BLOCK_SIZE as usize);
    encode_frame_greedy_no_dict(src, level)
}

pub(crate) fn encode_frame_greedy_no_dict(src: &[u8], level: i32) -> Vec<u8> {
    encode_frame_hash_chain_no_dict(src, level, LazyBlockStrategy::Greedy)
}

pub(crate) fn encode_single_block_frame_lazy_no_dict(src: &[u8], level: i32) -> Vec<u8> {
    debug_assert!(src.len() <= MAX_BLOCK_SIZE as usize);
    encode_frame_lazy_no_dict(src, level)
}

pub(crate) fn encode_frame_lazy_no_dict(src: &[u8], level: i32) -> Vec<u8> {
    encode_frame_hash_chain_no_dict(src, level, LazyBlockStrategy::Lazy)
}

pub(crate) fn encode_single_block_frame_lazy2_no_dict(src: &[u8], level: i32) -> Vec<u8> {
    debug_assert!(src.len() <= MAX_BLOCK_SIZE as usize);
    encode_frame_lazy2_no_dict(src, level)
}

pub(crate) fn encode_frame_lazy2_no_dict(src: &[u8], level: i32) -> Vec<u8> {
    encode_frame_hash_chain_no_dict(src, level, LazyBlockStrategy::Lazy2)
}

pub(crate) fn encode_frame_greedy_with_dictionary(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
) -> Vec<u8> {
    encode_frame_hash_chain_with_dictionary(src, level, dictionary, LazyBlockStrategy::Greedy)
}

pub(crate) fn encode_frame_lazy_with_dictionary(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
) -> Vec<u8> {
    encode_frame_hash_chain_with_dictionary(src, level, dictionary, LazyBlockStrategy::Lazy)
}

pub(crate) fn encode_frame_lazy2_with_dictionary(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
) -> Vec<u8> {
    encode_frame_hash_chain_with_dictionary(src, level, dictionary, LazyBlockStrategy::Lazy2)
}

pub(crate) fn encode_single_block_frame_btlazy2_no_dict(src: &[u8], level: i32) -> Vec<u8> {
    debug_assert!(src.len() <= MAX_BLOCK_SIZE as usize);
    encode_frame_btlazy2_no_dict(src, level)
}

pub(crate) fn encode_frame_btlazy2_no_dict(src: &[u8], level: i32) -> Vec<u8> {
    encode_frame_hash_chain_no_dict(src, level, LazyBlockStrategy::BtLazy2)
}

pub(crate) fn encode_frame_btlazy2_with_dictionary(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
) -> Vec<u8> {
    encode_frame_hash_chain_with_dictionary(src, level, dictionary, LazyBlockStrategy::BtLazy2)
}

pub(crate) fn encode_frame_hash_chain_with_dictionary_and_cctx(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
    cctx: CctxParameters,
    depth: LazyBlockStrategy,
) -> Vec<u8> {
    encode_frame_hash_chain_with_dictionary_with_cctx(src, level, dictionary, depth, cctx, false)
}

pub(crate) fn encode_frame_hash_chain_with_prepared_dictionary_and_cctx(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
    cctx: CctxParameters,
    depth: LazyBlockStrategy,
) -> Vec<u8> {
    encode_frame_hash_chain_with_dictionary_with_cctx(src, level, dictionary, depth, cctx, true)
}

fn encode_frame_hash_chain_no_dict(src: &[u8], level: i32, depth: LazyBlockStrategy) -> Vec<u8> {
    let cctx = CctxParameters::for_level(level, src.len() as u64, 0);
    encode_frame_hash_chain_no_dict_with_cctx(src, cctx, depth)
}

pub(crate) fn encode_frame_hash_chain_no_dict_with_cctx(
    src: &[u8],
    cctx: CctxParameters,
    depth: LazyBlockStrategy,
) -> Vec<u8> {
    let mut output = Vec::with_capacity(compress_bound(src.len()));
    cctx.assert_resolved();
    let params = cctx.compression;
    let mut frame_state = FrameBlockState::new(params, cctx.max_block_size);
    let mut match_state = GreedyMatchState::new();
    encode_frame_hash_chain_no_dict_with_cctx_in(
        src,
        cctx,
        depth,
        &mut frame_state,
        &mut match_state,
        &mut output,
    );
    output
}

pub(crate) fn encode_frame_hash_chain_no_dict_with_cctx_in(
    src: &[u8],
    cctx: CctxParameters,
    depth: LazyBlockStrategy,
    frame_state: &mut FrameBlockState,
    match_state: &mut GreedyMatchState,
    output: &mut Vec<u8>,
) {
    cctx.assert_resolved();
    let block_encode_mode = BlockEncodeMode::from_cctx(cctx);
    let params = cctx.compression;
    output.clear();
    frame_state.reset_for_frame_with_huffman_scratch(
        params,
        cctx.max_block_size,
        Some(&mut match_state.entropy_huffman_scratch),
    );
    match_state.reset_for_frame(params);
    write_frame_header_no_dict(output, src.len(), params);
    let mut dict_limit = 0_usize;

    if src.is_empty() {
        let reusable_bytes = match_state.take_block_bytes();
        let (block_bytes, block_bytes_lease) = reusable_bytes.lease_vec();
        let encoded_block = encode_block_hash_chain_no_dict_with_state_and_policy_in_mode_into(
            GreedyBlockSource {
                src,
                block_range: 0..0,
                loaded_dict_end: 0,
            },
            true,
            params,
            frame_state.block_config,
            frame_state.repeat_offsets,
            match_state,
            GreedyBlockEncodeContext {
                previous_huff_table: None,
                fse_tables: &mut frame_state.fse_tables,
                offset_history: &mut frame_state.offset_history,
            },
            depth,
            FrameBlockState::block_policy(true),
            block_encode_mode,
            block_bytes,
        );
        output.extend_from_slice(&encoded_block.bytes);
        match_state.recycle_block_bytes(crate::workspace::ReusableVec::recover_vec(
            encoded_block.bytes,
            block_bytes_lease,
        ));
        return;
    }

    let mut block_start = 0;
    while block_start < src.len() {
        let block_size = frame_state.next_frame_chunk_block_size(
            &src[block_start..],
            block_start,
            params.strategy,
        );
        let block_end = block_start + block_size;
        dict_limit = streaming_dict_limit(dict_limit, block_start, params.window_log);
        let policy = FrameBlockState::block_policy(block_start == 0);
        let block_context = GreedyBlockEncodeContext {
            previous_huff_table: frame_state.last_huff_table.as_ref(),
            fse_tables: &mut frame_state.fse_tables,
            offset_history: &mut frame_state.offset_history,
        };
        let reusable_bytes = match_state.take_block_bytes();
        let (block_bytes, block_bytes_lease) = reusable_bytes.lease_vec();
        let encoded_block = if dict_limit == 0 {
            encode_block_hash_chain_no_dict_with_state_and_policy_in_mode_into(
                GreedyBlockSource {
                    src,
                    block_range: block_start..block_end,
                    loaded_dict_end: 0,
                },
                block_end == src.len(),
                params,
                frame_state.block_config,
                frame_state.repeat_offsets,
                match_state,
                block_context,
                depth,
                policy,
                block_encode_mode,
                block_bytes,
            )
        } else {
            encode_block_hash_chain_ext_dict_with_state_and_policy_in_mode_into(
                GreedyExtDictBlockSource {
                    src,
                    block_range: block_start..block_end,
                    dict_limit,
                    loaded_dict_end: 0,
                },
                block_end == src.len(),
                params,
                frame_state.block_config,
                frame_state.repeat_offsets,
                match_state,
                block_context,
                depth,
                policy,
                block_encode_mode,
                block_bytes,
            )
        };
        let encoded_size = encoded_block.bytes.len();
        output.extend_from_slice(&encoded_block.bytes);
        let reusable_bytes =
            crate::workspace::ReusableVec::recover_vec(encoded_block.bytes, block_bytes_lease);
        match_state.recycle_block_bytes(reusable_bytes);
        frame_state.record_encoded_block_with_huffman_scratch(
            block_size,
            encoded_size,
            encoded_block.repeat_offsets,
            encoded_block.new_huffman_table,
            Some(&mut match_state.entropy_huffman_scratch),
        );
        block_start = block_end;
    }
}

fn encode_frame_hash_chain_with_dictionary(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
    depth: LazyBlockStrategy,
) -> Vec<u8> {
    let cctx = CctxParameters::for_level_with_mode(
        level,
        src.len() as u64,
        dictionary.content.len(),
        super::params::CParamMode::NoAttachDict,
    );
    encode_frame_hash_chain_with_dictionary_with_cctx(src, level, dictionary, depth, cctx, false)
}

fn encode_frame_hash_chain_with_dictionary_with_cctx(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
    depth: LazyBlockStrategy,
    cctx: CctxParameters,
    prepared_dictionary: bool,
) -> Vec<u8> {
    let attached_dict = prepared_dictionary
        .then(|| attached_dict_cctx(level, src.len(), dictionary.raw_size, cctx, depth))
        .flatten();
    let dictionary_params =
        attached_dict.map_or(cctx.compression, |attached| attached.dictionary_params);
    let cctx = attached_dict.map_or(cctx, |attached| attached.active_cctx);
    let mut context = DictionaryFrameContext::new_with_cctx_and_dictionary_params(
        src,
        dictionary,
        cctx,
        dictionary_params,
    );
    let params = context.cctx.compression;
    let block_encode_mode = BlockEncodeMode::from_cctx(context.cctx);

    let mut match_state = GreedyMatchState::new();
    match_state.reset_for_frame(params);
    let mut attached_dictionary_state = None;
    if let Some(attached) = attached_dict {
        let dictionary_src = &context.combined[..context.dict_len];
        let mut dictionary_state = GreedyMatchState::new();
        dictionary_state.ensure_tables(attached.dictionary_params);
        dictionary_state.next_to_update = C_WINDOW_START_INDEX;
        if depth == LazyBlockStrategy::BtLazy2 {
            load_attached_dictionary_binary_tree(
                dictionary_src,
                context.dict_len.saturating_sub(8),
                C_WINDOW_START_INDEX,
                attached.dictionary_params,
                attached.dictionary_params.min_match.clamp(4, 6),
                &mut dictionary_state,
            );
        } else {
            dictionary_state.tag_table.fill(0);
            load_dictionary_rows_at_index_base(
                dictionary_src,
                context.dict_len.saturating_sub(8),
                C_WINDOW_START_INDEX,
                attached.dictionary_params,
                attached.dictionary_params.min_match.clamp(4, 6),
                &mut dictionary_state,
            );
        }
        dictionary_state.next_to_update = context.dict_len + C_WINDOW_START_INDEX;
        dictionary_state.next_to_update3 = dictionary_state.next_to_update;
        match_state.next_to_update = context.dict_len;
        match_state.next_to_update3 = context.dict_len;
        attached_dictionary_state = Some((dictionary_state, attached.dictionary_params));
    } else if depth == LazyBlockStrategy::BtLazy2 {
        load_binary_tree_prefix(
            &mut match_state,
            &context.combined,
            context.dict_len,
            params,
        );
    } else {
        load_prefix(
            &mut match_state,
            &context.combined,
            context.dict_len,
            params,
        );
    }

    if src.is_empty() {
        let encoded_block = encode_block_hash_chain_no_dict(
            src,
            true,
            params,
            context.frame_state.block_config,
            context.frame_state.repeat_offsets,
            GreedyBlockEncodeContext {
                previous_huff_table: context.frame_state.last_huff_table.as_ref(),
                fse_tables: &mut context.frame_state.fse_tables,
                offset_history: &mut context.frame_state.offset_history,
            },
            depth,
        );
        context.output.extend_from_slice(&encoded_block.bytes);
        return context.output;
    }

    let mut block_start = context.dict_len;
    let src_end = context.src_end();
    while block_start < src_end {
        let block_size = context.frame_state.next_frame_chunk_block_size(
            &context.combined[block_start..src_end],
            block_start - context.dict_len,
            params.strategy,
        );
        let block_end = block_start + block_size;
        let policy = FrameBlockState::block_policy(block_start == context.dict_len);
        let loaded_dict_end = context.loaded_dict_end_for_block(block_end, params);
        let block_context = GreedyBlockEncodeContext {
            previous_huff_table: context.frame_state.last_huff_table.as_ref(),
            fse_tables: &mut context.frame_state.fse_tables,
            offset_history: &mut context.frame_state.offset_history,
        };
        let encoded_block = if let Some((dictionary_state, dictionary_params)) =
            attached_dictionary_state.as_ref()
        {
            encode_block_attached_dict_with_state_and_policy_in_mode(
                GreedyAttachedDictBlockSource {
                    src: &context.combined,
                    block_range: block_start..block_end,
                    active_dict_limit: context.dict_len + C_WINDOW_START_INDEX,
                    active_prefix_start: context.dict_len,
                    dictionary_src: &context.combined[..context.dict_len],
                    dictionary_state,
                    dictionary_params: *dictionary_params,
                    dictionary_index_start: C_WINDOW_START_INDEX,
                },
                block_end == src_end,
                params,
                context.frame_state.block_config,
                context.frame_state.repeat_offsets,
                &mut match_state,
                block_context,
                depth,
                policy,
                block_encode_mode,
            )
        } else if loaded_dict_end == 0 {
            encode_block_hash_chain_no_dict_with_state_and_policy_in_mode(
                GreedyBlockSource {
                    src: &context.combined,
                    block_range: block_start..block_end,
                    loaded_dict_end,
                },
                block_end == src_end,
                params,
                context.frame_state.block_config,
                context.frame_state.repeat_offsets,
                &mut match_state,
                block_context,
                depth,
                policy,
                block_encode_mode,
            )
        } else {
            encode_block_hash_chain_ext_dict_with_state_and_policy_in_mode(
                GreedyExtDictBlockSource {
                    src: &context.combined,
                    block_range: block_start..block_end,
                    dict_limit: context.dict_len,
                    loaded_dict_end,
                },
                block_end == src_end,
                params,
                context.frame_state.block_config,
                context.frame_state.repeat_offsets,
                &mut match_state,
                block_context,
                depth,
                policy,
                block_encode_mode,
            )
        };
        context.frame_state.record_encoded_block(
            block_size,
            encoded_block.bytes.len(),
            encoded_block.repeat_offsets,
            encoded_block.new_huffman_table,
        );
        context.output.extend_from_slice(&encoded_block.bytes);
        block_start = block_end;
    }

    context.output
}

#[derive(Clone, Copy, Debug)]
struct AttachedDictCctx {
    active_cctx: CctxParameters,
    dictionary_params: CompressionParameters,
}

fn attached_dict_cctx(
    level: i32,
    src_size: usize,
    dictionary_size: usize,
    active_cctx: CctxParameters,
    depth: LazyBlockStrategy,
) -> Option<AttachedDictCctx> {
    let dictionary_params = CompressionParameters::for_level_with_mode(
        level,
        super::params::ZSTD_CONTENTSIZE_UNKNOWN,
        dictionary_size,
        CParamMode::CreateCDict,
    );
    if !should_attach_dict_by_default(dictionary_params.strategy, src_size as u64) {
        return None;
    }

    let mut active_params = dictionary_params.adjusted_for_mode(
        src_size as u64,
        dictionary_size,
        CParamMode::AttachDict,
    );
    active_params.window_log = active_cctx.compression.window_log;
    let search_matches_dictionary = match depth {
        LazyBlockStrategy::BtLazy2 => active_params.strategy == super::params::Strategy::BtLazy2,
        LazyBlockStrategy::Greedy | LazyBlockStrategy::Lazy | LazyBlockStrategy::Lazy2 => {
            row_match_finder_enabled(active_params)
                && row_log(active_params) == row_log(dictionary_params)
        }
    };
    if active_params.strategy != active_cctx.compression.strategy || !search_matches_dictionary {
        return None;
    }

    let mut attached_cctx =
        CctxParameters::from_compression_parameters(level, active_params, src_size as u64);
    attached_cctx.target_c_block_size = active_cctx.target_c_block_size;
    Some(AttachedDictCctx {
        active_cctx: attached_cctx,
        dictionary_params,
    })
}

#[cfg(test)]
#[path = "greedy_frame/attached_tests.rs"]
mod attached_dict_tests;
