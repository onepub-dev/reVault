//! Post-sequence block splitter, following `ZSTD_deriveBlockSplits()`.

use alloc::vec::Vec;

use super::{
    block_emit::{append_stored_block_or_raw, PreparedBlockEmission},
    block_policy::BlockEncodingPolicy,
    greedy_block::{GreedyBlockEncodeContext, GreedyEncodedBlock, GreedyPreparedBlock},
    params::Strategy,
    sequence_store::{OffBase, RepeatOffsets, StoredSequence},
};
use crate::{
    encoding::{
        blocks::{
            estimate_prepared_block_size_with_sequences, BlockCompressionConfig, EstimateScratch,
            PreparedBlock, PreparedBlockRef, PreparedSequence, StoredBlockRef,
        },
        frame_compressor::{FseTables, OffsetHistory},
    },
    fse::fse_encoder::FSETableBuildScratch,
    huff0::huff0_encoder::{HuffmanBuildScratch, HuffmanTable},
    workspace::{Arena, ArenaError, ArenaSize, ReusableVec},
};

const MIN_SEQUENCES_BLOCK_SPLITTING: usize = 300;
const MAX_NB_BLOCK_SPLITS: usize = 196;

pub(super) struct PostSplitScratch {
    prefixes: ReusableVec<SequencePrefix>,
    partitions: ReusableVec<usize>,
    resolved_sequences: ReusableVec<StoredSequence>,
    estimate: EstimateScratch,
}

impl PostSplitScratch {
    pub(super) fn new() -> Self {
        Self {
            prefixes: ReusableVec::new(),
            partitions: ReusableVec::new(),
            resolved_sequences: ReusableVec::new(),
            estimate: EstimateScratch::new(),
        }
    }

    pub(super) fn add_workspace_size(
        size: &mut ArenaSize,
        block_size: usize,
    ) -> Result<(), ArenaError> {
        let max_sequences = block_size / 3 + 1;
        size.add::<SequencePrefix>(max_sequences + 1)?;
        size.add::<usize>((max_sequences + 1).min(MAX_NB_BLOCK_SPLITS + 1))?;
        size.add::<StoredSequence>(max_sequences)?;
        EstimateScratch::add_workspace_size(size, max_sequences)
    }

    pub(super) fn new_in(arena: &mut Arena<'_>, block_size: usize) -> Result<Self, ArenaError> {
        let max_sequences = block_size / 3 + 1;
        Ok(Self {
            prefixes: arena.allocate_reusable_vec(max_sequences + 1)?,
            partitions: arena
                .allocate_reusable_vec((max_sequences + 1).min(MAX_NB_BLOCK_SPLITS + 1))?,
            resolved_sequences: arena.allocate_reusable_vec(max_sequences)?,
            estimate: EstimateScratch::new_in(arena, max_sequences)?,
        })
    }

    fn is_workspace_backed(&self) -> bool {
        !self.prefixes.is_owned()
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn encode_split_block(
    block: &[u8],
    last_block: bool,
    policy: BlockEncodingPolicy,
    strategy: Strategy,
    config: BlockCompressionConfig,
    repeat_offsets: RepeatOffsets,
    prepared: &GreedyPreparedBlock,
    previous_offsets: OffsetHistory,
    context: &mut GreedyBlockEncodeContext<'_, '_>,
    scratch: &mut PostSplitScratch,
    huffman_scratch: &mut HuffmanBuildScratch,
    fse_scratch: &mut FSETableBuildScratch,
    mut bytes: Vec<u8>,
) -> Result<GreedyEncodedBlock, Vec<u8>> {
    sequence_prefixes_into(&prepared.prepared, &mut scratch.prefixes);
    derive_block_splits_into(
        block,
        &prepared.prepared,
        &scratch.prefixes,
        EstimateContext {
            strategy,
            config,
            fse_tables: context.fse_tables,
            offset_history: previous_offsets,
            previous_huff_table: context.previous_huff_table,
        },
        &mut scratch.estimate,
        &mut scratch.partitions,
    );
    if scratch.partitions.len() <= 1 {
        if !scratch.is_workspace_backed() {
            return Err(bytes);
        }
        scratch.partitions.clear();
        scratch.partitions.push(prepared.prepared.sequences.len());
    }

    bytes.clear();
    let mut last_huff_table = None;
    let mut decompression_repeat_offsets = repeat_offsets;
    let mut compression_repeat_offsets = repeat_offsets;

    let mut start_seq = 0usize;
    for (idx, &end_seq) in scratch.partitions.iter().enumerate() {
        let last_partition = idx + 1 == scratch.partitions.len();
        let chunk = prepared_chunk_ref(
            block,
            &prepared.prepared,
            &scratch.prefixes,
            start_seq,
            end_seq,
        );
        let decompression_repeat_offsets_before = decompression_repeat_offsets;
        let mut candidate_decompression_offsets = decompression_repeat_offsets;
        resolved_partition_sequences_into(
            chunk.prepared.sequences,
            &mut candidate_decompression_offsets,
            &mut compression_repeat_offsets,
            &mut scratch.resolved_sequences,
        );
        let emission = append_stored_block_or_raw(
            chunk.source,
            last_block && last_partition,
            strategy,
            policy,
            config,
            StoredBlockRef {
                literals: chunk.prepared.literals,
                sequences: &scratch.resolved_sequences,
            },
            candidate_decompression_offsets,
            last_huff_table.as_ref().or(context.previous_huff_table),
            Some(huffman_scratch),
            Some(fse_scratch),
            context.fse_tables,
            context.offset_history,
            &mut bytes,
        );
        if let PreparedBlockEmission::Compressed { new_huffman_table } = emission {
            decompression_repeat_offsets = candidate_decompression_offsets;
            if let Some(new_huffman_table) = new_huffman_table {
                if let Some(previous) = last_huff_table.replace(new_huffman_table) {
                    huffman_scratch.recycle_table(previous);
                }
            }
        } else {
            decompression_repeat_offsets = decompression_repeat_offsets_before;
        }
        start_seq = end_seq;
    }

    Ok(GreedyEncodedBlock {
        bytes,
        repeat_offsets: decompression_repeat_offsets,
        new_huffman_table: last_huff_table,
    })
}

fn derive_block_splits_into(
    block: &[u8],
    prepared: &PreparedBlock,
    prefixes: &[SequencePrefix],
    context: EstimateContext<'_>,
    estimate_scratch: &mut EstimateScratch,
    splits: &mut ReusableVec<usize>,
) {
    let nb_seq = prepared.sequences.len();
    splits.clear();
    if nb_seq <= 4 {
        return;
    }

    derive_block_splits_helper(
        splits,
        0,
        nb_seq,
        block,
        prepared,
        prefixes,
        context,
        estimate_scratch,
        None,
    );
    splits.push(nb_seq);
}

#[allow(clippy::too_many_arguments)]
fn derive_block_splits_helper(
    splits: &mut Vec<usize>,
    start_idx: usize,
    end_idx: usize,
    block: &[u8],
    prepared: &PreparedBlock,
    prefixes: &[SequencePrefix],
    context: EstimateContext<'_>,
    estimate_scratch: &mut EstimateScratch,
    known_original_size: Option<usize>,
) {
    if end_idx - start_idx < MIN_SEQUENCES_BLOCK_SPLITTING || splits.len() >= MAX_NB_BLOCK_SPLITS {
        return;
    }

    let mid_idx = (start_idx + end_idx) / 2;
    let original_size = known_original_size.unwrap_or_else(|| {
        estimate_partition_size_with_sequences(
            block,
            prepared,
            prefixes,
            start_idx,
            end_idx,
            context,
            estimate_scratch,
        )
    });
    let first_half_size = estimate_partition_size_with_sequences(
        block,
        prepared,
        prefixes,
        start_idx,
        mid_idx,
        context,
        estimate_scratch,
    );
    let second_half_size = estimate_partition_size_with_sequences(
        block,
        prepared,
        prefixes,
        mid_idx,
        end_idx,
        context,
        estimate_scratch,
    );

    if should_split(
        first_half_size,
        second_half_size,
        original_size,
        context.strategy,
    ) {
        derive_block_splits_helper(
            splits,
            start_idx,
            mid_idx,
            block,
            prepared,
            prefixes,
            context,
            estimate_scratch,
            Some(first_half_size),
        );
        splits.push(mid_idx);
        derive_block_splits_helper(
            splits,
            mid_idx,
            end_idx,
            block,
            prepared,
            prefixes,
            context,
            estimate_scratch,
            Some(second_half_size),
        );
    }
}

fn should_split(
    first_half_size: usize,
    second_half_size: usize,
    original_size: usize,
    _strategy: Strategy,
) -> bool {
    // Match `ZSTD_deriveBlockSplitsHelper()`: C only accepts a split when the
    // estimated halves are strictly smaller than the unsplit estimate.
    first_half_size.saturating_add(second_half_size) < original_size
}

#[derive(Clone, Copy)]
struct EstimateContext<'a> {
    strategy: Strategy,
    config: BlockCompressionConfig,
    fse_tables: &'a FseTables,
    offset_history: OffsetHistory,
    previous_huff_table: Option<&'a HuffmanTable>,
}

fn estimate_partition_size(
    block: &[u8],
    prepared: &PreparedBlock,
    prefixes: &[SequencePrefix],
    start_seq: usize,
    end_seq: usize,
    context: EstimateContext<'_>,
) -> usize {
    let mut scratch = EstimateScratch::new();
    estimate_partition_size_with_sequences(
        block,
        prepared,
        prefixes,
        start_seq,
        end_seq,
        context,
        &mut scratch,
    )
}

fn estimate_partition_size_with_sequences(
    block: &[u8],
    prepared: &PreparedBlock,
    prefixes: &[SequencePrefix],
    start_seq: usize,
    end_seq: usize,
    context: EstimateContext<'_>,
    scratch: &mut EstimateScratch,
) -> usize {
    let chunk = prepared_chunk_ref(block, prepared, prefixes, start_seq, end_seq);
    if chunk.source.is_empty() {
        return 3;
    }

    estimate_prepared_block_size_with_sequences(
        context.config.for_c_block_split_estimate(),
        chunk.prepared,
        context.fse_tables,
        context.offset_history,
        context.previous_huff_table,
        context.previous_huff_table.is_some(),
        scratch,
    )
}

struct PreparedChunk<'a> {
    source: &'a [u8],
    literals: &'a [u8],
    sequences: Vec<PreparedSequence>,
}

fn prepared_chunk<'a>(
    block: &'a [u8],
    prepared: &'a PreparedBlock,
    prefixes: &[SequencePrefix],
    start_seq: usize,
    end_seq: usize,
) -> PreparedChunk<'a> {
    let chunk = prepared_chunk_ref(block, prepared, prefixes, start_seq, end_seq);
    PreparedChunk {
        source: chunk.source,
        literals: chunk.prepared.literals,
        sequences: chunk.prepared.sequences.to_vec(),
    }
}

struct PreparedChunkRef<'a> {
    source: &'a [u8],
    prepared: PreparedBlockRef<'a>,
}

fn prepared_chunk_ref<'a>(
    block: &'a [u8],
    prepared: &'a PreparedBlock,
    prefixes: &[SequencePrefix],
    start_seq: usize,
    end_seq: usize,
) -> PreparedChunkRef<'a> {
    debug_assert!(start_seq <= end_seq);
    debug_assert!(end_seq <= prepared.sequences.len());
    debug_assert_eq!(prefixes.len(), prepared.sequences.len() + 1);

    let start = prefixes[start_seq];
    let mut end = prefixes[end_seq];
    if end_seq == prepared.sequences.len() {
        end.literal_pos = prepared.literals.len();
        end.source_pos = block.len();
    }

    PreparedChunkRef {
        source: &block[start.source_pos..end.source_pos],
        prepared: PreparedBlockRef {
            literals: &prepared.literals[start.literal_pos..end.literal_pos],
            sequences: &prepared.sequences[start_seq..end_seq],
        },
    }
}

#[derive(Clone, Copy)]
struct SequencePrefix {
    literal_pos: usize,
    source_pos: usize,
}

fn sequence_prefixes_into(prepared: &PreparedBlock, prefixes: &mut ReusableVec<SequencePrefix>) {
    prefixes.clear();
    let mut prefix = SequencePrefix {
        literal_pos: 0,
        source_pos: 0,
    };
    prefixes.push(prefix);

    for sequence in &prepared.sequences {
        let lit_len = sequence.ll as usize;
        let match_len = sequence.ml as usize;
        prefix.literal_pos += lit_len;
        prefix.source_pos += lit_len + match_len;
        prefixes.push(prefix);
    }
}

fn resolved_partition_sequences_into(
    sequences: &[PreparedSequence],
    decompression_repeat_offsets: &mut RepeatOffsets,
    compression_repeat_offsets: &mut RepeatOffsets,
    resolved: &mut ReusableVec<StoredSequence>,
) {
    resolved.clear();
    for sequence in sequences.iter().copied() {
        let original_off_base = OffBase::from_c_value(sequence.encoded_offset_value)
            .expect("C-port split partitions require C offBase values");
        let mut decompression_off_base = original_off_base;

        if matches!(original_off_base, OffBase::Repeat(_)) {
            let decompression_raw_offset =
                decompression_repeat_offsets.resolve(original_off_base, sequence.ll);
            let compression_raw_offset =
                compression_repeat_offsets.resolve(original_off_base, sequence.ll);
            if decompression_raw_offset != compression_raw_offset {
                decompression_off_base = OffBase::Offset(compression_raw_offset);
            }
        }

        decompression_repeat_offsets.update(decompression_off_base, sequence.ll);
        compression_repeat_offsets.update(original_off_base, sequence.ll);
        resolved.push(StoredSequence::new(
            sequence.ll,
            decompression_off_base,
            sequence.ml,
        ));
    }
}

#[cfg(test)]
fn sequence_prefixes(prepared: &PreparedBlock) -> Vec<SequencePrefix> {
    let mut prefixes = ReusableVec::new();
    sequence_prefixes_into(prepared, &mut prefixes);
    prefixes.into_owned_vec()
}

#[cfg(test)]
fn derive_block_splits(
    block: &[u8],
    prepared: &PreparedBlock,
    prefixes: &[SequencePrefix],
    context: EstimateContext<'_>,
    estimate_scratch: &mut EstimateScratch,
) -> Vec<usize> {
    let mut splits = ReusableVec::new();
    derive_block_splits_into(
        block,
        prepared,
        prefixes,
        context,
        estimate_scratch,
        &mut splits,
    );
    splits.into_owned_vec()
}

#[cfg(test)]
fn resolved_partition_sequences(
    sequences: &[PreparedSequence],
    decompression_repeat_offsets: &mut RepeatOffsets,
    compression_repeat_offsets: &mut RepeatOffsets,
) -> Vec<StoredSequence> {
    let mut resolved = ReusableVec::new();
    resolved_partition_sequences_into(
        sequences,
        decompression_repeat_offsets,
        compression_repeat_offsets,
        &mut resolved,
    );
    resolved.into_owned_vec()
}

#[cfg(test)]
struct PartitionEncodeContext<'a> {
    policy: BlockEncodingPolicy,
    strategy: Strategy,
    config: BlockCompressionConfig,
    fse_tables: &'a mut FseTables,
    offset_history: &'a mut OffsetHistory,
    previous_huff_table: Option<&'a HuffmanTable>,
}

#[cfg(test)]
fn encode_partition(
    block: &[u8],
    last_block: bool,
    repeat_offsets: RepeatOffsets,
    prepared: PreparedBlockRef<'_>,
    context: PartitionEncodeContext<'_>,
) -> GreedyEncodedBlock {
    let mut decompression_offsets = repeat_offsets;
    let mut compression_offsets = repeat_offsets;
    let mut resolved = ReusableVec::new();
    resolved_partition_sequences_into(
        prepared.sequences,
        &mut decompression_offsets,
        &mut compression_offsets,
        &mut resolved,
    );
    let mut bytes = Vec::new();
    let mut huffman_scratch = HuffmanBuildScratch::new();
    let mut fse_scratch = FSETableBuildScratch::new();
    let emission = append_stored_block_or_raw(
        block,
        last_block,
        context.strategy,
        context.policy,
        context.config,
        StoredBlockRef {
            literals: prepared.literals,
            sequences: &resolved,
        },
        decompression_offsets,
        context.previous_huff_table,
        Some(&mut huffman_scratch),
        Some(&mut fse_scratch),
        context.fse_tables,
        context.offset_history,
        &mut bytes,
    );
    match emission {
        PreparedBlockEmission::Compressed { new_huffman_table } => GreedyEncodedBlock {
            bytes,
            repeat_offsets: decompression_offsets,
            new_huffman_table,
        },
        PreparedBlockEmission::Raw | PreparedBlockEmission::Rle => GreedyEncodedBlock {
            bytes,
            repeat_offsets,
            new_huffman_table: None,
        },
    }
}

#[cfg(test)]
mod tests;
