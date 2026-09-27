//! Bounded parser for the file-only feasibility catalogue. This is not the full
//! public record model: directories, links, variables, forms and mirror ownership
//! still need explicit semantics. All paths and encoded bodies remain wipeable.
use super::*;
use crate::crypto::strong_checksum;
use crate::file_format::allocation_map::Extent;
use crate::file_format::publication_anchor::REGION_LEN;
use crate::page_buffer::ZeroizingBytes;
use std::collections::BTreeSet;
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
pub(super) struct Catalogue {
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
    pub fn decode(body: &[u8], codec: &Codec, sealed: u64) -> Result<Self> {
        if body.len() > MAX_BODY || sealed < REGION_LEN as u64 {
            return Err(Error::CorruptRecord);
        }
        let mut cursor = Cursor(body);
        if cursor.take(8)? != b"RV4COST1" {
            return Err(Error::CorruptRecord);
        }
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
            if permissions & !0o7777 != 0 {
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
            if start != position || len == 0 || len > 320 * 1024 {
                return Err(Error::CorruptRecord);
            }
            position = position.checked_add(len).ok_or(Error::CorruptRecord)?;
            if position > sealed {
                return Err(Error::CorruptRecord);
            }
            packs.push(Pack {
                extent: Extent { start, len, digest },
                padding_digest,
                used: 0,
            });
        }
        // This fresh-image representation has no free/pending/overflow state.
        if cursor.uint()? != 0 || !cursor.0.is_empty() || position != sealed {
            return Err(Error::CorruptRecord);
        }
        let mut coverage: Vec<Vec<(usize, usize)>> = (0..packs.len()).map(|_| Vec::new()).collect();
        for file in &files {
            for fragment in &file.fragments {
                let pack = packs.get(fragment.pack).ok_or(Error::CorruptRecord)?;
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
        for (pack, mut spans) in packs.iter_mut().zip(coverage) {
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
        Ok(Self { files, packs })
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
