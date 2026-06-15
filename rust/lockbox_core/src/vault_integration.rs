//! Narrow integration surface used by `lockbox_vault`.
//!
//! Normal callers should use `Lockbox::open_file`. This module exists because
//! the vault crate needs to cache an opened content key and recover from
//! vault-stored key-directory backups.

use std::path::Path;

use crate::{ContactKeyPair, Lockbox, LockboxId, Result, SecretString};

pub use crate::lockbox::OpenedContentKey;

/// Open helpers needed by the separate vault crate.
pub struct VaultOpen;

impl VaultOpen {
    /// Read the lockbox id from a lockbox file header for vault cache lookup.
    pub fn read_lockbox_id(path: &Path) -> Result<LockboxId> {
        Lockbox::read_lockbox_id(path)
    }

    /// Export key-directory backup bytes for vault-managed recovery.
    pub fn export_key_directory_backup(lockbox: &Lockbox) -> Result<Vec<u8>> {
        lockbox.export_key_directory_backup()
    }

    /// Open the embedded key directory with a password and return the content key.
    pub fn path_with_password(path: &Path, password: &SecretString) -> Result<OpenedContentKey> {
        Lockbox::open_path_with_password(path, password)
    }

    /// Open key-directory backup bytes with a password.
    pub fn key_directory_backup_with_password(
        bytes: &[u8],
        password: &SecretString,
    ) -> Result<OpenedContentKey> {
        Lockbox::open_key_directory_backup_with_password(bytes, password)
    }

    /// Open the embedded key directory with a contact keypair and return the content key.
    pub fn path_with_contact(path: &Path, contact: &ContactKeyPair) -> Result<OpenedContentKey> {
        Lockbox::open_path_with_contact(path, contact)
    }

    /// Open key-directory backup bytes with a contact keypair.
    pub fn key_directory_backup_with_contact(
        bytes: &[u8],
        contact: &ContactKeyPair,
    ) -> Result<OpenedContentKey> {
        Lockbox::open_key_directory_backup_with_contact(bytes, contact)
    }
}
