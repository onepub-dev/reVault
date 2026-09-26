//! Frame-level adapter for the C optimal no-dictionary path.

mod attached;
mod strategy;

use attached::{attached_dict_cctx, initialize_attached_dictionary};
pub(crate) use strategy::OptFrameStrategy;
use strategy::{
    encode_block_opt_no_dict_with_state, opt_parser_strategy, selected_opt_frame_strategy,
};

use alloc::vec::Vec;

use super::{
    block_compressor::{select_block_compressor, DictionaryMode},
    c_frame_header::write_frame_header_no_dict,
    cctx_params::{CctxParameters, ParamSwitch},
    compress_bound::compress_bound,
    dictionary::ParsedDictionary,
    dictionary_frame::DictionaryFrameContext,
    frame_state::{streaming_dict_limit, BlockEncodeMode, FrameBlockState},
    greedy_block::{GreedyBlockEncodeContext, GreedyBlockSource},
    greedy_ext_block::GreedyExtDictBlockSource,
    ldm::{
        opt::{LdmOptCursor, LdmRawSeqStore},
        sequence::{
            fill_prefix_hash_table, generate_sequences_no_dict, generate_sequences_with_prefix,
        },
        LdmHashTable, LdmWorkspace,
    },
    opt_block::prime_btultra2_stats_no_dict,
    opt_dict::load_prefix,
    opt_encode::{
        encode_block_opt_attached_dict_with_state_and_policy_and_ldm_in_mode,
        encode_block_opt_ext_dict_with_state_and_policy_and_ldm_in_mode,
        OptAttachedDictBlockSource,
    },
    opt_state::OptBlockState,
    params::Strategy,
};

const ZSTD_PREDEF_THRESHOLD: usize = 8;

pub(crate) fn encode_frame_btopt_no_dict(src: &[u8], level: i32) -> Vec<u8> {
    encode_frame_opt_no_dict(src, level, OptFrameStrategy::BtOpt)
}

pub(crate) fn encode_frame_btultra_no_dict(src: &[u8], level: i32) -> Vec<u8> {
    encode_frame_opt_no_dict(src, level, OptFrameStrategy::BtUltra)
}

pub(crate) fn encode_frame_btultra2_no_dict(src: &[u8], level: i32) -> Vec<u8> {
    encode_frame_opt_no_dict(src, level, OptFrameStrategy::BtUltra2)
}

pub(crate) fn encode_frame_opt_no_dict_with_cctx(src: &[u8], cctx: CctxParameters) -> Vec<u8> {
    cctx.assert_resolved();
    encode_frame_opt_no_dict_resolved(
        src,
        cctx,
        selected_opt_frame_strategy(cctx.compression.strategy),
    )
}

pub(crate) fn encode_frame_btopt_with_dictionary(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
) -> Vec<u8> {
    encode_frame_opt_with_dictionary(src, level, dictionary, OptFrameStrategy::BtOpt)
}

pub(crate) fn encode_frame_btopt_with_dictionary_and_cctx(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
    cctx: CctxParameters,
) -> Vec<u8> {
    encode_frame_opt_with_dictionary_with_cctx(
        src,
        level,
        dictionary,
        OptFrameStrategy::BtOpt,
        cctx,
        false,
    )
}

pub(crate) fn encode_frame_btultra_with_dictionary(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
) -> Vec<u8> {
    encode_frame_opt_with_dictionary(src, level, dictionary, OptFrameStrategy::BtUltra)
}

pub(crate) fn encode_frame_btultra_with_dictionary_and_cctx(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
    cctx: CctxParameters,
) -> Vec<u8> {
    encode_frame_opt_with_dictionary_with_cctx(
        src,
        level,
        dictionary,
        OptFrameStrategy::BtUltra,
        cctx,
        false,
    )
}

pub(crate) fn encode_frame_btultra2_with_dictionary(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
) -> Vec<u8> {
    let selected = select_block_compressor(
        Strategy::BtUltra2,
        ParamSwitch::Disable,
        DictionaryMode::DictMatchState,
    )
    .expect("C supports btultra2 dictionary routing through btultra");
    encode_frame_opt_with_dictionary(
        src,
        level,
        dictionary,
        selected_opt_frame_strategy(selected.strategy),
    )
}

pub(crate) fn encode_frame_btultra2_with_dictionary_and_cctx(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
    cctx: CctxParameters,
) -> Vec<u8> {
    let selected = select_block_compressor(
        Strategy::BtUltra2,
        ParamSwitch::Disable,
        DictionaryMode::DictMatchState,
    )
    .expect("C supports btultra2 dictionary routing through btultra");
    encode_frame_opt_with_dictionary_with_cctx(
        src,
        level,
        dictionary,
        selected_opt_frame_strategy(selected.strategy),
        cctx,
        false,
    )
}

pub(crate) fn encode_frame_opt_with_prepared_dictionary_and_cctx(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
    cctx: CctxParameters,
) -> Vec<u8> {
    let selected = select_block_compressor(
        cctx.compression.strategy,
        ParamSwitch::Disable,
        DictionaryMode::DictMatchState,
    )
    .expect("C supports prepared dictionaries for every optimal strategy");
    encode_frame_opt_with_dictionary_with_cctx(
        src,
        level,
        dictionary,
        selected_opt_frame_strategy(selected.strategy),
        cctx,
        true,
    )
}

fn encode_frame_opt_no_dict(src: &[u8], level: i32, strategy: OptFrameStrategy) -> Vec<u8> {
    let cctx = CctxParameters::for_level(level, src.len() as u64, 0);
    cctx.assert_resolved();
    encode_frame_opt_no_dict_resolved(src, cctx, strategy)
}

fn encode_frame_opt_no_dict_resolved(
    src: &[u8],
    cctx: CctxParameters,
    strategy: OptFrameStrategy,
) -> Vec<u8> {
    let mut output = Vec::with_capacity(compress_bound(src.len()));
    let params = cctx.compression;
    let block_encode_mode = BlockEncodeMode::from_cctx(cctx);
    let ldm_sequences = if cctx.ldm.enable_ldm == ParamSwitch::Enable {
        let mut ldm_table = LdmHashTable::new(cctx.ldm);
        Some(generate_sequences_no_dict(src, cctx.ldm, &mut ldm_table))
    } else {
        None
    };
    let mut ldm_store = ldm_sequences
        .as_ref()
        .map(|result| LdmRawSeqStore::new(&result.sequences));
    write_frame_header_no_dict(&mut output, src.len(), params);
    let mut frame_state = FrameBlockState::new(params, cctx.max_block_size);
    let mut opt_state = OptBlockState::new();
    opt_state.reset_for_frame(params);
    let mut dict_limit = 0_usize;

    if src.is_empty() {
        let encoded_block = encode_block_opt_no_dict_with_state(
            GreedyBlockSource {
                src,
                block_range: 0..0,
                loaded_dict_end: 0,
            },
            true,
            params,
            frame_state.block_config,
            frame_state.repeat_offsets,
            &mut opt_state,
            GreedyBlockEncodeContext {
                previous_huff_table: None,
                fse_tables: &mut frame_state.fse_tables,
                offset_history: &mut frame_state.offset_history,
            },
            strategy,
            block_encode_mode,
            FrameBlockState::block_policy(true),
            None,
        );
        output.extend_from_slice(&encoded_block.bytes);
        return output;
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
        if !opt_state.is_workspace_backed()
            && block_start == 0
            && strategy == OptFrameStrategy::BtUltra2
            && src[block_start..block_end].len() > ZSTD_PREDEF_THRESHOLD
        {
            prime_btultra2_stats_no_dict(src, block_start..block_end, params, &mut opt_state);
        }
        let mut ldm_cursor =
            ldm_store.map(|store| LdmOptCursor::from_store_for_block(store, block_size as u32));
        let block_context = GreedyBlockEncodeContext {
            previous_huff_table: frame_state.last_huff_table.as_ref(),
            fse_tables: &mut frame_state.fse_tables,
            offset_history: &mut frame_state.offset_history,
        };
        let policy = FrameBlockState::block_policy(block_start == 0);

        let encoded_block = if dict_limit == 0 {
            encode_block_opt_no_dict_with_state(
                GreedyBlockSource {
                    src,
                    block_range: block_start..block_end,
                    loaded_dict_end: 0,
                },
                block_end == src.len(),
                params,
                frame_state.block_config,
                frame_state.repeat_offsets,
                &mut opt_state,
                block_context,
                strategy,
                block_encode_mode,
                policy,
                ldm_cursor.as_mut(),
            )
        } else {
            encode_block_opt_ext_dict_with_state_and_policy_and_ldm_in_mode(
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
                &mut opt_state,
                block_context,
                opt_parser_strategy(strategy),
                block_encode_mode,
                policy,
                ldm_cursor.as_mut(),
            )
        };
        if let Some(store) = ldm_store.as_mut() {
            store.skip_bytes(block_size as u32);
        }
        let encoded_size = encoded_block.bytes.len();
        frame_state.record_encoded_block(
            block_size,
            encoded_size,
            encoded_block.repeat_offsets,
            encoded_block.new_huffman_table,
        );
        output.extend_from_slice(&encoded_block.bytes);
        opt_state.recycle_block_bytes(encoded_block.bytes);
        block_start = block_end;
    }

    output
}

pub(crate) fn encode_frame_opt_no_dict_with_cctx_in(
    src: &[u8],
    cctx: CctxParameters,
    strategy: OptFrameStrategy,
    frame_state: &mut FrameBlockState,
    opt_state: &mut OptBlockState,
    mut ldm_workspace: Option<&mut LdmWorkspace>,
    output: &mut Vec<u8>,
) {
    let params = cctx.compression;
    let block_encode_mode = BlockEncodeMode::from_cctx(cctx);
    output.clear();
    frame_state.reset_for_frame_with_huffman_scratch(
        params,
        cctx.max_block_size,
        Some(&mut opt_state.entropy_huffman_scratch),
    );
    opt_state.reset_for_frame(params);
    write_frame_header_no_dict(output, src.len(), params);
    let mut dict_limit = 0_usize;

    if src.is_empty() {
        let encoded_block = encode_block_opt_no_dict_with_state(
            GreedyBlockSource {
                src,
                block_range: 0..0,
                loaded_dict_end: 0,
            },
            true,
            params,
            frame_state.block_config,
            frame_state.repeat_offsets,
            opt_state,
            GreedyBlockEncodeContext {
                previous_huff_table: None,
                fse_tables: &mut frame_state.fse_tables,
                offset_history: &mut frame_state.offset_history,
            },
            strategy,
            block_encode_mode,
            FrameBlockState::block_policy(true),
            None,
        );
        output.extend_from_slice(&encoded_block.bytes);
        opt_state.recycle_block_bytes(encoded_block.bytes);
        return;
    }

    let mut ldm_sequences = if cctx.ldm.enable_ldm == ParamSwitch::Enable {
        Some(
            ldm_workspace
                .as_deref_mut()
                .expect("enabled LDM requires a prepared workspace")
                .generate(src, cctx.ldm),
        )
    } else {
        None
    };
    let mut ldm_store = ldm_sequences
        .as_ref()
        .map(|result| LdmRawSeqStore::new(&result.sequences));

    let mut block_start = 0;
    while block_start < src.len() {
        let block_size = frame_state.next_frame_chunk_block_size(
            &src[block_start..],
            block_start,
            params.strategy,
        );
        let block_end = block_start + block_size;
        dict_limit = streaming_dict_limit(dict_limit, block_start, params.window_log);
        if !opt_state.is_workspace_backed()
            && block_start == 0
            && strategy == OptFrameStrategy::BtUltra2
            && block_size > ZSTD_PREDEF_THRESHOLD
        {
            prime_btultra2_stats_no_dict(src, block_start..block_end, params, opt_state);
        }
        let block_context = GreedyBlockEncodeContext {
            previous_huff_table: frame_state.last_huff_table.as_ref(),
            fse_tables: &mut frame_state.fse_tables,
            offset_history: &mut frame_state.offset_history,
        };
        let policy = FrameBlockState::block_policy(block_start == 0);
        let mut ldm_cursor =
            ldm_store.map(|store| LdmOptCursor::from_store_for_block(store, block_size as u32));
        let encoded_block = if dict_limit == 0 {
            encode_block_opt_no_dict_with_state(
                GreedyBlockSource {
                    src,
                    block_range: block_start..block_end,
                    loaded_dict_end: 0,
                },
                block_end == src.len(),
                params,
                frame_state.block_config,
                frame_state.repeat_offsets,
                opt_state,
                block_context,
                strategy,
                block_encode_mode,
                policy,
                ldm_cursor.as_mut(),
            )
        } else {
            encode_block_opt_ext_dict_with_state_and_policy_and_ldm_in_mode(
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
                opt_state,
                block_context,
                opt_parser_strategy(strategy),
                block_encode_mode,
                policy,
                ldm_cursor.as_mut(),
            )
        };
        if let Some(store) = ldm_store.as_mut() {
            store.skip_bytes(block_size as u32);
        }
        let encoded_size = encoded_block.bytes.len();
        frame_state.record_encoded_block_with_huffman_scratch(
            block_size,
            encoded_size,
            encoded_block.repeat_offsets,
            encoded_block.new_huffman_table,
            Some(&mut opt_state.entropy_huffman_scratch),
        );
        output.extend_from_slice(&encoded_block.bytes);
        opt_state.recycle_block_bytes(encoded_block.bytes);
        block_start = block_end;
    }
    if let (Some(workspace), Some(result)) = (ldm_workspace, ldm_sequences.take()) {
        workspace.recycle(result);
    }
}

fn encode_frame_opt_with_dictionary(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
    strategy: OptFrameStrategy,
) -> Vec<u8> {
    let cctx = CctxParameters::for_level_with_mode(
        level,
        src.len() as u64,
        dictionary.content.len(),
        super::params::CParamMode::NoAttachDict,
    );
    encode_frame_opt_with_dictionary_with_cctx(src, level, dictionary, strategy, cctx, false)
}

fn encode_frame_opt_with_dictionary_with_cctx(
    src: &[u8],
    level: i32,
    dictionary: ParsedDictionary<'_>,
    strategy: OptFrameStrategy,
    cctx: CctxParameters,
    prepared_dictionary: bool,
) -> Vec<u8> {
    let attached_dict = prepared_dictionary
        .then(|| attached_dict_cctx(level, src.len(), dictionary.raw_size, cctx, strategy))
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
    let ldm_sequences = if context.cctx.ldm.enable_ldm == ParamSwitch::Enable {
        let mut ldm_table = LdmHashTable::new(context.cctx.ldm);
        fill_prefix_hash_table(
            &context.combined,
            0..context.dict_len,
            context.cctx.ldm,
            &mut ldm_table,
        );
        Some(generate_sequences_with_prefix(
            &context.combined,
            context.dict_len..context.combined.len(),
            context.cctx.ldm,
            &mut ldm_table,
        ))
    } else {
        None
    };
    let mut ldm_store = ldm_sequences
        .as_ref()
        .map(|result| LdmRawSeqStore::new(&result.sequences));

    let mut opt_state = OptBlockState::new();
    opt_state.reset_for_frame(params);
    if let Some(seeds) = context.opt_price_seeds.take() {
        opt_state.price_state.set_dictionary_seeds(seeds);
    }
    let attached_dictionary = attached_dict.map(|attached| {
        initialize_attached_dictionary(
            &context.combined[..context.dict_len],
            attached.dictionary_params,
            &mut opt_state,
        )
    });
    if attached_dictionary.is_none() {
        load_prefix(&mut opt_state, &context.combined, context.dict_len, params);
    }

    if src.is_empty() {
        let encoded_block = encode_block_opt_no_dict_with_state(
            GreedyBlockSource {
                src,
                block_range: 0..0,
                loaded_dict_end: context.dict_len,
            },
            true,
            params,
            context.frame_state.block_config,
            context.frame_state.repeat_offsets,
            &mut opt_state,
            GreedyBlockEncodeContext {
                previous_huff_table: context.frame_state.last_huff_table.as_ref(),
                fse_tables: &mut context.frame_state.fse_tables,
                offset_history: &mut context.frame_state.offset_history,
            },
            strategy,
            block_encode_mode,
            FrameBlockState::block_policy(true),
            None,
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
        if block_start == context.dict_len
            && strategy == OptFrameStrategy::BtUltra2
            && context.combined[block_start..block_end].len() > ZSTD_PREDEF_THRESHOLD
        {
            prime_btultra2_stats_no_dict(
                &context.combined,
                block_start..block_end,
                params,
                &mut opt_state,
            );
        }

        let mut ldm_cursor =
            ldm_store.map(|store| LdmOptCursor::from_store_for_block(store, block_size as u32));
        let loaded_dict_end = context.loaded_dict_end_for_block(block_end, params);
        let block_context = GreedyBlockEncodeContext {
            previous_huff_table: context.frame_state.last_huff_table.as_ref(),
            fse_tables: &mut context.frame_state.fse_tables,
            offset_history: &mut context.frame_state.offset_history,
        };
        let policy = FrameBlockState::block_policy(block_start == context.dict_len);
        let encoded_block = if let Some(dictionary) = attached_dictionary {
            encode_block_opt_attached_dict_with_state_and_policy_and_ldm_in_mode(
                OptAttachedDictBlockSource {
                    src: &context.combined,
                    block_range: block_start..block_end,
                    dictionary,
                },
                block_end == src_end,
                params,
                context.frame_state.block_config,
                context.frame_state.repeat_offsets,
                &mut opt_state,
                block_context,
                opt_parser_strategy(strategy),
                block_encode_mode,
                policy,
                ldm_cursor.as_mut(),
            )
        } else if loaded_dict_end == 0 {
            encode_block_opt_no_dict_with_state(
                GreedyBlockSource {
                    src: &context.combined,
                    block_range: block_start..block_end,
                    loaded_dict_end,
                },
                block_end == src_end,
                params,
                context.frame_state.block_config,
                context.frame_state.repeat_offsets,
                &mut opt_state,
                block_context,
                strategy,
                block_encode_mode,
                policy,
                ldm_cursor.as_mut(),
            )
        } else {
            encode_block_opt_ext_dict_with_state_and_policy_and_ldm_in_mode(
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
                &mut opt_state,
                block_context,
                opt_parser_strategy(strategy),
                block_encode_mode,
                policy,
                ldm_cursor.as_mut(),
            )
        };
        if let Some(store) = ldm_store.as_mut() {
            store.skip_bytes(block_size as u32);
        }
        let encoded_size = encoded_block.bytes.len();
        context.frame_state.record_encoded_block(
            block_size,
            encoded_size,
            encoded_block.repeat_offsets,
            encoded_block.new_huffman_table,
        );
        context.output.extend_from_slice(&encoded_block.bytes);
        opt_state.recycle_block_bytes(encoded_block.bytes);
        block_start = block_end;
    }

    context.output
}
