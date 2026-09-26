//! Block emission policy for C frame-level edge cases.

use super::params::Strategy;

pub(super) const MIN_SEQUENCE_BUILD_SRC_SIZE: usize = 7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BlockEncodingPolicy {
    allow_rle: bool,
}

impl BlockEncodingPolicy {
    pub(super) const fn normal() -> Self {
        Self { allow_rle: true }
    }

    pub(super) const fn frame_first_block() -> Self {
        Self { allow_rle: false }
    }

    pub(super) const fn allows_rle(self) -> bool {
        self.allow_rle
    }
}

pub(super) const fn min_compression_gain(src_size: usize, strategy: Strategy) -> usize {
    let min_log = if strategy as u8 >= Strategy::BtUltra as u8 {
        strategy as usize - 1
    } else {
        6
    };
    (src_size >> min_log) + 2
}

pub(super) const fn compressed_block_is_worthwhile(
    src_size: usize,
    compressed_size: usize,
    strategy: Strategy,
) -> bool {
    let max_compressed_size = src_size.saturating_sub(min_compression_gain(src_size, strategy));
    compressed_size < max_compressed_size
}

pub(super) const fn should_skip_sequence_build(src_size: usize) -> bool {
    src_size < MIN_SEQUENCE_BUILD_SRC_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_compression_gain_matches_c_strategy_formula() {
        assert_eq!(min_compression_gain(128 * 1024, Strategy::Fast), 2050);
        assert_eq!(min_compression_gain(128 * 1024, Strategy::BtOpt), 2050);
        assert_eq!(min_compression_gain(128 * 1024, Strategy::BtUltra), 1026);
        assert_eq!(min_compression_gain(128 * 1024, Strategy::BtUltra2), 514);
    }

    #[test]
    fn compressed_block_requires_c_minimum_gain() {
        let src_size = 128 * 1024;
        assert!(!compressed_block_is_worthwhile(
            src_size,
            src_size - 2050,
            Strategy::Fast
        ));
        assert!(compressed_block_is_worthwhile(
            src_size,
            src_size - 2051,
            Strategy::Fast
        ));
    }

    #[test]
    fn tiny_blocks_skip_sequence_build_like_c() {
        assert!(should_skip_sequence_build(0));
        assert!(should_skip_sequence_build(6));
        assert!(!should_skip_sequence_build(7));
    }
}
