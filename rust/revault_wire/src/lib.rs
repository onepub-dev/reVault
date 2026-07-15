//! Small, protocol-independent binary framing primitives.
//!
//! The publish protocol keeps its existing wire format. New cross-language
//! messages use this same framing discipline around a versioned payload (the
//! binding payloads are Protobuf messages).

use std::fmt;

pub const MAGIC: [u8; 4] = *b"LBWF";
pub const VERSION: u16 = 1;
pub const HEADER_LEN: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireError {
    TooShort,
    BadMagic,
    UnsupportedVersion,
    LengthMismatch,
    PayloadTooLarge,
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for WireError {}

pub fn encode(payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(HEADER_LEN + payload.len());
    frame.extend_from_slice(&MAGIC);
    frame.extend_from_slice(&VERSION.to_be_bytes());
    frame.extend_from_slice(&0u16.to_be_bytes());
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

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
