//! Staged common native block descriptors, protected indexes and frame packets.
//!
//! The caller must obtain `commitment` from authenticated committed metadata,
//! not from the same untrusted frame. Index verification precedes block I/O.
//! A successful partial read authenticates every touched block, not untouched
//! blocks. Existing v4 whole-frame validation semantics remain unchanged.

use crate::crypto::strong_checksum;
use crate::page_buffer::ZeroizingBytes;
use crate::storage::Storage;
use crate::{Error, Result};
use sha2::Sha256;
use std::ops::Range;

#[path = "block_page.rs"]
pub(crate) mod block_page;

pub(crate) const BLOCK_BYTES: usize = 16 * 1024;
const MAX_FRAME_BYTES: usize = 4 * 1024 * 1024;
const DIGEST_BYTES: usize = 32;

/// Common persisted descriptor for the staged native block representation.
/// The TOC must authenticate these bytes before they can authorize block reads.
/// Signing remains the native commit's responsibility, never a separate,
/// weaker per-frame signature. One encoding covers every protection mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BlockFrameDescriptor {
    pub(crate) archive: crate::LockboxId,
    pub(crate) frame_id: u64,
    pub(crate) mode: crate::creation_options::FormatMode,
    pub(crate) compression: u8,
    pub(crate) logical_len: u64,
    pub(crate) stored_len: u64,
    pub(crate) salt: [u8; 32],
    pub(crate) index_commitment: [u8; 32],
}

impl BlockFrameDescriptor {
    pub(crate) const ENCODED_LEN: usize = 128;
    const CONTEXT_LEN: usize = 96;
    const MAGIC: &'static [u8; 8] = b"LBXBF001";

    fn fresh(
        archive: crate::LockboxId,
        frame_id: u64,
        mode: crate::creation_options::FormatMode,
        compression: u8,
        logical_len: u64,
        stored_len: u64,
    ) -> Result<Self> {
        let mut descriptor = Self {
            archive,
            frame_id,
            mode,
            compression,
            logical_len,
            stored_len,
            salt: [0; 32],
            index_commitment: [0; 32],
        };
        descriptor.validate()?;
        getrandom::fill(&mut descriptor.salt).map_err(|error| Error::Io(error.to_string()))?;
        Ok(descriptor)
    }

    fn validate(&self) -> Result<()> {
        use crate::compression::{COMPRESSION_NONE, COMPRESSION_ZSTD};
        // Block pages are an explicit-mode extension, not an implicit change
        // to the frozen legacy mode. Raw fallback remains legal with Zstd set.
        crate::creation_options::FormatMode::parse(self.mode.0)?;
        if self.mode.0 == 0
            || self.frame_id == 0
            || self.logical_len > MAX_FRAME_BYTES as u64
            || self.stored_len > self.logical_len
            || match self.compression {
                COMPRESSION_NONE => self.stored_len != self.logical_len,
                COMPRESSION_ZSTD => {
                    self.stored_len == 0
                        || self.stored_len >= self.logical_len
                        || self.mode.options().compression == crate::Compression::None
                }
                _ => true,
            }
        {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }

    pub(crate) fn encode(&self) -> Result<[u8; Self::ENCODED_LEN]> {
        self.validate()?;
        let mut out = [0; Self::ENCODED_LEN];
        out[..8].copy_from_slice(Self::MAGIC);
        out[8..10].copy_from_slice(&self.mode.0.to_le_bytes());
        out[10] = self.compression;
        out[16..32].copy_from_slice(self.archive.as_bytes());
        out[32..40].copy_from_slice(&self.frame_id.to_le_bytes());
        out[40..48].copy_from_slice(&self.logical_len.to_le_bytes());
        out[48..56].copy_from_slice(&self.stored_len.to_le_bytes());
        out[56..60].copy_from_slice(&(BLOCK_BYTES as u32).to_le_bytes());
        out[64..96].copy_from_slice(&self.salt);
        out[96..].copy_from_slice(&self.index_commitment);
        Ok(out)
    }

    pub(crate) fn decode(
        bytes: &[u8],
        archive: crate::LockboxId,
        mode: crate::creation_options::FormatMode,
    ) -> Result<Self> {
        if bytes.len() != Self::ENCODED_LEN
            || &bytes[..8] != Self::MAGIC
            || bytes[11..16]
                .iter()
                .chain(&bytes[60..64])
                .any(|byte| *byte != 0)
            || bytes[16..32] != *archive.as_bytes()
            || crate::checked::read_u16_le(&bytes[8..10])? != mode.0
            || crate::checked::read_u32_le(&bytes[56..60])? as usize != BLOCK_BYTES
        {
            return Err(Error::CorruptRecord);
        }
        let descriptor = Self {
            archive,
            mode,
            compression: bytes[10],
            frame_id: crate::checked::read_u64_le(&bytes[32..40])?,
            logical_len: crate::checked::read_u64_le(&bytes[40..48])?,
            stored_len: crate::checked::read_u64_le(&bytes[48..56])?,
            salt: bytes[64..96].try_into().map_err(|_| Error::CorruptRecord)?,
            index_commitment: bytes[96..128]
                .try_into()
                .map_err(|_| Error::CorruptRecord)?,
        };
        descriptor.validate()?;
        Ok(descriptor)
    }

    fn block_count(&self) -> Result<usize> {
        self.validate()?;
        Ok((self.stored_len as usize).div_ceil(BLOCK_BYTES))
    }

    fn tag_len(&self) -> usize {
        if self.mode.plaintext() {
            0
        } else {
            16
        }
    }

    fn index_len(&self) -> Result<usize> {
        Ok(Self::CONTEXT_LEN + self.block_count()? * DIGEST_BYTES + self.tag_len())
    }

    pub(crate) fn physical_len(&self) -> Result<usize> {
        Ok(self.index_len()? + self.stored_len as usize + self.block_count()? * self.tag_len())
    }

    /// Verify the stored table before parsing it or allocating from its fields.
    /// For encrypted frames this commits to ciphertext, never plaintext hashes.
    fn verify_stored_index(&self, index: &[u8]) -> Result<()> {
        if index.len() != self.index_len()? || strong_checksum(index) != self.index_commitment {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }

    fn index_cipher(&self, key: &[u8]) -> Result<chacha20poly1305::ChaCha20Poly1305> {
        use chacha20poly1305::KeyInit;
        if key.len() != 32 {
            return Err(Error::CorruptRecord);
        }
        let context = self.encode()?;
        let mut info = b"revault-native-block-key-v1\0".to_vec();
        info.extend_from_slice(&context[..Self::CONTEXT_LEN]);
        let mut derived = zeroize::Zeroizing::new([0u8; 32]);
        hkdf::Hkdf::<Sha256>::new(Some(&self.salt), key)
            .expand(&info, &mut *derived)
            .map_err(|_| Error::CorruptRecord)?;
        chacha20poly1305::ChaCha20Poly1305::new_from_slice(&*derived)
            .map_err(|_| Error::CorruptRecord)
    }

    /// Called after block ciphertext hashes are ready. A native writer must
    /// choose a fresh random salt for EVERY encoding attempt, including retries.
    fn seal_index(mut self, hashes: &[[u8; 32]], key: &[u8]) -> Result<(Self, Vec<u8>)> {
        use chacha20poly1305::aead::{Aead, Payload};
        if hashes.len() != self.block_count()? {
            return Err(Error::CorruptRecord);
        }
        let context = self.encode()?;
        let mut plain = zeroize::Zeroizing::new(context[..Self::CONTEXT_LEN].to_vec());
        for hash in hashes {
            plain.extend_from_slice(hash);
        }
        let stored = if self.mode.plaintext() {
            std::mem::take(&mut *plain)
        } else {
            self.index_cipher(key)?
                .encrypt(
                    &chacha20poly1305::Nonce::from(index_nonce()),
                    Payload {
                        msg: &plain,
                        aad: &context[..Self::CONTEXT_LEN],
                    },
                )
                .map_err(|_| Error::CorruptRecord)?
        };
        self.index_commitment = strong_checksum(&stored);
        Ok((self, stored))
    }

    /// Authenticate stored bytes first, decrypt when required, then validate
    /// the bound context before interpreting any block commitments.
    fn open_index(&self, stored: &[u8], key: &[u8]) -> Result<Vec<[u8; 32]>> {
        use chacha20poly1305::aead::{Aead, Payload};
        self.verify_stored_index(stored)?;
        let context = self.encode()?;
        let plain = zeroize::Zeroizing::new(if self.mode.plaintext() {
            stored.to_vec()
        } else {
            self.index_cipher(key)?
                .decrypt(
                    &chacha20poly1305::Nonce::from(index_nonce()),
                    Payload {
                        msg: stored,
                        aad: &context[..Self::CONTEXT_LEN],
                    },
                )
                .map_err(|_| Error::CorruptRecord)?
        });
        if plain.len() != Self::CONTEXT_LEN + self.block_count()? * DIGEST_BYTES
            || plain[..Self::CONTEXT_LEN] != context[..Self::CONTEXT_LEN]
        {
            return Err(Error::CorruptRecord);
        }
        plain[Self::CONTEXT_LEN..]
            .chunks_exact(DIGEST_BYTES)
            .map(|hash| hash.try_into().map_err(|_| Error::CorruptRecord))
            .collect()
    }

    /// Coordinates within the frame extent, not absolute archive offsets.
    /// Compressed content has no arbitrary restart points: read the full frame.
    fn stored_range(&self, logical: Range<u64>) -> Result<Range<usize>> {
        self.validate()?;
        if logical.start > logical.end || logical.end > self.logical_len {
            return Err(Error::CorruptRecord);
        }
        let start = self.index_len()?;
        if logical.is_empty() {
            return Ok(start..start);
        }
        if self.compression != crate::compression::COMPRESSION_NONE {
            return Ok(start..self.physical_len()?);
        }
        let first = logical.start as usize / BLOCK_BYTES;
        let last = (logical.end as usize).div_ceil(BLOCK_BYTES);
        Ok(start + first * (BLOCK_BYTES + self.tag_len())
            ..start + (last * BLOCK_BYTES).min(self.stored_len as usize) + last * self.tag_len())
    }
}

fn index_nonce() -> [u8; 12] {
    // Domain 1 is reserved for the index; data blocks must use domain 0 with
    // their ordinal. Fresh frame salt separates retries and replacement writes.
    let mut nonce = [0; 12];
    nonce[0] = 1;
    nonce
}

fn data_nonce(ordinal: usize) -> [u8; 12] {
    let mut nonce = [0; 12];
    nonce[4..].copy_from_slice(&(ordinal as u64).to_le_bytes());
    nonce
}

fn block_aad(context: &[u8], ordinal: usize) -> Vec<u8> {
    let mut aad = context.to_vec();
    aad.extend_from_slice(&(ordinal as u64).to_le_bytes());
    aad
}

/// Create one common native frame packet. The caller will persist the returned
/// descriptor in trusted TOC metadata, separately from these untrusted bytes.
/// Fresh construction on every call prevents nonce reuse across writer retries.
pub(crate) fn encode_block_frame(
    archive: crate::LockboxId,
    frame_id: u64,
    mode: crate::creation_options::FormatMode,
    input: &[u8],
    key: &[u8],
) -> Result<(BlockFrameDescriptor, Vec<u8>)> {
    use chacha20poly1305::aead::AeadInOut;
    if input.len() > MAX_FRAME_BYTES || mode.0 == 0 || (!mode.plaintext() && key.len() != 32) {
        return Err(Error::CorruptRecord);
    }
    crate::creation_options::FormatMode::parse(mode.0)?;
    let (compression, stored) =
        crate::compression::encode_with_compression(input, mode.options().compression);
    let stored = ZeroizingBytes::new(stored);
    let descriptor = BlockFrameDescriptor::fresh(
        archive,
        frame_id,
        mode,
        compression,
        input.len() as u64,
        stored.len() as u64,
    )?;
    let cipher = if mode.plaintext() {
        None
    } else {
        Some(descriptor.index_cipher(key)?)
    };
    let context = descriptor.encode()?;
    let index_len = descriptor.index_len()?;
    let mut packet = ZeroizingBytes::new(vec![0; descriptor.physical_len()?]);
    let mut hashes = Vec::with_capacity(descriptor.block_count()?);
    for (ordinal, block) in stored.chunks(BLOCK_BYTES).enumerate() {
        let start = index_len + ordinal * (BLOCK_BYTES + descriptor.tag_len());
        let end = start + block.len() + descriptor.tag_len();
        packet[start..start + block.len()].copy_from_slice(block);
        if let Some(cipher) = &cipher {
            let tag = cipher
                .encrypt_inout_detached(
                    &chacha20poly1305::Nonce::from(data_nonce(ordinal)),
                    &block_aad(&context[..BlockFrameDescriptor::CONTEXT_LEN], ordinal),
                    (&mut packet[start..start + block.len()]).into(),
                )
                .map_err(|_| Error::CorruptRecord)?;
            packet[start + block.len()..end].copy_from_slice(&tag);
        }
        hashes.push(strong_checksum(&packet[start..end]));
    }
    let (descriptor, index) = descriptor.seal_index(&hashes, key)?;
    let index = ZeroizingBytes::new(index);
    packet[..index_len].copy_from_slice(&index);
    Ok((descriptor, std::mem::take(&mut *packet)))
}

pub(crate) struct BlockFrameReader<'a> {
    descriptor: BlockFrameDescriptor,
    storage: &'a crate::storage::StorageBackend,
    offset: u64,
    hashes: Vec<[u8; 32]>,
    cipher: Option<chacha20poly1305::ChaCha20Poly1305>,
}

impl<'a> BlockFrameReader<'a> {
    /// A validated descriptor is not necessarily trusted. The native TOC must
    /// establish its commitment before calling this function. No signature is
    /// verified here; signed archive opening must retain full commit validation.
    pub(crate) fn open(
        descriptor: &BlockFrameDescriptor,
        storage: &'a crate::storage::StorageBackend,
        offset: u64,
        physical_len: usize,
        key: &[u8],
    ) -> Result<Self> {
        if descriptor.physical_len()? != physical_len {
            return Err(Error::CorruptRecord);
        }
        storage.ensure_current()?;
        let storage_len = storage.len()?;
        if offset
            .checked_add(physical_len as u64)
            .is_none_or(|end| end > storage_len)
        {
            return Err(Error::Truncated);
        }
        let mut stored = ZeroizingBytes::new(vec![0; descriptor.index_len()?]);
        storage.read_at_into(offset, &mut stored)?;
        let hashes = descriptor.open_index(&stored, key)?;
        let cipher = if descriptor.mode.plaintext() {
            None
        } else {
            Some(descriptor.index_cipher(key)?)
        };
        storage.ensure_current()?;
        Ok(Self {
            descriptor: descriptor.clone(),
            storage,
            offset,
            hashes,
            cipher,
        })
    }

    pub(crate) fn read(&self, range: Range<u64>) -> Result<Vec<u8>> {
        use chacha20poly1305::aead::AeadInOut;
        let descriptor = &self.descriptor;
        let extent = descriptor.stored_range(range.clone())?;
        self.storage.ensure_current()?;
        let storage_len = self.storage.len()?;
        if self
            .offset
            .checked_add(descriptor.physical_len()? as u64)
            .is_none_or(|end| end > storage_len)
        {
            return Err(Error::Truncated);
        }
        if range.is_empty() {
            return Ok(Vec::new());
        }
        let mut bytes = ZeroizingBytes::new(vec![0; extent.len()]);
        self.storage.read_at_into(
            self.offset
                .checked_add(extent.start as u64)
                .ok_or(Error::CorruptRecord)?,
            &mut bytes,
        )?;
        let raw = descriptor.compression == crate::compression::COMPRESSION_NONE;
        let first = if raw {
            range.start as usize / BLOCK_BYTES
        } else {
            0
        };
        let last = if raw {
            (range.end as usize).div_ceil(BLOCK_BYTES)
        } else {
            descriptor.block_count()?
        };
        let context = descriptor.encode()?;
        let stride = BLOCK_BYTES + descriptor.tag_len();
        let mut plain_len = 0;
        for ordinal in first..last {
            let len = (descriptor.stored_len as usize - ordinal * BLOCK_BYTES).min(BLOCK_BYTES);
            let start = (ordinal - first) * stride;
            let block = &mut bytes[start..start + len + descriptor.tag_len()];
            if strong_checksum(block) != self.hashes[ordinal] {
                return Err(Error::CorruptRecord);
            }
            if let Some(cipher) = &self.cipher {
                let (message, tag) = block.split_at_mut(len);
                let tag =
                    chacha20poly1305::Tag::try_from(&*tag).map_err(|_| Error::CorruptRecord)?;
                cipher
                    .decrypt_inout_detached(
                        &chacha20poly1305::Nonce::from(data_nonce(ordinal)),
                        &block_aad(&context[..BlockFrameDescriptor::CONTEXT_LEN], ordinal),
                        message.into(),
                        &tag,
                    )
                    .map_err(|_| Error::CorruptRecord)?;
            }
            bytes.copy_within(start..start + len, plain_len);
            plain_len += len;
        }
        // Wipe removed tag bytes before shortening the initialized vector.
        zeroize::Zeroize::zeroize(&mut bytes[plain_len..]);
        bytes.truncate(plain_len);
        let (mut logical, wanted) = if raw {
            let start = range.start as usize - first * BLOCK_BYTES;
            (bytes, start..start + (range.end - range.start) as usize)
        } else {
            (
                ZeroizingBytes::new(crate::compression::decode_compression_frame(
                    descriptor.compression,
                    &bytes,
                    descriptor.logical_len,
                )?),
                range.start as usize..range.end as usize,
            )
        };
        self.storage.ensure_current()?;
        logical.copy_within(wanted.clone(), 0);
        zeroize::Zeroize::zeroize(&mut logical[wanted.len()..]);
        logical.truncate(wanted.len());
        Ok(std::mem::take(&mut *logical))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageBackend;

    #[test]
    fn native_block_packets_roundtrip_all_policies_and_single_pass_codec_output() {
        use crate::creation_options::FormatMode;
        use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
        let archive = crate::LockboxId::from_bytes([71; 16]);
        let key = [53; 32];
        for encryption in [EncryptionMode::None, EncryptionMode::ChaCha20Poly1305] {
            for signing in [SigningMode::None, SigningMode::Owner] {
                for size_padding in [SizePadding::Default, SizePadding::None] {
                    for compression in [Compression::None, Compression::default()] {
                        let mode = FormatMode::new(LockboxFormatOptions {
                            encryption,
                            signing,
                            size_padding,
                            compression,
                        });
                        for len in [0usize, 1, 16383, 16384, 16385, 65539] {
                            let input: Vec<_> = (0..len).map(|n| (n % 251) as u8).collect();
                            let (descriptor, packet) =
                                encode_block_frame(archive, 31, mode, &input, &key).unwrap();
                            let (algorithm, encoded) =
                                crate::compression::encode_with_compression(&input, compression);
                            assert_eq!(descriptor.compression, algorithm);
                            assert_eq!(descriptor.stored_len, encoded.len() as u64);
                            // Independently unpack with the allocating AEAD API,
                            // not the reader's in-place path, and bind codec bytes.
                            use chacha20poly1305::aead::{Aead, Payload};
                            let context = descriptor.encode().unwrap();
                            let mut actual_stored = ZeroizingBytes::new(Vec::new());
                            for (ordinal, block) in packet[descriptor.index_len().unwrap()..]
                                .chunks(BLOCK_BYTES + descriptor.tag_len())
                                .enumerate()
                            {
                                let plain = ZeroizingBytes::new(if mode.plaintext() {
                                    block.to_vec()
                                } else {
                                    descriptor
                                        .index_cipher(&key)
                                        .unwrap()
                                        .decrypt(
                                            &chacha20poly1305::Nonce::from(data_nonce(ordinal)),
                                            Payload {
                                                msg: block,
                                                aad: &block_aad(
                                                    &context[..BlockFrameDescriptor::CONTEXT_LEN],
                                                    ordinal,
                                                ),
                                            },
                                        )
                                        .unwrap()
                                });
                                actual_stored.extend_from_slice(&plain);
                            }
                            assert_eq!(&*actual_stored, &encoded);
                            let mut archive_bytes = vec![79; 317];
                            archive_bytes.extend_from_slice(&packet);
                            archive_bytes.extend_from_slice(&[83; 211]);
                            let storage = StorageBackend::memory(archive_bytes);
                            let reader = BlockFrameReader::open(
                                &descriptor,
                                &storage,
                                317,
                                packet.len(),
                                &key,
                            )
                            .unwrap();
                            assert_eq!(reader.read(0..len as u64).unwrap(), input);
                            assert!(reader.read(0..len as u64 + 1).is_err());
                            assert_eq!(reader.read(len as u64..len as u64).unwrap(), b"");
                            for start in [0, len / 2, len.saturating_sub(13)] {
                                let end = (start + 29).min(len);
                                assert_eq!(
                                    reader.read(start as u64..end as u64).unwrap(),
                                    input[start..end]
                                );
                            }
                            assert!(BlockFrameReader::open(
                                &descriptor,
                                &storage,
                                u64::MAX,
                                packet.len(),
                                &key
                            )
                            .is_err());
                            assert!(BlockFrameReader::open(
                                &descriptor,
                                &storage,
                                317,
                                packet.len() + 1,
                                &key
                            )
                            .is_err());
                            assert_eq!(storage.read_at(0, 317).unwrap(), vec![79; 317]);
                            assert_eq!(
                                storage.read_at(317 + packet.len() as u64, 211).unwrap(),
                                vec![83; 211]
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_blocks_reject_damage_and_ordinal_substitution_after_index_authentication() {
        use crate::creation_options::FormatMode;
        use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
        let archive = crate::LockboxId::from_bytes([71; 16]);
        let key = [53; 32];
        for encryption in [EncryptionMode::None, EncryptionMode::ChaCha20Poly1305] {
            let mode = FormatMode::new(LockboxFormatOptions {
                encryption,
                signing: SigningMode::None,
                size_padding: SizePadding::Default,
                compression: Compression::None,
            });
            let input = vec![31; BLOCK_BYTES * 2 + 13];
            let (descriptor, packet) = encode_block_frame(archive, 31, mode, &input, &key).unwrap();
            let index_len = descriptor.index_len().unwrap();
            let mut damaged = packet.clone();
            damaged[index_len + 100] ^= 1;
            let storage = StorageBackend::memory(damaged);
            let reader =
                BlockFrameReader::open(&descriptor, &storage, 0, packet.len(), &key).unwrap();
            // A range verifies the complete touched block, not only returned bytes.
            assert!(reader.read(0..1).is_err());
            assert_eq!(
                reader
                    .read(BLOCK_BYTES as u64..BLOCK_BYTES as u64 + 1)
                    .unwrap(),
                [31]
            );
            assert!(reader.read(0..input.len() as u64).is_err());
            let mut index_damage = packet.clone();
            index_damage[0] ^= 1;
            assert!(BlockFrameReader::open(
                &descriptor,
                &StorageBackend::memory(index_damage),
                0,
                packet.len(),
                &key
            )
            .is_err());
            assert!(BlockFrameReader::open(
                &descriptor,
                &StorageBackend::memory(packet[..packet.len() - 1].to_vec()),
                0,
                packet.len(),
                &key
            )
            .is_err());
            if !mode.plaintext() {
                assert!(BlockFrameReader::open(
                    &descriptor,
                    &StorageBackend::memory(packet.clone()),
                    0,
                    packet.len(),
                    &[54; 32]
                )
                .is_err());
                let stride = BLOCK_BYTES + descriptor.tag_len();
                let mut swapped = packet.clone();
                swapped[index_len..index_len + stride]
                    .copy_from_slice(&packet[index_len + stride..index_len + 2 * stride]);
                let mut hashes = descriptor.open_index(&packet[..index_len], &key).unwrap();
                hashes[0] = strong_checksum(&swapped[index_len..index_len + stride]);
                // Deliberately regenerate even the authenticated index: the data
                // block's own ordinal/domain binding must still reject the swap.
                let (recommitted, index) = descriptor.clone().seal_index(&hashes, &key).unwrap();
                swapped[..index_len].copy_from_slice(&index);
                let storage = StorageBackend::memory(swapped);
                let reader =
                    BlockFrameReader::open(&recommitted, &storage, 0, packet.len(), &key).unwrap();
                assert!(reader.read(0..1).is_err());
            }
            assert_ne!(data_nonce(0), index_nonce());
            assert_ne!(data_nonce(0), data_nonce(1));
        }
    }

    #[test]
    fn native_block_file_extents_reject_later_damage_and_truncation_in_all_modes() {
        use crate::creation_options::FormatMode;
        use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
        let root = std::env::temp_dir().join(format!(
            "revault-native-block-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let archive = crate::LockboxId::from_bytes([71; 16]);
        let key = [53; 32];
        for encrypted in [false, true] {
            for signed in [false, true] {
                for compressed in [false, true] {
                    let mode = FormatMode::new(LockboxFormatOptions {
                        encryption: if encrypted {
                            EncryptionMode::ChaCha20Poly1305
                        } else {
                            EncryptionMode::None
                        },
                        signing: if signed {
                            SigningMode::Owner
                        } else {
                            SigningMode::None
                        },
                        compression: if compressed {
                            Compression::default()
                        } else {
                            Compression::None
                        },
                        size_padding: SizePadding::None,
                    });
                    let input: Vec<_> = (0..65539).map(|n| (n % 251) as u8).collect();
                    let (descriptor, packet) =
                        encode_block_frame(archive, 31, mode, &input, &key).unwrap();
                    let mut bytes = vec![71; 317];
                    bytes.extend_from_slice(&packet);
                    bytes.extend_from_slice(&[79; 211]);
                    let path = root.join(format!("{encrypted}-{signed}-{compressed}"));
                    let storage = StorageBackend::create_file(&path, &bytes).unwrap();
                    let reader =
                        BlockFrameReader::open(&descriptor, &storage, 317, packet.len(), &key)
                            .unwrap();
                    assert_eq!(reader.read(17000..17013).unwrap(), input[17000..17013]);
                    assert_eq!(reader.read(0..input.len() as u64).unwrap(), input);
                    // Deliberate damage through a shared storage clone; no public
                    // CLI operation can create this unit-level fault condition.
                    let mut damaged = storage.clone();
                    damaged
                        .write_at(
                            317 + packet.len() as u64 - 1,
                            &[packet[packet.len() - 1] ^ 1],
                        )
                        .unwrap();
                    assert!(reader.read(0..input.len() as u64).is_err());
                    damaged.truncate(317 + packet.len() as u64 - 1).unwrap();
                    assert!(reader.read(0..1).is_err());
                    assert!(reader.read(0..0).is_err());
                }
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn native_block_maximum_frames_preserve_incompressible_fallback() {
        use crate::creation_options::FormatMode;
        use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
        let mut state = 31917u32;
        let input: Vec<_> = (0..MAX_FRAME_BYTES)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                state as u8
            })
            .collect();
        for encryption in [EncryptionMode::None, EncryptionMode::ChaCha20Poly1305] {
            let mode = FormatMode::new(LockboxFormatOptions {
                encryption,
                signing: SigningMode::None,
                size_padding: SizePadding::Default,
                compression: Compression::default(),
            });
            let (descriptor, packet) = encode_block_frame(
                crate::LockboxId::from_bytes([71; 16]),
                31,
                mode,
                &input,
                &[53; 32],
            )
            .unwrap();
            assert_eq!(descriptor.compression, crate::compression::COMPRESSION_NONE);
            let storage = StorageBackend::memory(packet.clone());
            let reader =
                BlockFrameReader::open(&descriptor, &storage, 0, packet.len(), &[53; 32]).unwrap();
            assert_eq!(reader.read(0..input.len() as u64).unwrap(), input);
            let tail = input.len() - 13;
            assert_eq!(
                reader.read(tail as u64..input.len() as u64).unwrap(),
                input[tail..]
            );
            assert!(encode_block_frame(
                descriptor.archive,
                32,
                mode,
                &vec![0; MAX_FRAME_BYTES + 1],
                &[53; 32]
            )
            .is_err());
        }
    }

    #[test]
    fn native_block_descriptor_roundtrips_all_modes_and_bounds_physical_reads() {
        use crate::creation_options::FormatMode;
        use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
        for encryption in [EncryptionMode::None, EncryptionMode::ChaCha20Poly1305] {
            for signing in [SigningMode::None, SigningMode::Owner] {
                for size_padding in [SizePadding::Default, SizePadding::None] {
                    for compression in [Compression::None, Compression::default()] {
                        let mode = FormatMode::new(LockboxFormatOptions {
                            encryption,
                            signing,
                            size_padding,
                            compression,
                        });
                        for logical_len in [0, 1, 16383, 16384, 16385, MAX_FRAME_BYTES as u64] {
                            // A Zstd creation choice can still store incompressible
                            // content raw, with exactly the same descriptor schema.
                            let mut descriptor = BlockFrameDescriptor {
                                archive: crate::LockboxId::from_bytes([17; 16]),
                                frame_id: 23,
                                mode,
                                compression: crate::compression::COMPRESSION_NONE,
                                logical_len,
                                stored_len: logical_len,
                                salt: [37; 32],
                                index_commitment: [0; 32],
                            };
                            let hashes = vec![[31; 32]; descriptor.block_count().unwrap()];
                            let fresh = BlockFrameDescriptor::fresh(
                                descriptor.archive,
                                descriptor.frame_id,
                                mode,
                                0,
                                logical_len,
                                logical_len,
                            )
                            .unwrap();
                            let retry = BlockFrameDescriptor::fresh(
                                descriptor.archive,
                                descriptor.frame_id,
                                mode,
                                0,
                                logical_len,
                                logical_len,
                            )
                            .unwrap();
                            assert_ne!(fresh.salt, retry.salt);
                            let (_, fresh_index) = fresh.seal_index(&hashes, &[67; 32]).unwrap();
                            let (_, retry_index) = retry.seal_index(&hashes, &[67; 32]).unwrap();
                            assert_ne!(fresh_index, retry_index);
                            let (sealed, index) =
                                descriptor.clone().seal_index(&hashes, &[67; 32]).unwrap();
                            assert_eq!(sealed.open_index(&index, &[67; 32]).unwrap(), hashes);
                            assert_eq!(index.len(), descriptor.index_len().unwrap());
                            let mut swapped = sealed.clone();
                            swapped.frame_id += 1;
                            assert!(swapped.open_index(&index, &[67; 32]).is_err());
                            let mut swapped = sealed.clone();
                            swapped.salt[0] ^= 1;
                            assert!(swapped.open_index(&index, &[67; 32]).is_err());
                            for policy_bit in [2, 0x100] {
                                let mut swapped = sealed.clone();
                                swapped.mode.0 ^= policy_bit;
                                assert!(swapped.open_index(&index, &[67; 32]).is_err());
                            }
                            let mut swapped = sealed.clone();
                            swapped.archive = crate::LockboxId::from_bytes([18; 16]);
                            assert!(swapped.open_index(&index, &[67; 32]).is_err());
                            if !mode.plaintext() {
                                assert!(sealed.open_index(&index, &[68; 32]).is_err());
                                assert!(sealed.open_index(&index, &[67; 31]).is_err());
                                let mut altered = index.clone();
                                altered[0] ^= 1;
                                let mut rehashed = sealed.clone();
                                rehashed.index_commitment = strong_checksum(&altered);
                                assert!(rehashed.open_index(&altered, &[67; 32]).is_err());
                            }
                            assert!(descriptor
                                .clone()
                                .seal_index(&vec![[31; 32]; hashes.len() + 1], &[67; 32])
                                .is_err());
                            let table = vec![53; descriptor.index_len().unwrap()];
                            descriptor.index_commitment = strong_checksum(&table);
                            descriptor.verify_stored_index(&table).unwrap();
                            let bytes = descriptor.encode().unwrap();
                            assert_eq!(bytes.len(), 128);
                            assert_eq!(
                                BlockFrameDescriptor::decode(&bytes, descriptor.archive, mode)
                                    .unwrap(),
                                descriptor
                            );
                            assert!(BlockFrameDescriptor::decode(
                                &bytes[..127],
                                descriptor.archive,
                                mode
                            )
                            .is_err());
                            let mut extended = bytes.to_vec();
                            extended.push(0);
                            assert!(BlockFrameDescriptor::decode(
                                &extended,
                                descriptor.archive,
                                mode
                            )
                            .is_err());
                            assert!(BlockFrameDescriptor::decode(
                                &bytes,
                                crate::LockboxId::from_bytes([18; 16]),
                                mode
                            )
                            .is_err());
                            assert!(BlockFrameDescriptor::decode(
                                &bytes,
                                descriptor.archive,
                                FormatMode(mode.0 ^ 1)
                            )
                            .is_err());
                            assert!(descriptor
                                .verify_stored_index(&table[..table.len() - 1])
                                .is_err());
                            let mut damaged = table;
                            damaged[0] ^= 1;
                            assert!(descriptor.verify_stored_index(&damaged).is_err());
                            assert_eq!(
                                descriptor
                                    .stored_range(logical_len..logical_len)
                                    .unwrap()
                                    .len(),
                                0
                            );
                            assert!(descriptor.stored_range(0..logical_len + 1).is_err());
                            if logical_len > 0 {
                                let range = descriptor
                                    .stored_range(logical_len - 1..logical_len)
                                    .unwrap();
                                let tail = ((logical_len - 1) as usize % BLOCK_BYTES) + 1;
                                assert_eq!(range.len(), tail + descriptor.tag_len());
                                assert_eq!(range.end, descriptor.physical_len().unwrap());
                            }
                            if compression != Compression::None && logical_len > 1 {
                                descriptor.compression = crate::compression::COMPRESSION_ZSTD;
                                descriptor.stored_len = 1;
                                let (compressed, index) = descriptor
                                    .clone()
                                    .seal_index(&[[31; 32]], &[67; 32])
                                    .unwrap();
                                assert_eq!(
                                    compressed.open_index(&index, &[67; 32]).unwrap(),
                                    [[31; 32]]
                                );
                                let bytes = descriptor.encode().unwrap();
                                assert_eq!(
                                    BlockFrameDescriptor::decode(&bytes, descriptor.archive, mode)
                                        .unwrap(),
                                    descriptor
                                );
                                assert_eq!(
                                    descriptor.stored_range(0..1).unwrap(),
                                    descriptor.index_len().unwrap()
                                        ..descriptor.physical_len().unwrap()
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_block_descriptor_rejects_unknown_fields_and_oversized_inputs() {
        use crate::creation_options::FormatMode;
        use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
        let mode = FormatMode::new(LockboxFormatOptions {
            encryption: EncryptionMode::None,
            signing: SigningMode::None,
            size_padding: SizePadding::Default,
            compression: Compression::None,
        });
        let descriptor = BlockFrameDescriptor {
            archive: crate::LockboxId::from_bytes([17; 16]),
            frame_id: 23,
            mode,
            compression: 0,
            logical_len: 17000,
            stored_len: 17000,
            salt: [37; 32],
            index_commitment: [53; 32],
        };
        let bytes = descriptor.encode().unwrap();
        for index in [0, 8, 9, 11, 12, 13, 14, 15, 56, 57, 58, 59, 60, 61, 62, 63] {
            let mut damaged = bytes;
            damaged[index] ^= 1;
            assert!(
                BlockFrameDescriptor::decode(&damaged, descriptor.archive, mode).is_err(),
                "byte {index}"
            );
        }
        for field in [32..40, 40..48, 48..56] {
            let mut damaged = bytes;
            damaged[field.clone()].copy_from_slice(&u64::MAX.to_le_bytes());
            // Frame IDs may span u64; lengths may never authorize huge reads.
            if field.start != 32 {
                assert!(BlockFrameDescriptor::decode(&damaged, descriptor.archive, mode).is_err());
            }
        }
        let mut invalid = descriptor.clone();
        invalid.frame_id = 0;
        assert!(invalid.encode().is_err());
        let mut invalid = descriptor.clone();
        invalid.mode = FormatMode(0);
        assert!(invalid.encode().is_err());
        let mut invalid = descriptor.clone();
        invalid.compression = 1;
        assert!(invalid.encode().is_err());
        let mut invalid = descriptor;
        invalid.compression = 255;
        assert!(invalid.encode().is_err());
    }
}
