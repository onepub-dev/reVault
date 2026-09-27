//! Fresh file-only shared-control image for correctness/size comparison. No
//! mutation, public adapter or migration contract is exposed. Caller supplies a
//! stable source snapshot/read lock; destination must be a fresh owned backend.
use super::dense_catalogue::Catalogue;
use super::*;
use crate::file_format::allocation_map::Extent;
use crate::file_format::publication_anchor::{shared, REGION_LEN};
use crate::page_buffer::ZeroizingBytes;
pub(super) struct Image<S: Storage> {
    pub storage: S,
    anchor: Anchor,
    catalogue: Catalogue,
    codec: Codec,
}
impl<S: Storage> Image<S> {
    pub fn open(
        storage: S,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        key: Option<&[u8]>,
    ) -> Result<Self> {
        let (anchor, body) = shared::open_private(&storage, archive, mode, authority, key)?;
        let codec = Codec::shared_packed(archive, mode, key)?;
        let catalogue = Catalogue::decode(&body, &codec, anchor.sealed_len)?;
        catalogue.graph(&anchor)?.verify_reclaimed(&storage)?;
        catalogue.verify_padding(&storage, &codec)?;
        let mut image = Self {
            storage,
            anchor,
            catalogue,
            codec,
        };
        if mode.plaintext() && mode.signed() {
            image.verify_all()?;
        }
        Ok(image)
    }
    pub fn open_credential(
        storage: S,
        archive: LockboxId,
        mode: FormatMode,
        owner: Option<&crate::OwnerSigningPublicKey>,
        credential: publication::bootstrap::Credential<'_>,
        slot: Option<u64>,
    ) -> Result<Self> {
        let opened = shared::credential_open(&storage, archive, mode, owner, credential, slot)?;
        opened.key.with_bytes(|key| {
            let authority = owner.map_or(Authority::Symmetric(key), Authority::Owner);
            let image = Self::open(storage, archive, mode, &authority, Some(key))?;
            if image.anchor != opened.anchor {
                return Err(Error::CorruptRecord);
            }
            Ok(image)
        })?
    }
    pub fn filesystem_metadata(&self) -> Result<Vec<super::dense_catalogue::Metadata>> {
        self.catalogue.filesystem_metadata()
    }
    pub fn info(&self, path: &[u8]) -> Result<Option<FileInfo>> {
        Ok(self
            .catalogue
            .files
            .binary_search_by(|file| file.path.as_slice().cmp(path))
            .ok()
            .map(|index| self.catalogue.files[index].info.clone()))
    }
    pub fn read_range(
        &mut self,
        path: &[u8],
        offset: u64,
        len: u64,
        mut visitor: impl FnMut(&[u8]) -> Result<()>,
    ) -> Result<()> {
        let index = self
            .catalogue
            .files
            .binary_search_by(|file| file.path.as_slice().cmp(path))
            .map_err(|_| Error::InvalidInput("file not found".into()))?;
        let file = &self.catalogue.files[index];
        let end = offset
            .checked_add(len)
            .ok_or(Error::InvalidInput("range overflow".into()))?;
        if end > file.info.len {
            return Err(Error::InvalidInput("range outside file".into()));
        }
        if len == 0 {
            return Ok(());
        }
        let first = offset / file.info.unit as u64;
        let after = end.div_ceil(file.info.unit as u64);
        for ordinal in first..after {
            let fragment = &file.fragments[ordinal as usize];
            let pack = &self.catalogue.packs[fragment.pack];
            let extent = Extent {
                start: pack.extent.start + fragment.relative as u64,
                len: fragment.descriptor.stored_len() as u64,
                digest: fragment.digest,
            };
            let bytes = self.codec.load(
                &self.storage,
                extent,
                self.anchor.sealed_len,
                &fragment.descriptor,
            )?;
            let logical = ordinal * file.info.unit as u64;
            let start = offset.saturating_sub(logical) as usize;
            let stop = (end - logical).min(fragment.descriptor.logical_len as u64) as usize;
            visitor(&bytes[start..stop])?;
        }
        Ok(())
    }
    pub(super) fn verify_all(&mut self) -> Result<()> {
        for index in 0..self.catalogue.files.len() {
            let file = &self.catalogue.files[index];
            let path = file.path.clone();
            let len = file.info.len;
            let expected = file.info.digest;
            let mut hash = Sha256::new();
            self.read_range(&path, 0, len, |bytes| {
                hash.update(bytes);
                Ok(())
            })?;
            if <[u8; 32]>::from(hash.finalize()) != expected {
                return Err(Error::CorruptRecord);
            }
        }
        Ok(())
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn from_candidate<S: Storage, T: Storage>(
    source: &mut Files<S>,
    mut destination: T,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    slots: &[crate::key_slot::KeySlot],
) -> Result<T> {
    if destination.len()? != 0 {
        return Err(Error::InvalidInput(
            "shared image destination must be empty".into(),
        ));
    }
    if source.anchor.keys != publication::RootRef::default() {
        return Err(Error::InvalidInput(
            "shared comparison image cannot translate the source access tree".into(),
        ));
    }
    let selected = publication::select(
        &source.storage,
        source.anchor.archive,
        source.anchor.mode,
        authority,
    )?;
    if selected.commitment != source.anchor.commitment()? {
        return Err(Error::CorruptRecord);
    }
    let result = (|| {
        destination.append(&vec![0; REGION_LEN])?;
        let mut body = None;
        let projected = super::cost_model::project_into(
            source,
            |extent, bytes| {
                if destination.append(bytes)? != extent.start {
                    return Err(Error::CorruptRecord);
                }
                Ok(())
            },
            |bytes| {
                body = Some(ZeroizingBytes::new(bytes.to_vec()));
                Ok(())
            },
        )?;
        let body = body.ok_or_else(|| {
            Error::SecurityLimitExceeded("shared image needs catalogue overflow".into())
        })?;
        if projected["projected_compacted_bytes"].as_u64() != Some(destination.len()?) {
            return Err(Error::CorruptRecord);
        }
        let archive = source.anchor.archive;
        let mode = source.anchor.mode;
        // Validate new placement and the supplied key before publication. Use a
        // borrowed storage view so this does not clone a potentially large image.
        let codec = Codec::shared_packed(archive, mode, key)?;
        let catalogue = Catalogue::decode(&body, &codec, destination.len()?)?;
        catalogue.verify_padding(&destination, &codec)?;
        let mut unpublished = Image {
            storage: allocation::compaction::View(&destination),
            anchor: Anchor {
                sealed_len: destination.len()?,
                ..source.anchor.clone()
            },
            catalogue,
            codec,
        };
        unpublished.verify_all()?;
        drop(unpublished);
        if publication::select(&source.storage, archive, mode, authority)?.commitment
            != selected.commitment
        {
            return Err(Error::CorruptRecord);
        }
        shared::initialize(
            &mut destination,
            archive,
            mode,
            authority,
            signer,
            key,
            &body,
            slots,
        )?;
        let mut reopened = Image::open(
            allocation::compaction::View(&destination),
            archive,
            mode,
            authority,
            key,
        )?;
        reopened.verify_all()?;
        if publication::select(&source.storage, archive, mode, authority)?.commitment
            != selected.commitment
        {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    })();
    if let Err(error) = result {
        if let Err(cleanup) = allocation::compaction::discard(&mut destination) {
            return Err(Error::InvalidInput(format!(
                "shared image creation failed ({error}); destination cleanup failed ({cleanup})"
            )));
        }
        return Err(error);
    }
    Ok(destination)
}

pub(super) struct SalvageReport {
    pub generation: u64,
    pub complete: u64,
    pub incomplete: u64,
}
/// Caller retains a stable snapshot/read lock and stages the entire sink batch
/// until this returns Ok. The sink's existing per-file finish contract applies.
/// Missing selected catalogue authority is fatal; no old-generation scan occurs.
pub(super) fn salvage<S: Storage>(
    storage: &S,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
    sink: &mut impl super::recovery::Sink,
) -> Result<SalvageReport> {
    let (anchor, body) = shared::salvage_private(storage, archive, mode, authority, key)?;
    let mut codec = Codec::shared_packed(archive, mode, key)?;
    let catalogue = Catalogue::decode(&body, &codec, anchor.sealed_len)?;
    catalogue.graph(&anchor)?;
    if !catalogue.nodes.is_empty() {
        return Err(Error::InvalidOperation(
            "typed node recovery requires a node-aware sink".into(),
        ));
    }
    let actual = storage.len()?;
    let mut report = SalvageReport {
        generation: anchor.generation,
        complete: 0,
        incomplete: 0,
    };
    for file in &catalogue.files {
        sink.begin(&file.path, file.info.len)?;
        let mut valid = true;
        let mut hash = Sha256::new();
        for fragment in &file.fragments {
            let pack = &catalogue.packs[fragment.pack];
            let extent = Extent {
                start: pack.extent.start + fragment.relative as u64,
                len: fragment.descriptor.stored_len() as u64,
                digest: fragment.digest,
            };
            if extent
                .start
                .checked_add(extent.len)
                .is_none_or(|end| end > actual)
            {
                valid = false;
                break;
            }
            match codec.load(storage, extent, anchor.sealed_len, &fragment.descriptor) {
                Ok(bytes) => {
                    sink.data(&bytes)?;
                    hash.update(bytes.as_slice());
                }
                Err(Error::CorruptRecord | Error::CorruptHeader | Error::Truncated) => {
                    valid = false;
                    break;
                }
                Err(error) => return Err(error),
            }
        }
        valid &= <[u8; 32]>::from(hash.finalize()) == file.info.digest;
        sink.finish(valid)?;
        if valid {
            report.complete += 1;
        } else {
            report.incomplete += 1;
        }
    }
    Ok(report)
}
