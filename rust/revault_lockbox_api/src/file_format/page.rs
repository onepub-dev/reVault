use crate::checked::{array_12, read_u16_le, read_u32_le, read_u64_le};
use crate::compression::{decode_page_body_view, COMPRESSION_NONE};
use crate::crypto::{
    derive_page_content_key, open_with_content_key_secure, open_with_nonce,
    seal_in_place_with_random_nonce, seal_with_content_key_secure, seal_with_random_nonce,
    strong_checksum,
};
use crate::lockbox_id::LockboxId;
use crate::page_buffer::{zeroize_bytes, PageBuffer, SensitiveBytes, ZeroizingBytes};
use crate::page_inspection::{PageInspection, PageObjectInspection};
#[cfg(test)]
use crate::record::{DecodedRecord, RecordHeader, RecordKind};
#[cfg(test)]
use crate::scan::Scan;
use crate::secret_vec::{secure_read_access, SecureVec};
use crate::{Error, Result};
use std::borrow::Cow;
use zeroize::Zeroize;

pub(crate) const PAGE_MAGIC: &[u8; 8] = b"LBX1PAG\0";
pub(crate) const PAGE_HEADER_LEN: usize = 96;
#[cfg(test)]
pub(crate) mod secure_storage;
pub(crate) use crate::constants::{
    DEFAULT_DATA_PAGE_BYTES, DEFAULT_METADATA_PAGE_BYTES, DEFAULT_PAGE_BYTES,
};

const PAGE_SIZE_GRANULARITY: usize = 1024;
const MIN_PAGE_BYTES: usize = PAGE_SIZE_GRANULARITY;
const PAGE_VERSION: u16 = 2;
const PAGE_BODY_VERSION: u8 = 1;
const COMPRESSION_NORMAL: u8 = 1;
const PAGE_FLAG_CLEAR_TEXT: u16 = 0x0001;
const PAGE_FLAG_UNPADDED: u16 = 0x0002;
const PAGE_UNCOMPRESSED_BODY_OVERHEAD: usize = 16 + 17 + 32;
const PAGE_CHECKSUM_START: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PageObjectKind {
    CommitRoot = 1,
    TocLeaf = 2,
    TocInternal = 3,
    FileData = 4,
    PackedFileData = 5,
    Symlink = 6,
    VariableSet = 7,
    VariableDelete = 8,
    KeyDirectory = 9,
    FreeIndexLeaf = 10,
    FreeIndexInternal = 11,
    Delete = 12,
    VariableLeaf = 13,
    VariableInternal = 14,
    FormLeaf = 15,
    FormInternal = 16,
    CommitAuth = 17,
    RedactionManifest = 18,
}

pub(crate) fn page_size_for_objects(objects: &[PageObject]) -> usize {
    let max_page_size = max_page_size_for_objects(objects);
    let object_stream_len = encoded_object_stream_len(objects).unwrap_or(max_page_size);
    page_size_for_object_stream_len(object_stream_len, max_page_size).unwrap_or(max_page_size)
}

pub(crate) fn page_size_for_encoded_objects_with_format(
    objects: &[PageObject],
    mode: crate::creation_options::FormatMode,
) -> Result<usize> {
    if mode.0 != 0 {
        // Explicit raw bodies have a fixed header and object framing. Sizing
        // must not copy (and later wipe) whole file frames or read secret
        // payload bytes merely to count them. Compressed metadata still needs
        // its encoder to determine the stored length.
        let body_len = if !objects_allow_body_compression(objects, mode)
            || mode.options().compression == crate::Compression::None
        {
            16usize
                .checked_add(encoded_object_stream_len(objects)?)
                .ok_or_else(|| Error::SecurityLimitExceeded("page body is too large".into()))?
        } else {
            let body = encode_objects_body(objects, mode)?;
            body.len()
        };
        let protection_len = if mode.plaintext() || page_objects_are_clear_text(objects)? {
            32
        } else {
            16
        };
        let len = body_len
            .checked_add(PAGE_HEADER_LEN + protection_len)
            .ok_or_else(|| Error::SecurityLimitExceeded("page body is too large".into()))?;
        return if mode.unpadded() {
            checked_unpadded_len(len, max_page_size_for_objects(objects))
        } else {
            page_size_for_stored_len(len, max_page_size_for_objects(objects))
        };
    }
    if objects.iter().any(|object| {
        matches!(
            object.kind,
            PageObjectKind::FileData | PageObjectKind::PackedFileData
        )
    }) {
        return Ok(page_size_for_objects(objects));
    }
    let max_page_size = max_page_size_for_objects(objects);
    let sensitive = !page_objects_are_clear_text(objects)?;
    let object_stream = encode_object_stream_sensitive(objects, sensitive)?;
    page_size_for_object_stream_len(object_stream.len(), max_page_size)?;
    let compress_body = objects_allow_body_compression(objects, mode);
    let body = encode_page_body_plaintext(&object_stream, compress_body, sensitive)?;
    let stored_body_len = if page_objects_are_clear_text(objects)? {
        body.len()
            .checked_add(32)
            .ok_or_else(|| Error::SecurityLimitExceeded("page body is too large".to_string()))?
    } else {
        body.len()
            .checked_add(16)
            .ok_or_else(|| Error::SecurityLimitExceeded("page body is too large".to_string()))?
    };
    let stored_len = PAGE_HEADER_LEN
        .checked_add(stored_body_len)
        .ok_or_else(|| Error::SecurityLimitExceeded("page body is too large".to_string()))?;
    page_size_for_stored_len(stored_len, max_page_size)
}

fn max_page_size_for_objects(objects: &[PageObject]) -> usize {
    if objects.iter().any(|object| {
        matches!(
            object.kind,
            PageObjectKind::FileData | PageObjectKind::PackedFileData
        )
    }) {
        DEFAULT_DATA_PAGE_BYTES
    } else {
        DEFAULT_METADATA_PAGE_BYTES
    }
}

pub(crate) fn page_size_for_object_stream_len(
    object_stream_len: usize,
    max_page_size: usize,
) -> Result<usize> {
    let needed = PAGE_HEADER_LEN
        .checked_add(PAGE_UNCOMPRESSED_BODY_OVERHEAD)
        .and_then(|len| len.checked_add(object_stream_len))
        .ok_or_else(|| Error::SecurityLimitExceeded("page is too large".to_string()))?;
    page_size_for_stored_len(needed, max_page_size)
}

pub(crate) fn page_size_for_stored_len(stored_len: usize, max_page_size: usize) -> Result<usize> {
    let rounded = stored_len
        .checked_add(PAGE_SIZE_GRANULARITY - 1)
        .ok_or_else(|| Error::SecurityLimitExceeded("page is too large".to_string()))?
        / PAGE_SIZE_GRANULARITY
        * PAGE_SIZE_GRANULARITY;
    let page_size = rounded.max(MIN_PAGE_BYTES);
    if page_size > max_page_size {
        return Err(Error::SecurityLimitExceeded(
            "page body exceeds maximum page size".to_string(),
        ));
    }
    Ok(page_size)
}

pub(crate) fn is_canonical_control_kind(kind: PageObjectKind) -> bool {
    matches!(
        kind,
        PageObjectKind::CommitRoot | PageObjectKind::CommitAuth | PageObjectKind::KeyDirectory
    )
}

/// Fixed raw framing, with no compression or encoder-version dependency.
pub(crate) fn canonical_control_page_len(
    payload_len: usize,
    kind: PageObjectKind,
    mode: crate::creation_options::FormatMode,
) -> Result<u64> {
    if !is_canonical_control_kind(kind) {
        return Err(Error::CorruptRecord);
    }
    Ok(canonical_control_page_len_from_flags(
        payload_len,
        mode.unpadded(),
        mode.plaintext() || kind == PageObjectKind::KeyDirectory,
    )? as u64)
}

fn canonical_control_page_len_from_flags(
    payload_len: usize,
    unpadded: bool,
    clear: bool,
) -> Result<usize> {
    let len = payload_len
        .checked_add(PAGE_HEADER_LEN + 16 + 4 + 20 + if clear { 32 } else { 16 })
        .ok_or(Error::CorruptRecord)?;
    if unpadded {
        checked_unpadded_len(len, DEFAULT_METADATA_PAGE_BYTES)
    } else {
        page_size_for_stored_len(len, DEFAULT_METADATA_PAGE_BYTES)
    }
}

pub(crate) fn validate_control_model(
    page: &DecodedPage,
    offset: u64,
    kind: PageObjectKind,
) -> Result<()> {
    let [object] = page.objects.as_slice() else {
        return Err(Error::CorruptRecord);
    };
    if object.kind != kind || page.page_id != offset || object.id != page.sequence {
        return Err(Error::CorruptRecord);
    }
    let sequence = object.with_payload(|payload| match kind {
        PageObjectKind::CommitRoot => {
            crate::commit_root::decode_commit_root(payload).map(|root| root.sequence)
        }
        PageObjectKind::CommitAuth => {
            crate::commit_auth::decode_commit_auth(payload).map(|auth| auth.sequence)
        }
        _ => Err(Error::CorruptRecord),
    })??;
    if page.sequence != sequence {
        return Err(Error::CorruptRecord);
    }
    Ok(())
}

/// Recover only a damaged physical-length field using the retained header
/// checksum. The search is bounded by the format maximum and surviving bytes;
/// callers must still authenticate the page body and its owning publication.
pub(crate) fn recover_physical_length_header(bytes: &[u8]) -> Option<[u8; PAGE_HEADER_LEN]> {
    let mut header: [u8; PAGE_HEADER_LEN] = bytes.get(..PAGE_HEADER_LEN)?.try_into().ok()?;
    if &header[..8] != PAGE_MAGIC
        || read_u16_le(&header[8..10]).ok()? != PAGE_VERSION
        || read_u32_le(&header[12..16]).ok()? as usize != PAGE_HEADER_LEN
        || header[PAGE_CHECKSUM_START..] == strong_checksum(&header[..PAGE_CHECKSUM_START])
    {
        return None;
    }
    let stored_len = PAGE_HEADER_LEN.checked_add(read_u32_le(&header[44..48]).ok()? as usize)?;
    let flags = read_u16_le(&header[10..12]).ok()?;
    if flags & !(PAGE_FLAG_CLEAR_TEXT | PAGE_FLAG_UNPADDED) != 0 {
        return None;
    }
    let maximum = bytes.len().min(DEFAULT_DATA_PAGE_BYTES);
    let (first, step, last) = if flags & PAGE_FLAG_UNPADDED != 0 {
        (stored_len, 1, stored_len.min(maximum))
    } else {
        (
            page_size_for_stored_len(stored_len, maximum).ok()?,
            PAGE_SIZE_GRANULARITY,
            maximum,
        )
    };
    for physical_len in (first..=last).step_by(step) {
        header[48..56].copy_from_slice(&(physical_len as u64).to_le_bytes());
        if header[PAGE_CHECKSUM_START..] == strong_checksum(&header[..PAGE_CHECKSUM_START]) {
            physical_page_size_from_page_slice(&header).ok()?;
            return Some(header);
        }
    }
    None
}

pub(crate) fn physical_page_size_from_page_slice(page: &[u8]) -> Result<usize> {
    if page.len() < PAGE_HEADER_LEN || page.get(0..8) != Some(PAGE_MAGIC.as_slice()) {
        return Err(Error::CorruptRecord);
    }
    let header_len = read_u32_le(&page[12..16])? as usize;
    let stored_body_len = read_u32_le(&page[44..48])? as usize;
    let stored_len = header_len
        .checked_add(stored_body_len)
        .ok_or(Error::CorruptRecord)?;
    let physical_len = read_u64_le(&page[48..56])?;
    let flags = read_u16_le(&page[10..12])?;
    let unpadded = flags & PAGE_FLAG_UNPADDED != 0;
    if physical_len < stored_len as u64
        || physical_len > DEFAULT_DATA_PAGE_BYTES as u64
        || physical_len == 0
        || flags & !(PAGE_FLAG_CLEAR_TEXT | PAGE_FLAG_UNPADDED) != 0
        || if unpadded {
            physical_len != stored_len as u64
        } else {
            physical_len % PAGE_SIZE_GRANULARITY as u64 != 0
        }
    {
        return Err(Error::CorruptRecord);
    }
    Ok(physical_len as usize)
}

fn checked_unpadded_len(len: usize, maximum: usize) -> Result<usize> {
    if len > maximum {
        return Err(Error::SecurityLimitExceeded(
            "page body exceeds maximum page size".into(),
        ));
    }
    Ok(len)
}

/// Calculate a private tree page extent without exposing its secret payload.
pub(crate) fn secure_page_size(
    payload_len: usize,
    mode: crate::creation_options::FormatMode,
) -> Result<usize> {
    if !mode.unpadded() {
        return Ok(DEFAULT_METADATA_PAGE_BYTES);
    }
    // Object count + single object header + uncompressed body header + protection.
    let overhead = PAGE_HEADER_LEN + 4 + 20 + if mode.plaintext() { 16 + 32 } else { 33 + 16 };
    let len = payload_len
        .checked_add(overhead)
        .ok_or(Error::CorruptRecord)?;
    checked_unpadded_len(len, DEFAULT_METADATA_PAGE_BYTES)
}

fn validate_padding_flag(flags: u16, mode: crate::creation_options::FormatMode) -> Result<()> {
    if flags & !(PAGE_FLAG_CLEAR_TEXT | PAGE_FLAG_UNPADDED) != 0
        || (flags & PAGE_FLAG_UNPADDED != 0) != mode.unpadded()
    {
        return Err(Error::CorruptRecord);
    }
    Ok(())
}

impl PageObjectKind {
    fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::CommitRoot),
            2 => Ok(Self::TocLeaf),
            3 => Ok(Self::TocInternal),
            4 => Ok(Self::FileData),
            5 => Ok(Self::PackedFileData),
            6 => Ok(Self::Symlink),
            7 => Ok(Self::VariableSet),
            8 => Ok(Self::VariableDelete),
            9 => Ok(Self::KeyDirectory),
            10 => Ok(Self::FreeIndexLeaf),
            11 => Ok(Self::FreeIndexInternal),
            12 => Ok(Self::Delete),
            13 => Ok(Self::VariableLeaf),
            14 => Ok(Self::VariableInternal),
            15 => Ok(Self::FormLeaf),
            16 => Ok(Self::FormInternal),
            17 => Ok(Self::CommitAuth),
            18 => Ok(Self::RedactionManifest),
            _ => Err(Error::CorruptRecord),
        }
    }

    fn is_clear_text_page_object(self) -> bool {
        matches!(self, Self::KeyDirectory)
    }
}

/// Payload ranges are created only after complete page/object validation.
/// Ordinary clones copy the range, so detached objects do not retain a page.
pub(crate) struct OwnedPageRange {
    bytes: std::sync::Arc<SensitiveBytes>,
    range: std::ops::Range<usize>,
}

impl std::fmt::Debug for OwnedPageRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedPageRange")
            .field("len", &self.range.len())
            .finish()
    }
}

enum PageDecodeBytes<'a> {
    Borrowed {
        bytes: &'a [u8],
        range: std::ops::Range<usize>,
    },
    Owned {
        bytes: SensitiveBytes,
        range: std::ops::Range<usize>,
    },
}

impl<'a> PageDecodeBytes<'a> {
    fn borrowed(bytes: &'a [u8]) -> Self {
        Self::Borrowed {
            bytes,
            range: 0..bytes.len(),
        }
    }

    fn owned(bytes: SensitiveBytes) -> Self {
        let len = bytes.len();
        Self::Owned {
            bytes,
            range: 0..len,
        }
    }

    fn as_slice(&self) -> &[u8] {
        match self {
            Self::Borrowed { bytes, range } => &bytes[range.clone()],
            Self::Owned { bytes, range } => &bytes[range.clone()],
        }
    }

    fn narrow(mut self, selected: std::ops::Range<usize>) -> Result<Self> {
        self.as_slice()
            .get(selected.clone())
            .ok_or(Error::CorruptRecord)?;
        let range = match &mut self {
            Self::Borrowed { range, .. } | Self::Owned { range, .. } => range,
        };
        let start = range
            .start
            .checked_add(selected.start)
            .ok_or(Error::CorruptRecord)?;
        let end = range
            .start
            .checked_add(selected.end)
            .ok_or(Error::CorruptRecord)?;
        *range = start..end;
        Ok(self)
    }

    fn materialize(
        self,
        descriptors: Vec<PageObjectDescriptor>,
        share: bool,
    ) -> Result<Vec<PageObject>> {
        if share {
            if let Self::Owned { bytes, range } = self {
                // Match Normal payload's existing full-capacity destruction
                // contract, including unused headers and truncated tail bytes.
                let bytes = std::sync::Arc::new(SensitiveBytes::new(bytes.into_vec(), true));
                return descriptors
                    .into_iter()
                    .map(|object| {
                        let start = range
                            .start
                            .checked_add(object.payload.start)
                            .ok_or(Error::CorruptRecord)?;
                        let end = range
                            .start
                            .checked_add(object.payload.end)
                            .ok_or(Error::CorruptRecord)?;
                        bytes.get(start..end).ok_or(Error::CorruptRecord)?;
                        Ok(PageObject {
                            kind: object.kind,
                            id: object.id,
                            payload: PagePayload::OwnedRange(OwnedPageRange {
                                bytes: std::sync::Arc::clone(&bytes),
                                range: start..end,
                            }),
                        })
                    })
                    .collect();
            }
        }
        descriptors
            .into_iter()
            .map(|object| {
                let payload = self
                    .as_slice()
                    .get(object.payload)
                    .ok_or(Error::CorruptRecord)?;
                Ok(PageObject::new(object.kind, object.id, payload.to_vec()))
            })
            .collect()
    }
}

struct PageObjectDescriptor {
    kind: PageObjectKind,
    id: u64,
    payload: std::ops::Range<usize>,
}

#[derive(Debug)]
pub(crate) enum PagePayload {
    Normal(Vec<u8>),
    Secure(SecureVec),
    OwnedRange(OwnedPageRange),
}

impl PagePayload {
    pub(crate) fn normal(payload: Vec<u8>) -> Self {
        Self::Normal(payload)
    }

    pub(crate) fn secure(payload: SecureVec) -> Self {
        Self::Secure(payload)
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Normal(payload) => payload.len(),
            Self::OwnedRange(payload) => payload.range.len(),
            Self::Secure(payload) => payload.len(),
        }
    }

    pub(crate) fn with_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Result<R> {
        match self {
            Self::Normal(payload) => Ok(f(payload)),
            Self::OwnedRange(payload) => Ok(f(&payload.bytes[payload.range.clone()])),
            Self::Secure(payload) => {
                secure_read_access(|access| payload.with_bytes_in(access, f)).map_err(Into::into)
            }
        }
    }

    pub(crate) fn try_clone(&self) -> Result<Self> {
        match self {
            Self::Normal(payload) => Ok(Self::Normal(payload.clone())),
            Self::OwnedRange(payload) => {
                Ok(Self::Normal(payload.bytes[payload.range.clone()].to_vec()))
            }
            Self::Secure(payload) => Ok(Self::Secure(payload.try_clone()?)),
        }
    }

    pub(crate) fn as_secure(&self) -> Option<&SecureVec> {
        match self {
            Self::Normal(_) | Self::OwnedRange(_) => None,
            Self::Secure(payload) => Some(payload),
        }
    }
}

impl Drop for PagePayload {
    fn drop(&mut self) {
        if let Self::Normal(payload) = self {
            zeroize_bytes(payload);
        }
    }
}

impl Clone for PagePayload {
    fn clone(&self) -> Self {
        match self.try_clone() {
            Ok(payload) => payload,
            Err(_) => std::process::abort(),
        }
    }
}

impl PartialEq for PagePayload {
    fn eq(&self, other: &Self) -> bool {
        self.with_bytes(|left| other.with_bytes(|right| left == right))
            .ok()
            .and_then(Result::ok)
            .unwrap_or(false)
    }
}

impl Eq for PagePayload {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PageObject {
    pub(crate) kind: PageObjectKind,
    pub(crate) id: u64,
    pub(crate) payload: PagePayload,
}

impl PageObject {
    pub(crate) fn new(kind: PageObjectKind, id: u64, payload: Vec<u8>) -> Self {
        Self {
            kind,
            id,
            payload: PagePayload::normal(payload),
        }
    }

    pub(crate) fn new_secure(kind: PageObjectKind, id: u64, payload: SecureVec) -> Self {
        Self {
            kind,
            id,
            payload: PagePayload::secure(payload),
        }
    }

    pub(crate) fn payload_len(&self) -> usize {
        self.payload.len()
    }

    pub(crate) fn with_payload<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Result<R> {
        self.payload.with_bytes(f)
    }

    pub(crate) fn secure_payload(&self) -> Option<&SecureVec> {
        self.payload.as_secure()
    }
}

pub(crate) fn encoded_object_stream_len(objects: &[PageObject]) -> Result<usize> {
    let mut len = 4usize;
    for object in objects {
        len = len
            .checked_add(encoded_object_len(object)?)
            .ok_or_else(|| Error::SecurityLimitExceeded("page is too large".to_string()))?;
    }
    Ok(len)
}

pub(crate) fn encoded_object_len(object: &PageObject) -> Result<usize> {
    20usize
        .checked_add(object.payload_len())
        .ok_or_else(|| Error::SecurityLimitExceeded("page object is too large".to_string()))
}

pub(crate) fn uncompressed_objects_fit(page_size: usize, object_stream_len: usize) -> bool {
    page_size_for_object_stream_len(object_stream_len, page_size)
        .is_ok_and(|needed| needed <= page_size)
}

#[derive(Debug, Clone)]
pub(crate) struct DecodedPage {
    pub(crate) page_id: u64,
    pub(crate) sequence: u64,
    pub(crate) objects: Vec<PageObject>,
}

pub(crate) struct SecureSingleObjectPage<'a> {
    pub(crate) format_mode: crate::creation_options::FormatMode,
    pub(crate) page_size: usize,
    pub(crate) lockbox_id: LockboxId,
    pub(crate) page_id: u64,
    pub(crate) sequence: u64,
    pub(crate) content_key: &'a [u8; 32],
    pub(crate) kind: PageObjectKind,
    pub(crate) id: u64,
    pub(crate) payload: &'a SecureVec,
}

// Raw page codecs used by PageCache. Production lockbox read/write paths should
// not call these directly; recovery and low-level format tests are the
// exceptions for raw page codecs in this module.
#[cfg(test)]
pub(crate) fn encode_page(
    page_size: usize,
    lockbox_id: LockboxId,
    page_id: u64,
    sequence: u64,
    key: &[u8],
    objects: &[PageObject],
) -> Result<Vec<u8>> {
    encode_page_with_format(
        page_size,
        lockbox_id,
        page_id,
        sequence,
        key,
        objects,
        Default::default(),
    )
}

fn objects_allow_body_compression(
    objects: &[PageObject],
    mode: crate::creation_options::FormatMode,
) -> bool {
    !objects.iter().any(|object| {
        matches!(
            object.kind,
            PageObjectKind::FileData
                | PageObjectKind::PackedFileData
                | PageObjectKind::CommitRoot
                | PageObjectKind::CommitAuth
                | PageObjectKind::KeyDirectory
        ) || (mode.0 != 0
            && matches!(
                object.kind,
                PageObjectKind::VariableLeaf
                    | PageObjectKind::VariableInternal
                    | PageObjectKind::FormLeaf
                    | PageObjectKind::FormInternal
            ))
    })
}

fn encode_objects_body(
    objects: &[PageObject],
    mode: crate::creation_options::FormatMode,
) -> Result<SensitiveBytes> {
    let sensitive = !mode.plaintext() && !page_objects_are_clear_text(objects)?;
    let object_stream = encode_object_stream_sensitive(objects, sensitive)?;
    let compress_body = objects_allow_body_compression(objects, mode);
    let body = if mode.0 == 0
        || !compress_body
        || mode.options().compression == crate::Compression::None
    {
        encode_page_body_plaintext(
            &object_stream,
            compress_body
                && (mode.0 == 0 || mode.options().compression != crate::Compression::None),
            sensitive,
        )?
    } else {
        let (algorithm, stored) = crate::compression::encode_with_compression_sensitive(
            &object_stream,
            mode.options().compression,
            sensitive,
        );
        let stored = SensitiveBytes::new(stored, sensitive);
        let capacity = 33usize
            .checked_add(stored.len())
            .ok_or_else(|| Error::SecurityLimitExceeded("page body is too large".into()))?;
        let mut body = SensitiveBytes::new(Vec::with_capacity(capacity), sensitive);
        body.extend_from_slice(&[PAGE_BODY_VERSION, COMPRESSION_NORMAL, 0, 0]);
        body.extend_from_slice(&(object_stream.len() as u64).to_le_bytes());
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend_from_slice(&(object_stream.len() as u64).to_le_bytes());
        body.push(algorithm);
        body.extend_from_slice(&(stored.len() as u64).to_le_bytes());
        body.extend_from_slice(&stored);
        body
    };
    Ok(body)
}

pub(crate) fn encode_page_with_format(
    page_size: usize,
    lockbox_id: LockboxId,
    page_id: u64,
    sequence: u64,
    key: &[u8],
    objects: &[PageObject],
    mode: crate::creation_options::FormatMode,
) -> Result<Vec<u8>> {
    if page_size < PAGE_HEADER_LEN {
        return Err(Error::CorruptRecord);
    }
    let clear_text = mode.plaintext() || page_objects_are_clear_text(objects)?;
    let flags = if clear_text { PAGE_FLAG_CLEAR_TEXT } else { 0 }
        | if mode.unpadded() {
            PAGE_FLAG_UNPADDED
        } else {
            0
        };
    if clear_text
        && (!objects_allow_body_compression(objects, mode)
            || (mode.0 != 0 && mode.options().compression == crate::Compression::None))
    {
        return encode_clear_raw_page(page_size, page_id, sequence, flags, objects);
    }
    if !clear_text
        && (!objects_allow_body_compression(objects, mode)
            || (mode.0 != 0 && mode.options().compression == crate::Compression::None))
    {
        return encode_encrypted_raw_page(
            page_size, lockbox_id, page_id, sequence, flags, key, objects,
        );
    }
    let body = encode_objects_body(objects, mode)?;
    let (nonce, mut stored_body) = if clear_text {
        let mut stored = Vec::with_capacity(32 + body.len());
        stored.extend_from_slice(&strong_checksum(&body));
        stored.extend_from_slice(&body);
        ([0; 12], stored)
    } else {
        let encrypted_len = body
            .len()
            .checked_add(16)
            .ok_or_else(|| Error::SecurityLimitExceeded("page body is too large".to_string()))?;
        let encrypted_len = u32::try_from(encrypted_len)
            .map_err(|_| Error::SecurityLimitExceeded("page body is too large".to_string()))?;
        let aad = page_aad(
            lockbox_id,
            page_id,
            sequence,
            flags,
            encrypted_len,
            page_size as u64,
        );
        seal_with_random_nonce(&body, key, &aad)?
    };
    let stored_body_len = u32::try_from(stored_body.len())
        .map_err(|_| Error::SecurityLimitExceeded("page body is too large".to_string()))?;
    if PAGE_HEADER_LEN + stored_body.len() > page_size {
        return Err(Error::SecurityLimitExceeded(
            "page body exceeds physical page size".to_string(),
        ));
    }
    if mode.unpadded() && PAGE_HEADER_LEN + stored_body.len() != page_size {
        return Err(Error::CorruptRecord);
    }

    let mut page = vec![0; page_size];
    write_page_header(&mut page, page_id, sequence, flags, &nonce, stored_body_len);
    page[PAGE_HEADER_LEN..PAGE_HEADER_LEN + stored_body.len()].copy_from_slice(&stored_body);
    stored_body.zeroize();
    Ok(page)
}

fn write_page_header(
    page: &mut [u8],
    page_id: u64,
    sequence: u64,
    flags: u16,
    nonce: &[u8; 12],
    stored_body_len: u32,
) {
    page[0..8].copy_from_slice(PAGE_MAGIC);
    page[8..10].copy_from_slice(&PAGE_VERSION.to_le_bytes());
    page[10..12].copy_from_slice(&flags.to_le_bytes());
    page[12..16].copy_from_slice(&(PAGE_HEADER_LEN as u32).to_le_bytes());
    page[16..24].copy_from_slice(&page_id.to_le_bytes());
    page[24..32].copy_from_slice(&sequence.to_le_bytes());
    page[32..44].copy_from_slice(nonce);
    page[44..48].copy_from_slice(&stored_body_len.to_le_bytes());
    let physical_len = page.len() as u64;
    page[48..56].copy_from_slice(&physical_len.to_le_bytes());
    let header_digest = strong_checksum(&page[0..PAGE_CHECKSUM_START]);
    page[PAGE_CHECKSUM_START..PAGE_HEADER_LEN].copy_from_slice(&header_digest);
}

fn encode_encrypted_raw_page(
    page_size: usize,
    lockbox_id: LockboxId,
    page_id: u64,
    sequence: u64,
    flags: u16,
    key: &[u8],
    objects: &[PageObject],
) -> Result<Vec<u8>> {
    let stream_len = encoded_object_stream_len(objects)?;
    let stored_len = stream_len
        .checked_add(16 + 16)
        .ok_or_else(|| Error::SecurityLimitExceeded("page body is too large".to_string()))?;
    let stored_body_len = u32::try_from(stored_len)
        .map_err(|_| Error::SecurityLimitExceeded("page body is too large".to_string()))?;
    let used = PAGE_HEADER_LEN.checked_add(stored_len).ok_or_else(|| {
        Error::SecurityLimitExceeded("page body exceeds physical page size".to_string())
    })?;
    if used > page_size {
        return Err(Error::SecurityLimitExceeded(
            "page body exceeds physical page size".to_string(),
        ));
    }
    if flags & PAGE_FLAG_UNPADDED != 0 && used != page_size {
        return Err(Error::CorruptRecord);
    }
    let mut page = ZeroizingBytes::new(Vec::with_capacity(page_size));
    page.resize(PAGE_HEADER_LEN, 0);
    page.extend_from_slice(&[PAGE_BODY_VERSION, COMPRESSION_NONE, 0, 0]);
    page.extend_from_slice(&(stream_len as u64).to_le_bytes());
    page.extend_from_slice(&0u32.to_le_bytes());
    append_object_stream(&mut page, objects)?;
    if page.len() != used - 16 {
        return Err(Error::CorruptRecord);
    }
    let aad = page_aad(
        lockbox_id,
        page_id,
        sequence,
        flags,
        stored_body_len,
        page_size as u64,
    );
    let (nonce, tag) = seal_in_place_with_random_nonce(&mut page[PAGE_HEADER_LEN..], key, &aad)?;
    page.extend_from_slice(&tag);
    page.resize(page_size, 0);
    write_page_header(&mut page, page_id, sequence, flags, &nonce, stored_body_len);
    Ok(std::mem::take(&mut *page))
}

fn encode_clear_raw_page(
    page_size: usize,
    page_id: u64,
    sequence: u64,
    flags: u16,
    objects: &[PageObject],
) -> Result<Vec<u8>> {
    let stream_len = encoded_object_stream_len(objects)?;
    let stored_len = stream_len
        .checked_add(32 + 16)
        .ok_or_else(|| Error::SecurityLimitExceeded("page body is too large".to_string()))?;
    let stored_body_len = u32::try_from(stored_len)
        .map_err(|_| Error::SecurityLimitExceeded("page body is too large".to_string()))?;
    let used = PAGE_HEADER_LEN.checked_add(stored_len).ok_or_else(|| {
        Error::SecurityLimitExceeded("page body exceeds physical page size".to_string())
    })?;
    if used > page_size {
        return Err(Error::SecurityLimitExceeded(
            "page body exceeds physical page size".to_string(),
        ));
    }
    if flags & PAGE_FLAG_UNPADDED != 0 && used != page_size {
        return Err(Error::CorruptRecord);
    }
    let mut page = ZeroizingBytes::new(Vec::with_capacity(page_size));
    page.resize(PAGE_HEADER_LEN + 32, 0);
    page.extend_from_slice(&[PAGE_BODY_VERSION, COMPRESSION_NONE, 0, 0]);
    page.extend_from_slice(&(stream_len as u64).to_le_bytes());
    page.extend_from_slice(&0u32.to_le_bytes());
    append_object_stream(&mut page, objects)?;
    if page.len() != used {
        return Err(Error::CorruptRecord);
    }
    let body_digest = strong_checksum(&page[PAGE_HEADER_LEN + 32..]);
    page[PAGE_HEADER_LEN..PAGE_HEADER_LEN + 32].copy_from_slice(&body_digest);
    page.resize(page_size, 0);
    write_page_header(
        &mut page,
        page_id,
        sequence,
        flags,
        &[0; 12],
        stored_body_len,
    );
    Ok(std::mem::take(&mut *page))
}

pub(crate) fn decode_page(page: &[u8], lockbox_id: LockboxId, key: &[u8]) -> Result<DecodedPage> {
    decode_page_with_format(page, lockbox_id, key, Default::default())
}

pub(crate) fn decode_page_with_format(
    page: &[u8],
    lockbox_id: LockboxId,
    key: &[u8],
    mode: crate::creation_options::FormatMode,
) -> Result<DecodedPage> {
    decode_page_input(PageDecodeBytes::borrowed(page), lockbox_id, key, mode)
}

pub(crate) fn decode_owned_page_with_format(
    page: Vec<u8>,
    lockbox_id: LockboxId,
    key: &[u8],
    mode: crate::creation_options::FormatMode,
) -> Result<DecodedPage> {
    decode_page_input(
        PageDecodeBytes::owned(SensitiveBytes::new(page, false)),
        lockbox_id,
        key,
        mode,
    )
}

fn decode_page_input(
    input: PageDecodeBytes<'_>,
    lockbox_id: LockboxId,
    key: &[u8],
    mode: crate::creation_options::FormatMode,
) -> Result<DecodedPage> {
    let allow_shared = matches!(&input, PageDecodeBytes::Owned { .. });
    let page = input.as_slice();
    if page.len() < PAGE_HEADER_LEN {
        return Err(Error::Truncated);
    }
    if &page[0..8] != PAGE_MAGIC || read_u16_le(&page[8..10])? != PAGE_VERSION {
        return Err(Error::CorruptRecord);
    }
    let flags = read_u16_le(&page[10..12])?;
    if flags & !(PAGE_FLAG_CLEAR_TEXT | PAGE_FLAG_UNPADDED) != 0 {
        return Err(Error::CorruptRecord);
    }
    let physical_len = physical_page_size_from_page_slice(page)?;
    let header_len = read_u32_le(&page[12..16])? as usize;
    if header_len != PAGE_HEADER_LEN || header_len > page.len() {
        return Err(Error::CorruptRecord);
    }
    if page[56..PAGE_CHECKSUM_START].iter().any(|byte| *byte != 0) {
        return Err(Error::CorruptRecord);
    }
    let expected_digest = strong_checksum(&page[0..PAGE_CHECKSUM_START]);
    if page[PAGE_CHECKSUM_START..PAGE_HEADER_LEN] != expected_digest {
        return Err(Error::CorruptRecord);
    }
    let page_id = read_u64_le(&page[16..24])?;
    let sequence = read_u64_le(&page[24..32])?;
    let nonce = &page[32..44];
    let stored_body_len = read_u32_le(&page[44..48])? as usize;
    let stored_end = header_len
        .checked_add(stored_body_len)
        .ok_or(Error::Truncated)?;
    let stored_body = page.get(header_len..stored_end).ok_or(Error::Truncated)?;
    let sensitive = flags & PAGE_FLAG_CLEAR_TEXT == 0;
    let body = if !sensitive {
        if nonce.iter().any(|byte| *byte != 0) || stored_body.len() < 32 {
            return Err(Error::CorruptRecord);
        }
        let (digest, body) = stored_body.split_at(32);
        if digest != strong_checksum(body) {
            return Err(Error::CorruptRecord);
        }
        input.narrow(header_len + 32..stored_end)?
    } else {
        let aad = page_aad(
            lockbox_id,
            page_id,
            sequence,
            flags,
            stored_body_len as u32,
            physical_len as u64,
        );
        let nonce = array_12(nonce)?;
        match input {
            PageDecodeBytes::Owned { bytes, range } => {
                // Before any mutation, the entire allocation becomes sensitive:
                // authentication failures and later parse errors wipe all capacity.
                let mut bytes = SensitiveBytes::new(bytes.into_vec(), true);
                let start = range
                    .start
                    .checked_add(header_len)
                    .ok_or(Error::CorruptRecord)?;
                let end = range
                    .start
                    .checked_add(stored_end)
                    .ok_or(Error::CorruptRecord)?;
                let encrypted = bytes.get_mut(start..end).ok_or(Error::Truncated)?;
                let len = crate::crypto::open_in_place_with_nonce(encrypted, key, &nonce, &aad)?;
                let end = start.checked_add(len).ok_or(Error::CorruptRecord)?;
                PageDecodeBytes::Owned {
                    bytes,
                    range: start..end,
                }
            }
            PageDecodeBytes::Borrowed { bytes, range } => {
                let page = &bytes[range];
                let decrypted = SensitiveBytes::new(
                    open_with_nonce(&page[header_len..stored_end], key, &nonce, &aad)?,
                    true,
                );
                PageDecodeBytes::owned(decrypted)
            }
        }
    };
    let body_bytes = body.as_slice();
    if mode.0 != 0
        && mode.options().compression == crate::Compression::None
        && body_bytes.get(1) == Some(&COMPRESSION_NORMAL)
        && body_bytes.get(24) != Some(&COMPRESSION_NONE)
    {
        return Err(Error::CorruptRecord);
    }
    // Preserve the original canonical-control checks before moving ownership.
    let canonical_body = body_bytes.get(1) == Some(&COMPRESSION_NONE)
        && body_bytes.get(2..4) == Some(&[0, 0][..])
        && body_bytes.get(12..16) == Some(&[0, 0, 0, 0][..]);
    let raw_body = body_bytes.get(1) == Some(&COMPRESSION_NONE);
    let decoded = match decode_page_body_plaintext(body_bytes, sensitive)? {
        Cow::Borrowed(bytes) => {
            // The legacy compression envelope permits trailing body bytes.
            // Preserve the parser's exact borrowed subslice, not a suffix guess.
            let start = (bytes.as_ptr() as usize)
                .checked_sub(body_bytes.as_ptr() as usize)
                .ok_or(Error::CorruptRecord)?;
            let end = start.checked_add(bytes.len()).ok_or(Error::CorruptRecord)?;
            let selected = body_bytes.get(start..end).ok_or(Error::CorruptRecord)?;
            if selected.as_ptr() != bytes.as_ptr() {
                return Err(Error::CorruptRecord);
            }
            body.narrow(start..end)?
        }
        Cow::Owned(bytes) => {
            let bytes = SensitiveBytes::new(bytes, sensitive);
            drop(body);
            PageDecodeBytes::owned(bytes)
        }
    };
    let descriptors = decode_object_descriptors(decoded.as_slice())?;
    if descriptors
        .iter()
        .any(|object| is_canonical_control_kind(object.kind))
    {
        let [object] = descriptors.as_slice() else {
            return Err(Error::CorruptRecord);
        };
        if !canonical_body
            || object.id != sequence
            || physical_len
                != canonical_control_page_len_from_flags(
                    object.payload.len(),
                    flags & PAGE_FLAG_UNPADDED != 0,
                    flags & PAGE_FLAG_CLEAR_TEXT != 0,
                )?
        {
            return Err(Error::CorruptRecord);
        }
    }
    let public_objects = page_kinds_are_clear_text(descriptors.iter().map(|object| object.kind))?;
    if !public_objects {
        validate_padding_flag(flags, mode)?;
    }
    if (mode.plaintext() || public_objects) != (flags & PAGE_FLAG_CLEAR_TEXT != 0) {
        return Err(Error::CorruptRecord);
    }
    // Keep metadata, mixed pages, and whole-page compression on the original
    // independently owned payload representation and cache-accounting behavior.
    let share = allow_shared
        && raw_body
        && !descriptors.is_empty()
        && descriptors
            .iter()
            .all(|object| object.kind == PageObjectKind::FileData);
    let objects = decoded.materialize(descriptors, share)?;
    Ok(DecodedPage {
        page_id,
        sequence,
        objects,
    })
}

pub(crate) fn decode_single_object_page_secure_with_format(
    page: &mut SecureVec,
    lockbox_id: LockboxId,
    content_key: &[u8; 32],
    mode: crate::creation_options::FormatMode,
) -> Result<DecodedPage> {
    let (page_id, sequence) = decrypt_page_body_secure(page, lockbox_id, content_key, mode)?;
    decode_page_body_plaintext_in_place(page)?;
    let (kind, id, payload) = decode_single_object_stream_in_place(page)?;
    let object = PageObject::new_secure(kind, id, payload);
    Ok(DecodedPage {
        page_id,
        sequence,
        objects: vec![object],
    })
}

pub(crate) fn encode_single_object_page_secure(
    request: SecureSingleObjectPage<'_>,
) -> Result<Vec<u8>> {
    encode_single_object_page_secure_using(request, seal_with_content_key_secure)
}
fn encode_single_object_page_secure_using(
    request: SecureSingleObjectPage<'_>,
    seal: impl FnOnce(&mut SecureVec, &[u8; 32], &[u8]) -> Result<[u8; 12]>,
) -> Result<Vec<u8>> {
    if request.page_size < PAGE_HEADER_LEN {
        return Err(Error::SecurityLimitExceeded(
            "page is smaller than the header".to_string(),
        ));
    }
    if request.kind.is_clear_text_page_object() {
        return Err(Error::CorruptRecord);
    }

    let mut body = SecureVec::new();
    body.try_extend_from_slice(&1u32.to_le_bytes())?;
    body.try_extend_from_slice(&[request.kind as u8, 1])?;
    body.try_extend_from_slice(&0u16.to_le_bytes())?;
    body.try_extend_from_slice(&request.id.to_le_bytes())?;
    body.try_extend_from_slice(&(request.payload.len() as u64).to_le_bytes())?;
    body.try_extend_from_secure(request.payload)?;

    let object_stream_len = body.len();
    let mut page_body = SecureVec::new();
    page_body.try_extend_from_slice(&[PAGE_BODY_VERSION, COMPRESSION_NORMAL, 0, 0])?;
    page_body.try_extend_from_slice(&(object_stream_len as u64).to_le_bytes())?;
    page_body.try_extend_from_slice(&0u32.to_le_bytes())?;
    page_body.try_extend_from_slice(&(object_stream_len as u64).to_le_bytes())?;
    page_body.try_extend_from_slice(&[COMPRESSION_NONE])?;
    page_body.try_extend_from_slice(&(object_stream_len as u64).to_le_bytes())?;
    page_body.try_extend_from_secure(&body)?;
    body.zeroize()?;

    let encrypted_len = page_body
        .len()
        .checked_add(16)
        .ok_or_else(|| Error::SecurityLimitExceeded("page body is too large".to_string()))?;
    let encrypted_len = u32::try_from(encrypted_len)
        .map_err(|_| Error::SecurityLimitExceeded("page body is too large".to_string()))?;
    let flags = if request.format_mode.unpadded() {
        PAGE_FLAG_UNPADDED
    } else {
        0
    };
    let aad = page_aad(
        request.lockbox_id,
        request.page_id,
        request.sequence,
        flags,
        encrypted_len,
        request.page_size as u64,
    );
    let nonce = seal(&mut page_body, request.content_key, &aad)?;
    let stored_body_len = u32::try_from(page_body.len())
        .map_err(|_| Error::SecurityLimitExceeded("page body is too large".to_string()))?;
    if PAGE_HEADER_LEN + page_body.len() > request.page_size {
        return Err(Error::SecurityLimitExceeded(
            "page body exceeds physical page size".to_string(),
        ));
    }
    if request.format_mode.unpadded() && PAGE_HEADER_LEN + page_body.len() != request.page_size {
        return Err(Error::CorruptRecord);
    }

    let mut page = vec![0; request.page_size];
    page[0..8].copy_from_slice(PAGE_MAGIC);
    page[8..10].copy_from_slice(&PAGE_VERSION.to_le_bytes());
    page[10..12].copy_from_slice(&flags.to_le_bytes());
    page[12..16].copy_from_slice(&(PAGE_HEADER_LEN as u32).to_le_bytes());
    page[16..24].copy_from_slice(&request.page_id.to_le_bytes());
    page[24..32].copy_from_slice(&request.sequence.to_le_bytes());
    page[32..44].copy_from_slice(&nonce);
    page[44..48].copy_from_slice(&stored_body_len.to_le_bytes());
    page[48..56].copy_from_slice(&(request.page_size as u64).to_le_bytes());
    let header_digest = strong_checksum(&page[0..PAGE_CHECKSUM_START]);
    page[PAGE_CHECKSUM_START..PAGE_HEADER_LEN].copy_from_slice(&header_digest);
    secure_read_access(|access| {
        page_body.with_bytes_in(access, |encrypted| {
            page[PAGE_HEADER_LEN..PAGE_HEADER_LEN + encrypted.len()].copy_from_slice(encrypted);
        })
    })?;
    page_body.zeroize()?;
    Ok(page)
}

fn decrypt_page_body_secure(
    page: &mut SecureVec,
    lockbox_id: LockboxId,
    content_key: &[u8; 32],
    mode: crate::creation_options::FormatMode,
) -> Result<(u64, u64)> {
    let header: (u64, u64, [u8; 12], u16, usize, usize, u64) = {
        let parsed = secure_read_access(|access| {
            page.with_bytes_in(access, |page| {
                if page.len() < PAGE_HEADER_LEN {
                    return Err(Error::Truncated);
                }
                if &page[0..8] != PAGE_MAGIC {
                    return Err(Error::CorruptRecord);
                }
                if read_u16_le(&page[8..10])? != PAGE_VERSION {
                    return Err(Error::CorruptRecord);
                }
                let flags = read_u16_le(&page[10..12])?;
                validate_padding_flag(flags, mode)?;
                let header_len = read_u32_le(&page[12..16])? as usize;
                if header_len != PAGE_HEADER_LEN || header_len > page.len() {
                    return Err(Error::CorruptRecord);
                }
                if page[56..PAGE_CHECKSUM_START].iter().any(|byte| *byte != 0) {
                    return Err(Error::CorruptRecord);
                }
                let expected_digest = strong_checksum(&page[0..PAGE_CHECKSUM_START]);
                if page[PAGE_CHECKSUM_START..PAGE_HEADER_LEN] != expected_digest {
                    return Err(Error::CorruptRecord);
                }
                let stored_body_len = read_u32_le(&page[44..48])? as usize;
                if header_len + stored_body_len > page.len() {
                    return Err(Error::Truncated);
                }
                Ok((
                    read_u64_le(&page[16..24])?,
                    read_u64_le(&page[24..32])?,
                    array_12(&page[32..44])?,
                    flags,
                    header_len,
                    stored_body_len,
                    physical_page_size_from_page_slice(page)? as u64,
                ))
            })
        })?;
        parsed?
    };
    let (page_id, sequence, nonce, flags, header_len, stored_body_len, physical_len) = header;
    if (flags & PAGE_FLAG_CLEAR_TEXT != 0) != mode.plaintext() {
        return Err(Error::CorruptRecord);
    }
    if mode.plaintext() {
        if nonce != [0; 12] || stored_body_len < 32 {
            return Err(Error::CorruptRecord);
        }
        let valid = page.with_bytes(|bytes| {
            let stored = &bytes[header_len..header_len + stored_body_len];
            stored[..32] == strong_checksum(&stored[32..])
        })?;
        if !valid {
            return Err(Error::CorruptRecord);
        }
        page.with_mut_bytes(|bytes| {
            bytes.copy_within(header_len + 32..header_len + stored_body_len, 0)
        })?;
        page.truncate(stored_body_len - 32)?;
        return Ok((page_id, sequence));
    }
    page.with_mut_bytes(|bytes| {
        bytes.copy_within(header_len..header_len + stored_body_len, 0);
    })?;
    page.truncate(stored_body_len)?;
    let aad = page_aad(
        lockbox_id,
        page_id,
        sequence,
        flags,
        stored_body_len as u32,
        physical_len,
    );
    open_with_content_key_secure(page, content_key, &nonce, &aad)?;
    Ok((page_id, sequence))
}

fn decode_page_body_plaintext_in_place<B: PageBuffer>(body: &mut B) -> Result<()> {
    let (stored_offset, stored_len) = {
        let parsed = body.with_bytes(|body| {
            if body.len() >= 16 && body[0] == PAGE_BODY_VERSION && body[1] == COMPRESSION_NONE {
                let len = usize::try_from(read_u64_le(&body[4..12])?)
                    .map_err(|_| Error::CorruptRecord)?;
                if len != body.len() - 16 {
                    return Err(Error::CorruptRecord);
                }
                return Ok((16, len));
            }
            if body.len() < 33 {
                return Err(Error::CorruptRecord);
            }
            if body[0] != PAGE_BODY_VERSION || body[1] != COMPRESSION_NORMAL {
                return Err(Error::CorruptRecord);
            }
            let expected_len = read_u64_le(&body[4..12])? as usize;
            let compression = &body[16..];
            let real_len = read_u64_le(&compression[0..8])? as usize;
            let algorithm = compression[8];
            let stored_len = read_u64_le(&compression[9..17])? as usize;
            if algorithm != COMPRESSION_NONE {
                return Err(Error::CorruptRecord);
            }
            if expected_len != real_len || stored_len != real_len {
                return Err(Error::CorruptRecord);
            }
            let stored_offset = 16usize + 17;
            if stored_offset + stored_len > body.len() {
                return Err(Error::CorruptRecord);
            }
            Ok((stored_offset, stored_len))
        })?;
        parsed?
    };
    body.with_mut_bytes(|bytes| {
        bytes.copy_within(stored_offset..stored_offset + stored_len, 0);
    })?;
    body.truncate(stored_len)?;
    Ok(())
}

fn decode_single_object_stream_in_place<B: PageBuffer>(
    stream: &mut B,
) -> Result<(PageObjectKind, u64, B)> {
    let (kind, id, payload_offset, payload_len) = {
        let parsed = stream.with_bytes(|bytes| {
            if bytes.len() < 24 {
                return Err(Error::CorruptRecord);
            }
            let count = read_u32_le(&bytes[0..4])?;
            if count != 1 {
                return Err(Error::CorruptRecord);
            }
            let kind = PageObjectKind::from_u8(bytes[4])?;
            let version = bytes[5];
            let flags = read_u16_le(&bytes[6..8])?;
            if version != 1 || flags != 0 {
                return Err(Error::CorruptRecord);
            }
            let id = read_u64_le(&bytes[8..16])?;
            let payload_len = read_u64_le(&bytes[16..24])? as usize;
            let payload_offset = 24usize;
            if payload_offset + payload_len != bytes.len() {
                return Err(Error::CorruptRecord);
            }
            Ok((kind, id, payload_offset, payload_len))
        })?;
        parsed?
    };
    stream.with_mut_bytes(|bytes| {
        bytes.copy_within(payload_offset..payload_offset + payload_len, 0);
    })?;
    stream.truncate(payload_len)?;
    let payload = stream.try_clone_range(0, payload_len)?;
    Ok((kind, id, payload))
}

#[cfg(test)]
pub(crate) fn scan_page_records(bytes: &[u8], lockbox_id: LockboxId, key: &[u8]) -> Scan {
    #[cfg(any(test, feature = "native-block-layout"))]
    let mut native_pages = Vec::new();
    let mode = crate::file_format::current_header::read_header_for_scan(bytes)
        .map(|header| header.format_mode)
        .unwrap_or_default();
    let mut records = Vec::new();
    let mut corrupt_records = 0usize;
    let mut content_key = derive_page_content_key(key);
    let mut i = crate::constants::HEADER_LEN;
    while i + PAGE_HEADER_LEN <= bytes.len() {
        if &bytes[i..i + 8] == PAGE_MAGIC {
            #[cfg(any(test, feature = "native-block-layout"))]
            if crate::file_format::indexed_frame::block_page::is_native_header(&bytes[i..]) {
                match crate::file_format::indexed_frame::block_page::scan(
                    bytes, i, lockbox_id, mode, key,
                ) {
                    Ok(page) => {
                        i += page.physical_len;
                        native_pages.push(page);
                    }
                    Err(_) => {
                        corrupt_records += 1;
                        i += 1;
                    }
                }
                continue;
            }
            let Some(page_bytes) = page_decode_slice(bytes, i) else {
                corrupt_records += 1;
                break;
            };
            if decode_secure_variable_page_inspection(page_bytes, lockbox_id, &content_key, mode)
                .is_some()
            {
                i += physical_page_size_from_page_slice(page_bytes)
                    .unwrap_or(DEFAULT_METADATA_PAGE_BYTES);
                continue;
            }
            match decode_page_with_format(page_bytes, lockbox_id, key, mode) {
                Ok(page) => {
                    let page_size = physical_page_size_from_page_slice(page_bytes)
                        .unwrap_or_else(|_| page_size_for_objects(&page.objects));
                    for object in page.objects {
                        if let Some(kind) = record_kind_from_object_kind(object.kind) {
                            let Ok(()) = object.with_payload(|_| ()) else {
                                corrupt_records += 1;
                                continue;
                            };
                            records.push(DecodedRecord {
                                header: RecordHeader {
                                    kind,
                                    sequence: page.sequence,
                                    total_len: page_size as u64,
                                },
                                offset: i as u64,
                                object_id: object.id,
                            });
                        }
                    }
                    i += page_size;
                }
                Err(_) => {
                    corrupt_records += 1;
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }
    content_key.zeroize();
    records.sort_by_key(|record| record.header.sequence);
    #[cfg(any(test, feature = "native-block-layout"))]
    native_pages.sort_by_key(|page| page.sequence);
    Scan {
        #[cfg(any(test, feature = "native-block-layout"))]
        native_pages,
        records,
        corrupt_records,
    }
}

pub(crate) fn inspect_pages(
    bytes: &[u8],
    lockbox_id: LockboxId,
    key: &[u8],
) -> Vec<PageInspection> {
    let mode = crate::file_format::current_header::read_header(bytes)
        .map(|header| header.format_mode)
        .unwrap_or_default();
    let mut pages = Vec::new();
    let mut content_key = derive_page_content_key(key);
    let mut i = crate::constants::HEADER_LEN;
    while i + PAGE_HEADER_LEN <= bytes.len() {
        if &bytes[i..i + 8] == PAGE_MAGIC {
            #[cfg(any(test, feature = "native-block-layout"))]
            if crate::file_format::indexed_frame::block_page::is_native_header(&bytes[i..]) {
                match crate::file_format::indexed_frame::block_page::scan(
                    bytes, i, lockbox_id, mode, key,
                ) {
                    Ok(page) => {
                        // scan validates the complete frame and its protected
                        // metadata before these diagnostic fields are exposed.
                        let stored_body_len = read_u32_le(&bytes[i + 44..i + 48])
                            .expect("scanned native header has a body length");
                        pages.push(PageInspection {
                            offset: i as u64,
                            page_id: page.page_id,
                            sequence: page.sequence,
                            page_size: page.physical_len as u32,
                            encrypted_body_len: stored_body_len,
                            unused_bytes: (page.physical_len
                                - PAGE_HEADER_LEN
                                - stored_body_len as usize)
                                as u32,
                            object_count: 1,
                            objects: vec![PageObjectInspection {
                                id: page.descriptor.frame_id,
                                kind: "file-data",
                                payload_len: page.descriptor.logical_len as usize,
                            }],
                        });
                        i += page.physical_len;
                    }
                    Err(_) => i += 1,
                }
                continue;
            }
            let Some(page_bytes) = page_decode_slice(bytes, i) else {
                break;
            };
            if let Some(inspection) =
                inspect_secure_variable_page(page_bytes, i as u64, lockbox_id, &content_key, mode)
            {
                pages.push(inspection);
                i += physical_page_size_from_page_slice(page_bytes)
                    .unwrap_or(DEFAULT_METADATA_PAGE_BYTES);
                continue;
            }
            if let Ok(decoded) = decode_page_with_format(page_bytes, lockbox_id, key, mode) {
                let page_size = physical_page_size_from_page_slice(page_bytes)
                    .unwrap_or_else(|_| page_size_for_objects(&decoded.objects));
                let Ok(encrypted_body_len) = read_u32_le(&page_bytes[44..48]) else {
                    i += DEFAULT_METADATA_PAGE_BYTES;
                    continue;
                };
                let objects = decoded
                    .objects
                    .iter()
                    .map(|object| PageObjectInspection {
                        id: object.id,
                        kind: page_object_kind_name(object.kind),
                        payload_len: object.payload_len(),
                    })
                    .collect::<Vec<_>>();
                pages.push(PageInspection {
                    offset: i as u64,
                    page_id: decoded.page_id,
                    sequence: decoded.sequence,
                    page_size: page_size as u32,
                    encrypted_body_len,
                    unused_bytes: page_size
                        .saturating_sub(PAGE_HEADER_LEN)
                        .saturating_sub(encrypted_body_len as usize)
                        as u32,
                    object_count: objects.len(),
                    objects,
                });
                i += page_size;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    content_key.zeroize();
    pages
}

fn inspect_secure_variable_page(
    page_bytes: &[u8],
    offset: u64,
    lockbox_id: LockboxId,
    content_key: &[u8; 32],
    mode: crate::creation_options::FormatMode,
) -> Option<PageInspection> {
    let (page_id, sequence, stored_body_len) = public_page_header_metadata(page_bytes)?;
    let object = decode_secure_variable_page_inspection(page_bytes, lockbox_id, content_key, mode)?;
    let page_size = physical_page_size_from_page_slice(page_bytes).ok()?;
    Some(PageInspection {
        offset,
        page_id,
        sequence,
        page_size: page_size as u32,
        encrypted_body_len: stored_body_len as u32,
        unused_bytes: page_size
            .saturating_sub(PAGE_HEADER_LEN)
            .saturating_sub(stored_body_len) as u32,
        object_count: 1,
        objects: vec![object],
    })
}

fn decode_secure_variable_page_inspection(
    page_bytes: &[u8],
    lockbox_id: LockboxId,
    content_key: &[u8; 32],
    mode: crate::creation_options::FormatMode,
) -> Option<PageObjectInspection> {
    let mut page = SecureVec::try_from_slice(page_bytes).ok()?;
    let decoded =
        decode_single_object_page_secure_with_format(&mut page, lockbox_id, content_key, mode)
            .ok()?;
    let object = decoded.objects.first()?;
    if !matches!(
        object.kind,
        PageObjectKind::VariableLeaf
            | PageObjectKind::VariableInternal
            | PageObjectKind::FormLeaf
            | PageObjectKind::FormInternal
    ) {
        return None;
    }
    Some(PageObjectInspection {
        id: object.id,
        kind: page_object_kind_name(object.kind),
        payload_len: object.payload_len(),
    })
}

fn public_page_header_metadata(page: &[u8]) -> Option<(u64, u64, usize)> {
    if page.len() < PAGE_HEADER_LEN || &page[0..8] != PAGE_MAGIC {
        return None;
    }
    Some((
        u64::from_le_bytes(page[16..24].try_into().ok()?),
        u64::from_le_bytes(page[24..32].try_into().ok()?),
        u32::from_le_bytes(page[44..48].try_into().ok()?) as usize,
    ))
}

pub(crate) fn page_decode_slice(bytes: &[u8], offset: usize) -> Option<&[u8]> {
    if offset + PAGE_HEADER_LEN > bytes.len() || bytes.get(offset..offset + 8)? != PAGE_MAGIC {
        return None;
    }
    let header_len =
        u32::from_le_bytes(bytes.get(offset + 12..offset + 16)?.try_into().ok()?) as usize;
    let encrypted_len =
        u32::from_le_bytes(bytes.get(offset + 44..offset + 48)?.try_into().ok()?) as usize;
    let len = header_len.checked_add(encrypted_len)?;
    bytes.get(offset..offset.checked_add(len)?)
}

fn page_objects_are_clear_text(objects: &[PageObject]) -> Result<bool> {
    page_kinds_are_clear_text(objects.iter().map(|object| object.kind))
}

fn page_kinds_are_clear_text(kinds: impl Iterator<Item = PageObjectKind>) -> Result<bool> {
    let mut clear_text = false;
    let mut private = false;
    for kind in kinds {
        if kind.is_clear_text_page_object() {
            clear_text = true;
        } else {
            private = true;
        }
    }
    if clear_text && private {
        return Err(Error::CorruptRecord);
    }
    Ok(clear_text)
}

#[cfg(test)]
fn record_kind_from_object_kind(kind: PageObjectKind) -> Option<RecordKind> {
    match kind {
        PageObjectKind::PackedFileData | PageObjectKind::FileData => Some(RecordKind::FilePage),
        PageObjectKind::Symlink => Some(RecordKind::Symlink),
        PageObjectKind::VariableSet => Some(RecordKind::Variable),
        PageObjectKind::VariableDelete => Some(RecordKind::VariableDelete),
        PageObjectKind::Delete => Some(RecordKind::Delete),
        PageObjectKind::TocLeaf | PageObjectKind::TocInternal => Some(RecordKind::TocNode),
        PageObjectKind::CommitRoot => Some(RecordKind::CommitRoot),
        PageObjectKind::CommitAuth => Some(RecordKind::CommitAuth),
        PageObjectKind::FreeIndexLeaf | PageObjectKind::FreeIndexInternal => {
            Some(RecordKind::FreeIndex)
        }
        PageObjectKind::KeyDirectory
        | PageObjectKind::VariableLeaf
        | PageObjectKind::VariableInternal
        | PageObjectKind::FormLeaf
        | PageObjectKind::FormInternal
        | PageObjectKind::RedactionManifest => None,
    }
}

fn page_object_kind_name(kind: PageObjectKind) -> &'static str {
    match kind {
        PageObjectKind::CommitRoot => "commit-root",
        PageObjectKind::CommitAuth => "commit-auth",
        PageObjectKind::TocLeaf => "toc-leaf",
        PageObjectKind::TocInternal => "toc-internal",
        PageObjectKind::FileData => "file-data",
        PageObjectKind::PackedFileData => "packed-file-data",
        PageObjectKind::Symlink => "symlink",
        PageObjectKind::VariableSet => "variable-set",
        PageObjectKind::VariableDelete => "variable-delete",
        PageObjectKind::KeyDirectory => "key-directory",
        PageObjectKind::FreeIndexLeaf => "free-index-leaf",
        PageObjectKind::FreeIndexInternal => "free-index-internal",
        PageObjectKind::Delete => "delete",
        PageObjectKind::VariableLeaf => "variable-leaf",
        PageObjectKind::VariableInternal => "variable-internal",
        PageObjectKind::FormLeaf => "form-leaf",
        PageObjectKind::FormInternal => "form-internal",
        PageObjectKind::RedactionManifest => "redaction-manifest",
    }
}

fn encode_page_body_plaintext(
    object_stream: &[u8],
    compress: bool,
    sensitive: bool,
) -> Result<SensitiveBytes> {
    let stored = if compress {
        crate::compression::encode_page_body_sensitive(object_stream, sensitive).into_vec()
    } else {
        object_stream.to_vec()
    };
    let stored = SensitiveBytes::new(stored, sensitive);
    let capacity = 16usize
        .checked_add(stored.len())
        .ok_or_else(|| Error::SecurityLimitExceeded("page body is too large".into()))?;
    let mut body = SensitiveBytes::new(Vec::with_capacity(capacity), sensitive);
    body.push(PAGE_BODY_VERSION);
    body.push(if compress {
        COMPRESSION_NORMAL
    } else {
        COMPRESSION_NONE
    });
    body.push(0);
    body.push(0);
    body.extend_from_slice(&(object_stream.len() as u64).to_le_bytes());
    body.extend_from_slice(&0u32.to_le_bytes());
    body.extend_from_slice(&stored);
    Ok(body)
}

fn decode_page_body_plaintext(body: &[u8], sensitive: bool) -> Result<Cow<'_, [u8]>> {
    if body.len() < 16 {
        return Err(Error::CorruptRecord);
    }
    if body[0] != PAGE_BODY_VERSION {
        return Err(Error::CorruptRecord);
    }
    let expected_len = read_u64_le(&body[4..12])?;
    let mut decoded = match body[1] {
        COMPRESSION_NONE => Cow::Borrowed(&body[16..]),
        COMPRESSION_NORMAL => decode_page_body_view(&body[16..])?,
        _ => return Err(Error::CorruptRecord),
    };
    if decoded.len() as u64 != expected_len {
        if sensitive {
            if let Cow::Owned(bytes) = &mut decoded {
                zeroize_bytes(bytes);
            }
        }
        return Err(Error::CorruptRecord);
    }
    Ok(decoded)
}

#[cfg(test)]
fn encode_object_stream(objects: &[PageObject]) -> Result<Vec<u8>> {
    Ok(encode_object_stream_sensitive(objects, false)?.into_vec())
}

fn encode_object_stream_sensitive(
    objects: &[PageObject],
    sensitive: bool,
) -> Result<SensitiveBytes> {
    let capacity = if sensitive {
        encoded_object_stream_len(objects)?
    } else {
        0
    };
    let mut out = SensitiveBytes::new(Vec::with_capacity(capacity), sensitive);
    append_object_stream(&mut out, objects)?;
    Ok(out)
}

fn append_object_stream(out: &mut Vec<u8>, objects: &[PageObject]) -> Result<()> {
    out.extend_from_slice(&(objects.len() as u32).to_le_bytes());
    for object in objects {
        out.push(object.kind as u8);
        out.push(1);
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&object.id.to_le_bytes());
        out.extend_from_slice(&(object.payload_len() as u64).to_le_bytes());
        object.with_payload(|payload| out.extend_from_slice(payload))?;
    }
    Ok(())
}

fn decode_object_descriptors(bytes: &[u8]) -> Result<Vec<PageObjectDescriptor>> {
    if bytes.len() < 4 {
        return Err(Error::CorruptRecord);
    }
    let count = read_u32_le(&bytes[0..4])? as usize;
    if count > (bytes.len() - 4) / 20 {
        return Err(Error::CorruptRecord);
    }
    let mut offset = 4usize;
    let mut objects = Vec::with_capacity(count);
    for _ in 0..count {
        let header_end = offset.checked_add(20).ok_or(Error::CorruptRecord)?;
        let header = bytes.get(offset..header_end).ok_or(Error::CorruptRecord)?;
        let kind = PageObjectKind::from_u8(header[0])?;
        let version = header[1];
        let flags = read_u16_le(&header[2..4])?;
        if version != 1 || flags != 0 {
            return Err(Error::CorruptRecord);
        }
        let id = read_u64_le(&header[4..12])?;
        let payload_len =
            usize::try_from(read_u64_le(&header[12..20])?).map_err(|_| Error::CorruptRecord)?;
        let end = header_end
            .checked_add(payload_len)
            .ok_or(Error::CorruptRecord)?;
        bytes.get(header_end..end).ok_or(Error::CorruptRecord)?;
        objects.push(PageObjectDescriptor {
            kind,
            id,
            payload: header_end..end,
        });
        offset = end;
    }
    if offset != bytes.len() {
        return Err(Error::CorruptRecord);
    }
    Ok(objects)
}

#[cfg(test)]
fn decode_object_stream(bytes: &[u8]) -> Result<Vec<PageObject>> {
    let descriptors = decode_object_descriptors(bytes)?;
    PageDecodeBytes::borrowed(bytes).materialize(descriptors, false)
}

fn page_aad(
    lockbox_id: LockboxId,
    page_id: u64,
    sequence: u64,
    flags: u16,
    encrypted_len: u32,
    physical_len: u64,
) -> Vec<u8> {
    let mut aad = Vec::with_capacity(8 + 2 + 16 + 8 + 8 + 2 + 4);
    aad.extend_from_slice(b"LBX1PAGE");
    aad.extend_from_slice(&PAGE_VERSION.to_le_bytes());
    aad.extend_from_slice(lockbox_id.as_bytes());
    aad.extend_from_slice(&page_id.to_le_bytes());
    aad.extend_from_slice(&sequence.to_le_bytes());
    aad.extend_from_slice(&flags.to_le_bytes());
    aad.extend_from_slice(&encrypted_len.to_le_bytes());
    aad.extend_from_slice(&physical_len.to_le_bytes());
    aad
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owned_metadata_mixed_control_and_compressed_fallbacks_match_borrowed() {
        use crate::{EncryptionMode, SizePadding};
        let archive = LockboxId::from_bytes([82; 16]);
        for encryption in [EncryptionMode::None, EncryptionMode::ChaCha20Poly1305] {
            let mode = owned_test_mode(encryption, SizePadding::Default);
            for objects in [
                vec![PageObject::new(PageObjectKind::TocLeaf, 7, vec![3; 200])],
                vec![
                    PageObject::new(PageObjectKind::FileData, 6, vec![4; 100]),
                    PageObject::new(PageObjectKind::TocLeaf, 7, vec![3; 200]),
                ],
                vec![PageObject::new(PageObjectKind::CommitRoot, 7, vec![3; 392])],
            ] {
                let size = page_size_for_encoded_objects_with_format(&objects, mode).unwrap();
                let page =
                    encode_page_with_format(size, archive, 4096, 7, b"owned", &objects, mode)
                        .unwrap();
                let a = decode_page_with_format(&page, archive, b"owned", mode).unwrap();
                let b = decode_owned_page_with_format(page, archive, b"owned", mode).unwrap();
                assert_eq!(a.objects, objects);
                assert_eq!(b.objects, objects);
                assert!(b
                    .objects
                    .iter()
                    .all(|v| matches!(&v.payload, PagePayload::Normal(_))));
            }
            let objects = vec![PageObject::new(
                PageObjectKind::FileData,
                7,
                vec![0x53; 1500],
            )];
            let stream = encode_object_stream(&objects).unwrap();
            let compressed = crate::compression::encode_page_body(&stream);
            assert_eq!(compressed[8], crate::compression::COMPRESSION_ZSTD);
            let mut body = vec![PAGE_BODY_VERSION, COMPRESSION_NORMAL, 0, 0];
            body.extend_from_slice(&(stream.len() as u64).to_le_bytes());
            body.extend_from_slice(&[0; 4]);
            body.extend_from_slice(&compressed);
            let page = owned_test_wrap_body(&body, mode);
            let a = decode_page_with_format(&page, archive, b"owned", mode).unwrap();
            let b = decode_owned_page_with_format(page.clone(), archive, b"owned", mode).unwrap();
            assert_eq!(a.objects, objects);
            assert_eq!(b.objects, objects);
            assert!(b
                .objects
                .iter()
                .all(|v| matches!(&v.payload, PagePayload::Normal(_))));
            let wrong = owned_test_mode(
                if encryption == EncryptionMode::None {
                    EncryptionMode::ChaCha20Poly1305
                } else {
                    EncryptionMode::None
                },
                SizePadding::Default,
            );
            assert!(matches!(
                decode_page_with_format(&page, archive, b"owned", wrong),
                Err(Error::CorruptRecord)
            ));
            assert!(matches!(
                decode_owned_page_with_format(page, archive, b"owned", wrong),
                Err(Error::CorruptRecord)
            ));
        }
    }

    fn owned_test_mode(
        encryption: crate::EncryptionMode,
        padding: crate::SizePadding,
    ) -> crate::creation_options::FormatMode {
        crate::creation_options::FormatMode::new(crate::LockboxFormatOptions {
            encryption,
            signing: crate::SigningMode::None,
            compression: crate::Compression::default(),
            size_padding: padding,
        })
    }

    #[test]
    fn owned_filedata_pages_share_backing_but_detached_clones_do_not() {
        use crate::{EncryptionMode, SizePadding};
        let archive = LockboxId::from_bytes([81; 16]);
        for encryption in [EncryptionMode::None, EncryptionMode::ChaCha20Poly1305] {
            for padding in [SizePadding::Default, SizePadding::None] {
                let mode = owned_test_mode(encryption, padding);
                let expected = vec![
                    PageObject::new(PageObjectKind::FileData, 10, vec![5; 3000]),
                    PageObject::new(PageObjectKind::FileData, 11, vec![7; 2000]),
                ];
                let size = page_size_for_encoded_objects_with_format(&expected, mode).unwrap();
                let page =
                    encode_page_with_format(size, archive, 4096, 11, b"owned", &expected, mode)
                        .unwrap();
                let borrowed = decode_page_with_format(&page, archive, b"owned", mode).unwrap();
                assert!(borrowed
                    .objects
                    .iter()
                    .all(|v| matches!(&v.payload, PagePayload::Normal(_))));
                let original_pointer = page.as_ptr();
                let original_capacity = page.capacity();
                let owned = decode_owned_page_with_format(page, archive, b"owned", mode).unwrap();
                assert_eq!(owned.objects, borrowed.objects);
                let (PagePayload::OwnedRange(first), PagePayload::OwnedRange(second)) =
                    (&owned.objects[0].payload, &owned.objects[1].payload)
                else {
                    panic!("shared file page");
                };
                assert!(std::sync::Arc::ptr_eq(&first.bytes, &second.bytes));
                assert_eq!(first.bytes.as_ptr(), original_pointer);
                assert_eq!(first.bytes.capacity(), original_capacity);
                let weak = std::sync::Arc::downgrade(&first.bytes);
                let detached = owned.objects[0].clone();
                assert!(matches!(&detached.payload, PagePayload::Normal(_)));
                drop(owned);
                assert!(weak.upgrade().is_none());
                assert_eq!(detached, expected[0]);
            }
        }
    }

    // Valid envelope wrapping makes malformed inner records reach the shared
    // descriptor parser instead of failing only the outer checksum/AEAD.
    fn owned_test_wrap_body(body: &[u8], mode: crate::creation_options::FormatMode) -> Vec<u8> {
        let archive = LockboxId::from_bytes([82; 16]);
        let flags = if mode.plaintext() {
            PAGE_FLAG_CLEAR_TEXT
        } else {
            0
        };
        let mut page = vec![0; 4096];
        let (nonce, stored) = if mode.plaintext() {
            let mut stored = strong_checksum(body).to_vec();
            stored.extend_from_slice(body);
            ([0; 12], stored)
        } else {
            let aad = page_aad(archive, 4096, 7, flags, (body.len() + 16) as u32, 4096);
            seal_with_random_nonce(body, b"owned", &aad).unwrap()
        };
        page[..8].copy_from_slice(PAGE_MAGIC);
        page[8..10].copy_from_slice(&PAGE_VERSION.to_le_bytes());
        page[10..12].copy_from_slice(&flags.to_le_bytes());
        page[12..16].copy_from_slice(&(PAGE_HEADER_LEN as u32).to_le_bytes());
        page[16..24].copy_from_slice(&4096u64.to_le_bytes());
        page[24..32].copy_from_slice(&7u64.to_le_bytes());
        page[32..44].copy_from_slice(&nonce);
        page[44..48].copy_from_slice(&(stored.len() as u32).to_le_bytes());
        page[48..56].copy_from_slice(&4096u64.to_le_bytes());
        let digest = strong_checksum(&page[..PAGE_CHECKSUM_START]);
        page[PAGE_CHECKSUM_START..PAGE_HEADER_LEN].copy_from_slice(&digest);
        page[PAGE_HEADER_LEN..PAGE_HEADER_LEN + stored.len()].copy_from_slice(&stored);
        page
    }

    #[test]
    fn owned_and_borrowed_inner_validation_and_legacy_tail_match() {
        use crate::{EncryptionMode, SizePadding};
        let archive = LockboxId::from_bytes([82; 16]);
        let expected = vec![PageObject::new(
            PageObjectKind::FileData,
            7,
            b"payload".to_vec(),
        )];
        let stream = encode_object_stream(&expected).unwrap();
        for encryption in [EncryptionMode::None, EncryptionMode::ChaCha20Poly1305] {
            let mode = owned_test_mode(encryption, SizePadding::Default);
            for corruption in 0..5 {
                let mut bad = stream.clone();
                match corruption {
                    0 => bad[..4].copy_from_slice(&10000u32.to_le_bytes()),
                    1 => bad[5] = 2,
                    2 => bad[6] = 1,
                    3 => bad[16..24].copy_from_slice(&u64::MAX.to_le_bytes()),
                    _ => bad.push(0),
                }
                let mut body = vec![PAGE_BODY_VERSION, COMPRESSION_NONE, 0, 0];
                body.extend_from_slice(&(bad.len() as u64).to_le_bytes());
                body.extend_from_slice(&[0; 4]);
                body.extend_from_slice(&bad);
                let page = owned_test_wrap_body(&body, mode);
                let a = decode_page_with_format(&page, archive, b"owned", mode).unwrap_err();
                let b = decode_owned_page_with_format(page, archive, b"owned", mode).unwrap_err();
                assert!(matches!(a, Error::CorruptRecord));
                assert!(matches!(b, Error::CorruptRecord));
            }
            // Existing legacy compression envelopes accept trailing body bytes
            // after their declared raw stream. They are not part of that stream.
            let mut body = vec![PAGE_BODY_VERSION, COMPRESSION_NORMAL, 0, 0];
            body.extend_from_slice(&(stream.len() as u64).to_le_bytes());
            body.extend_from_slice(&[0; 4]);
            body.extend_from_slice(&(stream.len() as u64).to_le_bytes());
            body.push(COMPRESSION_NONE);
            body.extend_from_slice(&(stream.len() as u64).to_le_bytes());
            body.extend_from_slice(&stream);
            body.extend_from_slice(b"ignored legacy tail");
            let page = owned_test_wrap_body(&body, mode);
            let a = decode_page_with_format(&page, archive, b"owned", mode).unwrap();
            let b = decode_owned_page_with_format(page, archive, b"owned", mode).unwrap();
            assert_eq!(a.objects, expected);
            assert_eq!(b.objects, expected);
            assert!(b
                .objects
                .iter()
                .all(|v| matches!(&v.payload, PagePayload::Normal(_))));
        }
    }

    #[test]
    fn owned_filedata_last_owner_and_authenticated_error_wipe_live_capacity() {
        use crate::{EncryptionMode, SizePadding};
        let archive = LockboxId::from_bytes([82; 16]);
        let mode = owned_test_mode(EncryptionMode::ChaCha20Poly1305, SizePadding::Default);
        let objects = vec![PageObject::new(
            PageObjectKind::FileData,
            7,
            vec![0x59; 500],
        )];
        let size = page_size_for_encoded_objects_with_format(&objects, mode).unwrap();
        let page =
            encode_page_with_format(size, archive, 4096, 7, b"owned", &objects, mode).unwrap();
        let (decoded, before) = crate::page_buffer::observe_sensitive_drops(|| {
            decode_owned_page_with_format(page, archive, b"owned", mode).unwrap()
        });
        assert!(
            before.is_empty(),
            "successful shared plaintext remains owned"
        );
        let PagePayload::OwnedRange(range) = &decoded.objects[0].payload else {
            panic!("shared owner");
        };
        let retained = std::sync::Arc::clone(&range.bytes);
        let (_, first_drop) = crate::page_buffer::observe_sensitive_drops(|| drop(decoded));
        assert!(first_drop.is_empty());
        let (_, last_drop) = crate::page_buffer::observe_sensitive_drops(|| drop(retained));
        assert_eq!(last_drop.len(), 1);
        assert!(last_drop[0].2 && last_drop[0].1 >= last_drop[0].0 && last_drop[0].0 > 500);

        let mut body = vec![PAGE_BODY_VERSION, COMPRESSION_NONE, 0, 0];
        body.extend_from_slice(&4u64.to_le_bytes());
        body.extend_from_slice(&[0; 4]);
        body.extend_from_slice(&10000u32.to_le_bytes());
        let malformed = owned_test_wrap_body(&body, mode);
        let (result, failed_drop) = crate::page_buffer::observe_sensitive_drops(|| {
            decode_owned_page_with_format(malformed, archive, b"owned", mode)
        });
        assert!(matches!(result, Err(Error::CorruptRecord)));
        assert_eq!(failed_drop.len(), 1);
        assert!(failed_drop[0].2 && failed_drop[0].1 >= failed_drop[0].0);
        let mut invalid_tag = owned_test_wrap_body(&body, mode);
        let stored_len = read_u32_le(&invalid_tag[44..48]).unwrap() as usize;
        invalid_tag[PAGE_HEADER_LEN + stored_len - 1] ^= 1;
        let capacity = invalid_tag.capacity();
        let (result, auth_drop) = crate::page_buffer::observe_sensitive_drops(|| {
            decode_owned_page_with_format(invalid_tag, archive, b"owned", mode)
        });
        assert!(matches!(result, Err(Error::InvalidKey)));
        assert_eq!(auth_drop.len(), 1);
        assert_eq!(auth_drop[0].1, capacity);
        assert!(auth_drop[0].2);
    }

    #[test]
    fn canonical_control_extent_matches_writer_across_modes_and_boundaries() {
        use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
        let archive = LockboxId::from_bytes([31; 16]);
        let mut modes = vec![crate::creation_options::FormatMode::default()];
        for encryption in [EncryptionMode::None, EncryptionMode::ChaCha20Poly1305] {
            for size_padding in [SizePadding::Default, SizePadding::None] {
                for compression in [Compression::None, Compression::default()] {
                    modes.push(crate::creation_options::FormatMode::new(
                        LockboxFormatOptions {
                            encryption,
                            signing: SigningMode::Owner,
                            compression,
                            size_padding,
                        },
                    ));
                }
            }
        }
        for mode in modes {
            for kind in [
                PageObjectKind::CommitRoot,
                PageObjectKind::CommitAuth,
                PageObjectKind::KeyDirectory,
            ] {
                for payload_len in [0, 1, 850, 851, 856, 872, 1024, 4097, 8192] {
                    let objects = vec![PageObject::new(kind, 7, vec![37; payload_len])];
                    let actual = page_size_for_encoded_objects_with_format(&objects, mode).unwrap();
                    assert_eq!(
                        canonical_control_page_len(payload_len, kind, mode).unwrap(),
                        actual as u64
                    );
                    let encoded = encode_page_with_format(
                        actual,
                        archive,
                        512,
                        7,
                        b"controls",
                        &objects,
                        mode,
                    )
                    .unwrap();
                    assert_eq!(encoded.len(), actual);
                    assert!(decode_page_with_format(&encoded, archive, b"controls", mode).is_ok());
                    if !mode.unpadded() {
                        let noncanonical = encode_page_with_format(
                            actual + 1024,
                            archive,
                            512,
                            7,
                            b"controls",
                            &objects,
                            mode,
                        )
                        .unwrap();
                        assert!(
                            decode_page_with_format(&noncanonical, archive, b"controls", mode)
                                .is_err()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn canonical_control_decoder_requires_one_object_and_matching_identity() {
        let archive = LockboxId::from_bytes([41; 16]);
        for objects in [
            vec![PageObject::new(PageObjectKind::CommitRoot, 8, vec![1; 32])],
            vec![
                PageObject::new(PageObjectKind::CommitRoot, 7, vec![1; 32]),
                PageObject::new(PageObjectKind::CommitAuth, 7, vec![2; 32]),
            ],
        ] {
            let encoded = encode_page(1024, archive, 512, 7, b"controls", &objects).unwrap();
            assert!(decode_page(&encoded, archive, b"controls").is_err());
        }
    }

    #[test]
    fn encrypted_raw_pages_match_legacy_full_bytes_with_fresh_nonces() {
        use crate::creation_options::FormatMode;
        use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
        use chacha20poly1305::aead::{Aead, KeyInit, Payload};
        use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};

        fn legacy(
            size: usize,
            archive: LockboxId,
            objects: &[PageObject],
            mode: FormatMode,
            nonce: [u8; 12],
        ) -> Vec<u8> {
            let body = encode_objects_body(objects, mode).unwrap();
            let flags = if mode.unpadded() {
                PAGE_FLAG_UNPADDED
            } else {
                0
            };
            let stored_len = u32::try_from(body.len() + 16).unwrap();
            // Independently assemble authenticated context and legacy allocating AEAD.
            let mut aad = b"LBX1PAGE".to_vec();
            aad.extend_from_slice(&PAGE_VERSION.to_le_bytes());
            aad.extend_from_slice(archive.as_bytes());
            aad.extend_from_slice(&4096u64.to_le_bytes());
            aad.extend_from_slice(&7u64.to_le_bytes());
            aad.extend_from_slice(&flags.to_le_bytes());
            aad.extend_from_slice(&stored_len.to_le_bytes());
            aad.extend_from_slice(&(size as u64).to_le_bytes());
            let mut key = derive_page_content_key(b"test-key");
            let cipher = ChaCha20Poly1305::new(&Key::from(key));
            key.zeroize();
            let encrypted = ZeroizingBytes::new(
                cipher
                    .encrypt(
                        &Nonce::from(nonce),
                        Payload {
                            msg: &body,
                            aad: &aad,
                        },
                    )
                    .unwrap(),
            );
            let mut page = vec![0; size];
            page[..8].copy_from_slice(PAGE_MAGIC);
            page[8..10].copy_from_slice(&PAGE_VERSION.to_le_bytes());
            page[10..12].copy_from_slice(&flags.to_le_bytes());
            page[12..16].copy_from_slice(&(PAGE_HEADER_LEN as u32).to_le_bytes());
            page[16..24].copy_from_slice(&4096u64.to_le_bytes());
            page[24..32].copy_from_slice(&7u64.to_le_bytes());
            page[32..44].copy_from_slice(&nonce);
            page[44..48].copy_from_slice(&stored_len.to_le_bytes());
            page[48..56].copy_from_slice(&(size as u64).to_le_bytes());
            let digest = strong_checksum(&page[..PAGE_CHECKSUM_START]);
            page[PAGE_CHECKSUM_START..PAGE_HEADER_LEN].copy_from_slice(&digest);
            page[PAGE_HEADER_LEN..PAGE_HEADER_LEN + encrypted.len()].copy_from_slice(&encrypted);
            page
        }

        let archive = LockboxId::from_bytes([61; 16]);
        let mut modes = vec![FormatMode::default()];
        for signing in [SigningMode::None, SigningMode::Owner] {
            for size_padding in [SizePadding::Default, SizePadding::None] {
                for compression in [Compression::None, Compression::default()] {
                    modes.push(FormatMode::new(LockboxFormatOptions {
                        encryption: EncryptionMode::ChaCha20Poly1305,
                        signing,
                        size_padding,
                        compression,
                    }));
                }
            }
        }
        for mode in modes {
            for kind in [
                PageObjectKind::FileData,
                PageObjectKind::PackedFileData,
                PageObjectKind::VariableLeaf,
            ] {
                for length in [0, 1, 4096, 8193] {
                    let expected = (0..length).map(|i| (i % 251) as u8).collect::<Vec<_>>();
                    let objects = vec![
                        PageObject::new(kind, 6, expected.clone()),
                        PageObject::new_secure(
                            kind,
                            7,
                            SecureVec::try_from_slice(&expected).unwrap(),
                        ),
                    ];
                    let body = encode_objects_body(&objects, mode).unwrap();
                    let used = PAGE_HEADER_LEN + body.len() + 16;
                    let size = if mode.unpadded() {
                        used
                    } else {
                        (used + 257).div_ceil(PAGE_SIZE_GRANULARITY) * PAGE_SIZE_GRANULARITY
                    };
                    let page = encode_page_with_format(
                        size,
                        archive,
                        4096,
                        7,
                        b"test-key",
                        &objects,
                        mode,
                    )
                    .unwrap();
                    let nonce: [u8; 12] = page[32..44].try_into().unwrap();
                    assert_eq!(page, legacy(size, archive, &objects, mode, nonce));
                    assert!(page[used..].iter().all(|b| *b == 0));
                    assert_eq!(
                        decode_page_with_format(&page, archive, b"test-key", mode)
                            .unwrap()
                            .objects,
                        objects
                    );
                    let second = encode_page_with_format(
                        size,
                        archive,
                        4096,
                        7,
                        b"test-key",
                        &objects,
                        mode,
                    )
                    .unwrap();
                    assert_ne!(&page[32..44], &second[32..44]);
                    assert_eq!(
                        decode_page_with_format(&second, archive, b"test-key", mode)
                            .unwrap()
                            .objects,
                        objects
                    );
                    for object in &objects {
                        object
                            .with_payload(|bytes| assert_eq!(bytes, expected))
                            .unwrap();
                    }
                    assert!(encode_page_with_format(
                        used - 1,
                        archive,
                        4096,
                        7,
                        b"test-key",
                        &objects,
                        mode
                    )
                    .is_err());
                    if mode.unpadded() {
                        assert!(matches!(
                            encode_page_with_format(
                                used + 1,
                                archive,
                                4096,
                                7,
                                b"test-key",
                                &objects,
                                mode
                            ),
                            Err(Error::CorruptRecord)
                        ));
                    }
                }
            }
            if mode.0 != 0 && mode.options().compression == Compression::None {
                let used = PAGE_HEADER_LEN + 16 + 4 + 16;
                let size = if mode.unpadded() {
                    used
                } else {
                    (used + 257).div_ceil(PAGE_SIZE_GRANULARITY) * PAGE_SIZE_GRANULARITY
                };
                let page = encode_page_with_format(size, archive, 4096, 7, b"test-key", &[], mode)
                    .unwrap();
                assert_eq!(
                    page,
                    legacy(size, archive, &[], mode, page[32..44].try_into().unwrap())
                );
                assert!(decode_page_with_format(&page, archive, b"test-key", mode)
                    .unwrap()
                    .objects
                    .is_empty());
            }
        }
    }

    #[test]
    fn encrypted_raw_pages_reject_authenticated_context_and_payload_changes() {
        let archive = LockboxId::from_bytes([62; 16]);
        let objects = vec![PageObject::new(
            PageObjectKind::FileData,
            7,
            vec![0x5a; 128],
        )];
        let page = encode_page(4096, archive, 4096, 7, b"key", &objects).unwrap();
        let stored = u32::from_le_bytes(page[44..48].try_into().unwrap()) as usize;
        assert!(decode_page(&page, archive, b"wrong").is_err());
        assert!(decode_page(&page, LockboxId::from_bytes([63; 16]), b"key").is_err());
        for index in [PAGE_HEADER_LEN, PAGE_HEADER_LEN + stored - 1, 16, 24, 32] {
            let mut changed = page.clone();
            changed[index] ^= 1;
            if index < PAGE_HEADER_LEN {
                let digest = strong_checksum(&changed[..PAGE_CHECKSUM_START]);
                changed[PAGE_CHECKSUM_START..PAGE_HEADER_LEN].copy_from_slice(&digest);
            }
            assert!(matches!(
                decode_page(&changed, archive, b"key"),
                Err(Error::InvalidKey)
            ));
        }
        let mut enlarged = page.clone();
        enlarged.resize(page.len() + 1024, 0);
        let length = enlarged.len() as u64;
        enlarged[48..56].copy_from_slice(&length.to_le_bytes());
        let digest = strong_checksum(&enlarged[..PAGE_CHECKSUM_START]);
        enlarged[PAGE_CHECKSUM_START..PAGE_HEADER_LEN].copy_from_slice(&digest);
        assert!(matches!(
            decode_page(&enlarged, archive, b"key"),
            Err(Error::InvalidKey)
        ));
        // Refused extents and unchanged scoped inputs are covered above; full-capacity
        // wiping primitive is covered by existing buffer tests. This is not
        // an injected seal-failure or unwinding integration test.
    }

    #[test]
    fn direct_clear_raw_pages_match_legacy_full_bytes_and_extent_rules() {
        use crate::creation_options::FormatMode;
        use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};

        // Independent pre-change cleartext assembly, including the full header.
        fn legacy(page_size: usize, objects: &[PageObject], mode: FormatMode) -> Vec<u8> {
            let body = encode_objects_body(objects, mode).unwrap();
            let flags = PAGE_FLAG_CLEAR_TEXT
                | if mode.unpadded() {
                    PAGE_FLAG_UNPADDED
                } else {
                    0
                };
            let mut page = vec![0; page_size];
            page[0..8].copy_from_slice(PAGE_MAGIC);
            page[8..10].copy_from_slice(&PAGE_VERSION.to_le_bytes());
            page[10..12].copy_from_slice(&flags.to_le_bytes());
            page[12..16].copy_from_slice(&(PAGE_HEADER_LEN as u32).to_le_bytes());
            page[16..24].copy_from_slice(&4096u64.to_le_bytes());
            page[24..32].copy_from_slice(&7u64.to_le_bytes());
            page[44..48].copy_from_slice(&((32 + body.len()) as u32).to_le_bytes());
            page[48..56].copy_from_slice(&(page_size as u64).to_le_bytes());
            let header_digest = strong_checksum(&page[..PAGE_CHECKSUM_START]);
            page[PAGE_CHECKSUM_START..PAGE_HEADER_LEN].copy_from_slice(&header_digest);
            page[PAGE_HEADER_LEN..PAGE_HEADER_LEN + 32].copy_from_slice(&strong_checksum(&body));
            page[PAGE_HEADER_LEN + 32..PAGE_HEADER_LEN + 32 + body.len()].copy_from_slice(&body);
            page
        }

        let archive = LockboxId::from_bytes([51; 16]);
        let mut modes = vec![FormatMode::default()];
        for encryption in [EncryptionMode::None, EncryptionMode::ChaCha20Poly1305] {
            for signing in [SigningMode::None, SigningMode::Owner] {
                for size_padding in [SizePadding::Default, SizePadding::None] {
                    for compression in [Compression::None, Compression::default()] {
                        modes.push(FormatMode::new(LockboxFormatOptions {
                            encryption,
                            signing,
                            size_padding,
                            compression,
                        }));
                    }
                }
            }
        }
        for mode in modes {
            if mode.plaintext() && mode.options().compression == Compression::None {
                let used = PAGE_HEADER_LEN + 32 + 16 + 4;
                let page_size = if mode.unpadded() { used } else { used + 257 };
                let actual =
                    encode_page_with_format(page_size, archive, 4096, 7, b"test-key", &[], mode)
                        .unwrap();
                assert_eq!(actual, legacy(page_size, &[], mode));
            }
            for kind in [PageObjectKind::FileData, PageObjectKind::KeyDirectory] {
                if kind == PageObjectKind::FileData && !mode.plaintext() {
                    continue;
                }
                for length in [0, 1, 4096, 8193] {
                    let expected = vec![0x5a; length];
                    let mut objects = vec![PageObject::new_secure(
                        kind,
                        7,
                        SecureVec::try_from_slice(&expected).unwrap(),
                    )];
                    if kind == PageObjectKind::FileData {
                        objects.insert(0, PageObject::new(kind, 6, expected.clone()));
                    }
                    let body_len = encode_objects_body(&objects, mode).unwrap().len();
                    let used = PAGE_HEADER_LEN + 32 + body_len;
                    let page_size = if mode.unpadded() { used } else { used + 257 };
                    let actual = encode_page_with_format(
                        page_size,
                        archive,
                        4096,
                        7,
                        b"test-key",
                        &objects,
                        mode,
                    )
                    .unwrap();
                    assert_eq!(actual, legacy(page_size, &objects, mode));
                    assert!(actual[used..].iter().all(|byte| *byte == 0));
                    for object in &objects {
                        object
                            .with_payload(|bytes| assert_eq!(bytes, expected))
                            .unwrap();
                    }
                    assert!(matches!(
                        encode_page_with_format(
                            used - 1,
                            archive,
                            4096,
                            7,
                            b"test-key",
                            &objects,
                            mode,
                        ),
                        Err(Error::SecurityLimitExceeded(_))
                    ));
                    if mode.unpadded() {
                        assert!(matches!(
                            encode_page_with_format(
                                used + 1,
                                archive,
                                4096,
                                7,
                                b"test-key",
                                &objects,
                                mode,
                            ),
                            Err(Error::CorruptRecord)
                        ));
                    }
                }
            }
        }
        let mixed = [
            PageObject::new(PageObjectKind::KeyDirectory, 7, vec![1]),
            PageObject::new(PageObjectKind::FileData, 8, vec![2]),
        ];
        assert!(encode_page_with_format(
            1024,
            archive,
            4096,
            7,
            b"test-key",
            &mixed,
            FormatMode::default(),
        )
        .is_err());
    }
    #[test]
    fn object_stream_reservation_preserves_fixed_wire_bytes_and_scoped_payloads() {
        let empty = encode_object_stream(&[]).unwrap();
        assert_eq!(empty, [0, 0, 0, 0]);
        let objects = [
            PageObject::new(PageObjectKind::FileData, 1, vec![0xab, 0xcd]),
            PageObject::new_secure(
                PageObjectKind::PackedFileData,
                2,
                SecureVec::try_from_slice(&[0xef]).unwrap(),
            ),
        ];
        let encoded = encode_object_stream(&objects).unwrap();
        assert_eq!(
            encoded,
            [
                2, 0, 0, 0, 4, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0xab, 0xcd,
                5, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0xef,
            ]
        );
        assert_eq!(encoded.len(), encoded_object_stream_len(&objects).unwrap());
        objects[0]
            .with_payload(|bytes| assert_eq!(bytes, [0xab, 0xcd]))
            .unwrap();
        objects[1]
            .with_payload(|bytes| assert_eq!(bytes, [0xef]))
            .unwrap();
        let decoded = decode_object_stream(&encoded).unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].id, 1);
        assert_eq!(decoded[1].id, 2);
        decoded[0]
            .with_payload(|bytes| assert_eq!(bytes, [0xab, 0xcd]))
            .unwrap();
        decoded[1]
            .with_payload(|bytes| assert_eq!(bytes, [0xef]))
            .unwrap();
    }

    #[test]
    fn arithmetic_page_sizes_match_serialized_bodies_in_every_explicit_mode() {
        use crate::creation_options::FormatMode;
        use crate::{
            Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding, ZstdLevel,
        };
        for kind in [
            PageObjectKind::FileData,
            PageObjectKind::PackedFileData,
            PageObjectKind::VariableLeaf,
            PageObjectKind::FormInternal,
            PageObjectKind::TocLeaf,
            PageObjectKind::KeyDirectory,
        ] {
            for len in [0, 1, 855, 856, 857, 871, 872, 873, 4097] {
                let objects = [PageObject::new(kind, 19, vec![37; len])];
                for encryption in [EncryptionMode::None, EncryptionMode::ChaCha20Poly1305] {
                    for signing in [SigningMode::None, SigningMode::Owner] {
                        for size_padding in [SizePadding::Default, SizePadding::None] {
                            for compression in [
                                Compression::None,
                                Compression::default(),
                                Compression::Zstd {
                                    level: ZstdLevel::new(22).unwrap(),
                                },
                            ] {
                                let mode = FormatMode::new(LockboxFormatOptions {
                                    encryption,
                                    signing,
                                    size_padding,
                                    compression,
                                });
                                let body = encode_objects_body(&objects, mode).unwrap();
                                // Bind the codec decision as well as its length: file frames
                                // and private tree bodies must not acquire page compression.
                                let eligible = matches!(kind, PageObjectKind::TocLeaf)
                                    && compression != Compression::None;
                                assert_eq!(
                                    body[1],
                                    if eligible {
                                        COMPRESSION_NORMAL
                                    } else {
                                        COMPRESSION_NONE
                                    }
                                );
                                let stored = PAGE_HEADER_LEN
                                    + body.len()
                                    + if mode.plaintext() || kind == PageObjectKind::KeyDirectory {
                                        32
                                    } else {
                                        16
                                    };
                                let expected = if mode.unpadded() {
                                    stored
                                } else {
                                    page_size_for_stored_len(
                                        stored,
                                        max_page_size_for_objects(&objects),
                                    )
                                    .unwrap()
                                };
                                assert_eq!(
                                    page_size_for_encoded_objects_with_format(&objects, mode)
                                        .unwrap(),
                                    expected,
                                    "{kind:?}, {len}, {mode:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn arithmetic_sizing_handles_shared_secure_payloads_and_physical_limits() {
        use crate::creation_options::FormatMode;
        use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
        for encryption in [EncryptionMode::None, EncryptionMode::ChaCha20Poly1305] {
            for size_padding in [SizePadding::Default, SizePadding::None] {
                let mode = FormatMode::new(LockboxFormatOptions {
                    encryption,
                    signing: SigningMode::None,
                    size_padding,
                    compression: Compression::default(),
                });
                let objects = [
                    PageObject::new(PageObjectKind::FileData, 1, vec![3; 4 * 1024 * 1024]),
                    PageObject::new_secure(
                        PageObjectKind::PackedFileData,
                        2,
                        SecureVec::try_from_slice(&[5; 17]).unwrap(),
                    ),
                ];
                let body = encode_objects_body(&objects, mode).unwrap();
                let stored = PAGE_HEADER_LEN + body.len() + if mode.plaintext() { 32 } else { 16 };
                let expected = if mode.unpadded() {
                    stored
                } else {
                    page_size_for_stored_len(stored, DEFAULT_DATA_PAGE_BYTES).unwrap()
                };
                assert_eq!(
                    page_size_for_encoded_objects_with_format(&objects, mode).unwrap(),
                    expected
                );
                let oversized = [PageObject::new(
                    PageObjectKind::FileData,
                    1,
                    vec![0; DEFAULT_DATA_PAGE_BYTES],
                )];
                assert!(matches!(
                    page_size_for_encoded_objects_with_format(&oversized, mode),
                    Err(Error::SecurityLimitExceeded(_))
                ));
            }
        }
    }

    #[test]
    fn physical_length_recovery_preserves_checksum_and_rejects_other_damage() {
        use crate::creation_options::FormatMode;
        use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
        let id = LockboxId::new_random().unwrap();
        let objects = [PageObject::new(PageObjectKind::CommitRoot, 7, vec![7; 88])];
        for encryption in [EncryptionMode::None, EncryptionMode::ChaCha20Poly1305] {
            for size_padding in [SizePadding::Default, SizePadding::None] {
                let mode = FormatMode::new(LockboxFormatOptions {
                    encryption,
                    size_padding,
                    signing: SigningMode::Owner,
                    compression: Compression::None,
                });
                let size = canonical_control_page_len(
                    objects[0].payload_len(),
                    PageObjectKind::CommitRoot,
                    mode,
                )
                .unwrap() as usize;
                let page =
                    encode_page_with_format(size, id, 3, 7, b"secret", &objects, mode).unwrap();
                assert!(recover_physical_length_header(&page).is_none());
                for field_byte in 48..56 {
                    let mut damaged = page.clone();
                    damaged[field_byte] ^= 0x55;
                    let header = recover_physical_length_header(&damaged).unwrap();
                    assert_eq!(header, page[..PAGE_HEADER_LEN]);
                    assert!(recover_physical_length_header(&damaged[..size - 1]).is_none());
                    damaged[PAGE_CHECKSUM_START] ^= 1;
                    assert!(recover_physical_length_header(&damaged).is_none());
                    damaged[PAGE_CHECKSUM_START] ^= 1;
                    damaged[24] ^= 1;
                    assert!(recover_physical_length_header(&damaged).is_none());
                    damaged[24] ^= 1;
                    damaged[..PAGE_HEADER_LEN].copy_from_slice(&header);
                    assert_eq!(
                        decode_page_with_format(&damaged, id, b"secret", mode)
                            .unwrap()
                            .objects,
                        objects
                    );
                    damaged[PAGE_HEADER_LEN + 8] ^= 1;
                    assert!(decode_page_with_format(&damaged, id, b"secret", mode).is_err());
                }
            }
        }
    }

    #[test]
    fn page_round_trips_objects_and_has_fixed_size() {
        let lockbox_id = LockboxId::new_random().unwrap();
        let key = b"secret";
        let objects = vec![
            PageObject::new(PageObjectKind::TocLeaf, 10, b"toc".to_vec()),
            PageObject::new(PageObjectKind::PackedFileData, 11, b"file".to_vec()),
        ];

        let page = encode_page(128 * 1024, lockbox_id, 3, 7, key, &objects).unwrap();
        assert_eq!(page.len(), 128 * 1024);
        assert_eq!(&page[0..8], PAGE_MAGIC);

        let decoded = decode_page(&page, lockbox_id, key).unwrap();
        assert_eq!(decoded.page_id, 3);
        assert_eq!(decoded.sequence, 7);
        assert_eq!(decoded.objects, objects);
    }

    #[test]
    fn page_rejects_tampering() {
        let lockbox_id = LockboxId::new_random().unwrap();
        let key = b"secret";
        let objects = vec![PageObject::new(
            PageObjectKind::TocLeaf,
            10,
            b"toc".to_vec(),
        )];

        let mut page = encode_page(128 * 1024, lockbox_id, 3, 7, key, &objects).unwrap();
        page[PAGE_HEADER_LEN + 8] ^= 0x01;

        assert!(decode_page(&page, lockbox_id, key).is_err());
    }

    #[test]
    fn page_rejects_public_header_checksum_tampering() {
        let lockbox_id = LockboxId::new_random().unwrap();
        let key = b"secret";
        let objects = vec![PageObject::new(
            PageObjectKind::TocLeaf,
            10,
            b"toc".to_vec(),
        )];

        let mut page = encode_page(128 * 1024, lockbox_id, 3, 7, key, &objects).unwrap();
        page[16] ^= 0x01;

        assert!(decode_page(&page, lockbox_id, key).is_err());
    }

    #[test]
    fn clear_text_page_uses_page_checksum_for_body_integrity() {
        let lockbox_id = LockboxId::new_random().unwrap();
        let objects = vec![PageObject::new(
            PageObjectKind::KeyDirectory,
            7,
            b"keys".to_vec(),
        )];

        let size = canonical_control_page_len(
            objects[0].payload_len(),
            PageObjectKind::KeyDirectory,
            crate::creation_options::FormatMode::default(),
        )
        .unwrap() as usize;
        let mut page = encode_page(size, lockbox_id, 3, 7, b"", &objects).unwrap();
        assert_eq!(
            u16::from_le_bytes(page[10..12].try_into().unwrap()),
            PAGE_FLAG_CLEAR_TEXT
        );
        let decoded = decode_page(&page, LockboxId::from_bytes([0; 16]), b"").unwrap();
        assert_eq!(decoded.objects, objects);

        page[PAGE_HEADER_LEN + 40] ^= 0x01;
        assert!(decode_page(&page, LockboxId::from_bytes([0; 16]), b"").is_err());
    }

    #[test]
    fn secure_payload_can_be_read_only_through_scoped_access() {
        let payload = SecureVec::try_from_slice(b"secret").unwrap();
        let object = PageObject::new_secure(PageObjectKind::VariableLeaf, 10, payload);

        assert!(object.with_payload(|payload| payload == b"secret").unwrap());
    }
}
