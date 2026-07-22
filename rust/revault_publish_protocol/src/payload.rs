use std::fmt;

use sha2::{Digest, Sha256};

const MAGIC: &[u8; 4] = b"LBSP";
const HEADER_LEN: usize = 12;
/// Represents the fingerprint len constant case.
pub const CONTACT_FINGERPRINT_LEN: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
/// Represents payload type.
pub enum PayloadType {
    /// Represents the contact publish case.
    ContactPublish = 1,
    /// Represents the signed key replacement case.
    SignedKeyReplacement = 2,
    /// Represents the unsigned key replacement case.
    UnsignedKeyReplacement = 3,
}

impl PayloadType {
    fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(Self::ContactPublish),
            2 => Some(Self::SignedKeyReplacement),
            3 => Some(Self::UnsignedKeyReplacement),
            _ => None,
        }
    }
}

#[derive(Debug)]
/// Represents payload error.
pub enum PayloadError {
    /// Represents the too short case.
    TooShort,
    /// Represents the bad magic case.
    BadMagic,
    /// Represents the unsupported version case.
    UnsupportedVersion,
    /// Represents the unknown type case.
    UnknownType,
    /// Represents the trailing bytes case.
    TrailingBytes,
    /// Represents the field too large case.
    FieldTooLarge,
    /// Represents the missing field case.
    MissingField,
    /// Represents the invalid field case.
    InvalidField,
}

impl fmt::Display for PayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PayloadError {}

/// Returns the validate payload.
pub fn validate_payload(bytes: &[u8]) -> Result<PayloadType, PayloadError> {
    if bytes.len() < HEADER_LEN {
        return Err(PayloadError::TooShort);
    }
    if &bytes[0..4] != MAGIC {
        return Err(PayloadError::BadMagic);
    }
    let version = read_u16(bytes, 4);
    if version != 1 {
        return Err(PayloadError::UnsupportedVersion);
    }
    let payload_type =
        PayloadType::from_u16(read_u16(bytes, 6)).ok_or(PayloadError::UnknownType)?;
    let body_len = read_u32(bytes, 8) as usize;
    if bytes.len() != HEADER_LEN + body_len {
        return Err(PayloadError::TrailingBytes);
    }
    let mut reader = PayloadReader::new(&bytes[HEADER_LEN..]);
    match payload_type {
        PayloadType::ContactPublish => validate_contact_publish(&mut reader)?,
        PayloadType::SignedKeyReplacement => validate_signed_key_replacement(&mut reader)?,
        PayloadType::UnsignedKeyReplacement => validate_unsigned_key_replacement(&mut reader)?,
    }
    if !reader.is_done() {
        return Err(PayloadError::TrailingBytes);
    }
    Ok(payload_type)
}

/// Encodes contact publish.
pub fn encode_contact_publish(
    profile: &str,
    public_key: &[u8],
    signing_public_key: &[u8],
    fingerprint: &[u8],
    publish_nonce: &[u8],
    created_at_unix_ms: u64,
    expires_at_unix_ms: u64,
) -> Vec<u8> {
    let mut body = Vec::new();
    put_string(&mut body, profile);
    put_bytes(&mut body, public_key);
    put_bytes(&mut body, signing_public_key);
    put_bytes(&mut body, fingerprint);
    put_bytes(&mut body, publish_nonce);
    put_u64(&mut body, created_at_unix_ms);
    put_u64(&mut body, expires_at_unix_ms);
    encode_payload(PayloadType::ContactPublish, &body)
}

/// Returns the normalize contact email.
pub fn normalize_contact_email(email: &str) -> Result<String, PayloadError> {
    let normalized = email.trim().to_ascii_lowercase();
    if normalized.is_empty()
        || normalized.len() > 254
        || normalized.bytes().any(|byte| byte.is_ascii_control())
        || !normalized.contains('@')
        || normalized.starts_with('@')
        || normalized.ends_with('@')
    {
        return Err(PayloadError::InvalidField);
    }
    Ok(normalized)
}

/// Returns the contact fingerprint.
pub fn contact_fingerprint(
    email: &str,
    contact_public_key: &[u8],
    signing_public_key: &[u8],
) -> Result<Vec<u8>, PayloadError> {
    let email = normalize_contact_email(email)?;
    let mut hasher = Sha256::new();
    update_fingerprint_field(&mut hasher, b"revault-contact-fingerprint-v1");
    update_fingerprint_field(&mut hasher, email.as_bytes());
    update_fingerprint_field(&mut hasher, b"contact-public-key-bytes-v1");
    update_fingerprint_field(&mut hasher, contact_public_key);
    update_fingerprint_field(&mut hasher, b"owner-signing-public-key-bytes-v1");
    update_fingerprint_field(&mut hasher, signing_public_key);
    Ok(hasher.finalize()[..CONTACT_FINGERPRINT_LEN].to_vec())
}

fn update_fingerprint_field(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u32).to_be_bytes());
    hasher.update(value);
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents decoded contact publish.
pub struct DecodedContactPublish {
    /// Represents the profile carried by this record case.
    pub profile: String,
    /// Represents the public key carried by this record case.
    pub public_key: Vec<u8>,
    /// Represents the signing public key carried by this record case.
    pub signing_public_key: Vec<u8>,
    /// Represents the fingerprint carried by this record case.
    pub fingerprint: Vec<u8>,
    /// Represents the publish nonce carried by this record case.
    pub publish_nonce: Vec<u8>,
    /// Represents the created at unix ms carried by this record case.
    pub created_at_unix_ms: u64,
    /// Represents the expires at unix ms carried by this record case.
    pub expires_at_unix_ms: u64,
}

/// Decodes contact publish.
pub fn decode_contact_publish(bytes: &[u8]) -> Result<DecodedContactPublish, PayloadError> {
    if validate_payload(bytes)? != PayloadType::ContactPublish {
        return Err(PayloadError::UnknownType);
    }
    let body_len = read_u32(bytes, 8) as usize;
    let mut reader = PayloadReader::new(&bytes[HEADER_LEN..HEADER_LEN + body_len]);
    Ok(DecodedContactPublish {
        profile: reader.string(254)?,
        public_key: reader.bytes(4096)?,
        signing_public_key: reader.bytes(4096)?,
        fingerprint: reader.bytes(128)?,
        publish_nonce: reader.bytes(64)?,
        created_at_unix_ms: reader.u64()?,
        expires_at_unix_ms: reader.u64()?,
    })
}

/// Represents signed key replacement.
pub struct SignedKeyReplacement<'a> {
    /// Represents the profile carried by this record case.
    pub profile: &'a str,
    /// Represents the old fingerprint carried by this record case.
    pub old_fingerprint: &'a [u8],
    /// Represents the new public key carried by this record case.
    pub new_public_key: &'a [u8],
    /// Represents the new signing public key carried by this record case.
    pub new_signing_public_key: &'a [u8],
    /// Represents the new fingerprint carried by this record case.
    pub new_fingerprint: &'a [u8],
    /// Represents the replacement nonce carried by this record case.
    pub replacement_nonce: &'a [u8],
    /// Represents the signature by old key carried by this record case.
    pub signature_by_old_key: &'a [u8],
    /// Represents the created at unix ms carried by this record case.
    pub created_at_unix_ms: u64,
    /// Represents the expires at unix ms carried by this record case.
    pub expires_at_unix_ms: u64,
}

/// Represents key replacement.
pub type KeyReplacement<'a> = SignedKeyReplacement<'a>;

/// Encodes key replacement.
pub fn encode_key_replacement(replacement: KeyReplacement<'_>) -> Vec<u8> {
    encode_signed_key_replacement(replacement)
}

/// Encodes signed key replacement.
pub fn encode_signed_key_replacement(replacement: SignedKeyReplacement<'_>) -> Vec<u8> {
    let mut body = Vec::new();
    put_string(&mut body, replacement.profile);
    put_bytes(&mut body, replacement.old_fingerprint);
    put_bytes(&mut body, replacement.new_public_key);
    put_bytes(&mut body, replacement.new_signing_public_key);
    put_bytes(&mut body, replacement.new_fingerprint);
    put_bytes(&mut body, replacement.replacement_nonce);
    put_bytes(&mut body, replacement.signature_by_old_key);
    put_u64(&mut body, replacement.created_at_unix_ms);
    put_u64(&mut body, replacement.expires_at_unix_ms);
    encode_payload(PayloadType::SignedKeyReplacement, &body)
}

/// Represents unsigned key replacement.
pub struct UnsignedKeyReplacement<'a> {
    /// Represents the profile carried by this record case.
    pub profile: &'a str,
    /// Represents the old fingerprint carried by this record case.
    pub old_fingerprint: &'a [u8],
    /// Represents the new public key carried by this record case.
    pub new_public_key: &'a [u8],
    /// Represents the new signing public key carried by this record case.
    pub new_signing_public_key: &'a [u8],
    /// Represents the new fingerprint carried by this record case.
    pub new_fingerprint: &'a [u8],
    /// Represents the replacement nonce carried by this record case.
    pub replacement_nonce: &'a [u8],
    /// Represents the created at unix ms carried by this record case.
    pub created_at_unix_ms: u64,
    /// Represents the expires at unix ms carried by this record case.
    pub expires_at_unix_ms: u64,
}

/// Encodes unsigned key replacement.
pub fn encode_unsigned_key_replacement(replacement: UnsignedKeyReplacement<'_>) -> Vec<u8> {
    let mut body = Vec::new();
    put_string(&mut body, replacement.profile);
    put_bytes(&mut body, replacement.old_fingerprint);
    put_bytes(&mut body, replacement.new_public_key);
    put_bytes(&mut body, replacement.new_signing_public_key);
    put_bytes(&mut body, replacement.new_fingerprint);
    put_bytes(&mut body, replacement.replacement_nonce);
    put_u64(&mut body, replacement.created_at_unix_ms);
    put_u64(&mut body, replacement.expires_at_unix_ms);
    encode_payload(PayloadType::UnsignedKeyReplacement, &body)
}

fn validate_contact_publish(reader: &mut PayloadReader<'_>) -> Result<(), PayloadError> {
    let profile = reader.string(254)?;
    validate_profile(&profile)?;
    let public_key = reader.bytes(4096)?;
    validate_non_empty(&public_key)?;
    let signing_public_key = reader.bytes(4096)?;
    validate_non_empty(&signing_public_key)?;
    let fingerprint = reader.bytes(128)?;
    validate_fingerprint(&fingerprint)?;
    let nonce = reader.bytes(64)?;
    validate_nonce(&nonce)?;
    let created_at = reader.u64()?;
    let expires_at = reader.u64()?;
    validate_times(created_at, expires_at)
}

fn validate_signed_key_replacement(reader: &mut PayloadReader<'_>) -> Result<(), PayloadError> {
    let profile = reader.string(254)?;
    validate_profile(&profile)?;
    let old_fingerprint = reader.bytes(128)?;
    validate_fingerprint(&old_fingerprint)?;
    let new_public_key = reader.bytes(4096)?;
    validate_non_empty(&new_public_key)?;
    let new_signing_public_key = reader.bytes(4096)?;
    validate_non_empty(&new_signing_public_key)?;
    let new_fingerprint = reader.bytes(128)?;
    validate_fingerprint(&new_fingerprint)?;
    let nonce = reader.bytes(64)?;
    validate_nonce(&nonce)?;
    let signature = reader.bytes(4096)?;
    validate_non_empty(&signature)?;
    let created_at = reader.u64()?;
    let expires_at = reader.u64()?;
    validate_times(created_at, expires_at)
}

fn validate_unsigned_key_replacement(reader: &mut PayloadReader<'_>) -> Result<(), PayloadError> {
    let profile = reader.string(254)?;
    validate_profile(&profile)?;
    let old_fingerprint = reader.bytes(128)?;
    validate_fingerprint(&old_fingerprint)?;
    let new_public_key = reader.bytes(4096)?;
    validate_non_empty(&new_public_key)?;
    let new_signing_public_key = reader.bytes(4096)?;
    validate_non_empty(&new_signing_public_key)?;
    let new_fingerprint = reader.bytes(128)?;
    validate_fingerprint(&new_fingerprint)?;
    let nonce = reader.bytes(64)?;
    validate_nonce(&nonce)?;
    let created_at = reader.u64()?;
    let expires_at = reader.u64()?;
    validate_times(created_at, expires_at)
}

fn validate_profile(profile: &str) -> Result<(), PayloadError> {
    if profile.is_empty()
        || profile.len() > 254
        || profile
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        return Err(PayloadError::InvalidField);
    }
    Ok(())
}

fn validate_fingerprint(fingerprint: &[u8]) -> Result<(), PayloadError> {
    if !(16..=128).contains(&fingerprint.len()) {
        return Err(PayloadError::InvalidField);
    }
    Ok(())
}

fn validate_nonce(nonce: &[u8]) -> Result<(), PayloadError> {
    if !(16..=64).contains(&nonce.len()) {
        return Err(PayloadError::InvalidField);
    }
    Ok(())
}

fn validate_non_empty(bytes: &[u8]) -> Result<(), PayloadError> {
    if bytes.is_empty() {
        return Err(PayloadError::MissingField);
    }
    Ok(())
}

fn validate_times(created_at: u64, expires_at: u64) -> Result<(), PayloadError> {
    if created_at == 0 || expires_at <= created_at {
        return Err(PayloadError::InvalidField);
    }
    Ok(())
}

struct PayloadReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> PayloadReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn is_done(&self) -> bool {
        self.offset == self.bytes.len()
    }

    fn u64(&mut self) -> Result<u64, PayloadError> {
        if self.offset + 8 > self.bytes.len() {
            return Err(PayloadError::TooShort);
        }
        let value = u64::from_be_bytes([
            self.bytes[self.offset],
            self.bytes[self.offset + 1],
            self.bytes[self.offset + 2],
            self.bytes[self.offset + 3],
            self.bytes[self.offset + 4],
            self.bytes[self.offset + 5],
            self.bytes[self.offset + 6],
            self.bytes[self.offset + 7],
        ]);
        self.offset += 8;
        Ok(value)
    }

    fn string(&mut self, max_len: usize) -> Result<String, PayloadError> {
        let bytes = self.bytes(max_len)?;
        String::from_utf8(bytes).map_err(|_| PayloadError::InvalidField)
    }

    fn bytes(&mut self, max_len: usize) -> Result<Vec<u8>, PayloadError> {
        if self.offset + 4 > self.bytes.len() {
            return Err(PayloadError::TooShort);
        }
        let len = read_u32(self.bytes, self.offset) as usize;
        self.offset += 4;
        if len > max_len {
            return Err(PayloadError::FieldTooLarge);
        }
        if self.offset + len > self.bytes.len() {
            return Err(PayloadError::TooShort);
        }
        let out = self.bytes[self.offset..self.offset + len].to_vec();
        self.offset += len;
        Ok(out)
    }
}

fn encode_payload(payload_type: PayloadType, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN + body.len());
    out.extend_from_slice(MAGIC);
    put_u16(&mut out, 1);
    put_u16(&mut out, payload_type as u16);
    put_u32(&mut out, body.len() as u32);
    out.extend_from_slice(body);
    out
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    put_bytes(out, value.as_bytes());
}

fn put_bytes(out: &mut Vec<u8>, value: &[u8]) {
    put_u32(out, value.len() as u32);
    out.extend_from_slice(value);
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}
