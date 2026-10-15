use super::*;

/// This is the default implementation of the `Matcher` trait. It allocates and reuses the buffers when possible.
pub struct MatchGeneratorDriver {
    pub(super) vec_pool: Vec<Vec<u8>>,
    pub(super) suffix_pool: Vec<SuffixStore>,
    pub(super) match_generator: MatchGenerator,
    pub(super) slice_size: usize,
    pub(super) suffix_store_capacity: usize,
    pub(super) adaptive_binary_no_match_probe: bool,
    pub(super) use_fast_small_dense_binary_probe: bool,
    pub(super) prefer_binary_next_position_repeat_lookahead: bool,
    pub(super) prefer_fast_binary_next_position_repeat_lookahead: bool,
    pub(super) prefer_binary_next_position_lookahead: bool,
    pub(super) prefer_oldest_first_window_probe: bool,
    pub(super) use_complementary_end_insertion: bool,
    pub(super) use_second_newest_probe: bool,
    pub(super) use_fast_binary_small_second_newest: bool,
    pub(super) use_text_repeat_pipeline: bool,
    pub(super) file_type_hint: CompressionFileType,
    pub(super) file_profile_hint: CompressionFileProfile,
}

impl MatchGeneratorDriver {
    /// slice_size says how big the slices should be that are allocated to work with
    /// max_slices_in_window says how many slices should at most be used while looking for matches
    pub(crate) fn new(slice_size: usize, max_slices_in_window: usize) -> Self {
        Self {
            vec_pool: Vec::new(),
            suffix_pool: Vec::new(),
            match_generator: MatchGenerator::new(max_slices_in_window * slice_size),
            slice_size,
            suffix_store_capacity: slice_size / SUFFIX_STORE_CAPACITY_DIVISOR,
            adaptive_binary_no_match_probe: false,
            use_fast_small_dense_binary_probe: false,
            prefer_binary_next_position_repeat_lookahead: false,
            prefer_fast_binary_next_position_repeat_lookahead: false,
            prefer_binary_next_position_lookahead: false,
            prefer_oldest_first_window_probe: false,
            use_complementary_end_insertion: false,
            use_second_newest_probe: false,
            use_fast_binary_small_second_newest: false,
            use_text_repeat_pipeline: false,
            file_type_hint: CompressionFileType::Unknown,
            file_profile_hint: CompressionFileProfile::None,
        }
    }

    #[cfg(test)]
    pub(crate) fn repeat_offsets(&self) -> (u32, u32, u32) {
        self.match_generator.offset_history.as_offsets()
    }

    #[cfg(all(test, feature = "std"))]
    pub(crate) fn diagnostics(&self) -> Ref<'_, MatcherDiagnostics> {
        self.match_generator.diagnostics.borrow()
    }
}

impl Matcher for MatchGeneratorDriver {
    fn set_file_type_hint(&mut self, file_type: CompressionFileType) {
        self.file_type_hint = file_type;
        self.match_generator.file_type_hint = file_type;
    }

    fn set_internal_file_profile_hint(&mut self, file_profile_code: u8) {
        let file_profile = CompressionFileProfile::from_internal_hint_code(file_profile_code);
        self.file_profile_hint = file_profile;
        self.match_generator.file_profile_hint = file_profile;
    }

    fn reset(&mut self, level: CompressionLevel) {
        let vec_pool = &mut self.vec_pool;
        let suffix_pool = &mut self.suffix_pool;
        let fast_window_size = self.slice_size * FASTEST_WINDOW_BLOCKS;
        self.suffix_store_capacity = Self::suffix_store_capacity(self.slice_size, level);
        self.adaptive_binary_no_match_probe = Self::adaptive_binary_no_match_probe(level);
        self.use_fast_small_dense_binary_probe = Self::use_fast_small_dense_binary_probe(level);
        self.prefer_binary_next_position_repeat_lookahead =
            Self::prefer_binary_next_position_repeat_lookahead(level);
        self.prefer_fast_binary_next_position_repeat_lookahead =
            Self::prefer_fast_binary_next_position_repeat_lookahead(level);
        self.prefer_binary_next_position_lookahead =
            Self::prefer_binary_next_position_lookahead(level);
        self.prefer_oldest_first_window_probe = Self::prefer_oldest_first_window_probe(level);
        self.use_complementary_end_insertion = Self::use_complementary_end_insertion(level);
        self.use_second_newest_probe = Self::use_second_newest_probe(level);
        self.use_fast_binary_small_second_newest = Self::use_fast_binary_small_second_newest(level);
        self.use_text_repeat_pipeline = Self::use_text_repeat_pipeline(level);

        self.match_generator.reset(|mut data, mut suffixes| {
            data.resize(data.capacity(), 0);
            vec_pool.push(data);
            suffixes.clear();
            suffix_pool.push(suffixes);
        });
        self.match_generator.set_window_sizes(
            self.slice_size * Self::window_blocks(level),
            fast_window_size,
        );
        self.match_generator.adaptive_binary_no_match_probe = self.adaptive_binary_no_match_probe;
        self.match_generator.use_fast_small_dense_binary_probe =
            self.use_fast_small_dense_binary_probe;
        self.match_generator
            .prefer_binary_next_position_repeat_lookahead =
            self.prefer_binary_next_position_repeat_lookahead;
        self.match_generator
            .prefer_fast_binary_next_position_repeat_lookahead =
            self.prefer_fast_binary_next_position_repeat_lookahead;
        self.match_generator.prefer_binary_next_position_lookahead =
            self.prefer_binary_next_position_lookahead;
        self.match_generator.prefer_oldest_first_window_probe =
            self.prefer_oldest_first_window_probe;
        self.match_generator.use_complementary_end_insertion = self.use_complementary_end_insertion;
        self.match_generator.use_second_newest_probe = self.use_second_newest_probe;
        self.match_generator.use_fast_binary_small_second_newest =
            self.use_fast_binary_small_second_newest;
        self.match_generator.use_text_repeat_pipeline = self.use_text_repeat_pipeline;
        self.match_generator.file_type_hint = self.file_type_hint;
        self.match_generator.file_profile_hint = self.file_profile_hint;
    }

    fn window_size(&self) -> u64 {
        self.match_generator.max_window_size as u64
    }

    fn get_next_space(&mut self) -> Vec<u8> {
        match self.vec_pool.pop() {
            Some(space) => space,
            None => {
                let mut space = alloc::vec![0; self.slice_size];
                space.resize(space.capacity(), 0);
                space
            }
        }
    }

    fn get_last_space(&self) -> &[u8] {
        self.match_generator.last_entry().data.as_slice()
    }

    fn commit_space(&mut self, space: Vec<u8>) {
        let vec_pool = &mut self.vec_pool;
        let suffix_capacity = self.suffix_store_capacity;
        let suffixes = match self.suffix_pool.pop() {
            Some(suffixes)
                if suffixes.capacity() == SuffixStore::normalized_capacity(suffix_capacity) =>
            {
                suffixes
            }
            _ => SuffixStore::with_capacity(suffix_capacity),
        };
        let suffix_pool = &mut self.suffix_pool;
        self.match_generator
            .add_data(space, suffixes, |mut data, mut suffixes| {
                data.resize(data.capacity(), 0);
                vec_pool.push(data);
                suffixes.clear();
                suffix_pool.push(suffixes);
            });
    }

    fn start_matching(&mut self, mut handle_sequence: impl for<'a> FnMut(Sequence<'a>)) {
        while self.match_generator.next_sequence(&mut handle_sequence) {}
    }

    fn set_repeat_offsets(&mut self, newest: u32, second: u32, third: u32) {
        self.match_generator.offset_history = OffsetHistory::from_offsets(newest, second, third);
    }

    fn skip_matching(&mut self) {
        self.match_generator.skip_matching();
    }

    fn skip_matching_for_incompressible(&mut self) {
        self.match_generator.skip_matching_for_incompressible();
    }

    fn skip_matching_for_rle(&mut self) {
        self.match_generator.skip_matching_for_rle();
    }
}

impl MatchGeneratorDriver {
    fn window_blocks(level: CompressionLevel) -> usize {
        if level.uses_best_legacy_profile() {
            BEST_WINDOW_BLOCKS
        } else {
            FASTEST_WINDOW_BLOCKS
        }
    }

    fn suffix_store_capacity(slice_size: usize, level: CompressionLevel) -> usize {
        if level.uses_best_legacy_profile() {
            slice_size * BEST_SUFFIX_STORE_CAPACITY_MULTIPLIER
        } else {
            slice_size / SUFFIX_STORE_CAPACITY_DIVISOR
        }
    }

    fn adaptive_binary_no_match_probe(level: CompressionLevel) -> bool {
        level.uses_best_legacy_profile()
    }

    fn use_fast_small_dense_binary_probe(level: CompressionLevel) -> bool {
        level.uses_fastest_legacy_profile()
    }

    fn prefer_binary_next_position_repeat_lookahead(level: CompressionLevel) -> bool {
        level.uses_best_legacy_profile()
    }

    fn prefer_fast_binary_next_position_repeat_lookahead(level: CompressionLevel) -> bool {
        level.uses_fastest_legacy_profile()
    }

    fn prefer_binary_next_position_lookahead(level: CompressionLevel) -> bool {
        level.uses_best_legacy_profile()
    }

    fn prefer_oldest_first_window_probe(level: CompressionLevel) -> bool {
        level.uses_best_legacy_profile()
    }

    fn use_complementary_end_insertion(level: CompressionLevel) -> bool {
        level.uses_best_legacy_profile()
    }

    fn use_second_newest_probe(level: CompressionLevel) -> bool {
        level.uses_best_legacy_profile()
    }

    fn use_fast_binary_small_second_newest(level: CompressionLevel) -> bool {
        level.uses_fastest_legacy_profile()
    }

    fn use_text_repeat_pipeline(level: CompressionLevel) -> bool {
        level.uses_best_legacy_profile()
    }
}
