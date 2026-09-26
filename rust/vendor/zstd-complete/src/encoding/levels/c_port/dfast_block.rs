//! Adapters from C double-fast sequences to the existing Rust block encoder.

mod mode;
mod target;

use alloc::vec::Vec;
use core::ops::Range;

use super::block_emit::{
    append_prepared_block_or_raw, append_special_block, append_stored_block_or_raw,
    PreparedBlockEmission,
};
use super::block_policy::BlockEncodingPolicy;
use super::dfast::{
    compress_block_double_fast_no_dict,
    compress_block_double_fast_no_dict_with_state_and_loaded_dict, DFastBlockOutput,
    DFastMatchState,
};
use super::dfast_ext::compress_block_double_fast_ext_dict_with_state;
use super::frame_state::BlockEncodeMode;
use super::params::CompressionParameters;
use super::sequence_store::{
    prepare_stored_literal_words_in, prepare_stored_sequence_words_in, prepare_stored_sequences,
    prepared_words_as_ref, prepared_words_into_block, PreparedStoreWords, RepeatOffsets,
};
use crate::{
    encoding::{
        blocks::{BlockCompressionConfig, PreparedBlock, PreparedBlockRef, StoredBlockRef},
        frame_compressor::{FseTableSnapshot, FseTables, OffsetHistory},
    },
    fse::fse_encoder::FSETableBuildScratch,
    huff0::huff0_encoder::{HuffmanBuildScratch, HuffmanTable},
};

pub(crate) use mode::{
    append_block_double_fast_ext_dict_with_state_and_policy_in_mode,
    append_block_double_fast_no_dict_with_state_and_policy_in_mode,
};

pub(crate) struct DFastPreparedBlock {
    pub(crate) prepared: PreparedBlock,
    pub(crate) repeat_offsets: RepeatOffsets,
}

pub(super) struct DFastPreparedWords {
    prepared: PreparedStoreWords,
    repeat_offsets: RepeatOffsets,
}

pub(super) struct DFastStoredWords {
    literals: PreparedStoreWords,
    matcher_output: DFastBlockOutput,
}

impl DFastPreparedWords {
    pub(super) fn into_owned(self) -> DFastPreparedBlock {
        DFastPreparedBlock {
            prepared: prepared_words_into_block(self.prepared),
            repeat_offsets: self.repeat_offsets,
        }
    }
}

pub(crate) struct DFastEncodedBlock {
    pub(crate) bytes: Vec<u8>,
    pub(crate) repeat_offsets: RepeatOffsets,
    pub(crate) new_huffman_table: Option<HuffmanTable>,
}

pub(crate) struct DFastBlockEncoding {
    pub(crate) repeat_offsets: RepeatOffsets,
    pub(crate) new_huffman_table: Option<HuffmanTable>,
}

pub(crate) struct DFastBlockEncodeContext<'a, 'table> {
    pub(crate) previous_huff_table: Option<&'table HuffmanTable>,
    pub(crate) huffman_build_scratch: &'a mut HuffmanBuildScratch,
    pub(crate) fse_build_scratch: &'a mut FSETableBuildScratch,
    pub(crate) fse_tables: &'a mut FseTables,
    pub(crate) offset_history: &'a mut OffsetHistory,
}

pub(crate) struct DFastBlockSource<'a> {
    pub(crate) src: &'a [u8],
    pub(crate) block_range: Range<usize>,
    pub(crate) loaded_dict_end: usize,
}

pub(crate) struct DFastExtDictBlockSource<'a> {
    pub(crate) src: &'a [u8],
    pub(crate) block_range: Range<usize>,
    pub(crate) dict_limit: usize,
    pub(crate) loaded_dict_end: usize,
}

pub(crate) fn prepare_block_double_fast_no_dict(
    src: &[u8],
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
) -> DFastPreparedBlock {
    let output = compress_block_double_fast_no_dict(src, params, repeat_offsets);
    let prepared = prepare_from_dfast_output(src, repeat_offsets, &output);

    DFastPreparedBlock {
        prepared,
        repeat_offsets: output.repeat_offsets,
    }
}

pub(crate) fn prepare_block_double_fast_no_dict_with_state(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut DFastMatchState,
) -> DFastPreparedBlock {
    prepare_block_double_fast_no_dict_with_state_and_loaded_dict(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        0,
    )
}

pub(crate) fn prepare_block_double_fast_no_dict_with_state_and_loaded_dict(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut DFastMatchState,
    loaded_dict_end: usize,
) -> DFastPreparedBlock {
    prepare_block_double_fast_no_dict_words_with_state_and_loaded_dict(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        loaded_dict_end,
    )
    .into_owned()
}

pub(super) fn prepare_block_double_fast_no_dict_words_with_state_and_loaded_dict(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut DFastMatchState,
    loaded_dict_end: usize,
) -> DFastPreparedWords {
    let block = &src[block_range.clone()];
    let mut output = compress_block_double_fast_no_dict_with_state_and_loaded_dict(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        loaded_dict_end,
    );
    let reuse = state.take_prepared_store();
    let prepared = prepare_from_dfast_output_in(block, repeat_offsets, &output, reuse);
    let output_repeat_offsets = output.repeat_offsets;
    state.recycle_sequence_store(core::mem::take(&mut output.sequences));

    DFastPreparedWords {
        prepared,
        repeat_offsets: output_repeat_offsets,
    }
}

pub(super) fn prepare_block_double_fast_no_dict_stored_with_state_and_loaded_dict(
    src: &[u8],
    block_range: Range<usize>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut DFastMatchState,
    loaded_dict_end: usize,
) -> DFastStoredWords {
    let block = &src[block_range.clone()];
    let matcher_output = compress_block_double_fast_no_dict_with_state_and_loaded_dict(
        src,
        block_range,
        params,
        repeat_offsets,
        state,
        loaded_dict_end,
    );
    let reuse = state.take_prepared_store();
    let literals = prepare_stored_literal_words_in(
        block,
        &matcher_output.sequences,
        matcher_output.last_literals,
        reuse,
    );
    DFastStoredWords {
        literals,
        matcher_output,
    }
}

pub(crate) fn prepare_block_double_fast_ext_dict_with_state(
    source: DFastExtDictBlockSource<'_>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut DFastMatchState,
) -> DFastPreparedBlock {
    prepare_block_double_fast_ext_dict_words_with_state(source, params, repeat_offsets, state)
        .into_owned()
}

pub(super) fn prepare_block_double_fast_ext_dict_words_with_state(
    source: DFastExtDictBlockSource<'_>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut DFastMatchState,
) -> DFastPreparedWords {
    let block = &source.src[source.block_range.clone()];
    let mut output = compress_block_double_fast_ext_dict_with_state(
        source.src,
        source.block_range,
        source.dict_limit,
        params,
        repeat_offsets,
        state,
        source.loaded_dict_end,
    );
    let reuse = state.take_prepared_store();
    let prepared = prepare_from_dfast_output_in(block, repeat_offsets, &output, reuse);
    let output_repeat_offsets = output.repeat_offsets;
    state.recycle_sequence_store(core::mem::take(&mut output.sequences));

    DFastPreparedWords {
        prepared,
        repeat_offsets: output_repeat_offsets,
    }
}

pub(super) fn prepare_block_double_fast_ext_dict_stored_with_state(
    source: DFastExtDictBlockSource<'_>,
    params: CompressionParameters,
    repeat_offsets: RepeatOffsets,
    state: &mut DFastMatchState,
) -> DFastStoredWords {
    let block = &source.src[source.block_range.clone()];
    let matcher_output = compress_block_double_fast_ext_dict_with_state(
        source.src,
        source.block_range,
        source.dict_limit,
        params,
        repeat_offsets,
        state,
        source.loaded_dict_end,
    );
    let reuse = state.take_prepared_store();
    let literals = prepare_stored_literal_words_in(
        block,
        &matcher_output.sequences,
        matcher_output.last_literals,
        reuse,
    );
    DFastStoredWords {
        literals,
        matcher_output,
    }
}

pub(crate) fn encode_block_double_fast_no_dict(
    src: &[u8],
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    context: DFastBlockEncodeContext<'_, '_>,
) -> DFastEncodedBlock {
    encode_block_double_fast_no_dict_with_policy(
        src,
        last_block,
        params,
        config,
        repeat_offsets,
        context,
        BlockEncodingPolicy::normal(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_double_fast_no_dict_with_policy(
    src: &[u8],
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    context: DFastBlockEncodeContext<'_, '_>,
    policy: BlockEncodingPolicy,
) -> DFastEncodedBlock {
    let mut bytes = Vec::new();
    let encoding = append_block_double_fast_no_dict_with_policy(
        src,
        last_block,
        params,
        config,
        repeat_offsets,
        context,
        policy,
        &mut bytes,
    );

    DFastEncodedBlock {
        bytes,
        repeat_offsets: encoding.repeat_offsets,
        new_huffman_table: encoding.new_huffman_table,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn append_block_double_fast_no_dict_with_policy(
    src: &[u8],
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    context: DFastBlockEncodeContext<'_, '_>,
    policy: BlockEncodingPolicy,
    output: &mut Vec<u8>,
) -> DFastBlockEncoding {
    if append_special_block(src, last_block, output) {
        return DFastBlockEncoding {
            repeat_offsets,
            new_huffman_table: None,
        };
    }

    let previous_fse = context.fse_tables.snapshot_previous();
    let previous_offsets = *context.offset_history;
    let prepared = prepare_block_double_fast_no_dict(src, params, repeat_offsets);
    encode_prepared_block(
        src,
        last_block,
        params,
        config,
        repeat_offsets,
        prepared,
        policy,
        previous_fse,
        previous_offsets,
        context,
        output,
    )
}

pub(crate) fn encode_block_double_fast_no_dict_with_state(
    source: DFastBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    match_state: &mut DFastMatchState,
    context: DFastBlockEncodeContext<'_, '_>,
) -> DFastEncodedBlock {
    encode_block_double_fast_no_dict_with_state_and_policy(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        match_state,
        context,
        BlockEncodingPolicy::normal(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_double_fast_no_dict_with_state_and_policy(
    source: DFastBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    match_state: &mut DFastMatchState,
    context: DFastBlockEncodeContext<'_, '_>,
    policy: BlockEncodingPolicy,
) -> DFastEncodedBlock {
    let mut bytes = Vec::new();
    let encoding = append_block_double_fast_no_dict_with_state_and_policy(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        match_state,
        context,
        policy,
        &mut bytes,
    );

    DFastEncodedBlock {
        bytes,
        repeat_offsets: encoding.repeat_offsets,
        new_huffman_table: encoding.new_huffman_table,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn append_block_double_fast_no_dict_with_state_and_policy(
    source: DFastBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    match_state: &mut DFastMatchState,
    context: DFastBlockEncodeContext<'_, '_>,
    policy: BlockEncodingPolicy,
    output: &mut Vec<u8>,
) -> DFastBlockEncoding {
    append_block_double_fast_no_dict_with_state_and_policy_in_mode(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        match_state,
        context,
        policy,
        BlockEncodeMode::Normal,
        output,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_block_double_fast_ext_dict_with_state_and_policy(
    source: DFastExtDictBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    match_state: &mut DFastMatchState,
    context: DFastBlockEncodeContext<'_, '_>,
    policy: BlockEncodingPolicy,
) -> DFastEncodedBlock {
    let mut bytes = Vec::new();
    let encoding = append_block_double_fast_ext_dict_with_state_and_policy(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        match_state,
        context,
        policy,
        &mut bytes,
    );

    DFastEncodedBlock {
        bytes,
        repeat_offsets: encoding.repeat_offsets,
        new_huffman_table: encoding.new_huffman_table,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn append_block_double_fast_ext_dict_with_state_and_policy(
    source: DFastExtDictBlockSource<'_>,
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    match_state: &mut DFastMatchState,
    context: DFastBlockEncodeContext<'_, '_>,
    policy: BlockEncodingPolicy,
    output: &mut Vec<u8>,
) -> DFastBlockEncoding {
    append_block_double_fast_ext_dict_with_state_and_policy_in_mode(
        source,
        last_block,
        params,
        config,
        repeat_offsets,
        match_state,
        context,
        policy,
        BlockEncodeMode::Normal,
        output,
    )
}

#[allow(clippy::too_many_arguments)]
fn encode_prepared_block(
    block: &[u8],
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    prepared: DFastPreparedBlock,
    policy: BlockEncodingPolicy,
    previous_fse: FseTableSnapshot,
    previous_offsets: OffsetHistory,
    context: DFastBlockEncodeContext<'_, '_>,
    output: &mut Vec<u8>,
) -> DFastBlockEncoding {
    encode_prepared_block_ref(
        block,
        last_block,
        params,
        config,
        repeat_offsets,
        prepared.prepared.as_ref(),
        prepared.repeat_offsets,
        policy,
        previous_fse,
        previous_offsets,
        context,
        output,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn encode_prepared_words_block(
    block: &[u8],
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    prepared: DFastPreparedWords,
    policy: BlockEncodingPolicy,
    previous_fse: FseTableSnapshot,
    previous_offsets: OffsetHistory,
    context: DFastBlockEncodeContext<'_, '_>,
    match_state: &mut DFastMatchState,
    output: &mut Vec<u8>,
) -> DFastBlockEncoding {
    let encoding = encode_prepared_block_ref(
        block,
        last_block,
        params,
        config,
        repeat_offsets,
        prepared_words_as_ref(&prepared.prepared),
        prepared.repeat_offsets,
        policy,
        previous_fse,
        previous_offsets,
        context,
        output,
    );
    match_state.recycle_prepared_store(prepared.prepared);
    encoding
}

#[allow(clippy::too_many_arguments)]
pub(super) fn encode_stored_words_block(
    block: &[u8],
    last_block: bool,
    params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    mut stored: DFastStoredWords,
    policy: BlockEncodingPolicy,
    context: DFastBlockEncodeContext<'_, '_>,
    match_state: &mut DFastMatchState,
    output: &mut Vec<u8>,
) -> DFastBlockEncoding {
    let compressed_repeat_offsets = stored.matcher_output.repeat_offsets;
    let emission = append_stored_block_or_raw(
        block,
        last_block,
        params.strategy,
        policy,
        config,
        StoredBlockRef {
            literals: &stored.literals.literals,
            sequences: &stored.matcher_output.sequences,
        },
        compressed_repeat_offsets,
        context.previous_huff_table,
        Some(context.huffman_build_scratch),
        Some(context.fse_build_scratch),
        context.fse_tables,
        context.offset_history,
        output,
    );
    match_state.recycle_prepared_store(stored.literals);
    match_state.recycle_sequence_store(core::mem::take(&mut stored.matcher_output.sequences));

    match emission {
        PreparedBlockEmission::Raw | PreparedBlockEmission::Rle => DFastBlockEncoding {
            repeat_offsets,
            new_huffman_table: None,
        },
        PreparedBlockEmission::Compressed { new_huffman_table } => DFastBlockEncoding {
            repeat_offsets: compressed_repeat_offsets,
            new_huffman_table,
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn encode_prepared_block_ref(
    block: &[u8],
    last_block: bool,
    strategy_params: CompressionParameters,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    prepared: PreparedBlockRef<'_>,
    compressed_repeat_offsets: RepeatOffsets,
    policy: BlockEncodingPolicy,
    previous_fse: FseTableSnapshot,
    previous_offsets: OffsetHistory,
    context: DFastBlockEncodeContext<'_, '_>,
    output: &mut Vec<u8>,
) -> DFastBlockEncoding {
    match append_prepared_block_or_raw(
        block,
        last_block,
        strategy_params.strategy,
        policy,
        config,
        prepared,
        previous_fse,
        previous_offsets,
        context.previous_huff_table,
        context.fse_tables,
        context.offset_history,
        output,
    ) {
        PreparedBlockEmission::Raw | PreparedBlockEmission::Rle => DFastBlockEncoding {
            repeat_offsets,
            new_huffman_table: None,
        },
        PreparedBlockEmission::Compressed { new_huffman_table } => DFastBlockEncoding {
            repeat_offsets: compressed_repeat_offsets,
            new_huffman_table,
        },
    }
}

fn prepare_from_dfast_output(
    src: &[u8],
    initial_repeat_offsets: RepeatOffsets,
    output: &DFastBlockOutput,
) -> PreparedBlock {
    prepare_stored_sequences(
        src,
        initial_repeat_offsets,
        &output.sequences,
        output.last_literals,
    )
}

fn prepare_from_dfast_output_in(
    src: &[u8],
    initial_repeat_offsets: RepeatOffsets,
    output: &DFastBlockOutput,
    reuse: PreparedStoreWords,
) -> PreparedStoreWords {
    prepare_stored_sequence_words_in(
        src,
        initial_repeat_offsets,
        &output.sequences,
        output.last_literals,
        reuse,
    )
}
