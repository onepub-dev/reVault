//! Native v4 header facade and historical format discriminators for migration.

use crate::checked::read_u16_le;
use crate::crypto::strong_checksum;
use crate::file_format::header_v2::{self, Publication};
use crate::lockbox_id::LockboxId;
use crate::storage::{Storage, StorageBackend};
use crate::{ArtifactKind, Error, Result};

/// Current native lockbox format written by the crash-recoverable protocol.
pub const LOCKBOX_FORMAT_VERSION: u16 = 4;
pub(crate) const HEADER_LEN: usize = header_v2::REGION_LEN;
const V1_HEADER_LEN: usize = 96;
const V1_CHECKSUM_START: usize = 64;
const V1_MAGIC: &[u8; 8] = b"LBX1HDR\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LockboxHeader {
    pub(crate) sealed_len: u64,
    pub(crate) trim_origin_len: u64,
    pub(crate) preparing: bool,
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
            sealed_len: HEADER_LEN as u64,
            preparing: false,
            trim_origin_len: 0,
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

/// Authenticate the initial empty state using the same header domain as every
/// later publication. Sequence zero is a real state, never an authentication opt-out.
pub(crate) fn authenticated_initial_header(
    lockbox_id: LockboxId,
    key: &crate::SecretVec,
) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    write_header(&mut bytes, 0, 0, 0, lockbox_id, 0);
    let tag = key.with_bytes(|key| crate::crypto::metadata_auth_tag(key, &bytes[..128]))?;
    bytes[128..152].copy_from_slice(&tag);
    let digest = strong_checksum(&bytes[..160]);
    bytes[160..192].copy_from_slice(&digest);
    Ok(bytes)
}

/// Recovery-only repair of damaged slot magics. All remaining bytes, including
/// checksums and authentication tags, are preserved. A checksum-valid candidate
/// still requires normal publication/owner authentication before it is trusted.
pub(crate) fn recover_header_magic(bytes: &[u8]) -> Option<[u8; HEADER_LEN]> {
    let mut header: [u8; HEADER_LEN] = bytes.get(..HEADER_LEN)?.try_into().ok()?;
    for slot in header.chunks_exact_mut(header_v2::SLOT_LEN) {
        slot[..8].copy_from_slice(header_v2::MAGIC);
    }
    read_header(&header).ok()?;
    Some(header)
}

/// Construct a recovery candidate for a lost root pointer. Preserve the original
/// metadata authentication tag: normal open must authenticate the restored bytes
/// before any entry is trusted. This does not authorize the supplied pointer.
pub(crate) fn recover_header_root_pointer(
    bytes: &[u8],
    commit_root_offset: u64,
) -> Option<[u8; HEADER_LEN]> {
    let selected = read_header(bytes).ok()?;
    if selected.commit_root_offset != 0 || commit_root_offset == 0 {
        return None;
    }
    let mut header: [u8; HEADER_LEN] = bytes.get(..HEADER_LEN)?.try_into().ok()?;
    let start = selected.slot_index * header_v2::SLOT_LEN;
    let slot = &mut header[start..start + header_v2::SLOT_LEN];
    slot[24..32].copy_from_slice(&commit_root_offset.to_le_bytes());
    let checksum_start = header_v2::SLOT_LEN - 32;
    let checksum = strong_checksum(&slot[..checksum_start]);
    slot[checksum_start..].copy_from_slice(&checksum);
    read_header(&header).ok()?;
    Some(header)
}

pub(crate) fn read_header_for_scan(bytes: &[u8]) -> Result<LockboxHeader> {
    let header = read_header(bytes);
    if header.is_err() && !matches!(header, Err(Error::UnsupportedFormatVersion { .. })) {
        if let Some(repaired) = recover_header_magic(bytes) {
            return read_header(&repaired);
        }
    }
    header
}

pub(crate) fn read_header(bytes: &[u8]) -> Result<LockboxHeader> {
    if let Some(found) = future_format_version(bytes) {
        return Err(Error::UnsupportedFormatVersion {
            artifact: ArtifactKind::Lockbox,
            found: u32::from(found),
            supported: u32::from(LOCKBOX_FORMAT_VERSION),
        });
    }
    if bytes
        .get(..8)
        .is_some_and(|magic| magic == b"LBX2HDR\0" || magic == b"LBX3HDR\0")
    {
        return Err(Error::UnsupportedFormatVersion {
            artifact: ArtifactKind::Lockbox,
            found: u32::from(probe_lockbox_format_version(bytes)?),
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
        sealed_len: header.sealed_len,
        trim_origin_len: header.trim_origin_len,
        preparing: header.preparing,
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

// A newer magic identifies an unsupported format, not a valid archive. Its
// checksum and layout may have changed, so never try to validate or decode it
// using this build's layout. Require agreement with the version field to avoid
// treating a single damaged magic byte as a format upgrade.
fn future_magic_version(bytes: &[u8]) -> Option<u16> {
    let prefix = bytes.get(..10)?;
    let version = u16::from_le_bytes([prefix[8], prefix[9]]);
    (prefix[..3] == *b"LBX"
        && prefix[4..8] == *b"HDR\0"
        && prefix[3].is_ascii_digit()
        && u16::from(prefix[3] - b'0') == version
        && version > LOCKBOX_FORMAT_VERSION)
        .then_some(version)
}

// Refuse a future slot even if the other slot remains readable by this build.
// If the magic is unchanged, require the existing checksummed discriminator.
fn future_format_version(bytes: &[u8]) -> Option<u16> {
    bytes
        .chunks(192)
        .take(2)
        .filter_map(|slot| {
            if let Some(version) = future_magic_version(slot) {
                return Some(version);
            }
            if slot.len() < 192 {
                return None;
            }
            let version = u16::from_le_bytes([slot[8], slot[9]]);
            (slot[..8] == *b"LBX4HDR\0"
                && version > LOCKBOX_FORMAT_VERSION
                && slot[160..192] == strong_checksum(&slot[..160]))
            .then_some(version)
        })
        .max()
}

/// Read a native format discriminator without opening its contents.
///
/// Known layouts require a valid checksum. A newer versioned magic is reported
/// without validating its unknown layout; this does not establish integrity.
pub fn probe_lockbox_format_version(bytes: &[u8]) -> Result<u16> {
    if let Some(version) = future_format_version(bytes) {
        return Ok(version);
    }
    if bytes.get(..8) == Some(V1_MAGIC.as_slice()) {
        return probe_v1(bytes);
    }
    // A discriminator only: historical decoding lives in versioned exporters.
    let mut historical = None;
    for slot in bytes.chunks_exact(160).take(2) {
        let version = read_u16_le(&slot[8..10])?;
        if ((version == 2 && &slot[..8] == b"LBX2HDR\0")
            || (version == 3 && &slot[..8] == b"LBX3HDR\0"))
            && slot[128..160] == strong_checksum(&slot[..128])
        {
            let generation = crate::checked::read_u64_le(&slot[16..24])?;
            if historical.is_none_or(|(old, _)| generation > old) {
                historical = Some((generation, version));
            }
        }
    }
    if let Some((_, version)) = historical {
        return Ok(version);
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
    fn future_magic_does_not_require_the_current_layout_or_checksum() {
        let prefix = b"LBX5HDR\0\x05\0";
        assert_eq!(probe_lockbox_format_version(prefix).unwrap(), 5);
        for offset in [0, header_v2::SLOT_LEN] {
            let mut bytes = Vec::new();
            write_header(&mut bytes, 100, 4, 200, LockboxId::from_bytes([7; 16]), 300);
            bytes[offset..offset + prefix.len()].copy_from_slice(prefix);
            let unchanged = bytes.clone();
            assert_eq!(probe_lockbox_format_version(&bytes).unwrap(), 5);
            assert!(matches!(
                read_header(&bytes),
                Err(Error::UnsupportedFormatVersion {
                    found: 5,
                    supported: 4,
                    ..
                })
            ));
            assert!(matches!(
                read_header_for_scan(&bytes),
                Err(Error::UnsupportedFormatVersion { found: 5, .. })
            ));
            assert_eq!(bytes, unchanged);
        }
        let mut bytes = vec![0; HEADER_LEN];
        bytes[..prefix.len()].copy_from_slice(prefix);
        bytes[header_v2::SLOT_LEN..header_v2::SLOT_LEN + 10].copy_from_slice(b"LBX9HDR\0\x09\0");
        assert_eq!(probe_lockbox_format_version(&bytes).unwrap(), 9);
    }

    #[test]
    fn future_magic_requires_a_complete_matching_discriminator() {
        for prefix in [b"LBX5HDR\0\x04\0", b"LBX4HDR\0\x05\0", b"XYZ5HDR\0\x05\0"] {
            assert!(probe_lockbox_format_version(prefix).is_err());
        }
        assert!(probe_lockbox_format_version(b"LBX5HDR\0").is_err());
    }

    #[test]
    fn future_discriminator_requires_checksum_and_refuses_either_slot() {
        for slot in [0, 192] {
            let mut bytes = vec![0; 384];
            bytes[slot..slot + 8].copy_from_slice(b"LBX4HDR\0");
            bytes[slot + 8..slot + 10].copy_from_slice(&5_u16.to_le_bytes());
            assert!(probe_lockbox_format_version(&bytes).is_err());
            let checksum = strong_checksum(&bytes[slot..slot + 160]);
            bytes[slot + 160..slot + 192].copy_from_slice(&checksum);
            assert_eq!(probe_lockbox_format_version(&bytes).unwrap(), 5);
            assert!(matches!(
                read_header(&bytes),
                Err(Error::UnsupportedFormatVersion { found: 5, .. })
            ));
            bytes[slot + 32] ^= 1;
            assert!(matches!(read_header(&bytes), Err(Error::CorruptHeader)));
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
        assert_eq!(probe_lockbox_format_version(&bytes).unwrap(), 4);
    }

    #[test]
    fn recovery_magic_repair_preserves_checksums_and_rejects_other_damage() {
        let mut bytes = Vec::new();
        write_header(&mut bytes, 100, 4, 200, LockboxId::from_bytes([7; 16]), 300);
        let expected = read_header(&bytes).unwrap();
        bytes[0] ^= 0xff;
        assert!(read_header(&bytes).is_err());
        let unchanged = bytes.clone();
        assert_eq!(read_header_for_scan(&bytes).unwrap(), expected);
        assert_eq!(bytes, unchanged, "repair never changes source storage");
        bytes[32] ^= 1;
        assert!(recover_header_magic(&bytes).is_none());
        assert!(recover_header_magic(&bytes[..HEADER_LEN - 1]).is_none());
        assert!(recover_header_magic(&vec![0; HEADER_LEN]).is_none());
    }
}
