//! Experimental typed filesystem adapter over the authenticated shared tree.
//! Fresh export preserves its source. Filesystem and guarded variable transactions
//! and form snapshots are test-only; public activation, complete form mutation
//! and migration remain separate.
use super::dense_catalogue::{Catalogue, Metadata};
use super::dense_image::Image;
use super::*;
use crate::file_format::publication_anchor::{shared, FAILURE_REGION, REGION_LEN};
use shared::tree::{self, Tree};
mod fresh;
pub(super) use fresh::from_candidate;
pub(super) mod forms;
mod mutation;
pub(super) mod variables;
pub(super) use mutation::{remove_files, update_files};

pub(super) struct TreeImage<S: Storage> {
    pub image: Image<S>,
    pub tree: Tree,
}
impl<S: Storage> TreeImage<S> {
    pub fn open(
        storage: S,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        key: Option<&[u8]>,
    ) -> Result<Self> {
        let codec = Codec::shared_packed(archive, mode, key)?;
        let (catalogue, tree) = Catalogue::with_tree(&codec, |visitor| {
            Tree::open_visit(&storage, archive, mode, authority, key, visitor)
        })?;
        catalogue.verify_padding(&storage, &codec)?;
        let mut image = Image {
            storage,
            anchor: tree.anchor.clone(),
            value_key: super::dense_image::value_key_for(&catalogue, mode, key)?,
            catalogue,
            codec,
        };
        if mode.plaintext() && mode.signed() {
            image.verify_all()?;
        }
        Ok(Self { image, tree })
    }
}

/// Called by the transition writer itself so no caller-supplied graph becomes
/// destructive-write authority. Re-authenticate storage and verify all payloads.
pub(crate) fn dense_base(
    storage: &impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<(Anchor, shared::ownership::Graph)> {
    let mut image = Image::open(
        allocation::compaction::View(storage),
        archive,
        mode,
        authority,
        key,
    )?;
    image.verify_all()?;
    let graph = image.catalogue.graph(&image.anchor)?;
    Ok((image.anchor, graph))
}

/// Recovery-only graph from an already owner-authenticated selected snapshot.
/// Do not require idle preparation or intact uncommitted vacant bytes here.
pub(crate) fn dense_recovery_graph(
    anchor: &Anchor,
    body: &[u8],
    key: Option<&[u8]>,
) -> Result<shared::ownership::Graph> {
    let codec = Codec::shared_packed(anchor.archive, anchor.mode, key)?;
    Catalogue::decode(body, &codec, anchor.sealed_len)?.graph(anchor)
}

pub(super) fn recover(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<()> {
    let (anchor, body) = shared::snapshot(storage, archive, mode, authority, key)?;
    let session = crate::file_format::preparation_journal::compact::session::OverflowSession::open(
        storage, archive, mode, key,
    )?;
    if body.starts_with(b"RV4TRE01")
        || (session.active() && session.base() == shared::commitment(&anchor)?)
    {
        tree::recover_update(storage, archive, mode, authority, key)
    } else {
        super::dense_update::recover(storage, archive, mode, authority, key).map(|_| ())
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn grow_dense(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    entries: &[Metadata],
) -> Result<bool> {
    let mut image = Image::open(
        allocation::compaction::View(&*storage),
        archive,
        mode,
        authority,
        key,
    )?;
    image.catalogue.set_tree_metadata(entries)?;
    let records = image.catalogue.tree_records()?;
    drop(image);
    tree::grow_dense_records(storage, archive, mode, authority, signer, key, records)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn return_inline(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
) -> Result<bool> {
    let (_, body) = shared::open_private(storage, archive, mode, authority, key)?;
    if !body.starts_with(b"RV4TRE01") {
        return super::dense_update::return_inline(storage, archive, mode, authority, signer, key);
    }
    let mut opened = TreeImage::open(
        allocation::compaction::View(&*storage),
        archive,
        mode,
        authority,
        key,
    )?;
    opened.image.verify_all()?;
    // Prove the target dense capacity before a possible relocation transaction.
    let sealed = opened
        .image
        .catalogue
        .packs
        .last()
        .map_or(REGION_LEN as u64, |pack| {
            pack.extent.start + pack.extent.len
        });
    opened.image.catalogue.vacant = opened.tree.graph.catalogue_vacant();
    opened
        .image
        .catalogue
        .vacant
        .retain(|entry| entry.span.start < sealed);
    for entry in &mut opened.image.catalogue.vacant {
        entry.span.len = entry.span.len.min(sealed - entry.span.start);
    }
    let body = opened
        .image
        .catalogue
        .dense_body_if_fits(&opened.image.codec, sealed)?
        .ok_or_else(|| Error::SecurityLimitExceeded("dense return capacity".into()))?;
    shared::encode_private(archive, mode, key, &body)?;
    if opened.tree.anchor.index.primary < sealed || opened.tree.anchor.index.mirror < sealed {
        let records = opened.image.catalogue.tree_records()?;
        drop(opened);
        tree::relocate_records(storage, archive, mode, authority, signer, key, records)?;
    } else {
        drop(opened);
    }
    let opened = TreeImage::open(
        allocation::compaction::View(&*storage),
        archive,
        mode,
        authority,
        key,
    )?;
    let anchor = opened.tree.anchor.clone();
    let mut catalogue = opened.image.catalogue;
    catalogue.vacant = opened.tree.graph.catalogue_vacant();
    super::dense_update::publish_catalogue(
        storage, archive, mode, authority, signer, key, anchor, catalogue, true,
    )?;
    Ok(true)
}

/// Read-only admission. A later cleanup error must never be mistaken for an
/// oversized catalogue and silently reported as a completed update.
pub(super) fn can_return_inline(
    storage: &impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<bool> {
    let mut opened = TreeImage::open(
        allocation::compaction::View(storage),
        archive,
        mode,
        authority,
        key,
    )?;
    let sealed = opened
        .image
        .catalogue
        .packs
        .last()
        .map_or(REGION_LEN as u64, |pack| {
            pack.extent.start + pack.extent.len
        });
    opened.image.catalogue.vacant = opened.tree.graph.catalogue_vacant();
    opened
        .image
        .catalogue
        .vacant
        .retain(|entry| entry.span.start < sealed);
    for entry in &mut opened.image.catalogue.vacant {
        entry.span.len = entry.span.len.min(sealed - entry.span.start);
    }
    let Some(body) = opened
        .image
        .catalogue
        .dense_body_if_fits(&opened.image.codec, sealed)?
    else {
        return Ok(false);
    };
    match shared::encode_private(archive, mode, key, &body) {
        Ok(_) => Ok(true),
        Err(Error::SecurityLimitExceeded(_)) => Ok(false),
        Err(error) => Err(error),
    }
}

/// Caller holds a stable snapshot and stages metadata/files until success.
/// Missing selected membership is fatal. Unavailable payloads are reported per
/// file; an unrelated corrupt file cannot replace owner publication authority.
pub(super) fn salvage_filesystem<S: Storage>(
    storage: &S,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
    sink: &mut impl super::dense_image::FilesystemSink,
) -> Result<super::dense_image::SalvageReport> {
    let codec = Codec::shared_packed(archive, mode, key)?;
    let (catalogue, tree) = Catalogue::with_tree(&codec, |visitor| {
        Tree::salvage_visit(storage, archive, mode, authority, key, visitor)
    })?;
    sink.metadata(&catalogue.filesystem_metadata()?)?;
    super::dense_image::salvage_catalogue(storage, &tree.anchor, &catalogue, codec, sink)
}

/// Exclusive writer access; validate complete typed semantics before the raw
/// transaction can mirror or write anything. The old file set is unchanged.
#[allow(clippy::too_many_arguments)]
pub(super) fn replace_metadata(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    entries: &[Metadata],
) -> Result<bool> {
    let (_, body) = shared::open_private(storage, archive, mode, authority, key)?;
    if !body.starts_with(b"RV4TRE01") {
        let mut image = Image::open(
            allocation::compaction::View(&*storage),
            archive,
            mode,
            authority,
            key,
        )?;
        image.catalogue.upgrade(&image.anchor)?;
        image.catalogue.set_tree_metadata(entries)?;
        let fits = match image
            .catalogue
            .dense_body_if_fits(&image.codec, image.anchor.sealed_len)?
        {
            Some(body) => match shared::encode_private(archive, mode, key, &body) {
                Ok(_) => true,
                Err(Error::SecurityLimitExceeded(_)) => false,
                Err(error) => return Err(error),
            },
            None => false,
        };
        let records = if fits {
            Vec::new()
        } else {
            image.catalogue.tree_records()?
        };
        shared::prepare(&image.anchor, authority, signer)?;
        drop(image);
        return if fits {
            super::dense_update::replace_filesystem_metadata(
                storage, archive, mode, authority, signer, key, entries,
            )
        } else {
            tree::grow_dense_records(storage, archive, mode, authority, signer, key, records)
        };
    }
    let mut opened = TreeImage::open(
        allocation::compaction::View(&*storage),
        archive,
        mode,
        authority,
        key,
    )?;
    opened.image.catalogue.set_tree_metadata(entries)?;
    let records = opened.image.catalogue.tree_records()?;
    drop(opened);
    tree::rewrite_records(storage, archive, mode, authority, signer, key, records)
}

/// Stable source/read lock and a fresh owned destination are required. This is
/// an experimental export, not an in-place migration or public format choice.
#[allow(clippy::too_many_arguments)]
pub(super) fn from_dense<S: Storage, T: Storage>(
    source: &S,
    destination: T,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
) -> Result<T> {
    let mut destination = Aligned(destination);
    if destination.len()? != 0 {
        return Err(Error::InvalidInput(
            "tree export destination must be empty".into(),
        ));
    }
    let mut opened = Image::open(
        allocation::compaction::View(source),
        archive,
        mode,
        authority,
        key,
    )?;
    if opened.anchor.keys != publication::RootRef::default() {
        return Err(Error::InvalidOperation(
            "tree export does not translate access slots".into(),
        ));
    }
    opened.catalogue.filesystem_metadata()?;
    opened.verify_all()?;
    let base = shared::commitment(&opened.anchor)?;
    let result = (|| {
        destination.append(&vec![0; REGION_LEN])?;
        let mut rows = Vec::new();
        for pack in &mut opened.catalogue.packs {
            let bytes = crate::page_buffer::ZeroizingBytes::new(
                source.read_at(pack.extent.start, pack.extent.len as usize)?,
            );
            if crate::crypto::strong_checksum(&bytes) != pack.extent.digest {
                return Err(Error::CorruptRecord);
            }
            pack.extent.start = destination.append(&bytes)?;
            let name = [vec![2], pack.extent.start.to_be_bytes().to_vec()].concat();
            let value = [
                pack.extent.len.to_le_bytes().to_vec(),
                pack.extent.digest.to_vec(),
            ]
            .concat();
            rows.push(Entry::new(0, &name, &value)?);
        }
        let end = destination.len()?;
        let aligned = end.div_ceil(FAILURE_REGION) * FAILURE_REGION;
        if aligned > end {
            destination.append(&vec![0; (aligned - end) as usize])?;
            rows.push(Entry::new(
                0,
                &[vec![0], end.to_be_bytes().to_vec()].concat(),
                &(aligned - end).to_le_bytes(),
            )?);
        }
        rows.extend(opened.catalogue.tree_records()?);
        rows.sort_by(|a, b| (a.namespace, a.key.as_slice()).cmp(&(b.namespace, b.key.as_slice())));
        let root = Index::new(archive, mode, key)?
            .build_sorted(&mut destination, rows.into_iter().map(Ok))?
            .root;
        shared::initialize(
            &mut destination,
            archive,
            mode,
            authority,
            signer,
            key,
            &tree::manifest(root),
            &[],
        )?;
        TreeImage::open(
            allocation::compaction::View(&destination),
            archive,
            mode,
            authority,
            key,
        )?
        .image
        .verify_all()?;
        if shared::commitment(&shared::open_private(source, archive, mode, authority, key)?.0)?
            != base
        {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    })();
    if let Err(error) = result {
        if let Err(cleanup) = allocation::compaction::discard(&mut destination) {
            return Err(Error::InvalidOperation(format!(
                "tree export failed ({error}); destination cleanup failed ({cleanup})"
            )));
        }
        return Err(error);
    }
    Ok(destination.0)
}

#[derive(Clone, Debug)]
struct Aligned<S>(S);
impl<S: Storage> Storage for Aligned<S> {
    fn len(&self) -> Result<u64> {
        self.0.len()
    }
    fn read_at(&self, at: u64, len: usize) -> Result<Vec<u8>> {
        self.0.read_at(at, len)
    }
    fn read_at_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        self.0.read_at_into(at, out)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        self.0.append(bytes)
    }
    fn append_pair(&mut self, bytes: &[u8]) -> Result<(u64, u64)> {
        if bytes.len() > FAILURE_REGION as usize || self.len()? % FAILURE_REGION != 0 {
            return Err(Error::CorruptRecord);
        }
        let primary = self.append(bytes)?;
        self.append(&vec![0; FAILURE_REGION as usize - bytes.len()])?;
        let mirror = self.append(bytes)?;
        self.append(&vec![0; FAILURE_REGION as usize - bytes.len()])?;
        Ok((primary, mirror))
    }
    fn write_at(&mut self, at: u64, bytes: &[u8]) -> Result<()> {
        self.0.write_at(at, bytes)
    }
    fn truncate(&mut self, len: u64) -> Result<()> {
        self.0.truncate(len)
    }
    fn sync(&self) -> Result<()> {
        self.0.sync()
    }
}
