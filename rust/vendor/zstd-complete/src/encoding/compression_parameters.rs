//! Typed advanced compression controls.

/// Stable strategy choices understood by the native Rust compressor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompressionStrategy {
    Fast,
    DoubleFast,
    Greedy,
    Lazy,
    Lazy2,
    BinaryTreeLazy2,
    Optimal,
    Ultra,
    Ultra2,
}

/// Controls whether each independently emitted frame declares its decoded size.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ContentSizePolicy {
    #[default]
    Include,
    Omit,
}

/// Typed long-distance matching controls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LongDistanceMatching {
    pub(crate) window_log: Option<u8>,
    pub(crate) hash_log: Option<u8>,
    pub(crate) min_match: Option<u16>,
    pub(crate) bucket_size_log: Option<u8>,
    pub(crate) hash_rate_log: Option<u8>,
}

impl LongDistanceMatching {
    pub const fn new() -> Self {
        Self {
            window_log: None,
            hash_log: None,
            min_match: None,
            bucket_size_log: None,
            hash_rate_log: None,
        }
    }

    pub const fn with_window_log(mut self, value: u8) -> Self {
        self.window_log = Some(value);
        self
    }

    pub const fn with_hash_log(mut self, value: u8) -> Self {
        self.hash_log = Some(value);
        self
    }

    pub const fn with_min_match(mut self, value: u16) -> Self {
        self.min_match = Some(value);
        self
    }

    pub const fn with_bucket_size_log(mut self, value: u8) -> Self {
        self.bucket_size_log = Some(value);
        self
    }

    pub const fn with_hash_rate_log(mut self, value: u8) -> Self {
        self.hash_rate_log = Some(value);
        self
    }

    #[cfg(feature = "std")]
    pub(crate) fn validate(self) -> Result<(), &'static str> {
        if self
            .window_log
            .is_some_and(|value| !(10..=maximum_window_log()).contains(&value))
        {
            return Err("long-distance window log is outside the platform range");
        }
        if self
            .hash_log
            .is_some_and(|value| !(6..=30).contains(&value))
        {
            return Err("long-distance hash log must be in 6..=30");
        }
        if self
            .min_match
            .is_some_and(|value| !(4..=4096).contains(&value))
        {
            return Err("long-distance minimum match must be in 4..=4096");
        }
        if self
            .bucket_size_log
            .is_some_and(|value| !(1..=8).contains(&value))
        {
            return Err("long-distance bucket-size log must be in 1..=8");
        }
        if self.hash_rate_log.is_some_and(|value| value > 30) {
            return Err("long-distance hash-rate log must be at most 30");
        }
        Ok(())
    }
}

impl Default for LongDistanceMatching {
    fn default() -> Self {
        Self::new()
    }
}

/// Optional low-level tuning layered over a [`super::CompressionLevel`] preset.
///
/// Unset fields retain the level-derived value. Validation is performed before
/// the encoder allocates its frame workspace.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CompressionTuning {
    pub(crate) window_log: Option<u8>,
    pub(crate) hash_log: Option<u8>,
    pub(crate) chain_log: Option<u8>,
    pub(crate) search_log: Option<u8>,
    pub(crate) min_match: Option<u8>,
    pub(crate) target_length: Option<u32>,
    pub(crate) strategy: Option<CompressionStrategy>,
    pub(crate) target_compressed_block_size: Option<usize>,
    pub(crate) long_distance_matching: Option<LongDistanceMatching>,
}

impl CompressionTuning {
    pub const fn new() -> Self {
        Self {
            window_log: None,
            hash_log: None,
            chain_log: None,
            search_log: None,
            min_match: None,
            target_length: None,
            strategy: None,
            target_compressed_block_size: None,
            long_distance_matching: None,
        }
    }

    pub const fn with_window_log(mut self, value: u8) -> Self {
        self.window_log = Some(value);
        self
    }
    pub const fn with_hash_log(mut self, value: u8) -> Self {
        self.hash_log = Some(value);
        self
    }
    pub const fn with_chain_log(mut self, value: u8) -> Self {
        self.chain_log = Some(value);
        self
    }
    pub const fn with_search_log(mut self, value: u8) -> Self {
        self.search_log = Some(value);
        self
    }
    pub const fn with_min_match(mut self, value: u8) -> Self {
        self.min_match = Some(value);
        self
    }
    pub const fn with_target_length(mut self, value: u32) -> Self {
        self.target_length = Some(value);
        self
    }
    pub const fn with_strategy(mut self, value: CompressionStrategy) -> Self {
        self.strategy = Some(value);
        self
    }
    pub const fn with_target_compressed_block_size(mut self, bytes: usize) -> Self {
        self.target_compressed_block_size = Some(bytes);
        self
    }
    pub const fn with_long_distance_matching(mut self, value: LongDistanceMatching) -> Self {
        self.long_distance_matching = Some(value);
        self
    }

    #[cfg(feature = "std")]
    pub(crate) fn validate(self) -> Result<(), &'static str> {
        if self
            .window_log
            .is_some_and(|value| !(10..=maximum_window_log()).contains(&value))
        {
            return Err("window log is outside the platform range");
        }
        if self
            .hash_log
            .is_some_and(|value| !(6..=30).contains(&value))
        {
            return Err("hash log must be in 6..=30");
        }
        if self
            .chain_log
            .is_some_and(|value| !(6..=30).contains(&value))
        {
            return Err("chain log must be in 6..=30");
        }
        if self
            .search_log
            .is_some_and(|value| !(1..=30).contains(&value))
        {
            return Err("search log must be in 1..=30");
        }
        if self
            .min_match
            .is_some_and(|value| !(3..=7).contains(&value))
        {
            return Err("minimum match must be in 3..=7");
        }
        if self
            .target_compressed_block_size
            .is_some_and(|value| !(1340..=128 * 1024).contains(&value))
        {
            return Err("target compressed block size must be in 1340..=131072");
        }
        if let Some(ldm) = self.long_distance_matching {
            ldm.validate()?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
const fn maximum_window_log() -> u8 {
    if usize::BITS >= 64 {
        31
    } else {
        30
    }
}
