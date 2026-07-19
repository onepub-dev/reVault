use std::fmt;

/// Represents the magic constant case.
pub const MAGIC: &[u8; 4] = b"LBSR";
/// Represents the version constant case.
pub const VERSION: u16 = 1;
/// Represents the version constant case.
pub const MESSAGE_VERSION: u16 = 1;
const ENVELOPE_LEN: usize = 14;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
/// Represents operation.
pub enum Operation {
    /// Represents the publish case.
    Publish = 1,
    /// Represents the receive case.
    Receive = 2,
    /// Represents the delete case.
    Delete = 3,
    /// Represents the replicate case.
    Replicate = 4,
}

impl Operation {
    /// Returns the from u16.
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(Self::Publish),
            2 => Some(Self::Receive),
            3 => Some(Self::Delete),
            4 => Some(Self::Replicate),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
/// Represents status.
pub enum Status {
    /// Represents the success case.
    Success = 0,
    /// Represents the malformed request case.
    MalformedRequest = 1,
    /// Represents the unsupported version case.
    UnsupportedVersion = 2,
    /// Represents the unknown operation case.
    UnknownOperation = 3,
    /// Represents the payload too large case.
    PayloadTooLarge = 4,
    /// Represents the publish not found case.
    PublishNotFound = 5,
    /// Represents the publish expired case.
    PublishExpired = 6,
    /// Represents the publish exhausted case.
    PublishExhausted = 7,
    /// Represents the delete token invalid case.
    DeleteTokenInvalid = 8,
    /// Represents the rate limited case.
    RateLimited = 9,
    /// Represents the store unavailable case.
    StoreUnavailable = 10,
    /// Represents the internal error case.
    InternalError = 11,
    /// Represents the replication unauthorized case.
    ReplicationUnauthorized = 12,
    /// Represents the email unverified case.
    EmailUnverified = 13,
}

impl Status {
    /// Returns the from u16.
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::Success),
            1 => Some(Self::MalformedRequest),
            2 => Some(Self::UnsupportedVersion),
            3 => Some(Self::UnknownOperation),
            4 => Some(Self::PayloadTooLarge),
            5 => Some(Self::PublishNotFound),
            6 => Some(Self::PublishExpired),
            7 => Some(Self::PublishExhausted),
            8 => Some(Self::DeleteTokenInvalid),
            9 => Some(Self::RateLimited),
            10 => Some(Self::StoreUnavailable),
            11 => Some(Self::InternalError),
            12 => Some(Self::ReplicationUnauthorized),
            13 => Some(Self::EmailUnverified),
            _ => None,
        }
    }
}

#[derive(Debug)]
/// Represents request envelope.
pub struct RequestEnvelope {
    /// Represents the operation carried by this record case.
    pub operation: Operation,
    /// Represents the flags carried by this record case.
    pub flags: u16,
    /// Represents the payload carried by this record case.
    pub payload: Vec<u8>,
}

#[derive(Debug)]
/// Represents response envelope.
pub struct ResponseEnvelope {
    /// Represents the operation carried by this record case.
    pub operation: Operation,
    /// Represents the status carried by this record case.
    pub status: Status,
    /// Represents the payload carried by this record case.
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents publish response.
pub struct PublishResponse {
    /// Represents the publish code carried by this record case.
    pub publish_code: String,
    /// Represents the delete token carried by this record case.
    pub delete_token: Vec<u8>,
    /// Represents the expires at unix ms carried by this record case.
    pub expires_at_unix_ms: u64,
    /// Represents the max receives carried by this record case.
    pub max_receives: u16,
    /// Represents the verification url carried by this record case.
    pub verification_url: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents receive response.
pub struct ReceiveResponse {
    /// Represents the publish payload carried by this record case.
    pub publish_payload: Vec<u8>,
    /// Represents the expires at unix ms carried by this record case.
    pub expires_at_unix_ms: u64,
    /// Represents the remaining receives carried by this record case.
    pub remaining_receives: u16,
    /// Represents the email verification carried by this record case.
    pub email_verification: Option<EmailVerification>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents email verification.
pub struct EmailVerification {
    /// Represents the email carried by this record case.
    pub email: String,
    /// Represents the verified carried by this record case.
    pub verified: bool,
    /// Represents the verified at unix ms carried by this record case.
    pub verified_at_unix_ms: u64,
    /// Represents the attestation carried by this record case.
    pub attestation: Vec<u8>,
}

#[derive(Debug)]
/// Represents protocol error.
pub enum ProtocolError {
    /// Represents the too short case.
    TooShort,
    /// Represents the bad magic case.
    BadMagic,
    /// Represents the unsupported version case.
    UnsupportedVersion,
    /// Represents the unknown operation case.
    UnknownOperation,
    /// Represents the unknown status case.
    UnknownStatus,
    /// Represents the length mismatch case.
    LengthMismatch,
    /// Represents the payload too large case.
    PayloadTooLarge,
    /// Represents the unsupported message version case.
    UnsupportedMessageVersion,
    /// The utf8.
    Utf8,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ProtocolError {}

/// Decodes request.
pub fn decode_request(bytes: &[u8], max_payload: usize) -> Result<RequestEnvelope, ProtocolError> {
    if bytes.len() < ENVELOPE_LEN {
        return Err(ProtocolError::TooShort);
    }
    if &bytes[0..4] != MAGIC {
        return Err(ProtocolError::BadMagic);
    }
    let version = read_u16_at(bytes, 4);
    if version != VERSION {
        return Err(ProtocolError::UnsupportedVersion);
    }
    let operation =
        Operation::from_u16(read_u16_at(bytes, 6)).ok_or(ProtocolError::UnknownOperation)?;
    let flags = read_u16_at(bytes, 8);
    let payload_len = read_u32_at(bytes, 10) as usize;
    if payload_len > max_payload {
        return Err(ProtocolError::PayloadTooLarge);
    }
    if bytes.len() != ENVELOPE_LEN + payload_len {
        return Err(ProtocolError::LengthMismatch);
    }
    Ok(RequestEnvelope {
        operation,
        flags,
        payload: bytes[ENVELOPE_LEN..].to_vec(),
    })
}

/// Decodes response.
pub fn decode_response(
    bytes: &[u8],
    max_payload: usize,
) -> Result<ResponseEnvelope, ProtocolError> {
    Ok(decode_response_with_tail(bytes, max_payload)?.envelope)
}

/// Represents response with tail.
pub struct ResponseWithTail {
    /// Represents the envelope carried by this record case.
    pub envelope: ResponseEnvelope,
    /// Represents the tail carried by this record case.
    pub tail: Vec<u8>,
}

/// Decodes response with tail.
pub fn decode_response_with_tail(
    bytes: &[u8],
    max_payload: usize,
) -> Result<ResponseWithTail, ProtocolError> {
    if bytes.len() < ENVELOPE_LEN {
        return Err(ProtocolError::TooShort);
    }
    if &bytes[0..4] != MAGIC {
        return Err(ProtocolError::BadMagic);
    }
    let version = read_u16_at(bytes, 4);
    if version != VERSION {
        return Err(ProtocolError::UnsupportedVersion);
    }
    let status = Status::from_u16(read_u16_at(bytes, 6)).ok_or(ProtocolError::UnknownStatus)?;
    let operation =
        Operation::from_u16(read_u16_at(bytes, 8)).ok_or(ProtocolError::UnknownOperation)?;
    let payload_len = read_u32_at(bytes, 10) as usize;
    if payload_len > max_payload {
        return Err(ProtocolError::PayloadTooLarge);
    }
    if bytes.len() < ENVELOPE_LEN + payload_len {
        return Err(ProtocolError::LengthMismatch);
    }
    Ok(ResponseWithTail {
        envelope: ResponseEnvelope {
            operation,
            status,
            payload: bytes[ENVELOPE_LEN..ENVELOPE_LEN + payload_len].to_vec(),
        },
        tail: bytes[ENVELOPE_LEN + payload_len..].to_vec(),
    })
}

/// Decodes error payload.
pub fn decode_error_payload(payload: &[u8]) -> Result<(Status, String), ProtocolError> {
    let mut reader = Reader::new(payload);
    reader.message_version()?;
    let status = Status::from_u16(reader.u16()?).ok_or(ProtocolError::UnknownStatus)?;
    let message = reader.string()?;
    Ok((status, message))
}

/// Decodes publish response.
pub fn decode_publish_response(
    payload: &[u8],
) -> Result<(String, Vec<u8>, u64, u16), ProtocolError> {
    let decoded = decode_publish_response_document(payload)?;
    Ok((
        decoded.publish_code,
        decoded.delete_token,
        decoded.expires_at_unix_ms,
        decoded.max_receives,
    ))
}

/// Decodes publish response document.
pub fn decode_publish_response_document(payload: &[u8]) -> Result<PublishResponse, ProtocolError> {
    let mut reader = Reader::new(payload);
    reader.message_version()?;
    let publish_code = reader.string()?;
    let delete_token = reader.bytes()?;
    let expires_at_unix_ms = reader.u64()?;
    let max_receives = reader.u16()?;
    let verification_url = if reader.is_done() {
        None
    } else {
        Some(reader.string()?)
    };
    Ok(PublishResponse {
        publish_code,
        delete_token,
        expires_at_unix_ms,
        max_receives,
        verification_url,
    })
}

/// Decodes receive response.
pub fn decode_receive_response(payload: &[u8]) -> Result<(Vec<u8>, u64, u16), ProtocolError> {
    let decoded = decode_receive_response_document(payload)?;
    Ok((
        decoded.publish_payload,
        decoded.expires_at_unix_ms,
        decoded.remaining_receives,
    ))
}

/// Decodes receive response document.
pub fn decode_receive_response_document(payload: &[u8]) -> Result<ReceiveResponse, ProtocolError> {
    let mut reader = Reader::new(payload);
    reader.message_version()?;
    let publish_payload = reader.bytes()?;
    let expires_at_unix_ms = reader.u64()?;
    let remaining_receives = reader.u16()?;
    let email_verification = if reader.is_done() {
        None
    } else {
        Some(EmailVerification {
            email: reader.string()?,
            verified: reader.u8()? != 0,
            verified_at_unix_ms: reader.u64()?,
            attestation: reader.bytes()?,
        })
    };
    Ok(ReceiveResponse {
        publish_payload,
        expires_at_unix_ms,
        remaining_receives,
        email_verification,
    })
}

/// Decodes delete response.
pub fn decode_delete_response(payload: &[u8]) -> Result<bool, ProtocolError> {
    let mut reader = Reader::new(payload);
    reader.message_version()?;
    Ok(reader.u8()? != 0)
}

/// Encodes response.
pub fn encode_response(operation: Operation, status: Status, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(ENVELOPE_LEN + payload.len());
    out.extend_from_slice(MAGIC);
    put_u16(&mut out, VERSION);
    put_u16(&mut out, status as u16);
    put_u16(&mut out, operation as u16);
    put_u32(&mut out, payload.len() as u32);
    out.extend_from_slice(payload);
    out
}

/// Encodes response with tail.
pub fn encode_response_with_tail(
    operation: Operation,
    status: Status,
    payload: &[u8],
    tail: &[u8],
) -> Vec<u8> {
    let mut out = encode_response(operation, status, payload);
    out.extend_from_slice(tail);
    out
}

/// Encodes error.
pub fn encode_error(operation: Operation, status: Status, message: &str) -> Vec<u8> {
    let mut payload = Vec::new();
    put_u16(&mut payload, MESSAGE_VERSION);
    put_u16(&mut payload, status as u16);
    put_string(&mut payload, message);
    encode_response(operation, status, &payload)
}

/// Encodes publish request.
pub fn encode_publish_request(ttl_seconds: u32, max_receives: u16, payload: &[u8]) -> Vec<u8> {
    encode_publish_request_with_email(ttl_seconds, max_receives, payload, None)
}

/// Encodes publish request with email.
pub fn encode_publish_request_with_email(
    ttl_seconds: u32,
    max_receives: u16,
    payload: &[u8],
    verification_email: Option<&str>,
) -> Vec<u8> {
    let mut body = Vec::with_capacity(8 + 4 + payload.len());
    put_u16(&mut body, MESSAGE_VERSION);
    put_u32(&mut body, ttl_seconds);
    put_u16(&mut body, max_receives);
    put_bytes(&mut body, payload);
    if let Some(email) = verification_email {
        put_string(&mut body, email);
    }
    encode_request(Operation::Publish, &body)
}

/// Encodes receive request.
pub fn encode_receive_request(publish_code: &str) -> Vec<u8> {
    let mut body = Vec::new();
    put_u16(&mut body, MESSAGE_VERSION);
    put_string(&mut body, publish_code);
    encode_request(Operation::Receive, &body)
}

/// Encodes delete request.
pub fn encode_delete_request(publish_code: &str, delete_token: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    put_u16(&mut body, MESSAGE_VERSION);
    put_string(&mut body, publish_code);
    put_bytes(&mut body, delete_token);
    encode_request(Operation::Delete, &body)
}

/// Encodes request.
pub fn encode_request(operation: Operation, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(ENVELOPE_LEN + payload.len());
    out.extend_from_slice(MAGIC);
    put_u16(&mut out, VERSION);
    put_u16(&mut out, operation as u16);
    put_u16(&mut out, 0);
    put_u32(&mut out, payload.len() as u32);
    out.extend_from_slice(payload);
    out
}

/// Represents reader.
pub struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    /// Creates a value from the supplied data.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    /// Returns the u16.
    pub fn u16(&mut self) -> Result<u16, ProtocolError> {
        if self.offset + 2 > self.bytes.len() {
            return Err(ProtocolError::TooShort);
        }
        let value = read_u16_at(self.bytes, self.offset);
        self.offset += 2;
        Ok(value)
    }

    /// Returns the u8.
    pub fn u8(&mut self) -> Result<u8, ProtocolError> {
        if self.offset + 1 > self.bytes.len() {
            return Err(ProtocolError::TooShort);
        }
        let value = self.bytes[self.offset];
        self.offset += 1;
        Ok(value)
    }

    /// Returns the message version.
    pub fn message_version(&mut self) -> Result<(), ProtocolError> {
        let version = self.u16()?;
        if version != MESSAGE_VERSION {
            return Err(ProtocolError::UnsupportedMessageVersion);
        }
        Ok(())
    }

    /// Returns the u32.
    pub fn u32(&mut self) -> Result<u32, ProtocolError> {
        if self.offset + 4 > self.bytes.len() {
            return Err(ProtocolError::TooShort);
        }
        let value = read_u32_at(self.bytes, self.offset);
        self.offset += 4;
        Ok(value)
    }

    /// Returns the u64.
    pub fn u64(&mut self) -> Result<u64, ProtocolError> {
        if self.offset + 8 > self.bytes.len() {
            return Err(ProtocolError::TooShort);
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

    /// Returns the string.
    pub fn string(&mut self) -> Result<String, ProtocolError> {
        let bytes = self.bytes()?;
        String::from_utf8(bytes).map_err(|_| ProtocolError::Utf8)
    }

    /// Returns the bytes.
    pub fn bytes(&mut self) -> Result<Vec<u8>, ProtocolError> {
        let len = self.u32()? as usize;
        self.fixed_bytes(len).map(|bytes| bytes.to_vec())
    }

    /// Returns the fixed bytes.
    pub fn fixed_bytes(&mut self, len: usize) -> Result<&'a [u8], ProtocolError> {
        if self.offset + len > self.bytes.len() {
            return Err(ProtocolError::TooShort);
        }
        let out = &self.bytes[self.offset..self.offset + len];
        self.offset += len;
        Ok(out)
    }

    /// Reports whether done.
    pub fn is_done(&self) -> bool {
        self.offset == self.bytes.len()
    }

    /// Returns the remaining len.
    pub fn remaining_len(&self) -> usize {
        self.bytes.len().saturating_sub(self.offset)
    }
}

/// Stores u16.
pub fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_be_bytes());
}

/// Stores u32.
pub fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_be_bytes());
}

/// Stores u64.
pub fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}

/// Stores string.
pub fn put_string(out: &mut Vec<u8>, value: &str) {
    put_bytes(out, value.as_bytes());
}

/// Stores bytes.
pub fn put_bytes(out: &mut Vec<u8>, value: &[u8]) {
    put_u32(out, value.len() as u32);
    out.extend_from_slice(value);
}

fn read_u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}
