use core::{convert::TryFrom, fmt};

/// A validated Zstandard compression level.
///
/// Positive levels 1 through 22 select progressively stronger compression.
/// Negative levels select the Zstandard fast strategy, with the absolute value
/// controlling the acceleration step.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CompressionLevel(i32);

impl CompressionLevel {
    /// Emit raw blocks without compression.
    pub const UNCOMPRESSED: Self = Self(0);
    /// Fast compression, corresponding to level 1.
    pub const FASTEST: Self = Self(1);
    /// The recommended default, corresponding to level 3.
    pub const DEFAULT: Self = Self(3);
    /// A stronger general-purpose setting, corresponding to level 7.
    pub const BETTER: Self = Self(7);
    /// The historical ruzstd "best" profile, corresponding to level 11.
    pub const BEST: Self = Self(11);
    /// The strongest supported standard level.
    pub const MAXIMUM: Self = Self(22);

    // Source-compatible aliases for the enum variants exposed by ruzstd 0.9.
    #[allow(non_upper_case_globals)]
    pub const Uncompressed: Self = Self::UNCOMPRESSED;
    #[allow(non_upper_case_globals)]
    pub const Fastest: Self = Self::FASTEST;
    #[allow(non_upper_case_globals)]
    pub const Default: Self = Self::DEFAULT;
    #[allow(non_upper_case_globals)]
    pub const Better: Self = Self::BETTER;
    #[allow(non_upper_case_globals)]
    pub const Best: Self = Self::BEST;

    /// Largest supported fast-strategy acceleration step.
    pub const MAX_FAST_ACCELERATION: u32 = 128 * 1024;

    /// Creates a precise positive compression level.
    pub const fn new(level: u8) -> Result<Self, InvalidCompressionLevel> {
        if level >= 1 && level as i32 <= Self::MAXIMUM.0 {
            Ok(Self(level as i32))
        } else {
            Err(InvalidCompressionLevel {
                level: level as i32,
            })
        }
    }

    /// Returns the numeric level used by the compressor parameter table.
    /// Creates a fast level with an explicit match-search acceleration step.
    pub const fn fast(acceleration: u32) -> Result<Self, InvalidCompressionLevel> {
        if acceleration >= 1 && acceleration <= Self::MAX_FAST_ACCELERATION {
            Ok(Self(-(acceleration as i32)))
        } else {
            Err(InvalidCompressionLevel {
                level: if acceleration > i32::MAX as u32 {
                    i32::MIN
                } else {
                    -(acceleration as i32)
                },
            })
        }
    }

    /// Returns the standard numeric level. Negative values are fast levels.
    pub const fn get(self) -> i32 {
        self.0
    }

    pub(crate) const fn c_level(self) -> i32 {
        self.0
    }

    pub(crate) const fn is_uncompressed(self) -> bool {
        self.0 == Self::UNCOMPRESSED.0
    }

    pub(crate) const fn uses_fastest_legacy_profile(self) -> bool {
        self.0 == Self::FASTEST.0
    }

    pub(crate) const fn uses_best_legacy_profile(self) -> bool {
        self.0 >= Self::BEST.0
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl TryFrom<i32> for CompressionLevel {
    type Error = InvalidCompressionLevel;

    fn try_from(level: i32) -> Result<Self, Self::Error> {
        if (-(Self::MAX_FAST_ACCELERATION as i32)..=-1).contains(&level)
            || (1..=Self::MAXIMUM.0).contains(&level)
        {
            Ok(Self(level))
        } else {
            Err(InvalidCompressionLevel { level })
        }
    }
}

impl From<CompressionLevel> for i32 {
    fn from(level: CompressionLevel) -> Self {
        level.c_level()
    }
}

/// Error returned when a numeric compression level is outside the supported
/// fast or positive ranges.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidCompressionLevel {
    level: i32,
}

impl InvalidCompressionLevel {
    /// Returns the rejected numeric value.
    pub const fn level(self) -> i32 {
        self.level
    }
}

impl fmt::Display for InvalidCompressionLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "compression level {} is outside -131072..=-1 or 1..=22",
            self.level,
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidCompressionLevel {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precise_levels_validate_the_public_range() {
        assert_eq!(CompressionLevel::new(1).unwrap().get(), 1);
        assert_eq!(CompressionLevel::new(22).unwrap().get(), 22);
        assert_eq!(CompressionLevel::new(0).unwrap_err().level(), 0);
        assert_eq!(CompressionLevel::new(23).unwrap_err().level(), 23);
        assert_eq!(CompressionLevel::fast(1).unwrap().get(), -1);
        assert_eq!(CompressionLevel::fast(100).unwrap().get(), -100);
        assert_eq!(
            CompressionLevel::try_from(-131_072).unwrap().get(),
            -131_072
        );
        assert_eq!(
            CompressionLevel::try_from(-131_073).unwrap_err().level(),
            -131_073
        );
        assert_eq!(CompressionLevel::fast(0).unwrap_err().level(), 0);
    }

    #[test]
    fn named_levels_map_to_expected_values() {
        assert_eq!(CompressionLevel::FASTEST.get(), 1);
        assert_eq!(CompressionLevel::DEFAULT.get(), 3);
        assert_eq!(CompressionLevel::BETTER.get(), 7);
        assert_eq!(CompressionLevel::BEST.get(), 11);
        assert_eq!(CompressionLevel::MAXIMUM.get(), 22);
    }
}
