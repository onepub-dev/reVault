//! Test-only whole-tree path installation. No public format activation or
//! automatic adoption of abandoned temporary files is implied by this adapter.
use super::*;
use crate::storage::{atomic_file_replacement::AtomicFileReplacement, StorageBackend};
use std::path::Path;

pub(in crate::file_format::candidate_files) fn compact_path(
    path: &Path,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
) -> Result<TreeImage<StorageBackend>> {
    compact_path_with(path, archive, mode, authority, signer, key, |phase| {
        super::super::compaction::checkpoint(phase);
        Ok(())
    })
}

pub(in crate::file_format::candidate_files) fn compact_path_with(
    path: &Path,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    mut checkpoint: impl FnMut(&str) -> Result<()>,
) -> Result<TreeImage<StorageBackend>> {
    let metadata = std::fs::symlink_metadata(path).map_err(|e| Error::Io(e.to_string()))?;
    if !metadata.file_type().is_file() {
        return Err(Error::InvalidOperation(
            "compact a regular archive file, not a symlink or directory".into(),
        ));
    }
    // Retain the exclusive source inode lock through directory synchronization.
    let source = StorageBackend::file_for_write(path)?;
    let replacement = AtomicFileReplacement::for_compaction(path)?;
    let temporary = StorageBackend::create_file(replacement.temp_path(), &[])?;
    let mut cleanup = temporary.clone();
    let mut installed = false;
    let result = (|| {
        std::fs::set_permissions(replacement.temp_path(), metadata.permissions())
            .map_err(|e| Error::Io(e.to_string()))?;
        checkpoint("tree-created")?;
        let destination = compact(&source, temporary, archive, mode, authority, signer, key)?;
        let mut checked = TreeImage::open(destination, archive, mode, authority, key)?;
        checked.image.verify_all()?;
        checked.image.storage.sync()?;
        checkpoint("tree-verified")?;
        // Refuse stale source and temporary paths before crossing publication.
        source.ensure_current()?;
        checked.image.storage.ensure_current()?;
        replacement.publish()?;
        installed = true;
        checked.image.storage.relocate(path);
        checkpoint("tree-installed")?;
        replacement.sync_parent().map_err(|error| Error::Io(format!(
            "tree compaction installed but directory durability is uncertain; reopen before continuing: {error}"
        )))?;
        checkpoint("tree-synced")?;
        Ok(checked)
    })();
    match result {
        Err(error) if !installed => {
            // Identity must still name our inode. Never erase/unlink a substitute
            // path, or claim successful cleanup after an erasure/sync failure.
            if let Err(cleanup_error) = cleanup
                .ensure_current()
                .and_then(|()| allocation::compaction::discard(&mut cleanup))
            {
                return Err(Error::InvalidOperation(format!(
                    "tree installation failed ({error}); temporary cleanup failed ({cleanup_error})"
                )));
            }
            drop(cleanup);
            std::fs::remove_file(replacement.temp_path()).map_err(|cleanup_error| {
                Error::Io(format!(
                "tree installation failed ({error}); temporary removal failed ({cleanup_error})"
            ))
            })?;
            replacement.sync_parent().map_err(|cleanup_error| Error::Io(format!(
                "tree installation failed ({error}); temporary removal durability uncertain ({cleanup_error})"
            )))?;
            Err(error)
        }
        // Never wipe the installed archive when the directory sync fails.
        other => other,
    }
}
