use super::Lockbox;
use crate::file_format::read_header;
#[cfg(feature = "vault-integration")]
use crate::key_directory::encode_key_directory;
use crate::key_directory::read_key_directory;
#[cfg(feature = "vault-integration")]
use crate::key_directory::read_key_directory_backup;
use crate::key_directory::{best_key_directory, scan_key_directories};
use crate::key_slot::{next_key_slot_id, random_content_key, random_salt, KeySlot, LockboxKeySlot};
use crate::key_wrap::{ContactKeyPair, ContactPublicKey};
use crate::lockbox_id::LockboxId;
use crate::secret_vec::{SecretString, SecretVec};
use crate::signing::OwnerSigningKeyPair;
use crate::storage::{Storage, StorageBackend};
use crate::{Error, LockboxEntryKind, LockboxOptions, ReadOnly, Result};
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

/// Decrypted content key produced by opening a key slot.
#[derive(Debug, PartialEq, Eq)]
pub struct OpenedContentKey {
    /// Lockbox id associated with the opened key.
    pub lockbox_id: LockboxId,
    key: SecretVec,
    read_only: bool,
}

/// Key material used when creating a new lockbox file.
///
/// `Password` and `ContactPublicKey` generate a fresh random content key and
/// store only a wrapped copy of that key in the lockbox key directory.
/// `ContentKey` is for callers that already manage the high-entropy secret
/// used to derive page encryption keys.
#[allow(clippy::large_enum_variant)]
pub enum LockboxProtection<'a> {
    /// Protect the lockbox directly with a caller-provided content key.
    ///
    /// The bytes should be a high-entropy application secret. They are not a
    /// password and are not stored as a key slot.
    ContentKey(SecretVec),
    /// Generate a content key and protect it with a password key slot.
    Password(&'a SecretString),
    /// Generate a content key and protect it with a contact public key.
    ///
    /// This is the public half of a hybrid contact keypair.
    ContactPublicKey {
        /// Optional human-readable recipient label stored with the key slot.
        name: Option<String>,
        /// Recipient public key used to wrap the generated content key.
        contact: ContactPublicKey,
    },
}

/// Key material used to open an existing lockbox file.
///
/// `Password` and `ContactKeyPair` unwrap a stored content key from the
/// lockbox key directory. `ContentKey` is for callers that already hold the
/// content key and therefore do not need a key slot.
#[allow(clippy::large_enum_variant)]
pub enum LockboxOpen<'a> {
    /// Open directly with a caller-provided content key.
    ///
    /// This must be the same high-entropy secret used with
    /// `LockboxProtection::ContentKey`.
    ContentKey(SecretVec),
    /// Open with a password key slot.
    Password(&'a SecretString),
    /// Open with a contact keypair.
    ///
    /// The keypair contains the private decapsulation material needed to unwrap
    /// a content key stored for its public key.
    ContactKeyPair(ContactKeyPair),
}

impl OpenedContentKey {
    /// Clone the decrypted content key into a new secure allocation.
    ///
    /// This lets vault integrations hand a copy to an external key cache
    /// without borrowing the original key under a secure read guard.
    #[cfg(feature = "vault-integration")]
    pub fn try_clone_key(&self) -> Result<SecretVec> {
        self.key.try_clone().map_err(Into::into)
    }

    /// Borrow the decrypted content key for the duration of the callback.
    ///
    /// Returns `Error::SecurityLimitExceeded` if secure memory access fails.
    #[cfg(feature = "vault-integration")]
    pub fn with_key<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Result<R> {
        Ok(self.key.with_bytes(f)?)
    }

    /// Open in-memory lockbox bytes with this opened content key.
    ///
    /// This is primarily useful for bytes-oriented callers such as WASM
    /// wrappers or vault integrations.
    #[cfg(feature = "vault-integration")]
    pub fn open_bytes(self, bytes: Vec<u8>) -> Result<Lockbox<ReadOnly>> {
        Ok(self.open_bytes_opened(bytes)?.into_state())
    }

    /// Open in-memory lockbox bytes for mutation with this content key.
    #[cfg(feature = "vault-integration")]
    pub fn open_bytes_for_write(
        self,
        bytes: Vec<u8>,
        signing_key: &OwnerSigningKeyPair,
    ) -> Result<Lockbox> {
        let mut lockbox = self.open_bytes_opened(bytes)?;
        if lockbox.read_only {
            return Err(Error::InvalidOperation(
                "lockbox was opened read-only".to_string(),
            ));
        }
        lockbox.set_owner_signing_key(signing_key.try_clone()?);
        Ok(lockbox)
    }

    fn open_bytes_opened(self, bytes: Vec<u8>) -> Result<Lockbox> {
        let mut lockbox = Lockbox::open_storage_with_secret_key(
            StorageBackend::memory(bytes),
            self.key,
            LockboxOptions::default(),
        )?;
        if self.read_only {
            lockbox.mark_read_only();
        }
        Ok(lockbox)
    }

    /// Open a lockbox file with this opened content key.
    ///
    /// Returns `Error::Io` if the host file cannot be read, `Error::InvalidKey`
    /// if authentication fails, or corrupt/truncated errors if the lockbox
    /// structure cannot be parsed.
    #[cfg(feature = "vault-integration")]
    pub fn open_path(self, path: &Path) -> Result<Lockbox<ReadOnly>> {
        Ok(self.open_path_opened(path)?.into_state())
    }

    /// Open a lockbox file for mutation with this content key.
    #[cfg(feature = "vault-integration")]
    pub fn open_path_for_write(
        self,
        path: &Path,
        signing_key: &OwnerSigningKeyPair,
    ) -> Result<Lockbox> {
        let mut lockbox = self.open_path_opened_for_write(path)?;
        if lockbox.read_only {
            return Err(Error::InvalidOperation(
                "lockbox was opened read-only".to_string(),
            ));
        }
        lockbox.set_owner_signing_key(signing_key.try_clone()?);
        Ok(lockbox)
    }

    fn open_path_opened(self, path: &Path) -> Result<Lockbox> {
        let mut lockbox =
            Lockbox::open_path_with_secret_key_options(path, self.key, LockboxOptions::default())?;
        if self.read_only {
            lockbox.mark_read_only();
        }
        Ok(lockbox)
    }

    #[cfg(feature = "vault-integration")]
    fn open_path_opened_for_write(self, path: &Path) -> Result<Lockbox> {
        let mut lockbox = Lockbox::open_path_with_secret_key_options_for_write(
            path,
            self.key,
            LockboxOptions::default(),
        )?;
        if self.read_only {
            lockbox.mark_read_only();
        }
        Ok(lockbox)
    }
}

impl Lockbox {
    /// Create a new in-memory lockbox using the supplied key material.
    ///
    /// This is the bytes-oriented counterpart to `Lockbox::create_file`.
    /// Call `commit` after mutations, then `try_to_bytes` to serialize the
    /// lockbox.
    pub fn create_in_memory(
        protection: LockboxProtection<'_>,
        signing_key: &OwnerSigningKeyPair,
    ) -> Result<Self> {
        let mut lockbox = Self::create_in_memory_uncommitted(protection)?;
        lockbox.set_owner_signing_key(signing_key.try_clone()?);
        lockbox.commit()?;
        Ok(lockbox)
    }

    fn create_in_memory_uncommitted(protection: LockboxProtection<'_>) -> Result<Self> {
        Ok(match protection {
            LockboxProtection::ContentKey(key) => Self::create_with_secret_key_and_options(
                key,
                LockboxId::new_random()?,
                LockboxOptions::default(),
            ),
            LockboxProtection::Password(password) => {
                let content_key = SecretVec::try_from_slice(&random_content_key()?)?;
                let mut lockbox = Self::create_with_secret_key_and_options(
                    content_key,
                    LockboxId::new_random()?,
                    LockboxOptions::default(),
                );
                lockbox.add_password(password)?;
                lockbox
            }
            LockboxProtection::ContactPublicKey { name, contact } => {
                let content_key = SecretVec::try_from_slice(&random_content_key()?)?;
                let mut lockbox = Self::create_with_secret_key_and_options(
                    content_key,
                    LockboxId::new_random()?,
                    LockboxOptions::default(),
                );
                match name {
                    Some(name) => {
                        lockbox.add_contact_named(name, &contact)?;
                    }
                    None => {
                        lockbox.add_contact(&contact)?;
                    }
                }
                lockbox
            }
        })
    }

    /// Create a new lockbox file using the supplied key material.
    ///
    /// Returns `Error::Io` if the host file cannot be created or written,
    /// `Error::SecurityLimitExceeded` if key material cannot be generated or
    /// wrapped, and storage/encoding errors from the initial commit.
    pub fn create_file(
        path: &Path,
        protection: LockboxProtection<'_>,
        signing_key: &OwnerSigningKeyPair,
    ) -> Result<Self> {
        let mut lockbox = Self::create_file_uncommitted(path, protection)?;
        lockbox.set_owner_signing_key(signing_key.try_clone()?);
        lockbox.commit()?;
        Ok(lockbox)
    }

    #[doc(hidden)]
    pub fn create_file_assuming_locked(
        path: &Path,
        protection: LockboxProtection<'_>,
        signing_key: &OwnerSigningKeyPair,
    ) -> Result<Self> {
        let mut lockbox = Self::create_file_uncommitted_assuming_locked(path, protection)?;
        lockbox.set_owner_signing_key(signing_key.try_clone()?);
        lockbox.commit()?;
        Ok(lockbox)
    }

    fn create_file_uncommitted(path: &Path, protection: LockboxProtection<'_>) -> Result<Self> {
        Ok(match protection {
            LockboxProtection::ContentKey(key) => Self::create_path_with_secret_key_and_options(
                path,
                key,
                LockboxId::new_random()?,
                LockboxOptions::default(),
            )?,
            LockboxProtection::Password(password) => {
                let content_key = SecretVec::try_from_slice(&random_content_key()?)?;
                let mut lockbox = Self::create_path_with_secret_key_and_options(
                    path,
                    content_key,
                    LockboxId::new_random()?,
                    LockboxOptions::default(),
                )?;
                lockbox.add_password(password)?;
                lockbox
            }
            LockboxProtection::ContactPublicKey { name, contact } => {
                let content_key = SecretVec::try_from_slice(&random_content_key()?)?;
                let mut lockbox = Self::create_path_with_secret_key_and_options(
                    path,
                    content_key,
                    LockboxId::new_random()?,
                    LockboxOptions::default(),
                )?;
                match name {
                    Some(name) => {
                        lockbox.add_contact_named(name, &contact)?;
                    }
                    None => {
                        lockbox.add_contact(&contact)?;
                    }
                }
                lockbox
            }
        })
    }

    fn create_file_uncommitted_assuming_locked(
        path: &Path,
        protection: LockboxProtection<'_>,
    ) -> Result<Self> {
        Ok(match protection {
            LockboxProtection::ContentKey(key) => {
                Self::create_path_with_secret_key_and_options_unlocked(
                    path,
                    key,
                    LockboxId::new_random()?,
                    LockboxOptions::default(),
                )?
            }
            LockboxProtection::Password(password) => {
                let content_key = SecretVec::try_from_slice(&random_content_key()?)?;
                let mut lockbox = Self::create_path_with_secret_key_and_options_unlocked(
                    path,
                    content_key,
                    LockboxId::new_random()?,
                    LockboxOptions::default(),
                )?;
                lockbox.add_password(password)?;
                lockbox
            }
            LockboxProtection::ContactPublicKey { name, contact } => {
                let content_key = SecretVec::try_from_slice(&random_content_key()?)?;
                let mut lockbox = Self::create_path_with_secret_key_and_options_unlocked(
                    path,
                    content_key,
                    LockboxId::new_random()?,
                    LockboxOptions::default(),
                )?;
                match name {
                    Some(name) => {
                        lockbox.add_contact_named(name, &contact)?;
                    }
                    None => {
                        lockbox.add_contact(&contact)?;
                    }
                }
                lockbox
            }
        })
    }

    /// Open an in-memory lockbox using the supplied open key material.
    pub fn open_bytes(bytes: Vec<u8>, open: LockboxOpen<'_>) -> Result<Lockbox<ReadOnly>> {
        Ok(Self::open_bytes_opened(bytes, open)?.into_state())
    }

    /// Open an in-memory lockbox for mutation and attach `signing_key` for commits.
    pub fn open_bytes_for_write(
        bytes: Vec<u8>,
        open: LockboxOpen<'_>,
        signing_key: &OwnerSigningKeyPair,
    ) -> Result<Self> {
        let mut lockbox = Self::open_bytes_opened(bytes, open)?;
        lockbox.read_only = false;
        lockbox.set_owner_signing_key(signing_key.try_clone()?);
        Ok(lockbox)
    }

    fn open_bytes_opened(bytes: Vec<u8>, open: LockboxOpen<'_>) -> Result<Self> {
        match open {
            LockboxOpen::ContentKey(key) => Self::open_storage_with_secret_key(
                StorageBackend::memory(bytes),
                key,
                LockboxOptions::default(),
            ),
            LockboxOpen::Password(password) => {
                let opened = Self::open_bytes_with_password(&bytes, password)?;
                opened.open_bytes_opened(bytes)
            }
            LockboxOpen::ContactKeyPair(contact) => {
                let opened = Self::open_bytes_with_contact(&bytes, &contact)?;
                opened.open_bytes_opened(bytes)
            }
        }
    }

    /// Open an existing lockbox file using the supplied open key material.
    ///
    /// Password and contact opens use only key slots embedded in the
    /// lockbox file. This method does not read the local vault, cached content
    /// keys, or vault-stored key-directory backups. Use `revault_vault_api::Vault`
    /// when that behavior is required.
    ///
    /// Returns `Error::Io` if the host file cannot be read, `Error::InvalidKey`
    /// when the supplied open material cannot authenticate the content key, or
    /// corrupt/truncated errors if the lockbox structure cannot be parsed.
    pub fn open(path: &Path, open: LockboxOpen<'_>) -> Result<Lockbox<ReadOnly>> {
        Ok(Self::open_file_opened(path, open)?.into_state())
    }

    /// Open a lockbox file for mutation and attach `signing_key` for commits.
    pub fn open_for_write(
        path: &Path,
        open: LockboxOpen<'_>,
        signing_key: &OwnerSigningKeyPair,
    ) -> Result<Self> {
        let mut lockbox = Self::open_file_opened_for_write(path, open)?;
        lockbox.read_only = false;
        lockbox.set_owner_signing_key(signing_key.try_clone()?);
        Ok(lockbox)
    }

    /// Open a lockbox file for mutation after loading its owner signing key.
    ///
    /// This is for lockboxes that store their own owner signing key, such as
    /// the local vault. The callback receives a read-only borrow of the opened
    /// lockbox and must return the key that will sign future commits.
    pub fn open_for_write_with_signing_key(
        path: &Path,
        open: LockboxOpen<'_>,
        load_signing_key: impl FnOnce(&Lockbox<ReadOnly>) -> Result<OwnerSigningKeyPair>,
    ) -> Result<Self> {
        let mut lockbox = Self::open_file_opened_for_write(path, open)?;
        let read_view = lockbox.try_clone()?.into_state();
        let signing_key = load_signing_key(&read_view)?;
        lockbox.read_only = false;
        lockbox.set_owner_signing_key(signing_key);
        Ok(lockbox)
    }

    #[doc(hidden)]
    pub fn open_for_write_with_signing_key_assuming_locked(
        path: &Path,
        open: LockboxOpen<'_>,
        load_signing_key: impl FnOnce(&Lockbox<ReadOnly>) -> Result<OwnerSigningKeyPair>,
    ) -> Result<Self> {
        let mut lockbox = Self::open_file_opened(path, open)?;
        let read_view = lockbox.try_clone()?.into_state();
        let signing_key = load_signing_key(&read_view)?;
        lockbox.read_only = false;
        lockbox.set_owner_signing_key(signing_key);
        Ok(lockbox)
    }

    fn open_file_opened(path: &Path, open: LockboxOpen<'_>) -> Result<Self> {
        match open {
            LockboxOpen::ContentKey(key) => {
                Self::open_path_with_secret_key_options(path, key, LockboxOptions::default())
            }
            LockboxOpen::Password(password) => {
                let opened = Self::open_path_with_password(path, password)?;
                opened.open_path_opened(path)
            }
            LockboxOpen::ContactKeyPair(contact) => {
                let opened = Self::open_path_with_contact(path, &contact)?;
                opened.open_path_opened(path)
            }
        }
    }

    fn open_file_opened_for_write(path: &Path, open: LockboxOpen<'_>) -> Result<Self> {
        let storage = StorageBackend::file_for_write(path)?;
        Self::open_locked_storage(storage, open)
    }

    fn open_locked_storage(storage: StorageBackend, open: LockboxOpen<'_>) -> Result<Self> {
        match open {
            LockboxOpen::ContentKey(key) => {
                Self::open_storage_with_secret_key(storage, key, LockboxOptions::default())
            }
            LockboxOpen::Password(password) => {
                let bytes = storage.read_all()?;
                let opened = Self::open_bytes_with_password(&bytes, password)?;
                let mut lockbox = Self::open_storage_with_secret_key(
                    storage,
                    opened.key,
                    LockboxOptions::default(),
                )?;
                if opened.read_only {
                    lockbox.mark_read_only();
                }
                Ok(lockbox)
            }
            LockboxOpen::ContactKeyPair(contact) => {
                let bytes = storage.read_all()?;
                let opened = Self::open_bytes_with_contact(&bytes, &contact)?;
                let mut lockbox = Self::open_storage_with_secret_key(
                    storage,
                    opened.key,
                    LockboxOptions::default(),
                )?;
                if opened.read_only {
                    lockbox.mark_read_only();
                }
                Ok(lockbox)
            }
        }
    }

    #[cfg(feature = "vault-integration")]
    pub(crate) fn read_lockbox_id(path: &Path) -> Result<LockboxId> {
        let storage = StorageBackend::file(path)?;
        let header = storage.read_at(0, crate::constants::HEADER_LEN)?;
        crate::file_format::header::read_lockbox_id(&header)
    }

    #[cfg(any(test, feature = "bindings"))]
    /// Creates an uncommitted in-memory lockbox protected by `password`.
    ///
    /// A random content key is generated and wrapped in a password key slot.
    /// Call [`Lockbox::commit`] before serializing the lockbox.
    pub fn create_with_password(password: &SecretString) -> Result<Self> {
        let content_key = random_content_key()?;
        let mut lockbox = Self::create(content_key);
        lockbox.add_password(password)?;
        Ok(lockbox)
    }

    pub(crate) fn open_bytes_with_password(
        bytes: &[u8],
        password: &SecretString,
    ) -> Result<OpenedContentKey> {
        for directory in key_directories_from_bytes(bytes)? {
            for slot in directory.slots {
                let Ok(key) = slot.try_password(password) else {
                    continue;
                };
                return Ok(OpenedContentKey {
                    lockbox_id: directory.lockbox_id,
                    key: SecretVec::try_from_vec(key)?,
                    read_only: false,
                });
            }
        }
        Err(Error::InvalidKey)
    }

    /// Open a lockbox file with a password and return its decrypted content key.
    pub(crate) fn open_path_with_password(
        path: &Path,
        password: &SecretString,
    ) -> Result<OpenedContentKey> {
        let storage = StorageBackend::file(path)?;
        for directory in key_directories_from_storage(&storage)? {
            for slot in directory.slots {
                let Ok(key) = slot.try_password(password) else {
                    continue;
                };
                return Ok(OpenedContentKey {
                    lockbox_id: directory.lockbox_id,
                    key: SecretVec::try_from_vec(key)?,
                    read_only: false,
                });
            }
        }
        Err(Error::InvalidKey)
    }

    /// Open a key-directory backup with a password.
    #[cfg(feature = "vault-integration")]
    pub(crate) fn open_key_directory_backup_with_password(
        bytes: &[u8],
        password: &SecretString,
    ) -> Result<OpenedContentKey> {
        let directory = read_key_directory_backup(bytes)?;
        for slot in directory.slots {
            let Ok(key) = slot.try_password(password) else {
                continue;
            };
            return Ok(OpenedContentKey {
                lockbox_id: directory.lockbox_id,
                key: SecretVec::try_from_vec(key)?,
                read_only: false,
            });
        }
        Err(Error::InvalidKey)
    }

    #[cfg(any(test, feature = "bindings"))]
    /// Creates an uncommitted in-memory lockbox for a contact public key.
    ///
    /// A random content key is generated and wrapped to `contact`. The contact
    /// recipient receives read-only access unless an owner signing key is set.
    pub fn create_with_contact(contact: &ContactPublicKey) -> Result<Self> {
        let content_key = random_content_key()?;
        let mut lockbox = Self::create(content_key);
        lockbox.add_contact(contact)?;
        Ok(lockbox)
    }

    #[cfg(any(test, feature = "bindings"))]
    /// Opens in-memory lockbox bytes for writing using a password key slot.
    ///
    /// Returns [`Error::InvalidKey`] when no password slot can be unwrapped.
    pub fn open_with_password(bytes: Vec<u8>, password: &SecretString) -> Result<Self> {
        let opened = Self::open_bytes_with_password(&bytes, password)?;
        opened.open_bytes_opened(bytes)
    }

    #[cfg(any(test, feature = "bindings"))]
    /// Opens in-memory lockbox bytes using the matching contact private key.
    ///
    /// Contact recipients open read-only unless the caller subsequently
    /// supplies the owner signing key through the supported binding API.
    pub fn open_with_contact(bytes: Vec<u8>, contact: &ContactKeyPair) -> Result<Self> {
        let opened = Self::open_bytes_with_contact(&bytes, contact)?;
        opened.open_bytes_opened(bytes)
    }

    pub(crate) fn open_bytes_with_contact(
        bytes: &[u8],
        contact: &ContactKeyPair,
    ) -> Result<OpenedContentKey> {
        for directory in key_directories_from_bytes(bytes)? {
            for slot in directory.slots {
                let Ok(key) = slot.try_contact(contact) else {
                    continue;
                };
                return Ok(OpenedContentKey {
                    lockbox_id: directory.lockbox_id,
                    key: SecretVec::try_from_vec(key)?,
                    read_only: true,
                });
            }
        }
        Err(Error::InvalidKey)
    }

    /// Open a lockbox file with a contact private key.
    pub(crate) fn open_path_with_contact(
        path: &Path,
        contact: &ContactKeyPair,
    ) -> Result<OpenedContentKey> {
        let storage = StorageBackend::file(path)?;
        for directory in key_directories_from_storage(&storage)? {
            for slot in directory.slots {
                let Ok(key) = slot.try_contact(contact) else {
                    continue;
                };
                return Ok(OpenedContentKey {
                    lockbox_id: directory.lockbox_id,
                    key: SecretVec::try_from_vec(key)?,
                    read_only: true,
                });
            }
        }
        Err(Error::InvalidKey)
    }

    /// Open a key-directory backup with a contact private key.
    #[cfg(feature = "vault-integration")]
    pub(crate) fn open_key_directory_backup_with_contact(
        bytes: &[u8],
        contact: &ContactKeyPair,
    ) -> Result<OpenedContentKey> {
        let directory = read_key_directory_backup(bytes)?;
        for slot in directory.slots {
            let Ok(key) = slot.try_contact(contact) else {
                continue;
            };
            return Ok(OpenedContentKey {
                lockbox_id: directory.lockbox_id,
                key: SecretVec::try_from_vec(key)?,
                read_only: true,
            });
        }
        Err(Error::InvalidKey)
    }

    /// Add another password that can open this lockbox and return its key id.
    ///
    /// A password does not encrypt file content directly. The lockbox content
    /// key is random; each password wraps that same content key in an embedded
    /// key-directory entry. `Lockbox::open` with `LockboxOpen::Password`
    /// tries each embedded password entry until one unwraps the content key.
    ///
    /// Returns `Error::Io` if random salt generation fails,
    /// `Error::InvalidInput` if internal password-derivation parameters are
    /// invalid, `Error::InvalidKey` if authenticated key wrapping fails, or
    /// `Error::SecurityLimitExceeded` if secure memory access fails.
    pub fn add_password(&mut self, password: &SecretString) -> Result<u64> {
        let id = next_key_slot_id(&self.key_slots);
        let salt = random_salt()?;
        let slot = revault_page_api::read_access(|access| {
            access.with_bytes(&self.key, |content_key| {
                password.with_bytes_in(access, |password| {
                    KeySlot::password_bytes(id, password, salt, content_key)
                })
            })
        })???;
        self.key_slots.push(slot);
        self.mark_key_directory_dirty();
        Ok(id)
    }

    /// Add a contact public key to the lockbox and return its key id.
    ///
    /// Once a contact's public key has been added to a lockbox, the matching
    /// contact's private keypair can be used to open the lockbox. Add
    /// contacts with their public key, not their private keypair.
    ///
    /// To add a contact to the box, you must be able to open the box with
    /// your own key.
    ///
    /// Returns `Error::SecurityLimitExceeded` if secure key access or key
    /// wrapping fails.
    pub fn add_contact(&mut self, contact: &ContactPublicKey) -> Result<u64> {
        let id = next_key_slot_id(&self.key_slots);
        let slot = self
            .key
            .with_bytes(|content_key| KeySlot::hybrid_contact(id, contact, content_key))??;
        self.key_slots.push(slot);
        self.mark_key_directory_dirty();
        Ok(id)
    }

    /// Add a contact public key selected by a local name and return its key id.
    ///
    /// The name is validated for caller-side label storage, but is not stored
    /// in the lockbox. Persisting names would leak who can open a shared
    /// lockbox.
    pub fn add_contact_named(
        &mut self,
        name: impl Into<String>,
        contact: &ContactPublicKey,
    ) -> Result<u64> {
        let name = name.into();
        crate::key_slot::validate_key_slot_name(&name)?;
        self.add_contact(contact)
    }

    fn remove_key_slot(&mut self, id: u64) -> Result<()> {
        let before = self.key_slots.len();
        self.key_slots.retain(|slot| slot.id() != id);
        if self.key_slots.len() == before {
            return Err(Error::NotFound(format!("key slot {id}")));
        }
        self.mark_key_directory_dirty();
        Ok(())
    }

    /// Delete a key from the lockbox and compact obsolete key directory pages.
    ///
    /// Returns `Error::NotFound` if `id` does not exist,
    /// `Error::SecurityLimitExceeded` when attempting to remove the last key,
    /// or storage/encoding errors if compaction fails.
    pub fn delete_key(&mut self, id: u64) -> Result<()> {
        self.remove_key_slot_and_compact(id)
    }

    fn remove_key_slot_and_compact(&mut self, id: u64) -> Result<()> {
        let Some(index) = self.key_slots.iter().position(|slot| slot.id() == id) else {
            return Err(Error::NotFound(format!("key slot {id}")));
        };
        if self.key_slots.len() == 1 {
            return Err(Error::SecurityLimitExceeded(
                "refusing to remove the last key slot".to_string(),
            ));
        }
        let removed = self.key_slots.remove(index);
        self.mark_key_directory_dirty();
        let result = self.compact();
        if result.is_err() {
            self.key_slots.insert(index, removed);
            self.mark_key_directory_dirty();
        }
        result
    }

    /// Export a backup copy of the key directory.
    ///
    /// Returns storage/encoding errors if the key directory cannot be encoded.
    #[cfg(any(feature = "vault-integration", feature = "migration"))]
    pub(crate) fn export_key_directory_backup(&self) -> Result<Vec<u8>> {
        encode_key_directory(
            &self.key_slots,
            self.lockbox_id,
            self.key_directory_generation,
            0,
        )
    }

    /// Restores access slots exported with the same content key while creating
    /// a new native archive representation.
    #[cfg(feature = "migration")]
    #[doc(hidden)]
    pub fn import_migration_key_directory(&mut self, bytes: &[u8]) -> Result<()> {
        let decoded = crate::key_directory::read_key_directory_backup(bytes)?;
        if decoded.lockbox_id != self.lockbox_id {
            return Err(Error::CorruptHeader);
        }
        self.key_slots = decoded.slots;
        self.key_directory_generation = decoded.generation;
        self.mark_key_directory_dirty();
        Ok(())
    }

    /// List the keys that can open this lockbox.
    pub fn list_key_slots(&self) -> Vec<LockboxKeySlot> {
        self.key_slots.iter().map(KeySlot::info).collect()
    }

    /// Replace one existing password with a new password and return the new key id.
    ///
    /// The old password is used only to find a matching embedded password entry.
    /// Other passwords and contact keys are left unchanged. Returns
    /// `Error::InvalidKey` if no embedded password entry matches `old_password`;
    /// returns storage or encoding errors if compaction fails after the
    /// replacement.
    pub fn replace_password(
        &mut self,
        old_password: &SecretString,
        new_password: &SecretString,
    ) -> Result<u64> {
        let mut matching_id = None;
        for slot in &self.key_slots {
            if slot.try_password(old_password).is_ok() {
                matching_id = Some(slot.id());
                break;
            }
        }
        let Some(old_id) = matching_id else {
            return Err(Error::InvalidKey);
        };
        let new_id = self.add_password(new_password)?;
        self.remove_key_slot(old_id)?;
        self.compact()?;
        Ok(new_id)
    }

    pub(crate) fn compact(&mut self) -> Result<()> {
        let entries = self
            .toc_entries
            .values()
            .filter(|entry| !entry.deleted)
            .cloned()
            .collect::<Vec<_>>();
        let variables = self.clone_all_variable_values()?;
        let forms = self.clone_all_form_state()?;
        if let Some(path) = self.storage.path().map(Path::to_path_buf) {
            return self.compact_file_backed(path, entries, variables, forms);
        }

        let key = self.key.try_clone()?;
        let signing_key = self.require_owner_signing_key()?.try_clone()?;
        let mut compacted = Lockbox::create_with_secret_key_and_options(
            key,
            self.lockbox_id,
            self.compaction_options(),
        );
        compacted.set_owner_signing_key(signing_key);
        self.populate_compacted(&mut compacted, entries, variables, forms)?;
        compacted.commit()?;
        *self = compacted;
        Ok(())
    }

    /// Replace the lockbox content key and grant access to the supplied contacts.
    ///
    /// This is the low-level primitive for true revocation. It rewrites the
    /// archive with a fresh content key and creates a new key directory
    /// containing only `retained_contacts`. Password slots and contacts not
    /// supplied by the caller are intentionally not preserved.
    pub fn replace_content_key_with_contacts(
        &mut self,
        retained_contacts: &[(String, ContactPublicKey)],
    ) -> Result<Vec<(String, u64)>> {
        if retained_contacts.is_empty() {
            return Err(Error::SecurityLimitExceeded(
                "refusing to rekey without retained access".to_string(),
            ));
        }
        let entries = self
            .toc_entries
            .values()
            .filter(|entry| !entry.deleted)
            .cloned()
            .collect::<Vec<_>>();
        let variables = self.clone_all_variable_values()?;
        let forms = self.clone_all_form_state()?;
        if let Some(path) = self.storage.path().map(Path::to_path_buf) {
            return self.rekey_file_backed(path, entries, variables, forms, retained_contacts);
        }

        let key = SecretVec::try_from_slice(&random_content_key()?)?;
        let signing_key = self.require_owner_signing_key()?.try_clone()?;
        let mut rekeyed = Lockbox::create_with_secret_key_and_options(
            key,
            self.lockbox_id,
            self.compaction_options(),
        );
        rekeyed.set_owner_signing_key(signing_key);
        let slot_ids = add_retained_contacts(&mut rekeyed, retained_contacts)?;
        self.populate_compacted_content(&mut rekeyed, entries, variables, forms)?;
        rekeyed.commit()?;
        *self = rekeyed;
        Ok(slot_ids)
    }

    fn rekey_file_backed(
        &mut self,
        path: PathBuf,
        entries: Vec<crate::toc_entry::TocEntry>,
        variables: std::collections::BTreeMap<
            crate::VariableName,
            crate::variable_btree::VariableValue,
        >,
        forms: (
            std::collections::BTreeMap<String, crate::form::FormDefinition>,
            std::collections::BTreeMap<crate::LockboxPath, crate::form::FormRecord>,
        ),
        retained_contacts: &[(String, ContactPublicKey)],
    ) -> Result<Vec<(String, u64)>> {
        let temp_path = compact_temp_path(&path);
        let _ = fs::remove_file(&temp_path);
        let options = self.compaction_options();
        let signing_key = self.require_owner_signing_key()?.try_clone()?;
        let result = (|| {
            let key = SecretVec::try_from_slice(&random_content_key()?)?;
            let reopen_key = key.try_clone()?;
            let mut rekeyed = Lockbox::create_path_with_secret_key_and_options(
                &temp_path,
                key,
                self.lockbox_id,
                options,
            )?;
            rekeyed.set_owner_signing_key(signing_key.try_clone()?);
            let slot_ids = add_retained_contacts(&mut rekeyed, retained_contacts)?;
            self.populate_compacted_content(&mut rekeyed, entries, variables, forms)?;
            rekeyed.commit()?;
            drop(rekeyed);
            replace_file_with_compacted(&temp_path, &path)?;
            let mut reopened =
                Lockbox::open_path_with_secret_key_options(&path, reopen_key, options)?;
            reopened.set_owner_signing_key(signing_key);
            *self = reopened;
            Ok(slot_ids)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }
        result
    }

    fn compact_file_backed(
        &mut self,
        path: PathBuf,
        entries: Vec<crate::toc_entry::TocEntry>,
        variables: std::collections::BTreeMap<
            crate::VariableName,
            crate::variable_btree::VariableValue,
        >,
        forms: (
            std::collections::BTreeMap<String, crate::form::FormDefinition>,
            std::collections::BTreeMap<crate::LockboxPath, crate::form::FormRecord>,
        ),
    ) -> Result<()> {
        let temp_path = compact_temp_path(&path);
        let _ = fs::remove_file(&temp_path);
        let options = self.compaction_options();
        let signing_key = self.require_owner_signing_key()?.try_clone()?;
        let result = (|| {
            let key = self.key.try_clone()?;
            let reopen_key = key.try_clone()?;
            let mut compacted = Lockbox::create_path_with_secret_key_and_options(
                &temp_path,
                key,
                self.lockbox_id,
                options,
            )?;
            compacted.set_owner_signing_key(signing_key.try_clone()?);
            self.populate_compacted(&mut compacted, entries, variables, forms)?;
            compacted.commit()?;
            drop(compacted);
            replace_file_with_compacted(&temp_path, &path)?;
            let mut reopened =
                Lockbox::open_path_with_secret_key_options(&path, reopen_key, options)?;
            reopened.set_owner_signing_key(signing_key);
            *self = reopened;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }
        result
    }

    fn populate_compacted(
        &self,
        compacted: &mut Lockbox,
        entries: Vec<crate::toc_entry::TocEntry>,
        variables: std::collections::BTreeMap<
            crate::VariableName,
            crate::variable_btree::VariableValue,
        >,
        forms: (
            std::collections::BTreeMap<String, crate::form::FormDefinition>,
            std::collections::BTreeMap<crate::LockboxPath, crate::form::FormRecord>,
        ),
    ) -> Result<()> {
        compacted.key_slots = self.key_slots.clone();
        compacted.key_directory_generation = self.key_directory_generation;
        compacted.dirty_key_directory = !compacted.key_slots.is_empty();

        self.populate_compacted_content(compacted, entries, variables, forms)
    }

    fn populate_compacted_content(
        &self,
        compacted: &mut Lockbox,
        entries: Vec<crate::toc_entry::TocEntry>,
        variables: std::collections::BTreeMap<
            crate::VariableName,
            crate::variable_btree::VariableValue,
        >,
        forms: (
            std::collections::BTreeMap<String, crate::form::FormDefinition>,
            std::collections::BTreeMap<crate::LockboxPath, crate::form::FormRecord>,
        ),
    ) -> Result<()> {
        for (name, value) in variables {
            compacted.set_variable_value(name, value)?;
        }
        for (key, definition) in forms.0 {
            compacted.set_form_definition_value(key, definition)?;
        }
        for entry in entries
            .iter()
            .filter(|entry| entry.entry_kind() == LockboxEntryKind::Directory)
        {
            compacted.create_dir(&entry.path, true)?;
            compacted.set_permissions(&entry.path, entry.permissions)?;
        }
        for (path, record) in forms.1 {
            compacted.create_parent_dirs_for(&path)?;
            compacted.set_form_record_value(path, record)?;
        }

        for entry in entries {
            match entry.entry_kind() {
                LockboxEntryKind::File => {
                    let reader = FileEntryReader::new(self, &entry)?;
                    compacted.create_parent_dirs_for(&entry.path)?;
                    compacted.add_file_from_reader_with_permissions(
                        &entry.path,
                        reader,
                        entry.permissions,
                        false,
                    )?;
                }
                LockboxEntryKind::Symlink => {
                    let target = self.get_symlink_target(&entry.path)?;
                    compacted.create_parent_dirs_for(&entry.path)?;
                    compacted.add_symlink(&entry.path, &target, false)?;
                }
                LockboxEntryKind::Directory => {}
            }
        }
        Ok(())
    }

    fn compaction_options(&self) -> LockboxOptions {
        LockboxOptions {
            workload_profile: self.workload_profile,
            ..LockboxOptions::default()
        }
    }
}

fn compact_temp_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("lockbox");
    path.with_file_name(format!(".{file_name}.compact-{}", std::process::id()))
}

fn replace_file_with_compacted(temp_path: &Path, path: &Path) -> Result<()> {
    fs::rename(temp_path, path).map_err(|err| {
        Error::Io(format!(
            "replace compacted lockbox {}: {err}",
            path.display()
        ))
    })?;
    sync_parent_dir(path)
}

fn sync_parent_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let dir = fs::File::open(parent)
            .map_err(|err| Error::Io(format!("open {}: {err}", parent.display())))?;
        dir.sync_data()
            .map_err(|err| Error::Io(format!("sync {}: {err}", parent.display())))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn add_retained_contacts(
    lockbox: &mut Lockbox,
    retained_contacts: &[(String, ContactPublicKey)],
) -> Result<Vec<(String, u64)>> {
    let mut slot_ids = Vec::with_capacity(retained_contacts.len());
    for (name, contact) in retained_contacts {
        let slot_id = lockbox.add_contact_named(access_entry_name(name), contact)?;
        slot_ids.push((name.clone(), slot_id));
    }
    Ok(slot_ids)
}

fn access_entry_name(label: &str) -> String {
    label
        .strip_prefix("profile:")
        .or_else(|| label.strip_prefix("contact:"))
        .unwrap_or(label)
        .to_string()
}

impl<State> Lockbox<State> {
    /// Exports the content key and encoded access directory for a migration
    /// artifact. Both values must remain inside zeroizing/encrypted migration
    /// handling and must never be displayed.
    #[cfg(feature = "migration")]
    #[doc(hidden)]
    pub fn export_migration_key_material(&self) -> Result<(SecretVec, Vec<u8>)> {
        Ok((
            self.key.try_clone()?,
            encode_key_directory(
                &self.key_slots,
                self.lockbox_id,
                self.key_directory_generation,
                0,
            )?,
        ))
    }

    pub(crate) fn mark_key_directory_dirty(&mut self) {
        self.key_directory_generation = self.key_directory_generation.saturating_add(1);
        self.dirty_key_directory = true;
    }
}

struct FileEntryReader<'a> {
    lockbox: &'a Lockbox,
    entry: &'a crate::toc_entry::TocEntry,
    chunks: Vec<crate::file_chunk::FileChunk>,
    next_chunk: usize,
    current: Cursor<Vec<u8>>,
    written: u64,
}

impl<'a> FileEntryReader<'a> {
    fn new(lockbox: &'a Lockbox, entry: &'a crate::toc_entry::TocEntry) -> Result<Self> {
        if let Some(pending) = lockbox.pending_small_files.get(&entry.path) {
            if pending.data.len() as u64 != entry.len {
                return Err(Error::CorruptRecord);
            }
            return Ok(Self {
                lockbox,
                entry,
                chunks: Vec::new(),
                next_chunk: 0,
                current: Cursor::new(pending.data.to_vec()),
                written: 0,
            });
        }
        if entry.chunks.is_empty() {
            return Err(Error::CorruptRecord);
        }
        let mut chunks = entry.chunks.clone();
        chunks.sort_by_key(|chunk| chunk.file_offset);
        Ok(Self {
            lockbox,
            entry,
            chunks,
            next_chunk: 0,
            current: Cursor::new(Vec::new()),
            written: 0,
        })
    }
}

impl Read for FileEntryReader<'_> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let read = self.current.read(out)?;
            if read != 0 {
                self.written = self.written.saturating_add(read as u64);
                return Ok(read);
            }
            if self.next_chunk >= self.chunks.len() {
                if self.written == self.entry.len {
                    return Ok(0);
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "lockbox file length mismatch during compaction",
                ));
            }
            let chunk = &self.chunks[self.next_chunk];
            if chunk.file_offset != self.written {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "lockbox file chunk offset mismatch during compaction",
                ));
            }
            self.next_chunk += 1;
            let decoded = self
                .lockbox
                .read_file_chunk_compression_frame(self.entry.len, chunk)
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
            self.current = Cursor::new(decoded);
        }
    }
}

fn key_directories_from_bytes(
    bytes: &[u8],
) -> Result<Vec<crate::key_directory::DecodedKeyDirectory>> {
    let mut directories = Vec::new();
    if let Ok(header) = read_header(bytes) {
        let key_directory_offset = header.key_directory_offset;
        let lockbox_id = header.lockbox_id;
        if let Ok(directory) = read_key_directory(bytes, key_directory_offset, Some(lockbox_id)) {
            directories.push(directory);
        }
        directories.extend(scan_key_directories(bytes, Some(lockbox_id)));
    } else {
        directories.extend(scan_key_directories(bytes, None));
    }
    if directories.is_empty() {
        return Err(Error::CorruptHeader);
    }
    let Some(best) = best_key_directory(directories.clone()) else {
        return Err(Error::CorruptHeader);
    };
    directories.sort_by_key(|directory| {
        (
            std::cmp::Reverse(directory.lockbox_id == best.lockbox_id),
            std::cmp::Reverse(directory.generation),
            directory.copy_index,
        )
    });
    Ok(directories)
}

pub(crate) fn key_directories_from_storage(
    storage: &StorageBackend,
) -> Result<Vec<crate::key_directory::DecodedKeyDirectory>> {
    let header = storage.read_at(0, crate::constants::HEADER_LEN)?;
    let mut directories = Vec::new();
    if let Ok(header) = read_header(&header) {
        let key_directory_offset = header.key_directory_offset;
        let lockbox_id = header.lockbox_id;
        if let Ok(directory) = crate::key_directory::read_key_directory_via_page_cache(
            storage,
            key_directory_offset,
            Some(lockbox_id),
        ) {
            directories.push(directory);
        }
        if directories.is_empty() {
            let bytes = storage.read_all()?;
            directories.extend(scan_key_directories(&bytes, Some(lockbox_id)));
        }
    } else {
        let bytes = storage.read_all()?;
        directories.extend(scan_key_directories(&bytes, None));
    }
    if directories.is_empty() {
        return Err(Error::CorruptHeader);
    }
    let Some(best) = best_key_directory(directories.clone()) else {
        return Err(Error::CorruptHeader);
    };
    directories.sort_by_key(|directory| {
        (
            std::cmp::Reverse(directory.lockbox_id == best.lockbox_id),
            std::cmp::Reverse(directory.generation),
            directory.copy_index,
        )
    });
    Ok(directories)
}
