//! Shared state for C-style frame block emission.

use super::{
    block_policy::BlockEncodingPolicy,
    cctx_params::{CctxParameters, ParamSwitch},
    dictionary::ParsedDictionary,
    params::{CompressionParameters, Strategy},
    pre_split::FrameProgress,
    sequence_store::RepeatOffsets,
};
use crate::{
    encoding::{
        blocks::BlockCompressionConfig,
        frame_compressor::{FseTables, OffsetHistory},
    },
    fse::fse_encoder::FSETableBuildScratch,
    huff0::huff0_encoder::{HuffmanBuildScratch, HuffmanTable},
    workspace::{Arena, ArenaError, ArenaSize},
};

pub(crate) struct FrameBlockState {
    pub(crate) fse_tables: FseTables,
    pub(crate) offset_history: OffsetHistory,
    pub(crate) last_huff_table: Option<HuffmanTable>,
    pub(crate) huffman_build_scratch: HuffmanBuildScratch,
    pub(crate) fse_build_scratch: FSETableBuildScratch,
    pub(crate) repeat_offsets: RepeatOffsets,
    pub(crate) block_config: BlockCompressionConfig,
    block_size_max: usize,
    progress: FrameProgress,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BlockEncodeMode {
    TargetCompressedBlockSize { target_size: usize },
    SplitBlock,
    Normal,
}

impl BlockEncodeMode {
    pub(crate) fn from_cctx(cctx: CctxParameters) -> Self {
        if cctx.use_target_c_block_size() {
            Self::TargetCompressedBlockSize {
                target_size: cctx.target_c_block_size,
            }
        } else if cctx.post_block_splitter == ParamSwitch::Enable {
            Self::SplitBlock
        } else {
            Self::Normal
        }
    }

    pub(crate) fn split_block_enabled(self) -> bool {
        matches!(self, Self::SplitBlock)
    }

    pub(crate) fn target_c_block_size(self) -> Option<usize> {
        match self {
            Self::TargetCompressedBlockSize { target_size } => Some(target_size),
            Self::SplitBlock | Self::Normal => None,
        }
    }
}

impl FrameBlockState {
    pub(crate) fn add_workspace_size(size: &mut ArenaSize) -> Result<(), ArenaError> {
        FSETableBuildScratch::add_workspace_size(size)?;
        HuffmanBuildScratch::add_workspace_size(size)
    }

    pub(crate) fn new_in(
        arena: &mut Arena<'_>,
        params: CompressionParameters,
        block_size_max: usize,
    ) -> Result<Self, ArenaError> {
        let mut fse_build_scratch = FSETableBuildScratch::new_in(arena)?;
        let fse_tables = FseTables::new_in(&mut fse_build_scratch);
        Ok(Self {
            fse_tables,
            offset_history: OffsetHistory::new(),
            last_huff_table: None,
            huffman_build_scratch: HuffmanBuildScratch::new_in(arena)?,
            fse_build_scratch,
            repeat_offsets: RepeatOffsets::new(),
            block_config: block_config(params),
            block_size_max,
            progress: FrameProgress::new(),
        })
    }

    pub(crate) fn new(params: CompressionParameters, block_size_max: usize) -> Self {
        let block_config = block_config(params);
        Self {
            fse_tables: FseTables::new(),
            offset_history: OffsetHistory::new(),
            last_huff_table: None,
            huffman_build_scratch: HuffmanBuildScratch::new(),
            fse_build_scratch: FSETableBuildScratch::new(),
            repeat_offsets: RepeatOffsets::new(),
            block_config,
            block_size_max,
            progress: FrameProgress::new(),
        }
    }

    pub(crate) fn reset_for_frame(&mut self, params: CompressionParameters, block_size_max: usize) {
        self.reset_for_frame_with_huffman_scratch(params, block_size_max, None);
    }

    pub(crate) fn reset_for_frame_with_huffman_scratch(
        &mut self,
        params: CompressionParameters,
        block_size_max: usize,
        mut external_huffman_scratch: Option<&mut HuffmanBuildScratch>,
    ) {
        self.fse_tables.reset();
        if let Some(table) = self.last_huff_table.take() {
            external_huffman_scratch
                .as_deref_mut()
                .unwrap_or(&mut self.huffman_build_scratch)
                .recycle_table(table);
        }
        self.offset_history = OffsetHistory::new();
        self.repeat_offsets = RepeatOffsets::new();
        self.block_config = block_config(params);
        if self.huffman_build_scratch.is_workspace_backed()
            || external_huffman_scratch
                .as_deref()
                .is_some_and(HuffmanBuildScratch::is_workspace_backed)
        {
            self.block_config.prepare_for_allocation_free_workspace();
        }
        self.block_size_max = block_size_max;
        self.progress = FrameProgress::new();
    }

    pub(crate) fn with_dictionary(
        params: CompressionParameters,
        block_size_max: usize,
        dictionary: &ParsedDictionary<'_>,
    ) -> Self {
        let offsets = dictionary.repeat_offsets.as_offsets();
        let mut block_config = block_config(params);
        block_config.set_prefer_valid_repeat_huffman(
            params.strategy < Strategy::Lazy && dictionary.initial_huffman_table_is_valid(),
        );
        Self {
            fse_tables: dictionary.initial_fse_tables(),
            offset_history: OffsetHistory::from_offsets(offsets[0], offsets[1], offsets[2]),
            last_huff_table: dictionary.initial_huffman_table(),
            huffman_build_scratch: HuffmanBuildScratch::new(),
            fse_build_scratch: FSETableBuildScratch::new(),
            repeat_offsets: dictionary.repeat_offsets,
            block_config,
            block_size_max,
            progress: FrameProgress::new(),
        }
    }

    pub(crate) fn next_block_size(&mut self, remaining: &[u8], strategy: Strategy) -> usize {
        self.progress
            .next_block_size(remaining, strategy, self.block_size_max)
    }

    pub(crate) fn next_frame_chunk_block_size(
        &mut self,
        remaining: &[u8],
        _source_offset: usize,
        strategy: Strategy,
    ) -> usize {
        self.next_block_size(remaining, strategy)
    }

    pub(crate) fn block_policy(first_block: bool) -> BlockEncodingPolicy {
        if first_block {
            BlockEncodingPolicy::frame_first_block()
        } else {
            BlockEncodingPolicy::normal()
        }
    }

    pub(crate) fn record_encoded_block(
        &mut self,
        block_size: usize,
        encoded_size: usize,
        repeat_offsets: RepeatOffsets,
        new_huffman_table: Option<HuffmanTable>,
    ) {
        self.record_encoded_block_with_huffman_scratch(
            block_size,
            encoded_size,
            repeat_offsets,
            new_huffman_table,
            None,
        );
    }

    pub(crate) fn record_encoded_block_with_huffman_scratch(
        &mut self,
        block_size: usize,
        encoded_size: usize,
        repeat_offsets: RepeatOffsets,
        new_huffman_table: Option<HuffmanTable>,
        external_huffman_scratch: Option<&mut HuffmanBuildScratch>,
    ) {
        self.fse_tables.downgrade_offset_repeat_validity();
        self.repeat_offsets = repeat_offsets;
        if let Some(table) = new_huffman_table {
            let previous = self.last_huff_table.replace(table);
            if self.block_config.uses_c_fast_entropy_path() {
                if let Some(previous) = previous {
                    external_huffman_scratch
                        .unwrap_or(&mut self.huffman_build_scratch)
                        .recycle_table(previous);
                }
            }
            self.block_config.set_prefer_valid_repeat_huffman(false);
        }
        self.progress.record_block(block_size, encoded_size);
    }
}

pub(crate) fn streaming_dict_limit(
    dict_limit: usize,
    block_start: usize,
    window_log: u32,
) -> usize {
    let max_distance = 1_usize << window_log;
    // C's streaming no-dictionary path moves the previous prefix behind
    // `dictLimit` once the active block starts beyond the match window.
    if block_start.saturating_sub(dict_limit) > max_distance {
        block_start
    } else {
        dict_limit
    }
}

fn block_config(params: CompressionParameters) -> BlockCompressionConfig {
    let mut config = BlockCompressionConfig::for_c_strategy(params.strategy as u8);
    if params.strategy == Strategy::Fast && params.target_length > 0 {
        config.disable_literal_compression();
    }
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(strategy: Strategy, target_length: u32) -> CompressionParameters {
        CompressionParameters {
            window_log: 19,
            chain_log: 12,
            hash_log: 13,
            search_log: 1,
            min_match: 6,
            target_length,
            strategy,
        }
    }

    #[test]
    fn c_fast_target_length_disables_literal_compression_like_auto_mode() {
        let config = block_config(params(Strategy::Fast, 4));

        assert!(config.literal_compression_disabled());
    }

    #[test]
    fn c_fast_zero_target_length_keeps_literal_compression_enabled() {
        let config = block_config(params(Strategy::Fast, 0));

        assert!(!config.literal_compression_disabled());
    }

    #[test]
    fn c_non_fast_target_length_keeps_literal_compression_enabled() {
        let config = block_config(params(Strategy::DFast, 4));

        assert!(!config.literal_compression_disabled());
    }

    #[test]
    fn block_encode_mode_uses_target_c_block_size_before_splitter_like_c() {
        let mut cctx = CctxParameters::for_level(16, 512 * 1024, 0);
        assert_eq!(cctx.post_block_splitter, ParamSwitch::Enable);
        assert!(cctx.set_target_c_block_size(2048));

        assert_eq!(
            BlockEncodeMode::from_cctx(cctx),
            BlockEncodeMode::TargetCompressedBlockSize { target_size: 2048 }
        );
    }

    #[test]
    fn block_encode_mode_uses_splitter_when_target_c_block_size_is_disabled() {
        let cctx = CctxParameters::for_level(16, 512 * 1024, 0);

        assert_eq!(
            BlockEncodeMode::from_cctx(cctx),
            BlockEncodeMode::SplitBlock
        );
    }

    #[test]
    fn block_encode_mode_uses_normal_path_when_no_optional_mode_is_enabled() {
        let cctx = CctxParameters::for_level(15, 512 * 1024, 0);

        assert_eq!(BlockEncodeMode::from_cctx(cctx), BlockEncodeMode::Normal);
    }

    #[test]
    fn frame_chunk_size_uses_one_shot_remaining_input_like_c_api() {
        let mut state = FrameBlockState::new(params(Strategy::Greedy, 0), 1024);
        let data = alloc::vec![0_u8; 4096];

        assert_eq!(
            state.next_frame_chunk_block_size(&data, 0, Strategy::Greedy),
            1024
        );
        assert_eq!(
            state.next_frame_chunk_block_size(&data[900..], 900, Strategy::Greedy),
            1024
        );
    }

    #[test]
    fn recording_first_block_downgrades_dictionary_offset_validity_like_c() {
        let mut state = FrameBlockState::new(params(Strategy::Fast, 0), 1024);
        state.fse_tables.of_repeat_valid = true;

        state.record_encoded_block(1024, 900, RepeatOffsets::new(), None);

        assert!(!state.fse_tables.of_repeat_valid);
    }

    #[test]
    fn recording_fast_table_recycles_superseded_huffman_allocation() {
        let mut state = FrameBlockState::new(params(Strategy::Fast, 0), 1024);
        state.last_huff_table = Some(HuffmanTable::build_from_counts(&[8, 5, 3, 2]));
        let next = HuffmanTable::build_from_counts(&[9, 7, 4, 2]);

        state.record_encoded_block(1024, 900, RepeatOffsets::new(), Some(next));

        assert!(state.last_huff_table.is_some());
        assert_eq!(state.huffman_build_scratch.recycled_table_count(), 1);
    }

    #[test]
    fn optimal_frame_chunk_can_see_full_one_shot_remaining_input() {
        let mut state = FrameBlockState::new(params(Strategy::BtOpt, 0), 1024);
        let data = alloc::vec![0_u8; 4096];

        assert_eq!(
            state.next_frame_chunk_block_size(&data[900..], 900, Strategy::BtOpt),
            1024
        );
    }

    #[test]
    fn streaming_dict_limit_stays_put_within_window() {
        assert_eq!(streaming_dict_limit(0, 1 << 21, 21), 0);
    }

    #[test]
    fn streaming_dict_limit_advances_after_window_is_exceeded() {
        assert_eq!(streaming_dict_limit(0, (1 << 21) + 1, 21), (1 << 21) + 1);
    }

    #[test]
    fn streaming_dict_limit_uses_previous_limit_as_base() {
        let previous = (1 << 21) + 1;

        assert_eq!(
            streaming_dict_limit(previous, previous + (1 << 21), 21),
            previous
        );
        assert_eq!(
            streaming_dict_limit(previous, previous + (1 << 21) + 1, 21),
            previous + (1 << 21) + 1
        );
    }
}
