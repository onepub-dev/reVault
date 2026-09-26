use alloc::{string::ToString, vec::Vec};
#[cfg(feature = "hash")]
use core::hash::Hasher;
use core::{convert::TryFrom, fmt};
use std::io::{self, Read, Write};

use super::{levels::c_port, CompressionLevel, CompressionTuning, ContentSizePolicy};
use crate::{
    blocks::block::BlockType,
    common::MAX_BLOCK_SIZE,
    encoding::{block_header::BlockHeader, frame_header::FrameHeader},
};

/// Default maximum estimated resident working set: 96 MiB.
pub const DEFAULT_MEMORY_LIMIT: usize = 96 * 1024 * 1024;
/// Default amount of input stored in each independent Zstandard frame: 8 MiB.
pub const DEFAULT_FRAME_CHUNK_SIZE: usize = 8 * 1024 * 1024;

/// Configuration for the bounded-memory streaming encoder.
#[derive(Clone)]
pub struct EncoderOptions {
    level: CompressionLevel,
    memory_limit: usize,
    frame_chunk_size: usize,
    dictionary: Option<EncoderDictionary>,
    checksum: bool,
    tuning: CompressionTuning,
    pledged_source_size: Option<u64>,
    content_size_policy: ContentSizePolicy,
}

impl EncoderOptions {
    /// Creates options using the 96 MiB memory budget and 8 MiB frame chunks.
    pub const fn new(level: CompressionLevel) -> Self {
        Self {
            level,
            memory_limit: DEFAULT_MEMORY_LIMIT,
            frame_chunk_size: DEFAULT_FRAME_CHUNK_SIZE,
            dictionary: None,
            checksum: false,
            tuning: CompressionTuning::new(),
            pledged_source_size: None,
            content_size_policy: ContentSizePolicy::Include,
        }
    }

    pub const fn level(&self) -> CompressionLevel {
        self.level
    }

    pub const fn memory_limit(&self) -> usize {
        self.memory_limit
    }

    pub const fn frame_chunk_size(&self) -> usize {
        self.frame_chunk_size
    }

    pub fn dictionary(&self) -> Option<&EncoderDictionary> {
        self.dictionary.as_ref()
    }

    pub const fn checksum(&self) -> bool {
        self.checksum
    }

    pub const fn tuning(&self) -> CompressionTuning {
        self.tuning
    }

    pub const fn pledged_source_size(&self) -> Option<u64> {
        self.pledged_source_size
    }

    pub const fn content_size_policy(&self) -> ContentSizePolicy {
        self.content_size_policy
    }

    /// Returns the conservative memory estimate checked by [`Encoder::new`].
    pub fn estimated_memory_usage(&self) -> usize {
        let required = c_port::estimated_frame_memory(self.level.c_level(), self.frame_chunk_size);
        self.dictionary.as_ref().map_or(required, |dictionary| {
            let retained_copies = if cfg!(feature = "multithreading") {
                3
            } else {
                2
            };
            required.saturating_add(dictionary.raw_size().saturating_mul(retained_copies))
        })
    }

    /// Sets the maximum conservative working-memory estimate.
    pub const fn with_memory_limit(mut self, bytes: usize) -> Self {
        self.memory_limit = bytes;
        self
    }

    /// Sets the input bytes collected per independent Zstandard frame.
    ///
    /// Smaller chunks reduce latency and memory use but can reduce compression
    /// ratio because matches do not cross frame boundaries.
    pub const fn with_frame_chunk_size(mut self, bytes: usize) -> Self {
        self.frame_chunk_size = bytes;
        self
    }

    /// Uses a prepared dictionary for every frame in the output archive.
    pub fn with_dictionary(mut self, dictionary: EncoderDictionary) -> Self {
        self.dictionary = Some(dictionary);
        self
    }

    /// Enables or disables the standard 32-bit frame content checksum.
    pub const fn with_checksum(mut self, enabled: bool) -> Self {
        self.checksum = enabled;
        self
    }

    /// Applies validated, typed overrides to the selected level preset.
    pub const fn with_tuning(mut self, tuning: CompressionTuning) -> Self {
        self.tuning = tuning;
        self
    }

    /// Declares the exact number of input bytes the encoder must receive.
    pub const fn with_pledged_source_size(mut self, bytes: u64) -> Self {
        self.pledged_source_size = Some(bytes);
        self
    }

    /// Includes or omits the decoded size in each emitted frame header.
    pub const fn with_content_size_policy(mut self, policy: ContentSizePolicy) -> Self {
        self.content_size_policy = policy;
        self
    }

    pub(super) fn validate(&self) -> Result<(), EncodeError> {
        if self.frame_chunk_size == 0 {
            return Err(EncodeError::InvalidOptions(
                "frame chunk size must be greater than zero",
            ));
        }
        if self.level.is_uncompressed() && self.dictionary.is_some() {
            return Err(EncodeError::InvalidOptions(
                "an uncompressed encoder cannot use a dictionary",
            ));
        }
        if self.level.is_uncompressed() && self.tuning != CompressionTuning::new() {
            return Err(EncodeError::InvalidOptions(
                "an uncompressed encoder cannot use compression tuning",
            ));
        }
        self.tuning
            .validate()
            .map_err(EncodeError::InvalidOptions)?;
        if self.checksum && !cfg!(feature = "hash") {
            return Err(EncodeError::InvalidOptions(
                "checksums require the zstd-complete `hash` feature",
            ));
        }
        let required = self.estimated_memory_usage();
        if required > self.memory_limit {
            return Err(EncodeError::MemoryLimitExceeded {
                limit: self.memory_limit,
                required,
            });
        }
        Ok(())
    }
}

impl fmt::Debug for EncoderOptions {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EncoderOptions")
            .field("level", &self.level)
            .field("memory_limit", &self.memory_limit)
            .field("frame_chunk_size", &self.frame_chunk_size)
            .field("dictionary", &self.dictionary)
            .field("checksum", &self.checksum)
            .field("tuning", &self.tuning)
            .field("pledged_source_size", &self.pledged_source_size)
            .field("content_size_policy", &self.content_size_policy)
            .finish()
    }
}

/// An owned, parsed dictionary reusable across independent frames.
#[derive(Clone)]
pub struct EncoderDictionary {
    prepared: c_port::PreparedDictionary,
    raw_size: usize,
    #[cfg(feature = "multithreading")]
    raw: Vec<u8>,
}

impl EncoderDictionary {
    /// Copies and validates raw-content or formatted Zstandard dictionary data.
    pub fn copy(dictionary: &[u8]) -> Result<Self, DictionaryError> {
        let prepared = c_port::PreparedDictionary::from_bytes(dictionary)
            .map_err(|error| match error {
                c_port::DictionaryParseError::WrongDictionary => DictionaryError::WrongDictionary,
                c_port::DictionaryParseError::CorruptedDictionary => {
                    DictionaryError::CorruptedDictionary
                }
            })?
            .ok_or(DictionaryError::WrongDictionary)?;
        Ok(Self {
            prepared,
            raw_size: dictionary.len(),
            #[cfg(feature = "multithreading")]
            raw: dictionary.to_vec(),
        })
    }

    pub const fn raw_size(&self) -> usize {
        self.raw_size
    }

    #[cfg(feature = "multithreading")]
    pub(super) fn raw(&self) -> &[u8] {
        &self.raw
    }
}

impl fmt::Debug for EncoderDictionary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EncoderDictionary")
            .field("raw_size", &self.raw_size)
            .finish_non_exhaustive()
    }
}

/// Error returned when dictionary bytes are not a usable Zstandard dictionary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DictionaryError {
    WrongDictionary,
    CorruptedDictionary,
}

impl fmt::Display for DictionaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WrongDictionary => "input is not a usable Zstandard dictionary",
            Self::CorruptedDictionary => "Zstandard dictionary metadata is corrupted",
        })
    }
}

impl std::error::Error for DictionaryError {}

impl Default for EncoderOptions {
    fn default() -> Self {
        Self::new(CompressionLevel::DEFAULT)
    }
}

/// Errors produced by the bounded streaming API.
#[derive(Debug)]
pub enum EncodeError {
    Io(io::Error),
    InvalidOptions(&'static str),
    MemoryLimitExceeded {
        limit: usize,
        required: usize,
    },
    PledgedSourceSizeMismatch {
        pledged: u64,
        actual: u64,
    },
    #[cfg(feature = "multithreading")]
    WorkerFailed,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::InvalidOptions(message) => formatter.write_str(message),
            Self::MemoryLimitExceeded { limit, required } => write!(
                formatter,
                "encoder needs an estimated {required} bytes, exceeding the {limit}-byte memory limit"
            ),
            Self::PledgedSourceSizeMismatch { pledged, actual } => write!(
                formatter,
                "encoder received {actual} bytes but was pledged exactly {pledged} bytes"
            ),
            #[cfg(feature = "multithreading")]
            Self::WorkerFailed => formatter.write_str("a compression worker failed"),
        }
    }
}

impl std::error::Error for EncodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for EncodeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// A bounded-memory Zstandard encoder implementing [`Write`].
///
/// Each full input chunk becomes an independent frame in the output archive.
/// Call [`Encoder::finish`] to emit the final frame and recover the writer.
pub struct Encoder<W: Write> {
    inner: W,
    options: EncoderOptions,
    input: Vec<u8>,
    emitted_frame: bool,
    accepted_input: u64,
}

impl<W: Write> Encoder<W> {
    pub fn new(inner: W, options: EncoderOptions) -> Result<Self, EncodeError> {
        options.validate()?;
        let input = Vec::with_capacity(options.frame_chunk_size);
        Ok(Self {
            inner,
            options,
            input,
            emitted_frame: false,
            accepted_input: 0,
        })
    }

    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Emits the final frame and returns the wrapped writer.
    pub fn finish(mut self) -> Result<W, EncodeError> {
        self.verify_pledged_size()?;
        if !self.input.is_empty() || !self.emitted_frame {
            self.emit_frame()?;
        }
        self.inner.flush()?;
        Ok(self.inner)
    }

    fn emit_frame(&mut self) -> io::Result<()> {
        let mut compressed = encode_frame_payload(&self.input, &self.options);
        apply_content_size_policy(&mut compressed, self.options.content_size_policy)?;
        if self.options.checksum {
            append_checksum(&mut compressed, &self.input);
        }
        self.inner.write_all(&compressed)?;
        self.input.clear();
        self.emitted_frame = true;
        Ok(())
    }

    fn verify_pledged_size(&self) -> Result<(), EncodeError> {
        if let Some(pledged) = self.options.pledged_source_size {
            if pledged != self.accepted_input {
                return Err(EncodeError::PledgedSourceSizeMismatch {
                    pledged,
                    actual: self.accepted_input,
                });
            }
        }
        Ok(())
    }
}

fn encode_frame_payload(source: &[u8], options: &EncoderOptions) -> Vec<u8> {
    if options.level.is_uncompressed() {
        return encode_uncompressed_frame(source);
    }
    let has_tuning = options.tuning != CompressionTuning::new();
    match (options.dictionary.as_ref(), has_tuning) {
        (Some(dictionary), false) => c_port::encode_frame_with_prepared_dictionary(
            source,
            options.level.c_level(),
            &dictionary.prepared,
        ),
        (Some(dictionary), true) => c_port::encode_frame_with_prepared_dictionary_and_tuning(
            source,
            options.level.c_level(),
            &dictionary.prepared,
            options.tuning,
        ),
        (None, false) => c_port::encode_frame_no_dict(source, options.level.c_level()),
        (None, true) => c_port::encode_frame_no_dict_with_tuning(
            source,
            options.level.c_level(),
            options.tuning,
        ),
    }
}

fn apply_content_size_policy(frame: &mut Vec<u8>, policy: ContentSizePolicy) -> io::Result<()> {
    if policy == ContentSizePolicy::Include {
        return Ok(());
    }
    let (parsed, old_header_size) = crate::decoding::frame::read_frame_header(frame.as_slice())
        .map_err(|error| io::Error::other(error.to_string()))?;
    let window_size = parsed.window_size().unwrap_or(1024).max(1024);
    let mut header = Vec::with_capacity(18);
    FrameHeader {
        frame_content_size: None,
        single_segment: false,
        content_checksum: false,
        dictionary_id: parsed.dictionary_id().map(u64::from),
        window_size: Some(window_size),
    }
    .serialize(&mut header);
    frame.splice(..usize::from(old_header_size), header);
    Ok(())
}

fn append_checksum(frame: &mut Vec<u8>, source: &[u8]) {
    #[cfg(feature = "hash")]
    {
        debug_assert!(frame.len() >= 5);
        frame[4] |= 1 << 2;
        let mut hasher = twox_hash::XxHash64::with_seed(0);
        hasher.write(source);
        frame.extend_from_slice(&(hasher.finish() as u32).to_le_bytes());
    }
    #[cfg(not(feature = "hash"))]
    {
        let _ = (frame, source);
        unreachable!("checksum options are rejected without the hash feature");
    }
}

#[cfg(feature = "multithreading")]
pub(super) fn encode_frame_for_options(source: &[u8], options: &EncoderOptions) -> Vec<u8> {
    let mut compressed = encode_frame_payload(source, options);
    apply_content_size_policy(&mut compressed, options.content_size_policy)
        .expect("validated frame header can be rewritten");
    if options.checksum {
        append_checksum(&mut compressed, source);
    }
    compressed
}

fn encode_uncompressed_frame(source: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(source.len().saturating_add(16));
    FrameHeader {
        frame_content_size: Some(source.len() as u64),
        single_segment: true,
        content_checksum: false,
        dictionary_id: None,
        window_size: None,
    }
    .serialize(&mut output);

    if source.is_empty() {
        BlockHeader {
            last_block: true,
            block_type: BlockType::Raw,
            block_size: 0,
        }
        .serialize(&mut output);
        return output;
    }

    for (index, block) in source.chunks(MAX_BLOCK_SIZE as usize).enumerate() {
        BlockHeader {
            last_block: (index + 1) * MAX_BLOCK_SIZE as usize >= source.len(),
            block_type: BlockType::Raw,
            block_size: block.len() as u32,
        }
        .serialize(&mut output);
        output.extend_from_slice(block);
    }
    output
}

impl<W: Write> Write for Encoder<W> {
    fn write(&mut self, source: &[u8]) -> io::Result<usize> {
        if source.is_empty() {
            return Ok(0);
        }
        if self.input.len() == self.options.frame_chunk_size {
            self.emit_frame()?;
        }
        let available = self.options.frame_chunk_size - self.input.len();
        let consumed = available.min(source.len());
        if let Some(pledged) = self.options.pledged_source_size {
            let remaining = pledged.saturating_sub(self.accepted_input);
            if remaining == 0 {
                return Err(io::Error::other(EncodeError::PledgedSourceSizeMismatch {
                    pledged,
                    actual: self.accepted_input.saturating_add(source.len() as u64),
                }));
            }
            let consumed = consumed.min(usize::try_from(remaining).unwrap_or(usize::MAX));
            self.input.extend_from_slice(&source[..consumed]);
            self.accepted_input += consumed as u64;
            return Ok(consumed);
        }
        self.input.extend_from_slice(&source[..consumed]);
        self.accepted_input += consumed as u64;
        Ok(consumed)
    }

    /// Closes the current frame before flushing the wrapped writer.
    fn flush(&mut self) -> io::Result<()> {
        if !self.input.is_empty() {
            self.emit_frame()?;
        }
        self.inner.flush()
    }
}

/// Streams all input into a bounded-memory encoder.
pub fn encode<R: Read, W: Write>(
    mut source: R,
    target: W,
    options: EncoderOptions,
) -> Result<(), EncodeError> {
    let mut encoder = Encoder::new(target, options)?;
    io::copy(&mut source, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

/// Compresses all input and returns the resulting Zstandard archive.
pub fn encode_all<R: Read>(source: R, options: EncoderOptions) -> Result<Vec<u8>, EncodeError> {
    let mut output = Vec::new();
    encode(source, &mut output, options)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoding::FrameDecoder;
    use alloc::vec;

    #[test]
    fn multi_frame_stream_roundtrips() {
        let input = b"abcdefgh".repeat(4096);
        let options = EncoderOptions::new(CompressionLevel::FASTEST)
            .with_frame_chunk_size(1024)
            .with_memory_limit(16 * 1024 * 1024);
        let compressed = encode_all(input.as_slice(), options).unwrap();
        let mut decoded = Vec::with_capacity(input.len());
        FrameDecoder::new()
            .decode_all_to_vec(&compressed, &mut decoded)
            .unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn empty_input_is_a_valid_frame() {
        let compressed = encode_all(&[][..], EncoderOptions::default()).unwrap();
        let mut decoded = Vec::with_capacity(1);
        FrameDecoder::new()
            .decode_all_to_vec(&compressed, &mut decoded)
            .unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn uncompressed_stream_roundtrips() {
        let input = vec![42; 300_000];
        let options =
            EncoderOptions::new(CompressionLevel::UNCOMPRESSED).with_frame_chunk_size(150_000);
        let compressed = encode_all(input.as_slice(), options).unwrap();
        assert_eq!(zstd::decode_all(compressed.as_slice()).unwrap(), input);
    }

    #[test]
    fn archive_is_interoperable_with_the_c_decoder() {
        let input = b"interoperable streaming input".repeat(10_000);
        let options = EncoderOptions::new(CompressionLevel::DEFAULT)
            .with_frame_chunk_size(32 * 1024)
            .with_memory_limit(32 * 1024 * 1024);
        let compressed = encode_all(input.as_slice(), options).unwrap();
        assert_eq!(zstd::decode_all(compressed.as_slice()).unwrap(), input);
    }

    #[test]
    fn default_options_keep_the_existing_specialized_frame_path() {
        let input = b"unchanged default single-thread frame path".repeat(10_000);
        let expected = c_port::encode_frame_no_dict(&input, CompressionLevel::DEFAULT.c_level());
        let actual = encode_all(input.as_slice(), EncoderOptions::default()).unwrap();
        assert_eq!(actual, expected);
    }

    #[cfg(feature = "hash")]
    #[test]
    fn checksummed_frames_roundtrip() {
        let input = b"checksummed content".repeat(10_000);
        let options = EncoderOptions::new(CompressionLevel::DEFAULT)
            .with_frame_chunk_size(32 * 1024)
            .with_memory_limit(32 * 1024 * 1024)
            .with_checksum(true);
        let compressed = encode_all(input.as_slice(), options).unwrap();
        assert_eq!(zstd::decode_all(compressed.as_slice()).unwrap(), input);
    }

    #[test]
    fn prepared_dictionary_stream_roundtrips() {
        let dictionary_bytes = b"dictionary words and repeated prefixes for the test".repeat(8);
        let input = dictionary_bytes.repeat(200);
        let dictionary = EncoderDictionary::copy(&dictionary_bytes).unwrap();
        let options = EncoderOptions::new(CompressionLevel::DEFAULT)
            .with_frame_chunk_size(16 * 1024)
            .with_memory_limit(32 * 1024 * 1024)
            .with_dictionary(dictionary);
        let compressed = encode_all(input.as_slice(), options).unwrap();

        let mut decoded = Vec::with_capacity(input.len());
        let mut decoder =
            zstd::stream::read::Decoder::with_dictionary(compressed.as_slice(), &dictionary_bytes)
                .unwrap();
        decoder.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn too_short_dictionary_is_rejected_instead_of_silently_ignored() {
        assert_eq!(
            EncoderDictionary::copy(b"short").unwrap_err(),
            DictionaryError::WrongDictionary
        );
    }

    #[test]
    fn invalid_budget_is_reported_before_writing() {
        let options = EncoderOptions::new(CompressionLevel::MAXIMUM);
        assert!(matches!(
            Encoder::new(io::sink(), options),
            Err(EncodeError::MemoryLimitExceeded { .. })
        ));
    }

    #[test]
    fn target_io_errors_are_preserved() {
        struct FailingWriter;

        impl Write for FailingWriter {
            fn write(&mut self, _source: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "test failure"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let error = encode(
            b"accepted input".as_slice(),
            FailingWriter,
            EncoderOptions::default(),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            EncodeError::Io(ref error) if error.kind() == io::ErrorKind::BrokenPipe
        ));
    }

    #[test]
    fn input_buffer_never_exceeds_the_configured_chunk() {
        let options = EncoderOptions::new(CompressionLevel::FASTEST)
            .with_frame_chunk_size(1024)
            .with_memory_limit(16 * 1024 * 1024);
        let mut encoder = Encoder::new(io::sink(), options).unwrap();
        for _ in 0..1024 {
            encoder.write_all(&[7; 4096]).unwrap();
            assert!(encoder.input.len() <= 1024);
            assert_eq!(encoder.input.capacity(), 1024);
        }
        encoder.finish().unwrap();
    }

    #[test]
    fn large_logical_source_does_not_require_a_large_input_allocation() {
        struct RepeatingReader {
            remaining: usize,
        }

        impl Read for RepeatingReader {
            fn read(&mut self, target: &mut [u8]) -> io::Result<usize> {
                let count = self.remaining.min(target.len());
                target[..count].fill(19);
                self.remaining -= count;
                Ok(count)
            }
        }

        let options = EncoderOptions::new(CompressionLevel::UNCOMPRESSED)
            .with_frame_chunk_size(1024 * 1024)
            .with_memory_limit(4 * 1024 * 1024);
        encode(
            RepeatingReader {
                remaining: 64 * 1024 * 1024,
            },
            io::sink(),
            options,
        )
        .unwrap();
    }

    #[test]
    fn negative_fast_levels_interoperate() {
        let input = b"fast acceleration content".repeat(20_000);
        let level = CompressionLevel::fast(8).unwrap();
        let compressed = encode_all(input.as_slice(), EncoderOptions::new(level)).unwrap();
        assert_eq!(zstd::decode_all(compressed.as_slice()).unwrap(), input);
    }

    #[test]
    fn typed_tuning_changes_real_compressor_parameters() {
        let input = b"typed compressor tuning".repeat(20_000);
        let tuning = CompressionTuning::new()
            .with_strategy(super::super::CompressionStrategy::Greedy)
            .with_target_compressed_block_size(16 * 1024);
        let options = EncoderOptions::default().with_tuning(tuning);
        let compressed = encode_all(input.as_slice(), options).unwrap();
        assert_eq!(zstd::decode_all(compressed.as_slice()).unwrap(), input);
    }

    #[test]
    fn omitted_content_size_interoperates() {
        let input = b"content size omitted".repeat(10_000);
        let options = EncoderOptions::default().with_content_size_policy(ContentSizePolicy::Omit);
        let compressed = encode_all(input.as_slice(), options).unwrap();
        let (header, _) = crate::decoding::frame::read_frame_header(compressed.as_slice()).unwrap();
        assert!(!header.descriptor.single_segment_flag());
        assert_eq!(header.frame_content_size(), 0);
        assert_eq!(zstd::decode_all(compressed.as_slice()).unwrap(), input);
    }

    #[test]
    fn pledged_size_must_match_exactly() {
        let options = EncoderOptions::default().with_pledged_source_size(4);
        let error = encode_all(b"abc".as_slice(), options).unwrap_err();
        assert!(matches!(
            error,
            EncodeError::PledgedSourceSizeMismatch {
                pledged: 4,
                actual: 3
            }
        ));
    }
}
