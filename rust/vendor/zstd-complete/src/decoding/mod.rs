//! Structures and utilities used for decoding zstd formatted data

pub mod errors;
mod frame_decoder;
#[cfg(feature = "std")]
mod multi_frame_decoder;
mod streaming_decoder;
mod workspace;

pub use dictionary::Dictionary;
pub use frame_decoder::{BlockDecodingStrategy, FrameDecoder, DEFAULT_MAX_WINDOW_SIZE};
#[cfg(feature = "std")]
pub use multi_frame_decoder::{
    MultiFrameDecoder, MultiFrameDecoderError, MultiFrameDecoderOptions, SkippableFramePolicy,
};
pub use streaming_decoder::StreamingDecoder;
pub use workspace::{DecoderWorkspace, DecoderWorkspaceError, StaticDecoderWorkspace};

pub(crate) mod block_decoder;
pub(crate) mod decode_buffer;
pub(crate) mod dictionary;
pub(crate) mod frame;
pub(crate) mod literals_section_decoder;
mod ringbuffer;
#[allow(dead_code)]
pub(crate) mod scratch;
pub(crate) mod sequence_execution;
pub(crate) mod sequence_section_decoder;
