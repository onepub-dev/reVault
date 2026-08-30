#![deny(missing_docs)]
//! Complete source-native Rust API for reVault `Lockbox` archives and `Vault`s.
//!
//! Use [`lockbox`] for portable encrypted archives and [`vault`] for local key,
//! Contact, Form, platform credential store, and Session Agent operations. This package
//! re-exports the native core directly; transport framing exists only at the
//! foreign-language boundary. Rust links the implementation at build time, so
//! there is no runtime library path or `REVAULT_LIBRARY` setting.
//!
//! See the [reVault repository README](https://github.com/onepub-dev/reVault#readme)
//! for installation, security guidance, and examples.

/// Portable encrypted archive API.
pub mod lockbox {
    pub use revault_lockbox_api::{
        ArtifactKind, CacheLimit, CacheStats, ContactKeyPair, ContactPublicKey, ContentChunk,
        ContentStreamOptions, ContentStreamOrder, Error, ExtractPolicy, FormDefinition,
        FormFieldDefinition, FormFieldKind, FormFieldValue, FormRecord, FormTypeId, FormValue,
        ImportStats, ListOptions, Lockbox, LockboxEntry, LockboxEntryKind, LockboxFileInspection,
        LockboxFileMut, LockboxFileReader, LockboxId, LockboxInspector, LockboxKeySlot,
        LockboxKeySlotAlgorithm, LockboxKeySlotProtection, LockboxOpen, LockboxOptions,
        LockboxOwnerInspection, LockboxPath, LockboxProtection, MirrorMissingFilePolicy,
        MirrorProject, OpenFileOptions, PageInspection, PageObjectInspection, ReadOnly,
        RecoveryReport, RecoveryReportOptions, RecoveryScanner, Result, SecretString, SecretVec,
        VariableName, VariableNamePattern, VariableSensitivity, VariableValueRef, WorkerPolicy,
        WorkloadProfile, Writable, WritableLockboxState,
    };

    /// A profile's signing identity. The owner role is assigned only when the
    /// key is attached to a [`Lockbox`].
    pub type ProfileSigningKeyPair = revault_lockbox_api::OwnerSigningKeyPair;

    /// The shareable verification half of a profile signing identity.
    pub type ProfileSigningPublicKey = revault_lockbox_api::OwnerSigningPublicKey;

    /// A contact-encrypted content-key envelope.
    pub type WrappedContactKey = revault_lockbox_api::ContactWrappedKey;
}

/// Vault, platform credential store, and Session Agent API.
pub mod vault {
    use revault_lockbox_api::{ContactKeyPair, ContactPublicKey};

    pub use revault_vault_api::{
        backup_default_vault, decode_fingerprint_crockford_96, decode_fingerprint_hex,
        default_vault_dir, default_vault_path, disable_platform_secret_store,
        enable_platform_secret_store, encode_hex, export_private_key, export_public_key, forget,
        forget_all, forget_platform_vault_password, forget_vault_unlock_key,
        format_fingerprint_crockford_96, format_fingerprint_crockford_96_reading,
        format_fingerprint_hex_pairs, get, get_platform_vault_password, get_vault_unlock_key,
        import_private_key, import_private_key_file, import_public_key, is_running, list,
        platform_secret_store_disabled, platform_secret_store_status, public_key_fingerprint, put,
        put_platform_vault_password, put_vault_unlock_key, restore_default_vault, serve_agent,
        set_auto_open_scope, start, stop, verify_agent_transport_security, AccessSlotLabel,
        AgentSleepSupport, AutoOpenScope, CachedLockbox, ContentKeyStore, KeyFormat, KnownLockbox,
        NoopStore, PlatformSecretStoreStatus, ProfileGeneration, ProfileGenerationStatus,
        ProfileHistory, SecretActivityGuard, SecretActivityKind, SecretString, SecretVec,
        StoredContact, VaultBackupManifest, CURRENT_VAULT_STRUCTURE_VERSION,
        FINGERPRINT_CODE_96_LEN,
    };

    /// Persistent encrypted local store for profiles, keys, contacts, and
    /// remembered lockbox metadata.
    ///
    /// The implementation handle is private: callers see the reviewed
    /// `Vault` name and cannot depend on the core storage type.
    #[derive(Debug)]
    pub struct Vault(revault_vault_api::VaultDirectory);

    impl Vault {
        /// Opens the default persistent store.
        pub fn open_default(password: &SecretString) -> revault_lockbox_api::Result<Self> {
            let path = revault_vault_api::default_vault_path()?;
            revault_vault_api::VaultDirectory::open_file(path, password).map(Self)
        }
        /// Opens or creates the default persistent store.
        pub fn open_or_create_default(
            password: &SecretString,
        ) -> revault_lockbox_api::Result<Self> {
            revault_vault_api::VaultDirectory::open_or_create_default(password).map(Self)
        }
        /// Creates a new store at an explicit file path.
        pub fn create_file(
            path: impl AsRef<std::path::Path>,
            password: &SecretString,
        ) -> revault_lockbox_api::Result<Self> {
            revault_vault_api::VaultDirectory::create_file(path, password).map(Self)
        }
        /// Opens an existing store at an explicit file path.
        pub fn open_file(
            path: impl AsRef<std::path::Path>,
            password: &SecretString,
        ) -> revault_lockbox_api::Result<Self> {
            revault_vault_api::VaultDirectory::open_file(path, password).map(Self)
        }
        /// Opens or creates a store below `root`.
        pub fn open_or_create(
            root: impl AsRef<std::path::Path>,
            password: &SecretString,
        ) -> revault_lockbox_api::Result<Self> {
            revault_vault_api::VaultDirectory::open_or_create(root, password).map(Self)
        }
        /// Replaces the store below `root`.
        pub fn replace(
            root: impl AsRef<std::path::Path>,
            password: &SecretString,
        ) -> revault_lockbox_api::Result<Self> {
            revault_vault_api::VaultDirectory::replace(root, password).map(Self)
        }
        /// Returns the structure version without opening the store for writes.
        pub fn probe_structure_version(
            root: impl AsRef<std::path::Path>,
            password: &SecretString,
        ) -> revault_lockbox_api::Result<u32> {
            revault_vault_api::VaultDirectory::probe_structure_version(root, password)
        }
        /// Returns the store's containing directory.
        pub fn root(&self) -> &std::path::Path {
            self.0.root()
        }
        /// Returns the encrypted store file path.
        pub fn path(&self) -> &std::path::Path {
            self.0.path()
        }
        /// Returns the on-disk structure version.
        pub fn structure_version(&self) -> revault_lockbox_api::Result<u32> {
            self.0.structure_version()
        }
        /// Stores a contact private key under a profile name.
        pub fn store_private_key(
            &self,
            name: &str,
            key: &ContactKeyPair,
        ) -> revault_lockbox_api::Result<()> {
            self.0.store_private_key(name, key)
        }
        /// Loads a contact private key for a profile.
        pub fn load_private_key(&self, name: &str) -> revault_lockbox_api::Result<ContactKeyPair> {
            self.0.load_private_key(name)
        }
        /// Reports whether a profile private key exists.
        pub fn private_key_exists(&self, name: &str) -> revault_lockbox_api::Result<bool> {
            self.0.private_key_exists(name)
        }
        /// Lists profile names with private keys.
        pub fn list_private_keys(&self) -> revault_lockbox_api::Result<Vec<String>> {
            self.0.list_private_keys()
        }
        /// Deletes a profile private key and its metadata.
        pub fn delete_private_key(&self, name: &str) -> revault_lockbox_api::Result<()> {
            self.0.delete_private_key(name)
        }
        /// Stores a profile's non-secret email metadata.
        pub fn store_profile_email(
            &self,
            name: &str,
            email: &str,
        ) -> revault_lockbox_api::Result<()> {
            self.0.store_profile_email(name, email)
        }
        /// Reads a profile's email metadata.
        pub fn profile_email(&self, name: &str) -> revault_lockbox_api::Result<Option<String>> {
            self.0.profile_email(name)
        }
        /// Lists Profile key generations.
        pub fn list_profile_generations(
            &self,
            name: &str,
        ) -> revault_lockbox_api::Result<ProfileHistory> {
            self.0.list_profile_generations(name)
        }
        /// Rotates a profile's private key.
        pub fn rotate_private_key(
            &self,
            name: &str,
        ) -> revault_lockbox_api::Result<ProfileHistory> {
            self.0.rotate_private_key(name)
        }
        /// Loads the current profile signing identity.
        pub fn load_profile_signing_key(
            &self,
            name: &str,
        ) -> revault_lockbox_api::Result<super::ProfileSigningKeyPair> {
            self.0.load_owner_signing_key(name)
        }
        /// Loads a historical profile signing identity.
        pub fn load_profile_signing_key_generation(
            &self,
            name: &str,
            index: u16,
        ) -> revault_lockbox_api::Result<super::ProfileSigningKeyPair> {
            self.0.load_owner_signing_key_generation(name, index)
        }
        /// Stores a contact's public key.
        pub fn store_contact(
            &self,
            name: &str,
            key: &ContactPublicKey,
        ) -> revault_lockbox_api::Result<()> {
            self.0.store_contact(name, key)
        }
        /// Loads a contact's public key.
        pub fn load_contact(&self, name: &str) -> revault_lockbox_api::Result<ContactPublicKey> {
            self.0.load_contact(name)
        }
        /// Reports whether a contact exists.
        pub fn contact_exists(&self, name: &str) -> revault_lockbox_api::Result<bool> {
            self.0.contact_exists(name)
        }
        /// Deletes a contact's public key.
        pub fn delete_contact(&self, name: &str) -> revault_lockbox_api::Result<()> {
            self.0.delete_contact(name)
        }
        /// Lists stored contacts.
        pub fn list_contacts(&self) -> revault_lockbox_api::Result<Vec<StoredContact>> {
            self.0.list_contacts()
        }
        /// Stores a contact signing public key.
        pub fn store_contact_signing_key(
            &self,
            name: &str,
            key: &super::ProfileSigningPublicKey,
        ) -> revault_lockbox_api::Result<()> {
            self.0.store_contact_signing_key(name, key)
        }
        /// Loads a contact signing public key.
        pub fn load_contact_signing_key(
            &self,
            name: &str,
        ) -> revault_lockbox_api::Result<super::ProfileSigningPublicKey> {
            self.0.load_contact_signing_key(name)
        }
    }

    /// Read-only view of a persistent [`Vault`].
    #[derive(Debug)]
    pub struct ReadOnlyVault(revault_vault_api::ReadOnlyVaultDirectory);

    impl ReadOnlyVault {
        /// Opens the default store without loading private signing material.
        pub fn open_default(password: &SecretString) -> revault_lockbox_api::Result<Self> {
            revault_vault_api::ReadOnlyVaultDirectory::open_default(password).map(Self)
        }
        /// Opens a store below `root` without loading private signing material.
        pub fn open(
            root: impl AsRef<std::path::Path>,
            password: &SecretString,
        ) -> revault_lockbox_api::Result<Self> {
            revault_vault_api::ReadOnlyVaultDirectory::open(root, password).map(Self)
        }
        /// Lists profile names without loading private keys.
        pub fn list_private_key_names(&self) -> revault_lockbox_api::Result<Vec<String>> {
            self.0.list_private_key_names()
        }
        /// Lists contact names without loading contact key material.
        pub fn list_contact_names(&self) -> revault_lockbox_api::Result<Vec<String>> {
            self.0.list_contact_names()
        }
        /// Lists form aliases stored in the encrypted metadata.
        pub fn list_form_aliases(&self) -> revault_lockbox_api::Result<Vec<String>> {
            self.0.list_form_aliases()
        }
        /// Lists remembered Lockbox paths.
        pub fn list_known_lockboxes(&self) -> revault_lockbox_api::Result<Vec<KnownLockbox>> {
            self.0.list_known_lockboxes()
        }
    }

    /// Explicit controller for the optional Session Agent. It caches selected
    /// content keys only when asked and never represents persistent Vault data.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct AgentSession;

    impl AgentSession {
        /// Returns the process-local session controller.
        pub const fn instance() -> Self {
            Self
        }
        /// Starts the optional Session Agent process.
        pub fn start(&self) -> std::io::Result<()> {
            revault_vault_api::start()
        }
        /// Stops the optional Session Agent process.
        pub fn stop(&self) -> std::io::Result<()> {
            revault_vault_api::stop()
        }
        /// Forgets every cached lockbox and Profile key.
        pub fn close_all(&self) -> std::io::Result<()> {
            revault_vault_api::forget_all()
        }
        /// Reads a cached profile signing identity, if one is present.
        ///
        /// The private key remains in the Session Agent; callers should drop
        /// the returned value as soon as signing is complete.
        pub fn profile_signing_key(
            &self,
            vault_id: &str,
            profile: &str,
        ) -> std::io::Result<Option<super::ProfileSigningKeyPair>> {
            revault_vault_api::get_owner_signing_key(vault_id, profile)
        }
        /// Caches a profile signing identity for the requested session TTL.
        pub fn cache_profile_signing_key(
            &self,
            vault_id: &str,
            profile: &str,
            key: super::ProfileSigningKeyPair,
            ttl_seconds: Option<u64>,
        ) -> std::io::Result<()> {
            revault_vault_api::put_owner_signing_key(vault_id, profile, key, ttl_seconds)
        }
        /// Removes one cached profile signing identity.
        pub fn forget_profile_signing_key(
            &self,
            vault_id: &str,
            profile: &str,
        ) -> std::io::Result<()> {
            revault_vault_api::forget_owner_signing_key(vault_id, profile)
        }
        /// Reports whether the Session Agent process is running.
        pub fn is_running(&self) -> bool {
            revault_vault_api::is_running()
        }
        /// Serves the Session Agent protocol in the current process.
        pub fn serve(&self) -> std::io::Result<()> {
            revault_vault_api::serve_agent()
        }
    }
}

pub use lockbox::{ProfileSigningKeyPair, ProfileSigningPublicKey, WrappedContactKey};
pub use revault_lockbox_api::{ContactKeyPair, ContactPublicKey, Lockbox};
pub use vault::{AgentSession, ReadOnlyVault, Vault};

/// Source-native runtime entry point.
///
/// Rust links the implementation when the crate is built, therefore loading
/// is a no-op. The explicit entry point keeps startup discoverable without
/// introducing a library-path escape hatch.
#[derive(Debug, Clone, Copy, Default)]
pub struct Revault;

impl Revault {
    /// Returns the linked runtime entry point.
    ///
    /// ```rust
    /// let _runtime = revault_api::Revault::load();
    /// # let _ = _runtime;
    /// ```
    pub const fn load() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentSession, ProfileSigningKeyPair, ReadOnlyVault, Revault, Vault};

    #[test]
    fn reviewed_runtime_and_profile_identity_are_process_local() {
        let _runtime = Revault::load();
        let session = AgentSession::instance();
        let _running = session.is_running();
        let signing_key = ProfileSigningKeyPair::generate().expect("generate Profile key");
        let public_key = signing_key.public_key();
        assert!(!public_key.to_bytes().is_empty());

        let root = std::env::temp_dir().join(format!(
            "revault-api-facade-{}-{}",
            std::process::id(),
            public_key.to_bytes()[0]
        ));
        let passphrase = super::vault::SecretString::try_from_slice(b"facade test passphrase")
            .expect("construct Vault passphrase");
        let vault = Vault::replace(&root, &passphrase).expect("replace test Vault");
        assert!(vault.structure_version().expect("read structure version") > 0);
        drop(vault);
        let readonly = ReadOnlyVault::open(&root, &passphrase).expect("open read-only Vault");
        assert!(readonly
            .list_private_key_names()
            .expect("list profile names")
            .is_empty());
        let _ = std::fs::remove_dir_all(root);
    }
}
