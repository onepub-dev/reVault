//! File installation for the test-only candidate. Keep the original locked until
//! the fully validated replacement has been renamed and its directory synced.
use super::*;
use crate::storage::{atomic_file_replacement::AtomicFileReplacement, StorageBackend};
use std::path::Path;

impl Files<StorageBackend> {
    pub(crate) fn compact_path(
        path: &Path,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        signer: Option<&OwnerSigningKeyPair>,
        key: Option<&[u8]>,
    ) -> Result<Self> {
        let metadata = std::fs::symlink_metadata(path).map_err(|e| Error::Io(e.to_string()))?;
        if metadata.file_type().is_symlink() {
            return Err(Error::InvalidOperation(
                "compact the archive's real path, not a symlink".into(),
            ));
        }
        let mut source = Self::open(
            StorageBackend::file_for_write(path)?,
            archive,
            mode,
            authority,
            key,
        )?;
        let replacement = AtomicFileReplacement::for_compaction(path)?;
        // create_file refuses existing paths; only clean up a temporary we own.
        let temporary = StorageBackend::create_file(replacement.temp_path(), &[])?;
        let mut cleanup = temporary.clone();
        let mut installed = false;
        let result = (|| {
            std::fs::set_permissions(replacement.temp_path(), metadata.permissions())
                .map_err(|e| Error::Io(e.to_string()))?;
            let destination = source.compact_into(temporary, authority, signer, key)?;
            let mut checked = Self::open(destination, archive, mode, authority, key)?;
            checked.audit_with(true)?;
            checkpoint("verified");
            source.storage.ensure_current()?;
            replacement.publish()?;
            installed = true;
            checked.storage.relocate(path);
            checkpoint("installed");
            replacement.sync_parent().map_err(|error| Error::Io(format!(
                "compaction installed but directory durability is uncertain; reopen before continuing: {error}"
            )))?;
            Ok(checked)
        })();
        // Never wipe the installed archive after a directory-sync failure.
        if result.is_err() && !installed {
            let erased = allocation::compaction::discard(&mut cleanup);
            drop(cleanup);
            let removed = std::fs::remove_file(replacement.temp_path());
            if let Err(error) = erased {
                return Err(Error::Io(format!("compaction failed ({:?}); temporary erasure failed ({error}); removal: {removed:?}", result.err())));
            }
            removed.map_err(|error| {
                Error::Io(format!(
                    "compaction failed ({:?}); temporary removal failed: {error}",
                    result.as_ref().err()
                ))
            })?;
        }
        result
    }
}

// Process-local environment is set only in the dedicated child test. Candidate
// C itself is cfg(test); this is never a production environment-variable hook.
pub(in crate::file_format) fn checkpoint(phase: &str) {
    if std::env::var("REVAULT_CANDIDATE_COMPACTION_EXIT").as_deref() == Ok(phase) {
        std::process::exit(71);
    }
}
