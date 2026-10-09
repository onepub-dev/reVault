//! Admission of a complete interrupted extent-copy replacement. Never infer
//! ownership from a filename, or erase an incomplete/unrecognized candidate.
use super::*;
use crate::storage::{atomic_file_replacement::AtomicFileReplacement, StorageBackend};
use allocation::Extent;
use std::path::Path;

/// A valid successor alone may contain an ordinary edit. Require the exact
/// selected catalogue and encoded payloads of our deterministic extent copy.
fn verify_copy<S: Storage, T: Storage>(
    source: &S,
    candidate: &T,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<()> {
    let mut old = TreeImage::open(
        allocation::compaction::View(source),
        archive,
        mode,
        authority,
        key,
    )?;
    let mut new = TreeImage::open(
        allocation::compaction::View(candidate),
        archive,
        mode,
        authority,
        key,
    )?;
    let base = old.image.anchor.clone();
    let next = new.image.anchor.clone();
    if shared::retained_public_directory(source, &base)?
        != shared::retained_public_directory(candidate, &next)?
        || base.generation.checked_add(1) != Some(next.generation)
        || next.previous != shared::commitment(&base)?
        || source.len()? != base.sealed_len
        || candidate.len()? != next.sealed_len
    {
        return Err(Error::InvalidOperation(
            "replacement is not a complete direct compaction successor".into(),
        ));
    }
    old.image.verify_all()?;
    new.image.verify_all()?;
    let original = old.tree.graph.payloads();
    let copied = new.tree.graph.payloads();
    if original.len() != copied.len() {
        return Err(Error::CorruptRecord);
    }
    let mut inverse = BTreeMap::new();
    let mut at = REGION_LEN as u64;
    for (old, new) in original.iter().zip(&copied) {
        if *new != (Extent { start: at, ..*old })
            || super::compact::digest(source, *old)? != old.digest
            || super::compact::digest(candidate, *new)? != new.digest
        {
            return Err(Error::CorruptRecord);
        }
        inverse.insert(new.start, (*new, *old));
        at = at.checked_add(old.len).ok_or(Error::CorruptRecord)?;
    }
    let map = |extent: Extent| -> Result<Extent> {
        match inverse.get(&extent.start) {
            Some((expected, original)) if *expected == extent => Ok(*original),
            _ => Err(Error::CorruptRecord),
        }
    };
    for pack in &mut new.image.catalogue.packs {
        pack.extent = map(pack.extent)?;
    }
    for variable in &mut new.image.catalogue.variables {
        let extents = variable
            .layout
            .extents
            .iter()
            .copied()
            .map(&map)
            .collect::<Result<Vec<_>>>()?;
        variable.layout.rebind(&extents)?;
    }
    for text in new.image.catalogue.forms.texts_mut() {
        let extents = text
            .extents
            .iter()
            .copied()
            .map(&map)
            .collect::<Result<Vec<_>>>()?;
        text.rebind(&extents)?;
    }
    let old_rows = old.image.catalogue.tree_records()?;
    let new_rows = new.image.catalogue.tree_records()?;
    if old_rows.len() != new_rows.len()
        || old_rows
            .iter()
            .zip(&new_rows)
            .any(|(a, b)| a.namespace != b.namespace || a.key != b.key || a.value != b.value)
    {
        return Err(Error::InvalidOperation(
            "replacement changes selected archive state".into(),
        ));
    }
    // Stable locked storage is required, but also reject a changed publication.
    if shared::snapshot(source, archive, mode, authority, key)?.0 != base
        || shared::snapshot(candidate, archive, mode, authority, key)?.0 != next
    {
        return Err(Error::CorruptRecord);
    }
    Ok(())
}

/// Explicit candidate selection only. No directory scan, partial-copy cleanup,
/// stale-source adoption, or post-install receipt inference is authorized here.
pub(in crate::file_format::candidate_files) fn resume_path(
    path: &Path,
    candidate: &Path,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<TreeImage<StorageBackend>> {
    let metadata = std::fs::symlink_metadata(path).map_err(|e| Error::Io(e.to_string()))?;
    let replacement_metadata =
        std::fs::symlink_metadata(candidate).map_err(|e| Error::Io(e.to_string()))?;
    if !metadata.is_file() || !replacement_metadata.is_file() {
        return Err(Error::InvalidOperation(
            "resume requires two regular archive files".into(),
        ));
    }
    let path = std::fs::canonicalize(path).map_err(|e| Error::Io(e.to_string()))?;
    let candidate = std::fs::canonicalize(candidate).map_err(|e| Error::Io(e.to_string()))?;
    if path == candidate || path.parent() != candidate.parent() {
        return Err(Error::InvalidOperation(
            "replacement must be a distinct file in the archive directory".into(),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.dev() == replacement_metadata.dev()
            && metadata.ino() == replacement_metadata.ino()
        {
            return Err(Error::InvalidOperation(
                "replacement is a hard link to the source".into(),
            ));
        }
    }
    let source = StorageBackend::file_for_write(&path)?;
    let temporary = StorageBackend::file_for_write(&candidate)?;
    verify_copy(&source, &temporary, archive, mode, authority, key)?;
    source.ensure_current()?;
    temporary.ensure_current()?;
    std::fs::set_permissions(&candidate, metadata.permissions())
        .map_err(|e| Error::Io(e.to_string()))?;
    temporary.sync()?;
    super::super::compaction::checkpoint("tree-resume-verified");
    source.ensure_current()?;
    temporary.ensure_current()?;
    let replacement = AtomicFileReplacement::existing_for_test(&path, &candidate);
    replacement.publish()?;
    let mut installed = temporary;
    installed.relocate(&path);
    super::super::compaction::checkpoint("tree-resume-installed");
    replacement.sync_parent().map_err(|error| Error::Io(format!(
        "resumed compaction installed but directory durability is uncertain; reopen before continuing: {error}"
    )))?;
    super::super::compaction::checkpoint("tree-resume-synced");
    TreeImage::open(installed, archive, mode, authority, key)
}
