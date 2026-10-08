//! This module contains the decompress_literals function, used to take a
//! parsed literals header and a source and decompress it.

use super::super::blocks::literals_section::{LiteralsSection, LiteralsSectionType};
use super::scratch::HuffmanScratch;
use super::wiping::WipingBytes;
use crate::bit_io::BitReaderReversed;
use crate::common::MAX_BLOCK_SIZE;
use crate::decoding::errors::DecompressLiteralsError;
use crate::huff0::HuffmanDecoder;
use alloc::vec::Vec;

// Keep Vec-backed internal encoder tests usable without exposing a mutable Vec
// facade from decoder-owned plaintext storage.
pub(crate) trait LiteralOutput {
    fn len(&self) -> usize;
    fn reserve(&mut self, additional: usize);
    fn extend_bytes(&mut self, bytes: &[u8]);
    fn resize(&mut self, length: usize, byte: u8);
    fn push(&mut self, byte: u8);
}
impl LiteralOutput for Vec<u8> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
    fn reserve(&mut self, additional: usize) {
        Vec::reserve(self, additional);
    }
    fn extend_bytes(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
    fn resize(&mut self, length: usize, byte: u8) {
        Vec::resize(self, length, byte);
    }
    fn push(&mut self, byte: u8) {
        Vec::push(self, byte);
    }
}
impl LiteralOutput for WipingBytes {
    fn len(&self) -> usize {
        self.as_slice().len()
    }
    fn reserve(&mut self, additional: usize) {
        WipingBytes::reserve(self, additional);
    }
    fn extend_bytes(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
    fn resize(&mut self, length: usize, byte: u8) {
        WipingBytes::resize(self, length, byte);
    }
    fn push(&mut self, byte: u8) {
        WipingBytes::push(self, byte);
    }
}

fn push_literal(
    target: &mut impl LiteralOutput,
    byte: u8,
    expected: usize,
) -> Result<(), DecompressLiteralsError> {
    if target.len() >= expected {
        return Err(DecompressLiteralsError::DecodedLiteralCountMismatch {
            decoded: target.len().saturating_add(1),
            expected,
        });
    }
    target.push(byte);
    Ok(())
}

/// Decode and decompress the provided literals section into `target`, returning the number of bytes read.
pub(crate) fn decode_literals(
    section: &LiteralsSection,
    scratch: &mut HuffmanScratch,
    source: &[u8],
    target: &mut impl LiteralOutput,
) -> Result<u32, DecompressLiteralsError> {
    if section.regenerated_size > MAX_BLOCK_SIZE {
        return Err(DecompressLiteralsError::TooManyLiterals {
            declared: section.regenerated_size,
        });
    }
    let needed = match section.ls_type {
        LiteralsSectionType::Raw => section.regenerated_size as usize,
        LiteralsSectionType::RLE => 1,
        _ => section
            .compressed_size
            .ok_or(DecompressLiteralsError::MissingCompressedSize)? as usize,
    };
    if source.len() < needed {
        return Err(DecompressLiteralsError::MissingBytesForLiterals {
            got: source.len(),
            needed,
        });
    }
    match section.ls_type {
        LiteralsSectionType::Raw => {
            target.extend_bytes(&source[0..section.regenerated_size as usize]);
            Ok(section.regenerated_size)
        }
        LiteralsSectionType::RLE => {
            target.resize(target.len() + section.regenerated_size as usize, source[0]);
            Ok(1)
        }
        LiteralsSectionType::Compressed | LiteralsSectionType::Treeless => {
            let bytes_read = decompress_literals(section, scratch, source, target)?;

            //return sum of used bytes
            Ok(bytes_read)
        }
    }
}

/// Decompress the provided literals section and source into the provided `target`.
/// This function is used when the literals section is `Compressed` or `Treeless`
///
/// Returns the number of bytes read.
fn decompress_literals(
    section: &LiteralsSection,
    scratch: &mut HuffmanScratch,
    source: &[u8],
    target: &mut impl LiteralOutput,
) -> Result<u32, DecompressLiteralsError> {
    use DecompressLiteralsError as err;

    let compressed_size = section.compressed_size.ok_or(err::MissingCompressedSize)? as usize;
    let num_streams = section.num_streams.ok_or(err::MissingNumStreams)?;

    target.reserve(section.regenerated_size as usize);
    let source = &source[0..compressed_size];
    let mut bytes_read = 0;

    match section.ls_type {
        LiteralsSectionType::Compressed => {
            //read Huffman tree description
            bytes_read += scratch.table.build_decoder(source)?;
            vprintln!("Built huffman table using {} bytes", bytes_read);
        }
        LiteralsSectionType::Treeless if scratch.table.max_num_bits == 0 => {
            return Err(err::UninitializedHuffmanTable);
        }

        _ => { /* nothing to do, huffman tree has been provided by previous block */ }
    }

    let source = &source[bytes_read as usize..];

    if num_streams == 4 {
        //build jumptable
        if source.len() < 6 {
            return Err(err::MissingBytesForJumpHeader { got: source.len() });
        }
        let jump1 = source[0] as usize + ((source[1] as usize) << 8);
        let jump2 = jump1 + source[2] as usize + ((source[3] as usize) << 8);
        let jump3 = jump2 + source[4] as usize + ((source[5] as usize) << 8);
        bytes_read += 6;
        let source = &source[6..];

        if source.len() < jump3 {
            return Err(err::MissingBytesForLiterals {
                got: source.len(),
                needed: jump3,
            });
        }

        //decode 4 streams
        let stream1 = &source[..jump1];
        let stream2 = &source[jump1..jump2];
        let stream3 = &source[jump2..jump3];
        let stream4 = &source[jump3..];

        for stream in &[stream1, stream2, stream3, stream4] {
            let mut decoder = HuffmanDecoder::new(&scratch.table);
            let mut br = BitReaderReversed::new(stream);
            //skip the 0 padding at the end of the last byte of the bit stream and throw away the first 1 found
            let mut skipped_bits = 0;
            loop {
                let val = br.get_bits(1);
                skipped_bits += 1;
                if val == 1 || skipped_bits > 8 {
                    break;
                }
            }
            if skipped_bits > 8 {
                //if more than 7 bits are 0, this is not the correct end of the bitstream. Either a bug or corrupted data
                return Err(DecompressLiteralsError::ExtraPadding { skipped_bits });
            }
            decoder.init_state(&mut br);

            while br.bits_remaining() > -(scratch.table.max_num_bits as isize) {
                push_literal(
                    target,
                    decoder.decode_symbol(),
                    section.regenerated_size as usize,
                )?;
                decoder.next_state(&mut br);
            }
            if br.bits_remaining() != -(scratch.table.max_num_bits as isize) {
                return Err(DecompressLiteralsError::BitstreamReadMismatch {
                    read_til: br.bits_remaining(),
                    expected: -(scratch.table.max_num_bits as isize),
                });
            }
        }

        bytes_read += source.len() as u32;
    } else {
        //just decode the one stream
        assert!(num_streams == 1);
        let mut decoder = HuffmanDecoder::new(&scratch.table);
        let mut br = BitReaderReversed::new(source);
        let mut skipped_bits = 0;
        loop {
            let val = br.get_bits(1);
            skipped_bits += 1;
            if val == 1 || skipped_bits > 8 {
                break;
            }
        }
        if skipped_bits > 8 {
            //if more than 7 bits are 0, this is not the correct end of the bitstream. Either a bug or corrupted data
            return Err(DecompressLiteralsError::ExtraPadding { skipped_bits });
        }
        decoder.init_state(&mut br);
        while br.bits_remaining() > -(scratch.table.max_num_bits as isize) {
            push_literal(
                target,
                decoder.decode_symbol(),
                section.regenerated_size as usize,
            )?;
            decoder.next_state(&mut br);
        }
        bytes_read += source.len() as u32;
    }

    if target.len() != section.regenerated_size as usize {
        return Err(DecompressLiteralsError::DecodedLiteralCountMismatch {
            decoded: target.len(),
            expected: section.regenerated_size as usize,
        });
    }

    Ok(bytes_read)
}

#[cfg(test)]
mod bounds_tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn short_literal_sources_return_errors_before_output_changes() {
        for kind in [
            LiteralsSectionType::Raw,
            LiteralsSectionType::RLE,
            LiteralsSectionType::Compressed,
        ] {
            let mut section = LiteralsSection::new();
            section.ls_type = kind;
            section.regenerated_size = 1;
            section.compressed_size = Some(1);
            let mut scratch = HuffmanScratch::new();
            let mut output = vec![0x6d];
            assert!(matches!(
                decode_literals(&section, &mut scratch, &[], &mut output),
                Err(DecompressLiteralsError::MissingBytesForLiterals { got: 0, needed: 1 })
            ));
            assert_eq!(output, [0x6d]);
        }
    }

    #[test]
    fn excessive_literal_header_rejected_before_reserving() {
        let mut section = LiteralsSection::new();
        section.ls_type = LiteralsSectionType::RLE;
        section.regenerated_size = MAX_BLOCK_SIZE + 1;
        let mut output = Vec::new();
        assert!(matches!(
            decode_literals(&section, &mut HuffmanScratch::new(), &[0xab], &mut output),
            Err(DecompressLiteralsError::TooManyLiterals { .. })
        ));
        assert_eq!(output.capacity(), 0);
    }
}
