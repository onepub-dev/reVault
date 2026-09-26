//! Shared C-port block emission helpers.
//!
//! Match finders produce prepared sequences differently, but the final
//! raw/RLE/compressed block policy is shared by the C strategy adapters.

use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::sync::OnceLock;

use super::{
    block_policy::{
        compressed_block_is_worthwhile, should_skip_sequence_build, BlockEncodingPolicy,
    },
    params::Strategy,
};
use crate::{
    common::MAX_BLOCK_SIZE,
    encoding::{
        block_header::BlockHeader,
        blocks::{
            compress_c_prepared_block_with_stats,
            compress_c_stored_block_deferred_with_matcher_history,
            compress_c_stored_block_deferred_with_stats,
            compress_c_stored_block_with_matcher_history, compress_c_stored_block_with_stats,
            defers_stored_entropy_commit, BlockCompressionConfig, PendingStoredEntropyState,
            PreparedBlockRef, StoredBlockRef,
        },
        frame_compressor::{FseTableSnapshot, FseTables, OffsetHistory},
    },
    fse::fse_encoder::FSETableBuildScratch,
    huff0::huff0_encoder::{HuffmanBuildScratch, HuffmanTable},
};

#[cfg(feature = "std")]
static C_MATCHER_OFFSET_HANDOFF: OnceLock<bool> = OnceLock::new();

fn uses_matcher_offset_handoff(config: BlockCompressionConfig) -> bool {
    if config.is_allocation_free_workspace() {
        return true;
    }
    #[cfg(feature = "std")]
    {
        *C_MATCHER_OFFSET_HANDOFF.get_or_init(|| {
            std::env::var("RUZSTD_TUNE_C_MATCHER_OFFSET_HANDOFF")
                .map(|value| !matches!(value.as_str(), "0" | "false" | "FALSE" | "off" | "OFF"))
                .unwrap_or(true)
        })
    }
    #[cfg(not(feature = "std"))]
    {
        true
    }
}

pub(super) enum PreparedBlockEmission {
    Raw,
    Rle,
    Compressed {
        new_huffman_table: Option<HuffmanTable>,
    },
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_stored_block_or_raw(
    block: &[u8],
    last_block: bool,
    strategy: Strategy,
    policy: BlockEncodingPolicy,
    config: BlockCompressionConfig,
    stored: StoredBlockRef<'_>,
    compressed_repeat_offsets: super::sequence_store::RepeatOffsets,
    previous_huff_table: Option<&HuffmanTable>,
    huffman_scratch: Option<&mut HuffmanBuildScratch>,
    fse_build_scratch: Option<&mut FSETableBuildScratch>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    output: &mut Vec<u8>,
) -> PreparedBlockEmission {
    if (config.is_allocation_free_workspace() || defers_stored_entropy_commit())
        || fse_build_scratch
            .as_deref()
            .is_some_and(FSETableBuildScratch::has_shared_pool)
    {
        append_stored_block_or_raw_deferred(
            block,
            last_block,
            strategy,
            policy,
            config,
            stored,
            compressed_repeat_offsets,
            previous_huff_table,
            huffman_scratch,
            fse_build_scratch,
            fse_tables,
            offset_history,
            output,
        )
    } else {
        append_stored_block_or_raw_eager(
            block,
            last_block,
            strategy,
            policy,
            config,
            stored,
            compressed_repeat_offsets,
            previous_huff_table,
            huffman_scratch,
            fse_build_scratch,
            fse_tables,
            offset_history,
            output,
        )
    }
}

// Keep the deferred transaction out of the eager control path's frame. These
// paths are intentionally separate: the shared result/frame shape is hot in
// every C strategy, and carrying pending entropy state there measurably
// perturbs Greedy/Lazy generated code.
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn append_stored_block_or_raw_deferred(
    block: &[u8],
    last_block: bool,
    strategy: Strategy,
    policy: BlockEncodingPolicy,
    config: BlockCompressionConfig,
    stored: StoredBlockRef<'_>,
    compressed_repeat_offsets: super::sequence_store::RepeatOffsets,
    previous_huff_table: Option<&HuffmanTable>,
    mut huffman_scratch: Option<&mut HuffmanBuildScratch>,
    mut fse_build_scratch: Option<&mut FSETableBuildScratch>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    output: &mut Vec<u8>,
) -> PreparedBlockEmission {
    let block_start = output.len();
    output.extend_from_slice(&[0; 3]);
    let compressed_start = output.len();
    let mut pending_state = PendingStoredEntropyState::new();
    let mut compression_result = if uses_matcher_offset_handoff(config) {
        let offsets = compressed_repeat_offsets.as_offsets();
        compress_c_stored_block_deferred_with_matcher_history(
            output,
            config,
            stored,
            fse_tables,
            offset_history,
            OffsetHistory::from_offsets(offsets[0], offsets[1], offsets[2]),
            previous_huff_table,
            previous_huff_table.is_some(),
            huffman_scratch.as_deref_mut(),
            fse_build_scratch.as_deref_mut(),
            &mut pending_state,
        )
    } else {
        compress_c_stored_block_deferred_with_stats(
            output,
            config,
            stored,
            fse_tables,
            offset_history,
            previous_huff_table,
            previous_huff_table.is_some(),
            huffman_scratch.as_deref_mut(),
            fse_build_scratch.as_deref_mut(),
            &mut pending_state,
        )
    };
    let compressed_size = output.len() - compressed_start;

    if policy.allows_rle() && compressed_size < 25 && rle_byte(block).is_some() {
        output.truncate(block_start);
        pending_state.discard(fse_build_scratch.as_deref_mut());
        recycle_rejected_huffman_table(&mut compression_result, huffman_scratch.as_deref_mut());
        write_rle_block(last_block, block.len() as u32, block[0], output);
        PreparedBlockEmission::Rle
    } else if compression_result.should_emit_raw_block
        || !compressed_block_is_worthwhile(block.len(), compressed_size, strategy)
        || compressed_size > MAX_BLOCK_SIZE as usize
    {
        output.truncate(block_start);
        pending_state.discard(fse_build_scratch.as_deref_mut());
        recycle_rejected_huffman_table(&mut compression_result, huffman_scratch);
        write_raw_block(last_block, block.len() as u32, block, output);
        PreparedBlockEmission::Raw
    } else {
        pending_state.commit(fse_tables, offset_history, fse_build_scratch);
        let header = BlockHeader {
            last_block,
            block_type: crate::blocks::block::BlockType::Compressed,
            block_size: compressed_size as u32,
        };
        output[block_start..compressed_start].copy_from_slice(&header.serialize_to_bytes());
        PreparedBlockEmission::Compressed {
            new_huffman_table: compression_result.new_huffman_table,
        }
    }
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn append_stored_block_or_raw_eager(
    block: &[u8],
    last_block: bool,
    strategy: Strategy,
    policy: BlockEncodingPolicy,
    config: BlockCompressionConfig,
    stored: StoredBlockRef<'_>,
    compressed_repeat_offsets: super::sequence_store::RepeatOffsets,
    previous_huff_table: Option<&HuffmanTable>,
    mut huffman_scratch: Option<&mut HuffmanBuildScratch>,
    fse_build_scratch: Option<&mut FSETableBuildScratch>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    output: &mut Vec<u8>,
) -> PreparedBlockEmission {
    let previous_fse = fse_tables.snapshot_previous();
    let previous_offsets = *offset_history;
    let block_start = output.len();
    output.extend_from_slice(&[0; 3]);
    let compressed_start = output.len();
    let mut compression_result = if uses_matcher_offset_handoff(config) {
        let offsets = compressed_repeat_offsets.as_offsets();
        compress_c_stored_block_with_matcher_history(
            output,
            config,
            stored,
            fse_tables,
            offset_history,
            OffsetHistory::from_offsets(offsets[0], offsets[1], offsets[2]),
            previous_huff_table,
            previous_huff_table.is_some(),
            huffman_scratch.as_deref_mut(),
            fse_build_scratch,
        )
    } else {
        compress_c_stored_block_with_stats(
            output,
            config,
            stored,
            fse_tables,
            offset_history,
            previous_huff_table,
            previous_huff_table.is_some(),
            huffman_scratch.as_deref_mut(),
            fse_build_scratch,
        )
    };
    let compressed_size = output.len() - compressed_start;

    if policy.allows_rle() && compressed_size < 25 && rle_byte(block).is_some() {
        output.truncate(block_start);
        fse_tables.restore_previous(previous_fse);
        *offset_history = previous_offsets;
        recycle_rejected_huffman_table(&mut compression_result, huffman_scratch);
        write_rle_block(last_block, block.len() as u32, block[0], output);
        PreparedBlockEmission::Rle
    } else if compression_result.should_emit_raw_block
        || !compressed_block_is_worthwhile(block.len(), compressed_size, strategy)
        || compressed_size > MAX_BLOCK_SIZE as usize
    {
        output.truncate(block_start);
        fse_tables.restore_previous(previous_fse);
        *offset_history = previous_offsets;
        recycle_rejected_huffman_table(&mut compression_result, huffman_scratch);
        write_raw_block(last_block, block.len() as u32, block, output);
        PreparedBlockEmission::Raw
    } else {
        let header = BlockHeader {
            last_block,
            block_type: crate::blocks::block::BlockType::Compressed,
            block_size: compressed_size as u32,
        };
        output[block_start..compressed_start].copy_from_slice(&header.serialize_to_bytes());
        PreparedBlockEmission::Compressed {
            new_huffman_table: compression_result.new_huffman_table,
        }
    }
}

fn recycle_rejected_huffman_table(
    result: &mut crate::encoding::blocks::CompressedBlockResult,
    scratch: Option<&mut HuffmanBuildScratch>,
) {
    if let (Some(table), Some(scratch)) = (result.new_huffman_table.take(), scratch) {
        scratch.recycle_table(table);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_prepared_block_or_raw(
    block: &[u8],
    last_block: bool,
    strategy: Strategy,
    policy: BlockEncodingPolicy,
    config: BlockCompressionConfig,
    prepared: PreparedBlockRef<'_>,
    previous_fse: FseTableSnapshot,
    previous_offsets: OffsetHistory,
    previous_huff_table: Option<&HuffmanTable>,
    fse_tables: &mut FseTables,
    offset_history: &mut OffsetHistory,
    output: &mut Vec<u8>,
) -> PreparedBlockEmission {
    let block_start = output.len();
    output.extend_from_slice(&[0; 3]);
    let compressed_start = output.len();
    let compression_result = compress_c_prepared_block_with_stats(
        output,
        config,
        prepared,
        fse_tables,
        offset_history,
        previous_huff_table,
        previous_huff_table.is_some(),
    );
    let compressed_size = output.len() - compressed_start;

    if policy.allows_rle() && compressed_size < 25 && rle_byte(block).is_some() {
        output.truncate(block_start);
        fse_tables.restore_previous(previous_fse);
        *offset_history = previous_offsets;
        write_rle_block(last_block, block.len() as u32, block[0], output);
        PreparedBlockEmission::Rle
    } else if compression_result.should_emit_raw_block
        || !compressed_block_is_worthwhile(block.len(), compressed_size, strategy)
        || compressed_size > MAX_BLOCK_SIZE as usize
    {
        output.truncate(block_start);
        fse_tables.restore_previous(previous_fse);
        *offset_history = previous_offsets;
        write_raw_block(last_block, block.len() as u32, block, output);
        PreparedBlockEmission::Raw
    } else {
        let header = BlockHeader {
            last_block,
            block_type: crate::blocks::block::BlockType::Compressed,
            block_size: compressed_size as u32,
        };
        output[block_start..compressed_start].copy_from_slice(&header.serialize_to_bytes());
        PreparedBlockEmission::Compressed {
            new_huffman_table: compression_result.new_huffman_table,
        }
    }
}

pub(super) fn append_special_block(block: &[u8], last_block: bool, output: &mut Vec<u8>) -> bool {
    if block.is_empty() {
        write_raw_block(last_block, 0, block, output);
        return true;
    }

    if should_skip_sequence_build(block.len()) {
        write_raw_block(last_block, block.len() as u32, block, output);
        return true;
    }

    false
}

pub(super) fn append_raw_block(block: &[u8], last_block: bool, output: &mut Vec<u8>) {
    write_raw_block(last_block, block.len() as u32, block, output);
}

pub(super) fn append_rle_block(block: &[u8], last_block: bool, output: &mut Vec<u8>) -> bool {
    let Some(byte) = rle_byte(block) else {
        return false;
    };

    write_rle_block(last_block, block.len() as u32, byte, output);
    true
}

fn rle_byte(data: &[u8]) -> Option<u8> {
    let first = *data.first()?;
    data.iter().all(|byte| *byte == first).then_some(first)
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
mod tests {
    use super::*;
    use crate::{
        blocks::block::BlockType,
        decoding::block_decoder,
        encoding::{
            blocks::{PreparedBlock, StoredBlockRef},
            frame_compressor::{FseTables, OffsetHistory},
            levels::c_port::sequence_store::RepeatOffsets,
        },
        huff0::huff0_encoder::HuffmanTable,
    };

    #[test]
    fn rle_is_emitted_after_prepared_compression_without_confirming_state() {
        let block = alloc::vec![b'a'; 4096];
        let prepared = PreparedBlock {
            literals: block.clone(),
            sequences: Vec::new(),
        };
        let mut fse_tables = FseTables::new();
        let previous_fse = fse_tables.snapshot_previous();
        let mut offset_history = OffsetHistory::from_offsets(11, 22, 33);
        let previous_offsets = offset_history;
        let mut output = Vec::new();

        let emission = append_prepared_block_or_raw(
            &block,
            true,
            Strategy::Fast,
            BlockEncodingPolicy::normal(),
            BlockCompressionConfig::for_c_strategy(Strategy::Fast as u8),
            prepared.as_ref(),
            previous_fse,
            previous_offsets,
            None,
            &mut fse_tables,
            &mut offset_history,
            &mut output,
        );

        assert!(matches!(emission, PreparedBlockEmission::Rle));
        assert_eq!(offset_history, previous_offsets);
        assert!(fse_tables.ll_previous.is_none());
        assert!(fse_tables.ml_previous.is_none());
        assert!(fse_tables.of_previous.is_none());

        let (header, header_size) = block_decoder::new()
            .read_block_header(output.as_slice())
            .expect("RLE block header should parse");
        assert_eq!(header_size, 3);
        assert!(header.last_block);
        assert_eq!(header.block_type, BlockType::RLE);
        assert_eq!(header.decompressed_size, block.len() as u32);
        assert_eq!(header.content_size, 1);
        assert_eq!(output[3], b'a');
    }

    #[test]
    fn rejected_huffman_table_returns_to_frame_scratch() {
        let mut result = crate::encoding::blocks::CompressedBlockResult {
            new_huffman_table: Some(HuffmanTable::build_from_counts(&[8, 5, 3, 2])),
            should_emit_raw_block: true,
        };
        let mut scratch = HuffmanBuildScratch::new();

        recycle_rejected_huffman_table(&mut result, Some(&mut scratch));

        assert!(result.new_huffman_table.is_none());
        assert_eq!(scratch.recycled_table_count(), 1);
    }

    #[test]
    fn prepared_block_reuses_previous_huffman_table() {
        let mut block = alloc::vec![0; 512];
        for idx in (15..block.len()).step_by(16) {
            block[idx] = 1;
        }
        let previous_huff_table = HuffmanTable::build_from_counts(&literal_counts(&block));
        let prepared = PreparedBlock {
            literals: block.clone(),
            sequences: Vec::new(),
        };
        let mut fse_tables = FseTables::new();
        let previous_fse = fse_tables.snapshot_previous();
        let mut offset_history = OffsetHistory::new();
        let previous_offsets = offset_history;
        let mut output = Vec::new();

        let emission = append_prepared_block_or_raw(
            &block,
            true,
            Strategy::BtUltra,
            BlockEncodingPolicy::normal(),
            BlockCompressionConfig::for_c_strategy(Strategy::BtUltra as u8),
            prepared.as_ref(),
            previous_fse,
            previous_offsets,
            Some(&previous_huff_table),
            &mut fse_tables,
            &mut offset_history,
            &mut output,
        );

        assert!(matches!(emission, PreparedBlockEmission::Compressed { .. }));
        assert_eq!(
            output[3] & 0b11,
            3,
            "valid previous Huffman table should use treeless literals"
        );
    }

    #[test]
    fn rejected_stored_blocks_do_not_commit_matcher_offset_handoff() {
        let fixtures = [
            alloc::vec![b'a'; 4096],
            (0..4096)
                .map(|idx| ((idx * 73 + idx / 11) & 0xff) as u8)
                .collect(),
        ];

        for block in fixtures {
            let mut fse_tables = FseTables::new();
            let mut offset_history = OffsetHistory::from_offsets(11, 7, 3);
            let previous_offsets = offset_history;
            let mut output = Vec::new();
            let emission = append_stored_block_or_raw(
                &block,
                true,
                Strategy::Fast,
                BlockEncodingPolicy::normal(),
                BlockCompressionConfig::for_c_strategy(Strategy::Fast as u8),
                StoredBlockRef {
                    literals: &block,
                    sequences: &[],
                },
                RepeatOffsets::from_offsets(101, 67, 31),
                None,
                None,
                None,
                &mut fse_tables,
                &mut offset_history,
                &mut output,
            );

            assert!(matches!(
                emission,
                PreparedBlockEmission::Raw | PreparedBlockEmission::Rle
            ));
            assert_eq!(offset_history, previous_offsets);
            assert!(fse_tables.ll_previous.is_none());
            assert!(fse_tables.ml_previous.is_none());
            assert!(fse_tables.of_previous.is_none());
        }
    }

    fn literal_counts(literals: &[u8]) -> [usize; 256] {
        let mut counts = [0; 256];
        for &literal in literals {
            counts[usize::from(literal)] += 1;
        }
        counts
    }
}
