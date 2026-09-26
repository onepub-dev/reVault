#[cfg(all(test, feature = "std"))]
use alloc::format;

#[cfg(test)]
use alloc::vec;

#[cfg(test)]
use alloc::vec::Vec;

#[cfg(test)]
extern crate std;

#[cfg(all(test, feature = "std"))]
use std::hint::black_box;

#[cfg(all(test, not(feature = "std")))]
impl crate::io_nostd::Read for std::fs::File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, crate::io_nostd::Error> {
        std::io::Read::read(self, buf).map_err(|e| {
            if e.get_ref().is_none() {
                crate::io_nostd::Error::from(crate::io_nostd::ErrorKind::Other)
            } else {
                crate::io_nostd::Error::new(
                    crate::io_nostd::ErrorKind::Other,
                    alloc::boxed::Box::new(e.into_inner().unwrap()),
                )
            }
        })
    }
}

#[cfg(all(test, feature = "std"))]
#[allow(dead_code)]
fn assure_error_impl() {
    // not a real test just there to throw an compiler error if Error is not derived correctly

    use crate::decoding::errors::FrameDecoderError;
    let _err: &dyn std::error::Error = &FrameDecoderError::NotYetInitialized;
}

#[cfg(all(test, feature = "std"))]
#[allow(dead_code)]
fn assure_decoder_send_sync() {
    // not a real test just there to throw an compiler error if FrameDecoder is Send + Sync

    use crate::decoding::FrameDecoder;
    let decoder = FrameDecoder::new();
    std::thread::spawn(move || {
        drop(decoder);
    });
}

#[test]
fn skippable_frame() {
    use crate::decoding::errors;
    use crate::decoding::frame;

    let mut content = vec![];
    content.extend_from_slice(&0x184D2A50u32.to_le_bytes());
    content.extend_from_slice(&300u32.to_le_bytes());
    assert_eq!(8, content.len());
    let err = frame::read_frame_header(content.as_slice());
    assert!(matches!(
        err,
        Err(errors::ReadFrameHeaderError::SkipFrame {
            magic_number: 0x184D2A50u32,
            length: 300
        })
    ));

    content.clear();
    content.extend_from_slice(&0x184D2A5Fu32.to_le_bytes());
    content.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());
    assert_eq!(8, content.len());
    let err = frame::read_frame_header(content.as_slice());
    assert!(matches!(
        err,
        Err(errors::ReadFrameHeaderError::SkipFrame {
            magic_number: 0x184D2A5Fu32,
            length: 0xFFFFFFFF
        })
    ));
}

#[cfg(test)]
#[test]
fn test_frame_header_reading() {
    use crate::decoding::frame;
    use std::fs;

    let mut content = fs::File::open("./decodecorpus_files/z000088.zst").unwrap();
    let (_frame, _) = frame::read_frame_header(&mut content).unwrap();
}

#[test]
fn test_block_header_reading() {
    use crate::decoding;
    use crate::decoding::frame;
    use std::fs;

    let mut content = fs::File::open("./decodecorpus_files/z000088.zst").unwrap();
    let (_frame, _) = frame::read_frame_header(&mut content).unwrap();

    let mut block_dec = decoding::block_decoder::new();
    let block_header = block_dec.read_block_header(&mut content).unwrap();
    let _ = block_header; //TODO validate blockheader in a smart way
}

#[test]
fn test_frame_decoder() {
    use crate::decoding::BlockDecodingStrategy;
    use crate::decoding::FrameDecoder;
    use std::fs;

    let mut content = fs::File::open("./decodecorpus_files/z000088.zst").unwrap();

    struct NullWriter(());
    impl std::io::Write for NullWriter {
        fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> Result<(), std::io::Error> {
            Ok(())
        }
    }
    let mut _null_target = NullWriter(());

    let mut frame_dec = FrameDecoder::new();
    frame_dec.reset(&mut content).unwrap();
    frame_dec
        .decode_blocks(&mut content, BlockDecodingStrategy::All)
        .unwrap();
    let decoded_size = frame_dec.decoded_size();
    let decoded = frame_dec.collect().expect("decoded bytes");
    assert_eq!(decoded_size as usize, decoded.len());
}

#[test]
fn test_decode_from_to() {
    use crate::decoding::FrameDecoder;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Read;
    let f = BufReader::new(File::open("./decodecorpus_files/z000088.zst").unwrap());
    let mut frame_dec = FrameDecoder::new();

    let content: Vec<u8> = f.bytes().map(|x| x.unwrap()).collect();

    let mut target = vec![0u8; 1024 * 1024];

    // first part
    let source1 = &content[..50 * 1024];
    let (read1, written1) = frame_dec
        .decode_from_to(source1, target.as_mut_slice())
        .unwrap();

    //second part explicitely without checksum
    let source2 = &content[read1..content.len() - 4];
    let (read2, written2) = frame_dec
        .decode_from_to(source2, &mut target[written1..])
        .unwrap();

    //must have decoded until checksum
    assert!(read1 + read2 == content.len() - 4);

    //insert checksum separatly to test that this is handled correctly
    let chksum_source = &content[read1 + read2..];
    let (read3, written3) = frame_dec
        .decode_from_to(chksum_source, &mut target[written1 + written2..])
        .unwrap();

    //this must result in these values because just the checksum was processed
    assert!(read3 == 4);
    assert!(written3 == 0);

    let read = read1 + read2 + read3;
    let written = written1 + written2;

    let result = &target.as_slice()[..written];

    if read != content.len() {
        panic!(
            "Byte counter: {} was wrong. Should be: {}",
            read,
            content.len()
        );
    }

    match frame_dec.get_checksum_from_data() {
        Some(chksum) => {
            #[cfg(feature = "hash")]
            if frame_dec.get_calculated_checksum().unwrap() != chksum {
                std::println!(
                    "Checksum did not match! From data: {}, calculated while decoding: {}\n",
                    chksum,
                    frame_dec.get_calculated_checksum().unwrap()
                );
            } else {
                std::println!("Checksums are ok!\n");
            }
            #[cfg(not(feature = "hash"))]
            std::println!(
                "Checksum feature not enabled, skipping. From data: {}\n",
                chksum
            );
        }
        None => std::println!("No checksums to test\n"),
    }

    let original_f = BufReader::new(File::open("./decodecorpus_files/z000088").unwrap());
    let original: Vec<u8> = original_f.bytes().map(|x| x.unwrap()).collect();

    if original.len() != result.len() {
        panic!(
            "Result has wrong length: {}, should be: {}",
            result.len(),
            original.len()
        );
    }

    let mut counter = 0;
    let min = if original.len() < result.len() {
        original.len()
    } else {
        result.len()
    };
    for idx in 0..min {
        if original[idx] != result[idx] {
            counter += 1;
            //std::println!(
            //    "Original {:3} not equal to result {:3} at byte: {}",
            //    original[idx], result[idx], idx,
            //);
        }
    }
    if counter > 0 {
        panic!("Result differs in at least {} bytes from original", counter);
    }
}

#[test]
fn test_specific_file() {
    use crate::decoding::BlockDecodingStrategy;
    use crate::decoding::FrameDecoder;
    use std::fs;
    use std::io::BufReader;
    use std::io::Read;

    let path = "./decodecorpus_files/z000068.zst";
    let mut content = fs::File::open(path).unwrap();

    struct NullWriter(());
    impl std::io::Write for NullWriter {
        fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> Result<(), std::io::Error> {
            Ok(())
        }
    }
    let mut _null_target = NullWriter(());

    let mut frame_dec = FrameDecoder::new();
    frame_dec.reset(&mut content).unwrap();
    frame_dec
        .decode_blocks(&mut content, BlockDecodingStrategy::All)
        .unwrap();
    let result = frame_dec.collect().unwrap();

    let original_f = BufReader::new(fs::File::open("./decodecorpus_files/z000088").unwrap());
    let original: Vec<u8> = original_f.bytes().map(|x| x.unwrap()).collect();

    std::println!("Results for file: {path}");

    if original.len() != result.len() {
        std::println!(
            "Result has wrong length: {}, should be: {}",
            result.len(),
            original.len()
        );
    }

    let mut counter = 0;
    let min = if original.len() < result.len() {
        original.len()
    } else {
        result.len()
    };
    for idx in 0..min {
        if original[idx] != result[idx] {
            counter += 1;
            //std::println!(
            //    "Original {:3} not equal to result {:3} at byte: {}",
            //    original[idx], result[idx], idx,
            //);
        }
    }
    if counter > 0 {
        std::println!("Result differs in at least {counter} bytes from original");
    }
}

#[test]
#[cfg(feature = "std")]
fn test_streaming() {
    use std::fs;
    use std::io::BufReader;
    use std::io::Read;

    let mut content = fs::File::open("./decodecorpus_files/z000088.zst").unwrap();
    let mut stream = crate::decoding::StreamingDecoder::new(&mut content).unwrap();

    let mut result = Vec::new();
    Read::read_to_end(&mut stream, &mut result).unwrap();

    let original_f = BufReader::new(fs::File::open("./decodecorpus_files/z000088").unwrap());
    let original: Vec<u8> = original_f.bytes().map(|x| x.unwrap()).collect();

    if original.len() != result.len() {
        panic!(
            "Result has wrong length: {}, should be: {}",
            result.len(),
            original.len()
        );
    }

    let mut counter = 0;
    let min = if original.len() < result.len() {
        original.len()
    } else {
        result.len()
    };
    for idx in 0..min {
        if original[idx] != result[idx] {
            counter += 1;
            //std::println!(
            //    "Original {:3} not equal to result {:3} at byte: {}",
            //    original[idx], result[idx], idx,
            //);
        }
    }
    if counter > 0 {
        panic!("Result differs in at least {} bytes from original", counter);
    }

    // Test resetting to a new file while keeping the old decoder

    let mut content = fs::File::open("./decodecorpus_files/z000068.zst").unwrap();
    let mut stream = crate::decoding::StreamingDecoder::new_with_decoder(
        &mut content,
        stream.into_frame_decoder(),
    )
    .unwrap();

    let mut result = Vec::new();
    Read::read_to_end(&mut stream, &mut result).unwrap();

    let original_f = BufReader::new(fs::File::open("./decodecorpus_files/z000068").unwrap());
    let original: Vec<u8> = original_f.bytes().map(|x| x.unwrap()).collect();

    std::println!("Results for file:");

    if original.len() != result.len() {
        panic!(
            "Result has wrong length: {}, should be: {}",
            result.len(),
            original.len()
        );
    }

    let mut counter = 0;
    let min = if original.len() < result.len() {
        original.len()
    } else {
        result.len()
    };
    for idx in 0..min {
        if original[idx] != result[idx] {
            counter += 1;
            //std::println!(
            //    "Original {:3} not equal to result {:3} at byte: {}",
            //    original[idx], result[idx], idx,
            //);
        }
    }
    if counter > 0 {
        panic!("Result differs in at least {} bytes from original", counter);
    }
}

#[test]
fn test_incremental_read() {
    use crate::decoding::FrameDecoder;

    let mut unread_compressed_content =
        include_bytes!("../../decodecorpus_files/abc.txt.zst").as_slice();

    let mut frame_dec = FrameDecoder::new();
    frame_dec.reset(&mut unread_compressed_content).unwrap();

    let mut output = [0u8; 3];
    let (_, written) = frame_dec
        .decode_from_to(unread_compressed_content, &mut output)
        .unwrap();

    assert_eq!(written, 3);
    assert_eq!(output.map(char::from), ['a', 'b', 'c']);

    assert!(frame_dec.is_finished());
    let written = frame_dec.collect_to_writer(&mut &mut output[..]).unwrap();
    assert_eq!(written, 3);
    assert_eq!(output.map(char::from), ['d', 'e', 'f']);
}

#[test]
#[cfg(not(feature = "std"))]
fn test_streaming_no_std() {
    use crate::io::Read;

    let content = include_bytes!("../../decodecorpus_files/z000088.zst");
    let mut content = content.as_slice();
    let mut stream = crate::decoding::StreamingDecoder::new(&mut content).unwrap();

    let original = include_bytes!("../../decodecorpus_files/z000088");
    let mut result = vec![0; original.len()];
    Read::read_exact(&mut stream, &mut result).unwrap();

    if original.len() != result.len() {
        panic!(
            "Result has wrong length: {}, should be: {}",
            result.len(),
            original.len()
        );
    }

    let mut counter = 0;
    let min = if original.len() < result.len() {
        original.len()
    } else {
        result.len()
    };
    for idx in 0..min {
        if original[idx] != result[idx] {
            counter += 1;
            //std::println!(
            //    "Original {:3} not equal to result {:3} at byte: {}",
            //    original[idx], result[idx], idx,
            //);
        }
    }
    if counter > 0 {
        panic!("Result differs in at least {} bytes from original", counter);
    }

    // Test resetting to a new file while keeping the old decoder

    let content = include_bytes!("../../decodecorpus_files/z000068.zst");
    let mut content = content.as_slice();
    let mut stream = crate::decoding::StreamingDecoder::new_with_decoder(
        &mut content,
        stream.into_frame_decoder(),
    )
    .unwrap();

    let original = include_bytes!("../../decodecorpus_files/z000068");
    let mut result = vec![0; original.len()];
    Read::read_exact(&mut stream, &mut result).unwrap();

    std::println!("Results for file:");

    if original.len() != result.len() {
        panic!(
            "Result has wrong length: {}, should be: {}",
            result.len(),
            original.len()
        );
    }

    let mut counter = 0;
    let min = if original.len() < result.len() {
        original.len()
    } else {
        result.len()
    };
    for idx in 0..min {
        if original[idx] != result[idx] {
            counter += 1;
            //std::println!(
            //    "Original {:3} not equal to result {:3} at byte: {}",
            //    original[idx], result[idx], idx,
            //);
        }
    }
    if counter > 0 {
        panic!("Result differs in at least {} bytes from original", counter);
    }
}

#[test]
fn test_decode_all() {
    use crate::decoding::errors::FrameDecoderError;
    use crate::decoding::FrameDecoder;

    let skip_frame = |input: &mut Vec<u8>, length: usize| {
        input.extend_from_slice(&0x184D2A50u32.to_le_bytes());
        input.extend_from_slice(&(length as u32).to_le_bytes());
        input.resize(input.len() + length, 0);
    };

    let mut original = Vec::new();
    let mut input = Vec::new();

    skip_frame(&mut input, 300);
    input.extend_from_slice(include_bytes!("../../decodecorpus_files/z000089.zst"));
    original.extend_from_slice(include_bytes!("../../decodecorpus_files/z000089"));
    skip_frame(&mut input, 400);
    input.extend_from_slice(include_bytes!("../../decodecorpus_files/z000090.zst"));
    original.extend_from_slice(include_bytes!("../../decodecorpus_files/z000090"));
    skip_frame(&mut input, 500);

    let mut decoder = FrameDecoder::new();

    // decode_all with correct buffers.
    let mut output = vec![0; original.len()];
    let result = decoder.decode_all(&input, &mut output).unwrap();
    assert_eq!(result, original.len());
    assert_eq!(output, original);

    // decode_all with smaller output length.
    let mut output = vec![0; original.len() - 1];
    let result = decoder.decode_all(&input, &mut output);
    assert!(
        matches!(result, Err(FrameDecoderError::TargetTooSmall)),
        "{:?}",
        result
    );

    // decode_all with larger output length.
    let mut output = vec![0; original.len() + 1];
    let result = decoder.decode_all(&input, &mut output).unwrap();
    assert_eq!(result, original.len());
    assert_eq!(&output[..result], original);

    // decode_all with truncated regular frame.
    let mut output = vec![0; original.len()];
    let result = decoder.decode_all(&input[..input.len() - 600], &mut output);
    assert!(
        matches!(result, Err(FrameDecoderError::FailedToReadBlockBody(_))),
        "{:?}",
        result
    );

    // decode_all with truncated skip frame.
    let mut output = vec![0; original.len()];
    let result = decoder.decode_all(&input[..input.len() - 1], &mut output);
    assert!(
        matches!(result, Err(FrameDecoderError::FailedToSkipFrame)),
        "{:?}",
        result
    );

    // decode_all_to_vec with correct output capacity.
    let mut output = Vec::new();
    output.reserve_exact(original.len());
    decoder.decode_all_to_vec(&input, &mut output).unwrap();
    assert_eq!(output, original);

    // decode_all_to_vec with smaller output capacity.
    let mut output = Vec::new();
    output.reserve_exact(original.len() - 1);
    let result = decoder.decode_all_to_vec(&input, &mut output);
    assert!(
        matches!(result, Err(FrameDecoderError::TargetTooSmall)),
        "{:?}",
        result
    );

    // decode_all_to_vec with larger output capacity.
    let mut output = Vec::new();
    output.reserve_exact(original.len() + 1);
    decoder.decode_all_to_vec(&input, &mut output).unwrap();
    assert_eq!(output, original);
}

#[cfg(all(test, feature = "std"))]
fn archive_mode_name(mode: crate::blocks::sequence_section::ModeType) -> &'static str {
    use crate::blocks::sequence_section::ModeType;

    match mode {
        ModeType::Predefined => "predefined",
        ModeType::RLE => "rle",
        ModeType::FSECompressed => "fse",
        ModeType::Repeat => "repeat",
    }
}

#[cfg(all(test, feature = "std"))]
fn archive_literal_length_extra_bits(len: u32) -> usize {
    match len {
        0..=15 => 0,
        16..=23 => 1,
        24..=31 => 2,
        32..=47 => 3,
        48..=63 => 4,
        64..=127 => 6,
        128..=255 => 7,
        256..=511 => 8,
        512..=1023 => 9,
        1024..=2047 => 10,
        2048..=4095 => 11,
        4096..=8191 => 12,
        8192..=16383 => 13,
        16384..=32767 => 14,
        32768..=65535 => 15,
        65536..=131071 => 16,
        _ => unreachable!(),
    }
}

#[cfg(all(test, feature = "std"))]
fn archive_match_length_extra_bits(len: u32) -> usize {
    match len {
        3..=34 => 0,
        35..=42 => 1,
        43..=50 => 2,
        51..=66 => 3,
        67..=98 => 4,
        99..=130 => 5,
        131..=258 => 7,
        259..=514 => 8,
        515..=1026 => 9,
        1027..=2050 => 10,
        2051..=4098 => 11,
        4099..=8194 => 12,
        8195..=16386 => 13,
        16387..=32770 => 14,
        32771..=65538 => 15,
        65539..=131074 => 16,
        _ => unreachable!(),
    }
}

#[cfg(all(test, feature = "std"))]
fn archive_offset_extra_bits(offset_value: u32) -> usize {
    offset_value.ilog2() as usize
}

#[cfg(all(test, feature = "std"))]
fn archive_literal_length_code(len: u32) -> u8 {
    match len {
        0..=15 => len as u8,
        16..=17 => 16,
        18..=19 => 17,
        20..=21 => 18,
        22..=23 => 19,
        24..=27 => 20,
        28..=31 => 21,
        32..=39 => 22,
        40..=47 => 23,
        48..=63 => 24,
        64..=127 => 25,
        128..=255 => 26,
        256..=511 => 27,
        512..=1023 => 28,
        1024..=2047 => 29,
        2048..=4095 => 30,
        4096..=8191 => 31,
        8192..=16383 => 32,
        16384..=32767 => 33,
        32768..=65535 => 34,
        65536..=131071 => 35,
        _ => unreachable!(),
    }
}

#[cfg(all(test, feature = "std"))]
fn archive_match_length_code(len: u32) -> u8 {
    match len {
        3..=34 => len as u8 - 3,
        35..=36 => 32,
        37..=38 => 33,
        39..=40 => 34,
        41..=42 => 35,
        43..=46 => 36,
        47..=50 => 37,
        51..=58 => 38,
        59..=66 => 39,
        67..=82 => 40,
        83..=98 => 41,
        99..=130 => 42,
        131..=258 => 43,
        259..=514 => 44,
        515..=1026 => 45,
        1027..=2050 => 46,
        2051..=4098 => 47,
        4099..=8194 => 48,
        8195..=16386 => 49,
        16387..=32770 => 50,
        32771..=65538 => 51,
        65539..=131074 => 52,
        _ => unreachable!(),
    }
}

#[cfg(all(test, feature = "std"))]
fn archive_offset_code(offset_value: u32) -> u8 {
    offset_value.ilog2() as u8
}

#[cfg(all(test, feature = "std"))]
fn print_top_code_counts(label: &str, counts: &[usize], limit: usize) {
    let mut pairs = counts
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, count)| *count != 0)
        .collect::<Vec<_>>();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let summary = pairs
        .into_iter()
        .take(limit)
        .map(|(code, count)| format!("{code}:{count}"))
        .collect::<Vec<_>>()
        .join(" ");
    std::println!("{label} {summary}");
}

#[cfg(all(test, feature = "std"))]
fn inspect_archive(path: &std::path::Path) {
    use crate::blocks::block::BlockType;
    use crate::blocks::literals_section::{LiteralsSection, LiteralsSectionType};
    use crate::blocks::sequence_section::SequencesHeader;
    use crate::decoding::dictionary::Dictionary;
    use crate::decoding::frame;
    use crate::decoding::literals_section_decoder::decode_literals;
    use crate::decoding::scratch::DecoderScratch;
    use crate::decoding::sequence_execution::execute_sequences;
    use crate::decoding::sequence_section_decoder::decode_sequences;
    use std::env;
    use std::fs;

    let frame_bytes = fs::read(path).unwrap();
    let dictionary = env::var("RUZSTD_INSPECT_DICTIONARY").ok().map(|path| {
        let bytes = fs::read(path).unwrap();
        Dictionary::decode_dict(&bytes).unwrap()
    });
    let mut source = frame_bytes.as_slice();
    let (frame_header, _) = frame::read_frame_header(&mut source).unwrap();
    let window_size = frame_header.window_size().unwrap() as usize;
    let mut scratch = DecoderScratch::new(window_size);
    if let Some(dictionary) = dictionary.as_ref() {
        scratch.init_from_dict(dictionary);
    }
    let mut block_decoder = crate::decoding::block_decoder::new();

    std::println!(
        "FRAME path={} compressed_bytes={} window_size={} content_size={} checksum={} single_segment={}",
        path.display(),
        frame_bytes.len(),
        window_size,
        frame_header.frame_content_size(),
        frame_header.descriptor.content_checksum_flag(),
        frame_header.descriptor.single_segment_flag(),
    );

    let mut block_index = 0usize;
    let mut total_sequences = 0usize;
    let mut total_literals = 0usize;
    let mut total_match_bytes = 0usize;
    let mut total_literal_section_bytes = 0usize;
    let mut total_sequence_payload_bytes = 0usize;
    let mut total_compressed_block_bytes = 0usize;
    let mut total_decompressed_block_bytes = 0usize;
    let mut compressed_blocks = 0usize;
    let mut raw_blocks = 0usize;
    let mut rle_blocks = 0usize;
    let mut raw_block_input_bytes = 0usize;
    let mut rle_block_output_bytes = 0usize;
    let verbose_blocks = env::var("RUZSTD_INSPECT_VERBOSE_BLOCKS").ok().as_deref() == Some("1");
    let sequence_hist = env::var("RUZSTD_INSPECT_SEQUENCE_HIST").ok().as_deref() == Some("1");
    let sequence_dump_block = env::var("RUZSTD_INSPECT_SEQUENCE_DUMP_BLOCK")
        .ok()
        .and_then(|value| value.parse::<usize>().ok());
    let sequence_dump_limit = env::var("RUZSTD_INSPECT_SEQUENCE_DUMP_LIMIT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(usize::MAX);
    let huffman_weights = env::var("RUZSTD_INSPECT_HUFFMAN_WEIGHTS").ok().as_deref() == Some("1");
    let huffman_weight_dump_block = env::var("RUZSTD_INSPECT_HUFFMAN_WEIGHT_DUMP_BLOCK")
        .ok()
        .and_then(|value| value.parse::<usize>().ok());
    let huffman_table_dump_block = env::var("RUZSTD_INSPECT_HUFFMAN_TABLE_DUMP_BLOCK")
        .ok()
        .and_then(|value| value.parse::<usize>().ok());
    let literal_counts_block = env::var("RUZSTD_INSPECT_LITERAL_COUNTS_BLOCK")
        .ok()
        .and_then(|value| value.parse::<usize>().ok());
    let sequence_table_dump_block = env::var("RUZSTD_INSPECT_SEQUENCE_TABLE_DUMP_BLOCK")
        .ok()
        .and_then(|value| value.parse::<usize>().ok());

    loop {
        let (block_header, _) = block_decoder.read_block_header(&mut source).unwrap();
        let block_before = scratch.buffer.len();

        match block_header.block_type {
            BlockType::Raw => {
                raw_blocks += 1;
                let size = block_header.content_size as usize;
                raw_block_input_bytes += size;
                scratch.buffer.push(&source[..size]);
                source = &source[size..];
                let block_output = scratch.buffer.len() - block_before;
                total_decompressed_block_bytes += block_output;
                if verbose_blocks {
                    std::println!(
                        "BLOCK idx={} type=raw compressed_bytes={} decompressed_bytes={}",
                        block_index,
                        size,
                        block_output,
                    );
                }
            }
            BlockType::RLE => {
                rle_blocks += 1;
                let byte = source[0];
                source = &source[1..];
                scratch
                    .buffer
                    .extend_and_fill(byte, block_header.decompressed_size as usize);
                let block_output = scratch.buffer.len() - block_before;
                rle_block_output_bytes += block_output;
                total_decompressed_block_bytes += block_output;
                if verbose_blocks {
                    std::println!(
                        "BLOCK idx={} type=rle compressed_bytes=1 decompressed_bytes={} byte={}",
                        block_index,
                        block_output,
                        byte,
                    );
                }
            }
            BlockType::Compressed => {
                compressed_blocks += 1;
                let block_size = block_header.content_size as usize;
                total_compressed_block_bytes += block_size;
                let block_bytes = &source[..block_size];
                source = &source[block_size..];

                let mut literals = LiteralsSection::new();
                let literals_header_bytes = literals.parse_from_header(block_bytes).unwrap();
                let after_literals_header = &block_bytes[literals_header_bytes as usize..];
                let literals_payload_bytes = match literals.compressed_size {
                    Some(size) => size as usize,
                    None => match literals.ls_type {
                        LiteralsSectionType::RLE => 1,
                        LiteralsSectionType::Raw => literals.regenerated_size as usize,
                        LiteralsSectionType::Compressed | LiteralsSectionType::Treeless => {
                            unreachable!("compressed literals must carry compressed size")
                        }
                    },
                };
                let literals_payload = &after_literals_header[..literals_payload_bytes];
                let (literals_table_desc_bytes, literal_weights) =
                    if matches!(literals.ls_type, LiteralsSectionType::Compressed) {
                        let mut temp_table = crate::huff0::HuffmanTable::new();
                        let bytes = temp_table.build_decoder(literals_payload).unwrap() as usize;
                        let weights = huffman_weights.then(|| temp_table.encoder_weights());
                        (bytes, weights)
                    } else {
                        (0, None)
                    };
                let literals_stream_bytes = literals_payload_bytes
                    .saturating_sub(literals_table_desc_bytes)
                    .saturating_sub(if literals.num_streams == Some(4) {
                        6
                    } else {
                        0
                    });

                scratch.literals_buffer.clear();
                decode_literals(
                    &literals,
                    &mut scratch.huf,
                    literals_payload,
                    &mut scratch.literals_buffer,
                )
                .unwrap();

                let after_literals = &after_literals_header[literals_payload_bytes..];
                let mut sequences = SequencesHeader::new();
                let sequence_header_bytes = sequences.parse_from_header(after_literals).unwrap();
                let sequence_payload = &after_literals[sequence_header_bytes as usize..];
                let sequence_table_desc_bytes =
                    sequence_table_description_bytes(&sequences, sequence_payload);
                let sequence_stream_bytes = sequence_payload
                    .len()
                    .saturating_sub(sequence_table_desc_bytes);

                scratch.sequences.clear();
                if sequences.num_sequences != 0 {
                    decode_sequences(
                        &sequences,
                        sequence_payload,
                        &mut scratch.fse,
                        &mut scratch.sequences,
                    )
                    .unwrap();
                } else {
                    assert!(
                        sequence_payload.is_empty(),
                        "compressed block with zero sequences had trailing bytes"
                    );
                }

                let sequence_count = scratch.sequences.len();
                let match_bytes = scratch
                    .sequences
                    .iter()
                    .map(|seq| seq.ml as usize)
                    .sum::<usize>();
                let ll_extra_bits = scratch
                    .sequences
                    .iter()
                    .map(|seq| archive_literal_length_extra_bits(seq.ll))
                    .sum::<usize>();
                let ml_extra_bits = scratch
                    .sequences
                    .iter()
                    .map(|seq| archive_match_length_extra_bits(seq.ml))
                    .sum::<usize>();
                let of_extra_bits = scratch
                    .sequences
                    .iter()
                    .map(|seq| archive_offset_extra_bits(seq.of))
                    .sum::<usize>();
                let max_offset = scratch
                    .sequences
                    .iter()
                    .map(|seq| seq.of)
                    .max()
                    .unwrap_or(0);

                total_sequences += sequence_count;
                total_literals += scratch.literals_buffer.len();
                total_match_bytes += match_bytes;
                total_literal_section_bytes += literals_payload_bytes;
                total_sequence_payload_bytes += sequence_payload.len();

                if sequence_count != 0 {
                    execute_sequences(&mut scratch).unwrap();
                } else {
                    scratch.buffer.push(&scratch.literals_buffer);
                }
                let block_output = scratch.buffer.len() - block_before;
                total_decompressed_block_bytes += block_output;

                let (ll_mode, of_mode, ml_mode) = match sequences.modes {
                    Some(modes) => (
                        archive_mode_name(modes.ll_mode()),
                        archive_mode_name(modes.of_mode()),
                        archive_mode_name(modes.ml_mode()),
                    ),
                    None => ("none", "none", "none"),
                };
                let literals_kind = match literals.ls_type {
                    LiteralsSectionType::Raw => "raw",
                    LiteralsSectionType::RLE => "rle",
                    LiteralsSectionType::Compressed => "compressed",
                    LiteralsSectionType::Treeless => "treeless",
                };

                if verbose_blocks {
                    std::println!(
                        "BLOCK idx={} type=compressed compressed_bytes={} decompressed_bytes={} literals_type={} literals_regen={} literals_payload={} literals_table_desc={} literals_stream={} literals_streams={} sequence_count={} sequence_payload={} sequence_table_desc={} sequence_stream={} ll_mode={} of_mode={} ml_mode={} match_bytes={} max_offset={} ll_extra_bits={} ml_extra_bits={} of_extra_bits={}",
                        block_index,
                        block_size,
                        block_output,
                        literals_kind,
                        scratch.literals_buffer.len(),
                        literals_payload_bytes,
                        literals_table_desc_bytes,
                        literals_stream_bytes,
                        literals.num_streams.unwrap_or(0),
                        sequence_count,
                        sequence_payload.len(),
                        sequence_table_desc_bytes,
                        sequence_stream_bytes,
                        ll_mode,
                        of_mode,
                        ml_mode,
                        match_bytes,
                        max_offset,
                        ll_extra_bits,
                        ml_extra_bits,
                        of_extra_bits,
                    );
                }

                if sequence_hist {
                    let mut ll_counts = [0usize;
                        crate::blocks::sequence_section::MAX_LITERAL_LENGTH_CODE as usize + 1];
                    let mut ml_counts = [0usize;
                        crate::blocks::sequence_section::MAX_MATCH_LENGTH_CODE as usize + 1];
                    let mut of_counts =
                        [0usize; crate::blocks::sequence_section::MAX_OFFSET_CODE as usize + 1];
                    for seq in &scratch.sequences {
                        ll_counts[archive_literal_length_code(seq.ll) as usize] += 1;
                        ml_counts[archive_match_length_code(seq.ml) as usize] += 1;
                        of_counts[archive_offset_code(seq.of) as usize] += 1;
                    }
                    std::println!(
                        "SEQHIST idx={} ll_extra_bits={} ml_extra_bits={} of_extra_bits={}",
                        block_index,
                        ll_extra_bits,
                        ml_extra_bits,
                        of_extra_bits
                    );
                    print_top_code_counts("  ll_codes", &ll_counts, 8);
                    print_top_code_counts("  ml_codes", &ml_counts, 8);
                    print_top_code_counts("  of_codes", &of_counts, 8);
                }

                if sequence_table_dump_block == Some(block_index) {
                    print_sequence_table_bytes(block_index, &sequences, sequence_payload);
                }
                if let Some(weights) = literal_weights {
                    print_huffman_weight_summary(block_index, &weights);
                    if huffman_weight_dump_block == Some(block_index) {
                        print_huffman_weights(block_index, &weights);
                    }
                }
                if literal_counts_block == Some(block_index) {
                    print_literal_counts(block_index, &scratch.literals_buffer);
                }
                if huffman_table_dump_block == Some(block_index) && literals_table_desc_bytes != 0 {
                    print_huffman_table_bytes(
                        block_index,
                        &literals_payload[..literals_table_desc_bytes],
                    );
                }

                if sequence_dump_block == Some(block_index) {
                    for (sequence_idx, sequence) in scratch
                        .sequences
                        .iter()
                        .take(sequence_dump_limit)
                        .enumerate()
                    {
                        std::println!(
                            "SEQ idx={} seq={} ll={} ml={} of={}",
                            block_index,
                            sequence_idx,
                            sequence.ll,
                            sequence.ml,
                            sequence.of,
                        );
                    }
                }
            }
            BlockType::Reserved => {
                unreachable!("reserved blocks are rejected by the header parser")
            }
        }

        block_index += 1;
        if block_header.last_block {
            break;
        }
    }

    std::println!(
        "TOTAL blocks={} compressed_blocks={} raw_blocks={} rle_blocks={} compressed_block_bytes={} raw_block_input_bytes={} rle_block_output_bytes={} decompressed_block_bytes={} literal_section_bytes={} sequence_payload_bytes={} decoded_literals={} sequences={} match_bytes={}",
        block_index,
        compressed_blocks,
        raw_blocks,
        rle_blocks,
        total_compressed_block_bytes,
        raw_block_input_bytes,
        rle_block_output_bytes,
        total_decompressed_block_bytes,
        total_literal_section_bytes,
        total_sequence_payload_bytes,
        total_literals,
        total_sequences,
        total_match_bytes,
    );
}

#[cfg(all(test, feature = "std"))]
fn print_sequence_table_bytes(
    block_index: usize,
    sequences: &crate::blocks::sequence_section::SequencesHeader,
    source: &[u8],
) {
    let Some(modes) = sequences.modes else {
        return;
    };

    let mut offset = 0usize;
    offset += print_sequence_mode_table_bytes(
        block_index,
        "ll",
        modes.ll_mode(),
        &source[offset..],
        crate::blocks::sequence_section::MAX_LITERAL_LENGTH_CODE,
        9,
    );
    offset += print_sequence_mode_table_bytes(
        block_index,
        "of",
        modes.of_mode(),
        &source[offset..],
        crate::blocks::sequence_section::MAX_OFFSET_CODE,
        8,
    );
    print_sequence_mode_table_bytes(
        block_index,
        "ml",
        modes.ml_mode(),
        &source[offset..],
        crate::blocks::sequence_section::MAX_MATCH_LENGTH_CODE,
        9,
    );
}

#[cfg(all(test, feature = "std"))]
fn print_sequence_mode_table_bytes(
    block_index: usize,
    label: &str,
    mode: crate::blocks::sequence_section::ModeType,
    source: &[u8],
    max_symbol: u8,
    max_log: u8,
) -> usize {
    match mode {
        crate::blocks::sequence_section::ModeType::FSECompressed => {
            let mut table = crate::fse::FSETable::new(max_symbol);
            let len = table.build_decoder(source, max_log).unwrap();
            let bytes = &source[..len];
            std::println!(
                "SEQTABLEDUMP idx={} table={} len={} bytes={}",
                block_index,
                label,
                len,
                bytes
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<Vec<_>>()
                    .join(""),
            );
            std::println!(
                "SEQTABLEFSE idx={} table={} probs={}",
                block_index,
                label,
                table
                    .symbol_probabilities()
                    .iter()
                    .map(|probability| format!("{probability}"))
                    .collect::<Vec<_>>()
                    .join(","),
            );
            len
        }
        crate::blocks::sequence_section::ModeType::RLE => {
            std::println!(
                "SEQTABLEDUMP idx={} table={} len=1 rle={}",
                block_index,
                label,
                source[0],
            );
            1
        }
        crate::blocks::sequence_section::ModeType::Predefined => {
            std::println!(
                "SEQTABLEDUMP idx={} table={} len=0 predefined",
                block_index,
                label,
            );
            0
        }
        crate::blocks::sequence_section::ModeType::Repeat => {
            std::println!(
                "SEQTABLEDUMP idx={} table={} len=0 repeat",
                block_index,
                label,
            );
            0
        }
    }
}

#[cfg(all(test, feature = "std"))]
fn print_literal_counts(block_index: usize, literals: &[u8]) {
    let mut counts = [0usize; 256];
    let mut max_symbol = 0usize;
    for symbol in literals {
        counts[usize::from(*symbol)] += 1;
        max_symbol = max_symbol.max(usize::from(*symbol));
    }
    std::println!(
        "LITERALCOUNTS idx={} counts={}",
        block_index,
        counts[..=max_symbol]
            .iter()
            .map(|count| format!("{count}"))
            .collect::<Vec<_>>()
            .join(","),
    );
}

#[cfg(all(test, feature = "std"))]
fn print_huffman_table_bytes(block_index: usize, bytes: &[u8]) {
    let fse_header = if bytes.first().is_some_and(|header| *header < 128) {
        let mut table = crate::fse::FSETable::new(crate::huff0::MAX_MAX_NUM_BITS);
        table
            .build_decoder(&bytes[1..], 6)
            .ok()
            .map(|len| (len, table.symbol_probabilities().to_vec()))
    } else {
        None
    };
    std::println!(
        "HUFFTABLEDUMP idx={} len={} fse_header_len={} bytes={}",
        block_index,
        bytes.len(),
        fse_header
            .as_ref()
            .map(|(len, _)| format!("{len}"))
            .unwrap_or_else(|| "none".into()),
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<_>>()
            .join(""),
    );
    if let Some((_, probabilities)) = fse_header {
        std::println!(
            "HUFFTABLEFSE idx={} probs={}",
            block_index,
            probabilities
                .iter()
                .map(|probability| format!("{probability}"))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
}

#[cfg(all(test, feature = "std"))]
fn print_huffman_weights(block_index: usize, weights: &[usize]) {
    std::println!(
        "HUFFWEIGHTDUMP idx={} weights={}",
        block_index,
        weights
            .iter()
            .map(|weight| format!("{weight}"))
            .collect::<Vec<_>>()
            .join(","),
    );
}

#[cfg(all(test, feature = "std"))]
fn print_huffman_weight_summary(block_index: usize, weights: &[usize]) {
    let mut counts = [0usize; 16];
    let mut max_symbol = 0usize;
    let mut nonzero = 0usize;
    for (symbol, weight) in weights.iter().copied().enumerate() {
        if weight != 0 {
            nonzero += 1;
            max_symbol = symbol;
        }
        if weight < counts.len() {
            counts[weight] += 1;
        }
    }
    std::println!(
        "HUFFWEIGHTS idx={} symbols={} nonzero={} counts={}",
        block_index,
        max_symbol + 1,
        nonzero,
        counts
            .iter()
            .enumerate()
            .filter(|(_, count)| **count != 0)
            .map(|(weight, count)| format!("{weight}:{count}"))
            .collect::<Vec<_>>()
            .join(" "),
    );
}

#[cfg(all(test, feature = "std"))]
fn sequence_table_description_bytes(
    sequences: &crate::blocks::sequence_section::SequencesHeader,
    source: &[u8],
) -> usize {
    let Some(modes) = sequences.modes else {
        return 0;
    };

    let mut bytes_read = 0usize;
    bytes_read += sequence_mode_description_bytes(
        modes.ll_mode(),
        &source[bytes_read..],
        crate::blocks::sequence_section::MAX_LITERAL_LENGTH_CODE,
        9,
    );
    bytes_read += sequence_mode_description_bytes(
        modes.of_mode(),
        &source[bytes_read..],
        crate::blocks::sequence_section::MAX_OFFSET_CODE,
        8,
    );
    bytes_read += sequence_mode_description_bytes(
        modes.ml_mode(),
        &source[bytes_read..],
        crate::blocks::sequence_section::MAX_MATCH_LENGTH_CODE,
        9,
    );
    bytes_read
}

#[cfg(all(test, feature = "std"))]
fn sequence_mode_description_bytes(
    mode: crate::blocks::sequence_section::ModeType,
    source: &[u8],
    max_symbol: u8,
    max_log: u8,
) -> usize {
    match mode {
        crate::blocks::sequence_section::ModeType::FSECompressed => {
            let mut table = crate::fse::FSETable::new(max_symbol);
            table.build_decoder(source, max_log).unwrap()
        }
        crate::blocks::sequence_section::ModeType::RLE => 1,
        crate::blocks::sequence_section::ModeType::Predefined
        | crate::blocks::sequence_section::ModeType::Repeat => 0,
    }
}

#[test]
#[cfg(all(test, feature = "std"))]
#[ignore = "diagnostic archive inspector"]
fn inspect_archive_from_env() {
    use std::env;
    use std::path::PathBuf;

    let path = PathBuf::from(env::var("RUZSTD_INSPECT_FRAME").expect("RUZSTD_INSPECT_FRAME"));
    inspect_archive(&path);
}

#[test]
#[cfg(all(test, feature = "std"))]
#[ignore = "diagnostic profile harness"]
fn profile_level1_fixture_from_env() {
    use std::env;
    use std::path::PathBuf;

    let path = PathBuf::from(env::var("RUZSTD_PROFILE_FIXTURE").expect("RUZSTD_PROFILE_FIXTURE"));
    let loops = env::var("RUZSTD_PROFILE_LOOPS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1000);
    let data = std::fs::read(&path).expect("fixture should be readable");
    let mut total_output = 0usize;

    for _ in 0..loops {
        let compressed = crate::encoding::compress_to_vec(
            data.as_slice(),
            crate::encoding::CompressionLevel::Fastest,
        );
        total_output += compressed.len();
        black_box(compressed);
    }

    assert!(black_box(total_output) > 0);
}

pub mod bit_reader;
pub mod decode_corpus;
pub mod dict_test;
#[cfg(feature = "std")]
pub mod encode_corpus;
pub mod fuzz_regressions;

#[cfg(feature = "std")]
#[test]
fn verbose_disabled() {
    use crate::VERBOSE;
    const {
        assert!(!VERBOSE);
    }
}
