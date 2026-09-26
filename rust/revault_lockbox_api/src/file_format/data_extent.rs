//! Candidate C's bounded data codec. Descriptors must come from the selected
//! authenticated index; a stored-byte digest alone does not prove membership.
use super::allocation_map::Extent;
use crate::compression::{encode_with_compression, COMPRESSION_NONE, COMPRESSION_ZSTD};
use crate::creation_options::FormatMode;
use crate::crypto::{open_with_nonce, seal_with_random_nonce, strong_checksum};
use crate::storage::Storage;
use crate::{Compression, Error, LockboxId, Result};
use sha2::Sha256;
use zeroize::Zeroizing;
use zstd_complete::decoding::StaticDecoderWorkspace;

pub(crate) const MAX_LOGICAL: usize = 256 * 1024;
const PAD_UNIT: usize = 64 * 1024;
const MAX_ALLOCATION: usize = MAX_LOGICAL + PAD_UNIT;
const MAGIC: &[u8; 8] = b"RV4DAT01";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Descriptor {
    pub object: [u8; 16],
    pub ordinal: u64,
    pub offset: u64,
    pub logical_len: u32,
    codec: u8,
    encoded_len: u32,
    allocation_len: u32,
}
impl Descriptor {
    pub(crate) fn encode(&self) -> [u8; 64] {
        let mut bytes = [0; 64];
        bytes[..8].copy_from_slice(MAGIC);
        bytes[8..24].copy_from_slice(&self.object);
        bytes[24..32].copy_from_slice(&self.ordinal.to_le_bytes());
        bytes[32..40].copy_from_slice(&self.offset.to_le_bytes());
        bytes[40..44].copy_from_slice(&self.logical_len.to_le_bytes());
        bytes[44..48].copy_from_slice(&self.encoded_len.to_le_bytes());
        bytes[48..52].copy_from_slice(&self.allocation_len.to_le_bytes());
        bytes[52] = self.codec;
        bytes
    }
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 64 || &bytes[..8] != MAGIC || bytes[53..].iter().any(|b| *b != 0) {
            return Err(Error::CorruptRecord);
        }
        let descriptor = Self {
            object: bytes[8..24].try_into().unwrap(),
            ordinal: u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
            offset: u64::from_le_bytes(bytes[32..40].try_into().unwrap()),
            logical_len: u32::from_le_bytes(bytes[40..44].try_into().unwrap()),
            encoded_len: u32::from_le_bytes(bytes[44..48].try_into().unwrap()),
            allocation_len: u32::from_le_bytes(bytes[48..52].try_into().unwrap()),
            codec: bytes[52],
        };
        descriptor.validate()?;
        Ok(descriptor)
    }
    fn validate(&self) -> Result<()> {
        if self.object == [0; 16]
            || self.logical_len == 0
            || self.logical_len as usize > MAX_LOGICAL
            || self.encoded_len == 0
            || self.encoded_len > self.logical_len
            || self.allocation_len < self.encoded_len
            || self.allocation_len as usize > MAX_ALLOCATION
            || (self.codec == COMPRESSION_NONE && self.encoded_len != self.logical_len)
            || !matches!(self.codec, COMPRESSION_NONE | COMPRESSION_ZSTD)
            || self.offset.checked_add(self.logical_len as u64).is_none()
        {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }
}

pub(crate) struct Codec {
    archive: LockboxId,
    mode: FormatMode,
    key: Option<Zeroizing<[u8; 32]>>,
    scratch: Zeroizing<Vec<u8>>,
}
impl Codec {
    pub(crate) fn new(archive: LockboxId, mode: FormatMode, key: Option<&[u8]>) -> Result<Self> {
        FormatMode::parse(mode.0)?;
        if mode.0 == 0 || (!mode.plaintext() && key.is_none_or(|k| k.len() != 32)) {
            return Err(Error::InvalidKey);
        }
        let key = if mode.plaintext() {
            None
        } else {
            let mut derived = Zeroizing::new([0; 32]);
            hkdf::Hkdf::<Sha256>::new(Some(archive.as_bytes()), key.unwrap())
                .expand(b"revault-candidate-data-key-v1\0", &mut *derived)
                .map_err(|_| Error::InvalidKey)?;
            Some(derived)
        };
        Ok(Self {
            archive,
            mode,
            key,
            scratch: Zeroizing::new(Vec::new()),
        })
    }
    /// Raw data stays inside a 64 KiB stored allocation, including nonce/tag.
    /// Logical boundaries stay 4 KiB-aligned, so aligned small ranges do not
    /// accidentally fetch two allocations because of encryption overhead.
    /// Compressed profiles compare independent 64/256 KiB logical access units.
    pub(crate) fn logical_unit(&self, compressed_unit: usize) -> Result<usize> {
        if ![PAD_UNIT, MAX_LOGICAL].contains(&compressed_unit) {
            return Err(Error::InvalidInput(
                "candidate extent unit must be 64 or 256 KiB".into(),
            ));
        }
        Ok(if self.mode.options().compression == Compression::None {
            (PAD_UNIT - self.overhead()) / 4096 * 4096
        } else {
            compressed_unit
        })
    }
    fn overhead(&self) -> usize {
        if self.mode.plaintext() {
            0
        } else {
            28
        }
    }
    fn allocation_len(&self, encoded_len: usize) -> usize {
        let size = encoded_len + self.overhead();
        if self.mode.unpadded() {
            size
        } else {
            size.div_ceil(PAD_UNIT) * PAD_UNIT
        }
    }
    fn aad(&self, descriptor: &Descriptor) -> Vec<u8> {
        let mut bytes = b"revault-candidate-data-v1\0".to_vec();
        bytes.extend_from_slice(self.archive.as_bytes());
        bytes.extend_from_slice(&self.mode.0.to_le_bytes());
        bytes.extend_from_slice(&descriptor.encode());
        bytes
    }
    pub(crate) fn encode(
        &self,
        object: [u8; 16],
        ordinal: u64,
        offset: u64,
        plain: &[u8],
    ) -> Result<(Descriptor, Zeroizing<Vec<u8>>)> {
        if plain.is_empty() || plain.len() > MAX_LOGICAL {
            return Err(Error::SecurityLimitExceeded(
                "candidate extent logical limit".into(),
            ));
        }
        let (codec, encoded) = encode_with_compression(plain, self.mode.options().compression);
        let mut encoded = Zeroizing::new(encoded);
        let descriptor = Descriptor {
            object,
            ordinal,
            offset,
            logical_len: plain.len() as u32,
            codec,
            encoded_len: encoded.len() as u32,
            allocation_len: self.allocation_len(encoded.len()) as u32,
        };
        descriptor.validate()?;
        encoded.resize(descriptor.allocation_len as usize - self.overhead(), 0);
        let stored = if let Some(key) = &self.key {
            let (nonce, ciphertext) =
                seal_with_random_nonce(&encoded, key.as_slice(), &self.aad(&descriptor))?;
            let mut stored = Zeroizing::new(Vec::with_capacity(descriptor.allocation_len as usize));
            stored.extend_from_slice(&nonce);
            stored.extend_from_slice(&ciphertext);
            stored
        } else {
            encoded
        };
        Ok((descriptor, stored))
    }
    pub(crate) fn validate_extent(
        &self,
        extent: Extent,
        sealed: u64,
        descriptor: &Descriptor,
    ) -> Result<()> {
        descriptor.validate()?;
        if descriptor.allocation_len as usize
            != self.allocation_len(descriptor.encoded_len as usize)
            || extent.len != descriptor.allocation_len as u64
            || extent.start < super::preparation_journal::DATA_START
            || extent
                .start
                .checked_add(extent.len)
                .is_none_or(|end| end > sealed)
            || (descriptor.codec == COMPRESSION_ZSTD
                && self.mode.options().compression == Compression::None)
        {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }
    /// Validates lengths before any read; verifies the owner's stored-byte
    /// commitment before decrypting or decompressing. Caller validates membership.
    pub(crate) fn load(
        &mut self,
        storage: &impl Storage,
        extent: Extent,
        sealed: u64,
        descriptor: &Descriptor,
    ) -> Result<Zeroizing<Vec<u8>>> {
        self.validate_extent(extent, sealed, descriptor)?;
        let stored = Zeroizing::new(storage.read_at(extent.start, extent.len as usize)?);
        if stored.len() != extent.len as usize || strong_checksum(&stored) != extent.digest {
            return Err(Error::CorruptRecord);
        }
        let decoded = if let Some(key) = &self.key {
            Zeroizing::new(open_with_nonce(
                &stored[12..],
                key.as_slice(),
                &stored[..12],
                &self.aad(descriptor),
            )?)
        } else {
            stored
        };
        let encoded_len = descriptor.encoded_len as usize;
        if decoded.len() < encoded_len || decoded[encoded_len..].iter().any(|b| *b != 0) {
            return Err(Error::CorruptRecord);
        }
        if descriptor.codec == COMPRESSION_NONE {
            let mut decoded = decoded;
            decoded.truncate(encoded_len);
            return Ok(decoded);
        }
        // Fixed workspace and fixed destination bound even malicious Zstd windows
        // and concatenated streams. There is no allocating fallback decoder.
        let required = StaticDecoderWorkspace::required_size(MAX_LOGICAL, 0)
            .map_err(|_| Error::CorruptRecord)?;
        if self.scratch.len() != required {
            self.scratch = Zeroizing::new(vec![0; required]);
        }
        let mut output = Zeroizing::new(vec![0; descriptor.logical_len as usize]);
        let mut decoder = StaticDecoderWorkspace::new(&mut self.scratch, MAX_LOGICAL, 0)
            .map_err(|_| Error::CorruptRecord)?;
        let len = decoder
            .decode_into(&decoded[..encoded_len], &mut output)
            .map_err(|_| Error::CorruptRecord)?;
        if len != output.len() {
            return Err(Error::CorruptRecord);
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests;
