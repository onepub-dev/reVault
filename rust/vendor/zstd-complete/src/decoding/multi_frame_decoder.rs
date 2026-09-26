//! Bounded high-level decoding of concatenated Zstandard frames.

use core::{fmt, num::NonZeroUsize};
use std::io::{self, Read};

use super::{
    errors::{FrameDecoderError, ReadFrameHeaderError},
    BlockDecodingStrategy, Dictionary, FrameDecoder, DEFAULT_MAX_WINDOW_SIZE,
};

/// Policy for RFC 8878 skippable frames encountered between data frames.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SkippableFramePolicy {
    /// Consume the skippable payload without exposing it to the decoded stream.
    #[default]
    Skip,
    /// Return an error as soon as a skippable-frame header is encountered.
    Reject,
}

/// Resource and format policy for [`MultiFrameDecoder`].
#[derive(Clone, Debug)]
pub struct MultiFrameDecoderOptions {
    skippable_frames: SkippableFramePolicy,
    max_frames: Option<NonZeroUsize>,
    max_skippable_frame_size: usize,
    max_window_size: u64,
    max_decoded_bytes: Option<NonZeroUsize>,
}

impl MultiFrameDecoderOptions {
    /// Creates a bounded default policy.
    pub const fn new() -> Self {
        Self {
            skippable_frames: SkippableFramePolicy::Skip,
            max_frames: None,
            max_skippable_frame_size: 16 * 1024 * 1024,
            max_window_size: DEFAULT_MAX_WINDOW_SIZE,
            max_decoded_bytes: None,
        }
    }

    pub const fn skippable_frame_policy(&self) -> SkippableFramePolicy {
        self.skippable_frames
    }

    pub const fn max_frames(&self) -> Option<NonZeroUsize> {
        self.max_frames
    }

    pub const fn max_skippable_frame_size(&self) -> usize {
        self.max_skippable_frame_size
    }

    pub const fn max_window_size(&self) -> u64 {
        self.max_window_size
    }

    pub const fn max_decoded_bytes(&self) -> Option<NonZeroUsize> {
        self.max_decoded_bytes
    }

    pub const fn with_skippable_frame_policy(mut self, policy: SkippableFramePolicy) -> Self {
        self.skippable_frames = policy;
        self
    }

    pub const fn with_max_frames(mut self, limit: Option<NonZeroUsize>) -> Self {
        self.max_frames = limit;
        self
    }

    pub const fn with_max_skippable_frame_size(mut self, bytes: usize) -> Self {
        self.max_skippable_frame_size = bytes;
        self
    }

    pub const fn with_max_window_size(mut self, bytes: u64) -> Self {
        self.max_window_size = bytes;
        self
    }

    pub const fn with_max_decoded_bytes(mut self, limit: Option<NonZeroUsize>) -> Self {
        self.max_decoded_bytes = limit;
        self
    }
}

impl Default for MultiFrameDecoderOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// A typed failure retained inside the [`io::Error`] returned by [`Read`].
#[derive(Debug)]
#[non_exhaustive]
pub enum MultiFrameDecoderError {
    Frame(FrameDecoderError),
    Io(io::Error),
    SkippableFrameRejected { magic_number: u32, length: u32 },
    SkippableFrameTooLarge { length: u32, limit: usize },
    FrameLimitExceeded { limit: usize },
    DecodedSizeLimitExceeded { limit: usize },
}

impl fmt::Display for MultiFrameDecoderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Frame(error) => error.fmt(formatter),
            Self::Io(error) => error.fmt(formatter),
            Self::SkippableFrameRejected {
                magic_number,
                length,
            } => write!(
                formatter,
                "skippable frame 0x{magic_number:X} with {length} payload bytes was rejected"
            ),
            Self::SkippableFrameTooLarge { length, limit } => write!(
                formatter,
                "skippable frame has {length} payload bytes, exceeding the {limit}-byte limit"
            ),
            Self::FrameLimitExceeded { limit } => {
                write!(formatter, "archive exceeds the {limit}-frame limit")
            }
            Self::DecodedSizeLimitExceeded { limit } => write!(
                formatter,
                "decoded archive exceeds the {limit}-byte output limit"
            ),
        }
    }
}

impl std::error::Error for MultiFrameDecoderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Frame(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

/// A `Read` decoder for concatenated data and skippable Zstandard frames.
pub struct MultiFrameDecoder<R: Read> {
    source: R,
    decoder: FrameDecoder,
    options: MultiFrameDecoderOptions,
    state: DecoderState,
    frame_count: usize,
    decoded_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DecoderState {
    NeedFrame,
    Decoding,
    Finished,
}

impl<R: Read> MultiFrameDecoder<R> {
    pub fn new(source: R) -> Self {
        Self::with_options(source, MultiFrameDecoderOptions::new())
    }

    pub fn with_options(source: R, options: MultiFrameDecoderOptions) -> Self {
        Self {
            source,
            decoder: FrameDecoder::new(),
            options,
            state: DecoderState::NeedFrame,
            frame_count: 0,
            decoded_bytes: 0,
        }
    }

    /// Adds a formatted dictionary selected automatically by frame dictionary ID.
    pub fn add_dictionary(&mut self, dictionary: Dictionary) -> Result<(), FrameDecoderError> {
        self.decoder.add_dict(dictionary)
    }

    pub fn get_ref(&self) -> &R {
        &self.source
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.source
    }

    pub fn into_inner(self) -> R {
        self.source
    }

    pub const fn frame_count(&self) -> usize {
        self.frame_count
    }

    pub const fn decoded_bytes(&self) -> usize {
        self.decoded_bytes
    }

    fn start_next_frame(&mut self) -> Result<bool, MultiFrameDecoderError> {
        loop {
            let Some(first) = read_one(&mut self.source)? else {
                self.state = DecoderState::Finished;
                return Ok(false);
            };
            if let Some(limit) = self.options.max_frames {
                if self.frame_count >= limit.get() {
                    return Err(MultiFrameDecoderError::FrameLimitExceeded { limit: limit.get() });
                }
            }
            let prefix = io::Cursor::new([first]);
            match self.decoder.reset_with_window_limit(
                prefix.chain(&mut self.source),
                self.options.max_window_size,
            ) {
                Ok(()) => {
                    self.frame_count += 1;
                    self.state = DecoderState::Decoding;
                    return Ok(true);
                }
                Err(FrameDecoderError::ReadFrameHeaderError(ReadFrameHeaderError::SkipFrame {
                    magic_number,
                    length,
                })) => {
                    if self.options.skippable_frames == SkippableFramePolicy::Reject {
                        return Err(MultiFrameDecoderError::SkippableFrameRejected {
                            magic_number,
                            length,
                        });
                    }
                    if length as usize > self.options.max_skippable_frame_size {
                        return Err(MultiFrameDecoderError::SkippableFrameTooLarge {
                            length,
                            limit: self.options.max_skippable_frame_size,
                        });
                    }
                    discard_exact(&mut self.source, length as usize)?;
                }
                Err(error) => return Err(MultiFrameDecoderError::Frame(error)),
            }
        }
    }

    fn read_decoded(&mut self, target: &mut [u8]) -> Result<usize, MultiFrameDecoderError> {
        let remaining = self.options.max_decoded_bytes.map_or(usize::MAX, |limit| {
            limit.get().saturating_sub(self.decoded_bytes)
        });

        while self.decoder.can_collect() == 0 && !self.decoder.is_finished() {
            self.decoder
                .decode_blocks(&mut self.source, BlockDecodingStrategy::UptoBytes(1))
                .map_err(MultiFrameDecoderError::Frame)?;
        }

        if remaining == 0 && self.decoder.can_collect() != 0 {
            return Err(MultiFrameDecoderError::DecodedSizeLimitExceeded {
                limit: self
                    .options
                    .max_decoded_bytes
                    .expect("zero remaining requires a configured limit")
                    .get(),
            });
        }

        let requested = target.len().min(remaining);
        let read = self
            .decoder
            .read(&mut target[..requested])
            .map_err(MultiFrameDecoderError::Io)?;
        self.decoded_bytes += read;
        Ok(read)
    }
}

impl<R: Read> Read for MultiFrameDecoder<R> {
    fn read(&mut self, target: &mut [u8]) -> io::Result<usize> {
        if target.is_empty() {
            return Ok(0);
        }

        loop {
            match self.state {
                DecoderState::Finished => return Ok(0),
                DecoderState::NeedFrame => {
                    self.start_next_frame().map_err(io::Error::other)?;
                }
                DecoderState::Decoding => {
                    let read = self.read_decoded(target).map_err(io::Error::other)?;
                    if read != 0 {
                        return Ok(read);
                    }
                    if self.decoder.is_finished() && self.decoder.can_collect() == 0 {
                        self.state = DecoderState::NeedFrame;
                    }
                }
            }
        }
    }
}

fn read_one(source: &mut impl Read) -> Result<Option<u8>, MultiFrameDecoderError> {
    let mut byte = [0_u8; 1];
    loop {
        match source.read(&mut byte) {
            Ok(0) => return Ok(None),
            Ok(_) => return Ok(Some(byte[0])),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(MultiFrameDecoderError::Io(error)),
        }
    }
}

fn discard_exact(
    source: &mut impl Read,
    mut remaining: usize,
) -> Result<(), MultiFrameDecoderError> {
    let mut buffer = [0_u8; 8192];
    while remaining != 0 {
        let requested = remaining.min(buffer.len());
        let read = source
            .read(&mut buffer[..requested])
            .map_err(MultiFrameDecoderError::Io)?;
        if read == 0 {
            return Err(MultiFrameDecoderError::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "truncated skippable frame",
            )));
        }
        remaining -= read;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::{encode_all, CompressionLevel, EncoderOptions};
    use std::vec::Vec;

    #[test]
    fn reads_concatenated_frames_and_skippable_payloads() {
        let first = b"first frame".repeat(100);
        let second = b"second frame".repeat(100);
        let mut archive = encode_all(
            first.as_slice(),
            EncoderOptions::new(CompressionLevel::FASTEST),
        )
        .unwrap();
        archive.extend_from_slice(&0x184D2A50_u32.to_le_bytes());
        archive.extend_from_slice(&4_u32.to_le_bytes());
        archive.extend_from_slice(b"skip");
        archive.extend_from_slice(
            &encode_all(
                second.as_slice(),
                EncoderOptions::new(CompressionLevel::DEFAULT),
            )
            .unwrap(),
        );

        let mut decoder = MultiFrameDecoder::new(archive.as_slice());
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, [first, second].concat());
        assert_eq!(decoder.frame_count(), 2);
    }

    #[test]
    fn accepts_an_empty_archive() {
        let mut decoded = Vec::new();
        MultiFrameDecoder::new(&[][..])
            .read_to_end(&mut decoded)
            .unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn decoded_size_limit_is_enforced() {
        let input = b"bounded decoded output".repeat(100);
        let archive = encode_all(input.as_slice(), EncoderOptions::default()).unwrap();
        let options = MultiFrameDecoderOptions::new()
            .with_max_decoded_bytes(NonZeroUsize::new(input.len() - 1));
        let mut decoder = MultiFrameDecoder::with_options(archive.as_slice(), options);
        let error = io::copy(&mut decoder, &mut io::sink()).unwrap_err();
        assert!(error
            .get_ref()
            .and_then(|error| error.downcast_ref::<MultiFrameDecoderError>())
            .is_some_and(|error| matches!(
                error,
                MultiFrameDecoderError::DecodedSizeLimitExceeded { .. }
            )));
    }
}
