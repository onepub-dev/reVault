//! Stateful double-fast block adapter for explicit block-encoding modes.

use alloc::vec::Vec;

use super::{
    append_special_block, encode_prepared_words_block, encode_stored_words_block,
    prepare_block_double_fast_ext_dict_stored_with_state,
    prepare_block_double_fast_ext_dict_words_with_state,
    prepare_block_double_fast_no_dict_stored_with_state_and_loaded_dict,
    prepare_block_double_fast_no_dict_words_with_state_and_loaded_dict, target,
    DFastBlockEncodeContext, DFastBlockEncoding, DFastBlockSource, DFastExtDictBlockSource,
    DFastMatchState,
};
use crate::encoding::{
    blocks::BlockCompressionConfig,
    levels::c_port::{
        block_policy::BlockEncodingPolicy, frame_state::BlockEncodeMode,
        params::CompressionParameters, sequence_store::RepeatOffsets,
    },
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn append_block_double_fast_no_dict_with_state_and_policy_in_mode(
    source: DFastBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    match_state: &mut DFastMatchState,
    context: DFastBlockEncodeContext<'_, '_>,
    policy: BlockEncodingPolicy,
    block_encode_mode: BlockEncodeMode,
    output: &mut Vec<u8>,
) -> DFastBlockEncoding {
    let block = &source.src[source.block_range.clone()];

    if append_special_block(block, last_block, output) {
        return DFastBlockEncoding {
            repeat_offsets,
            new_huffman_table: None,
        };
    }

    if let Some(target_size) = block_encode_mode.target_c_block_size() {
        let prepared = prepare_block_double_fast_no_dict_words_with_state_and_loaded_dict(
            source.src,
            source.block_range.clone(),
            params,
            repeat_offsets,
            match_state,
            source.loaded_dict_end,
        );
        return target::append_target_block(
            block,
            last_block,
            params,
            repeat_offsets,
            prepared.into_owned(),
            policy,
            context,
            target_size,
            output,
        );
    }

    if !config.uses_c_native_sequence_store() {
        let prepared = prepare_block_double_fast_no_dict_words_with_state_and_loaded_dict(
            source.src,
            source.block_range.clone(),
            params,
            repeat_offsets,
            match_state,
            source.loaded_dict_end,
        );
        let previous_fse = context.fse_tables.snapshot_previous();
        let previous_offsets = *context.offset_history;
        return encode_prepared_words_block(
            block,
            last_block,
            params,
            config,
            repeat_offsets,
            prepared,
            policy,
            previous_fse,
            previous_offsets,
            context,
            match_state,
            output,
        );
    }

    let stored = prepare_block_double_fast_no_dict_stored_with_state_and_loaded_dict(
        source.src,
        source.block_range,
        params,
        repeat_offsets,
        match_state,
        source.loaded_dict_end,
    );

    encode_stored_words_block(
        block,
        last_block,
        params,
        config,
        repeat_offsets,
        stored,
        policy,
        context,
        match_state,
        output,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn append_block_double_fast_ext_dict_with_state_and_policy_in_mode(
    source: DFastExtDictBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    match_state: &mut DFastMatchState,
    context: DFastBlockEncodeContext<'_, '_>,
    policy: BlockEncodingPolicy,
    block_encode_mode: BlockEncodeMode,
    output: &mut Vec<u8>,
) -> DFastBlockEncoding {
    let block = &source.src[source.block_range.clone()];

    if append_special_block(block, last_block, output) {
        return DFastBlockEncoding {
            repeat_offsets,
            new_huffman_table: None,
        };
    }

    if let Some(target_size) = block_encode_mode.target_c_block_size() {
        let prepared = prepare_block_double_fast_ext_dict_words_with_state(
            source,
            params,
            repeat_offsets,
            match_state,
        );
        return target::append_target_block(
            block,
            last_block,
            params,
            repeat_offsets,
            prepared.into_owned(),
            policy,
            context,
            target_size,
            output,
        );
    }

    if !config.uses_c_native_sequence_store() {
        let prepared = prepare_block_double_fast_ext_dict_words_with_state(
            source,
            params,
            repeat_offsets,
            match_state,
        );
        let previous_fse = context.fse_tables.snapshot_previous();
        let previous_offsets = *context.offset_history;
        return encode_prepared_words_block(
            block,
            last_block,
            params,
            config,
            repeat_offsets,
            prepared,
            policy,
            previous_fse,
            previous_offsets,
            context,
            match_state,
            output,
        );
    }

    let stored = prepare_block_double_fast_ext_dict_stored_with_state(
        source,
        params,
        repeat_offsets,
        match_state,
    );

    encode_stored_words_block(
        block,
        last_block,
        params,
        config,
        repeat_offsets,
        stored,
        policy,
        context,
        match_state,
        output,
    )
}
