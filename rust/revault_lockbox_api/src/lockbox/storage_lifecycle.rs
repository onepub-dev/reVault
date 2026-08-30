use super::Lockbox;
use crate::file_format::write_header;
use crate::host_path::HostPath;
use crate::storage::StorageBackend;
#[cfg(any(test, feature = "migration"))]
use crate::Error;
use crate::{LockboxOptions, Result};
#[cfg(any(test, feature = "migration"))]
use std::fs;
use std::path::Path;

impl Lockbox<crate::Writable> {
    /// Commits pending mutations and serializes the complete lockbox.
    ///
    /// File-backed lockboxes should normally use [`Lockbox::commit`] instead.
    pub fn try_to_bytes(&self) -> Result<Vec<u8>> {
        self.bytes()
    }

    #[cfg(test)]
    /// Serializes the lockbox for tests, panicking if materialization fails.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.try_to_bytes()
            .expect("failed to materialize lockbox bytes")
    }

    #[cfg(test)]
    pub(crate) fn open_path(path: impl AsRef<Path>, key: impl AsRef<[u8]>) -> Result<Self> {
        Self::open_path_with_options(path, key, LockboxOptions::default())
    }

    #[cfg(test)]
    pub(crate) fn open_path_with_options(
        path: impl AsRef<Path>,
        key: impl AsRef<[u8]>,
        options: LockboxOptions,
    ) -> Result<Self> {
        let key = crate::SecretVec::try_from_slice(key.as_ref())?;
        let mut lockbox = Self::open_path_with_secret_key_options(path, key, options)?;
        lockbox.set_owner_signing_key(crate::OwnerSigningKeyPair::generate()?);
        Ok(lockbox)
    }

    pub(crate) fn open_path_with_secret_key_options(
        path: impl AsRef<Path>,
        key: crate::SecretVec,
        options: LockboxOptions,
    ) -> Result<Self> {
        let path = HostPath::new(path);
        Self::open_storage_with_secret_key(StorageBackend::file(path.as_path())?, key, options)
    }

    #[cfg(feature = "vault-integration")]
    pub(crate) fn open_path_with_secret_key_options_for_write(
        path: impl AsRef<Path>,
        key: crate::SecretVec,
        options: LockboxOptions,
    ) -> Result<Self> {
        let path = HostPath::new(path);
        let mut lockbox = Self::open_storage_with_secret_key_mode(
            StorageBackend::file_for_write(path.as_path())?,
            key,
            options,
            true,
        )?;
        lockbox.complete_pending_transaction_cleanup()?;
        Ok(lockbox)
    }

    #[cfg(test)]
    pub(crate) fn create_path(path: impl AsRef<Path>, key: impl AsRef<[u8]>) -> Result<Self> {
        Self::create_path_with_options(path, key, LockboxOptions::default())
    }

    #[cfg(test)]
    pub(crate) fn create_path_with_options(
        path: impl AsRef<Path>,
        key: impl AsRef<[u8]>,
        options: LockboxOptions,
    ) -> Result<Self> {
        Self::create_path_with_lockbox_id_and_options(
            path,
            key,
            crate::lockbox_id::LockboxId::new_random()?,
            options,
        )
    }

    #[cfg(test)]
    pub(crate) fn create_path_with_lockbox_id_and_options(
        path: impl AsRef<Path>,
        key: impl AsRef<[u8]>,
        lockbox_id: crate::lockbox_id::LockboxId,
        options: LockboxOptions,
    ) -> Result<Self> {
        let key = crate::SecretVec::try_from_slice(key.as_ref())?;
        let mut lockbox =
            Self::create_path_with_secret_key_and_options(path, key, lockbox_id, options)?;
        lockbox.set_owner_signing_key(crate::OwnerSigningKeyPair::generate()?);
        Ok(lockbox)
    }

    pub(crate) fn create_path_with_secret_key_and_options(
        path: impl AsRef<Path>,
        key: crate::SecretVec,
        lockbox_id: crate::lockbox_id::LockboxId,
        options: LockboxOptions,
    ) -> Result<Self> {
        let path = HostPath::new(path);
        let mut bytes = vec![0; crate::constants::HEADER_LEN];
        write_header(&mut bytes, 0, 0, 0, lockbox_id, 0);
        let mut lockbox = Self::open_storage_with_secret_key(
            StorageBackend::create_file(path.as_path(), &bytes)?,
            key,
            options,
        )?;
        lockbox.lockbox_id = lockbox_id;
        Ok(lockbox)
    }

    pub(crate) fn create_path_with_secret_key_and_options_unlocked(
        path: impl AsRef<Path>,
        key: crate::SecretVec,
        lockbox_id: crate::lockbox_id::LockboxId,
        options: LockboxOptions,
    ) -> Result<Self> {
        let path = HostPath::new(path);
        let mut bytes = vec![0; crate::constants::HEADER_LEN];
        write_header(&mut bytes, 0, 0, 0, lockbox_id, 0);
        let mut lockbox = Self::open_storage_with_secret_key(
            StorageBackend::create_file_unlocked(path.as_path(), &bytes)?,
            key,
            options,
        )?;
        lockbox.lockbox_id = lockbox_id;
        Ok(lockbox)
    }

    /// Write the current lockbox bytes to a host filesystem path.
    ///
    /// Returns `Error::Io` if the host write fails. Returns storage or
    /// serialization errors if pending lockbox state cannot be materialized.
    #[cfg(any(test, feature = "migration"))]
    pub fn write_to_path(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = HostPath::new(path);
        fs::write(path.as_path(), self.bytes()?).map_err(|err| Error::Io(err.to_string()))
    }
}
