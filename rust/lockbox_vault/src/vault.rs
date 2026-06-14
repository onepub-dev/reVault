use lockbox_core::vault_bridge::{OpenedContentKey, VaultOpen};
use lockbox_core::{Error, Lockbox, LockboxOpen, LockboxProtection, Result, SecretString};
use std::path::Path;

use crate::{AgentClient, ContentKeyStore, VaultDirectory};

/// Vault using the default platform agent as its content-key store.
pub type LocalVault = Vault<AgentClient>;

/// Creates a `Vault` backed by the default platform agent.
pub fn local_vault() -> LocalVault {
    Vault::new(AgentClient)
}

/// High-level lockbox helper that integrates open operations with a key cache.
///
/// `Vault` wraps a `ContentKeyStore`. Creating or opening a lockbox stores
/// the resulting content key in that store. `open_lockbox` then reopens the
/// lockbox from the cached key without requiring the original password or
/// contact private key.
#[derive(Debug, Clone)]
pub struct Vault<S = AgentClient> {
    store: S,
}

impl<S> Vault<S> {
    /// Creates a vault around a custom content-key store.
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Returns the content-key store used by this vault.
    pub fn store(&self) -> &S {
        &self.store
    }
}

impl<S: ContentKeyStore> Vault<S> {
    /// Creates a password-protected lockbox and caches its content key.
    pub fn create_lockbox_with_password(
        &self,
        path: impl AsRef<Path>,
        password: &SecretString,
    ) -> Result<Lockbox> {
        self.create_lockbox(path, LockboxProtection::Password(password))
    }

    /// Opens a password-protected lockbox and caches its content key.
    pub fn open_lockbox_with_password(
        &self,
        path: impl AsRef<Path>,
        password: &SecretString,
    ) -> Result<Lockbox> {
        self.open_lockbox_with(path, LockboxOpen::Password(password))
    }

    /// Opens a password-protected lockbox and caches its content key for the
    /// requested number of seconds.
    pub fn open_lockbox_with_password_for_duration(
        &self,
        path: impl AsRef<Path>,
        password: &SecretString,
        ttl_seconds: u64,
    ) -> Result<Lockbox> {
        self.open_lockbox_with_for_duration(path, LockboxOpen::Password(password), ttl_seconds)
    }

    /// Refreshes a password-protected lockbox's cached content key for a
    /// requested number of seconds without reopening the lockbox payload.
    pub fn cache_lockbox_password_for_duration(
        &self,
        path: impl AsRef<Path>,
        password: &SecretString,
        ttl_seconds: u64,
    ) -> Result<()> {
        let path = path.as_ref();
        let opened = open_path_or_backup_with_password(path, password)?;
        self.store.put_content_key_for_path_with_ttl(
            opened.lockbox_id,
            opened.try_clone_key()?,
            path,
            ttl_seconds,
        )
    }

    /// Creates a lockbox with the supplied protection mode.
    ///
    /// Content-key and password modes cache the opened content key after the
    /// file is created. Contact-public-key mode creates the file but cannot
    /// cache a content key because no private material is available.
    pub fn create_lockbox(
        &self,
        path: impl AsRef<Path>,
        protection: LockboxProtection<'_>,
    ) -> Result<Lockbox> {
        let path = path.as_ref();
        match protection {
            LockboxProtection::ContentKey(key) => {
                let store_key = key.try_clone()?;
                let lockbox = create_lockbox_file(path, LockboxProtection::ContentKey(key))?;
                self.store
                    .put_content_key_for_path(lockbox.lockbox_id(), store_key, path)?;
                Ok(lockbox)
            }
            LockboxProtection::Password(password) => {
                let lockbox = create_lockbox_file(path, LockboxProtection::Password(password))?;
                let opened = VaultOpen::path_with_password(path, password)?;
                if let Err(err) = self.store.put_content_key_for_path(
                    opened.lockbox_id,
                    opened.try_clone_key()?,
                    path,
                ) {
                    if !matches!(err, Error::Io(_)) {
                        return Err(err);
                    }
                }
                Ok(lockbox)
            }
            LockboxProtection::ContactPublicKey { name, contact } => {
                create_lockbox_file(path, LockboxProtection::ContactPublicKey { name, contact })
            }
        }
    }

    /// Opens a lockbox using only a content key already present in the store.
    ///
    /// This fails if the key store has no cached key for the lockbox id.
    pub fn open_lockbox(&self, path: impl AsRef<Path>) -> Result<Lockbox> {
        let path = path.as_ref();
        let lockbox_id = VaultOpen::read_lockbox_id(path)?;
        let Some(key) = self.store.get_content_key(lockbox_id)? else {
            return Err(Error::VaultUnavailable(format!(
                "no cached content key for lockbox {lockbox_id}"
            )));
        };
        open_lockbox_file(path, LockboxOpen::ContentKey(key))
    }

    /// Opens a lockbox with explicit open material and caches its content key.
    ///
    /// Password and contact-key-pair opens may fall back to a key-directory
    /// backup stored in the default `VaultDirectory` when the embedded key
    /// directory cannot be read. That fallback requires `LOCKBOX_VAULT_PASSWORD`
    /// to be set so the default vault directory can be opened.
    pub fn open_lockbox_with(
        &self,
        path: impl AsRef<Path>,
        open: LockboxOpen<'_>,
    ) -> Result<Lockbox> {
        let path = path.as_ref();
        match open {
            LockboxOpen::ContentKey(key) => {
                let store_key = key.try_clone()?;
                let lockbox = open_lockbox_file(path, LockboxOpen::ContentKey(key))?;
                self.store
                    .put_content_key_for_path(lockbox.lockbox_id(), store_key, path)?;
                Ok(lockbox)
            }
            LockboxOpen::Password(password) => {
                let opened = open_path_or_backup_with_password(path, password)?;
                self.store.put_content_key_for_path(
                    opened.lockbox_id,
                    opened.try_clone_key()?,
                    path,
                )?;
                open_opened_path(opened, path)
            }
            LockboxOpen::ContactKeyPair(contact) => {
                let opened = open_path_or_backup_with_contact(path, &contact)?;
                self.store.put_content_key_for_path(
                    opened.lockbox_id,
                    opened.try_clone_key()?,
                    path,
                )?;
                open_opened_path(opened, path)
            }
        }
    }

    /// Opens a lockbox with explicit open material and caches its content
    /// key for the requested number of seconds.
    pub fn open_lockbox_with_for_duration(
        &self,
        path: impl AsRef<Path>,
        open: LockboxOpen<'_>,
        ttl_seconds: u64,
    ) -> Result<Lockbox> {
        let path = path.as_ref();
        match open {
            LockboxOpen::ContentKey(key) => {
                let store_key = key.try_clone()?;
                let lockbox = open_lockbox_file(path, LockboxOpen::ContentKey(key))?;
                self.store.put_content_key_for_path_with_ttl(
                    lockbox.lockbox_id(),
                    store_key,
                    path,
                    ttl_seconds,
                )?;
                Ok(lockbox)
            }
            LockboxOpen::Password(password) => {
                let opened = open_path_or_backup_with_password(path, password)?;
                let lockbox_id = opened.lockbox_id;
                let store_key = opened.try_clone_key()?;
                let lockbox = open_opened_path(opened, path)?;
                self.store.put_content_key_for_path_with_ttl(
                    lockbox_id,
                    store_key,
                    path,
                    ttl_seconds,
                )?;
                Ok(lockbox)
            }
            LockboxOpen::ContactKeyPair(contact) => {
                let opened = open_path_or_backup_with_contact(path, &contact)?;
                let lockbox_id = opened.lockbox_id;
                let store_key = opened.try_clone_key()?;
                let lockbox = open_opened_path(opened, path)?;
                self.store.put_content_key_for_path_with_ttl(
                    lockbox_id,
                    store_key,
                    path,
                    ttl_seconds,
                )?;
                Ok(lockbox)
            }
        }
    }

    /// Removes this lockbox's cached content key from the store.
    pub fn close_lockbox(&self, path: impl AsRef<Path>) -> Result<()> {
        let lockbox_id = VaultOpen::read_lockbox_id(path.as_ref())?;
        self.store.forget_content_key(lockbox_id)
    }

    /// Removes every cached content key from the store.
    pub fn close_all(&self) -> Result<()> {
        self.store.forget_all_content_keys()
    }
}

fn create_lockbox_file(path: &Path, protection: LockboxProtection<'_>) -> Result<Lockbox> {
    let signing_key = default_owner_signing_key()?.ok_or_else(|| {
        Error::VaultUnavailable(
            "vault owner signing key is unavailable; open or initialize the vault first"
                .to_string(),
        )
    })?;
    Lockbox::create_file(path, protection, &signing_key)
}

fn open_lockbox_file(path: &Path, open: LockboxOpen<'_>) -> Result<Lockbox> {
    if let Some(signing_key) = default_owner_signing_key()? {
        return Lockbox::open_file_with_owner_signing_key(path, open, signing_key);
    }
    Lockbox::open_file(path, open)
}

fn open_opened_path(opened: OpenedContentKey, path: &Path) -> Result<Lockbox> {
    let mut lockbox = opened.open_path(path)?;
    if let Some(signing_key) = default_owner_signing_key()? {
        lockbox.set_owner_signing_key(signing_key);
    }
    Ok(lockbox)
}

fn default_owner_signing_key() -> Result<Option<lockbox_core::OwnerSigningKeyPair>> {
    let password = match SecretString::try_from_env("LOCKBOX_VAULT_PASSWORD")? {
        Some(password) => Some(password),
        None => crate::get_platform_vault_password().ok().flatten(),
    };
    let Some(password) = password else {
        return Ok(None);
    };
    if !crate::default_vault_path()?.exists() {
        return Ok(None);
    }
    let Ok(vault) = VaultDirectory::open_or_create_default(&password) else {
        return Ok(None);
    };
    Ok(vault
        .load_owner_signing_key(VaultDirectory::DEFAULT_KEY_NAME)
        .ok())
}

fn open_path_or_backup_with_password(
    path: &Path,
    password: &SecretString,
) -> Result<OpenedContentKey> {
    match VaultOpen::path_with_password(path, password) {
        Ok(opened) => Ok(opened),
        Err(primary_err) => {
            let lockbox_id = VaultOpen::read_lockbox_id(path).map_err(|_| primary_err.clone())?;
            let vault_password = vault_password_from_env().map_err(|_| primary_err.clone())?;
            let backup = VaultDirectory::open_or_create_default(&vault_password)
                .and_then(|vault| vault.load_key_directory_backup(lockbox_id))
                .map_err(|_| primary_err.clone())?;
            VaultOpen::key_directory_backup_with_password(&backup, password)
                .map_err(|_| primary_err)
        }
    }
}

fn open_path_or_backup_with_contact(
    path: &Path,
    contact: &lockbox_core::ContactKeyPair,
) -> Result<OpenedContentKey> {
    match VaultOpen::path_with_contact(path, contact) {
        Ok(opened) => Ok(opened),
        Err(primary_err) => {
            let lockbox_id = VaultOpen::read_lockbox_id(path).map_err(|_| primary_err.clone())?;
            let vault_password = vault_password_from_env().map_err(|_| primary_err.clone())?;
            let backup = VaultDirectory::open_or_create_default(&vault_password)
                .and_then(|vault| vault.load_key_directory_backup(lockbox_id))
                .map_err(|_| primary_err.clone())?;
            VaultOpen::key_directory_backup_with_contact(&backup, contact).map_err(|_| primary_err)
        }
    }
}

fn vault_password_from_env() -> Result<SecretString> {
    SecretString::try_from_env("LOCKBOX_VAULT_PASSWORD")?
        .ok_or_else(|| Error::VaultUnavailable("LOCKBOX_VAULT_PASSWORD is not set".to_string()))
}
