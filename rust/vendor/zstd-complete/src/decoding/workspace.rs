//! Prepared, reusable decompression workspaces.

use core::{fmt, marker::PhantomData, mem::MaybeUninit};

use super::{errors::FrameDecoderError, frame_decoder::PreparedDictionaryBytes, FrameDecoder};
use crate::{
    decoding::scratch::DecoderScratch,
    workspace::{Arena, ArenaError},
};

/// A decoder whose internal tables and history storage are allocated up front.
///
/// After construction, decoding a valid frame whose window does not exceed
/// `max_window_size` into a sufficiently large caller buffer does not request
/// additional heap storage. Dictionaries must be installed before relying on
/// that prepared-operation guarantee.
pub struct DecoderWorkspace {
    decoder: FrameDecoder,
    max_window_size: usize,
    max_dictionary_size: usize,
}

impl DecoderWorkspace {
    /// Allocates and prepares all storage required by a bounded decoder.
    pub fn new(
        max_window_size: usize,
        max_dictionary_size: usize,
    ) -> Result<Self, DecoderWorkspaceError> {
        if max_window_size == 0 {
            return Err(DecoderWorkspaceError::InvalidWindowSize);
        }
        let mut decoder = FrameDecoder::new();
        decoder.prepare_workspace(max_window_size, max_dictionary_size);
        Ok(Self {
            decoder,
            max_window_size,
            max_dictionary_size,
        })
    }

    pub const fn max_window_size(&self) -> usize {
        self.max_window_size
    }

    pub const fn max_dictionary_size(&self) -> usize {
        self.max_dictionary_size
    }

    /// Decodes complete concatenated frames into caller-owned output storage.
    pub fn decode_into(
        &mut self,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, DecoderWorkspaceError> {
        self.decoder
            .decode_all_with_window_limit(input, output, self.max_window_size as u64)
            .map_err(DecoderWorkspaceError::Decode)
    }

    /// Decodes using one formatted dictionary without allocating an owning
    /// dictionary object or registering it in the decoder.
    pub fn decode_into_with_dictionary(
        &mut self,
        input: &[u8],
        dictionary: &[u8],
        output: &mut [u8],
    ) -> Result<usize, DecoderWorkspaceError> {
        ensure_dictionary_size(dictionary.len(), self.max_dictionary_size)?;
        self.decoder
            .decode_all_with_window_limit_and_dictionary(
                input,
                output,
                self.max_window_size as u64,
                Some(PreparedDictionaryBytes::Formatted(dictionary)),
            )
            .map_err(DecoderWorkspaceError::Decode)
    }

    /// Accesses the underlying decoder to install formatted dictionaries
    /// during workspace preparation.
    pub fn decoder_mut(&mut self) -> &mut FrameDecoder {
        &mut self.decoder
    }

    /// Decodes a frame prepared with a raw-content dictionary.
    pub fn decode_into_with_raw_dictionary(
        &mut self,
        input: &[u8],
        dictionary: &[u8],
        output: &mut [u8],
    ) -> Result<usize, DecoderWorkspaceError> {
        ensure_dictionary_size(dictionary.len(), self.max_dictionary_size)?;
        self.decoder
            .decode_all_with_window_limit_and_dictionary(
                input,
                output,
                self.max_window_size as u64,
                Some(PreparedDictionaryBytes::Raw(dictionary)),
            )
            .map_err(DecoderWorkspaceError::Decode)
    }
}

/// Errors produced by a prepared decoder workspace.
#[derive(Debug)]
pub enum DecoderWorkspaceError {
    InvalidWindowSize,
    InsufficientStorage { required: usize, provided: usize },
    SizeOverflow,
    DictionaryTooLarge { maximum: usize, provided: usize },
    Decode(FrameDecoderError),
}

impl fmt::Display for DecoderWorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWindowSize => formatter.write_str("maximum window size must be non-zero"),
            Self::InsufficientStorage { required, provided } => write!(
                formatter,
                "decoder workspace needs {required} bytes but only {provided} were provided"
            ),
            Self::SizeOverflow => formatter.write_str("decoder workspace size overflowed"),
            Self::DictionaryTooLarge { maximum, provided } => write!(
                formatter,
                "dictionary has {provided} bytes but this workspace permits at most {maximum}"
            ),
            Self::Decode(error) => error.fmt(formatter),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DecoderWorkspaceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Decode(error) => Some(error),
            Self::InvalidWindowSize
            | Self::InsufficientStorage { .. }
            | Self::SizeOverflow
            | Self::DictionaryTooLarge { .. } => None,
        }
    }
}

/// A decoder whose complete scratch state is placed in caller-provided bytes.
pub struct StaticDecoderWorkspace<'storage> {
    decoder: FrameDecoder,
    max_window_size: usize,
    max_dictionary_size: usize,
    workspace_bytes: usize,
    marker: PhantomData<&'storage mut [MaybeUninit<u8>]>,
}

impl<'storage> StaticDecoderWorkspace<'storage> {
    pub fn required_size(
        max_window_size: usize,
        max_dictionary_size: usize,
    ) -> Result<usize, DecoderWorkspaceError> {
        if max_window_size == 0 {
            return Err(DecoderWorkspaceError::InvalidWindowSize);
        }
        DecoderScratch::workspace_size(max_window_size, max_dictionary_size)
            .map_err(map_arena_error)
    }

    pub fn new(
        storage: &'storage mut [u8],
        max_window_size: usize,
        max_dictionary_size: usize,
    ) -> Result<Self, DecoderWorkspaceError> {
        Self::new_uninit(
            initialized_bytes_as_uninit(storage),
            max_window_size,
            max_dictionary_size,
        )
    }

    /// Constructs a decoder without initializing caller-provided bytes.
    pub fn new_uninit(
        storage: &'storage mut [MaybeUninit<u8>],
        max_window_size: usize,
        max_dictionary_size: usize,
    ) -> Result<Self, DecoderWorkspaceError> {
        let required = Self::required_size(max_window_size, max_dictionary_size)?;
        if storage.len() < required {
            return Err(DecoderWorkspaceError::InsufficientStorage {
                required,
                provided: storage.len(),
            });
        }
        let mut arena = Arena::new(storage);
        let scratch = DecoderScratch::new_in(&mut arena, max_window_size, max_dictionary_size)
            .map_err(map_arena_error)?;
        let mut decoder = FrameDecoder::new();
        decoder.install_prepared_scratch(scratch);
        Ok(Self {
            decoder,
            max_window_size,
            max_dictionary_size,
            workspace_bytes: storage.len(),
            marker: PhantomData,
        })
    }

    pub fn decode_into(
        &mut self,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<usize, DecoderWorkspaceError> {
        self.decoder
            .decode_all_with_window_limit(input, output, self.max_window_size as u64)
            .map_err(DecoderWorkspaceError::Decode)
    }

    /// Decodes with a formatted dictionary directly from caller storage.
    pub fn decode_into_with_dictionary(
        &mut self,
        input: &[u8],
        dictionary: &[u8],
        output: &mut [u8],
    ) -> Result<usize, DecoderWorkspaceError> {
        ensure_dictionary_size(dictionary.len(), self.max_dictionary_size)?;
        self.decoder
            .decode_all_with_window_limit_and_dictionary(
                input,
                output,
                self.max_window_size as u64,
                Some(PreparedDictionaryBytes::Formatted(dictionary)),
            )
            .map_err(DecoderWorkspaceError::Decode)
    }

    /// Decodes a frame prepared with a raw-content dictionary.
    pub fn decode_into_with_raw_dictionary(
        &mut self,
        input: &[u8],
        dictionary: &[u8],
        output: &mut [u8],
    ) -> Result<usize, DecoderWorkspaceError> {
        ensure_dictionary_size(dictionary.len(), self.max_dictionary_size)?;
        self.decoder
            .decode_all_with_window_limit_and_dictionary(
                input,
                output,
                self.max_window_size as u64,
                Some(PreparedDictionaryBytes::Raw(dictionary)),
            )
            .map_err(DecoderWorkspaceError::Decode)
    }

    pub const fn max_window_size(&self) -> usize {
        self.max_window_size
    }

    pub const fn max_dictionary_size(&self) -> usize {
        self.max_dictionary_size
    }

    pub const fn workspace_bytes(&self) -> usize {
        self.workspace_bytes
    }
}

fn initialized_bytes_as_uninit(storage: &mut [u8]) -> &mut [MaybeUninit<u8>] {
    // SAFETY: `MaybeUninit<u8>` has the same layout as `u8`, and treating
    // initialized bytes as possibly uninitialized only weakens the invariant.
    unsafe { core::slice::from_raw_parts_mut(storage.as_mut_ptr().cast(), storage.len()) }
}

fn ensure_dictionary_size(provided: usize, maximum: usize) -> Result<(), DecoderWorkspaceError> {
    if provided > maximum {
        Err(DecoderWorkspaceError::DictionaryTooLarge { maximum, provided })
    } else {
        Ok(())
    }
}

fn map_arena_error(error: ArenaError) -> DecoderWorkspaceError {
    match error {
        ArenaError::InsufficientStorage { required, provided } => {
            DecoderWorkspaceError::InsufficientStorage { required, provided }
        }
        ArenaError::CapacityExceeded { capacity } => DecoderWorkspaceError::InsufficientStorage {
            required: capacity.saturating_add(1),
            provided: capacity,
        },
        ArenaError::SizeOverflow => DecoderWorkspaceError::SizeOverflow,
    }
}
