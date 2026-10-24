//! Canonical authenticated native page references.
//!
//! Encrypted modes store these references only inside encrypted metadata. Hashing
//! streams secure payloads through scoped access without plaintext copies.

use crate::archive_reader::creation_options::FormatMode;
use crate::archive_reader::page::{DecodedPage, DEFAULT_DATA_PAGE_BYTES, PAGE_HEADER_LEN};
use crate::archive_reader::{Error, LockboxId, Result};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct PageCommitment {
    pub(crate) physical_len: u64,
    pub(crate) digest: [u8; 32],
}

impl PageCommitment {
    pub(crate) fn from_page(
        archive: LockboxId,
        mode: FormatMode,
        physical_len: u64,
        page: &DecodedPage,
    ) -> Result<Self> {
        if physical_len < PAGE_HEADER_LEN as u64 || physical_len > DEFAULT_DATA_PAGE_BYTES as u64 {
            return Err(Error::CorruptRecord);
        }
        let mut hash = Sha256::new();
        hash.update(b"revault-v4-canonical-page-v1\0");
        hash.update(archive.as_bytes());
        hash.update(mode.0.to_le_bytes());
        hash.update(physical_len.to_le_bytes());
        hash.update(page.page_id.to_le_bytes());
        hash.update(page.sequence.to_le_bytes());
        hash.update((page.objects.len() as u64).to_le_bytes());
        for object in &page.objects {
            hash.update([object.kind as u8]);
            hash.update(object.id.to_le_bytes());
            hash.update((object.payload_len() as u64).to_le_bytes());
            object.with_payload(|payload| hash.update(payload))?;
        }
        Ok(Self {
            physical_len,
            digest: hash.finalize().into(),
        })
    }

    pub(crate) fn verify(
        self,
        archive: LockboxId,
        mode: FormatMode,
        physical_len: u64,
        page: &DecodedPage,
    ) -> Result<()> {
        if self != Self::from_page(archive, mode, physical_len, page)? {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive_reader::page::{PageObject, PageObjectKind, DEFAULT_METADATA_PAGE_BYTES};

    #[test]
    fn canonical_page_commitment_round_trips() {
        let archive = LockboxId::from_bytes([7; 16]);
        let page = DecodedPage {
            page_id: 512,
            sequence: 3,
            objects: vec![PageObject::new(
                PageObjectKind::TocLeaf,
                9,
                b"metadata".to_vec(),
            )],
        };
        let len = DEFAULT_METADATA_PAGE_BYTES as u64;
        let commitment =
            PageCommitment::from_page(archive, FormatMode::default(), len, &page).unwrap();
        commitment
            .verify(archive, FormatMode::default(), len, &page)
            .unwrap();
        assert_eq!(commitment.physical_len, len);
        assert_ne!(commitment.digest, [0; 32]);
    }

    #[test]
    fn canonical_page_commitment_requires_expected_context_and_model() {
        let archive = LockboxId::from_bytes([7; 16]);
        let mode = FormatMode::default();
        let page = DecodedPage {
            page_id: 512,
            sequence: 3,
            objects: vec![PageObject::new(
                PageObjectKind::TocLeaf,
                9,
                b"metadata".to_vec(),
            )],
        };
        let len = DEFAULT_METADATA_PAGE_BYTES as u64;
        let expected = PageCommitment::from_page(archive, mode, len, &page).unwrap();
        assert!(expected
            .verify(LockboxId::from_bytes([8; 16]), mode, len, &page)
            .is_err());
        assert!(expected
            .verify(archive, FormatMode(mode.0 ^ 1), len, &page)
            .is_err());
        assert!(expected.verify(archive, mode, len / 2, &page).is_err());
        let mut alternate = page.clone();
        alternate.sequence += 1;
        assert!(expected.verify(archive, mode, len, &alternate).is_err());
        alternate = page.clone();
        alternate.page_id += 1;
        assert!(expected.verify(archive, mode, len, &alternate).is_err());
        alternate = page.clone();
        alternate.objects[0].id += 1;
        assert!(expected.verify(archive, mode, len, &alternate).is_err());
        alternate.objects[0] = PageObject::new(PageObjectKind::TocLeaf, 9, b"updated".to_vec());
        assert!(expected.verify(archive, mode, len, &alternate).is_err());
        alternate.objects.push(page.objects[0].clone());
        assert!(expected.verify(archive, mode, len, &alternate).is_err());
    }

    #[test]
    fn canonical_object_reference_requires_exact_object_and_extent() {
        let archive = LockboxId::from_bytes([7; 16]);
        let mode = FormatMode::default();
        let original = PageObject::new(PageObjectKind::Symlink, 9, b"target".to_vec());
        let reference = ObjectReference {
            physical_len: DEFAULT_METADATA_PAGE_BYTES as u64,
            object_id: original.id,
            digest: object_digest(archive, mode, &original).unwrap(),
        };
        let page = DecodedPage {
            page_id: 512,
            sequence: 3,
            objects: vec![original.clone()],
        };
        reference
            .verify(archive, mode, reference.physical_len, &page)
            .unwrap();
        assert!(reference
            .verify(archive, mode, reference.physical_len + 1, &page)
            .is_err());
        for object in [
            PageObject::new(PageObjectKind::Symlink, 9, b"other".to_vec()),
            PageObject::new(PageObjectKind::FileData, 9, b"target".to_vec()),
            PageObject::new(PageObjectKind::Symlink, 10, b"target".to_vec()),
        ] {
            let changed = DecodedPage {
                page_id: 512,
                sequence: 3,
                objects: vec![object],
            };
            assert!(reference
                .verify(archive, mode, reference.physical_len, &changed)
                .is_err());
        }
        let duplicate = DecodedPage {
            page_id: 512,
            sequence: 3,
            objects: vec![original.clone(), original],
        };
        assert!(reference
            .verify(archive, mode, reference.physical_len, &duplicate)
            .is_err());
    }
}

/// Object identity remains stable when a packed page is relocated. Its physical
/// extent and location are separately authenticated by the containing TOC entry.
pub(crate) fn object_digest(
    archive: LockboxId,
    mode: FormatMode,
    object: &crate::archive_reader::page::PageObject,
) -> Result<[u8; 32]> {
    let mut hash = Sha256::new();
    hash.update(b"revault-v4-canonical-object-v1\0");
    hash.update(archive.as_bytes());
    hash.update(mode.0.to_le_bytes());
    hash.update([object.kind as u8]);
    hash.update(object.id.to_le_bytes());
    hash.update((object.payload_len() as u64).to_le_bytes());
    object.with_payload(|payload| hash.update(payload))?;
    Ok(hash.finalize().into())
}

#[derive(Clone, Copy)]
pub(crate) struct ObjectReference {
    pub(crate) physical_len: u64,
    pub(crate) object_id: u64,
    pub(crate) digest: [u8; 32],
}

impl ObjectReference {
    pub(crate) fn verify(
        self,
        archive: LockboxId,
        mode: FormatMode,
        extent: u64,
        page: &DecodedPage,
    ) -> Result<()> {
        if extent != self.physical_len {
            return Err(Error::CorruptRecord);
        }
        let mut objects = page
            .objects
            .iter()
            .filter(|object| object.id == self.object_id);
        let object = objects.next().ok_or(Error::CorruptRecord)?;
        if objects.next().is_some() || object_digest(archive, mode, object)? != self.digest {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }
}
