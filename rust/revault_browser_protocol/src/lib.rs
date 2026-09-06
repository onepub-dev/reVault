//! Versioned, ciphertext-only browser delegation messages.
//! No generic agent commands or local paths are part of this protocol.
#![deny(unsafe_op_in_unsafe_fn)]

pub mod crypto;
pub mod receiver;
pub mod transport;
pub mod validation;
/// Protobuf schema, generated from the checked-in protocol definition.
// Generated protobuf messages are bounded by MAX_PROTO_BYTES at transport boundaries.
#[allow(clippy::large_enum_variant)]
pub mod wire;
pub use prost::Message;
pub use wire::*;

/// Initial protocol version.
pub const VERSION: u32 = 1;
/// Maximum decoded protobuf size.
pub const MAX_PROTO_BYTES: usize = 16 * 1024;
/// Entire-lockbox access. This includes every secret in the lockbox.
pub const SCOPE: &str = "unlock_entire_lockbox";
/// Domain separation for server request signatures.
pub const SIGNING_DOMAIN: &[u8] = b"reVault browser request v1\0";
/// Domain separation for HPKE info.
pub const HPKE_INFO: &[u8] = b"reVault browser unlock v1\0";

/// Sanitised, stable errors safe to expose across the browser boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidRequest,
    UpgradeRequired,
    UnpairedRecipient,
    Expired,
    Replayed,
    Denied,
    Cancelled,
    Busy,
    VaultLocked,
    LockboxUnavailable,
    SecureStoreUnavailable,
    NativeHelperMissing,
    ApprovalRequired,
    Internal,
}
impl Error {
    /// Stable SDK state; never contains a path, secret, or operating-system error.
    pub fn state(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::UpgradeRequired => "upgrade_required",
            Self::UnpairedRecipient => "unpaired_recipient",
            Self::Expired => "expired",
            Self::Replayed => "replayed",
            Self::Denied => "denied",
            Self::Cancelled => "cancelled",
            Self::Busy => "busy",
            Self::VaultLocked => "vault_locked",
            Self::LockboxUnavailable => "lockbox_unavailable",
            Self::SecureStoreUnavailable => "secure_store_unavailable",
            Self::NativeHelperMissing => "native_helper_missing",
            Self::ApprovalRequired => "approval_required",
            Self::Internal => "internal_error",
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.state())
    }
}
impl std::error::Error for Error {}

/// Construct a response without exposing implementation errors.
pub fn response(result: Result<Option<UnlockEnvelope>, Error>) -> BrowserResponse {
    match result {
        Ok(envelope) => BrowserResponse {
            protocol_version: VERSION,
            state: "ok".into(),
            envelope,
        },
        Err(error) => BrowserResponse {
            protocol_version: VERSION,
            state: error.state().into(),
            envelope: None,
        },
    }
}
