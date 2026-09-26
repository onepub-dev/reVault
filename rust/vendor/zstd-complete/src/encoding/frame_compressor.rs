//! Utilities and interfaces for encoding an entire frame. Allows reusing resources

use alloc::vec::Vec;
use core::convert::TryInto;
#[cfg(feature = "hash")]
use twox_hash::XxHash64;

#[cfg(feature = "hash")]
use core::hash::Hasher;

mod adaptive;
mod offset_history;

use adaptive::best_block_segment_lengths;
pub(crate) use offset_history::OffsetHistory;

use super::{
    block_header::BlockHeader, frame_header::FrameHeader, levels::*,
    match_generator::MatchGeneratorDriver, CompressionFileProfile, CompressionFileType,
    CompressionLevel, Matcher,
};
use crate::fse::fse_encoder::{
    default_ll_table, default_ml_table, default_of_table, FSETable, SharedFSETable,
};

use crate::io::{Read, Write};

/// An interface for compressing arbitrary data with the ZStandard compression algorithm.
///
/// `FrameCompressor` will generally be used by:
/// 1. Initializing a compressor by providing a buffer of data using `FrameCompressor::new()`
/// 2. Starting compression and writing that compression into a vec using `FrameCompressor::begin`
///
/// # Examples
/// ```
/// use zstd_complete::encoding::{FrameCompressor, CompressionLevel};
/// let mock_data: &[_] = &[0x1, 0x2, 0x3, 0x4];
/// let mut output = std::vec::Vec::new();
/// // Initialize a compressor.
/// let mut compressor = FrameCompressor::new(CompressionLevel::Uncompressed);
/// compressor.set_source(mock_data);
/// compressor.set_drain(&mut output);
///
/// // `compress` writes the compressed output into the provided buffer.
/// compressor.compress();
/// ```
pub struct FrameCompressor<R: Read, W: Write, M: Matcher> {
    uncompressed_data: Option<R>,
    compressed_data: Option<W>,
    compression_level: CompressionLevel,
    file_type_hint: CompressionFileType,
    file_profile_hint: CompressionFileProfile,
    state: CompressState<M>,
    #[cfg(feature = "hash")]
    hasher: XxHash64,
}

#[derive(Clone)]
pub(crate) struct FseTables {
    pub(crate) ll_default: FSETable,
    pub(crate) ll_previous: Option<SharedFSETable>,
    pub(crate) ll_repeat_valid: bool,
    pub(crate) ml_default: FSETable,
    pub(crate) ml_previous: Option<SharedFSETable>,
    pub(crate) ml_repeat_valid: bool,
    pub(crate) of_default: FSETable,
    pub(crate) of_previous: Option<SharedFSETable>,
    pub(crate) of_repeat_valid: bool,
}

#[derive(Clone)]
pub(crate) struct FseTableSnapshot {
    ll_previous: Option<SharedFSETable>,
    ll_repeat_valid: bool,
    ml_previous: Option<SharedFSETable>,
    ml_repeat_valid: bool,
    of_previous: Option<SharedFSETable>,
    of_repeat_valid: bool,
}

impl FseTables {
    pub fn new() -> Self {
        Self {
            ll_default: default_ll_table(),
            ll_previous: None,
            ll_repeat_valid: false,
            ml_default: default_ml_table(),
            ml_previous: None,
            ml_repeat_valid: false,
            of_default: default_of_table(),
            of_previous: None,
            of_repeat_valid: false,
        }
    }

    pub(crate) fn new_in(scratch: &mut crate::fse::fse_encoder::FSETableBuildScratch) -> Self {
        Self {
            ll_default: crate::fse::fse_encoder::default_ll_table_with_scratch(scratch),
            ll_previous: None,
            ll_repeat_valid: false,
            ml_default: crate::fse::fse_encoder::default_ml_table_with_scratch(scratch),
            ml_previous: None,
            ml_repeat_valid: false,
            of_default: crate::fse::fse_encoder::default_of_table_with_scratch(scratch),
            of_previous: None,
            of_repeat_valid: false,
        }
    }

    pub fn reset(&mut self) {
        self.ll_previous = None;
        self.ll_repeat_valid = false;
        self.ml_previous = None;
        self.ml_repeat_valid = false;
        self.of_previous = None;
        self.of_repeat_valid = false;
    }

    pub(crate) fn snapshot_previous(&self) -> FseTableSnapshot {
        FseTableSnapshot {
            ll_previous: self.ll_previous.clone(),
            ll_repeat_valid: self.ll_repeat_valid,
            ml_previous: self.ml_previous.clone(),
            ml_repeat_valid: self.ml_repeat_valid,
            of_previous: self.of_previous.clone(),
            of_repeat_valid: self.of_repeat_valid,
        }
    }

    pub(crate) fn restore_previous(&mut self, snapshot: FseTableSnapshot) {
        self.ll_previous = snapshot.ll_previous;
        self.ll_repeat_valid = snapshot.ll_repeat_valid;
        self.ml_previous = snapshot.ml_previous;
        self.ml_repeat_valid = snapshot.ml_repeat_valid;
        self.of_previous = snapshot.of_previous;
        self.of_repeat_valid = snapshot.of_repeat_valid;
    }

    /// C only treats a dictionary offset table as universally valid for the
    /// first source block. Later blocks can require larger offset codes.
    pub(crate) fn downgrade_offset_repeat_validity(&mut self) {
        self.of_repeat_valid = false;
    }
}

pub(crate) struct CompressState<M: Matcher> {
    pub(crate) matcher: M,
    pub(crate) last_huff_table: Option<crate::huff0::huff0_encoder::HuffmanTable>,
    pub(crate) fse_tables: FseTables,
    pub(crate) offset_history: OffsetHistory,
    pub(crate) file_type_hint: CompressionFileType,
    pub(crate) file_profile_hint: CompressionFileProfile,
}

impl<R: Read, W: Write> FrameCompressor<R, W, MatchGeneratorDriver> {
    /// Create a new `FrameCompressor`
    pub fn new(compression_level: CompressionLevel) -> Self {
        Self {
            uncompressed_data: None,
            compressed_data: None,
            compression_level,
            file_type_hint: CompressionFileType::Unknown,
            file_profile_hint: CompressionFileProfile::None,
            state: CompressState {
                matcher: MatchGeneratorDriver::new(1024 * 128, 4),
                last_huff_table: None,
                fse_tables: FseTables::new(),
                offset_history: OffsetHistory::new(),
                file_type_hint: CompressionFileType::Unknown,
                file_profile_hint: CompressionFileProfile::None,
            },
            #[cfg(feature = "hash")]
            hasher: XxHash64::with_seed(0),
        }
    }

    /// Create a new `FrameCompressor` with a coarse file-type hint.
    pub fn new_with_file_type(
        compression_level: CompressionLevel,
        file_type_hint: CompressionFileType,
    ) -> Self {
        Self::new_with_hints(
            compression_level,
            file_type_hint,
            CompressionFileProfile::None,
        )
    }

    pub(crate) fn new_with_hints(
        compression_level: CompressionLevel,
        file_type_hint: CompressionFileType,
        file_profile_hint: CompressionFileProfile,
    ) -> Self {
        let mut compressor = Self::new(compression_level);
        compressor.file_type_hint = file_type_hint;
        compressor.file_profile_hint = file_profile_hint;
        compressor
    }
}

impl<R: Read, W: Write, M: Matcher> FrameCompressor<R, W, M> {
    /// Create a new `FrameCompressor` with a custom matching algorithm implementation
    pub fn new_with_matcher(matcher: M, compression_level: CompressionLevel) -> Self {
        Self {
            uncompressed_data: None,
            compressed_data: None,
            file_type_hint: CompressionFileType::Unknown,
            file_profile_hint: CompressionFileProfile::None,
            state: CompressState {
                matcher,
                last_huff_table: None,
                fse_tables: FseTables::new(),
                offset_history: OffsetHistory::new(),
                file_type_hint: CompressionFileType::Unknown,
                file_profile_hint: CompressionFileProfile::None,
            },
            compression_level,
            #[cfg(feature = "hash")]
            hasher: XxHash64::with_seed(0),
        }
    }

    /// Before calling [FrameCompressor::compress] you need to set the source.
    ///
    /// This is the data that is compressed and written into the drain.
    pub fn set_source(&mut self, uncompressed_data: R) -> Option<R> {
        self.uncompressed_data.replace(uncompressed_data)
    }

    /// Before calling [FrameCompressor::compress] you need to set the drain.
    ///
    /// As the compressor compresses data, the drain serves as a place for the output to be writte.
    pub fn set_drain(&mut self, compressed_data: W) -> Option<W> {
        self.compressed_data.replace(compressed_data)
    }

    /// Compress the uncompressed data from the provided source as one Zstd frame and write it to the provided drain
    ///
    /// This will repeatedly call [Read::read] on the source to fill up blocks until the source returns 0 on the read call.
    /// Also [Write::write_all] will be called on the drain after each block has been encoded.
    ///
    /// To avoid endlessly encoding from a potentially endless source (like a network socket) you can use the
    /// [Read::take] function
    pub fn compress(&mut self) {
        // Clearing buffers to allow re-using of the compressor
        self.state.file_type_hint = self.file_type_hint;
        self.state.file_profile_hint = self.file_profile_hint;
        self.state.matcher.set_file_type_hint(self.file_type_hint);
        self.state
            .matcher
            .set_internal_file_profile_hint(self.file_profile_hint.internal_hint_code());
        self.state.matcher.reset(self.compression_level);
        self.state.last_huff_table = None;
        self.state.fse_tables.reset();
        self.state.offset_history = OffsetHistory::new();
        let (newest, second, third) = self.state.offset_history.as_offsets();
        self.state.matcher.set_repeat_offsets(newest, second, third);
        #[cfg(feature = "hash")]
        {
            self.hasher = XxHash64::with_seed(0);
        }
        let source = self.uncompressed_data.as_mut().unwrap();
        let drain = self.compressed_data.as_mut().unwrap();
        // As the frame is compressed, it's stored here
        let output: &mut Vec<u8> = &mut Vec::with_capacity(1024 * 130);
        // First write the frame header
        let header = FrameHeader {
            frame_content_size: None,
            single_segment: false,
            content_checksum: cfg!(feature = "hash"),
            dictionary_id: None,
            window_size: Some(self.state.matcher.window_size()),
        };
        header.serialize(output);
        // Now compress block by block
        let mut pending_byte = None;
        loop {
            // Read a single block's worth of uncompressed data from the input
            let mut uncompressed_data = self.state.matcher.get_next_space();
            let mut read_bytes = if let Some(byte) = pending_byte.take() {
                uncompressed_data[0] = byte;
                1
            } else {
                0
            };
            let mut last_block;
            'read_loop: loop {
                let new_bytes = source.read(&mut uncompressed_data[read_bytes..]).unwrap();
                if new_bytes == 0 {
                    last_block = true;
                    break 'read_loop;
                }
                read_bytes += new_bytes;
                if read_bytes == uncompressed_data.len() {
                    last_block = false;
                    break 'read_loop;
                }
            }
            if !last_block {
                let mut lookahead = [0u8; 1];
                match source.read(&mut lookahead).unwrap() {
                    0 => last_block = true,
                    1 => pending_byte = Some(lookahead[0]),
                    _ => unreachable!("single-byte read cannot return more than one byte"),
                }
            }
            uncompressed_data.resize(read_bytes, 0);
            // As we read, hash that data too
            #[cfg(feature = "hash")]
            self.hasher.write(&uncompressed_data);
            // Special handling is needed for compression of a totally empty file (why you'd want to do that, I don't know)
            if uncompressed_data.is_empty() {
                let header = BlockHeader {
                    last_block: true,
                    block_type: crate::blocks::block::BlockType::Raw,
                    block_size: 0,
                };
                // Write the header, then the block
                header.serialize(output);
                drain.write_all(output).unwrap();
                output.clear();
                break;
            }

            compress_with_level_policy(
                &mut self.state,
                self.compression_level,
                last_block,
                uncompressed_data,
                output,
            );
            drain.write_all(output).unwrap();
            output.clear();
            if last_block {
                break;
            }
        }

        // If the `hash` feature is enabled, then `content_checksum` is set to true in the header
        // and a 32 bit hash is written at the end of the data.
        #[cfg(feature = "hash")]
        {
            // Because we only have the data as a reader, we need to read all of it to calculate the checksum
            // Possible TODO: create a wrapper around self.uncompressed data that hashes the data as it's read?
            let content_checksum = self.hasher.finish();
            drain
                .write_all(&(content_checksum as u32).to_le_bytes())
                .unwrap();
        }
    }

    /// Get a mutable reference to the source
    pub fn source_mut(&mut self) -> Option<&mut R> {
        self.uncompressed_data.as_mut()
    }

    /// Get a mutable reference to the drain
    pub fn drain_mut(&mut self) -> Option<&mut W> {
        self.compressed_data.as_mut()
    }

    /// Get a reference to the source
    pub fn source(&self) -> Option<&R> {
        self.uncompressed_data.as_ref()
    }

    /// Get a reference to the drain
    pub fn drain(&self) -> Option<&W> {
        self.compressed_data.as_ref()
    }

    /// Retrieve the source
    pub fn take_source(&mut self) -> Option<R> {
        self.uncompressed_data.take()
    }

    /// Retrieve the drain
    pub fn take_drain(&mut self) -> Option<W> {
        self.compressed_data.take()
    }

    /// Before calling [FrameCompressor::compress] you can replace the matcher
    pub fn replace_matcher(&mut self, mut match_generator: M) -> M {
        core::mem::swap(&mut match_generator, &mut self.state.matcher);
        match_generator
    }

    /// Before calling [FrameCompressor::compress] you can replace the compression level
    pub fn set_compression_level(
        &mut self,
        compression_level: CompressionLevel,
    ) -> CompressionLevel {
        let old = self.compression_level;
        self.compression_level = compression_level;
        old
    }

    /// Get the current compression level
    pub fn compression_level(&self) -> CompressionLevel {
        self.compression_level
    }

    /// Replace the coarse file-type hint used to choose internal starting points.
    pub fn set_file_type_hint(
        &mut self,
        file_type_hint: CompressionFileType,
    ) -> CompressionFileType {
        let old = self.file_type_hint;
        self.file_type_hint = file_type_hint;
        old
    }
}

fn compress_with_level_policy<M: Matcher>(
    state: &mut CompressState<M>,
    level: CompressionLevel,
    last_block: bool,
    uncompressed_data: Vec<u8>,
    output: &mut Vec<u8>,
) {
    if level.is_uncompressed() {
        let header = BlockHeader {
            last_block,
            block_type: crate::blocks::block::BlockType::Raw,
            block_size: uncompressed_data.len().try_into().unwrap(),
        };
        header.serialize(output);
        output.extend_from_slice(&uncompressed_data);
    } else if level.uses_fastest_legacy_profile() {
        compress_fastest(state, last_block, uncompressed_data, output);
    } else if level.uses_best_legacy_profile() {
        compress_best_adaptive(state, level, last_block, uncompressed_data, output);
    } else {
        compress_at_level(state, level, last_block, uncompressed_data, output);
    }
}

fn compress_best_adaptive<M: Matcher>(
    state: &mut CompressState<M>,
    level: CompressionLevel,
    last_block: bool,
    uncompressed_data: Vec<u8>,
    output: &mut Vec<u8>,
) {
    let segment_lengths = best_block_segment_lengths(uncompressed_data.as_slice());
    if segment_lengths.len() == 1 {
        compress_at_level(state, level, last_block, uncompressed_data, output);
        return;
    }

    let mut remaining = uncompressed_data;
    let segment_count = segment_lengths.len();
    for (segment_idx, segment_len) in segment_lengths.into_iter().enumerate() {
        let segment_last = last_block && segment_idx + 1 == segment_count;
        let segment = if segment_idx + 1 == segment_count {
            core::mem::take(&mut remaining)
        } else {
            let tail = remaining.split_off(segment_len);
            core::mem::replace(&mut remaining, tail)
        };
        compress_at_level_without_incompressible_probe(state, level, segment_last, segment, output);
    }
}

#[cfg(test)]
mod tests;
