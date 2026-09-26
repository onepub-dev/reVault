//! Conservative working-memory estimates for the bounded public encoder.

use super::{compress_bound::compress_bound, params::CompressionParameters};

const TABLE_ENTRY_BYTES: usize = core::mem::size_of::<u32>();
const FRAME_OVERHEAD: usize = 8 * 1024 * 1024;

pub(crate) fn estimated_frame_memory(level: i32, source_size: usize) -> usize {
    let output = compress_bound(source_size);
    if output == 0 {
        return usize::MAX;
    }

    if level == 0 {
        return source_size
            .saturating_add(output)
            .saturating_add(1024 * 1024);
    }

    let params = CompressionParameters::for_level(level, source_size as u64, 0);
    let hash_table = TABLE_ENTRY_BYTES.saturating_mul(1_usize << params.hash_log);
    let chain_table = TABLE_ENTRY_BYTES.saturating_mul(1_usize << params.chain_log);

    source_size
        .saturating_add(output)
        .saturating_add(hash_table)
        .saturating_add(chain_table)
        .saturating_add(FRAME_OVERHEAD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_streaming_configuration_fits_its_budget() {
        assert!(estimated_frame_memory(3, 8 * 1024 * 1024) <= 96 * 1024 * 1024);
    }

    #[test]
    fn maximum_level_requires_an_explicitly_larger_budget() {
        assert!(estimated_frame_memory(22, 8 * 1024 * 1024) > 96 * 1024 * 1024);
    }
}
