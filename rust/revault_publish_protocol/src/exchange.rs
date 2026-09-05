//! Reciprocal exchange. The relay is not an identity authority.
//!
//! Verification uses the entire SHA-256 transcript digest, never a truncated
//! short authentication string. Clients pin their own contribution and compare
//! all 256 bits through an independently authenticated channel.

#[cfg(feature = "http")]
mod client;
mod model;
#[cfg(feature = "http")]
pub use client::ExchangeClient;
pub use model::*;

use sha2::{Digest, Sha256};

/// Exchange errors are safe to display and never contain bearer capabilities.
#[derive(Debug)]
pub struct ExchangeError(pub String);

impl std::fmt::Display for ExchangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ExchangeError {}

/// Result of an exchange operation.
pub type Result<T> = std::result::Result<T, ExchangeError>;

pub(crate) fn invalid(message: &str) -> ExchangeError {
    ExchangeError(message.to_owned())
}

/// Encodes bytes as lowercase hexadecimal.
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Generates an unguessable 256-bit identifier or bearer capability.
pub fn random_token() -> Result<String> {
    let mut bytes = [0; 32];
    getrandom::fill(&mut bytes).map_err(|_| invalid("random generation failed"))?;
    Ok(hex(&bytes))
}

/// Hashes a capability before persisting it on the relay.
pub fn token_hash(token: &str) -> String {
    hex(&Sha256::digest(token.as_bytes()))
}

/// Checks the canonical form of an identifier or capability.
pub fn valid_token(token: &str) -> bool {
    token.len() == 64
        && token
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Serializes a bounded protocol record.
pub fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
    let bytes = serde_json::to_vec(value).map_err(|_| invalid("cannot encode exchange"))?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(invalid("exchange message too large"));
    }
    Ok(bytes)
}

/// Deserializes a bounded protocol record.
pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(invalid("exchange message too large"));
    }
    serde_json::from_slice(bytes).map_err(|_| invalid("invalid exchange message"))
}
