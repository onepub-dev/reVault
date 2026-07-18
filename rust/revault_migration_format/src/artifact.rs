use crate::{ArtifactKind, MigrationHeader};
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use revault_page_api::{SecureString, SecureVec};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt;
use std::io::{Read, Write};
use zeroize::{Zeroize, Zeroizing};

const MAGIC: &[u8; 8] = b"LBXMIG01";
const ENVELOPE_VERSION: u16 = 1;
const HEADER_BYTES: usize = 64;
const FRAME_JSON: u8 = 1;
const FRAME_RAW: u8 = 2;
const FRAME_END: u8 = 255;
pub const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;
pub const JSON_FRAME_TYPE: u8 = FRAME_JSON;
pub const RAW_FRAME_TYPE: u8 = FRAME_RAW;

#[derive(Debug)]
pub enum MigrationError {
    Io(String),
    InvalidHeader(String),
    UnsupportedEnvelope(u16),
    InvalidKey,
    CorruptFrame(String),
    SecurityLimit(String),
    Serialization(String),
    Incomplete,
}

impl fmt::Display for MigrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(value) => write!(f, "migration I/O error: {value}"),
            Self::InvalidHeader(value) => write!(f, "invalid migration header: {value}"),
            Self::UnsupportedEnvelope(value) => {
                write!(f, "unsupported migration envelope version {value}")
            }
            Self::InvalidKey => write!(f, "migration artifact passphrase is incorrect"),
            Self::CorruptFrame(value) => write!(f, "corrupt migration frame: {value}"),
            Self::SecurityLimit(value) => write!(f, "migration security limit: {value}"),
            Self::Serialization(value) => write!(f, "migration record is invalid: {value}"),
            Self::Incomplete => write!(f, "migration artifact is incomplete"),
        }
    }
}

impl std::error::Error for MigrationError {}

pub type Result<T> = std::result::Result<T, MigrationError>;

/// A migration passphrase whose plaintext is exposed only while deriving an
/// artifact key. Secure values release their read guard before artifact
/// processing allocates or mutates any other secure memory.
pub trait MigrationPassphrase {
    fn with_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Result<R>;
}

impl MigrationPassphrase for SecureString {
    fn with_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Result<R> {
        SecureString::with_bytes(self, f).map_err(|err| MigrationError::Io(err.to_string()))
    }
}

impl MigrationPassphrase for SecureVec {
    fn with_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Result<R> {
        SecureVec::with_bytes(self, f).map_err(|err| MigrationError::Io(err.to_string()))
    }
}

impl MigrationPassphrase for [u8] {
    fn with_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Result<R> {
        Ok(f(self))
    }
}

impl<const N: usize> MigrationPassphrase for [u8; N] {
    fn with_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Result<R> {
        Ok(f(self))
    }
}

pub struct ArtifactWriter<W: Write> {
    output: Option<W>,
    header: MigrationHeader,
    encoded_header: [u8; HEADER_BYTES],
    key: [u8; 32],
    nonce_prefix: [u8; 4],
    sequence: u64,
    record_count: u64,
    checksum: Sha256,
    finished: bool,
}

impl<W: Write> ArtifactWriter<W> {
    pub fn new_with_passphrase<P: MigrationPassphrase + ?Sized>(
        output: W,
        header: MigrationHeader,
        passphrase: &P,
    ) -> Result<Self> {
        passphrase.with_bytes(|bytes| Self::new(output, header, bytes))?
    }

    pub fn new(mut output: W, header: MigrationHeader, passphrase: &[u8]) -> Result<Self> {
        if passphrase.is_empty() {
            return Err(MigrationError::InvalidHeader(
                "an artifact passphrase is required".to_string(),
            ));
        }
        let mut salt = [0u8; 16];
        let mut nonce_prefix = [0u8; 4];
        getrandom::fill(&mut salt).map_err(|err| MigrationError::Io(err.to_string()))?;
        getrandom::fill(&mut nonce_prefix).map_err(|err| MigrationError::Io(err.to_string()))?;
        let mut encoded_header = [0u8; HEADER_BYTES];
        encoded_header[0..8].copy_from_slice(MAGIC);
        encoded_header[8..10].copy_from_slice(&ENVELOPE_VERSION.to_le_bytes());
        encoded_header[10] = header.artifact_kind.code();
        encoded_header[12..16].copy_from_slice(&header.source_native_version.to_le_bytes());
        encoded_header[16..20].copy_from_slice(&header.migration_schema_version.to_le_bytes());
        encoded_header[20..24].copy_from_slice(
            &header
                .target_native_version
                .unwrap_or(u32::MAX)
                .to_le_bytes(),
        );
        encoded_header[24..40].copy_from_slice(&header.operation_id);
        encoded_header[40..56].copy_from_slice(&salt);
        encoded_header[56..60].copy_from_slice(&nonce_prefix);
        let public_checksum = Sha256::digest(&encoded_header[..60]);
        encoded_header[60..64].copy_from_slice(&public_checksum[..4]);
        output
            .write_all(&encoded_header)
            .map_err(|err| MigrationError::Io(err.to_string()))?;
        let key = derive_key(passphrase, &salt)?;
        Ok(Self {
            output: Some(output),
            header,
            encoded_header,
            key,
            nonce_prefix,
            sequence: 0,
            record_count: 0,
            checksum: Sha256::new(),
            finished: false,
        })
    }

    pub fn header(&self) -> &MigrationHeader {
        &self.header
    }

    pub fn write_json<T: Serialize>(&mut self, value: &T) -> Result<()> {
        let mut payload = serde_json::to_vec(value)
            .map_err(|err| MigrationError::Serialization(err.to_string()))?;
        let result = self.write_frame(FRAME_JSON, &payload);
        payload.zeroize();
        result
    }

    pub fn write_raw(&mut self, payload: &[u8]) -> Result<()> {
        self.write_frame(FRAME_RAW, payload)
    }

    /// Writes a secret raw frame without copying its plaintext into an
    /// ordinary caller-owned byte buffer. The frame assembly remains in the
    /// secure heap until it is handed to the AEAD implementation.
    pub fn write_secure_raw(&mut self, payload: &SecureVec) -> Result<()> {
        let frame_len = payload
            .len()
            .checked_add(1)
            .ok_or_else(|| MigrationError::SecurityLimit("frame is too large".to_string()))?;
        if frame_len > MAX_FRAME_BYTES {
            return Err(MigrationError::SecurityLimit(format!(
                "frame is {frame_len} bytes; maximum is {MAX_FRAME_BYTES}"
            )));
        }

        let mut plaintext = SecureVec::try_from_slice(&[FRAME_RAW])
            .map_err(|err| MigrationError::Io(err.to_string()))?;
        plaintext
            .try_extend_from_secure(payload)
            .map_err(|err| MigrationError::Io(err.to_string()))?;
        let result = plaintext.with_bytes(|bytes| {
            self.checksum.update(bytes);
            self.encrypt_and_write(bytes)
        });
        result.map_err(|err| MigrationError::Io(err.to_string()))??;
        self.record_count = self.record_count.saturating_add(1);
        Ok(())
    }

    pub fn records_written(&self) -> u64 {
        self.record_count
    }

    fn write_frame(&mut self, frame_type: u8, payload: &[u8]) -> Result<()> {
        if self.finished {
            return Err(MigrationError::CorruptFrame(
                "cannot append after completion".to_string(),
            ));
        }
        if payload.len() + 1 > MAX_FRAME_BYTES {
            return Err(MigrationError::SecurityLimit(format!(
                "frame is {} bytes; maximum is {MAX_FRAME_BYTES}",
                payload.len() + 1
            )));
        }
        let mut plaintext = Vec::with_capacity(payload.len() + 1);
        plaintext.push(frame_type);
        plaintext.extend_from_slice(payload);
        self.checksum.update(&plaintext);
        let result = self.encrypt_and_write(&plaintext);
        plaintext.zeroize();
        result?;
        self.record_count = self.record_count.saturating_add(1);
        Ok(())
    }

    fn encrypt_and_write(&mut self, plaintext: &[u8]) -> Result<()> {
        let cipher = ChaCha20Poly1305::new(&Key::from(self.key));
        let nonce = nonce(self.nonce_prefix, self.sequence);
        let aad = frame_aad(&self.encoded_header, self.sequence);
        let ciphertext = cipher
            .encrypt(
                &Nonce::from(nonce),
                Payload {
                    msg: plaintext,
                    aad: &aad,
                },
            )
            .map_err(|_| MigrationError::CorruptFrame("encryption failed".to_string()))?;
        let len = u32::try_from(ciphertext.len())
            .map_err(|_| MigrationError::SecurityLimit("frame is too large".to_string()))?;
        let output = self.output.as_mut().expect("writer output");
        output
            .write_all(&len.to_le_bytes())
            .and_then(|_| output.write_all(&self.sequence.to_le_bytes()))
            .and_then(|_| output.write_all(&ciphertext))
            .map_err(|err| MigrationError::Io(err.to_string()))?;
        self.sequence = self.sequence.saturating_add(1);
        Ok(())
    }

    pub fn finish(mut self) -> Result<W> {
        let digest: [u8; 32] = self.checksum.clone().finalize().into();
        let mut trailer = Vec::with_capacity(1 + 8 + 32);
        trailer.push(FRAME_END);
        trailer.extend_from_slice(&self.record_count.to_le_bytes());
        trailer.extend_from_slice(&digest);
        self.encrypt_and_write(&trailer)?;
        self.output
            .as_mut()
            .expect("writer output")
            .flush()
            .map_err(|err| MigrationError::Io(err.to_string()))?;
        self.finished = true;
        self.key.zeroize();
        Ok(self.output.take().expect("writer output"))
    }
}

impl<W: Write> Drop for ArtifactWriter<W> {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

pub struct ArtifactReader<R: Read> {
    input: R,
    header: MigrationHeader,
    encoded_header: [u8; HEADER_BYTES],
    key: [u8; 32],
    nonce_prefix: [u8; 4],
    sequence: u64,
    record_count: u64,
    checksum: Sha256,
    complete: bool,
}

impl<R: Read> ArtifactReader<R> {
    pub fn new_with_passphrase<P: MigrationPassphrase + ?Sized>(
        input: R,
        passphrase: &P,
    ) -> Result<Self> {
        passphrase.with_bytes(|bytes| Self::new(input, bytes))?
    }

    pub fn new(mut input: R, passphrase: &[u8]) -> Result<Self> {
        let mut encoded_header = [0u8; HEADER_BYTES];
        input
            .read_exact(&mut encoded_header)
            .map_err(|err| MigrationError::Io(err.to_string()))?;
        if &encoded_header[0..8] != MAGIC {
            return Err(MigrationError::InvalidHeader("wrong magic".to_string()));
        }
        let public_checksum = Sha256::digest(&encoded_header[..60]);
        if encoded_header[60..64] != public_checksum[..4] {
            return Err(MigrationError::InvalidHeader(
                "public checksum mismatch".to_string(),
            ));
        }
        let envelope = u16::from_le_bytes([encoded_header[8], encoded_header[9]]);
        if envelope != ENVELOPE_VERSION {
            return Err(MigrationError::UnsupportedEnvelope(envelope));
        }
        let artifact_kind = ArtifactKind::from_code(encoded_header[10])
            .ok_or_else(|| MigrationError::InvalidHeader("unknown artifact kind".to_string()))?;
        let source_native_version = read_u32(&encoded_header[12..16]);
        let migration_schema_version = read_u32(&encoded_header[16..20]);
        let target = read_u32(&encoded_header[20..24]);
        let mut operation_id = [0u8; 16];
        operation_id.copy_from_slice(&encoded_header[24..40]);
        let mut salt = [0u8; 16];
        salt.copy_from_slice(&encoded_header[40..56]);
        let mut nonce_prefix = [0u8; 4];
        nonce_prefix.copy_from_slice(&encoded_header[56..60]);
        let key = derive_key(passphrase, &salt)?;
        Ok(Self {
            input,
            header: MigrationHeader {
                artifact_kind,
                source_native_version,
                migration_schema_version,
                target_native_version: (target != u32::MAX).then_some(target),
                operation_id,
            },
            encoded_header,
            key,
            nonce_prefix,
            sequence: 0,
            record_count: 0,
            checksum: Sha256::new(),
            complete: false,
        })
    }

    pub fn header(&self) -> &MigrationHeader {
        &self.header
    }

    pub fn next_json<T: DeserializeOwned>(&mut self) -> Result<Option<T>> {
        let Some((frame_type, mut payload)) = self.next_frame()? else {
            return Ok(None);
        };
        if frame_type != FRAME_JSON {
            return Err(MigrationError::CorruptFrame(
                "expected a JSON record".to_string(),
            ));
        }
        let result = serde_json::from_slice(&payload)
            .map(Some)
            .map_err(|err| MigrationError::Serialization(err.to_string()));
        payload.zeroize();
        result
    }

    pub fn next_frame(&mut self) -> Result<Option<(u8, Vec<u8>)>> {
        if self.complete {
            return Ok(None);
        }
        let mut len_bytes = [0u8; 4];
        match self.input.read_exact(&mut len_bytes) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Err(MigrationError::Incomplete)
            }
            Err(err) => return Err(MigrationError::Io(err.to_string())),
        }
        let len = u32::from_le_bytes(len_bytes) as usize;
        if !(17..=MAX_FRAME_BYTES + 16).contains(&len) {
            return Err(MigrationError::SecurityLimit(format!(
                "encrypted frame length {len} is invalid"
            )));
        }
        let mut sequence_bytes = [0u8; 8];
        self.input
            .read_exact(&mut sequence_bytes)
            .map_err(|_| MigrationError::Incomplete)?;
        let sequence = u64::from_le_bytes(sequence_bytes);
        if sequence != self.sequence {
            return Err(MigrationError::CorruptFrame(format!(
                "expected sequence {}, got {sequence}",
                self.sequence
            )));
        }
        let mut ciphertext = vec![0u8; len];
        self.input
            .read_exact(&mut ciphertext)
            .map_err(|_| MigrationError::Incomplete)?;
        let cipher = ChaCha20Poly1305::new(&Key::from(self.key));
        let aad = frame_aad(&self.encoded_header, sequence);
        let plaintext = Zeroizing::new(
            cipher
                .decrypt(
                    &Nonce::from(nonce(self.nonce_prefix, sequence)),
                    Payload {
                        msg: &ciphertext,
                        aad: &aad,
                    },
                )
                .map_err(|_| {
                    if sequence == 0 {
                        MigrationError::InvalidKey
                    } else {
                        MigrationError::CorruptFrame("authentication failed".to_string())
                    }
                })?,
        );
        self.sequence = self.sequence.saturating_add(1);
        let Some((&frame_type, payload)) = plaintext.split_first() else {
            return Err(MigrationError::CorruptFrame("empty frame".to_string()));
        };
        if frame_type == FRAME_END {
            if payload.len() != 40 {
                return Err(MigrationError::CorruptFrame(
                    "invalid completion trailer".to_string(),
                ));
            }
            let expected_count = u64::from_le_bytes(payload[0..8].try_into().unwrap());
            let digest: [u8; 32] = self.checksum.clone().finalize().into();
            if expected_count != self.record_count || payload[8..40] != digest {
                return Err(MigrationError::CorruptFrame(
                    "completion summary mismatch".to_string(),
                ));
            }
            self.complete = true;
            return Ok(None);
        }
        if frame_type != FRAME_JSON && frame_type != FRAME_RAW {
            return Err(MigrationError::CorruptFrame(format!(
                "unknown frame type {frame_type}"
            )));
        }
        self.checksum.update(&plaintext);
        self.record_count = self.record_count.saturating_add(1);
        let output = payload.to_vec();
        Ok(Some((frame_type, output)))
    }

    /// Reads a secret raw frame directly into the secure heap. The AEAD
    /// implementation necessarily materializes its decrypted result before
    /// returning it; ownership is transferred immediately to `SecureVec`,
    /// which zeroizes that staging allocation as it is consumed.
    pub fn next_secure_frame(&mut self) -> Result<Option<(u8, SecureVec)>> {
        let Some((frame_type, payload)) = self.next_frame()? else {
            return Ok(None);
        };
        let payload =
            SecureVec::try_from_vec(payload).map_err(|err| MigrationError::Io(err.to_string()))?;
        Ok(Some((frame_type, payload)))
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }
}

impl<R: Read> Drop for ArtifactReader<R> {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

fn derive_key(passphrase: &[u8], salt: &[u8; 16]) -> Result<[u8; 32]> {
    let params = Params::new(64 * 1024, 3, 1, Some(32))
        .map_err(|err| MigrationError::SecurityLimit(err.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase, salt, &mut key)
        .map_err(|_| MigrationError::InvalidKey)?;
    Ok(key)
}

fn nonce(prefix: [u8; 4], sequence: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[..4].copy_from_slice(&prefix);
    nonce[4..].copy_from_slice(&sequence.to_le_bytes());
    nonce
}

fn frame_aad(header: &[u8; HEADER_BYTES], sequence: u64) -> Vec<u8> {
    let mut aad = Vec::with_capacity(HEADER_BYTES + 8);
    aad.extend_from_slice(header);
    aad.extend_from_slice(&sequence.to_le_bytes());
    aad
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().expect("fixed header slice"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn header(kind: ArtifactKind) -> MigrationHeader {
        MigrationHeader {
            artifact_kind: kind,
            source_native_version: 1,
            migration_schema_version: 1,
            target_native_version: Some(2),
            operation_id: [7; 16],
        }
    }

    #[test]
    fn encrypted_artifact_round_trips_and_requires_completion() {
        let mut writer =
            ArtifactWriter::new(Vec::new(), header(ArtifactKind::Vault), b"secret").unwrap();
        writer
            .write_json(&serde_json::json!({"profile":"default"}))
            .unwrap();
        writer.write_raw(b"chunk").unwrap();
        let bytes = writer.finish().unwrap();

        let mut reader = ArtifactReader::new(Cursor::new(bytes), b"secret").unwrap();
        let value: serde_json::Value = reader.next_json().unwrap().unwrap();
        assert_eq!(value["profile"], "default");
        assert_eq!(reader.next_frame().unwrap().unwrap().1, b"chunk");
        assert!(reader.next_frame().unwrap().is_none());
        assert!(reader.is_complete());
    }

    #[test]
    fn wrong_password_and_truncation_fail_closed() {
        let mut writer =
            ArtifactWriter::new(Vec::new(), header(ArtifactKind::Archive), b"secret").unwrap();
        writer.write_raw(b"chunk").unwrap();
        let mut bytes = writer.finish().unwrap();
        let mut wrong = ArtifactReader::new(Cursor::new(bytes.clone()), b"wrong").unwrap();
        assert!(matches!(
            wrong.next_frame(),
            Err(MigrationError::InvalidKey)
        ));
        bytes.truncate(bytes.len() - 3);
        let mut truncated = ArtifactReader::new(Cursor::new(bytes), b"secret").unwrap();
        assert!(truncated.next_frame().unwrap().is_some());
        assert!(matches!(
            truncated.next_frame(),
            Err(MigrationError::Incomplete)
        ));
    }

    #[test]
    fn oversized_frames_are_rejected_before_allocation() {
        let mut bytes = vec![0u8; HEADER_BYTES + 12];
        bytes[..8].copy_from_slice(MAGIC);
        bytes[8..10].copy_from_slice(&ENVELOPE_VERSION.to_le_bytes());
        bytes[10] = ArtifactKind::Vault.code();
        let checksum = Sha256::digest(&bytes[..60]);
        bytes[60..64].copy_from_slice(&checksum[..4]);
        bytes[64..68].copy_from_slice(&u32::MAX.to_le_bytes());
        let mut reader = ArtifactReader::new(Cursor::new(bytes), b"secret").unwrap();
        assert!(matches!(
            reader.next_frame(),
            Err(MigrationError::SecurityLimit(_))
        ));
    }
}
