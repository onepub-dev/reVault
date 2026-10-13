use revault_lockbox_api::vault_integration::{OpenedContentKey, VaultOpen};
use revault_lockbox_api::{
    Error, Lockbox, LockboxOpen, LockboxProtection, OwnerSigningKeyPair, ReadOnly, Result,
    SecretString,
};
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
        let signing_key = default_owner_signing_key()?.ok_or_else(|| {
            Error::VaultUnavailable(
                "vault owner signing key is unavailable; open or initialize the vault first"
                    .to_string(),
            )
        })?;
        self.create_lockbox_with_signing_key(path, protection, &signing_key)
    }

    /// Creates a lockbox using an owner signing key loaded by the caller.
    ///
    /// This variant lets interactive callers reuse an already-open encrypted
    /// vault instead of requiring its passphrase to also be available through
    /// process environment or platform secret-store state. Private signing
    /// material remains owned by [`OwnerSigningKeyPair`], whose serialized
    /// private key is held in secure, zeroizing memory.
    pub fn create_lockbox_with_signing_key(
        &self,
        path: impl AsRef<Path>,
        protection: LockboxProtection<'_>,
        signing_key: &OwnerSigningKeyPair,
    ) -> Result<Lockbox> {
        let path = path.as_ref();
        match protection {
            LockboxProtection::ContentKey(key) => {
                let store_key = key.try_clone()?;
                let lockbox =
                    Lockbox::create_file(path, LockboxProtection::ContentKey(key), signing_key)?;
                self.store
                    .put_content_key_for_path(lockbox.lockbox_id(), store_key, path)?;
                Ok(lockbox)
            }
            LockboxProtection::Password(password) => {
                let lockbox =
                    Lockbox::create_file(path, LockboxProtection::Password(password), signing_key)?;
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
            LockboxProtection::ContactPublicKey { name, contact } => Lockbox::create_file(
                path,
                LockboxProtection::ContactPublicKey { name, contact },
                signing_key,
            ),
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
        open_file(path, LockboxOpen::ContentKey(key))
    }

    /// Opens a cached lockbox for read-only metadata access without loading
    /// or requesting an owner-signing key.
    pub fn open_lockbox_read_only(&self, path: impl AsRef<Path>) -> Result<Lockbox<ReadOnly>> {
        let path = path.as_ref();
        let lockbox_id = VaultOpen::read_lockbox_id(path)?;
        let Some(key) = self.store.get_content_key(lockbox_id)? else {
            return Err(Error::VaultUnavailable(format!(
                "no cached content key for lockbox {lockbox_id}"
            )));
        };
        Lockbox::open(path, LockboxOpen::ContentKey(key))
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
        let signing_key = default_owner_signing_key_required()?;
        self.open_lockbox_with_signing_key(path, open, &signing_key)
    }

    /// Opens a lockbox using explicit open material and an owner signing key
    /// loaded by the caller, then caches its content key.
    pub fn open_lockbox_with_signing_key(
        &self,
        path: impl AsRef<Path>,
        open: LockboxOpen<'_>,
        signing_key: &OwnerSigningKeyPair,
    ) -> Result<Lockbox> {
        let path = path.as_ref();
        match open {
            LockboxOpen::ContentKey(key) => {
                let store_key = key.try_clone()?;
                let lockbox =
                    Lockbox::open_for_write(path, LockboxOpen::ContentKey(key), signing_key)?;
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
                opened.open_path_for_write(path, signing_key)
            }
            LockboxOpen::ContactKeyPair(contact) => {
                let opened = open_path_or_backup_with_contact(path, &contact)?;
                self.store.put_content_key_for_path(
                    opened.lockbox_id,
                    opened.try_clone_key()?,
                    path,
                )?;
                match opened.open_path_for_write(path, signing_key) {
                    Ok(lockbox) => Ok(lockbox),
                    Err(Error::InvalidOperation(_)) => {
                        self.open_cached_with_signing_key(path, signing_key)
                    }
                    Err(err) => Err(err),
                }
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
        let signing_key = default_owner_signing_key_required()?;
        self.open_lockbox_with_for_duration_and_signing_key(path, open, ttl_seconds, &signing_key)
    }

    /// Opens a lockbox for a requested duration using an owner signing key
    /// loaded by the caller.
    pub fn open_lockbox_with_for_duration_and_signing_key(
        &self,
        path: impl AsRef<Path>,
        open: LockboxOpen<'_>,
        ttl_seconds: u64,
        signing_key: &OwnerSigningKeyPair,
    ) -> Result<Lockbox> {
        let path = path.as_ref();
        match open {
            LockboxOpen::ContentKey(key) => {
                let store_key = key.try_clone()?;
                let lockbox =
                    Lockbox::open_for_write(path, LockboxOpen::ContentKey(key), signing_key)?;
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
                let lockbox = opened.open_path_for_write(path, signing_key)?;
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
                self.store.put_content_key_for_path_with_ttl(
                    lockbox_id,
                    store_key,
                    path,
                    ttl_seconds,
                )?;
                match opened.open_path_for_write(path, signing_key) {
                    Ok(lockbox) => Ok(lockbox),
                    Err(Error::InvalidOperation(_)) => {
                        self.open_cached_with_signing_key(path, signing_key)
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }

    fn open_cached_with_signing_key(
        &self,
        path: &Path,
        signing_key: &OwnerSigningKeyPair,
    ) -> Result<Lockbox> {
        let lockbox_id = VaultOpen::read_lockbox_id(path)?;
        let Some(key) = self.store.get_content_key(lockbox_id)? else {
            return Err(Error::VaultUnavailable(format!(
                "no cached content key for lockbox {lockbox_id}"
            )));
        };
        Lockbox::open_for_write(path, LockboxOpen::ContentKey(key), signing_key)
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

fn open_file(path: &Path, open: LockboxOpen<'_>) -> Result<Lockbox> {
    let signing_key = default_owner_signing_key_required()?;
    Lockbox::open_for_write(path, open, &signing_key)
}

fn default_owner_signing_key_required() -> Result<revault_lockbox_api::OwnerSigningKeyPair> {
    default_owner_signing_key()?.ok_or_else(|| {
        Error::VaultUnavailable(
            "vault owner signing key is unavailable; open or initialize the vault first"
                .to_string(),
        )
    })
}

fn default_owner_signing_key() -> Result<Option<revault_lockbox_api::OwnerSigningKeyPair>> {
    let vault_id = crate::default_vault_path()?.to_string_lossy().into_owned();
    if let Ok(Some(signing_key)) =
        crate::get_owner_signing_key(&vault_id, VaultDirectory::DEFAULT_KEY_NAME)
    {
        return Ok(Some(signing_key));
    }

    if !crate::default_vault_path()?.exists() {
        return Ok(None);
    }
    if let Some(password) = crate::get_platform_vault_password().ok().flatten() {
        if let Some(key) = load_owner_signing_key_with_password(&password, &vault_id, false)? {
            return Ok(Some(key));
        }
    }
    if let Some(password) = crate::get_vault_unlock_key(&vault_id).ok().flatten() {
        if let Some(key) = load_owner_signing_key_with_password(&password, &vault_id, true)? {
            return Ok(Some(key));
        }
    }
    if let Some(password) = SecretString::try_from_env("LOCKBOX_VAULT_PASSWORD")? {
        if let Some(key) = load_owner_signing_key_with_password(&password, &vault_id, false)? {
            return Ok(Some(key));
        }
    }
    Ok(None)
}

fn load_owner_signing_key_with_password(
    password: &SecretString,
    vault_id: &str,
    invalidate_on_failure: bool,
) -> Result<Option<revault_lockbox_api::OwnerSigningKeyPair>> {
    let Ok(vault) = VaultDirectory::open_or_create_default(password) else {
        if invalidate_on_failure {
            let _ = crate::forget_vault_unlock_key(vault_id);
            let _ = crate::forget_owner_signing_key(vault_id, VaultDirectory::DEFAULT_KEY_NAME);
        }
        return Ok(None);
    };
    let Ok(signing_key) = vault.load_owner_signing_key(VaultDirectory::DEFAULT_KEY_NAME) else {
        if invalidate_on_failure {
            let _ = crate::forget_vault_unlock_key(vault_id);
            let _ = crate::forget_owner_signing_key(vault_id, VaultDirectory::DEFAULT_KEY_NAME);
        }
        return Ok(None);
    };
    let _ = crate::put_vault_unlock_key(vault_id, password.try_clone()?, None);
    let _ = crate::put_owner_signing_key(
        vault_id,
        VaultDirectory::DEFAULT_KEY_NAME,
        signing_key.try_clone()?,
        None,
    );
    Ok(Some(signing_key))
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
    contact: &revault_lockbox_api::ContactKeyPair,
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
