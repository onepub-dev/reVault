#![deny(missing_docs)]

//! End-to-end encrypted approval requests and single-use unlock grants.
//!
//! The mailbox relay transports the encoded sealed records produced here. It
//! never receives the keys required to inspect either direction.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use base64ct::{Base64UrlUnpadded, Encoding};
use revault_lockbox_api::{
    HybridDetachedSignature, OwnerSigningKeyPair, OwnerSigningPublicKey, RecipientKeyPair,
    RecipientPublicKey, RecipientWrappedKey, SecretVec,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use zeroize::Zeroize;

/// Current approval protocol version.
pub const APPROVAL_PROTOCOL_VERSION: u16 = 1;
/// Maximum encoded request or response accepted by clients and relays.
pub const MAX_ENVELOPE_BYTES: usize = 16 * 1024;
/// Maximum request lifetime accepted by the protocol.
pub const MAX_REQUEST_LIFETIME_MS: u64 = 2 * 60 * 1000;
const ENVELOPE_AAD: &[u8] = b"revault-approval-envelope-v1";

/// Approval protocol failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    /// An encoded record is malformed or exceeds a protocol limit.
    InvalidEncoding(String),
    /// A request is expired, too far in the future, or otherwise invalid.
    InvalidRequest(String),
    /// Encryption, decryption, or signature verification failed.
    AuthenticationFailed,
    /// A request or grant has already been consumed.
    Replay,
    /// Secure random generation failed.
    Random(String),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEncoding(message) => write!(formatter, "invalid approval encoding: {message}"),
            Self::InvalidRequest(message) => write!(formatter, "invalid approval request: {message}"),
            Self::AuthenticationFailed => formatter.write_str("approval authentication failed"),
            Self::Replay => formatter.write_str("approval request has already been consumed"),
            Self::Random(message) => write!(formatter, "approval random generation failed: {message}"),
        }
    }
}

impl std::error::Error for ProtocolError {}

/// Approval protocol result.
pub type Result<T> = std::result::Result<T, ProtocolError>;

/// Cryptographically random request identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RequestId([u8; 32]);

impl RequestId {
    /// Generates a fresh request identifier.
    pub fn new_random() -> Result<Self> {
        let mut bytes = [0_u8; 32];
        getrandom::fill(&mut bytes).map_err(|error| ProtocolError::Random(error.to_string()))?;
        Ok(Self(bytes))
    }

    /// Creates an identifier from bytes.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the stable byte representation.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Operation requiring approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalOperation {
    /// Open a lockbox for read-only access.
    UnlockRead,
}

/// Secret evidence bytes that are wiped on drop.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct SecretPayload(#[serde(with = "base64_vec")] Vec<u8>);

impl SecretPayload {
    /// Takes ownership of secret evidence bytes.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Borrows the secret bytes for immediate verification.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn into_vec(mut self) -> Vec<u8> {
        std::mem::take(&mut self.0)
    }
}

impl Drop for SecretPayload {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// Evidence used by the phone to authenticate the request source.
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceEvidence {
    /// Desktop-generated request signature. This authenticates the enrolled
    /// machine but cannot distinguish same-user processes.
    LocalDesktop {
        /// Signature over the canonical unsigned request fields.
        #[serde(with = "base64_vec")]
        signature: Vec<u8>,
    },
    /// Provider-issued OIDC workload token, encrypted to the phone.
    Oidc {
        /// Compact JWT bytes. The phone must verify its signature and policy.
        token: SecretPayload,
    },
}

/// Serialized hybrid recipient slot included in an approval request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipientSlotCiphertext {
    /// Slot identifier in the lockbox key directory.
    pub slot_id: u64,
    /// Ephemeral X25519 public key.
    #[serde(with = "base64_vec")]
    pub x25519_ephemeral_public_key: Vec<u8>,
    /// ML-KEM-768 ciphertext.
    #[serde(with = "base64_vec")]
    pub mlkem_ciphertext: Vec<u8>,
    /// Authenticated encrypted content key.
    #[serde(with = "base64_vec")]
    pub encrypted_content_key: Vec<u8>,
}

impl RecipientSlotCiphertext {
    /// Captures a wrapped recipient slot for transport to its phone.
    pub fn from_wrapped(slot_id: u64, wrapped: &RecipientWrappedKey) -> Self {
        Self {
            slot_id,
            x25519_ephemeral_public_key: wrapped.x25519_ephemeral_public_key().to_vec(),
            mlkem_ciphertext: wrapped.ciphertext_bytes().to_vec(),
            encrypted_content_key: wrapped.encrypted_key().to_vec(),
        }
    }

    /// Reconstructs the cryptographic wrapped-key value.
    pub fn to_wrapped(&self) -> Result<RecipientWrappedKey> {
        RecipientWrappedKey::from_parts(
            self.x25519_ephemeral_public_key.clone(),
            self.mlkem_ciphertext.clone(),
            self.encrypted_content_key.clone(),
        )
        .map_err(|error| ProtocolError::InvalidEncoding(error.to_string()))
    }
}

/// Plain approval request visible only to the enrolled phone.
#[derive(Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Protocol version.
    pub version: u16,
    /// Globally unique one-time request id.
    pub request_id: RequestId,
    /// Independent random challenge bound into the response.
    pub challenge: [u8; 32],
    /// Target device id.
    pub device_id: [u8; 16],
    /// Enrolled source id whose local policy must be applied.
    pub source_id: [u8; 16],
    /// Target lockbox id.
    pub lockbox_id: [u8; 16],
    /// Requested operation.
    pub operation: ApprovalOperation,
    /// Digest of the exact operation parameters.
    pub operation_digest: [u8; 32],
    /// Human-readable context supplied by the requester. This is never marked
    /// as cryptographically verified by the UI.
    pub unverified_summary: String,
    /// Request creation time in Unix milliseconds.
    pub created_at_unix_ms: u64,
    /// Request expiry time in Unix milliseconds.
    pub expires_at_unix_ms: u64,
    /// One-time reply recipient public key encoded as a versioned record.
    #[serde(with = "base64_vec")]
    pub reply_public_key: Vec<u8>,
    /// Candidate recipient slots. The phone releases a grant only when one can
    /// be unwrapped by its locally protected recipient key.
    pub recipient_slots: Vec<RecipientSlotCiphertext>,
    /// Authenticated source evidence.
    pub source_evidence: SourceEvidence,
}

impl ApprovalRequest {
    /// Validates structural and time invariants without validating source evidence.
    pub fn validate(&self, now_unix_ms: u64) -> Result<()> {
        if self.version != APPROVAL_PROTOCOL_VERSION {
            return Err(ProtocolError::InvalidRequest(
                "unsupported protocol version".to_string(),
            ));
        }
        if self.expires_at_unix_ms <= now_unix_ms
            || self.expires_at_unix_ms <= self.created_at_unix_ms
            || self.expires_at_unix_ms - self.created_at_unix_ms > MAX_REQUEST_LIFETIME_MS
        {
            return Err(ProtocolError::InvalidRequest(
                "request is expired or exceeds the two-minute lifetime".to_string(),
            ));
        }
        if self.unverified_summary.len() > 512 || self.recipient_slots.is_empty() {
            return Err(ProtocolError::InvalidRequest(
                "request summary or recipient slots are invalid".to_string(),
            ));
        }
        RecipientPublicKey::from_bytes(&self.reply_public_key)
            .map_err(|error| ProtocolError::InvalidRequest(error.to_string()))?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct SealedEnvelope {
    version: u16,
    #[serde(with = "base64_vec")]
    wrapped_x25519: Vec<u8>,
    #[serde(with = "base64_vec")]
    wrapped_mlkem: Vec<u8>,
    #[serde(with = "base64_vec")]
    wrapped_key: Vec<u8>,
    nonce: [u8; 12],
    #[serde(with = "base64_vec")]
    ciphertext: Vec<u8>,
}

/// Encrypts an approval request to a phone transport key.
pub fn seal_request(request: &ApprovalRequest, phone_key: &RecipientPublicKey) -> Result<Vec<u8>> {
    seal_json(request, phone_key)
}

/// Decrypts and validates an approval request on the phone.
pub fn open_request(
    encoded: &[u8],
    phone_key: &RecipientKeyPair,
    now_unix_ms: u64,
) -> Result<ApprovalRequest> {
    let request: ApprovalRequest = open_json(encoded, phone_key)?;
    request.validate(now_unix_ms)?;
    Ok(request)
}

#[derive(Serialize, Deserialize)]
struct GrantPayload {
    version: u16,
    request_id: RequestId,
    challenge: [u8; 32],
    source_id: [u8; 16],
    lockbox_id: [u8; 16],
    operation_digest: [u8; 32],
    issued_at_unix_ms: u64,
    expires_at_unix_ms: u64,
    content_key: SecretPayload,
}

#[derive(Serialize, Deserialize)]
struct SignedGrant {
    version: u16,
    #[serde(with = "base64_vec")]
    envelope: Vec<u8>,
    #[serde(with = "base64_vec")]
    signature: Vec<u8>,
}

/// Successfully opened, request-bound content key.
pub struct OpenedApprovalGrant {
    /// Request id consumed by this grant.
    pub request_id: RequestId,
    /// Lockbox id whose key was released.
    pub lockbox_id: [u8; 16],
    content_key: SecretVec,
}

impl OpenedApprovalGrant {
    /// Borrows the content key only for the duration of `callback`.
    pub fn with_content_key<T>(&self, callback: impl FnOnce(&[u8]) -> T) -> Result<T> {
        self.content_key
            .with_bytes(callback)
            .map_err(|_| ProtocolError::AuthenticationFailed)
    }
}

/// Creates a signed, reply-key-encrypted one-time approval grant.
pub fn seal_approval_grant(
    request: &ApprovalRequest,
    content_key: &SecretVec,
    response_signing_key: &OwnerSigningKeyPair,
    now_unix_ms: u64,
) -> Result<Vec<u8>> {
    request.validate(now_unix_ms)?;
    let reply_key = RecipientPublicKey::from_bytes(&request.reply_public_key)
        .map_err(|_| ProtocolError::AuthenticationFailed)?;
    let content_key = content_key
        .with_bytes(|bytes| SecretPayload::new(bytes.to_vec()))
        .map_err(|_| ProtocolError::AuthenticationFailed)?;
    let payload = GrantPayload {
        version: APPROVAL_PROTOCOL_VERSION,
        request_id: request.request_id,
        challenge: request.challenge,
        source_id: request.source_id,
        lockbox_id: request.lockbox_id,
        operation_digest: request.operation_digest,
        issued_at_unix_ms: now_unix_ms,
        expires_at_unix_ms: request.expires_at_unix_ms,
        content_key,
    };
    let envelope = seal_json(&payload, &reply_key)?;
    let signature = response_signing_key.sign_detached(&envelope).to_bytes();
    encode_limited(&SignedGrant {
        version: APPROVAL_PROTOCOL_VERSION,
        envelope,
        signature,
    })
}

/// Verifies, decrypts, and binds a grant to the outstanding request.
pub fn open_approval_grant(
    encoded: &[u8],
    reply_key: &RecipientKeyPair,
    response_verification_key: &OwnerSigningPublicKey,
    expected: &ApprovalRequest,
    now_unix_ms: u64,
) -> Result<OpenedApprovalGrant> {
    let signed: SignedGrant = decode_limited(encoded)?;
    if signed.version != APPROVAL_PROTOCOL_VERSION {
        return Err(ProtocolError::InvalidEncoding(
            "unsupported signed grant version".to_string(),
        ));
    }
    let signature = HybridDetachedSignature::from_bytes(&signed.signature)
        .map_err(|_| ProtocolError::AuthenticationFailed)?;
    response_verification_key
        .verify_detached(&signed.envelope, &signature)
        .map_err(|_| ProtocolError::AuthenticationFailed)?;
    let grant: GrantPayload = open_json(&signed.envelope, reply_key)?;
    if grant.version != APPROVAL_PROTOCOL_VERSION
        || grant.request_id != expected.request_id
        || grant.challenge != expected.challenge
        || grant.source_id != expected.source_id
        || grant.lockbox_id != expected.lockbox_id
        || grant.operation_digest != expected.operation_digest
        || grant.expires_at_unix_ms != expected.expires_at_unix_ms
        || grant.issued_at_unix_ms < expected.created_at_unix_ms
        || now_unix_ms >= grant.expires_at_unix_ms
    {
        return Err(ProtocolError::AuthenticationFailed);
    }
    let content_key = SecretVec::try_from_vec(grant.content_key.into_vec())
        .map_err(|_| ProtocolError::AuthenticationFailed)?;
    if content_key.len() != 32 {
        return Err(ProtocolError::AuthenticationFailed);
    }
    Ok(OpenedApprovalGrant {
        request_id: grant.request_id,
        lockbox_id: grant.lockbox_id,
        content_key,
    })
}

/// In-memory replay cache used by clients in addition to relay-side atomic consumption.
#[derive(Debug, Default)]
pub struct ReplayCache {
    consumed: BTreeMap<RequestId, u64>,
}

impl ReplayCache {
    /// Atomically records a request id as consumed and rejects duplicates.
    pub fn consume(&mut self, request_id: RequestId, expires_at_unix_ms: u64, now_unix_ms: u64) -> Result<()> {
        self.consumed.retain(|_, expiry| *expiry > now_unix_ms);
        if self.consumed.insert(request_id, expires_at_unix_ms).is_some() {
            return Err(ProtocolError::Replay);
        }
        Ok(())
    }
}

fn seal_json<T: Serialize>(value: &T, recipient: &RecipientPublicKey) -> Result<Vec<u8>> {
    let mut plaintext = serde_json::to_vec(value)
        .map_err(|error| ProtocolError::InvalidEncoding(error.to_string()))?;
    let mut envelope_key = [0_u8; 32];
    getrandom::fill(&mut envelope_key).map_err(|error| ProtocolError::Random(error.to_string()))?;
    let wrapped = recipient
        .encrypt(&envelope_key)
        .map_err(|_| ProtocolError::AuthenticationFailed)?;
    let mut nonce = [0_u8; 12];
    getrandom::fill(&mut nonce).map_err(|error| ProtocolError::Random(error.to_string()))?;
    let cipher = ChaCha20Poly1305::new(&Key::from(envelope_key));
    let ciphertext = cipher
        .encrypt(
            &Nonce::from(nonce),
            Payload {
                msg: &plaintext,
                aad: ENVELOPE_AAD,
            },
        )
        .map_err(|_| ProtocolError::AuthenticationFailed)?;
    plaintext.zeroize();
    envelope_key.zeroize();
    encode_limited(&SealedEnvelope {
        version: APPROVAL_PROTOCOL_VERSION,
        wrapped_x25519: wrapped.x25519_ephemeral_public_key().to_vec(),
        wrapped_mlkem: wrapped.ciphertext_bytes().to_vec(),
        wrapped_key: wrapped.encrypted_key().to_vec(),
        nonce,
        ciphertext,
    })
}

fn open_json<T: for<'de> Deserialize<'de>>(encoded: &[u8], recipient: &RecipientKeyPair) -> Result<T> {
    let envelope: SealedEnvelope = decode_limited(encoded)?;
    if envelope.version != APPROVAL_PROTOCOL_VERSION {
        return Err(ProtocolError::InvalidEncoding(
            "unsupported envelope version".to_string(),
        ));
    }
    let wrapped = RecipientWrappedKey::from_parts(
        envelope.wrapped_x25519,
        envelope.wrapped_mlkem,
        envelope.wrapped_key,
    )
    .map_err(|_| ProtocolError::AuthenticationFailed)?;
    let mut envelope_key = recipient
        .decrypt(&wrapped)
        .map_err(|_| ProtocolError::AuthenticationFailed)?;
    if envelope_key.len() != 32 {
        envelope_key.zeroize();
        return Err(ProtocolError::AuthenticationFailed);
    }
    let mut key = [0_u8; 32];
    key.copy_from_slice(&envelope_key);
    envelope_key.zeroize();
    let cipher = ChaCha20Poly1305::new(&Key::from(key));
    key.zeroize();
    let mut plaintext = cipher
        .decrypt(
            &Nonce::from(envelope.nonce),
            Payload {
                msg: &envelope.ciphertext,
                aad: ENVELOPE_AAD,
            },
        )
        .map_err(|_| ProtocolError::AuthenticationFailed)?;
    let value = serde_json::from_slice(&plaintext)
        .map_err(|error| ProtocolError::InvalidEncoding(error.to_string()));
    plaintext.zeroize();
    value
}

fn encode_limited<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| ProtocolError::InvalidEncoding(error.to_string()))?;
    if bytes.len() > MAX_ENVELOPE_BYTES {
        return Err(ProtocolError::InvalidEncoding(
            "envelope exceeds 16 KiB".to_string(),
        ));
    }
    Ok(bytes)
}

fn decode_limited<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T> {
    if bytes.len() > MAX_ENVELOPE_BYTES {
        return Err(ProtocolError::InvalidEncoding(
            "envelope exceeds 16 KiB".to_string(),
        ));
    }
    serde_json::from_slice(bytes)
        .map_err(|error| ProtocolError::InvalidEncoding(error.to_string()))
}

mod base64_vec {
    use super::{Base64UrlUnpadded, Encoding};
    use serde::{Deserialize, Deserializer, Serializer};

    pub(super) fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&Base64UrlUnpadded::encode_string(bytes))
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Vec<u8>, D::Error> {
        let encoded = String::deserialize(deserializer)?;
        Base64UrlUnpadded::decode_vec(&encoded).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(reply_key: &RecipientPublicKey) -> ApprovalRequest {
        let slot_recipient = RecipientKeyPair::generate().unwrap();
        let wrapped = slot_recipient.public_key().encrypt(&[7_u8; 32]).unwrap();
        ApprovalRequest {
            version: APPROVAL_PROTOCOL_VERSION,
            request_id: RequestId::new_random().unwrap(),
            challenge: [3; 32],
            device_id: [4; 16],
            source_id: [5; 16],
            lockbox_id: [6; 16],
            operation: ApprovalOperation::UnlockRead,
            operation_digest: [8; 32],
            unverified_summary: "read deployment token".to_string(),
            created_at_unix_ms: 1_000,
            expires_at_unix_ms: 100_000,
            reply_public_key: reply_key.to_bytes(),
            recipient_slots: vec![RecipientSlotCiphertext::from_wrapped(9, &wrapped)],
            source_evidence: SourceEvidence::LocalDesktop {
                signature: vec![1, 2, 3],
            },
        }
    }

    #[test]
    fn request_and_grant_are_end_to_end_encrypted_and_bound() {
        let phone_transport = RecipientKeyPair::generate().unwrap();
        let reply = RecipientKeyPair::generate().unwrap();
        let response_signing = OwnerSigningKeyPair::generate().unwrap();
        let original = request(&reply.public_key());

        let sealed = seal_request(&original, &phone_transport.public_key()).unwrap();
        assert!(!sealed.windows(21).any(|window| window == b"read deployment token"));
        let opened = open_request(&sealed, &phone_transport, 2_000).unwrap();

        let content_key = SecretVec::try_from_slice(&[11_u8; 32]).unwrap();
        let grant = seal_approval_grant(&opened, &content_key, &response_signing, 3_000).unwrap();
        let opened_grant = open_approval_grant(
            &grant,
            &reply,
            &response_signing.public_key(),
            &opened,
            4_000,
        )
        .unwrap();
        assert_eq!(
            opened_grant.with_content_key(|bytes| bytes.to_vec()).unwrap(),
            vec![11_u8; 32]
        );
    }

    #[test]
    fn replay_cache_rejects_a_second_consumption() {
        let id = RequestId::new_random().unwrap();
        let mut cache = ReplayCache::default();
        cache.consume(id, 10_000, 1_000).unwrap();
        assert_eq!(cache.consume(id, 10_000, 2_000), Err(ProtocolError::Replay));
    }

    #[test]
    fn tampered_grant_fails_authentication() {
        let reply = RecipientKeyPair::generate().unwrap();
        let response_signing = OwnerSigningKeyPair::generate().unwrap();
        let original = request(&reply.public_key());
        let content_key = SecretVec::try_from_slice(&[11_u8; 32]).unwrap();
        let mut grant =
            seal_approval_grant(&original, &content_key, &response_signing, 3_000).unwrap();
        let last = grant.len() - 1;
        grant[last] ^= 1;
        assert!(open_approval_grant(
            &grant,
            &reply,
            &response_signing.public_key(),
            &original,
            4_000,
        )
        .is_err());
    }
}
