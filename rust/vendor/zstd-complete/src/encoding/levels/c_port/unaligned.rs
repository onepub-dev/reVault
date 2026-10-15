//! Shared unaligned little-endian reads for C-port match-finder hot paths.

#[inline(always)]
pub(super) fn read16(src: &[u8], pos: usize) -> u16 {
    debug_assert!(pos + 2 <= src.len());
    // SAFETY: Callers bound positions before reading. Unaligned loads mirror
    // zstd's MEM_read16/MEM_readST match-finder hot paths.
    unsafe {
        u16::from_le(core::ptr::read_unaligned(
            src.as_ptr().add(pos).cast::<u16>(),
        ))
    }
}

#[inline(always)]
pub(super) fn read32(src: &[u8], pos: usize) -> u32 {
    debug_assert!(pos + 4 <= src.len());
    // SAFETY: Callers bound positions before reading. Unaligned loads mirror
    // zstd's MEM_read32/MEM_readST match-finder hot paths.
    unsafe {
        u32::from_le(core::ptr::read_unaligned(
            src.as_ptr().add(pos).cast::<u32>(),
        ))
    }
}

#[inline(always)]
pub(super) fn read64(src: &[u8], pos: usize) -> u64 {
    debug_assert!(pos + 8 <= src.len());
    // SAFETY: Callers bound positions before reading. Unaligned loads mirror
    // zstd's MEM_read64/MEM_readST match-finder hot paths.
    unsafe {
        u64::from_le(core::ptr::read_unaligned(
            src.as_ptr().add(pos).cast::<u64>(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{read16, read32, read64};
    use core::convert::TryInto;

    #[test]
    fn unaligned_reads_use_little_endian_order() {
        let data = [0xAA, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];

        assert_eq!(read16(&data, 1), 0x0201);
        assert_eq!(read32(&data, 1), 0x0403_0201);
        assert_eq!(read64(&data, 1), 0x0807_0605_0403_0201);
    }

    #[test]
    fn unaligned_reads_match_safe_slice_conversions_at_offsets() {
        let data = [
            0x10, 0x20, 0x31, 0x42, 0x53, 0x64, 0x75, 0x86, 0x97, 0xA8, 0xB9,
        ];

        for pos in 0..=data.len() - 2 {
            assert_eq!(
                read16(&data, pos),
                u16::from_le_bytes(data[pos..pos + 2].try_into().unwrap())
            );
        }

        for pos in 0..=data.len() - 4 {
            assert_eq!(
                read32(&data, pos),
                u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap())
            );
        }

        for pos in 0..=data.len() - 8 {
            assert_eq!(
                read64(&data, pos),
                u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap())
            );
        }
    }
}
