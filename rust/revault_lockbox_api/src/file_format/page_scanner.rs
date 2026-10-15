use crate::commit_root::{decode_commit_root, CommitRoot};
use crate::crypto::derive_page_content_key;
use crate::lockbox_id::LockboxId;
use crate::page::{
    decode_page_with_format, decode_single_object_page_secure_with_format, page_decode_slice,
    physical_page_size_from_page_slice, PageObjectKind, PAGE_MAGIC,
};
#[cfg(test)]
use crate::scan::Scan;
use crate::secret_vec::SecureVec;
use crate::{Error, Result};
use zeroize::Zeroize;

pub(crate) struct PageScanner<'a> {
    pub(crate) format_mode: crate::creation_options::FormatMode,
    bytes: &'a [u8],
    lockbox_id: LockboxId,
    key: &'a [u8],
}

impl<'a> PageScanner<'a> {
    pub(crate) fn new(bytes: &'a [u8], lockbox_id: LockboxId, key: &'a [u8]) -> Self {
        Self {
            format_mode: crate::file_format::current_header::read_header_for_scan(bytes)
                .map(|header| header.format_mode)
                .unwrap_or_default(),
            bytes,
            lockbox_id,
            key,
        }
    }

    #[cfg(test)]
    pub(crate) fn scan_records(&self) -> Scan {
        crate::page::scan_page_records(self.bytes, self.lockbox_id, self.key)
    }

    #[cfg(any(test, feature = "native-block-layout"))]
    pub(crate) fn read_native_chunk(
        &self,
        total_len: u64,
        chunk: &crate::file_chunk::FileChunk,
    ) -> Result<Vec<u8>> {
        crate::file_format::indexed_frame::block_page::read_recovery_chunk(
            self.bytes,
            self.lockbox_id,
            self.format_mode,
            total_len,
            chunk,
            self.key,
        )
    }

    pub(crate) fn commit_root_at(&self, offset: u64) -> Result<CommitRoot> {
        let payload = self.commit_root_payload_at(offset)?;
        decode_commit_root(&payload)
    }

    fn control_payload_at(&self, offset: u64, kind: PageObjectKind) -> Result<Vec<u8>> {
        let mut bytes = std::borrow::Cow::Borrowed(self.page_at(offset).unwrap_or(&[]));
        {
            let start = usize::try_from(offset).map_err(|_| Error::CorruptRecord)?;
            let source = self.bytes.get(start..).ok_or(Error::Truncated)?;
            if let Some(header) = crate::page::recover_physical_length_header(source) {
                let stored = crate::checked::read_u32_le(&header[44..48])? as usize;
                let len = crate::page::PAGE_HEADER_LEN
                    .checked_add(stored)
                    .ok_or(Error::CorruptRecord)?;
                if len > crate::page::DEFAULT_METADATA_PAGE_BYTES {
                    return Err(Error::CorruptRecord);
                }
                let mut candidate = source.get(..len).ok_or(Error::Truncated)?.to_vec();
                candidate[..crate::page::PAGE_HEADER_LEN].copy_from_slice(&header);
                bytes = std::borrow::Cow::Owned(candidate);
            }
        }
        let page = decode_page_with_format(&bytes, self.lockbox_id, self.key, self.format_mode)?;
        crate::page::validate_control_model(&page, offset, kind)?;
        let object = &page.objects[0];
        if physical_page_size_from_page_slice(&bytes)? as u64
            != crate::page::canonical_control_page_len(
                object.payload_len(),
                kind,
                self.format_mode,
            )?
        {
            return Err(Error::CorruptRecord);
        }
        object.with_payload(|payload| payload.to_vec())
    }

    pub(crate) fn commit_root_payload_at(&self, offset: u64) -> Result<Vec<u8>> {
        self.control_payload_at(offset, PageObjectKind::CommitRoot)
    }

    pub(crate) fn commit_auth_payload_at(&self, offset: u64) -> Result<Vec<u8>> {
        self.control_payload_at(offset, PageObjectKind::CommitAuth)
    }

    pub(crate) fn toc_node_payload_at(
        &self,
        offset: u64,
        commitment: crate::file_format::page_commitment::PageCommitment,
    ) -> Result<Vec<u8>> {
        if self.bytes.get(checked_range(offset, 8)?) != Some(PAGE_MAGIC.as_slice()) {
            return Err(Error::CorruptRecord);
        }
        let page = self.page_at(offset).ok_or(Error::Truncated)?;
        let decoded = decode_page_with_format(page, self.lockbox_id, self.key, self.format_mode)?;
        commitment.verify(
            self.lockbox_id,
            self.format_mode,
            physical_page_size_from_page_slice(page)? as u64,
            &decoded,
        )?;
        let Some(toc_object) = decoded.objects.iter().find(|object| {
            matches!(
                object.kind,
                PageObjectKind::TocLeaf | PageObjectKind::TocInternal
            )
        }) else {
            return Err(Error::CorruptRecord);
        };
        toc_object.with_payload(|payload| payload.to_vec())
    }

    pub(crate) fn authenticated_header(
        &self,
    ) -> Result<crate::file_format::current_header::LockboxHeader> {
        let mut candidate = self
            .bytes
            .get(..crate::constants::HEADER_LEN)
            .ok_or(Error::CorruptHeader)?
            .to_vec();
        if crate::file_format::read_header(&candidate).is_err() {
            candidate = crate::file_format::current_header::recover_header_magic(&candidate)
                .ok_or(Error::CorruptHeader)?
                .to_vec();
        }
        let mut header = crate::file_format::read_header(&candidate)?;
        if header.commit_root_offset == 0 && header.commit_auth_offset != 0 {
            let auth = crate::commit_auth::decode_commit_auth(
                &self.commit_auth_payload_at(header.commit_auth_offset)?,
            )?;
            candidate = crate::file_format::current_header::recover_header_root_pointer(
                &candidate,
                auth.commit_root_offset,
            )
            .ok_or(Error::CorruptHeader)?
            .to_vec();
            header = crate::file_format::read_header(&candidate)?;
        }
        let start = header.slot_index * crate::file_format::header_v2::SLOT_LEN;
        let message = candidate
            .get(start..start + 128)
            .ok_or(Error::CorruptHeader)?;
        if crate::crypto::metadata_auth_tag(self.key, message) != header.metadata_auth_tag
            || header.lockbox_id != self.lockbox_id
            || header.format_mode != self.format_mode
        {
            return Err(Error::CorruptHeader);
        }
        Ok(header)
    }

    pub(crate) fn authenticated_secure_object_payload_at(
        &self,
        offset: u64,
        expected: &[PageObjectKind],
        commitment: crate::file_format::page_commitment::PageCommitment,
    ) -> Result<SecureVec> {
        self.secure_object_payload_at_inner(offset, expected, Some(commitment))
    }

    fn secure_object_payload_at_inner(
        &self,
        offset: u64,
        expected: &[PageObjectKind],
        commitment: Option<crate::file_format::page_commitment::PageCommitment>,
    ) -> Result<SecureVec> {
        if self.bytes.get(checked_range(offset, 8)?) != Some(PAGE_MAGIC.as_slice()) {
            return Err(Error::CorruptRecord);
        }
        let page = self.page_at(offset).ok_or(Error::Truncated)?;
        let mut secure_page = SecureVec::try_from_slice(page)?;
        let mut content_key = derive_page_content_key(self.key);
        let decoded = decode_single_object_page_secure_with_format(
            &mut secure_page,
            self.lockbox_id,
            &content_key,
            self.format_mode,
        );
        content_key.zeroize();
        let decoded = decoded?;
        if let Some(commitment) = commitment {
            commitment.verify(
                self.lockbox_id,
                self.format_mode,
                physical_page_size_from_page_slice(page)? as u64,
                &decoded,
            )?;
        }
        let Some(object) = decoded
            .objects
            .into_iter()
            .find(|object| expected.contains(&object.kind))
        else {
            return Err(Error::CorruptRecord);
        };
        Ok(object
            .secure_payload()
            .ok_or(Error::CorruptRecord)?
            .try_clone()?)
    }

    pub(crate) fn file_segment_payload_at(
        &self,
        offset: u64,
        object_id: u64,
        expected_extent: u64,
    ) -> Result<Vec<u8>> {
        let bytes = self.page_at(offset).ok_or(Error::Truncated)?;
        let page = decode_page_with_format(bytes, self.lockbox_id, self.key, self.format_mode)?;
        if physical_page_size_from_page_slice(bytes)? as u64 != expected_extent {
            return Err(Error::CorruptRecord);
        }
        let mut objects = page.objects.iter().filter(|object| object.id == object_id);
        let object = objects.next().ok_or(Error::CorruptRecord)?;
        if objects.next().is_some() || object.kind != PageObjectKind::FileData {
            return Err(Error::CorruptRecord);
        }
        object.with_payload(|payload| payload.to_vec())
    }

    pub(crate) fn authenticated_entry_object_payload(
        &self,
        entry: &crate::toc_entry::TocEntry,
    ) -> Result<Vec<u8>> {
        let bytes = self.page_at(entry.record_offset).ok_or(Error::Truncated)?;
        let page = decode_page_with_format(bytes, self.lockbox_id, self.key, self.format_mode)?;
        crate::file_format::page_commitment::ObjectReference {
            physical_len: entry.record_len,
            object_id: entry.record_object_id,
            digest: entry.record_digest,
        }
        .verify(
            self.lockbox_id,
            self.format_mode,
            physical_page_size_from_page_slice(bytes)? as u64,
            &page,
        )?;
        let object = page
            .objects
            .iter()
            .find(|object| object.id == entry.record_object_id)
            .ok_or(Error::CorruptRecord)?;
        if object.kind != PageObjectKind::Symlink {
            return Err(Error::CorruptRecord);
        }
        object.with_payload(|payload| payload.to_vec())
    }

    fn page_at(&self, offset: u64) -> Option<&'a [u8]> {
        page_decode_slice(self.bytes, usize::try_from(offset).ok()?)
    }
}

fn checked_range(offset: u64, len: usize) -> Result<std::ops::Range<usize>> {
    let start = usize::try_from(offset).map_err(|_| Error::CorruptRecord)?;
    let end = start.checked_add(len).ok_or(Error::CorruptRecord)?;
    Ok(start..end)
}
