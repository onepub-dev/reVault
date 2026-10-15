//! Fresh packed-C export without a dense or serialized catalogue intermediate.
use super::*;

/// Caller retains a stable source snapshot/read lock and owns an empty destination.
/// Metadata supplies filesystem kinds/permissions absent from packed C, but must
/// describe exactly its authenticated file set and lengths. No access slots are
/// translated. This is an experimental adapter, not public format activation.
pub(in crate::file_format::candidate_files) fn from_candidate<S: Storage, T: Storage>(
    source: &mut Files<S>,
    destination: T,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    entries: &[Metadata],
) -> Result<T> {
    let mut destination = Aligned(destination);
    if destination.len()? != 0 {
        return Err(Error::InvalidInput(
            "tree export destination must be empty".into(),
        ));
    }
    if source.anchor.keys != publication::RootRef::default() {
        return Err(Error::InvalidOperation(
            "tree export does not translate access slots".into(),
        ));
    }
    let archive = source.anchor.archive;
    let mode = source.anchor.mode;
    let selected = publication::select(&source.storage, archive, mode, authority)?;
    if selected.commitment != source.anchor.commitment()? {
        return Err(Error::CorruptRecord);
    }
    let result = (|| {
        destination.append(&vec![0; REGION_LEN])?;
        let repacked =
            super::super::cost_model::repack_tree_into(source, entries, |extent, bytes| {
                if destination.append(bytes)? != extent.start {
                    return Err(Error::CorruptRecord);
                }
                Ok(())
            })?;
        if repacked.end() != destination.len()? {
            return Err(Error::CorruptRecord);
        }
        let codec = Codec::shared_packed(archive, mode, key)?;
        let mut catalogue = repacked.into_catalogue(&codec)?;
        catalogue.set_tree_metadata(entries)?;
        catalogue.verify_padding(&destination, &codec)?;
        let catalogue = {
            let mut unpublished = Image {
                storage: allocation::compaction::View(&destination),
                anchor: Anchor {
                    sealed_len: destination.len()?,
                    ..source.anchor.clone()
                },
                value_key: super::super::dense_image::value_key_for(&catalogue, mode, key)?,
                catalogue,
                codec,
            };
            unpublished.verify_all()?;
            unpublished.catalogue
        };
        let end = destination.len()?;
        let aligned = end.div_ceil(FAILURE_REGION) * FAILURE_REGION;
        let vacancy = if aligned > end {
            destination.append(&vec![0; (aligned - end) as usize])?;
            Some(Entry::new(
                0,
                &[vec![0], end.to_be_bytes().to_vec()].concat(),
                &(aligned - end).to_le_bytes(),
            )?)
        } else {
            None
        };
        let ownership = catalogue.packs.iter().map(|pack| {
            Entry::new(
                0,
                &[vec![2], pack.extent.start.to_be_bytes().to_vec()].concat(),
                &[
                    pack.extent.len.to_le_bytes().to_vec(),
                    pack.extent.digest.to_vec(),
                ]
                .concat(),
            )
        });
        let rows = vacancy
            .into_iter()
            .map(Ok)
            .chain(ownership)
            .chain(catalogue.fresh_tree_records()?);
        let root = Index::new(archive, mode, key)?
            .build_sorted(&mut destination, rows)?
            .root;
        // Release construction metadata before independently reopening/auditing.
        drop(catalogue);
        // Detect source publication changes before publishing copied authority.
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
            &tree::manifest(root),
            &[],
        )?;
        AuditedTreeImage::open(
            allocation::compaction::View(&destination),
            archive,
            mode,
            authority,
            key,
        )?
        .image
        .verify_all()?;
        if publication::select(&source.storage, archive, mode, authority)?.commitment
            != selected.commitment
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
