use revault_lockbox_api::vault_integration::VaultOpen;
use revault_lockbox_api::{
    ContactKeyPair, Error, FileLockScope, Lockbox, LockboxOpen, LockboxPath, LockboxProtection,
    OwnerSigningKeyPair, Result, ScopedFileLock, SecretVec, VariableName,
};
use revault_vault_api::{
    export_private_key, import_private_key_file, ContentKeyStore, KeyFormat,
    ProfileGenerationStatus, ReadOnlyVaultDirectory, SecretString, Vault, VaultDirectory,
    CURRENT_VAULT_STRUCTURE_VERSION,
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn create_private_test_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }
}

fn p(path: impl AsRef<str>) -> LockboxPath {
    LockboxPath::new(path).unwrap()
}

fn signing_key() -> OwnerSigningKeyPair {
    OwnerSigningKeyPair::generate().unwrap()
}

#[derive(Default)]
struct MemoryStore {
    keys: RefCell<BTreeMap<String, Vec<u8>>>,
}

impl ContentKeyStore for MemoryStore {
    fn get_content_key(
        &self,
        lockbox_id: revault_lockbox_api::LockboxId,
    ) -> Result<Option<SecretVec>> {
        self.keys
            .borrow()
            .get(&lockbox_id.to_string())
            .map(|key| SecretVec::try_from_slice(key))
            .transpose()
            .map_err(Into::into)
    }

    fn put_content_key(
        &self,
        lockbox_id: revault_lockbox_api::LockboxId,
        key: SecretVec,
    ) -> Result<()> {
        let key = key.with_bytes(|key| key.to_vec())?;
        self.keys.borrow_mut().insert(lockbox_id.to_string(), key);
        Ok(())
    }

    fn forget_content_key(&self, lockbox_id: revault_lockbox_api::LockboxId) -> Result<()> {
        self.keys.borrow_mut().remove(&lockbox_id.to_string());
        Ok(())
    }

    fn forget_all_content_keys(&self) -> Result<()> {
        self.keys.borrow_mut().clear();
        Ok(())
    }
}

#[derive(Default)]
struct SecureCloneStore {
    keys: RefCell<BTreeMap<String, SecretVec>>,
}

impl ContentKeyStore for SecureCloneStore {
    fn get_content_key(
        &self,
        lockbox_id: revault_lockbox_api::LockboxId,
    ) -> Result<Option<SecretVec>> {
        self.keys
            .borrow()
            .get(&lockbox_id.to_string())
            .map(SecretVec::try_clone)
            .transpose()
            .map_err(Into::into)
    }

    fn put_content_key(
        &self,
        lockbox_id: revault_lockbox_api::LockboxId,
        key: SecretVec,
    ) -> Result<()> {
        let key = key.try_clone()?;
        self.keys.borrow_mut().insert(lockbox_id.to_string(), key);
        Ok(())
    }

    fn forget_content_key(&self, lockbox_id: revault_lockbox_api::LockboxId) -> Result<()> {
        self.keys.borrow_mut().remove(&lockbox_id.to_string());
        Ok(())
    }

    fn forget_all_content_keys(&self) -> Result<()> {
        self.keys.borrow_mut().clear();
        Ok(())
    }
}

#[test]
fn vault_create_open_and_lock_with_content_key() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let _platform_guard = EnvVarGuard::set("LOCKBOX_PLATFORM_SECRET_STORE", "disabled");
    let (vault_root, _vault_dir_guard, _vault_password_guard) =
        install_default_vault("raw-default-vault");
    let path = unique_path("raw");
    let vault = Vault::new(MemoryStore::default());
    let key = SecretVec::try_from_slice(b"0123456789abcdef0123456789abcdef").unwrap();

    let mut lockbox = vault
        .create_lockbox(&path, LockboxProtection::ContentKey(key))
        .unwrap();
    lockbox.create_dir(&p("/docs"), false).unwrap();
    lockbox
        .add_file(&p("/docs/a.txt"), b"alpha", false)
        .unwrap();
    lockbox.commit().unwrap();
    drop(lockbox);

    let opened = vault.open_lockbox(&path).unwrap();
    assert_eq!(opened.get_file(&p("/docs/a.txt")).unwrap(), b"alpha");
    drop(opened);

    vault.close_lockbox(&path).unwrap();
    assert!(matches!(
        vault.open_lockbox(&path),
        Err(Error::VaultUnavailable(_))
    ));

    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(vault_root);
}

#[test]
fn vault_password_create_and_open_cache_keys_outside_secure_read_access() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let _platform_guard = EnvVarGuard::set("LOCKBOX_PLATFORM_SECRET_STORE", "disabled");
    let (vault_root, _vault_dir_guard, _vault_password_guard) =
        install_default_vault("password-cache-default-vault");
    let path = unique_path("password-secure-cache");
    let vault = Vault::new(SecureCloneStore::default());
    let password = SecretString::try_from_slice(b"password").unwrap();

    let mut lockbox = vault
        .create_lockbox_with_password(&path, &password)
        .unwrap();
    lockbox.create_dir(&p("/docs"), false).unwrap();
    lockbox
        .add_file(&p("/docs/a.txt"), b"alpha", false)
        .unwrap();
    lockbox.commit().unwrap();
    drop(lockbox);

    let cached = vault.open_lockbox(&path).unwrap();
    assert_eq!(cached.get_file(&p("/docs/a.txt")).unwrap(), b"alpha");
    drop(cached);

    vault.close_lockbox(&path).unwrap();
    assert!(matches!(
        vault.open_lockbox(&path),
        Err(Error::VaultUnavailable(_))
    ));

    let opened = vault.open_lockbox_with_password(&path, &password).unwrap();
    assert_eq!(opened.get_file(&p("/docs/a.txt")).unwrap(), b"alpha");
    drop(opened);

    let recached = vault.open_lockbox(&path).unwrap();
    assert_eq!(recached.get_file(&p("/docs/a.txt")).unwrap(), b"alpha");
    let _ = fs::remove_dir_all(vault_root);
}

#[test]
fn vault_open_populates_cache_for_password_lockbox() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let _platform_guard = EnvVarGuard::set("LOCKBOX_PLATFORM_SECRET_STORE", "disabled");
    let (vault_root, _vault_dir_guard, _vault_password_guard) =
        install_default_vault("password-default-vault");
    let path = unique_path("password");
    let vault = Vault::new(MemoryStore::default());
    let password = SecretString::try_from_bytes(b"shared password".to_vec()).unwrap();

    let mut lockbox = vault
        .create_lockbox(&path, LockboxProtection::Password(&password))
        .unwrap();
    lockbox
        .add_file(&p("/secret.txt"), b"bravo", false)
        .unwrap();
    lockbox.commit().unwrap();
    drop(lockbox);
    vault.close_lockbox(&path).unwrap();

    assert!(matches!(
        vault.open_lockbox(&path),
        Err(Error::VaultUnavailable(_))
    ));

    vault
        .open_lockbox_with(&path, LockboxOpen::Password(&password))
        .unwrap();
    let opened = vault.open_lockbox(&path).unwrap();
    assert_eq!(opened.get_file(&p("/secret.txt")).unwrap(), b"bravo");

    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(vault_root);
}

#[test]
fn vault_mutation_and_close_survive_thread_handoff() {
    let root = unique_dir("thread-handoff");
    let password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let vault = VaultDirectory::open_or_create(&root, &password).unwrap();
    std::thread::spawn(move || {
        let contact = ContactKeyPair::generate().unwrap();
        vault.store_contact("alice", &contact.public_key()).unwrap();
        assert!(vault.contact_exists("alice").unwrap());
        // Drop on the receiving thread, as can happen with a Go caller.
    })
    .join()
    .unwrap();
    let reopened = VaultDirectory::open_or_create(&root, &password).unwrap();
    assert!(reopened.contact_exists("alice").unwrap());
}

#[test]
fn vault_directory_stores_local_keys_contacts_and_key_directory_backups() {
    let root = unique_dir("directory");
    let vault_password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();
    assert_eq!(
        vault.structure_version().unwrap(),
        CURRENT_VAULT_STRUCTURE_VERSION
    );
    drop(vault);
    assert_eq!(
        VaultDirectory::probe_structure_version(&root, &vault_password).unwrap(),
        CURRENT_VAULT_STRUCTURE_VERSION
    );
    let vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();
    let keypair = ContactKeyPair::generate().unwrap();
    vault.store_private_key("default", &keypair).unwrap();
    // Inspect persisted ciphertext only after releasing the writable file lock.
    drop(vault);
    let encrypted = fs::read(root.join("local-vault.lbox")).unwrap();
    assert!(!String::from_utf8_lossy(&encrypted).contains("default.key"));
    assert!(!String::from_utf8_lossy(&encrypted).contains("private-key-password"));
    let vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();
    let loaded = vault.load_private_key("default").unwrap();
    assert_eq!(
        loaded.private_key_record().unwrap(),
        keypair.private_key_record().unwrap()
    );

    vault.store_contact("alice", &keypair.public_key()).unwrap();
    let contacts = vault.list_contacts().unwrap();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].name, "alice");

    assert_eq!(vault.seed_default_form_definitions().unwrap(), 7);
    assert_eq!(vault.seed_default_form_definitions().unwrap(), 0);
    let form_aliases = vault
        .list_form_definitions()
        .unwrap()
        .into_iter()
        .map(|definition| definition.alias)
        .collect::<Vec<_>>();
    for alias in [
        "bank-account",
        "profile",
        "login",
        "payment-card",
        "secure-note",
        "server",
        "wifi",
    ] {
        assert!(form_aliases.contains(&alias.to_string()), "{alias}");
    }
    drop(vault);
    let reopened_vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();
    let reopened_aliases = reopened_vault
        .list_form_definitions()
        .unwrap()
        .into_iter()
        .map(|definition| definition.alias)
        .collect::<Vec<_>>();
    assert!(reopened_aliases.contains(&"login".to_string()));
    drop(reopened_vault);
    let vault_lockbox = Lockbox::open(
        &root.join("local-vault.lbox"),
        LockboxOpen::Password(&vault_password),
    )
    .unwrap();
    let lockbox_aliases = vault_lockbox
        .list_form_definitions()
        .unwrap()
        .into_iter()
        .map(|definition| definition.alias)
        .collect::<Vec<_>>();
    assert!(lockbox_aliases.contains(&"login".to_string()));
    drop(vault_lockbox);
    let vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();

    let password = SecretString::try_from_bytes(b"pw".to_vec()).unwrap();
    let lockbox_path = root.join("backup-source.lbox");
    let mut lockbox = Lockbox::create_file(
        &lockbox_path,
        LockboxProtection::Password(&password),
        &signing_key(),
    )
    .unwrap();
    lockbox.add_file(&p("/a.txt"), b"alpha", false).unwrap();
    lockbox.commit().unwrap();
    let backup = VaultOpen::export_key_directory_backup(&lockbox).unwrap();
    vault
        .store_key_directory_backup(lockbox.lockbox_id(), &backup)
        .unwrap();
    vault
        .store_key_directory_backup(lockbox.lockbox_id(), &backup)
        .unwrap();
    assert_eq!(
        vault
            .load_key_directory_backup(lockbox.lockbox_id())
            .unwrap(),
        backup
    );
    assert_eq!(vault.key_directory_backup_count().unwrap(), 1);

    vault
        .remember_known_lockbox(lockbox.lockbox_id(), &lockbox_path)
        .unwrap();
    let known = vault.list_known_lockboxes().unwrap();
    assert_eq!(known.len(), 1);
    assert_eq!(known[0].lockbox_id, lockbox.lockbox_id());
    assert_eq!(
        known[0].path,
        lockbox_path.canonicalize().unwrap().to_string_lossy()
    );

    let relocated_path = root.join("relocated-backup-source.lbox");
    let lockbox_id = lockbox.lockbox_id();
    // Windows cannot copy a file while its writable archive handle locks it.
    drop(lockbox);
    fs::copy(&lockbox_path, &relocated_path).unwrap();
    vault
        .remember_known_lockbox(lockbox_id, &relocated_path)
        .unwrap();
    let known = vault.list_known_lockboxes().unwrap();
    assert_eq!(known.len(), 1);
    assert_eq!(
        known[0].path,
        relocated_path.canonicalize().unwrap().to_string_lossy()
    );
    vault.forget_known_lockbox(&relocated_path).unwrap();
    assert!(vault.list_known_lockboxes().unwrap().is_empty());

    drop(vault);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn vault_directory_rejects_unversioned_existing_vaults() {
    let root = unique_dir("legacy-vault-version");
    create_private_test_dir(&root);
    let vault_path = root.join("local-vault.lbox");
    let vault_password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();

    Lockbox::create_file(
        &vault_path,
        LockboxProtection::Password(&vault_password),
        &signing_key(),
    )
    .unwrap();

    assert!(matches!(
        VaultDirectory::open_or_create(&root, &vault_password),
        Err(Error::Configuration(message))
            if message.contains("structure version is missing")
    ));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn vault_directory_rejects_older_structure_versions() {
    let root = unique_dir("explicit-legacy-vault-version");
    let vault_password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();
    let signing_key = vault
        .load_owner_signing_key(VaultDirectory::DEFAULT_KEY_NAME)
        .unwrap();
    drop(vault);

    let vault_path = root.join("local-vault.lbox");
    let mut lockbox = Lockbox::open_for_write(
        &vault_path,
        LockboxOpen::Password(&vault_password),
        &signing_key,
    )
    .unwrap();
    lockbox
        .add_file(&p("/vault/structure-version"), b"0\n", true)
        .unwrap();
    lockbox.commit().unwrap();
    drop(lockbox);

    assert!(matches!(
        VaultDirectory::open_or_create(&root, &vault_password),
        Err(Error::UnsupportedFormatVersion {
            artifact: revault_lockbox_api::ArtifactKind::Vault,
            found: 0,
            supported: CURRENT_VAULT_STRUCTURE_VERSION,
        })
    ));
    let new_password = SecretString::try_from_bytes(b"new-vault-password".to_vec()).unwrap();
    assert!(matches!(
        VaultDirectory::change_password(&root, &vault_password, &new_password),
        Err(Error::UnsupportedFormatVersion {
            artifact: revault_lockbox_api::ArtifactKind::Vault,
            found: 0,
            supported: CURRENT_VAULT_STRUCTURE_VERSION,
        })
    ));
    assert!(Lockbox::open(&vault_path, LockboxOpen::Password(&vault_password)).is_ok());
    assert!(Lockbox::open(&vault_path, LockboxOpen::Password(&new_password)).is_err());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn vault_directory_rejects_newer_structure_versions() {
    let root = unique_dir("future-vault-version");
    let vault_password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();
    let signing_key = vault
        .load_owner_signing_key(VaultDirectory::DEFAULT_KEY_NAME)
        .unwrap();
    drop(vault);

    let vault_path = root.join("local-vault.lbox");
    let mut lockbox = Lockbox::open_for_write(
        &vault_path,
        LockboxOpen::Password(&vault_password),
        &signing_key,
    )
    .unwrap();
    lockbox
        .add_file(&p("/vault/structure-version"), b"999\n", true)
        .unwrap();
    lockbox.commit().unwrap();
    drop(lockbox);

    assert!(matches!(
        VaultDirectory::open_or_create(&root, &vault_password),
        Err(Error::UnsupportedFormatVersion {
            artifact: revault_lockbox_api::ArtifactKind::Vault,
            found: 999,
            supported: CURRENT_VAULT_STRUCTURE_VERSION,
        })
    ));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn vault_open_uses_key_directory_backup_when_embedded_directory_is_corrupt() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    for signed in [false, true] {
        let path = unique_path("backup-fallback");
        let vault_root = unique_dir("backup-fallback-vault");
        let owner = signing_key();
        let password = SecretString::try_from_bytes(b"shared password".to_vec()).unwrap();
        let mut archive = Lockbox::create_file_with_options(
            &path,
            revault_lockbox_api::LockboxCreateOptions::new(
                revault_lockbox_api::Encryption::Encrypted(LockboxProtection::Password(&password)),
                if signed {
                    revault_lockbox_api::Signing::Owner(&owner)
                } else {
                    revault_lockbox_api::Signing::None
                },
            ),
        )
        .unwrap();
        archive
            .add_file(&p("/secret.txt"), b"bravo", false)
            .unwrap();
        archive.commit().unwrap();
        let backup = VaultOpen::export_key_directory_backup(&archive).unwrap();
        let vault_password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
        VaultDirectory::open_or_create(&vault_root, &vault_password)
            .unwrap()
            .store_key_directory_backup(archive.lockbox_id(), &backup)
            .unwrap();
        drop(archive);
        // No public operation deliberately damages embedded copies. This
        // fixture-only mutation exercises the documented backup recovery path.
        corrupt_key_directories(&path);
        let damaged = fs::read(&path).unwrap();
        let _directory = EnvVarGuard::set("LOCKBOX_VAULT_DIR", &vault_root);
        let _password = EnvVarGuard::set("LOCKBOX_VAULT_PASSWORD", "vault-password");
        let vault = Vault::new(MemoryStore::default());
        let opened = if signed {
            vault.open_lockbox_with_signing_key(&path, LockboxOpen::Password(&password), &owner)
        } else {
            vault.open_lockbox_with(&path, LockboxOpen::Password(&password))
        }
        .unwrap();
        assert_eq!(opened.get_file(&p("/secret.txt")).unwrap(), b"bravo");
        // Windows enforces the writable archive's file lock against a second
        // reader. Drop without committing before observing persisted bytes.
        drop(opened);
        assert_eq!(
            fs::read(&path).unwrap(),
            damaged,
            "opening must not publish repairs"
        );
        let mut opened = if signed {
            vault.open_lockbox_with_signing_key(&path, LockboxOpen::Password(&password), &owner)
        } else {
            vault.open_lockbox_with(&path, LockboxOpen::Password(&password))
        }
        .unwrap();
        opened.commit().unwrap();
        drop(opened);
        // The repaired archive must reopen without access to the Vault backup.
        let reopened = Lockbox::open(&path, LockboxOpen::Password(&password)).unwrap();
        assert_eq!(reopened.get_file(&p("/secret.txt")).unwrap(), b"bravo");
        drop(reopened);
        vault.close_all().unwrap();
        fs::remove_file(&path).unwrap();
        fs::remove_dir_all(&vault_root).unwrap();
    }
}

#[test]
fn backup_directory_must_match_the_latest_authenticated_access_state() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let vault_root = unique_dir("backup-admission-vault");
    let path = unique_path("backup-commitment");
    let other_path = unique_path("backup-other-id");
    let owner = signing_key();
    let password = SecretString::try_from_bytes(b"backup commitment password".to_vec()).unwrap();
    let next_password =
        SecretString::try_from_bytes(b"additional legitimate password".to_vec()).unwrap();
    let contact = ContactKeyPair::generate().unwrap();
    let mut archive =
        Lockbox::create_file(&path, LockboxProtection::Password(&password), &owner).unwrap();
    archive
        .add_file(&p("/data"), b"authenticated backup data", false)
        .unwrap();
    archive.commit().unwrap();
    let stale = VaultOpen::export_key_directory_backup(&archive).unwrap();
    archive.add_password(&next_password).unwrap();
    archive.commit().unwrap();
    archive.add_contact(&contact.public_key()).unwrap();
    archive.commit().unwrap();
    let current = VaultOpen::export_key_directory_backup(&archive).unwrap();
    let same_key = VaultOpen::key_directory_backup_with_password(&current, &password)
        .unwrap()
        .try_clone_key()
        .unwrap();
    let mut other =
        Lockbox::create_file(&other_path, LockboxProtection::ContentKey(same_key), &owner).unwrap();
    other.add_password(&password).unwrap();
    other.commit().unwrap();
    let wrong_identity = VaultOpen::export_key_directory_backup(&other).unwrap();
    drop(other);
    drop(archive);
    // The test-only corruption helper is needed because the public API never
    // deliberately removes both committed access-directory copies.
    corrupt_key_directories(&path);
    let damaged = fs::read(&path).unwrap();
    for rejected in [&stale, &wrong_identity] {
        let opened = VaultOpen::key_directory_backup_with_password(rejected, &password).unwrap();
        assert!(matches!(opened.open_path(&path), Err(Error::CorruptRecord)));
        assert_eq!(fs::read(&path).unwrap(), damaged);
    }
    let opened = VaultOpen::key_directory_backup_with_password(&current, &password)
        .unwrap()
        .open_path(&path)
        .unwrap();
    assert_eq!(
        opened.get_file(&p("/data")).unwrap(),
        b"authenticated backup data"
    );
    drop(opened);
    assert_eq!(
        fs::read(&path).unwrap(),
        damaged,
        "read-only recovery never repairs storage"
    );
    let wrong_owner = signing_key();
    let contact_open = VaultOpen::key_directory_backup_with_contact(&current, &contact).unwrap();
    assert!(matches!(
        contact_open.open_path_for_write(&path, &wrong_owner),
        Err(Error::InvalidOperation(_))
    ));
    assert_eq!(
        fs::read(&path).unwrap(),
        damaged,
        "refused recipient writes never repair storage"
    );
    let vault_password = SecretString::try_from_bytes(b"backup admission vault".to_vec()).unwrap();
    let directory = VaultDirectory::open_or_create(&vault_root, &vault_password).unwrap();
    let id = VaultOpen::read_lockbox_id(&path).unwrap();
    directory.store_key_directory_backup(id, &stale).unwrap();
    drop(directory); // The following public open acquires its own Vault lock.
    let _directory = EnvVarGuard::set("LOCKBOX_VAULT_DIR", &vault_root);
    let _password = EnvVarGuard::set("LOCKBOX_VAULT_PASSWORD", "backup admission vault");
    let vault = Vault::new(MemoryStore::default());
    let Err(refusal) =
        vault.open_lockbox_with_signing_key(&path, LockboxOpen::Password(&password), &owner)
    else {
        panic!("a stale access-directory backup must be refused");
    };
    assert!(
        matches!(refusal, Error::CorruptRecord),
        "unexpected refusal kind: {:?}",
        std::mem::discriminant(&refusal)
    );
    assert!(
        vault.store().keys.borrow().is_empty(),
        "failed backup must not publish a cached key"
    );
    VaultDirectory::open_or_create(&vault_root, &vault_password)
        .unwrap()
        .store_key_directory_backup(id, &current)
        .unwrap();
    assert!(matches!(
        vault.open_lockbox_with_signing_key(&path, LockboxOpen::Password(&password), &wrong_owner),
        Err(Error::InvalidKeyMaterial(_))
    ));
    assert!(
        vault.store().keys.borrow().is_empty(),
        "wrong password-open owner must not publish a cached key"
    );
    assert_eq!(fs::read(&path).unwrap(), damaged);
    assert!(matches!(
        vault.open_lockbox_with_signing_key(
            &path,
            LockboxOpen::ContactKeyPair(
                ContactKeyPair::from_private_key_record(contact.private_key_record().unwrap())
                    .unwrap()
            ),
            &wrong_owner
        ),
        Err(Error::InvalidOperation(_))
    ));
    assert!(
        vault.store().keys.borrow().is_empty(),
        "refused owner must not publish a cached key"
    );
    assert_eq!(fs::read(&path).unwrap(), damaged);
    let mut repaired = vault
        .open_lockbox_with_signing_key(&path, LockboxOpen::ContactKeyPair(contact), &owner)
        .unwrap();
    assert!(!vault.store().keys.borrow().is_empty());
    repaired.commit().unwrap();
    drop(repaired);
    let reopened = Lockbox::open(&path, LockboxOpen::Password(&next_password)).unwrap();
    assert_eq!(
        reopened.get_file(&p("/data")).unwrap(),
        b"authenticated backup data"
    );
    drop(reopened);
    fs::remove_file(path).unwrap();
    fs::remove_file(other_path).unwrap();
    vault.close_all().unwrap();
    fs::remove_dir_all(vault_root).unwrap();
}

#[test]
fn unsigned_contact_backup_preserves_recipient_write_access() {
    let path = unique_path("unsigned-contact-backup");
    let contact = ContactKeyPair::generate().unwrap();
    let mut archive = Lockbox::create_file_with_options(
        &path,
        revault_lockbox_api::LockboxCreateOptions::new(
            revault_lockbox_api::Encryption::Encrypted(LockboxProtection::ContactPublicKey {
                name: None,
                contact: contact.public_key(),
            }),
            revault_lockbox_api::Signing::None,
        ),
    )
    .unwrap();
    archive
        .add_file(&p("/data"), b"unsigned recipient content", false)
        .unwrap();
    archive.commit().unwrap();
    let backup = VaultOpen::export_key_directory_backup(&archive).unwrap();
    drop(archive);
    // No public API deliberately damages committed directory copies.
    corrupt_key_directories(&path);
    let damaged = fs::read(&path).unwrap();
    let restored = VaultOpen::key_directory_backup_with_contact(&backup, &contact)
        .unwrap()
        .open_path_for_write(&path, revault_lockbox_api::Signing::None)
        .unwrap();
    assert_eq!(
        restored.get_file(&p("/data")).unwrap(),
        b"unsigned recipient content"
    );
    // Observe storage after releasing the mandatory Windows writer lock; this
    // first recovered handle is deliberately dropped without a commit.
    drop(restored);
    assert_eq!(fs::read(&path).unwrap(), damaged);
    let mut restored = VaultOpen::key_directory_backup_with_contact(&backup, &contact)
        .unwrap()
        .open_path_for_write(&path, revault_lockbox_api::Signing::None)
        .unwrap();
    restored
        .add_file(&p("/added"), b"recipient addition", false)
        .unwrap();
    restored.commit().unwrap();
    drop(restored);
    let reopened = Lockbox::open(&path, LockboxOpen::ContactKeyPair(contact)).unwrap();
    assert_eq!(
        reopened.get_file(&p("/data")).unwrap(),
        b"unsigned recipient content"
    );
    assert_eq!(
        reopened.get_file(&p("/added")).unwrap(),
        b"recipient addition"
    );
    drop(reopened);
    fs::remove_file(path).unwrap();
}

#[test]
fn vault_convenience_password_store_and_close_all_flow() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let _platform_guard = EnvVarGuard::set("LOCKBOX_PLATFORM_SECRET_STORE", "disabled");
    let (vault_root, _vault_dir_guard, _vault_password_guard) =
        install_default_vault("password-convenience-default-vault");
    let path = unique_path("password-convenience");
    let store = MemoryStore::default();
    let vault = Vault::new(store);
    assert!(vault.store().keys.borrow().is_empty());

    let password = SecretString::try_from_bytes(b"shared password".to_vec()).unwrap();
    let mut lockbox = vault
        .create_lockbox_with_password(&path, &password)
        .unwrap();
    lockbox
        .add_file(&p("/secret.txt"), b"charlie", false)
        .unwrap();
    lockbox.commit().unwrap();
    drop(lockbox);
    vault.close_lockbox(&path).unwrap();
    assert!(matches!(
        vault.open_lockbox(&path),
        Err(Error::VaultUnavailable(_))
    ));

    let opened = vault.open_lockbox_with_password(&path, &password).unwrap();
    assert_eq!(opened.get_file(&p("/secret.txt")).unwrap(), b"charlie");
    drop(opened);
    assert_eq!(
        vault
            .open_lockbox(&path)
            .unwrap()
            .get_file(&p("/secret.txt"))
            .unwrap(),
        b"charlie"
    );

    vault.close_all().unwrap();
    assert!(matches!(
        vault.open_lockbox(&path),
        Err(Error::VaultUnavailable(_))
    ));

    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(vault_root);
}

#[test]
fn vault_directory_public_crud_helpers_flow() {
    let root = unique_dir("directory-crud");
    let vault_password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();
    assert_eq!(vault.root(), root.as_path());
    assert_eq!(vault.path(), root.join("local-vault.lbox").as_path());

    let keypair = ContactKeyPair::generate().unwrap();
    assert!(!vault.private_key_exists("default").unwrap());
    vault.store_private_key("default", &keypair).unwrap();
    assert!(vault.private_key_exists("default").unwrap());
    assert_eq!(
        vault.list_private_keys().unwrap(),
        vec!["default".to_string()]
    );
    assert_eq!(
        vault
            .load_private_key("default")
            .unwrap()
            .private_key_record()
            .unwrap(),
        keypair.private_key_record().unwrap()
    );
    vault.delete_private_key("default").unwrap();
    assert!(!vault.private_key_exists("default").unwrap());

    let contact = keypair.public_key();
    let signing = OwnerSigningKeyPair::generate().unwrap().public_key();
    assert!(!vault.contact_exists("alice").unwrap());
    vault.store_contact("alice", &contact).unwrap();
    vault.store_contact_signing_key("alice", &signing).unwrap();
    assert!(vault.contact_exists("alice").unwrap());
    assert_eq!(vault.load_contact("alice").unwrap(), contact);
    assert_eq!(vault.load_contact_signing_key("alice").unwrap(), signing);
    let contacts = vault.list_contacts().unwrap();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].name, "alice");
    assert_eq!(contacts[0].key, contact);
    let replacement = ContactKeyPair::generate().unwrap().public_key();
    vault.replace_contact("alice", &replacement).unwrap();
    assert_eq!(vault.load_contact("alice").unwrap(), replacement);
    assert!(vault.load_contact_signing_key("alice").is_err());
    vault.delete_contact("alice").unwrap();
    assert!(!vault.contact_exists("alice").unwrap());
    assert!(vault.load_contact_signing_key("alice").is_err());

    assert_eq!(vault.key_directory_backup_count().unwrap(), 0);
    let password = SecretString::try_from_bytes(b"pw".to_vec()).unwrap();
    let lockbox_path = root.join("backup-count-source.lbox");
    let mut lockbox = Lockbox::create_file(
        &lockbox_path,
        LockboxProtection::Password(&password),
        &signing_key(),
    )
    .unwrap();
    lockbox.commit().unwrap();
    vault
        .store_key_directory_backup(
            lockbox.lockbox_id(),
            &VaultOpen::export_key_directory_backup(&lockbox).unwrap(),
        )
        .unwrap();
    vault
        .store_key_directory_backup(
            lockbox.lockbox_id(),
            &VaultOpen::export_key_directory_backup(&lockbox).unwrap(),
        )
        .unwrap();
    assert_eq!(vault.key_directory_backup_count().unwrap(), 1);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn deleting_default_profile_does_not_strand_vault_container() {
    let root = unique_dir("delete-default-profile-reopen");
    let password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let vault = VaultDirectory::open_or_create(&root, &password).unwrap();
    vault
        .store_private_key("default", &ContactKeyPair::generate().unwrap())
        .unwrap();
    vault.delete_private_key("default").unwrap();
    drop(vault);

    let reopened = VaultDirectory::open_or_create(&root, &password).unwrap();
    assert!(reopened.list_private_keys().unwrap().is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn replacing_default_profile_signer_does_not_replace_vault_container_signer() {
    let root = unique_dir("replace-default-profile-signer");
    let password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let vault = VaultDirectory::open_or_create(&root, &password).unwrap();
    vault
        .store_private_key("default", &ContactKeyPair::generate().unwrap())
        .unwrap();

    let restored_key = ContactKeyPair::generate().unwrap();
    let restored_signer = OwnerSigningKeyPair::generate().unwrap();
    let expected_public = restored_signer.public_key();
    vault
        .restore_private_key("default", &restored_key, Some(&restored_signer), true)
        .unwrap();
    drop(vault);

    let reopened = VaultDirectory::open_or_create(&root, &password).unwrap();
    assert_eq!(
        reopened
            .load_owner_signing_key("default")
            .unwrap()
            .public_key(),
        expected_public
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn existing_current_format_vault_migrates_container_signer_record() {
    let root = unique_dir("migrate-container-signer");
    let password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let vault = VaultDirectory::open_or_create(&root, &password).unwrap();
    let profile_signer = vault.load_owner_signing_key("default").unwrap();
    drop(vault);

    let path = root.join("local-vault.lbox");
    let mut raw =
        Lockbox::open_for_write(&path, LockboxOpen::Password(&password), &profile_signer).unwrap();
    raw.delete_variable(&VariableName::new("LOCKBOX_VAULT_CONTAINER_SIGNING_KEY").unwrap())
        .unwrap();
    raw.commit().unwrap();
    drop(raw);

    VaultDirectory::open_or_create(&root, &password).unwrap();
    let reopened = Lockbox::open(&path, LockboxOpen::Password(&password)).unwrap();
    assert!(reopened
        .variable_sensitivity(&VariableName::new("LOCKBOX_VAULT_CONTAINER_SIGNING_KEY").unwrap())
        .unwrap()
        .is_some());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn vault_directory_tracks_profile_generations_and_rotation() {
    let root = unique_dir("profile-generations");
    let vault_password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();

    let original = ContactKeyPair::generate().unwrap();
    vault.store_private_key("default", &original).unwrap();
    let original_signing_public = vault
        .load_owner_signing_key_generation("default", 1)
        .unwrap()
        .public_key()
        .to_bytes();

    let history = vault.list_profile_generations("default").unwrap();
    assert_eq!(history.active_generation, 1);
    assert_eq!(history.generations.len(), 1);
    assert_eq!(history.generations[0].index, 1);
    assert_eq!(
        history.generations[0].status,
        ProfileGenerationStatus::Active
    );
    assert_eq!(
        vault
            .load_private_key_generation("default", 1)
            .unwrap()
            .private_key_record()
            .unwrap(),
        original.private_key_record().unwrap()
    );
    assert_eq!(
        vault.list_private_keys().unwrap(),
        vec!["default".to_string()]
    );

    let rotated = vault.rotate_private_key("default").unwrap();
    assert_eq!(rotated.active_generation, 2);
    assert_eq!(rotated.generations.len(), 2);
    assert_eq!(
        rotated.generations[0].status,
        ProfileGenerationStatus::Retired
    );
    assert!(rotated.generations[0].retired_at_unix_ms.is_some());
    assert_eq!(
        rotated.generations[1].status,
        ProfileGenerationStatus::Active
    );
    assert_eq!(
        vault
            .load_private_key_generation("default", 1)
            .unwrap()
            .private_key_record()
            .unwrap(),
        original.private_key_record().unwrap()
    );
    let active = vault.load_private_key("default").unwrap();
    let active_signing_public = vault
        .load_owner_signing_key("default")
        .unwrap()
        .public_key()
        .to_bytes();
    assert_eq!(
        vault
            .load_private_key_generation("default", 2)
            .unwrap()
            .private_key_record()
            .unwrap(),
        active.private_key_record().unwrap()
    );
    assert_ne!(
        active.private_key_record().unwrap(),
        original.private_key_record().unwrap()
    );
    assert_eq!(
        vault
            .load_owner_signing_key_generation("default", 2)
            .unwrap()
            .public_key()
            .to_bytes(),
        active_signing_public
    );
    assert_ne!(active_signing_public, original_signing_public);
    assert_eq!(
        vault.list_private_keys().unwrap(),
        vec!["default".to_string()]
    );

    drop(vault);
    let reopened = VaultDirectory::open_or_create(&root, &vault_password).unwrap();
    reopened
        .store_contact(
            "after-rotation",
            &ContactKeyPair::generate().unwrap().public_key(),
        )
        .unwrap();
    assert!(reopened.load_contact("after-rotation").is_ok());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn vault_reopen_uses_container_signer_when_profile_history_is_missing() {
    let root = unique_dir("missing-historical-signer");
    let vault_password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();
    vault
        .store_private_key("default", &ContactKeyPair::generate().unwrap())
        .unwrap();
    let established_signer = vault
        .load_owner_signing_key_generation("default", 1)
        .unwrap();
    vault.rotate_private_key("default").unwrap();
    drop(vault);

    let vault_path = root.join("local-vault.lbox");
    let mut raw_vault = Lockbox::open_for_write(
        &vault_path,
        LockboxOpen::Password(&vault_password),
        &established_signer,
    )
    .unwrap();
    raw_vault
        .delete_variable(
            &VariableName::new("LOCKBOX_VAULT_SIGNING_KEY_64656661756C74_GEN_0001").unwrap(),
        )
        .unwrap();
    raw_vault.commit().unwrap();
    drop(raw_vault);

    VaultDirectory::open_or_create(&root, &vault_password).unwrap();

    let _ = fs::remove_dir_all(root);
}

#[test]
fn vault_reopen_fails_closed_when_container_and_historical_signers_are_missing() {
    let root = unique_dir("missing-container-and-historical-signer");
    let vault_password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();
    vault
        .store_private_key("default", &ContactKeyPair::generate().unwrap())
        .unwrap();
    let established_signer = vault
        .load_owner_signing_key_generation("default", 1)
        .unwrap();
    vault.rotate_private_key("default").unwrap();
    drop(vault);

    let vault_path = root.join("local-vault.lbox");
    let mut raw_vault = Lockbox::open_for_write(
        &vault_path,
        LockboxOpen::Password(&vault_password),
        &established_signer,
    )
    .unwrap();
    for name in [
        "LOCKBOX_VAULT_CONTAINER_SIGNING_KEY",
        "LOCKBOX_VAULT_SIGNING_KEY_64656661756C74_GEN_0001",
    ] {
        raw_vault
            .delete_variable(&VariableName::new(name).unwrap())
            .unwrap();
    }
    raw_vault.commit().unwrap();
    drop(raw_vault);

    assert!(VaultDirectory::open_or_create(&root, &vault_password).is_err());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn vault_open_times_out_when_locked_by_thread() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let _timeout_guard = EnvVarGuard::set("LOCKBOX_LOCK_TIMEOUT_MS", "150");
    let root = unique_dir("vault-thread-lock");
    let vault_password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();
    drop(vault);
    let vault_path = root.join("local-vault.lbox");
    let (locked_tx, locked_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let locked_path = vault_path.clone();
    let handle = std::thread::spawn(move || {
        let _lock = ScopedFileLock::acquire(&locked_path, FileLockScope::Vault).unwrap();
        locked_tx.send(()).unwrap();
        release_rx.recv().unwrap();
    });
    locked_rx.recv().unwrap();

    let contact = ContactKeyPair::generate().unwrap().public_key();
    let err = VaultDirectory::open_or_create(&root, &vault_password).unwrap_err();
    assert!(matches!(err, Error::LockUnavailable(_)));

    release_tx.send(()).unwrap();
    handle.join().unwrap();
    let vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();
    vault.store_contact("allowed", &contact).unwrap();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn vault_open_times_out_when_locked_by_process() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let _timeout_guard = EnvVarGuard::set("LOCKBOX_LOCK_TIMEOUT_MS", "150");
    let root = unique_dir("vault-process-lock");
    let vault_password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();
    drop(vault);
    let ready_path = root.join("process-lock-ready");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("vault_lock_process_helper")
        .env("LOCKBOX_VAULT_LOCK_HELPER_ROOT", &root)
        .env("LOCKBOX_VAULT_LOCK_HELPER_READY", &ready_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    wait_for_path(&ready_path);

    let err = VaultDirectory::open_or_create(&root, &vault_password).unwrap_err();
    assert!(matches!(err, Error::LockUnavailable(_)));

    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn vault_lock_process_helper() {
    let Some(root) = std::env::var_os("LOCKBOX_VAULT_LOCK_HELPER_ROOT") else {
        return;
    };
    let ready = std::env::var_os("LOCKBOX_VAULT_LOCK_HELPER_READY")
        .map(PathBuf::from)
        .unwrap();
    let vault_path = PathBuf::from(root).join("local-vault.lbox");
    let _lock = ScopedFileLock::acquire(&vault_path, FileLockScope::Vault).unwrap();
    fs::write(ready, b"ready").unwrap();
    std::thread::sleep(std::time::Duration::from_secs(30));
}

#[test]
fn read_only_vault_credential_readers_share_locks_across_processes() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let root = unique_dir("vault-read-only-process-lock");
    create_private_test_dir(&root);
    let path = root.join("credentials.lbox");
    let password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let profile_password = SecretString::try_from_bytes(b"profile-password".to_vec()).unwrap();
    let vault = VaultDirectory::create_file(&path, &password).unwrap();
    vault
        .store_password_profile("service", &profile_password, false)
        .unwrap();
    drop(vault);

    let spawn_reader = |number: usize| {
        let ready = root.join(format!("reader-{number}-ready"));
        let release = root.join(format!("reader-{number}-release"));
        let child = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("read_only_vault_credential_process_helper")
            .env("LOCKBOX_VAULT_READ_ONLY_HELPER_PATH", &path)
            .env("LOCKBOX_VAULT_READ_ONLY_HELPER_READY", &ready)
            .env("LOCKBOX_VAULT_READ_ONLY_HELPER_RELEASE", &release)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        wait_for_path(&ready);
        (child, release)
    };
    let (mut first, first_release) = spawn_reader(1);
    let (mut second, second_release) = spawn_reader(2);

    let check_writer = |blocked: bool| {
        let output = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("read_only_vault_writer_process_helper")
            .env("LOCKBOX_VAULT_WRITER_HELPER_PATH", &path)
            .env("LOCKBOX_LOCK_TIMEOUT_MS", "150")
            .env(
                "LOCKBOX_VAULT_WRITER_EXPECT_BLOCKED",
                if blocked { "yes" } else { "no" },
            )
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    };

    check_writer(true);
    fs::write(&first_release, b"release").unwrap();
    assert!(first.wait().unwrap().success());
    check_writer(true);
    fs::write(&second_release, b"release").unwrap();
    assert!(second.wait().unwrap().success());
    check_writer(false);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn read_only_vault_credential_process_helper() {
    let Some(path) = std::env::var_os("LOCKBOX_VAULT_READ_ONLY_HELPER_PATH") else {
        return;
    };
    let ready = PathBuf::from(std::env::var_os("LOCKBOX_VAULT_READ_ONLY_HELPER_READY").unwrap());
    let release =
        PathBuf::from(std::env::var_os("LOCKBOX_VAULT_READ_ONLY_HELPER_RELEASE").unwrap());
    let password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let view = ReadOnlyVaultDirectory::open_file(Path::new(&path), &password).unwrap();
    assert_eq!(
        view.load_profile_password("service")
            .unwrap()
            .with_bytes(|bytes| bytes.to_vec())
            .unwrap(),
        b"profile-password"
    );
    fs::write(ready, b"ready").unwrap();
    wait_for_path(&release);
}

#[test]
fn read_only_vault_writer_process_helper() {
    let Some(path) = std::env::var_os("LOCKBOX_VAULT_WRITER_HELPER_PATH") else {
        return;
    };
    let password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let result = VaultDirectory::open_file(Path::new(&path), &password);
    if std::env::var("LOCKBOX_VAULT_WRITER_EXPECT_BLOCKED").unwrap() == "yes" {
        assert!(
            matches!(result, Err(Error::LockUnavailable(_))),
            "{result:?}"
        );
    } else {
        result.unwrap();
    }
}

#[test]
fn vault_directory_stores_profile_email_metadata() {
    let root = unique_dir("profile-email");
    let vault_password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let vault = VaultDirectory::open_or_create(&root, &vault_password).unwrap();

    vault
        .store_private_key("default", &ContactKeyPair::generate().unwrap())
        .unwrap();
    assert_eq!(vault.profile_email("default").unwrap(), None);

    vault
        .store_profile_email("default", "alice@example.test")
        .unwrap();
    assert_eq!(
        vault.profile_email("default").unwrap(),
        Some("alice@example.test".to_string())
    );

    vault.delete_private_key("default").unwrap();
    assert_eq!(vault.profile_email("default").unwrap(), None);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn private_key_file_import_uses_secure_import_path() {
    let root = unique_dir("private-key-file-import");
    create_private_test_dir(&root);
    let path = root.join("private.key");
    let keypair = ContactKeyPair::generate().unwrap();
    let private = export_private_key(&keypair, KeyFormat::RawHex).unwrap();
    private
        .with_bytes(|bytes| fs::write(&path, bytes))
        .unwrap()
        .unwrap();

    let loaded = import_private_key_file(&path).unwrap();
    assert_eq!(
        loaded.private_key_record().unwrap(),
        keypair.private_key_record().unwrap()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn standalone_vault_file_can_be_created_and_reopened() {
    let root = unique_dir("standalone-vault-file");
    let _ = fs::remove_dir_all(&root);
    create_private_test_dir(&root);
    let path = root.join("new-parent/nested/production.vault.lbx");
    let password = SecretString::try_from_bytes(b"standalone-vault-password".to_vec()).unwrap();
    let wrong_password = SecretString::try_from_bytes(b"wrong-vault-password".to_vec()).unwrap();
    let profile = ContactKeyPair::generate().unwrap();

    let vault = VaultDirectory::create_file(&path, &password).unwrap();
    vault.store_private_key("production", &profile).unwrap();
    drop(vault);

    assert!(path.is_file());
    assert!(VaultDirectory::open_file(&path, &wrong_password).is_err());
    let vault = VaultDirectory::open_file(&path, &password).unwrap();
    assert_eq!(vault.list_private_keys().unwrap(), vec!["production"]);
    assert_eq!(
        vault
            .load_private_key("production")
            .unwrap()
            .private_key_record()
            .unwrap(),
        profile.private_key_record().unwrap()
    );
    vault.load_owner_signing_key("production").unwrap();
    drop(vault);

    let read_only = ReadOnlyVaultDirectory::open_file(&path, &password).unwrap();
    assert_eq!(
        read_only.list_private_key_names().unwrap(),
        vec!["production"]
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for parent in [root.join("new-parent"), root.join("new-parent/nested")] {
            assert_eq!(
                fs::metadata(parent).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    let duplicate = VaultDirectory::create_file(&path, &password).unwrap_err();
    assert!(matches!(duplicate, Error::AlreadyExists(_)));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn read_only_vault_reads_credentials_without_creating_missing_signer() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let root = unique_dir("read-only-credentials");
    let _ = fs::remove_dir_all(&root);
    create_private_test_dir(&root);
    let path = root.join("credentials.lbox");
    let password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    let profile = ContactKeyPair::generate().unwrap();
    let profile_password = SecretString::try_from_bytes(b"profile-password".to_vec()).unwrap();
    let remembered = SecretString::try_from_bytes(b"lockbox-password".to_vec()).unwrap();
    let lockbox_id = revault_lockbox_api::LockboxId::new_random().unwrap();

    let vault = VaultDirectory::create_file(&path, &password).unwrap();
    vault.store_private_key("legacy", &profile).unwrap();
    vault
        .store_password_profile("service", &profile_password, false)
        .unwrap();
    vault
        .remember_lockbox_password(lockbox_id, &remembered)
        .unwrap();
    let signer = vault.load_owner_signing_key("legacy").unwrap();
    let container_signer = vault.load_owner_signing_key("default").unwrap();
    drop(vault);

    let first = ReadOnlyVaultDirectory::open_file(&path, &password).unwrap();
    let second = ReadOnlyVaultDirectory::open_file(&path, &password).unwrap();
    assert_eq!(
        first
            .load_private_key("legacy")
            .unwrap()
            .private_key_record()
            .unwrap(),
        profile.private_key_record().unwrap()
    );
    assert_eq!(
        second
            .load_private_key_generation("legacy", 1)
            .unwrap()
            .private_key_record()
            .unwrap(),
        profile.private_key_record().unwrap()
    );
    assert_eq!(
        first.load_owner_signing_key("legacy").unwrap().public_key(),
        signer.public_key()
    );
    assert_eq!(
        second
            .load_owner_signing_key_generation("legacy", 1)
            .unwrap()
            .public_key(),
        signer.public_key()
    );
    assert_eq!(
        first
            .load_profile_password("service")
            .unwrap()
            .with_bytes(|bytes| bytes.to_vec())
            .unwrap(),
        b"profile-password"
    );
    assert_eq!(
        second
            .remembered_lockbox_password(lockbox_id)
            .unwrap()
            .unwrap()
            .with_bytes(|bytes| bytes.to_vec())
            .unwrap(),
        b"lockbox-password"
    );
    let writer_path = path.clone();
    let (started_tx, started_rx) = mpsc::channel();
    let (opened_tx, opened_rx) = mpsc::channel();
    let writer = std::thread::spawn(move || {
        let writer_password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
        started_tx.send(()).unwrap();
        let result = VaultDirectory::open_file(&writer_path, &writer_password).map(|_| ());
        opened_tx.send(result).unwrap();
    });
    started_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    assert!(opened_rx
        .recv_timeout(std::time::Duration::from_millis(200))
        .is_err());
    drop(first);
    drop(second);
    opened_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap()
        .unwrap();
    writer.join().unwrap();

    let mut raw =
        Lockbox::open_for_write(&path, LockboxOpen::Password(&password), &container_signer)
            .unwrap();
    let signing_variable = VariableName::new("LOCKBOX_VAULT_SIGNING_KEY_6C6567616379").unwrap();
    raw.delete_variable(&signing_variable).unwrap();
    raw.commit().unwrap();
    drop(raw);
    let _ = revault_vault_api::forget_owner_signing_key(&path.to_string_lossy(), "legacy");
    let before = fs::read(&path).unwrap();
    let view = ReadOnlyVaultDirectory::open_file(&path, &password).unwrap();
    assert!(matches!(
        view.load_owner_signing_key("legacy"),
        Err(Error::NotFound(_))
    ));
    drop(view);
    assert_eq!(fs::read(&path).unwrap(), before);
    let raw = Lockbox::open(&path, LockboxOpen::Password(&password)).unwrap();
    assert!(raw
        .variable_sensitivity(&signing_variable)
        .unwrap()
        .is_none());
    drop(raw);
    let reopened = VaultDirectory::open_file(&path, &password).unwrap();
    let replacement = reopened.load_owner_signing_key("legacy").unwrap();
    drop(reopened);
    let view = ReadOnlyVaultDirectory::open_file(&path, &password).unwrap();
    assert_eq!(
        view.load_owner_signing_key("legacy").unwrap().public_key(),
        replacement.public_key()
    );
    drop(view);
    let _ = fs::remove_dir_all(root);
}

fn unique_path(label: &str) -> PathBuf {
    unique_dir(label).join("test.lbox")
}

fn unique_dir(label: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../target/test-tmp")
        .join(format!(
            "lockbox-vault-api-{label}-{}-{}.lbox",
            std::process::id(),
            monotonic_suffix()
        ))
}

fn install_default_vault(label: &str) -> (PathBuf, EnvVarGuard, EnvVarGuard) {
    let root = unique_dir(label);
    let _ = fs::remove_dir_all(&root);
    let dir_guard = EnvVarGuard::set("LOCKBOX_VAULT_DIR", &root);
    let password_guard = EnvVarGuard::set("LOCKBOX_VAULT_PASSWORD", "vault-password");
    let password = SecretString::try_from_bytes(b"vault-password".to_vec()).unwrap();
    VaultDirectory::open_or_create_default(&password).unwrap();
    (root, dir_guard, password_guard)
}

fn monotonic_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn wait_for_path(path: &Path) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !path.exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

struct EnvVarGuard {
    name: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvVarGuard {
    fn set(name: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
        let previous = std::env::var_os(name);
        std::env::set_var(name, value);
        Self { name, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var(self.name, value),
            None => std::env::remove_var(self.name),
        }
    }
}

fn corrupt_key_directories(path: &PathBuf) {
    let mut bytes = fs::read(path).unwrap();
    let magic = b"LBX1KEY\0";
    let mut offset = 0usize;
    while offset + magic.len() <= bytes.len() {
        if &bytes[offset..offset + magic.len()] == magic {
            bytes[offset] ^= 0x55;
            offset += magic.len();
        } else {
            offset += 1;
        }
    }
    fs::write(path, bytes).unwrap();
}
