//! Worst-case compressed-size bound matching `ZSTD_COMPRESSBOUND()`.

const ZSTD_BLOCKSIZE_MAX: usize = 128 * 1024;
const ZSTD_MAX_INPUT_SIZE_64: u64 = 0xFF00_FF00_FF00_FF00;

pub(crate) fn compress_bound(src_size: usize) -> usize {
    if usize::BITS == 64 && (src_size as u64) >= ZSTD_MAX_INPUT_SIZE_64 {
        return 0;
    }

    let small_margin = if src_size < ZSTD_BLOCKSIZE_MAX {
        (ZSTD_BLOCKSIZE_MAX - src_size) >> 11
    } else {
        0
    };

    src_size
        .checked_add(src_size >> 8)
        .and_then(|bound| bound.checked_add(small_margin))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_bound_matches_c_macro_examples() {
        assert_eq!(compress_bound(0), 64);
        assert_eq!(compress_bound(1), 64);
        assert_eq!(compress_bound(1024), 1091);
        assert_eq!(compress_bound(128 * 1024), 128 * 1024 + 512);
        assert_eq!(compress_bound(256 * 1024), 256 * 1024 + 1024);
    }
}
