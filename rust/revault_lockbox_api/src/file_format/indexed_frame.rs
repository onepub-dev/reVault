//! Bounded, committed block index for a future native raw-frame encoding.
//!
//! The caller must obtain `commitment` from authenticated committed metadata,
//! not from the same untrusted frame. Index verification precedes block I/O.
//! A successful partial read authenticates every touched block, not untouched
//! blocks. Existing v4 whole-frame validation semantics remain unchanged.

use crate::crypto::strong_checksum;
use crate::page_buffer::ZeroizingBytes;
use crate::storage::Storage;
use crate::{Error, Result};
use sha2::{Digest, Sha256};
use std::ops::Range;

const BLOCK_BYTES: usize = 16 * 1024;
const MAX_FRAME_BYTES: usize = 4 * 1024 * 1024;
const HEADER_BYTES: usize = 32;
const DIGEST_BYTES: usize = 32;
const MAGIC: &[u8; 8] = b"LBXBLK01";

/// Common persisted descriptor for the staged native block representation.
/// The TOC must authenticate these bytes before they can authorize block reads.
/// Signing remains the native commit's responsibility, never a separate,
/// weaker per-frame signature. One encoding covers every protection mode.
#[derive(Clone, Debug, PartialEq, Eq)]
struct BlockFrameDescriptor {
    archive: crate::LockboxId,
    frame_id: u64,
    mode: crate::creation_options::FormatMode,
    compression: u8,
    logical_len: u64,
    stored_len: u64,
    salt: [u8; 32],
    index_commitment: [u8; 32],
}

impl BlockFrameDescriptor {
    const ENCODED_LEN: usize = 128;
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

    fn encode(&self) -> Result<[u8; Self::ENCODED_LEN]> {
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

    fn decode(
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

    fn physical_len(&self) -> Result<usize> {
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

/// The index is small (at most 8,224 bytes) and independent of requested range.
/// Including frame identity and logical size prevents swapping a block or index
/// between different frames even if they contain identical data.
struct AuthenticatedBlockIndex {
    frame_id: u64,
    logical_len: usize,
    hashes: Vec<[u8; DIGEST_BYTES]>,
}

impl AuthenticatedBlockIndex {
    fn build(frame_id: u64, bytes: &[u8]) -> Result<Self> {
        Self::index_len(bytes.len())?;
        Ok(Self {
            frame_id,
            logical_len: bytes.len(),
            hashes: bytes
                .chunks(BLOCK_BYTES)
                .enumerate()
                .map(|(ordinal, block)| block_digest(frame_id, ordinal, block))
                .collect(),
        })
    }

    fn index_len(logical_len: usize) -> Result<usize> {
        if logical_len > MAX_FRAME_BYTES {
            return Err(Error::CorruptRecord);
        }
        Ok(HEADER_BYTES + logical_len.div_ceil(BLOCK_BYTES) * DIGEST_BYTES)
    }

    fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_BYTES + self.hashes.len() * DIGEST_BYTES);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&self.frame_id.to_le_bytes());
        out.extend_from_slice(&(self.logical_len as u64).to_le_bytes());
        out.extend_from_slice(&(BLOCK_BYTES as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        for hash in &self.hashes {
            out.extend_from_slice(hash);
        }
        out
    }

    /// Read exactly the bounded index, never an allocation length obtained from
    /// an unverified on-disk header. All lengths/identity come from the caller's
    /// committed frame descriptor and must match the verified header.
    fn load(
        storage: &impl Storage,
        offset: u64,
        frame_id: u64,
        logical_len: usize,
        commitment: &[u8; 32],
    ) -> Result<Self> {
        let len = Self::index_len(logical_len)?;
        offset.checked_add(len as u64).ok_or(Error::CorruptRecord)?;
        let bytes = storage.read_at(offset, len)?;
        if bytes.len() != len || strong_checksum(&bytes) != *commitment {
            return Err(Error::CorruptRecord);
        }
        let mut header = [0; HEADER_BYTES];
        header[..8].copy_from_slice(MAGIC);
        header[8..16].copy_from_slice(&frame_id.to_le_bytes());
        header[16..24].copy_from_slice(&(logical_len as u64).to_le_bytes());
        header[24..28].copy_from_slice(&(BLOCK_BYTES as u32).to_le_bytes());
        if bytes[..HEADER_BYTES] != header {
            return Err(Error::CorruptRecord);
        }
        let hashes = bytes[HEADER_BYTES..]
            .chunks_exact(DIGEST_BYTES)
            .map(|bytes| bytes.try_into().expect("exact digest width"))
            .collect();
        Ok(Self {
            frame_id,
            logical_len,
            hashes,
        })
    }

    /// The data immediately follows its index. Read complete touched blocks,
    /// authenticate them, and expose only the requested slice. One contiguous
    /// read fills the output allocation directly; full-frame reads need neither
    /// a scratch buffer nor a second frame-sized copy. The allocation is wiped
    /// on any storage or integrity failure, including its unused capacity.
    fn read_range(
        &self,
        storage: &impl Storage,
        index_offset: u64,
        range: Range<usize>,
    ) -> Result<ZeroizingBytes> {
        if range.start > range.end || range.end > self.logical_len {
            return Err(Error::CorruptRecord);
        }
        let data_offset = index_offset
            .checked_add(Self::index_len(self.logical_len)? as u64)
            .ok_or(Error::CorruptRecord)?;
        data_offset
            .checked_add(self.logical_len as u64)
            .ok_or(Error::CorruptRecord)?;
        if range.is_empty() {
            return Ok(ZeroizingBytes::new(Vec::new()));
        }
        let first = range.start / BLOCK_BYTES;
        let aligned_start = first * BLOCK_BYTES;
        let aligned_end = range.end.div_ceil(BLOCK_BYTES) * BLOCK_BYTES;
        let read_len = aligned_end.min(self.logical_len) - aligned_start;
        let mut output = ZeroizingBytes::new(vec![0; read_len]);
        storage.read_at_into(data_offset + aligned_start as u64, &mut output)?;
        for (relative, block) in output.chunks(BLOCK_BYTES).enumerate() {
            let ordinal = first + relative;
            if block_digest(self.frame_id, ordinal, block) != self.hashes[ordinal] {
                return Err(Error::CorruptRecord);
            }
        }
        if range.start != aligned_start {
            output.copy_within(range.start - aligned_start..range.end - aligned_start, 0);
        }
        output.truncate(range.len());
        Ok(output)
    }
}

fn block_digest(frame_id: u64, ordinal: usize, data: &[u8]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"reVault indexed raw block v1\0");
    digest.update(frame_id.to_le_bytes());
    digest.update((ordinal as u64).to_le_bytes());
    digest.update((data.len() as u64).to_le_bytes());
    digest.update(data);
    digest.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageBackend;
    use std::sync::{Arc, Mutex};

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

    #[derive(Clone, Debug)]
    struct ObservedStorage {
        inner: StorageBackend,
        reads: Arc<Mutex<Vec<(u64, usize)>>>,
    }

    impl Storage for ObservedStorage {
        fn len(&self) -> Result<u64> {
            self.inner.len()
        }
        fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
            let mut bytes = vec![0; len];
            self.read_at_into(offset, &mut bytes)?;
            Ok(bytes)
        }
        fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
            self.reads.lock().unwrap().push((offset, out.len()));
            self.inner.read_at_into(offset, out)
        }
        fn append(&mut self, bytes: &[u8]) -> Result<u64> {
            self.inner.append(bytes)
        }
        fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
            self.inner.write_at(offset, bytes)
        }
        fn truncate(&mut self, len: u64) -> Result<()> {
            self.inner.truncate(len)
        }
        fn sync(&self) -> Result<()> {
            self.inner.sync()
        }
    }

    fn fixture(bytes: &[u8]) -> (ObservedStorage, [u8; 32], usize) {
        let index = AuthenticatedBlockIndex::build(17, bytes).unwrap().encode();
        let commitment = strong_checksum(&index);
        let data_offset = index.len();
        let mut stored = index;
        stored.extend_from_slice(bytes);
        (
            ObservedStorage {
                inner: StorageBackend::memory(stored),
                reads: Arc::default(),
            },
            commitment,
            data_offset,
        )
    }

    #[test]
    fn ranges_read_only_the_committed_index_and_touched_complete_blocks() {
        let bytes: Vec<u8> = (0..MAX_FRAME_BYTES).map(|n| (n % 251) as u8).collect();
        let (storage, commitment, data_offset) = fixture(&bytes);
        let index =
            AuthenticatedBlockIndex::load(&storage, 0, 17, bytes.len(), &commitment).unwrap();
        let range = BLOCK_BYTES + 123..BLOCK_BYTES + 456;
        assert_eq!(
            &*index.read_range(&storage, 0, range.clone()).unwrap(),
            &bytes[range]
        );
        assert_eq!(
            *storage.reads.lock().unwrap(),
            [(0, 8224), ((data_offset + BLOCK_BYTES) as u64, BLOCK_BYTES)]
        );
        storage.reads.lock().unwrap().clear();
        assert_eq!(
            &*index.read_range(&storage, 0, 0..bytes.len()).unwrap(),
            &bytes
        );
        assert_eq!(
            *storage.reads.lock().unwrap(),
            [(data_offset as u64, bytes.len())]
        );
    }

    #[test]
    fn noncanonical_header_fields_are_rejected_even_with_matching_commitment() {
        let payload = vec![7; BLOCK_BYTES];
        for at in [0, 8, 16, 24, 28] {
            let mut encoded = AuthenticatedBlockIndex::build(17, &payload)
                .unwrap()
                .encode();
            encoded[at] ^= 1;
            let commitment = strong_checksum(&encoded);
            let storage = StorageBackend::memory(encoded);
            assert!(
                AuthenticatedBlockIndex::load(&storage, 0, 17, payload.len(), &commitment).is_err()
            );
        }
    }

    #[test]
    fn file_backed_nonzero_offset_roundtrip_and_truncation() {
        let path = std::env::temp_dir().join(format!(
            "revault-indexed-frame-{}",
            crate::LockboxId::new_random().unwrap()
        ));
        let payload: Vec<u8> = (0..BLOCK_BYTES * 3 + 1).map(|n| (n % 251) as u8).collect();
        let encoded = AuthenticatedBlockIndex::build(17, &payload)
            .unwrap()
            .encode();
        let commitment = strong_checksum(&encoded);
        let mut storage = StorageBackend::create_file(&path, &[0; 731]).unwrap();
        let index_offset = storage.append(&encoded).unwrap();
        storage.append(&payload).unwrap();
        storage.sync().unwrap();
        drop(storage);
        let storage = StorageBackend::file(&path).unwrap();
        let index =
            AuthenticatedBlockIndex::load(&storage, index_offset, 17, payload.len(), &commitment)
                .unwrap();
        for range in [
            0..payload.len(),
            19..BLOCK_BYTES + 99,
            payload.len() - 1..payload.len(),
        ] {
            assert_eq!(
                &*index
                    .read_range(&storage, index_offset, range.clone())
                    .unwrap(),
                &payload[range]
            );
        }
        drop(storage);
        let mut storage = StorageBackend::file_for_write(&path).unwrap();
        storage
            .truncate(index_offset + encoded.len() as u64 + payload.len() as u64 - 1)
            .unwrap();
        assert!(index
            .read_range(&storage, index_offset, 0..payload.len())
            .is_err());
        drop(storage);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn empty_tail_cross_block_and_full_ranges_roundtrip() {
        for len in [
            0,
            1,
            BLOCK_BYTES - 1,
            BLOCK_BYTES,
            BLOCK_BYTES + 1,
            MAX_FRAME_BYTES,
        ] {
            let bytes: Vec<u8> = (0..len).map(|n| (n % 251) as u8).collect();
            let (storage, commitment, _) = fixture(&bytes);
            let index = AuthenticatedBlockIndex::load(&storage, 0, 17, len, &commitment).unwrap();
            for range in [0..len, len / 2..len, len..len] {
                assert_eq!(
                    &*index.read_range(&storage, 0, range.clone()).unwrap(),
                    &bytes[range]
                );
            }
            assert!(index.read_range(&storage, 0, 0..len + 1).is_err());
            assert!(index.read_range(&storage, u64::MAX, 0..len).is_err());
        }
        assert!(AuthenticatedBlockIndex::build(17, &vec![0; MAX_FRAME_BYTES + 1]).is_err());
    }

    #[test]
    fn untrusted_index_is_rejected_before_data_io_or_large_allocation() {
        let (mut storage, commitment, _) = fixture(&vec![7; BLOCK_BYTES * 2]);
        assert!(
            AuthenticatedBlockIndex::load(&storage, 0, 18, BLOCK_BYTES * 2, &commitment).is_err()
        );
        storage.reads.lock().unwrap().clear();
        assert!(AuthenticatedBlockIndex::load(&storage, 0, 17, usize::MAX, &commitment).is_err());
        assert!(storage.reads.lock().unwrap().is_empty());
        storage
            .write_at(HEADER_BYTES as u64, &[0; DIGEST_BYTES])
            .unwrap();
        assert!(
            AuthenticatedBlockIndex::load(&storage, 0, 17, BLOCK_BYTES * 2, &commitment).is_err()
        );
        assert_eq!(
            *storage.reads.lock().unwrap(),
            [(0, HEADER_BYTES + 2 * DIGEST_BYTES)]
        );
    }

    #[test]
    fn corrupt_bytes_outside_slice_in_touched_block_fail_but_untouched_blocks_are_lazy() {
        let bytes = vec![7; BLOCK_BYTES * 3 + 1];
        let (mut storage, commitment, data_offset) = fixture(&bytes);
        let index =
            AuthenticatedBlockIndex::load(&storage, 0, 17, bytes.len(), &commitment).unwrap();
        storage
            .write_at((data_offset + BLOCK_BYTES - 1) as u64, &[8])
            .unwrap();
        assert!(index.read_range(&storage, 0, 0..1).is_err());
        assert_eq!(
            &*index
                .read_range(&storage, 0, BLOCK_BYTES..BLOCK_BYTES + 1)
                .unwrap(),
            &[7]
        );
        assert!(index.read_range(&storage, 0, 0..bytes.len()).is_err());
        storage
            .truncate((data_offset + BLOCK_BYTES * 3) as u64)
            .unwrap();
        assert!(index
            .read_range(&storage, 0, bytes.len() - 1..bytes.len())
            .is_err());
    }
}
