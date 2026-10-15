//! Experimental byte segmentation for selected variable values. Descriptors are
//! metadata only; callers must authenticate membership and physical ownership.
//! Plaintext archive images remain publicly readable. Guarded processing does
//! not add at-rest confidentiality or replace selected owner authentication.
use super::allocation_map::Extent;
use super::page::{
    decode_single_object_page_secure_with_format, secure_page_size, PageObjectKind,
    SecureSingleObjectPage,
};
use crate::creation_options::FormatMode;
use crate::crypto::strong_checksum;
use crate::secret_vec::SecureVec;
use crate::storage::Storage;
use crate::{Error, LockboxId, Result, SecretString, VariableSensitivity};

pub(crate) const SEGMENT_BYTES: usize = 64 * 1024;
const MAX_VALUE: usize = crate::constants::MAX_VARIABLE_VALUE_BYTES;
const META: usize = 56;
const PREFIX: usize = 96;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Layout {
    pub id: [u8; 16],
    pub revision: u64,
    pub sensitivity: VariableSensitivity,
    pub mode: FormatMode,
    pub length: usize,
    pub extents: Vec<Extent>,
}
pub(crate) struct Encoded {
    pub layout: Layout,
    pub pages: Vec<SecureVec>,
}
impl Layout {
    fn count(&self) -> usize {
        self.length.div_ceil(SEGMENT_BYTES).max(1)
    }
    fn part_len(&self, ordinal: usize) -> Result<usize> {
        if ordinal >= self.count() {
            return Err(Error::CorruptRecord);
        }
        Ok(self
            .length
            .saturating_sub(ordinal * SEGMENT_BYTES)
            .min(SEGMENT_BYTES))
    }
    fn page_id(&self, ordinal: usize) -> u64 {
        u64::from_le_bytes(self.id[..8].try_into().unwrap()) ^ ordinal as u64
    }
    fn metadata(&self) -> [u8; META] {
        let mut bytes = [0; META];
        bytes[..8].copy_from_slice(b"RV4VAL01");
        bytes[8..24].copy_from_slice(&self.id);
        bytes[24..32].copy_from_slice(&self.revision.to_le_bytes());
        bytes[32..34].copy_from_slice(&self.mode.0.to_le_bytes());
        bytes[34] = match self.sensitivity {
            VariableSensitivity::Normal => 0,
            VariableSensitivity::Secret => 1,
        };
        bytes[40..48].copy_from_slice(&(self.length as u64).to_le_bytes());
        bytes[48..52].copy_from_slice(&(self.count() as u32).to_le_bytes());
        bytes
    }
    fn prefix(&self, archive: LockboxId, ordinal: usize) -> [u8; PREFIX] {
        let mut bytes = [0; PREFIX];
        bytes[..META].copy_from_slice(&self.metadata());
        bytes[..8].copy_from_slice(b"RV4SEG01");
        bytes[56..60].copy_from_slice(&(ordinal as u32).to_le_bytes());
        bytes[64..72].copy_from_slice(&((ordinal * SEGMENT_BYTES) as u64).to_le_bytes());
        bytes[72..80].copy_from_slice(&self.page_id(ordinal).to_le_bytes());
        bytes[80..96].copy_from_slice(archive.as_bytes());
        bytes
    }
    pub(crate) fn validate(&self, sealed: u64) -> Result<()> {
        FormatMode::parse(self.mode.0)?;
        if self.mode.0 == 0 {
            return Err(Error::CorruptRecord);
        }
        if self.id == [0; 16]
            || self.revision == 0
            || self.length > MAX_VALUE
            || self.extents.len() != self.count()
        {
            return Err(Error::CorruptRecord);
        }
        let mut spans = Vec::with_capacity(self.count());
        for (ordinal, extent) in self.extents.iter().enumerate() {
            let expected = secure_page_size(PREFIX + self.part_len(ordinal)?, self.mode)?;
            let end = extent
                .start
                .checked_add(extent.len)
                .ok_or(Error::CorruptRecord)?;
            if extent.start < super::publication_anchor::REGION_LEN as u64
                || extent.len != expected as u64
                || end > sealed
            {
                return Err(Error::CorruptRecord);
            }
            spans.push((extent.start, end));
        }
        spans.sort_unstable();
        if spans.windows(2).any(|pair| pair[0].1 > pair[1].0) {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }
    pub(crate) fn encode_metadata(&self) -> Vec<u8> {
        let mut out = self.metadata().to_vec();
        for extent in &self.extents {
            out.extend_from_slice(&extent.start.to_le_bytes());
            out.extend_from_slice(&extent.len.to_le_bytes());
            out.extend_from_slice(&extent.digest);
        }
        out
    }
    pub(crate) fn decode_metadata(bytes: &[u8], mode: FormatMode, sealed: u64) -> Result<Self> {
        if bytes.len() < META
            || &bytes[..8] != b"RV4VAL01"
            || bytes[35..40]
                .iter()
                .chain(&bytes[52..56])
                .any(|byte| *byte != 0)
            || u16::from_le_bytes(bytes[32..34].try_into().unwrap()) != mode.0
        {
            return Err(Error::CorruptRecord);
        }
        let length = u64::from_le_bytes(bytes[40..48].try_into().unwrap());
        if length > MAX_VALUE as u64 {
            return Err(Error::CorruptRecord);
        }
        let count = (length as usize).div_ceil(SEGMENT_BYTES).max(1);
        if u32::from_le_bytes(bytes[48..52].try_into().unwrap()) as usize != count
            || bytes.len() != META + count * 48
        {
            return Err(Error::CorruptRecord);
        }
        let layout = Self {
            id: bytes[8..24].try_into().unwrap(),
            revision: u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
            sensitivity: match bytes[34] {
                0 => VariableSensitivity::Normal,
                1 => VariableSensitivity::Secret,
                _ => return Err(Error::CorruptRecord),
            },
            mode,
            length: length as usize,
            extents: bytes[META..]
                .chunks_exact(48)
                .map(|part| Extent {
                    start: u64::from_le_bytes(part[..8].try_into().unwrap()),
                    len: u64::from_le_bytes(part[8..16].try_into().unwrap()),
                    digest: part[16..48].try_into().unwrap(),
                })
                .collect(),
        };
        layout.validate(sealed)?;
        Ok(layout)
    }
    /// Rebind only placement. A caller cannot substitute differently sized or
    /// differently authenticated staged pages through an allocation callback.
    pub(crate) fn rebind(&mut self, extents: &[Extent]) -> Result<()> {
        if extents.len() != self.extents.len()
            || extents
                .iter()
                .zip(&self.extents)
                .any(|(new, old)| new.len != old.len || new.digest != old.digest)
        {
            return Err(Error::CorruptRecord);
        }
        self.extents = extents.to_vec();
        self.validate(u64::MAX)
    }
    /// The caller supplies a descriptor from the currently selected authenticated
    /// graph. This checks stored bytes/context; possession of this metadata alone
    /// does not authorize a read or prove current membership.
    pub(crate) fn read(
        &self,
        storage: &impl Storage,
        archive: LockboxId,
        content_key: &[u8; 32],
        sealed: u64,
    ) -> Result<SecureVec> {
        self.validate(sealed)?;
        let mut value = SecureVec::new();
        for (ordinal, extent) in self.extents.iter().enumerate() {
            let mut page = storage.read_at_secure(extent.start, extent.len as usize)?;
            if page.with_bytes(strong_checksum)? != extent.digest {
                return Err(Error::CorruptRecord);
            }
            let decoded = decode_single_object_page_secure_with_format(
                &mut page,
                archive,
                content_key,
                self.mode,
            )?;
            if decoded.page_id != self.page_id(ordinal)
                || decoded.sequence != self.revision
                || decoded.objects.len() != 1
            {
                return Err(Error::CorruptRecord);
            }
            let object = &decoded.objects[0];
            if object.kind != PageObjectKind::VariableLeaf || object.id != self.page_id(ordinal) {
                return Err(Error::CorruptRecord);
            }
            let payload = object.secure_payload().ok_or(Error::CorruptRecord)?;
            let length = self.part_len(ordinal)?;
            if payload.len() != PREFIX + length
                || !payload.with_bytes(|bytes| bytes[..PREFIX] == self.prefix(archive, ordinal))?
            {
                return Err(Error::CorruptRecord);
            }
            value.try_extend_secure_range(payload, PREFIX, length)?;
        }
        if value.len() != self.length {
            return Err(Error::CorruptRecord);
        }
        value.with_bytes(|bytes| {
            let text = std::str::from_utf8(bytes).map_err(|_| Error::CorruptRecord)?;
            crate::security::validate_variable_value_ref(text).map_err(|_| Error::CorruptRecord)
        })??;
        Ok(value)
    }
}

/// Borrowed guarded input. Neither variant exposes plaintext beyond its scoped
/// reader or requires an owned full-value copy before segment construction.
#[derive(Clone, Copy)]
pub(crate) enum Source<'a> {
    Bytes(&'a SecureVec),
    Secret(&'a SecretString),
}
impl Source<'_> {
    pub(crate) fn with_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Result<R> {
        match self {
            Self::Bytes(value) => Ok(value.with_bytes(f)?),
            Self::Secret(value) => Ok(value.with_bytes(f)?),
        }
    }
    fn append_range(&self, target: &mut SecureVec, offset: usize, len: usize) -> Result<()> {
        match self {
            Self::Bytes(value) => target.try_extend_secure_range(value, offset, len)?,
            Self::Secret(value) => value.append_range_to_secure_vec(target, offset, len)?,
        }
        Ok(())
    }
}

pub(crate) fn encode(
    value: &SecureVec,
    archive: LockboxId,
    mode: FormatMode,
    content_key: &[u8; 32],
    id: [u8; 16],
    revision: u64,
    sensitivity: VariableSensitivity,
) -> Result<Encoded> {
    encode_source(
        Source::Bytes(value),
        archive,
        mode,
        content_key,
        id,
        revision,
        sensitivity,
    )
}
pub(crate) fn encode_source(
    value: Source<'_>,
    archive: LockboxId,
    mode: FormatMode,
    content_key: &[u8; 32],
    id: [u8; 16],
    revision: u64,
    sensitivity: VariableSensitivity,
) -> Result<Encoded> {
    let mut layout = source_layout(value, mode, id, revision, sensitivity)?;
    let mut pages = Vec::new();
    for ordinal in 0..layout.count() {
        let mut payload = SecureVec::try_from_slice(&layout.prefix(archive, ordinal))?;
        value.append_range(
            &mut payload,
            ordinal * SEGMENT_BYTES,
            layout.part_len(ordinal)?,
        )?;
        let size = secure_page_size(payload.len(), mode)?;
        let page = super::page::secure_storage::encode_secure_storage(SecureSingleObjectPage {
            format_mode: mode,
            page_size: size,
            lockbox_id: archive,
            page_id: layout.page_id(ordinal),
            sequence: revision,
            content_key,
            kind: PageObjectKind::VariableLeaf,
            id: layout.page_id(ordinal),
            payload: &payload,
        })?;
        layout.extents.push(Extent {
            start: 0,
            len: page.len() as u64,
            digest: page.with_bytes(strong_checksum)?,
        });
        pages.push(page);
    }
    Ok(Encoded { layout, pages })
}

fn source_layout(
    value: Source<'_>,
    mode: FormatMode,
    id: [u8; 16],
    revision: u64,
    sensitivity: VariableSensitivity,
) -> Result<Layout> {
    FormatMode::parse(mode.0)?;
    if mode.0 == 0 {
        return Err(Error::CorruptRecord);
    }
    let length = value.with_bytes(|bytes| -> Result<usize> {
        if bytes.len() > MAX_VALUE {
            return Err(Error::SecurityLimitExceeded(
                "variable value exceeds 1 MiB".into(),
            ));
        }
        if id == [0; 16] || revision == 0 {
            return Err(Error::CorruptRecord);
        }
        let text = std::str::from_utf8(bytes)
            .map_err(|_| Error::InvalidInput("variable value must be UTF-8".into()))?;
        crate::security::validate_variable_value_ref(text)?;
        Ok(bytes.len())
    })??;
    Ok(Layout {
        id,
        revision,
        sensitivity,
        mode,
        length,
        extents: Vec::new(),
    })
}
pub(crate) struct PreparedSegments<'a> {
    pub layout: Layout,
    pub pages: super::page::secure_storage::PreparedSecurePages,
    pub payload: Box<dyn FnMut(usize) -> Result<SecureVec> + 'a>,
}
pub(crate) fn prepare_source<'a>(
    value: Source<'a>,
    archive: LockboxId,
    mode: FormatMode,
    content_key: &[u8; 32],
    id: [u8; 16],
    revision: u64,
    sensitivity: VariableSensitivity,
) -> Result<PreparedSegments<'a>> {
    let mut layout = source_layout(value, mode, id, revision, sensitivity)?;
    let template = layout.clone();
    let payload = move |ordinal: usize| -> Result<SecureVec> {
        if ordinal >= template.count() {
            return Err(Error::CorruptRecord);
        }
        let mut payload = SecureVec::try_from_slice(&template.prefix(archive, ordinal))?;
        value.append_range(
            &mut payload,
            ordinal * SEGMENT_BYTES,
            template.part_len(ordinal)?,
        )?;
        Ok(payload)
    };
    let mut pages = super::page::secure_storage::PreparedSecurePages::new(layout.count())?;
    for ordinal in 0..layout.count() {
        let raw = payload(ordinal)?;
        pages.push(SecureSingleObjectPage {
            format_mode: mode,
            page_size: secure_page_size(raw.len(), mode)?,
            lockbox_id: archive,
            page_id: layout.page_id(ordinal),
            sequence: revision,
            content_key,
            kind: PageObjectKind::VariableLeaf,
            id: layout.page_id(ordinal),
            payload: &raw,
        })?;
        layout.extents.push(Extent {
            start: 0,
            len: pages.size(ordinal)? as u64,
            digest: pages.digest(ordinal)?,
        });
    }
    Ok(PreparedSegments {
        layout,
        pages,
        payload: Box::new(payload),
    })
}

#[cfg(test)]
mod tests;
