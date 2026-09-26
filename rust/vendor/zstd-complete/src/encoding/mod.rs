//! Structures and utilities used for compressing/encoding data into the Zstd format.

pub(crate) mod block_header;
pub(crate) mod blocks;
mod file_profile;
pub(crate) mod frame_header;
pub(crate) mod match_generator;
pub(crate) mod util;

mod compression_level;
mod compression_parameters;
mod frame_compressor;
mod levels;
#[cfg(all(feature = "std", feature = "multithreading"))]
mod parallel_encoder;
#[cfg(feature = "std")]
mod streaming_encoder;
mod workspace;
pub use compression_level::{CompressionLevel, InvalidCompressionLevel};
pub use compression_parameters::{
    CompressionStrategy, CompressionTuning, ContentSizePolicy, LongDistanceMatching,
};
pub(crate) use file_profile::CompressionFileProfile;
pub use file_profile::CompressionFileType;
#[cfg(feature = "std")]
pub(crate) use file_profile::{compression_file_profile_for_path_and_data, read_file_type_sample};
#[cfg(feature = "std")]
pub use file_profile::{compression_file_type_for_path, compression_file_type_for_path_and_data};
pub use frame_compressor::FrameCompressor;
pub use match_generator::MatchGeneratorDriver;
#[cfg(all(feature = "std", feature = "multithreading"))]
pub use parallel_encoder::{encode_all_parallel, encode_parallel, ParallelEncoder};
#[cfg(feature = "std")]
pub use streaming_encoder::{
    encode, encode_all, DictionaryError, EncodeError, Encoder, EncoderDictionary, EncoderOptions,
    DEFAULT_FRAME_CHUNK_SIZE, DEFAULT_MEMORY_LIMIT,
};
pub use workspace::{EncoderWorkspace, EncoderWorkspaceError, StaticEncoderWorkspace};

use crate::io::{Read, Write};
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::path::Path;

/// Convenience function to compress some source into a target without reusing any resources of the compressor
/// ```rust
/// use zstd_complete::encoding::{compress, CompressionLevel};
/// let data: &[u8] = &[0,0,0,0,0,0,0,0,0,0,0,0];
/// let mut target = Vec::new();
/// compress(data, &mut target, CompressionLevel::Fastest);
/// ```
pub fn compress<R: Read, W: Write>(source: R, target: W, level: CompressionLevel) {
    let mut frame_enc = FrameCompressor::new(level);
    frame_enc.set_source(source);
    frame_enc.set_drain(target);
    frame_enc.compress();
}

/// Convenience function to compress some source into a target using a coarse file-type hint.
///
/// The public API stays narrow: callers provide only the requested compression level and
/// the file family. The encoder decides the internal starting point from there.
pub fn compress_with_file_type<R: Read, W: Write>(
    source: R,
    target: W,
    file_type: CompressionFileType,
    level: CompressionLevel,
) {
    let mut frame_enc =
        FrameCompressor::new_with_hints(level, file_type, CompressionFileProfile::None);
    frame_enc.set_source(source);
    frame_enc.set_drain(target);
    frame_enc.compress();
}

/// Convenience function to compress some source into a target using a path-based file-type hint.
#[cfg(feature = "std")]
pub fn compress_with_path<R: Read, W: Write>(
    mut source: R,
    target: W,
    path: &Path,
    level: CompressionLevel,
) {
    let sample = read_file_type_sample(&mut source);
    let file_type = compression_file_type_for_path_and_data(path, &sample);
    let file_profile = compression_file_profile_for_path_and_data(path, &sample);
    let mut frame_enc = FrameCompressor::new_with_hints(level, file_type, file_profile);
    frame_enc.set_source(sample.as_slice().chain(source));
    frame_enc.set_drain(target);
    frame_enc.compress();
}

/// Convenience function to compress some source into a Vec without reusing any resources of the compressor
/// ```rust
/// use zstd_complete::encoding::{compress_to_vec, CompressionLevel};
/// let data: &[u8] = &[0,0,0,0,0,0,0,0,0,0,0,0];
/// let compressed = compress_to_vec(data, CompressionLevel::Fastest);
/// ```
pub fn compress_to_vec<R: Read>(source: R, level: CompressionLevel) -> Vec<u8> {
    let mut vec = Vec::new();
    compress(source, &mut vec, level);
    vec
}

/// Compress a full source into a target using the faithful C no-dictionary level table.
///
/// This entry point accepts the same numeric level range as upstream zstd. It currently
/// targets the no-dictionary path and buffers the complete source so that the C-port
/// frame encoder can choose block strategies from the full content size.
#[cfg(any(test, feature = "c-port-validation"))]
pub fn compress_c_level<R: Read, W: Write>(mut source: R, mut target: W, level: i32) {
    let mut input = Vec::new();
    source.read_to_end(&mut input).unwrap();
    let compressed = levels::c_port::encode_frame_no_dict(&input, level);
    target.write_all(&compressed).unwrap();
}

/// Compress a full source into a Vec using the faithful C no-dictionary level table.
#[cfg(any(test, feature = "c-port-validation"))]
pub fn compress_to_vec_c_level<R: Read>(mut source: R, level: i32) -> Vec<u8> {
    let mut input = Vec::new();
    source.read_to_end(&mut input).unwrap();
    compress_slice_c_level(&input, level)
}

/// Compress a complete in-memory source into a Vec using the faithful C
/// no-dictionary level table.
///
/// This avoids the extra staging copy required by the generic `Read` entry
/// point when the caller already has the full source in memory.
#[cfg(any(test, feature = "c-port-validation"))]
pub fn compress_slice_c_level(source: &[u8], level: i32) -> Vec<u8> {
    levels::c_port::encode_frame_no_dict(source, level)
}

/// Compress a complete in-memory source with an explicit C `targetCBlockSize`.
///
/// This is a narrow validation hook for the faithful C-port path. It returns
/// `None` when the requested target size is outside C's accepted range.
#[cfg(any(test, feature = "c-port-validation"))]
pub fn compress_slice_c_level_with_target_c_block_size(
    source: &[u8],
    level: i32,
    target_c_block_size: usize,
) -> Option<Vec<u8>> {
    levels::c_port::encode_frame_no_dict_with_target_c_block_size(
        source,
        level,
        target_c_block_size,
    )
}

/// Errors returned while preparing a C-level dictionary compression.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg(any(test, feature = "c-port-validation"))]
pub enum CLevelDictionaryError {
    WrongDictionary,
    CorruptedDictionary,
}

/// A dictionary parsed and owned for repeated C-level compression.
///
/// Like upstream `ZSTD_CDict`, this binds dictionary parsing and compression
/// parameters to one numeric compression level. Reuse the value when
/// compressing multiple independent sources with the same dictionary.
#[derive(Clone)]
#[cfg(any(test, feature = "c-port-validation"))]
pub struct CLevelEncoderDictionary {
    level: i32,
    dictionary: Option<levels::c_port::PreparedDictionary>,
}

#[cfg(any(test, feature = "c-port-validation"))]
impl CLevelEncoderDictionary {
    /// Copies and prepares a dictionary for the supplied numeric C level.
    pub fn copy(dictionary: &[u8], level: i32) -> Result<Self, CLevelDictionaryError> {
        let dictionary = levels::c_port::PreparedDictionary::from_bytes(dictionary)
            .map_err(map_c_level_dictionary_error)?;
        Ok(Self { level, dictionary })
    }

    /// Returns the numeric compression level fixed at preparation time.
    pub fn level(&self) -> i32 {
        self.level
    }
}

/// Compress a complete in-memory source using a prepared C-level dictionary.
#[cfg(any(test, feature = "c-port-validation"))]
pub fn compress_slice_c_level_with_prepared_dictionary(
    source: &[u8],
    dictionary: &CLevelEncoderDictionary,
) -> Vec<u8> {
    match dictionary.dictionary.as_ref() {
        Some(prepared) => levels::c_port::encode_frame_with_prepared_dictionary(
            source,
            dictionary.level,
            prepared,
        ),
        None => levels::c_port::encode_frame_no_dict(source, dictionary.level),
    }
}

/// Compress a full source into a Vec using a prepared C-level dictionary.
#[cfg(any(test, feature = "c-port-validation"))]
pub fn compress_to_vec_c_level_with_prepared_dictionary<R: Read>(
    mut source: R,
    dictionary: &CLevelEncoderDictionary,
) -> Vec<u8> {
    let mut input = Vec::new();
    source.read_to_end(&mut input).unwrap();
    compress_slice_c_level_with_prepared_dictionary(&input, dictionary)
}

/// Compress a full source into a target using the faithful C level table and
/// C-style automatic dictionary parsing.
#[cfg(any(test, feature = "c-port-validation"))]
pub fn compress_c_level_with_dictionary<R: Read, W: Write>(
    mut source: R,
    mut target: W,
    level: i32,
    dictionary: &[u8],
) -> Result<(), CLevelDictionaryError> {
    let mut input = Vec::new();
    source.read_to_end(&mut input).unwrap();
    let compressed = levels::c_port::encode_frame_with_dictionary(&input, level, dictionary)
        .map_err(map_c_level_dictionary_error)?;
    target.write_all(&compressed).unwrap();
    Ok(())
}

/// Compress a full source into a Vec using the faithful C level table and
/// C-style automatic dictionary parsing.
#[cfg(any(test, feature = "c-port-validation"))]
pub fn compress_to_vec_c_level_with_dictionary<R: Read>(
    mut source: R,
    level: i32,
    dictionary: &[u8],
) -> Result<Vec<u8>, CLevelDictionaryError> {
    let mut input = Vec::new();
    source.read_to_end(&mut input).unwrap();
    compress_slice_c_level_with_dictionary(&input, level, dictionary)
}

/// Compress a complete in-memory source into a Vec using the faithful C level
/// table and C-style automatic dictionary parsing.
#[cfg(any(test, feature = "c-port-validation"))]
pub fn compress_slice_c_level_with_dictionary(
    source: &[u8],
    level: i32,
    dictionary: &[u8],
) -> Result<Vec<u8>, CLevelDictionaryError> {
    levels::c_port::encode_frame_with_dictionary(source, level, dictionary)
        .map_err(map_c_level_dictionary_error)
}

/// Compress a complete in-memory source with both a C-style dictionary and
/// explicit C `targetCBlockSize`.
///
/// This is a validation hook for the combined dictionary+target surface. It
/// returns `Ok(None)` when the target size is outside C's accepted range.
#[cfg(any(test, feature = "c-port-validation"))]
pub fn compress_slice_c_level_with_dictionary_and_target_c_block_size(
    source: &[u8],
    level: i32,
    dictionary: &[u8],
    target_c_block_size: usize,
) -> Result<Option<Vec<u8>>, CLevelDictionaryError> {
    levels::c_port::encode_frame_with_dictionary_and_target_c_block_size(
        source,
        level,
        dictionary,
        target_c_block_size,
    )
    .map_err(map_c_level_dictionary_error)
}

#[cfg(any(test, feature = "c-port-validation"))]
fn map_c_level_dictionary_error(
    error: levels::c_port::DictionaryParseError,
) -> CLevelDictionaryError {
    match error {
        levels::c_port::DictionaryParseError::WrongDictionary => {
            CLevelDictionaryError::WrongDictionary
        }
        levels::c_port::DictionaryParseError::CorruptedDictionary => {
            CLevelDictionaryError::CorruptedDictionary
        }
    }
}

/// Convenience function to compress some source into a Vec using a coarse file-type hint.
pub fn compress_to_vec_with_file_type<R: Read>(
    source: R,
    file_type: CompressionFileType,
    level: CompressionLevel,
) -> Vec<u8> {
    let mut vec = Vec::new();
    compress_with_file_type(source, &mut vec, file_type, level);
    vec
}

/// Convenience function to compress some source into a Vec using a path-based file-type hint.
#[cfg(feature = "std")]
pub fn compress_to_vec_with_path<R: Read>(
    source: R,
    path: &Path,
    level: CompressionLevel,
) -> Vec<u8> {
    let mut vec = Vec::new();
    compress_with_path(source, &mut vec, path, level);
    vec
}

/// Trait used by the encoder that users can use to extend the matching facilities with their own algorithm
/// making their own tradeoffs between runtime, memory usage and compression ratio
///
/// This trait operates on buffers that represent the chunks of data the matching algorithm wants to work on.
/// Each one of these buffers is referred to as a *space*. One or more of these buffers represent the window
/// the decoder will need to decode the data again.
///
/// This library asks the Matcher for a new buffer using `get_next_space` to allow reusing of allocated buffers when they are no longer part of the
/// window of data that is being used for matching.
///
/// The library fills the buffer with data that is to be compressed and commits them back to the matcher using `commit_space`.
///
/// Then it will either call `start_matching` or, if the space is deemed not worth compressing, `skip_matching` is called.
///
/// This is repeated until no more data is left to be compressed.
pub trait Matcher {
    /// Get a space where we can put data to be matched on. Will be encoded as one block. The maximum allowed size is 128 kB.
    fn get_next_space(&mut self) -> alloc::vec::Vec<u8>;
    /// Get a reference to the last commited space
    fn get_last_space(&self) -> &[u8];
    /// Commit a space to the matcher so it can be matched against
    fn commit_space(&mut self, space: alloc::vec::Vec<u8>);
    /// Just process the data in the last commited space for future matching
    fn skip_matching(&mut self);
    /// Process the data in the last commited space for future matching AND generate matches for the data
    fn start_matching(&mut self, handle_sequence: impl for<'a> FnMut(Sequence<'a>));
    /// Reset this matcher so it can be used for the next new frame
    fn reset(&mut self, level: CompressionLevel);
    /// Provide a coarse file-type hint so the matcher can choose an internal starting point.
    ///
    /// Implementations that do not care about path/extension hints can ignore this hook.
    fn set_file_type_hint(&mut self, _file_type: CompressionFileType) {}
    /// Provide a narrower internal file profile when one is known.
    ///
    /// This is encoded as a small integer so the public matcher trait does not expose the
    /// encoder's private profile enum.
    fn set_internal_file_profile_hint(&mut self, _file_profile_code: u8) {}
    /// Synchronize the matcher with the encoder's current repeat-offset history.
    ///
    /// Matchers that do not use repeat-offset history can ignore this hook.
    fn set_repeat_offsets(&mut self, _newest: u32, _second: u32, _third: u32) {}
    /// Mark the last committed space as processed without indexing it for future matches.
    ///
    /// This is intended for data that has already been classified as very unlikely to
    /// be useful match history, such as incompressible raw blocks.
    fn skip_matching_for_incompressible(&mut self) {
        self.skip_matching();
    }
    /// Mark the last committed space as processed after it was emitted as an RLE block.
    ///
    /// The default behavior preserves the existing matcher contract by indexing the block
    /// normally. Matchers can specialize this because every minimum-match suffix in an RLE
    /// block has the same key.
    fn skip_matching_for_rle(&mut self) {
        self.skip_matching();
    }
    /// The size of the window the decoder will need to execute all sequences produced by this matcher
    ///
    /// May change after a call to reset with a different compression level
    fn window_size(&self) -> u64;
}

#[derive(PartialEq, Eq, Debug)]
/// Sequences that a [`Matcher`] can produce
pub enum Sequence<'data> {
    /// Is encoded as a sequence for the decoder sequence execution.
    ///
    /// First the literals will be copied to the decoded data,
    /// then `match_len` bytes are copied from `offset` bytes back in the decoded data
    Triple {
        literals: &'data [u8],
        offset: usize,
        match_len: usize,
    },
    /// This is returned as the last sequence in a block
    ///
    /// These literals will just be copied at the end of the sequence execution by the decoder
    Literals { literals: &'data [u8] },
}

#[cfg(test)]
mod tests {
    use super::{
        compress_c_level, compress_c_level_with_dictionary, compress_slice_c_level,
        compress_slice_c_level_with_dictionary,
        compress_slice_c_level_with_dictionary_and_target_c_block_size, compress_to_vec_c_level,
        compress_to_vec_c_level_with_dictionary, CLevelDictionaryError, CompressionLevel,
    };
    use crate::decoding::{dictionary::Dictionary, FrameDecoder};
    use alloc::vec::Vec;

    #[test]
    fn compression_level_equality_is_available_for_api_comparisons() {
        assert_eq!(CompressionLevel::Fastest, CompressionLevel::Fastest);
    }

    #[test]
    fn c_level_compression_round_trips_representative_strategies() {
        let mut data = Vec::new();
        while data.len() < (crate::common::MAX_BLOCK_SIZE as usize * 2) + 2048 {
            data.extend_from_slice(b"public-c-level route=/archive status=200 bytes=1874\n");
        }

        for level in [1, 3, 5, 8, 13, 16, 18, 19, 22] {
            let encoded = compress_to_vec_c_level(data.as_slice(), level);
            assert_round_trips(&encoded, &data);
            let encoded_from_slice = compress_slice_c_level(&data, level);
            assert_eq!(encoded_from_slice, encoded);
        }
    }

    #[test]
    fn c_level_compression_writes_to_target() {
        let data = b"public-c-level-writer public-c-level-writer";
        let mut encoded = Vec::new();

        compress_c_level(data.as_slice(), &mut encoded, 16);

        assert_round_trips(&encoded, data);
    }

    #[test]
    fn c_level_dictionary_compression_round_trips_representative_strategies() {
        let dict = full_dictionary_fixture();
        let data = dictionary_payload();

        for level in [1, 3, 5, 8, 13, 16, 18, 19] {
            let encoded =
                compress_to_vec_c_level_with_dictionary(data.as_slice(), level, &dict).unwrap();
            assert_round_trips_with_dictionary(&encoded, &data, &dict);
            let encoded_from_slice =
                compress_slice_c_level_with_dictionary(&data, level, &dict).unwrap();
            assert_eq!(encoded_from_slice, encoded);
        }
    }

    #[test]
    fn c_level_dictionary_compression_writes_to_target() {
        let dict = full_dictionary_fixture();
        let data = dictionary_payload();
        let mut encoded = Vec::new();

        compress_c_level_with_dictionary(data.as_slice(), &mut encoded, 13, &dict).unwrap();

        assert_round_trips_with_dictionary(&encoded, &data, &dict);
    }

    #[test]
    fn c_level_dictionary_target_c_block_size_round_trips_optimal_strategy() {
        let dict = full_dictionary_fixture();
        let data = dictionary_payload();

        let encoded =
            compress_slice_c_level_with_dictionary_and_target_c_block_size(&data, 16, &dict, 2048)
                .unwrap()
                .expect("level 16 dictionary target mode is target-wired");

        assert_round_trips_with_dictionary(&encoded, &data, &dict);
    }

    #[test]
    fn c_level_dictionary_target_c_block_size_round_trips_hash_chain_strategy() {
        let dict = full_dictionary_fixture();
        let data = dictionary_payload();

        let encoded =
            compress_slice_c_level_with_dictionary_and_target_c_block_size(&data, 5, &dict, 2048)
                .unwrap()
                .expect("level 5 dictionary target mode is target-wired");

        assert_round_trips_with_dictionary(&encoded, &data, &dict);
    }

    #[test]
    fn c_level_dictionary_target_c_block_size_round_trips_fast_strategies() {
        let dict = full_dictionary_fixture();
        let data = dictionary_payload();

        for level in [1, 3] {
            let encoded = compress_slice_c_level_with_dictionary_and_target_c_block_size(
                &data, level, &dict, 2048,
            )
            .unwrap();

            assert_round_trips_with_dictionary(
                &encoded.expect("fast dictionary target mode is target-wired"),
                &data,
                &dict,
            );
        }
    }

    #[test]
    fn c_level_dictionary_target_c_block_size_rejects_invalid_sizes() {
        let dict = full_dictionary_fixture();
        let data = dictionary_payload();

        let encoded = compress_slice_c_level_with_dictionary_and_target_c_block_size(
            &data,
            1,
            &dict,
            128 * 1024 + 1,
        )
        .unwrap();

        assert_eq!(encoded, None);
    }

    #[test]
    fn c_level_dictionary_compression_reports_corrupt_full_dictionary() {
        let mut dict = full_dictionary_fixture();
        let repeat_offset_start = dict.len() - dictionary_content().len() - 12;
        dict[repeat_offset_start..repeat_offset_start + 4]
            .copy_from_slice(&0x00AB_CDEF_u32.to_le_bytes());

        let error = compress_to_vec_c_level_with_dictionary(b"data".as_slice(), 3, &dict)
            .expect_err("invalid full dictionary offsets must be rejected");

        assert_eq!(error, CLevelDictionaryError::CorruptedDictionary);
    }

    fn assert_round_trips(encoded: &[u8], expected: &[u8]) {
        let mut decoded = Vec::with_capacity(expected.len());
        FrameDecoder::new()
            .decode_all_to_vec(encoded, &mut decoded)
            .unwrap();

        assert_eq!(decoded, expected);
    }

    fn assert_round_trips_with_dictionary(encoded: &[u8], expected: &[u8], dict: &[u8]) {
        let mut decoded = Vec::with_capacity(expected.len());
        let mut decoder = FrameDecoder::new();
        decoder
            .add_dict(Dictionary::decode_dict(dict).unwrap())
            .unwrap();
        decoder.decode_all_to_vec(encoded, &mut decoded).unwrap();

        assert_eq!(decoded, expected);
    }

    fn dictionary_payload() -> Vec<u8> {
        let mut data = Vec::new();
        for _ in 0..10 {
            data.extend_from_slice(dictionary_content());
        }
        data
    }

    const DICT_ID: u32 = 0x4723_2101;

    fn full_dictionary_fixture() -> Vec<u8> {
        let mut raw = Vec::new();
        raw.extend_from_slice(&crate::decoding::dictionary::MAGIC_NUM);
        raw.extend_from_slice(&DICT_ID.to_le_bytes());
        raw.extend_from_slice(dictionary_tables());
        for offset in [3_u32, 10, 25] {
            raw.extend_from_slice(&offset.to_le_bytes());
        }
        raw.extend_from_slice(dictionary_content());
        raw
    }

    fn dictionary_tables() -> &'static [u8] {
        &[
            54, 16, 192, 155, 4, 0, 207, 59, 239, 121, 158, 116, 220, 93, 114, 229, 110, 41, 249,
            95, 165, 255, 83, 202, 254, 68, 74, 159, 63, 161, 100, 151, 137, 21, 184, 183, 189,
            100, 235, 209, 251, 174, 91, 75, 91, 185, 19, 39, 75, 146, 98, 177, 249, 14, 4, 35, 0,
            0, 0, 40, 40, 20, 10, 12, 204, 37, 196, 1, 173, 122, 0, 4, 0, 128, 1, 2, 2, 25, 32, 27,
            27, 22, 24, 26, 18, 12, 12, 15, 16, 11, 69, 37, 225, 48, 20, 12, 6, 2, 161, 80, 40, 20,
            44, 137, 145, 204, 46, 0, 0, 0, 0, 0, 116, 253, 16, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]
    }

    fn dictionary_content() -> &'static [u8] {
        b"method=GET path=/v1/projects/beta status=200 bytes=1847\n\
          method=POST path=/v1/projects/beta status=202 bytes=932\n"
    }
}
