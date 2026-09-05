use super::{encode, invalid, token_hash, valid_token, Result};
use revault_lockbox_api::{ContactPublicKey, OwnerSigningKeyPair, OwnerSigningPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Maximum JSON request or response, including both hybrid key bundles.
pub const MAX_MESSAGE_BYTES: usize = 128 * 1024;
/// Maximum invitation lifetime: seven days.
pub const MAX_LIFETIME_MS: u64 = 7 * 24 * 60 * 60 * 1000;

/// Exactly one selected profile's complete public identity.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    /// User-selected profile label; not independently authenticated.
    pub profile: String,
    /// Claimed email, authenticated only by the user's independent comparison.
    pub email: String,
    /// Monotonic local profile generation.
    pub generation: u16,
    /// Canonical encryption public key.
    pub encryption_key: Vec<u8>,
    /// Canonical hybrid signing public key.
    pub signing_key: Vec<u8>,
}

impl Bundle {
    /// Validates bounded, printable identities and canonical public-key encodings.
    pub fn validate(&self) -> Result<()> {
        if self.profile.is_empty()
            || self.profile.len() > 128
            || !self
                .profile
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(invalid("invalid profile name"));
        }
        if self.email.len() > 254
            || self.email.chars().any(char::is_control)
            || crate::normalize_contact_email(&self.email).ok().as_deref() != Some(&self.email)
        {
            return Err(invalid("invalid canonical email"));
        }
        let encryption = ContactPublicKey::from_bytes(&self.encryption_key)
            .map_err(|_| invalid("invalid encryption key"))?;
        let signing = OwnerSigningPublicKey::from_bytes(&self.signing_key)
            .map_err(|_| invalid("invalid signing key"))?;
        if encryption.to_bytes() != self.encryption_key || signing.to_bytes() != self.signing_key {
            return Err(invalid("noncanonical key"));
        }
        Ok(())
    }
}

/// Signed invitation. The random ID binds this exchange independently of email.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalExchange {
    /// Pinned relay origin.
    pub server: String,
    /// Whether this client created the invitation.
    pub inviter: bool,
    /// Private owner or recipient capability; never print this.
    pub token: String,
    /// Pinned signed offer.
    pub offer: Offer,
    /// Pinned reciprocal response, once received.
    pub acceptance: Option<Acceptance>,
    /// Locally verified contact name; remote state cannot set it.
    pub verified_contact: Option<String>,
}

/// Signed invitation. The random ID binds this exchange independently of email.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    /// Wire version.
    pub version: u8,
    /// Random exchange ID, also the invitation capability.
    pub id: String,
    /// Inviter's complete bundle.
    pub inviter: Bundle,
    /// Intended recipient email; not a server assertion of identity.
    pub recipient_email: String,
    /// Creation time in Unix milliseconds.
    pub created_ms: u64,
    /// Signed expiry in Unix milliseconds.
    pub expires_ms: u64,
    /// Hybrid signature over all preceding fields.
    pub signature: Vec<u8>,
}

impl Offer {
    fn message(&self) -> Result<Vec<u8>> {
        encode(&(
            "revault/exchange/offer/v1",
            self.version,
            &self.id,
            &self.inviter,
            &self.recipient_email,
            self.created_ms,
            self.expires_ms,
        ))
    }

    /// Signs an offer using the selected profile's signing key.
    pub fn sign(&mut self, key: &OwnerSigningKeyPair) -> Result<()> {
        if key.public_key().to_bytes() != self.inviter.signing_key {
            return Err(invalid("offer signing key mismatch"));
        }
        self.signature = key.sign_detached(&self.message()?);
        Ok(())
    }

    /// Verifies the signature, identity fields and lifetime.
    pub fn validate(&self, now_ms: u64) -> Result<()> {
        self.inviter.validate()?;
        if self.version != 1
            || !valid_token(&self.id)
            || self.expires_ms <= self.created_ms
            || self.expires_ms - self.created_ms > MAX_LIFETIME_MS
            || self.created_ms > now_ms.saturating_add(60_000)
            || now_ms >= self.expires_ms
            || self.recipient_email.len() > 254
            || self.recipient_email.chars().any(char::is_control)
            || crate::normalize_contact_email(&self.recipient_email)
                .ok()
                .as_deref()
                != Some(&self.recipient_email)
        {
            return Err(invalid("invalid or expired invitation"));
        }
        OwnerSigningPublicKey::from_bytes(&self.inviter.signing_key)
            .and_then(|key| {
                key.verify_detached(
                    &self
                        .message()
                        .map_err(|_| revault_lockbox_api::Error::CorruptRecord)?,
                    &self.signature,
                )
            })
            .map_err(|_| invalid("invalid invitation signature"))
    }
}

/// Reply signed over the exact offer and recipient bundle.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Acceptance {
    /// Complete selected recipient profile.
    pub recipient: Bundle,
    /// Signature binding both directions.
    pub signature: Vec<u8>,
}

impl Acceptance {
    fn message(&self, offer: &Offer) -> Result<Vec<u8>> {
        encode(&("revault/exchange/accept/v1", offer, &self.recipient))
    }

    /// Creates a signed reciprocal response. There is no receive-only mode.
    pub fn new(offer: &Offer, recipient: Bundle, key: &OwnerSigningKeyPair) -> Result<Self> {
        if key.public_key().to_bytes() != recipient.signing_key {
            return Err(invalid("acceptance signing key mismatch"));
        }
        let mut value = Self {
            recipient,
            signature: Vec::new(),
        };
        value.signature = key.sign_detached(&value.message(offer)?);
        Ok(value)
    }

    /// Validates recipient identity and the binding to the complete offer.
    pub fn validate(&self, offer: &Offer) -> Result<()> {
        self.recipient.validate()?;
        if self.recipient.email != offer.recipient_email {
            return Err(invalid(
                "selected profile email is not the invitation recipient",
            ));
        }
        OwnerSigningPublicKey::from_bytes(&self.recipient.signing_key)
            .and_then(|key| {
                key.verify_detached(
                    &self
                        .message(offer)
                        .map_err(|_| revault_lockbox_api::Error::CorruptRecord)?,
                    &self.signature,
                )
            })
            .map_err(|_| invalid("invalid acceptance signature"))
    }
}

/// Computes a full 256-bit shared fingerprint, with explicit participant roles.
/// Both parties must compare the entire result through a trusted second channel.
pub fn verification(offer: &Offer, acceptance: &Acceptance) -> Result<String> {
    let bytes = encode(&("revault/exchange/verification/v1", offer, acceptance))?;
    let digest = super::hex(&Sha256::digest(bytes));
    Ok(digest
        .as_bytes()
        .chunks(4)
        .map(|part| String::from_utf8_lossy(part).into_owned())
        .collect::<Vec<_>>()
        .join("-"))
}

/// Accepts only an exact, full fingerprint, ignoring display separators/case.
pub fn verification_matches(expected: &str, supplied: &str) -> bool {
    fn normalize(value: &str) -> String {
        value
            .chars()
            .filter(|c| *c != '-' && !c.is_ascii_whitespace())
            .flat_map(char::to_lowercase)
            .collect()
    }
    let expected = normalize(expected);
    let supplied = normalize(supplied);
    expected.len() == 64 && supplied.len() == 64 && expected == supplied
}

/// Request to the invitation-scoped relay.
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "operation", deny_unknown_fields)]
pub enum Request {
    /// Idempotently create an invitation; owner token is private to the inviter.
    Create {
        /// Signed offer.
        offer: Offer,
        /// Private management capability.
        owner_token: String,
    },
    /// Retrieve only the offer using the invitation capability.
    Inspect {
        /// Invitation capability.
        id: String,
    },
    /// Freeze a reciprocal response, with a separate recipient retry capability.
    Accept {
        /// Invitation capability.
        id: String,
        /// Recipient management capability.
        recipient_token: String,
        /// Signed response.
        acceptance: Acceptance,
    },
    /// Fetch the complete exchange using either private management capability.
    Poll {
        /// Exchange ID.
        id: String,
        /// Owner or recipient capability.
        token: String,
    },
    /// Acknowledge durable receipt; delete payloads only after both acknowledge.
    Complete {
        /// Exchange ID.
        id: String,
        /// Owner or recipient capability.
        token: String,
    },
    /// Cancel an unaccepted invitation.
    Cancel {
        /// Exchange ID.
        id: String,
        /// Inviter capability.
        owner_token: String,
    },
}

/// Relay response. It never asserts human identity or local trust.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    /// Public signed offer, absent after cleanup.
    pub offer: Option<Offer>,
    /// Public signed acceptance, visible only with a management capability.
    pub acceptance: Option<Acceptance>,
    /// Both sides acknowledged durable delivery.
    pub complete: bool,
    /// Safe, capability-free error description.
    pub error: Option<String>,
}

/// On-disk relay record. Bearer capabilities are stored only as hashes.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    /// Signed offer.
    pub offer: Offer,
    /// Hashed inviter management capability.
    pub owner_hash: String,
    /// Hashed recipient management capability.
    pub recipient_hash: Option<String>,
    /// Immutable reciprocal response.
    pub acceptance: Option<Acceptance>,
    /// Inviter has durably stored the response.
    pub owner_done: bool,
    /// Recipient has durably stored both bundles.
    pub recipient_done: bool,
}

impl Record {
    /// Checks the owner capability without persisting its plaintext.
    pub fn is_owner(&self, token: &str) -> bool {
        valid_token(token) && token_hash(token) == self.owner_hash
    }
    /// Checks the recipient capability.
    pub fn is_recipient(&self, token: &str) -> bool {
        valid_token(token) && self.recipient_hash.as_deref() == Some(&token_hash(token))
    }
}
