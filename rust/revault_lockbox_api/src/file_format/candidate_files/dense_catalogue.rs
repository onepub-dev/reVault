//! Bounded experimental filesystem, variable and form snapshot metadata.
//! Dense serialization remains filesystem-only and refuses other record families.
//! Public activation, complete form mutation and mirror semantics remain separate.
use super::*;
use crate::crypto::strong_checksum;
use crate::file_format::allocation_map::Extent;
use crate::file_format::publication_anchor::shared::ownership::{Graph, Span, Vacant, VacantKind};
use crate::file_format::publication_anchor::REGION_LEN;
use crate::page_buffer::ZeroizingBytes;
use std::collections::BTreeSet;
pub(super) mod forms;
mod nodes;
pub(super) mod overflow;
pub(super) use nodes::Metadata;
const MAX_BODY: usize = 65536;
const MAX_MODEL_FILES: usize = 1024;
const MAX_FRAGMENTS: usize = 4096;
const MAX_MEMBERS: usize = 4096;
pub(super) struct Fragment {
    pub descriptor: Descriptor,
    pub pack: usize,
    pub relative: usize,
    pub digest: [u8; 32],
}
pub(super) struct File {
    pub path: Zeroizing<Vec<u8>>,
    pub permissions: u32,
    pub info: FileInfo,
    pub fragments: Vec<Fragment>,
}
pub(super) struct Pack {
    pub extent: Extent,
    pub padding_digest: [u8; 32],
    pub used: usize,
}
pub(super) struct Variable {
    pub name: crate::VariableName,
    pub layout: crate::file_format::secure_segments::Layout,
}
pub(super) struct Catalogue {
    legacy: bool,
    typed: bool,
    pub nodes: Vec<nodes::Node>,
    pub variables: Vec<Variable>,
    pub forms: forms::Forms,
    pub vacant: Vec<Vacant>,
    pub files: Vec<File>,
    pub packs: Vec<Pack>,
}
struct Cursor<'a>(&'a [u8]);
impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if n > self.0.len() {
            return Err(Error::CorruptRecord);
        }
        let (value, rest) = self.0.split_at(n);
        self.0 = rest;
        Ok(value)
    }
    fn uint(&mut self) -> Result<u64> {
        let mut value = 0u64;
        for byte in 0..10 {
            let n = self.take(1)?[0];
            if byte == 9 && n > 1 {
                return Err(Error::CorruptRecord);
            }
            value |= u64::from(n & 127) << (byte * 7);
            if n & 128 == 0 {
                if byte > 0 && n == 0 {
                    return Err(Error::CorruptRecord);
                }
                return Ok(value);
            }
        }
        Err(Error::CorruptRecord)
    }
    fn count(&mut self, max: usize) -> Result<usize> {
        let n = self.uint()?;
        if n > max as u64 {
            return Err(Error::CorruptRecord);
        }
        Ok(n as usize)
    }
}
impl Catalogue {
    /// Build directly from the shared audited repacker; no dense body is created.
    /// Keep explicit experimental count limits even when metadata uses a tree.
    pub(super) fn from_repacked(
        files: Vec<File>,
        packs: Vec<Pack>,
        codec: &Codec,
        sealed: u64,
    ) -> Result<Self> {
        if files.len() > MAX_MODEL_FILES
            || packs.len() > MAX_FRAGMENTS
            || files.iter().map(|file| file.fragments.len()).sum::<usize>() > MAX_FRAGMENTS
        {
            return Err(Error::SecurityLimitExceeded(
                "fresh tree repacking count budget".into(),
            ));
        }
        let mut catalogue = Self {
            variables: Vec::new(),
            forms: forms::Forms::default(),
            legacy: false,
            typed: false,
            nodes: Vec::new(),
            vacant: Vec::new(),
            files,
            packs,
        };
        catalogue.validate_fragments(codec, sealed)?;
        Ok(catalogue)
    }

    pub fn decode(body: &[u8], codec: &Codec, sealed: u64) -> Result<Self> {
        if body.len() > MAX_BODY || sealed < REGION_LEN as u64 {
            return Err(Error::CorruptRecord);
        }
        let mut cursor = Cursor(body);
        let (legacy, typed) = match cursor.take(8)? {
            b"RV4COST1" => (true, false),
            b"RV4DENS2" => (false, false),
            b"RV4DENS3" => (false, true),
            _ => return Err(Error::CorruptRecord),
        };
        let count = cursor.count(MAX_MODEL_FILES)?;
        // Minimum file header exceeds 50 bytes; reject counts before reserving.
        if count > cursor.0.len() / 50 {
            return Err(Error::CorruptRecord);
        }
        let mut files: Vec<File> = Vec::with_capacity(count);
        let mut ids = BTreeSet::new();
        let mut fragment_count = 0;
        for _ in 0..count {
            if cursor.take(1)? != [1] {
                return Err(Error::CorruptRecord);
            }
            let permissions = u32::from_le_bytes(cursor.take(4)?.try_into().unwrap());
            if crate::security::validate_permissions(permissions).is_err() {
                return Err(Error::CorruptRecord);
            }
            let path_len = cursor.count(4096)?;
            let path = cursor.take(path_len)?;
            if !valid_path(path)
                || files
                    .last()
                    .is_some_and(|file| file.path.as_slice() >= path)
            {
                return Err(Error::CorruptRecord);
            }
            let path = Zeroizing::new(path.to_vec());
            let id = cursor.take(16)?.try_into().unwrap();
            if !ids.insert(id) {
                return Err(Error::CorruptRecord);
            }
            let len = cursor.uint()?;
            let unit = cursor.count(MAX_LOGICAL)? as u32;
            let digest = cursor.take(32)?.try_into().unwrap();
            let info = FileInfo {
                id,
                len,
                unit,
                digest,
            };
            // Reuse existing unit/identity/empty-file digest constraints.
            let info = FileInfo::decode(&info.encode(), codec)?;
            let n = cursor.count(MAX_FRAGMENTS - fragment_count)?;
            if n as u64 != info.count() || n > cursor.0.len() / 43 {
                return Err(Error::CorruptRecord);
            }
            fragment_count += n;
            let mut fragments = Vec::with_capacity(n);
            for ordinal in 0..n {
                let pack = cursor.count(MAX_FRAGMENTS - 1)?;
                let relative = cursor.count(320 * 1024)?;
                let mut descriptor = [0; 64];
                descriptor[..8].copy_from_slice(b"RV4DAT01");
                descriptor[8..24].copy_from_slice(&info.id);
                descriptor[24..32].copy_from_slice(&(ordinal as u64).to_le_bytes());
                let offset = ordinal as u64 * info.unit as u64;
                descriptor[32..40].copy_from_slice(&offset.to_le_bytes());
                let logical = (info.len - offset).min(info.unit as u64) as u32;
                descriptor[40..44].copy_from_slice(&logical.to_le_bytes());
                descriptor[44..53].copy_from_slice(cursor.take(9)?);
                fragments.push(Fragment {
                    descriptor: Descriptor::decode(&descriptor)?,
                    pack,
                    relative,
                    digest: cursor.take(32)?.try_into().unwrap(),
                });
            }
            files.push(File {
                path,
                permissions,
                info,
                fragments,
            });
        }
        let nodes = if typed {
            nodes::decode(&mut cursor, files.len())?
        } else {
            Vec::new()
        };
        if typed {
            nodes::validate(&files, &nodes)?;
        }
        let count = cursor.count(MAX_FRAGMENTS)?;
        if count > cursor.0.len() / 73 {
            return Err(Error::CorruptRecord);
        }
        let mut packs = Vec::with_capacity(count);
        let mut position = REGION_LEN as u64;
        for _ in 0..count {
            let start = u64::from_le_bytes(cursor.take(8)?.try_into().unwrap());
            let len = cursor.uint()?;
            let digest = cursor.take(32)?.try_into().unwrap();
            let padding_digest = cursor.take(32)?.try_into().unwrap();
            if (legacy && start != position) || start < position || len == 0 || len > 320 * 1024 {
                return Err(Error::CorruptRecord);
            }
            position = start.checked_add(len).ok_or(Error::CorruptRecord)?;
            if position > sealed {
                return Err(Error::CorruptRecord);
            }
            packs.push(Pack {
                extent: Extent { start, len, digest },
                padding_digest,
                used: 0,
            });
        }
        let count = cursor.count(8192)?;
        if count > cursor.0.len() / 10 || (legacy && (count != 0 || position != sealed)) {
            return Err(Error::CorruptRecord);
        }
        let mut vacant: Vec<Vacant> = Vec::with_capacity(count);
        for _ in 0..count {
            let kind = match cursor.take(1)?[0] {
                0 => VacantKind::Free,
                1 => VacantKind::Pending,
                _ => return Err(Error::CorruptRecord),
            };
            let start = u64::from_le_bytes(cursor.take(8)?.try_into().unwrap());
            let len = cursor.uint()?;
            if len == 0
                || start.checked_add(len).is_none_or(|end| end > sealed)
                || vacant
                    .last()
                    .is_some_and(|old| old.span.start + old.span.len > start)
            {
                return Err(Error::CorruptRecord);
            }
            vacant.push(Vacant {
                span: Span { start, len },
                kind,
            });
        }
        if !cursor.0.is_empty() {
            return Err(Error::CorruptRecord);
        }
        let mut catalogue = Self {
            variables: Vec::new(),
            forms: forms::Forms::default(),
            legacy,
            typed,
            nodes,
            vacant,
            files,
            packs,
        };
        catalogue.validate_fragments(codec, sealed)?;
        Ok(catalogue)
    }
    pub(super) fn validate_fragments(&mut self, codec: &Codec, sealed: u64) -> Result<()> {
        let mut coverage: Vec<Vec<(usize, usize)>> =
            (0..self.packs.len()).map(|_| Vec::new()).collect();
        for file in &self.files {
            for fragment in &file.fragments {
                let pack = self.packs.get(fragment.pack).ok_or(Error::CorruptRecord)?;
                let end = fragment
                    .relative
                    .checked_add(fragment.descriptor.stored_len())
                    .ok_or(Error::CorruptRecord)?;
                if end as u64 > pack.extent.len {
                    return Err(Error::CorruptRecord);
                }
                codec.validate_extent(
                    Extent {
                        start: pack.extent.start + fragment.relative as u64,
                        len: fragment.descriptor.stored_len() as u64,
                        digest: fragment.digest,
                    },
                    sealed,
                    &fragment.descriptor,
                )?;
                let spans = &mut coverage[fragment.pack];
                if spans.len() == MAX_MEMBERS {
                    return Err(Error::CorruptRecord);
                }
                spans.push((fragment.relative, end));
            }
        }
        for (pack, mut spans) in self.packs.iter_mut().zip(coverage) {
            if spans.is_empty() {
                return Err(Error::CorruptRecord);
            }
            spans.sort_unstable();
            let mut used = 0;
            for (start, end) in spans {
                if start != used {
                    return Err(Error::CorruptRecord);
                }
                used = end;
            }
            if used > codec.logical_unit(MAX_LOGICAL)? + 28
                || codec.pack_padded_len(used)? as u64 != pack.extent.len
            {
                return Err(Error::CorruptRecord);
            }
            pack.used = used;
        }
        Ok(())
    }
    pub(super) fn graph(&self, anchor: &Anchor) -> Result<Graph> {
        let mut packs: Vec<_> = self.packs.iter().map(|pack| pack.extent).collect();
        packs.extend(
            self.variables
                .iter()
                .flat_map(|v| v.layout.extents.iter().copied()),
        );
        packs.extend(
            self.forms
                .texts()
                .into_iter()
                .flat_map(|text| text.extents.iter().copied()),
        );
        packs.sort_by_key(|extent| extent.start);
        if self.legacy {
            Graph::fresh_files(anchor, &packs)
        } else {
            Graph::derive(anchor, &packs, &self.vacant)
        }
    }
    /// Convert a successfully authenticated legacy fresh catalogue in memory.
    /// Persisting the new representation still requires the COW journal protocol.
    pub(super) fn upgrade(&mut self, anchor: &Anchor) -> Result<()> {
        self.graph(anchor)?;
        if self.legacy {
            if anchor.keys == publication::RootRef::default() {
                self.vacant = [0, publication::FAILURE_REGION]
                    .into_iter()
                    .map(|bank| Vacant {
                        span: Span {
                            start: bank + 12288,
                            len: 4096,
                        },
                        kind: VacantKind::Free,
                    })
                    .collect();
            }
            self.legacy = false;
        }
        Ok(())
    }
    pub(super) fn encode(&self, codec: &Codec, sealed: u64) -> Result<ZeroizingBytes> {
        if !self.variables.is_empty() || !self.forms.is_empty() {
            return Err(Error::SecurityLimitExceeded(
                "variables and forms require typed tree metadata".into(),
            ));
        }
        if self.legacy {
            return Err(Error::InvalidInput(
                "upgrade catalogue before encoding ownership states".into(),
            ));
        }
        let mut out = Writer(ZeroizingBytes::new(Vec::with_capacity(MAX_BODY)));
        out.put(if self.typed { b"RV4DENS3" } else { b"RV4DENS2" })?;
        out.uint(self.files.len() as u64)?;
        for file in &self.files {
            out.put(&[1])?;
            out.put(&file.permissions.to_le_bytes())?;
            out.uint(file.path.len() as u64)?;
            out.put(&file.path)?;
            out.put(&file.info.id)?;
            out.uint(file.info.len)?;
            out.uint(file.info.unit as u64)?;
            out.put(&file.info.digest)?;
            out.uint(file.fragments.len() as u64)?;
            for (ordinal, fragment) in file.fragments.iter().enumerate() {
                let d = &fragment.descriptor;
                let offset = (ordinal as u64)
                    .checked_mul(file.info.unit as u64)
                    .ok_or(Error::CorruptRecord)?;
                if d.object != file.info.id
                    || d.ordinal != ordinal as u64
                    || d.offset != offset
                    || offset >= file.info.len
                    || d.logical_len as u64 != (file.info.len - offset).min(file.info.unit as u64)
                {
                    return Err(Error::CorruptRecord);
                }
                out.uint(fragment.pack as u64)?;
                out.uint(fragment.relative as u64)?;
                out.put(&d.encode()[44..53])?;
                out.put(&fragment.digest)?;
            }
        }
        if self.typed {
            nodes::encode(&self.nodes, &mut out)?;
        }
        out.uint(self.packs.len() as u64)?;
        for pack in &self.packs {
            out.put(&pack.extent.start.to_le_bytes())?;
            out.uint(pack.extent.len)?;
            out.put(&pack.extent.digest)?;
            out.put(&pack.padding_digest)?;
        }
        out.uint(self.vacant.len() as u64)?;
        for entry in &self.vacant {
            out.put(&[match entry.kind {
                VacantKind::Free => 0,
                VacantKind::Pending => 1,
            }])?;
            out.put(&entry.span.start.to_le_bytes())?;
            out.uint(entry.span.len)?;
        }
        // Reject inconsistent typed edits before producing a private envelope.
        Self::decode(&out.0, codec, sealed)?;
        Ok(out.0)
    }
    pub fn verify_padding(&self, storage: &impl Storage, codec: &Codec) -> Result<()> {
        for pack in &self.packs {
            let n = pack.extent.len as usize - pack.used;
            if n == 0 {
                if pack.padding_digest != strong_checksum(&[]) {
                    return Err(Error::CorruptRecord);
                }
                continue;
            }
            let bytes =
                ZeroizingBytes::new(storage.read_at(pack.extent.start + pack.used as u64, n)?);
            if bytes.len() != n || strong_checksum(&bytes) != pack.padding_digest {
                return Err(Error::CorruptRecord);
            }
            if n != 0 {
                codec.verify_pack_padding(&bytes)?;
            }
        }
        Ok(())
    }
}

// Reserve the complete bounded private buffer once; never reallocate secret bytes.
struct Writer(ZeroizingBytes);
impl Writer {
    fn put(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() > MAX_BODY - self.0.len() {
            return Err(Error::SecurityLimitExceeded(
                "bounded dense catalogue needs overflow".into(),
            ));
        }
        self.0.extend_from_slice(bytes);
        Ok(())
    }
    fn uint(&mut self, mut n: u64) -> Result<()> {
        let mut bytes = [0u8; 10];
        let mut len = 0;
        while n >= 128 {
            bytes[len] = n as u8 | 128;
            len += 1;
            n >>= 7;
        }
        bytes[len] = n as u8;
        self.put(&bytes[..len + 1])
    }
}
