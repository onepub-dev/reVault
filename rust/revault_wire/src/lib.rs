#![deny(missing_docs)]

//! Small, protocol-independent binary framing primitives.
//!
//! The publish protocol keeps its existing wire format. New cross-language
//! messages use this same framing discipline around a versioned payload (the
//! binding payloads are Protobuf messages).

use std::fmt;

/// Represents the magic constant case.
pub const MAGIC: [u8; 4] = *b"LBWF";
/// Represents the version constant case.
pub const VERSION: u16 = 1;
/// Represents the len constant case.
pub const HEADER_LEN: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Represents wire error.
pub enum WireError {
    /// Represents the too short case.
    TooShort,
    /// Represents the bad magic case.
    BadMagic,
    /// Represents the unsupported version case.
    UnsupportedVersion,
    /// Represents the length mismatch case.
    LengthMismatch,
    /// Represents the payload too large case.
    PayloadTooLarge,
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for WireError {}

/// Encodes encode.
pub fn encode(payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(HEADER_LEN + payload.len());
    frame.extend_from_slice(&MAGIC);
    frame.extend_from_slice(&VERSION.to_be_bytes());
    frame.extend_from_slice(&0u16.to_be_bytes());
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

/// Decodes decode.
pub fn decode(frame: &[u8], max_payload: usize) -> Result<&[u8], WireError> {
    if frame.len() < HEADER_LEN {
        return Err(WireError::TooShort);
    }
    if frame[..4] != MAGIC {
        return Err(WireError::BadMagic);
    }
    if u16::from_be_bytes([frame[4], frame[5]]) != VERSION {
        return Err(WireError::UnsupportedVersion);
    }
    let length = u32::from_be_bytes([frame[8], frame[9], frame[10], frame[11]]) as usize;
    if length > max_payload {
        return Err(WireError::PayloadTooLarge);
    }
    if frame.len() != HEADER_LEN + length {
        return Err(WireError::LengthMismatch);
    }
    Ok(&frame[HEADER_LEN..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let frame = encode(b"payload");
        assert_eq!(decode(&frame, 1024), Ok(b"payload".as_slice()));
    }
}
