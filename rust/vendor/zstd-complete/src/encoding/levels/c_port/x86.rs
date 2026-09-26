//! x86-specific helpers for C-port match-finder hot paths.

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(super) fn prefetch_read<T>(ptr: *const T) {
    // SAFETY: `_mm_prefetch` is a non-faulting cache hint. Callers may pass
    // wrapping row-table addresses just like the C implementation.
    unsafe {
        use core::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
        _mm_prefetch(ptr.cast::<i8>(), _MM_HINT_T0);
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[inline(always)]
pub(super) fn row_tag_match_mask(tag_row: &[u8], tag: u8) -> u64 {
    debug_assert!(matches!(tag_row.len(), 16 | 32 | 64));

    // SAFETY: row tables are allocated as 16, 32, or 64 byte rows. Each load
    // starts at a 16-byte chunk boundary inside that row, and unaligned loads
    // mirror the C implementation's `_mm_loadu_si128()` use.
    unsafe {
        use core::arch::x86_64::{
            __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
        };

        let comparison = _mm_set1_epi8(tag as i8);
        let load_mask = |chunk_start: usize| -> u64 {
            let chunk = _mm_loadu_si128(tag_row.as_ptr().add(chunk_start).cast::<__m128i>());
            let equal = _mm_cmpeq_epi8(chunk, comparison);
            _mm_movemask_epi8(equal) as u64
        };

        match tag_row.len() {
            16 => load_mask(0),
            32 => load_mask(0) | (load_mask(16) << 16),
            64 => {
                load_mask(0) | (load_mask(16) << 16) | (load_mask(32) << 32) | (load_mask(48) << 48)
            }
            _ => unreachable!("row width is clamped to 16, 32, or 64 bytes"),
        }
    }
}

#[cfg(all(test, target_arch = "x86_64", target_feature = "sse2"))]
mod tests {
    use super::row_tag_match_mask;
    use alloc::vec::Vec;

    fn scalar_tag_mask(row: &[u8], tag: u8) -> u64 {
        row.iter().enumerate().fold(0u64, |mask, (idx, &byte)| {
            if byte == tag {
                mask | (1u64 << idx)
            } else {
                mask
            }
        })
    }

    #[test]
    fn row_tag_match_mask_matches_scalar_for_supported_widths() {
        for width in [16usize, 32, 64] {
            let mut row = Vec::with_capacity(width);
            for idx in 0..width {
                row.push((idx.wrapping_mul(29).wrapping_add(width) & 0xff) as u8);
            }
            for idx in (1..width).step_by(5) {
                row[idx] = 0x5a;
            }

            assert_eq!(row_tag_match_mask(&row, 0x5a), scalar_tag_mask(&row, 0x5a));
        }
    }

    #[test]
    fn row_tag_match_mask_reports_no_matches() {
        let row = [0x11u8; 64];

        assert_eq!(row_tag_match_mask(&row, 0x5a), 0);
    }
}
