use crate::{
    common::MAX_BLOCK_SIZE,
    encoding::{
        block_header::BlockHeader,
        blocks::{
            compress_block_with_config, compress_prepared_block, prepare_block,
            BlockCompressionConfig, PreparedBlockRef,
        },
        frame_compressor::CompressState,
        util::{
            likely_composer_lockfile_text, likely_incompressible, likely_lockfile_text, likely_text,
        },
        CompressionFileProfile, CompressionLevel, Matcher,
    },
    huff0::huff0_encoder::HuffmanTable,
};
mod candidate_state;
mod partition;
#[cfg(feature = "std")]
mod tuning;

use candidate_state::{
    CandidateEncodeState, CandidateHuffmanState, CandidateResult, FsePreviousState,
};
#[cfg(test)]
use partition::best_split_mid_by_decompressed_bytes;
use partition::{derive_best_partitions, PreparedRange, BEST_SPLIT_MAX_PARTITIONS};
#[cfg(feature = "std")]
use tuning::fastest_tuning_overrides;

use alloc::vec::Vec;
use core::slice;

/// Compresses a single block at [`crate::encoding::CompressionLevel::Fastest`].
///
/// # Parameters
/// - `state`: [`CompressState`] so the compressor can refer to data before
///   the start of this block
/// - `last_block`: Whether or not this block is going to be the last block in the frame
///   (needed because this info is written into the block header)
/// - `uncompressed_data`: A block's worth of uncompressed data, taken from the
///   larger input
/// - `output`: As `uncompressed_data` is compressed, it's appended to `output`.
#[inline]
pub fn compress_fastest<M: Matcher>(
    state: &mut CompressState<M>,
    last_block: bool,
    uncompressed_data: Vec<u8>,
    output: &mut Vec<u8>,
) {
    compress_at_level_with_options(
        state,
        CompressionLevel::Fastest,
        last_block,
        uncompressed_data,
        output,
        true,
    );
}

#[inline]
pub fn compress_at_level<M: Matcher>(
    state: &mut CompressState<M>,
    level: CompressionLevel,
    last_block: bool,
    uncompressed_data: Vec<u8>,
    output: &mut Vec<u8>,
) {
    compress_at_level_with_options(state, level, last_block, uncompressed_data, output, true);
}

#[inline]
pub(crate) fn compress_at_level_without_incompressible_probe<M: Matcher>(
    state: &mut CompressState<M>,
    level: CompressionLevel,
    last_block: bool,
    uncompressed_data: Vec<u8>,
    output: &mut Vec<u8>,
) {
    compress_at_level_with_options(state, level, last_block, uncompressed_data, output, false);
}

#[inline]
fn compress_at_level_with_options<M: Matcher>(
    state: &mut CompressState<M>,
    level: CompressionLevel,
    last_block: bool,
    uncompressed_data: Vec<u8>,
    output: &mut Vec<u8>,
    allow_incompressible_probe: bool,
) {
    let block_size = uncompressed_data.len() as u32;
    let is_composer_dictionary_block =
        matches!(
            state.file_type_hint,
            crate::encoding::CompressionFileType::DictionaryText
        ) && (matches!(
            state.file_profile_hint,
            CompressionFileProfile::ComposerLock
        ) || likely_composer_lockfile_text(&uncompressed_data));
    let is_lockfile_dictionary_block = matches!(
        state.file_type_hint,
        crate::encoding::CompressionFileType::DictionaryText
    ) && likely_lockfile_text(&uncompressed_data);
    #[cfg(not(feature = "std"))]
    let _ = is_lockfile_dictionary_block;
    if uncompressed_data.is_empty() {
        write_raw_block(last_block, block_size, &uncompressed_data, output);
        return;
    }

    // First check to see if run length encoding can be used for the entire block
    if uncompressed_data.iter().all(|x| uncompressed_data[0].eq(x)) {
        let rle_byte = uncompressed_data[0];
        state.matcher.commit_space(uncompressed_data);
        state.matcher.skip_matching_for_rle();
        write_rle_block(last_block, block_size, rle_byte, output);
    } else if allow_incompressible_probe && likely_incompressible(&uncompressed_data) {
        state.matcher.commit_space(uncompressed_data);
        state.matcher.skip_matching_for_incompressible();
        write_raw_block(
            last_block,
            block_size,
            state.matcher.get_last_space(),
            output,
        );
    } else {
        // Compress as a standard compressed block
        state.matcher.commit_space(uncompressed_data);
        #[cfg(feature = "std")]
        let use_lockfile_fastest_splits = matches!(
            fastest_tuning_overrides().lockfile_fastest_splits,
            Some(true)
        ) && matches!(level, CompressionLevel::Fastest)
            && is_lockfile_dictionary_block;
        #[cfg(not(feature = "std"))]
        let use_lockfile_fastest_splits = false;
        if matches!(level, CompressionLevel::Best)
            || (matches!(level, CompressionLevel::Fastest) && is_composer_dictionary_block)
            || use_lockfile_fastest_splits
        {
            let partition_config = if matches!(level, CompressionLevel::Best) {
                BlockCompressionConfig::for_level(CompressionLevel::Best)
            } else {
                BlockCompressionConfig::for_level_and_hints(
                    level,
                    state.file_type_hint,
                    state.file_profile_hint,
                )
            };
            let max_partitions =
                if matches!(level, CompressionLevel::Fastest) && is_composer_dictionary_block {
                    #[cfg(feature = "std")]
                    if let Some(value) = fastest_tuning_overrides().composer_max_partitions {
                        value.max(1)
                    } else {
                        BEST_SPLIT_MAX_PARTITIONS
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        BEST_SPLIT_MAX_PARTITIONS
                    }
                } else {
                    BEST_SPLIT_MAX_PARTITIONS
                };
            #[cfg(feature = "std")]
            let force_compare_whole = use_lockfile_fastest_splits
                && matches!(
                    fastest_tuning_overrides().lockfile_compare_whole_text,
                    Some(true)
                );
            #[cfg(not(feature = "std"))]
            let force_compare_whole = false;
            compress_best_with_estimated_splits(
                state,
                last_block,
                output,
                max_partitions,
                partition_config,
                force_compare_whole,
            );
        } else {
            emit_matcher_block_or_raw(state, level, last_block, output);
        }
    }
}

fn emit_matcher_block_or_raw<M: Matcher>(
    state: &mut CompressState<M>,
    level: CompressionLevel,
    last_block: bool,
    output: &mut Vec<u8>,
) {
    let block_size = state.matcher.get_last_space().len() as u32;
    let config = BlockCompressionConfig::for_level_and_hints(
        level,
        state.file_type_hint,
        state.file_profile_hint,
    );
    let previous_ll = state.fse_tables.ll_previous.clone();
    let previous_ml = state.fse_tables.ml_previous.clone();
    let previous_of = state.fse_tables.of_previous.clone();
    let previous_offsets = state.offset_history;
    let block_start = output.len();
    output.extend_from_slice(&[0; 3]);
    output.reserve(compressed_block_reserve(block_size));
    let compressed_start = output.len();
    let new_huffman_table = compress_block_with_config(state, output, config);
    let compressed_size = output.len() - compressed_start;
    if compressed_size >= block_size as usize || compressed_size > MAX_BLOCK_SIZE as usize {
        output.truncate(block_start);
        state.fse_tables.ll_previous = previous_ll;
        state.fse_tables.ml_previous = previous_ml;
        state.fse_tables.of_previous = previous_of;
        state.offset_history = previous_offsets;
        let (newest, second, third) = previous_offsets.as_offsets();
        state.matcher.set_repeat_offsets(newest, second, third);

        write_raw_block(
            last_block,
            block_size,
            state.matcher.get_last_space(),
            output,
        );
    } else {
        let header = BlockHeader {
            last_block,
            block_type: crate::blocks::block::BlockType::Compressed,
            block_size: compressed_size as u32,
        };
        output[block_start..compressed_start].copy_from_slice(&header.serialize_to_bytes());
        if let Some(table) = new_huffman_table {
            state.last_huff_table = Some(table);
        }
    }
}

fn compress_best_with_estimated_splits<M: Matcher>(
    state: &mut CompressState<M>,
    last_block: bool,
    output: &mut Vec<u8>,
    max_partitions: usize,
    partition_config: BlockCompressionConfig,
    force_compare_whole: bool,
) {
    let baseline_offsets = state.offset_history;
    let baseline_last_huff = state.last_huff_table.take();
    let prepared = prepare_block(state);
    let data = state.matcher.get_last_space();
    let whole_range = PreparedRange {
        prepared: prepared.as_ref(),
        data,
    };
    let mut partitions = Vec::with_capacity(max_partitions);
    derive_best_partitions(
        whole_range,
        baseline_offsets,
        max_partitions,
        &mut partitions,
    );
    let compare_against_whole =
        partitions.len() == max_partitions && (force_compare_whole || !likely_text(data));
    let baseline_fse_previous =
        compare_against_whole.then(|| FsePreviousState::snapshot(&state.fse_tables));
    let whole_candidate = if compare_against_whole {
        let mut encode_state = CandidateEncodeState {
            fse_tables: &mut state.fse_tables,
            offset_history: &mut state.offset_history,
        };
        attempt_partitioned_candidate(
            slice::from_ref(&whole_range),
            last_block,
            partition_config,
            baseline_last_huff.as_ref(),
            &mut encode_state,
        )
    } else {
        None
    };

    if let Some(baseline_fse_previous) = baseline_fse_previous.as_ref() {
        baseline_fse_previous.clone().restore(&mut state.fse_tables);
        state.offset_history = baseline_offsets;
    }

    let partitioned_candidate = {
        let mut encode_state = CandidateEncodeState {
            fse_tables: &mut state.fse_tables,
            offset_history: &mut state.offset_history,
        };
        attempt_partitioned_candidate(
            &partitions,
            last_block,
            partition_config,
            baseline_last_huff.as_ref(),
            &mut encode_state,
        )
    };

    let chosen = choose_best_candidate(whole_candidate, partitioned_candidate);
    let Some(chosen) = chosen else {
        if let Some(baseline_fse_previous) = baseline_fse_previous {
            baseline_fse_previous.restore(&mut state.fse_tables);
        }
        state.offset_history = baseline_offsets;
        state.last_huff_table = baseline_last_huff;
        write_raw_block(last_block, data.len() as u32, data, output);
        return;
    };
    if partitions.len() > 1 && chosen.bytes.len() >= data.len() + 3 {
        state.fse_tables.reset();
        state.offset_history = baseline_offsets;
        state.last_huff_table = baseline_last_huff;
        write_raw_block(last_block, data.len() as u32, data, output);
        return;
    }

    chosen.final_fse_previous.restore(&mut state.fse_tables);
    state.offset_history = chosen.final_offset_history;
    state.last_huff_table = chosen.final_last_huff.or(baseline_last_huff);
    output.extend_from_slice(&chosen.bytes);
}

fn choose_best_candidate(
    whole: Option<CandidateResult>,
    partitioned: Option<CandidateResult>,
) -> Option<CandidateResult> {
    match (whole, partitioned) {
        (Some(whole), Some(partitioned)) => {
            if partitioned.bytes.len() < whole.bytes.len() {
                Some(partitioned)
            } else {
                Some(whole)
            }
        }
        (Some(whole), None) => Some(whole),
        (None, Some(partitioned)) => Some(partitioned),
        (None, None) => None,
    }
}

fn attempt_partitioned_candidate(
    partitions: &[PreparedRange<'_>],
    last_block: bool,
    config: BlockCompressionConfig,
    previous_huff_table: Option<&HuffmanTable>,
    encode_state: &mut CandidateEncodeState<'_>,
) -> Option<CandidateResult> {
    attempt_partitioned_candidate_with_config(
        partitions,
        last_block,
        config,
        previous_huff_table,
        encode_state,
    )
}

fn attempt_partitioned_candidate_with_config(
    partitions: &[PreparedRange<'_>],
    last_block: bool,
    config: BlockCompressionConfig,
    previous_huff_table: Option<&HuffmanTable>,
    encode_state: &mut CandidateEncodeState<'_>,
) -> Option<CandidateResult> {
    let mut bytes = Vec::new();
    let mut current_last_huff = CandidateHuffmanState::Unchanged(previous_huff_table);
    for (idx, partition) in partitions.iter().enumerate() {
        emit_prepared_candidate(
            partition.prepared,
            partition.data,
            last_block && idx + 1 == partitions.len(),
            config,
            &mut bytes,
            &mut current_last_huff,
            encode_state,
        )?;
    }
    Some(CandidateResult {
        bytes,
        final_fse_previous: FsePreviousState::snapshot(encode_state.fse_tables),
        final_offset_history: *encode_state.offset_history,
        final_last_huff: current_last_huff.into_owned(),
    })
}

fn emit_prepared_candidate(
    prepared: PreparedBlockRef<'_>,
    data: &[u8],
    last_block: bool,
    config: BlockCompressionConfig,
    output: &mut Vec<u8>,
    current_last_huff: &mut CandidateHuffmanState<'_>,
    encode_state: &mut CandidateEncodeState<'_>,
) -> Option<()> {
    emit_prepared_candidate_with_config(
        prepared,
        data,
        last_block,
        config,
        output,
        current_last_huff,
        encode_state,
    )
}

fn emit_prepared_candidate_with_config(
    prepared: PreparedBlockRef<'_>,
    data: &[u8],
    last_block: bool,
    config: BlockCompressionConfig,
    output: &mut Vec<u8>,
    current_last_huff: &mut CandidateHuffmanState<'_>,
    encode_state: &mut CandidateEncodeState<'_>,
) -> Option<()> {
    let block_size = data.len() as u32;
    if data.is_empty() {
        write_raw_block(last_block, block_size, data, output);
        return Some(());
    }

    if data.iter().all(|byte| *byte == data[0]) {
        let header = BlockHeader {
            last_block,
            block_type: crate::blocks::block::BlockType::RLE,
            block_size,
        };
        header.serialize(output);
        output.push(data[0]);
        return Some(());
    }

    let previous_fse = FsePreviousState::snapshot(encode_state.fse_tables);
    let previous_offsets = *encode_state.offset_history;
    let block_start = output.len();
    output.extend_from_slice(&[0; 3]);
    output.reserve(compressed_block_reserve(block_size));
    let compressed_start = output.len();
    let new_huffman_table = compress_prepared_block(
        output,
        config,
        prepared,
        encode_state.fse_tables,
        encode_state.offset_history,
        current_last_huff.as_ref(),
    );
    let compressed_size = output.len() - compressed_start;

    if compressed_size >= block_size as usize || compressed_size > MAX_BLOCK_SIZE as usize {
        output.truncate(block_start);
        previous_fse.restore(encode_state.fse_tables);
        *encode_state.offset_history = previous_offsets;
        write_raw_block(last_block, block_size, data, output);
    } else {
        let header = BlockHeader {
            last_block,
            block_type: crate::blocks::block::BlockType::Compressed,
            block_size: compressed_size as u32,
        };
        output[block_start..compressed_start].copy_from_slice(&header.serialize_to_bytes());
        if let Some(table) = new_huffman_table {
            current_last_huff.update(table);
        }
    }

    Some(())
}

fn compressed_block_reserve(block_size: u32) -> usize {
    (block_size as usize / 2).max(1024)
}

fn write_rle_block(last_block: bool, block_size: u32, rle_byte: u8, output: &mut Vec<u8>) {
    let header = BlockHeader {
        last_block,
        block_type: crate::blocks::block::BlockType::RLE,
        block_size,
    };
    header.serialize(output);
    output.push(rle_byte);
}

fn write_raw_block(last_block: bool, block_size: u32, data: &[u8], output: &mut Vec<u8>) {
    let header = BlockHeader {
        last_block,
        block_type: crate::blocks::block::BlockType::Raw,
        block_size,
    };
    header.serialize(output);
    output.extend_from_slice(data);
}

#[cfg(test)]
mod tests;
