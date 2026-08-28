//! Format-v2 header facade, staged separately until all v2 transaction paths are wired.

use crate::checked::read_u16_le;
use crate::crypto::strong_checksum;
use crate::file_format::header_v2::{self, Publication};
use crate::lockbox_id::LockboxId;
use crate::storage::{Storage, StorageBackend};
use crate::{ArtifactKind, Error, Result};

/// Current native lockbox format written by the crash-recoverable protocol.
pub const LOCKBOX_FORMAT_VERSION: u16 = 2;
pub(crate) const HEADER_LEN: usize = header_v2::REGION_LEN;
const V1_HEADER_LEN: usize = 96;
const V1_CHECKSUM_START: usize = 64;
const V1_MAGIC: &[u8; 8] = b"LBX1HDR\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LockboxHeader {
    pub(crate) slot_index: usize,
    pub(crate) generation: u64,
    pub(crate) commit_root_offset: u64,
    pub(crate) sequence: u64,
    pub(crate) key_directory_offset: u64,
    pub(crate) key_directory_mirror_offset: u64,
    pub(crate) lockbox_id: LockboxId,
    pub(crate) commit_auth_offset: u64,
    pub(crate) cleanup_sequence: u64,
    pub(crate) cleanup_completed_ranges: u32,
    pub(crate) cleanup_completed_pages: u32,
    pub(crate) cleanup_completed_bytes: u64,
    pub(crate) metadata_auth_tag: [u8; 24],
}

pub(crate) fn write_header(
    bytes: &mut Vec<u8>,
    commit_root_offset: u64,
    sequence: u64,
    key_directory_offset: u64,
    lockbox_id: LockboxId,
    commit_auth_offset: u64,
) {
    bytes.resize(HEADER_LEN, 0);
    bytes[..HEADER_LEN].fill(0);
    header_v2::write_slot(
        bytes,
        0,
        Publication {
            generation: 1,
            commit_root_offset,
            sequence,
            key_directory_offset,
            key_directory_mirror_offset: 0,
            lockbox_id,
            commit_auth_offset,
            cleanup_sequence: sequence,
            cleanup_completed_ranges: 0,
            cleanup_completed_pages: 0,
            cleanup_completed_bytes: 0,
            metadata_auth_tag: [0; 24],
        },
    )
    .expect("header initialization buffer is valid");
}

pub(crate) fn read_header(bytes: &[u8]) -> Result<LockboxHeader> {
    if bytes.get(..8) == Some(V1_MAGIC.as_slice()) {
        let found = probe_v1(bytes)?;
        return Err(Error::UnsupportedFormatVersion {
            artifact: ArtifactKind::Lockbox,
            found: u32::from(found),
            supported: u32::from(LOCKBOX_FORMAT_VERSION),
        });
    }
    let header = header_v2::read_region(bytes)?;
    Ok(LockboxHeader {
        slot_index: header.slot_index,
        generation: header.generation,
        commit_root_offset: header.commit_root_offset,
        sequence: header.sequence,
        key_directory_offset: header.key_directory_offset,
        key_directory_mirror_offset: header.key_directory_mirror_offset,
        lockbox_id: header.lockbox_id,
        commit_auth_offset: header.commit_auth_offset,
        cleanup_sequence: header.cleanup_sequence,
        cleanup_completed_ranges: header.cleanup_completed_ranges,
        cleanup_completed_pages: header.cleanup_completed_pages,
        cleanup_completed_bytes: header.cleanup_completed_bytes,
        metadata_auth_tag: header.metadata_auth_tag,
    })
}

pub(crate) fn publish_header(
    storage: &mut StorageBackend,
    current_slot: usize,
    publication: Publication,
) -> Result<usize> {
    let next_slot = (current_slot + 1) % header_v2::SLOT_COUNT;
    let slot = header_v2::encode_slot(publication)?;
    storage.write_at((next_slot * header_v2::SLOT_LEN) as u64, &slot)?;
    storage.sync()?;
    Ok(next_slot)
}

/// Read an authenticated v1 or v2 native format discriminator.
pub fn probe_lockbox_format_version(bytes: &[u8]) -> Result<u16> {
    if bytes.get(..8) == Some(V1_MAGIC.as_slice()) {
        return probe_v1(bytes);
    }
    header_v2::read_region(bytes)?;
    Ok(LOCKBOX_FORMAT_VERSION)
}

fn probe_v1(bytes: &[u8]) -> Result<u16> {
    if bytes.len() < V1_HEADER_LEN {
        return Err(Error::Truncated);
    }
    let expected = strong_checksum(&bytes[..V1_CHECKSUM_START]);
    if bytes[V1_CHECKSUM_START..V1_HEADER_LEN] != expected {
        return Err(Error::CorruptHeader);
    }
    read_u16_le(&bytes[8..10]).map_err(|_| Error::CorruptHeader)
}

#[cfg(feature = "vault-integration")]
pub fn read_lockbox_id(bytes: &[u8]) -> Result<LockboxId> {
    Ok(read_header(bytes)?.lockbox_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialized_header_round_trips() {
        let id = LockboxId::from_bytes([7; 16]);
        let mut bytes = Vec::new();
        write_header(&mut bytes, 100, 4, 200, id, 300);
        let header = read_header(&bytes).unwrap();
        assert_eq!(header.commit_root_offset, 100);
        assert_eq!(header.cleanup_sequence, 4);
        assert_eq!(header.lockbox_id, id);
        assert_eq!(probe_lockbox_format_version(&bytes).unwrap(), 2);
    }
}
