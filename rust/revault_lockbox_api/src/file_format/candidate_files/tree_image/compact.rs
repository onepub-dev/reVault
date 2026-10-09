//! Source-preserving, whole selected-tree extent relocation. No public activation.
use super::*;
use allocation::Extent;

/// Caller holds a stable source snapshot/read lock and owns an empty destination.
/// Keep UUID/mode/key/owner, all selected logical state and publication lineage.
/// This relocates extents, not pack interiors; bounded public key wrappers are retained.
#[allow(clippy::too_many_arguments)]
pub(in crate::file_format::candidate_files) fn compact<S: Storage, T: Storage>(
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
            "tree compaction destination must be empty".into(),
        ));
    }
    // All refusal/preparation below precedes even cleanup: discard(empty) mutates.
    let mut opened = TreeImage::open(
        allocation::compaction::View(source),
        archive,
        mode,
        authority,
        key,
    )?;
    let predecessor = opened.image.anchor.clone();
    shared::retained_public_directory(source, &predecessor)?;
    let generation = predecessor
        .generation
        .checked_add(1)
        .ok_or_else(|| Error::SecurityLimitExceeded("publication generation exhausted".into()))?;
    let previous = shared::commitment(&predecessor)?;
    shared::prepare(&predecessor, authority, signer)?;
    opened.image.verify_all()?;
    let old_payloads = opened.tree.graph.payloads();
    let mut placements = BTreeMap::new();
    let mut end = REGION_LEN as u64;
    for old in &old_payloads {
        let new = Extent { start: end, ..*old };
        end = end.checked_add(old.len).ok_or(Error::CorruptRecord)?;
        if placements.insert(old.start, (*old, new)).is_some() {
            return Err(Error::CorruptRecord);
        }
    }
    let mapped = |old: Extent| -> Result<Extent> {
        match placements.get(&old.start) {
            Some((expected, new)) if *expected == old => Ok(*new),
            _ => Err(Error::CorruptRecord),
        }
    };
    let mut catalogue = opened.image.catalogue;
    for pack in &mut catalogue.packs {
        pack.extent = mapped(pack.extent)?;
    }
    for variable in &mut catalogue.variables {
        let extents = variable
            .layout
            .extents
            .iter()
            .copied()
            .map(&mapped)
            .collect::<Result<Vec<_>>>()?;
        variable.layout.rebind(&extents)?;
    }
    for text in catalogue.forms.texts_mut() {
        let extents = text
            .extents
            .iter()
            .copied()
            .map(&mapped)
            .collect::<Result<Vec<_>>>()?;
        text.rebind(&extents)?;
    }
    catalogue.vacant.clear();
    let mut rows = catalogue.tree_records()?;
    for (_, extent) in placements.values() {
        rows.push(Entry::new(
            0,
            &[vec![2], extent.start.to_be_bytes().to_vec()].concat(),
            &[extent.len.to_le_bytes().to_vec(), extent.digest.to_vec()].concat(),
        )?);
    }
    let aligned = end
        .checked_add(FAILURE_REGION - 1)
        .ok_or(Error::CorruptRecord)?
        / FAILURE_REGION
        * FAILURE_REGION;
    if aligned > end {
        rows.push(Entry::new(
            0,
            &[vec![0], end.to_be_bytes().to_vec()].concat(),
            &(aligned - end).to_le_bytes(),
        )?);
    }
    if rows.iter().filter(|row| row.namespace == 0).count() > tree::MAX_OWNERSHIP_RECORDS {
        return Err(Error::SecurityLimitExceeded(
            "tree ownership record limit".into(),
        ));
    }
    rows.sort_by(|a, b| (a.namespace, a.key.as_slice()).cmp(&(b.namespace, b.key.as_slice())));
    // Catalogue admission and placement were checked before touching destination.
    let unchanged = || {
        let (anchor, _) = shared::snapshot(source, archive, mode, authority, key)?;
        if anchor != predecessor || shared::commitment(&anchor)? != previous {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    };
    unchanged()?;
    let result = (|| {
        append_exact(&mut destination, 0, &vec![0; REGION_LEN])?;
        for old in &old_payloads {
            let new = mapped(*old)?;
            let mut hash = crate::crypto::strong_checksum_hasher(old.len);
            let mut offset = 0;
            while offset < old.len {
                let len = (old.len - offset).min(FAILURE_REGION) as usize;
                source
                    .read_at_secure(
                        old.start.checked_add(offset).ok_or(Error::CorruptRecord)?,
                        len,
                    )?
                    .with_bytes(|bytes| {
                        hash.update(bytes);
                        append_exact(
                            &mut destination,
                            new.start.checked_add(offset).ok_or(Error::CorruptRecord)?,
                            bytes,
                        )
                    })??;
                super::super::compaction::checkpoint("tree-copying");
                offset += len as u64;
            }
            if <[u8; 32]>::from(hash.finalize()) != old.digest {
                return Err(Error::CorruptRecord);
            }
            if digest(&destination, new)? != new.digest {
                return Err(Error::CorruptRecord);
            }
        }
        if destination.len()? != end {
            return Err(Error::CorruptRecord);
        }
        if aligned > end {
            append_exact(&mut destination, end, &vec![0; (aligned - end) as usize])?;
        }
        let root = Index::new(archive, mode, key)?
            .build_sorted(&mut destination, rows.into_iter().map(Ok))?
            .root;
        super::super::compaction::checkpoint("tree-dependencies");
        unchanged()?;
        let next = shared::initialize_successor(
            source,
            &predecessor,
            &mut destination,
            authority,
            signer,
            key,
            &tree::manifest(root),
        )?;
        if next.generation != generation
            || next.previous != previous
            || next.sealed_len != destination.len()?
        {
            return Err(Error::CorruptRecord);
        }
        let mut reopened = TreeImage::open(
            allocation::compaction::View(&destination),
            archive,
            mode,
            authority,
            key,
        )?;
        if reopened.image.anchor != next {
            return Err(Error::CorruptRecord);
        }
        reopened.image.verify_all()?;
        unchanged()?;
        Ok(())
    })();
    if let Err(error) = result {
        if let Err(cleanup) = allocation::compaction::discard(&mut destination) {
            return Err(Error::InvalidOperation(format!(
                "tree compaction failed ({error}); destination cleanup failed ({cleanup})"
            )));
        }
        return Err(error);
    }
    Ok(destination.0)
}
fn append_exact(storage: &mut impl Storage, expected: u64, bytes: &[u8]) -> Result<()> {
    let end = expected
        .checked_add(bytes.len() as u64)
        .ok_or(Error::CorruptRecord)?;
    if storage.len()? != expected || storage.append(bytes)? != expected || storage.len()? != end {
        return Err(Error::CorruptRecord);
    }
    Ok(())
}
pub(super) fn digest(storage: &impl Storage, extent: Extent) -> Result<[u8; 32]> {
    let mut hash = crate::crypto::strong_checksum_hasher(extent.len);
    let mut offset = 0;
    while offset < extent.len {
        let len = (extent.len - offset).min(FAILURE_REGION) as usize;
        storage
            .read_at_secure(
                extent
                    .start
                    .checked_add(offset)
                    .ok_or(Error::CorruptRecord)?,
                len,
            )?
            .with_bytes(|bytes| hash.update(bytes))?;
        offset += len as u64;
    }
    Ok(hash.finalize().into())
}
