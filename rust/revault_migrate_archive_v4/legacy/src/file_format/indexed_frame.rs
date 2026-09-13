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
