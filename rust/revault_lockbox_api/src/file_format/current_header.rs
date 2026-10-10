//! Native header facade for legacy v2 archives and configurable v3 archives.

use crate::checked::read_u16_le;
use crate::crypto::strong_checksum;
use crate::file_format::header_v2::{self, Publication};
use crate::lockbox_id::LockboxId;
use crate::storage::{Storage, StorageBackend};
use crate::{ArtifactKind, Error, Result};

/// Current native lockbox format written by the crash-recoverable protocol.
pub const LOCKBOX_FORMAT_VERSION: u16 = 3;
pub(crate) const HEADER_LEN: usize = header_v2::REGION_LEN;
const V1_HEADER_LEN: usize = 96;
const V1_CHECKSUM_START: usize = 64;
const V1_MAGIC: &[u8; 8] = b"LBX1HDR\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LockboxHeader {
    pub(crate) format_mode: crate::creation_options::FormatMode,
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
            format_mode: Default::default(),
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
    if let Some(found) = unsupported_format_discriminator(bytes) {
        return Err(Error::UnsupportedFormatVersion {
            artifact: ArtifactKind::Lockbox,
            found: u32::from(found),
            supported: u32::from(LOCKBOX_FORMAT_VERSION),
        });
    }
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
        format_mode: header.format_mode,
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

/// Read a checksummed v1/v2/v3 header or recognize an unsupported discriminator.
///
/// Recognizing an unsupported discriminator does not authenticate or open its archive.
pub fn probe_lockbox_format_version(bytes: &[u8]) -> Result<u16> {
    if let Some(found) = unsupported_format_discriminator(bytes) {
        return Ok(found);
    }
    if bytes.get(..8) == Some(V1_MAGIC.as_slice()) {
        return probe_v1(bytes);
    }
    let header = header_v2::read_region(bytes)?;
    Ok(if header.format_mode.0 == 0 { 2 } else { 3 })
}

// Read only the family marker and version, never unsupported layout fields.
// Native readers support historical 2 and current 3. Slots in those layouts
// begin at 0/160; the known newer layout also has a slot at 192.
fn unsupported_format_discriminator(bytes: &[u8]) -> Option<u16> {
    for start in [0, 160, 192] {
        let Some(magic) = bytes.get(start..start + 8) else {
            continue;
        };
        let native_family =
            &magic[..3] == b"LBX" && (b'2'..=b'9').contains(&magic[3]) && &magic[4..] == b"HDR\0";
        if native_family {
            let Some(version_bytes) = bytes.get(start + 8..start + 10) else {
                continue;
            };
            let version = read_u16_le(version_bytes).ok()?;
            if !matches!(version, 2 | 3) {
                return Some(version);
            }
        }
    }
    None
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
    fn unsupported_headers_are_identified_but_never_opened() {
        for slot in [0, 160, 192] {
            for version in [0_u16, 1, 4, 5, 99] {
                let mut bytes = vec![0; 384];
                // Recognize the declared version even in a known older family.
                bytes[slot..slot + 8].copy_from_slice(b"LBX3HDR\0");
                bytes[slot + 8..slot + 10].copy_from_slice(&version.to_le_bytes());
                assert_eq!(probe_lockbox_format_version(&bytes).unwrap(), version);
                assert!(matches!(read_header(&bytes),
                    Err(Error::UnsupportedFormatVersion { found, supported: 3, .. })
                    if found == u32::from(version)));
            }
        }
    }

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
