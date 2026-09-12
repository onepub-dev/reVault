use super::Lockbox;
use crate::lockbox_path::LockboxPath;
use crate::{Error, LockboxProtection, OwnerSigningKeyPair, Result, SecretVec};
use std::path::Path;

/// Write a lockbox whose new logical state is published but whose redaction
/// cleanup has not completed. This is available only with `test-support`.
pub fn write_interrupted_transaction_fixture(path: &Path, key: &[u8]) -> Result<()> {
    let signing_key = OwnerSigningKeyPair::generate()?;
    let mut lockbox = Lockbox::create_in_memory(
        LockboxProtection::ContentKey(SecretVec::try_from_slice(key)?),
        &signing_key,
    )?;
    let removed_path = LockboxPath::new("/docs/remove.txt")?;
    let kept_path = LockboxPath::new("/docs/keep.txt")?;
    lockbox.create_parent_dirs_for(&removed_path)?;
    lockbox.add_file(&removed_path, b"remove me", false)?;
    lockbox.add_file(&kept_path, b"keep", false)?;
    lockbox.commit()?;

    let removed_offset = lockbox
        .toc_entries
        .get(&removed_path)
        .ok_or_else(|| Error::NotFound(removed_path.to_string()))?
        .record_offset;
    lockbox.delete(&removed_path)?;
    lockbox.storage.inject_test_write_failure_at(removed_offset);
    match lockbox.commit() {
        Err(Error::RecoveryRequired { .. }) => std::fs::write(path, lockbox.try_to_bytes()?)
            .map_err(|err| Error::Io(format!("write {}: {err}", path.display()))),
        Err(err) => Err(err),
        Ok(()) => Err(Error::InvalidOperation(
            "test fixture commit unexpectedly completed".to_string(),
        )),
    }
}
