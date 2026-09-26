//! File adapter for the architecture comparison, not a production file format.
//! Per-extent index records keep GB files out of a single oversized descriptor.
use super::allocation_map::{self as allocation, OwnedRecord, Snapshot, Transaction};
use super::authenticated_index::{Entry, Index};
use super::data_extent::{Codec, Descriptor, MAX_LOGICAL};
use super::publication_anchor::{self as publication, Anchor, Authority};
use crate::creation_options::FormatMode;
use crate::storage::Storage;
use crate::{Error, LockboxId, OwnerSigningKeyPair, Result};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Read;
use zeroize::Zeroizing;

const FILE: u8 = 10;
const CHUNK: u8 = 11;
const MAX_FILES: usize = 100_000;
const MAX_ENTRIES: usize = 1_000_000;
const MAGIC: &[u8; 8] = b"RV4FIL01";

pub(crate) struct Input<R: Read> {
    pub path: Vec<u8>,
    pub reader: R,
}
#[derive(Clone, Debug)]
pub(crate) struct FileInfo {
    id: [u8; 16],
    pub len: u64,
    unit: u32,
    digest: [u8; 32],
}
impl FileInfo {
    fn count(&self) -> u64 {
        self.len.div_ceil(self.unit as u64)
    }
    fn encode(&self) -> [u8; 72] {
        let mut out = [0; 72];
        out[..8].copy_from_slice(MAGIC);
        out[8..24].copy_from_slice(&self.id);
        out[24..32].copy_from_slice(&self.len.to_le_bytes());
        out[32..36].copy_from_slice(&self.unit.to_le_bytes());
        out[40..].copy_from_slice(&self.digest);
        out
    }
    fn decode(bytes: &[u8], codec: &Codec) -> Result<Self> {
        if bytes.len() != 72 || &bytes[..8] != MAGIC || bytes[36..40] != [0; 4] {
            return Err(Error::CorruptRecord);
        }
        let info = Self {
            id: bytes[8..24].try_into().unwrap(),
            len: u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
            unit: u32::from_le_bytes(bytes[32..36].try_into().unwrap()),
            digest: bytes[40..].try_into().unwrap(),
        };
        if info.id == [0; 16]
            || ![
                codec.logical_unit(65536)? as u32,
                codec.logical_unit(MAX_LOGICAL)? as u32,
            ]
            .contains(&info.unit)
            || info.count() > MAX_ENTRIES as u64
            || (info.len == 0 && info.digest != <[u8; 32]>::from(Sha256::digest([])))
        {
            return Err(Error::CorruptRecord);
        }
        Ok(info)
    }
    fn validate_chunk(&self, ordinal: u64, descriptor: &Descriptor) -> Result<()> {
        let offset = ordinal
            .checked_mul(self.unit as u64)
            .ok_or(Error::CorruptRecord)?;
        if ordinal >= self.count()
            || descriptor.object != self.id
            || descriptor.ordinal != ordinal
            || descriptor.offset != offset
            || descriptor.logical_len as u64 != (self.len - offset).min(self.unit as u64)
        {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }
}
fn chunk_key(id: [u8; 16], ordinal: u64) -> [u8; 24] {
    let mut key = [0; 24];
    key[..16].copy_from_slice(&id);
    key[16..].copy_from_slice(&ordinal.to_be_bytes());
    key
}
fn valid_path(path: &[u8]) -> bool {
    !path.is_empty() && path.len() <= 4096 && !path.contains(&0)
}

pub(crate) struct Files<S: Storage> {
    storage: S,
    anchor: Anchor,
    index: Index,
    codec: Codec,
}
impl<S: Storage> Files<S> {
    /// Full bulk construction for the A/B/C comparison. Failure before publication
    /// explicitly aborts prepared allocations. Existing archives use a separate
    /// update path; this entry point requires an empty destination.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create<R: Read>(
        mut storage: S,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        signer: Option<&OwnerSigningKeyPair>,
        key: Option<&[u8]>,
        unit: usize,
        inputs: impl IntoIterator<Item = Input<R>>,
    ) -> Result<S> {
        let codec = Codec::new(archive, mode, key)?;
        let logical_unit = codec.logical_unit(unit)?;
        allocation::create_empty(&mut storage, archive, mode, authority, signer, key)?;
        let mut tx = Transaction::begin(storage, archive, mode, authority, key)?;
        let result = (|| {
            let mut entries = Vec::new();
            let mut paths = std::collections::BTreeSet::new();
            let mut ids = std::collections::BTreeSet::new();
            let mut input_bytes = Zeroizing::new(vec![0; logical_unit]);
            for input in inputs {
                let Input { path, mut reader } = input;
                let path = Zeroizing::new(path);
                if paths.len() == MAX_FILES
                    || !valid_path(&path)
                    || !paths.insert(<[u8; 32]>::from(Sha256::digest(&path)))
                {
                    return Err(Error::InvalidInput(
                        "invalid, duplicate or excessive candidate file paths".into(),
                    ));
                }
                let mut id = [0; 16];
                getrandom::fill(&mut id).map_err(|e| Error::Io(e.to_string()))?;
                if id == [0; 16] || !ids.insert(id) {
                    return Err(Error::InvalidInput(
                        "duplicate candidate object identity".into(),
                    ));
                }
                let mut info = FileInfo {
                    id,
                    len: 0,
                    unit: logical_unit as u32,
                    digest: [0; 32],
                };
                let mut ordinal = 0;
                let mut hash = Sha256::new();
                loop {
                    let mut filled = 0;
                    while filled < logical_unit {
                        match reader.read(&mut input_bytes[filled..]) {
                            Ok(0) => break,
                            Ok(n) => {
                                filled += n;
                            }
                            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {
                                continue
                            }
                            Err(error) => return Err(Error::Io(error.to_string())),
                        }
                    }
                    if filled == 0 {
                        break;
                    }
                    if entries.len() >= MAX_ENTRIES - 1 {
                        return Err(Error::SecurityLimitExceeded(
                            "candidate file index limit".into(),
                        ));
                    }
                    let (descriptor, stored) =
                        codec.encode(id, ordinal, info.len, &input_bytes[..filled])?;
                    let extent = tx.append_encoded_extent(&stored)?;
                    entries.push(Entry::new(
                        CHUNK,
                        &chunk_key(id, ordinal),
                        &OwnedRecord::encode(&descriptor.encode(), &[extent])?,
                    )?);
                    hash.update(&input_bytes[..filled]);
                    info.len = info
                        .len
                        .checked_add(filled as u64)
                        .ok_or(Error::CorruptRecord)?;
                    ordinal += 1;
                    if filled < logical_unit {
                        break;
                    }
                }
                info.digest = hash.finalize().into();
                if entries.len() == MAX_ENTRIES {
                    return Err(Error::SecurityLimitExceeded(
                        "candidate file index limit".into(),
                    ));
                }
                entries.push(Entry::new(
                    FILE,
                    &path,
                    &OwnedRecord::encode(&info.encode(), &[])?,
                )?);
            }
            entries.sort_by(|a, b| {
                (a.namespace, a.key.as_slice()).cmp(&(b.namespace, b.key.as_slice()))
            });
            tx.replace_all_sorted(entries.into_iter().map(Ok))?;
            Ok(())
        })();
        if let Err(error) = result {
            tx.abort(authority)?;
            return Err(error);
        }
        tx.commit(authority, signer).map(|(storage, _)| storage)
    }
    pub(crate) fn open(
        storage: S,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        key: Option<&[u8]>,
    ) -> Result<Self> {
        let anchor = publication::select(&storage, archive, mode, authority)?.anchor;
        let index = Index::new(archive, mode, key)?;
        let codec = Codec::new(archive, mode, key)?;
        Snapshot::inspect(&storage, &anchor, &index)?;
        let mut files = Self {
            storage,
            anchor,
            index,
            codec,
        };
        files.audit()?;
        Ok(files)
    }
    fn audit(&mut self) -> Result<()> {
        struct State {
            info: FileInfo,
            next: u64,
            hash: Option<Sha256>,
        }
        let eager = self.anchor.mode.signed() && self.anchor.mode.plaintext();
        let mut files = BTreeMap::<[u8; 16], State>::new();
        let mut count = 0;
        self.index.visit(
            &self.storage,
            self.anchor.index,
            self.anchor.sealed_len,
            |entry| {
                count += 1;
                if count > MAX_ENTRIES {
                    return Err(Error::CorruptRecord);
                }
                let record = OwnedRecord::decode(&entry.value)?;
                match entry.namespace {
                    FILE => {
                        if !valid_path(&entry.key)
                            || !record.extents.is_empty()
                            || files.len() == MAX_FILES
                        {
                            return Err(Error::CorruptRecord);
                        }
                        let info = FileInfo::decode(&record.metadata, &self.codec)?;
                        if files
                            .insert(
                                info.id,
                                State {
                                    info,
                                    next: 0,
                                    hash: eager.then(Sha256::new),
                                },
                            )
                            .is_some()
                        {
                            return Err(Error::CorruptRecord);
                        }
                    }
                    CHUNK => {
                        if entry.key.len() != 24 || record.extents.len() != 1 {
                            return Err(Error::CorruptRecord);
                        }
                        let id: [u8; 16] = entry.key[..16].try_into().unwrap();
                        let ordinal = u64::from_be_bytes(entry.key[16..].try_into().unwrap());
                        let state = files.get_mut(&id).ok_or(Error::CorruptRecord)?;
                        if ordinal != state.next {
                            return Err(Error::CorruptRecord);
                        }
                        let descriptor = Descriptor::decode(&record.metadata)?;
                        state.info.validate_chunk(ordinal, &descriptor)?;
                        self.codec.validate_extent(
                            record.extents[0],
                            self.anchor.sealed_len,
                            &descriptor,
                        )?;
                        if let Some(hash) = &mut state.hash {
                            hash.update(&*self.codec.load(
                                &self.storage,
                                record.extents[0],
                                self.anchor.sealed_len,
                                &descriptor,
                            )?);
                        }
                        state.next += 1;
                    }
                    _ => return Err(Error::CorruptRecord),
                }
                Ok(())
            },
        )?;
        for state in files.into_values() {
            if state.next != state.info.count()
                || state
                    .hash
                    .is_some_and(|hash| <[u8; 32]>::from(hash.finalize()) != state.info.digest)
            {
                return Err(Error::CorruptRecord);
            }
        }
        Ok(())
    }
    pub(crate) fn info(&self, path: &[u8]) -> Result<Option<FileInfo>> {
        self.index
            .get(
                &self.storage,
                self.anchor.index,
                self.anchor.sealed_len,
                FILE,
                path,
            )?
            .map(|entry| {
                let record = OwnedRecord::decode(&entry.value)?;
                FileInfo::decode(&record.metadata, &self.codec)
            })
            .transpose()
    }
    /// Callback receives authenticated ranges bounded by the selected access unit.
    /// No file-sized output allocation, even for GB files or full extraction.
    pub(crate) fn read_range(
        &mut self,
        path: &[u8],
        offset: u64,
        len: u64,
        mut visitor: impl FnMut(&[u8]) -> Result<()>,
    ) -> Result<()> {
        let info = self
            .info(path)?
            .ok_or_else(|| Error::InvalidInput("candidate file does not exist".into()))?;
        let end = offset
            .checked_add(len)
            .filter(|end| *end <= info.len)
            .ok_or(Error::InvalidInput("candidate range outside file".into()))?;
        if len == 0 {
            return Ok(());
        }
        let first = offset / info.unit as u64;
        let after = (end - 1) / info.unit as u64 + 1;
        let first_key = chunk_key(info.id, first);
        let after_key = chunk_key(info.id, after);
        let mut next = first;
        // Traverse each selected metadata page once. Restarting a point lookup
        // for every extent repeatedly authenticates/decodes the same page.
        // No persistent cache or weakened membership check is introduced.
        self.index.visit_range(
            &self.storage,
            self.anchor.index,
            self.anchor.sealed_len,
            Some((CHUNK, &first_key)),
            Some((CHUNK, &after_key)),
            |entry| {
                if entry.namespace != CHUNK || entry.key.as_slice() != chunk_key(info.id, next) {
                    return Err(Error::CorruptRecord);
                }
                let record = OwnedRecord::decode(&entry.value)?;
                let descriptor = Descriptor::decode(&record.metadata)?;
                info.validate_chunk(next, &descriptor)?;
                if record.extents.len() != 1 {
                    return Err(Error::CorruptRecord);
                }
                let decoded = self.codec.load(
                    &self.storage,
                    record.extents[0],
                    self.anchor.sealed_len,
                    &descriptor,
                )?;
                let start = offset.saturating_sub(descriptor.offset) as usize;
                let stop = (end - descriptor.offset).min(descriptor.logical_len as u64) as usize;
                visitor(&decoded[start..stop])?;
                next += 1;
                Ok(())
            },
        )?;
        if next != after {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }
    pub(crate) fn into_storage(self) -> S {
        self.storage
    }
}

#[cfg(test)]
mod tests;

#[cfg(all(test, target_os = "linux"))]
mod resource_probe;
