use crate::checked::{array_16, read_u16_le, read_u32_le, read_u64_le};
use crate::crypto::strong_checksum;
use crate::lockbox_id::LockboxId;
use crate::{Error, Result};

pub(crate) const MAGIC: &[u8; 8] = b"LBX2HDR\0";
pub(crate) const FORMAT_VERSION: u16 = 2;
pub(crate) const SLOT_LEN: usize = 160;
pub(crate) const SLOT_COUNT: usize = 2;
pub(crate) const REGION_LEN: usize = SLOT_LEN * SLOT_COUNT;
const CHECKSUM_START: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Header {
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

#[derive(Debug, Clone, Copy)]
pub(crate) struct Publication {
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

#[cfg(test)]
pub(crate) fn initial_region(lockbox_id: LockboxId) -> Vec<u8> {
    let mut bytes = vec![0; REGION_LEN];
    let publication = Publication {
        generation: 1,
        commit_root_offset: 0,
        sequence: 0,
        key_directory_offset: 0,
        key_directory_mirror_offset: 0,
        lockbox_id,
        commit_auth_offset: 0,
        cleanup_sequence: 0,
        cleanup_completed_ranges: 0,
        cleanup_completed_pages: 0,
        cleanup_completed_bytes: 0,
        metadata_auth_tag: [0; 24],
    };
    write_slot(&mut bytes, 0, publication).expect("initial header buffer is valid");
    bytes
}

pub(crate) fn write_slot(
    bytes: &mut [u8],
    slot_index: usize,
    publication: Publication,
) -> Result<()> {
    if slot_index >= SLOT_COUNT || bytes.len() < REGION_LEN {
        return Err(Error::InvalidInput(
            "header publication buffer or slot is invalid".to_string(),
        ));
    }
    let start = slot_index * SLOT_LEN;
    let slot = &mut bytes[start..start + SLOT_LEN];
    slot.fill(0);
    slot[..104].copy_from_slice(&metadata_auth_message(publication));
    slot[104..128].copy_from_slice(&publication.metadata_auth_tag);
    let digest = strong_checksum(&slot[..CHECKSUM_START]);
    slot[CHECKSUM_START..SLOT_LEN].copy_from_slice(&digest);
    Ok(())
}

pub(crate) fn metadata_auth_message(publication: Publication) -> [u8; 104] {
    let mut message = [0u8; 104];
    message[0..8].copy_from_slice(MAGIC);
    message[8..10].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
    message[12..16].copy_from_slice(&(SLOT_LEN as u32).to_le_bytes());
    message[16..24].copy_from_slice(&publication.generation.to_le_bytes());
    message[24..32].copy_from_slice(&publication.commit_root_offset.to_le_bytes());
    message[32..40].copy_from_slice(&publication.sequence.to_le_bytes());
    message[40..48].copy_from_slice(&publication.key_directory_offset.to_le_bytes());
    message[48..56].copy_from_slice(&publication.key_directory_mirror_offset.to_le_bytes());
    message[56..72].copy_from_slice(publication.lockbox_id.as_bytes());
    message[72..80].copy_from_slice(&publication.commit_auth_offset.to_le_bytes());
    message[80..88].copy_from_slice(&publication.cleanup_sequence.to_le_bytes());
    message[88..92].copy_from_slice(&publication.cleanup_completed_ranges.to_le_bytes());
    message[92..96].copy_from_slice(&publication.cleanup_completed_pages.to_le_bytes());
    message[96..104].copy_from_slice(&publication.cleanup_completed_bytes.to_le_bytes());
    message
}

pub(crate) fn encode_slot(publication: Publication) -> Result<Vec<u8>> {
    let mut region = vec![0; REGION_LEN];
    write_slot(&mut region, 0, publication)?;
    region.truncate(SLOT_LEN);
    Ok(region)
}

pub(crate) fn read_region(bytes: &[u8]) -> Result<Header> {
    if bytes.len() < REGION_LEN {
        return Err(Error::Truncated);
    }
    let mut best = None;
    for slot_index in 0..SLOT_COUNT {
        let start = slot_index * SLOT_LEN;
        let slot = &bytes[start..start + SLOT_LEN];
        let Ok(header) = read_slot(slot, slot_index) else {
            continue;
        };
        if best
            .as_ref()
            .is_none_or(|existing: &Header| header.generation > existing.generation)
        {
            best = Some(header);
        }
    }
    best.ok_or(Error::CorruptHeader)
}

fn read_slot(slot: &[u8], slot_index: usize) -> Result<Header> {
    if slot.len() != SLOT_LEN || slot.get(0..8) != Some(MAGIC.as_slice()) {
        return Err(Error::CorruptHeader);
    }
    if read_u16_le(&slot[8..10]).map_err(|_| Error::CorruptHeader)? != FORMAT_VERSION
        || read_u16_le(&slot[10..12]).map_err(|_| Error::CorruptHeader)? != 0
        || read_u32_le(&slot[12..16]).map_err(|_| Error::CorruptHeader)? as usize != SLOT_LEN
    {
        return Err(Error::CorruptHeader);
    }
    let expected = strong_checksum(&slot[..CHECKSUM_START]);
    if slot[CHECKSUM_START..SLOT_LEN] != expected {
        return Err(Error::CorruptHeader);
    }
    let generation = read_u64_le(&slot[16..24]).map_err(|_| Error::CorruptHeader)?;
    if generation == 0 {
        return Err(Error::CorruptHeader);
    }
    Ok(Header {
        slot_index,
        generation,
        commit_root_offset: read_u64_le(&slot[24..32]).map_err(|_| Error::CorruptHeader)?,
        sequence: read_u64_le(&slot[32..40]).map_err(|_| Error::CorruptHeader)?,
        key_directory_offset: read_u64_le(&slot[40..48]).map_err(|_| Error::CorruptHeader)?,
        key_directory_mirror_offset: read_u64_le(&slot[48..56])
            .map_err(|_| Error::CorruptHeader)?,
        lockbox_id: LockboxId::from_bytes(
            array_16(&slot[56..72]).map_err(|_| Error::CorruptHeader)?,
        ),
        commit_auth_offset: read_u64_le(&slot[72..80]).map_err(|_| Error::CorruptHeader)?,
        cleanup_sequence: read_u64_le(&slot[80..88]).map_err(|_| Error::CorruptHeader)?,
        cleanup_completed_ranges: read_u32_le(&slot[88..92]).map_err(|_| Error::CorruptHeader)?,
        cleanup_completed_pages: read_u32_le(&slot[92..96]).map_err(|_| Error::CorruptHeader)?,
        cleanup_completed_bytes: read_u64_le(&slot[96..104]).map_err(|_| Error::CorruptHeader)?,
        metadata_auth_tag: slot[104..128]
            .try_into()
            .map_err(|_| Error::CorruptHeader)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn publication(generation: u64, sequence: u64) -> Publication {
        Publication {
            generation,
            commit_root_offset: sequence * 100,
            sequence,
            key_directory_offset: sequence * 200,
            key_directory_mirror_offset: sequence * 300,
            lockbox_id: LockboxId::from_bytes([sequence as u8; 16]),
            commit_auth_offset: sequence * 400,
            cleanup_sequence: sequence.saturating_sub(1),
            cleanup_completed_ranges: 0,
            cleanup_completed_pages: 0,
            cleanup_completed_bytes: 0,
            metadata_auth_tag: [0; 24],
        }
    }

    #[test]
    fn newest_valid_slot_wins() {
        let mut bytes = vec![0; REGION_LEN];
        write_slot(&mut bytes, 0, publication(1, 10)).unwrap();
        write_slot(&mut bytes, 1, publication(2, 11)).unwrap();
        let header = read_region(&bytes).unwrap();
        assert_eq!(header.slot_index, 1);
        assert_eq!(header.sequence, 11);
        assert_eq!(header.key_directory_mirror_offset, 3_300);
    }

    #[test]
    fn torn_newest_slot_falls_back_to_prior_slot() {
        let mut bytes = vec![0; REGION_LEN];
        write_slot(&mut bytes, 0, publication(1, 10)).unwrap();
        write_slot(&mut bytes, 1, publication(2, 11)).unwrap();
        bytes[SLOT_LEN + 24] ^= 0x55;
        let header = read_region(&bytes).unwrap();
        assert_eq!(header.slot_index, 0);
        assert_eq!(header.sequence, 10);
    }

    #[test]
    fn initial_region_has_one_valid_slot() {
        let id = LockboxId::from_bytes([7; 16]);
        let header = read_region(&initial_region(id)).unwrap();
        assert_eq!(header.generation, 1);
        assert_eq!(header.lockbox_id, id);
        assert_eq!(header.cleanup_sequence, 0);
    }
}
