//! Frame-level adapter for the C double-fast no-dictionary path.

use alloc::vec::Vec;

use super::{
    c_frame_header::write_frame_header_no_dict,
    cctx_params::CctxParameters,
    compress_bound::compress_bound,
    dfast::DFastMatchState,
    dfast_block::{
        append_block_double_fast_ext_dict_with_state_and_policy_in_mode,
        append_block_double_fast_no_dict_with_policy,
        append_block_double_fast_no_dict_with_state_and_policy_in_mode, DFastBlockEncodeContext,
        DFastBlockSource, DFastExtDictBlockSource,
    },
    dfast_dict::load_cdict_copy_prefix,
    dictionary::ParsedDictionary,
    dictionary_frame::DictionaryFrameContext,
    frame_state::{BlockEncodeMode, FrameBlockState},
    params::{CParamMode, CompressionParameters, ZSTD_CONTENTSIZE_UNKNOWN},
};
use crate::common::MAX_BLOCK_SIZE;

pub(crate) fn encode_single_block_frame_double_fast_no_dict(src: &[u8], level: i32) -> Vec<u8> {
    debug_assert!(src.len() <= MAX_BLOCK_SIZE as usize);
    encode_frame_double_fast_no_dict(src, level)
}

pub(crate) fn encode_frame_double_fast_no_dict(src: &[u8], level: i32) -> Vec<u8> {
    let cctx = CctxParameters::for_level(level, src.len() as u64, 0);
    encode_frame_double_fast_no_dict_with_cctx(src, cctx)
}

pub(crate) fn encode_frame_double_fast_no_dict_with_cctx(
    src: &[u8],
    cctx: CctxParameters,
) -> Vec<u8> {
    let mut output = Vec::with_capacity(compress_bound(src.len()));
    cctx.assert_resolved();
    let params = cctx.compression;
    let mut frame_state = FrameBlockState::new(params, cctx.max_block_size);
    let mut match_state = DFastMatchState::new();
    encode_frame_double_fast_no_dict_with_cctx_in(
        src,
        cctx,
        &mut frame_state,
        &mut match_state,
        &mut output,
    );
    output
}

pub(crate) fn encode_frame_double_fast_no_dict_with_cctx_in(
    src: &[u8],
    cctx: CctxParameters,
    frame_state: &mut FrameBlockState,
    match_state: &mut DFastMatchState,
    output: &mut Vec<u8>,
) {
    cctx.assert_resolved();
    let block_encode_mode = BlockEncodeMode::from_cctx(cctx);
    let params = cctx.compression;
    output.clear();
    frame_state.reset_for_frame(params, cctx.max_block_size);
    match_state.reset_for_frame(params);
    write_frame_header_no_dict(output, src.len(), params);

    if src.is_empty() {
        append_block_double_fast_no_dict_with_policy(
            src,
            true,
            params,
            frame_state.block_config,
            frame_state.repeat_offsets,
            DFastBlockEncodeContext {
                previous_huff_table: None,
                huffman_build_scratch: &mut frame_state.huffman_build_scratch,
                fse_build_scratch: &mut frame_state.fse_build_scratch,
                fse_tables: &mut frame_state.fse_tables,
                offset_history: &mut frame_state.offset_history,
            },
            FrameBlockState::block_policy(true),
            output,
        );
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
        let policy = FrameBlockState::block_policy(block_start == 0);
        let encoded_start = output.len();
        let encoded_block = append_block_double_fast_no_dict_with_state_and_policy_in_mode(
            DFastBlockSource {
                src,
                block_range: block_start..block_end,
                loaded_dict_end: 0,
            },
            block_end == src.len(),
            params,
            frame_state.block_config,
            frame_state.repeat_offsets,
            match_state,
            DFastBlockEncodeContext {
                previous_huff_table: frame_state.last_huff_table.as_ref(),
                huffman_build_scratch: &mut frame_state.huffman_build_scratch,
                fse_build_scratch: &mut frame_state.fse_build_scratch,
                fse_tables: &mut frame_state.fse_tables,
                offset_history: &mut frame_state.offset_history,
            },
            policy,
            block_encode_mode,
            output,
        );
        frame_state.record_encoded_block(
            block_size,
            output.len() - encoded_start,
            encoded_block.repeat_offsets,
            encoded_block.new_huffman_table,
        );
        block_start = block_end;
    }
}

pub(crate) fn encode_frame_double_fast_with_dictionary(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
) -> Vec<u8> {
    let cctx = CctxParameters::for_level_with_mode(
        level,
        src.len() as u64,
        dictionary.content.len(),
        CParamMode::NoAttachDict,
    );
    encode_frame_double_fast_with_dictionary_and_cctx(src, level, dictionary, cctx, false)
}

pub(crate) fn encode_frame_double_fast_with_dictionary_and_cctx(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
    cctx: CctxParameters,
    prepared_dictionary: bool,
) -> Vec<u8> {
    let cctx = if prepared_dictionary {
        cdict_copy_cctx(level, src.len() as u64, dictionary.raw_size, cctx)
    } else {
        cctx
    };
    let mut context = DictionaryFrameContext::new_with_cctx(src, dictionary, cctx);
    let params = context.cctx.compression;
    let block_encode_mode = BlockEncodeMode::from_cctx(context.cctx);

    let mut match_state = DFastMatchState::new();
    load_cdict_copy_prefix(
        &mut match_state,
        &context.combined,
        context.dict_len,
        params,
    );

    if src.is_empty() {
        append_block_double_fast_no_dict_with_policy(
            src,
            true,
            params,
            context.frame_state.block_config,
            context.frame_state.repeat_offsets,
            DFastBlockEncodeContext {
                previous_huff_table: context.frame_state.last_huff_table.as_ref(),
                huffman_build_scratch: &mut context.frame_state.huffman_build_scratch,
                fse_build_scratch: &mut context.frame_state.fse_build_scratch,
                fse_tables: &mut context.frame_state.fse_tables,
                offset_history: &mut context.frame_state.offset_history,
            },
            FrameBlockState::block_policy(true),
            &mut context.output,
        );
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
        let block_context = DFastBlockEncodeContext {
            previous_huff_table: context.frame_state.last_huff_table.as_ref(),
            huffman_build_scratch: &mut context.frame_state.huffman_build_scratch,
            fse_build_scratch: &mut context.frame_state.fse_build_scratch,
            fse_tables: &mut context.frame_state.fse_tables,
            offset_history: &mut context.frame_state.offset_history,
        };
        let encoded_start = context.output.len();
        let encoded_block = if loaded_dict_end == 0 {
            append_block_double_fast_no_dict_with_state_and_policy_in_mode(
                DFastBlockSource {
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
                policy,
                block_encode_mode,
                &mut context.output,
            )
        } else {
            append_block_double_fast_ext_dict_with_state_and_policy_in_mode(
                DFastExtDictBlockSource {
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
                policy,
                block_encode_mode,
                &mut context.output,
            )
        };
        context.frame_state.record_encoded_block(
            block_size,
            context.output.len() - encoded_start,
            encoded_block.repeat_offsets,
            encoded_block.new_huffman_table,
        );
        block_start = block_end;
    }

    context.output
}

fn cdict_copy_cctx(
    level: i32,
    pledged_src_size: u64,
    dictionary_size: usize,
    active_cctx: CctxParameters,
) -> CctxParameters {
    let mut compression = CompressionParameters::for_level_with_mode(
        level,
        ZSTD_CONTENTSIZE_UNKNOWN,
        dictionary_size,
        CParamMode::CreateCDict,
    );
    compression.window_log = active_cctx.compression.window_log;

    let mut cctx =
        CctxParameters::from_compression_parameters(level, compression, pledged_src_size);
    cctx.target_c_block_size = active_cctx.target_c_block_size;
    cctx
}
