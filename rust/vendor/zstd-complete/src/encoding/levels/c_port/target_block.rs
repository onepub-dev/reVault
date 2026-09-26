//! Target-compressed-block-size block adapter.

use alloc::vec::Vec;

use super::{
    block_emit::append_rle_block,
    greedy_block::{GreedyBlockEncodeContext, GreedyEncodedBlock, GreedyPreparedBlock},
    params::Strategy,
    sequence_store::RepeatOffsets,
    superblock::{
        append_literal_only_sub_block, select_sequence_entropy_modes, should_commit_sub_block,
        EntropyTableMode,
    },
    target_acceptance::{
        accept_target_or_raw_fallback, encode_target_block_raw_fallback, TargetAcceptanceContext,
    },
    target_modes::{
        basic_sequence_modes, compressed_sequence_modes, repeat_sequence_modes, rle_sequence_modes,
        sequence_modes_are_mixed,
    },
    target_multi::{
        try_basic_literal_multi_sub_blocks, try_huffman_literal_multi_sub_blocks, TargetMultiBlock,
    },
    target_single::{
        try_huffman_literal_only_sub_block, try_huffman_sequence_sub_block, try_sequence_sub_block,
    },
};
use crate::{
    encoding::blocks::{build_huffman_literal_table_with_optimal_depth, HuffmanLiteralMode},
    huff0::huff0_encoder::HuffmanTable,
};

const BLOCK_HEADER_SIZE: usize = 3;

#[derive(Clone, Copy)]
pub(super) struct TargetBlockOptions {
    pub(super) target_c_block_size: usize,
    pub(super) strategy: Strategy,
    pub(super) allow_rle: bool,
    pub(super) repeat_offsets: RepeatOffsets,
}

pub(super) fn encode_target_block_with_superblock_fallback(
    block: &[u8],
    last_block: bool,
    options: TargetBlockOptions,
    prepared: &GreedyPreparedBlock,
    context: GreedyBlockEncodeContext<'_, '_>,
    bytes: Vec<u8>,
) -> GreedyEncodedBlock {
    let repeat_offsets = options.repeat_offsets;
    let previous_fse = context.fse_tables.snapshot_previous();
    let previous_offsets = *context.offset_history;
    if options.allow_rle && target_maybe_rle(prepared) && literal_rle_byte(block).is_some() {
        let mut candidate = bytes;
        if append_rle_block(block, last_block, &mut candidate) {
            return GreedyEncodedBlock {
                bytes: candidate,
                repeat_offsets,
                new_huffman_table: None,
            };
        }
        unreachable!("literal_rle_byte and append_rle_block use the same predicate");
    }

    if prepared.prepared.sequences.is_empty()
        && literal_rle_byte(prepared.prepared.literals.as_slice()).is_some()
    {
        let mut candidate = bytes.clone();
        if let Some(emission) = append_literal_only_sub_block(
            prepared.prepared.literals.as_slice(),
            last_block,
            EntropyTableMode::Rle,
            basic_sequence_modes(),
            false,
            false,
            &mut candidate,
        ) {
            if should_commit_sub_block(emission.byte_size, block.len()) {
                return accept_target_or_raw_fallback(
                    GreedyEncodedBlock {
                        bytes: candidate,
                        repeat_offsets,
                        new_huffman_table: None,
                    },
                    TargetAcceptanceContext {
                        block,
                        last_block,
                        strategy: options.strategy,
                        repeat_offsets,
                        initial_bytes: &bytes,
                        fse_tables: context.fse_tables,
                        offset_history: context.offset_history,
                        previous_fse,
                        previous_offsets,
                    },
                );
            }
        }
    }

    if prepared.prepared.sequences.is_empty() {
        let previous_fse = context.fse_tables.snapshot_previous();
        let previous_offsets = *context.offset_history;
        if let Some(encoded) = try_huffman_literal_only_sub_block(
            block,
            last_block,
            prepared,
            context.fse_tables,
            context.offset_history,
            &bytes,
            options.strategy,
            repeat_offsets,
        ) {
            return accept_target_or_raw_fallback(
                encoded,
                TargetAcceptanceContext {
                    block,
                    last_block,
                    strategy: options.strategy,
                    repeat_offsets,
                    initial_bytes: &bytes,
                    fse_tables: context.fse_tables,
                    offset_history: context.offset_history,
                    previous_fse,
                    previous_offsets,
                },
            );
        }
    }

    if !prepared.prepared.sequences.is_empty() {
        let previous_huff_table = context.previous_huff_table;
        let fse_tables = context.fse_tables;
        let offset_history = context.offset_history;
        let selected_sequence_modes = select_sequence_entropy_modes(
            prepared.prepared.sequences.as_slice(),
            fse_tables,
            *offset_history,
            options.strategy,
        );
        let multi_target = TargetMultiBlock {
            block,
            last_block,
            target_c_block_size: options.target_c_block_size,
            strategy: options.strategy,
            initial_repeat_offsets: repeat_offsets,
            bytes: &bytes,
        };
        if let Some(encoded) =
            try_huffman_literal_multi_sub_blocks(multi_target, prepared, fse_tables, offset_history)
        {
            return accept_target_or_raw_fallback(
                encoded,
                TargetAcceptanceContext {
                    block,
                    last_block,
                    strategy: options.strategy,
                    repeat_offsets,
                    initial_bytes: &bytes,
                    fse_tables,
                    offset_history,
                    previous_fse: previous_fse.clone(),
                    previous_offsets,
                },
            );
        }
        let prefer_repeat_literals = c_target_selects_repeat_huffman_literals(
            options.strategy,
            prepared.prepared.literals.as_slice(),
            previous_huff_table,
        );
        if prefer_repeat_literals {
            if selected_sequence_modes != repeat_sequence_modes() {
                if let Some(encoded) = try_huffman_sequence_sub_block(
                    block,
                    last_block,
                    prepared,
                    previous_huff_table,
                    fse_tables,
                    offset_history,
                    &bytes,
                    HuffmanLiteralMode::Repeat,
                    options.strategy,
                    selected_sequence_modes,
                ) {
                    return accept_target_or_raw_fallback(
                        encoded,
                        TargetAcceptanceContext {
                            block,
                            last_block,
                            strategy: options.strategy,
                            repeat_offsets,
                            initial_bytes: &bytes,
                            fse_tables,
                            offset_history,
                            previous_fse: previous_fse.clone(),
                            previous_offsets,
                        },
                    );
                }
            }
            if let Some(encoded) = try_huffman_sequence_sub_block(
                block,
                last_block,
                prepared,
                previous_huff_table,
                fse_tables,
                offset_history,
                &bytes,
                HuffmanLiteralMode::Repeat,
                options.strategy,
                repeat_sequence_modes(),
            ) {
                return accept_target_or_raw_fallback(
                    encoded,
                    TargetAcceptanceContext {
                        block,
                        last_block,
                        strategy: options.strategy,
                        repeat_offsets,
                        initial_bytes: &bytes,
                        fse_tables,
                        offset_history,
                        previous_fse: previous_fse.clone(),
                        previous_offsets,
                    },
                );
            }
        }
        if selected_sequence_modes != compressed_sequence_modes() {
            if let Some(encoded) = try_huffman_sequence_sub_block(
                block,
                last_block,
                prepared,
                previous_huff_table,
                fse_tables,
                offset_history,
                &bytes,
                HuffmanLiteralMode::Compressed,
                options.strategy,
                selected_sequence_modes,
            ) {
                return accept_target_or_raw_fallback(
                    encoded,
                    TargetAcceptanceContext {
                        block,
                        last_block,
                        strategy: options.strategy,
                        repeat_offsets,
                        initial_bytes: &bytes,
                        fse_tables,
                        offset_history,
                        previous_fse: previous_fse.clone(),
                        previous_offsets,
                    },
                );
            }
        }
        if let Some(encoded) = try_huffman_sequence_sub_block(
            block,
            last_block,
            prepared,
            previous_huff_table,
            fse_tables,
            offset_history,
            &bytes,
            HuffmanLiteralMode::Compressed,
            options.strategy,
            compressed_sequence_modes(),
        ) {
            return accept_target_or_raw_fallback(
                encoded,
                TargetAcceptanceContext {
                    block,
                    last_block,
                    strategy: options.strategy,
                    repeat_offsets,
                    initial_bytes: &bytes,
                    fse_tables,
                    offset_history,
                    previous_fse: previous_fse.clone(),
                    previous_offsets,
                },
            );
        }
        if !prefer_repeat_literals && selected_sequence_modes != repeat_sequence_modes() {
            if let Some(encoded) = try_huffman_sequence_sub_block(
                block,
                last_block,
                prepared,
                previous_huff_table,
                fse_tables,
                offset_history,
                &bytes,
                HuffmanLiteralMode::Repeat,
                options.strategy,
                selected_sequence_modes,
            ) {
                return accept_target_or_raw_fallback(
                    encoded,
                    TargetAcceptanceContext {
                        block,
                        last_block,
                        strategy: options.strategy,
                        repeat_offsets,
                        initial_bytes: &bytes,
                        fse_tables,
                        offset_history,
                        previous_fse: previous_fse.clone(),
                        previous_offsets,
                    },
                );
            }
        }
        if !prefer_repeat_literals {
            if let Some(encoded) = try_huffman_sequence_sub_block(
                block,
                last_block,
                prepared,
                previous_huff_table,
                fse_tables,
                offset_history,
                &bytes,
                HuffmanLiteralMode::Repeat,
                options.strategy,
                repeat_sequence_modes(),
            ) {
                return accept_target_or_raw_fallback(
                    encoded,
                    TargetAcceptanceContext {
                        block,
                        last_block,
                        strategy: options.strategy,
                        repeat_offsets,
                        initial_bytes: &bytes,
                        fse_tables,
                        offset_history,
                        previous_fse: previous_fse.clone(),
                        previous_offsets,
                    },
                );
            }
        }
        if let Some(encoded) =
            try_basic_literal_multi_sub_blocks(multi_target, prepared, fse_tables, offset_history)
        {
            return accept_target_or_raw_fallback(
                encoded,
                TargetAcceptanceContext {
                    block,
                    last_block,
                    strategy: options.strategy,
                    repeat_offsets,
                    initial_bytes: &bytes,
                    fse_tables,
                    offset_history,
                    previous_fse: previous_fse.clone(),
                    previous_offsets,
                },
            );
        }
        if selected_sequence_modes == basic_sequence_modes() {
            if let Some(encoded) = try_sequence_sub_block(
                block,
                last_block,
                prepared,
                fse_tables,
                offset_history,
                &bytes,
                selected_sequence_modes,
            ) {
                return accept_target_or_raw_fallback(
                    encoded,
                    TargetAcceptanceContext {
                        block,
                        last_block,
                        strategy: options.strategy,
                        repeat_offsets,
                        initial_bytes: &bytes,
                        fse_tables,
                        offset_history,
                        previous_fse: previous_fse.clone(),
                        previous_offsets,
                    },
                );
            }
        }
        if let Some(encoded) = try_sequence_sub_block(
            block,
            last_block,
            prepared,
            fse_tables,
            offset_history,
            &bytes,
            repeat_sequence_modes(),
        ) {
            return accept_target_or_raw_fallback(
                encoded,
                TargetAcceptanceContext {
                    block,
                    last_block,
                    strategy: options.strategy,
                    repeat_offsets,
                    initial_bytes: &bytes,
                    fse_tables,
                    offset_history,
                    previous_fse: previous_fse.clone(),
                    previous_offsets,
                },
            );
        }
        if sequence_modes_are_mixed(selected_sequence_modes) {
            if let Some(encoded) = try_sequence_sub_block(
                block,
                last_block,
                prepared,
                fse_tables,
                offset_history,
                &bytes,
                selected_sequence_modes,
            ) {
                return accept_target_or_raw_fallback(
                    encoded,
                    TargetAcceptanceContext {
                        block,
                        last_block,
                        strategy: options.strategy,
                        repeat_offsets,
                        initial_bytes: &bytes,
                        fse_tables,
                        offset_history,
                        previous_fse: previous_fse.clone(),
                        previous_offsets,
                    },
                );
            }
        }
        if let Some(encoded) = try_sequence_sub_block(
            block,
            last_block,
            prepared,
            fse_tables,
            offset_history,
            &bytes,
            rle_sequence_modes(),
        ) {
            return accept_target_or_raw_fallback(
                encoded,
                TargetAcceptanceContext {
                    block,
                    last_block,
                    strategy: options.strategy,
                    repeat_offsets,
                    initial_bytes: &bytes,
                    fse_tables,
                    offset_history,
                    previous_fse: previous_fse.clone(),
                    previous_offsets,
                },
            );
        }
        if let Some(encoded) = try_sequence_sub_block(
            block,
            last_block,
            prepared,
            fse_tables,
            offset_history,
            &bytes,
            compressed_sequence_modes(),
        ) {
            return accept_target_or_raw_fallback(
                encoded,
                TargetAcceptanceContext {
                    block,
                    last_block,
                    strategy: options.strategy,
                    repeat_offsets,
                    initial_bytes: &bytes,
                    fse_tables,
                    offset_history,
                    previous_fse: previous_fse.clone(),
                    previous_offsets,
                },
            );
        }
        if let Some(encoded) = try_sequence_sub_block(
            block,
            last_block,
            prepared,
            fse_tables,
            offset_history,
            &bytes,
            basic_sequence_modes(),
        ) {
            return accept_target_or_raw_fallback(
                encoded,
                TargetAcceptanceContext {
                    block,
                    last_block,
                    strategy: options.strategy,
                    repeat_offsets,
                    initial_bytes: &bytes,
                    fse_tables,
                    offset_history,
                    previous_fse: previous_fse.clone(),
                    previous_offsets,
                },
            );
        }
    }

    encode_target_block_raw_fallback(block, last_block, repeat_offsets, bytes)
}

fn literal_rle_byte(literals: &[u8]) -> Option<u8> {
    let first = *literals.first()?;
    literals.iter().all(|byte| *byte == first).then_some(first)
}

fn c_target_selects_repeat_huffman_literals(
    strategy: Strategy,
    literals: &[u8],
    previous_huffman_table: Option<&HuffmanTable>,
) -> bool {
    let Some(previous_table) = previous_huffman_table else {
        return false;
    };
    let counts = literal_counts(literals);
    if !previous_table.can_encode_counts(&counts) {
        return false;
    }
    let Some(new_table) =
        build_huffman_literal_table_with_optimal_depth(literals, strategy >= Strategy::BtUltra)
    else {
        return false;
    };

    let old_size = previous_table.estimated_compressed_size_from_counts(&counts);
    let new_size = new_table.estimated_compressed_size_from_counts(&counts);
    let table_description_size = new_table.table_description_len();
    old_size < literals.len()
        && (old_size <= table_description_size + new_size
            || table_description_size + 12 >= literals.len())
}

fn literal_counts(literals: &[u8]) -> [usize; 256] {
    let mut counts = [0; 256];
    for &literal in literals {
        counts[usize::from(literal)] += 1;
    }
    counts
}

fn target_maybe_rle(prepared: &GreedyPreparedBlock) -> bool {
    prepared.prepared.sequences.len() < 4 && prepared.prepared.literals.len() < 10
}
