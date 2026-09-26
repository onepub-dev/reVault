//! C-style frame header selection for compression.

use alloc::vec::Vec;

use super::params::CompressionParameters;
use crate::encoding::frame_header::FrameHeader;

pub(super) fn write_frame_header_no_dict(
    output: &mut Vec<u8>,
    pledged_src_size: usize,
    params: CompressionParameters,
) {
    write_frame_header(output, pledged_src_size, params, None);
}

pub(super) fn write_frame_header(
    output: &mut Vec<u8>,
    pledged_src_size: usize,
    params: CompressionParameters,
    dictionary_id: Option<u32>,
) {
    let window_size = 1_u64 << params.window_log;
    let pledged_src_size = pledged_src_size as u64;
    let single_segment = window_size >= pledged_src_size;

    FrameHeader {
        frame_content_size: Some(pledged_src_size),
        single_segment,
        content_checksum: false,
        dictionary_id: dictionary_id.map(u64::from),
        window_size: (!single_segment).then_some(window_size),
    }
    .serialize(output);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoding::frame::read_frame_header;
    use crate::encoding::levels::c_port::params::Strategy;

    fn params(window_log: u32) -> CompressionParameters {
        CompressionParameters {
            window_log,
            chain_log: 16,
            hash_log: 17,
            search_log: 1,
            min_match: 5,
            target_length: 0,
            strategy: Strategy::DFast,
        }
    }

    #[test]
    fn frame_header_uses_single_segment_when_window_covers_source() {
        let mut output = Vec::new();

        write_frame_header_no_dict(&mut output, 128 * 1024, params(17));
        let (header, header_size) = read_frame_header(output.as_slice()).unwrap();

        assert!(header.descriptor.single_segment_flag());
        assert_eq!(header.frame_content_size(), 128 * 1024);
        assert_eq!(header_size, output.len() as u8);
    }

    #[test]
    fn frame_header_writes_window_when_source_exceeds_window() {
        let mut output = Vec::new();

        write_frame_header_no_dict(&mut output, 128 * 1024 + 1, params(17));
        let (header, header_size) = read_frame_header(output.as_slice()).unwrap();

        assert!(!header.descriptor.single_segment_flag());
        assert_eq!(header.frame_content_size(), 128 * 1024 + 1);
        assert_eq!(header.window_size().unwrap(), 128 * 1024);
        assert_eq!(header_size, output.len() as u8);
    }

    #[test]
    fn frame_header_writes_dictionary_id_when_present() {
        let mut output = Vec::new();

        write_frame_header(&mut output, 128 * 1024, params(17), Some(0x1234));
        let (header, header_size) = read_frame_header(output.as_slice()).unwrap();

        assert_eq!(header.dictionary_id(), Some(0x1234));
        assert_eq!(header_size, output.len() as u8);
    }
}
