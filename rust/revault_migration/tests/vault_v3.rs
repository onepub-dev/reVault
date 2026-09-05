use revault_migration::{import_vault_v2, upgrade_vault_artifact};
use revault_vault_api::{SecretString, VaultDirectory};

#[test]
fn historical_v2_upgrades_and_old_writer_refuses_v3_without_changes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("v2");
    let upgraded = temp.path().join("v3");
    // This is a migration-library fixture, using pinned historical APIs to
    // create the old format that the current writer cannot produce.
    let old_password =
        revault_vault_api_v2::SecretString::try_from_slice(b"vault password").unwrap();
    let old = revault_vault_api_v2::VaultDirectory::replace(&root, &old_password).unwrap();
    let key = revault_lockbox_api_v2::ContactKeyPair::generate().unwrap();
    old.store_private_key("default", &key).unwrap();
    old.store_profile_email("default", "owner@example.test")
        .unwrap();
    old.seed_default_form_definitions().unwrap();
    let original_key = key.public_key().to_bytes();
    let artifact = temp.path().join("old.migration");
    revault_migrate_vault_v2::export_vault_v2(&old, &artifact, b"artifact password", [42; 16])
        .unwrap();
    let current_artifact = temp.path().join("current.migration");
    upgrade_vault_artifact(&artifact, &current_artifact, b"artifact password").unwrap();
    let password = SecretString::try_from_slice(b"vault password").unwrap();
    import_vault_v2(
        &current_artifact,
        b"artifact password",
        &upgraded,
        &password,
    )
    .unwrap();
    let vault = VaultDirectory::open_or_create(&upgraded, &password).unwrap();
    assert_eq!(vault.structure_version().unwrap(), 3);
    assert_eq!(
        vault
            .load_private_key("default")
            .unwrap()
            .public_key()
            .to_bytes(),
        original_key
    );
    assert_eq!(
        vault.profile_email("default").unwrap().as_deref(),
        Some("owner@example.test")
    );
    assert!(!vault.list_form_definitions().unwrap().is_empty());
    vault.create_password_profile("server").unwrap();
    let server_password = vault.load_profile_password("server").unwrap();
    drop(vault);
    let file = upgraded.join("local-vault.lbox");
    let before = std::fs::read(&file).unwrap();
    assert!(
        revault_vault_api_v2::VaultDirectory::open_or_create(&upgraded, &old_password).is_err()
    );
    let new_password =
        revault_vault_api_v2::SecretString::try_from_slice(b"replacement password").unwrap();
    assert!(revault_vault_api_v2::VaultDirectory::change_password(
        &upgraded,
        &old_password,
        &new_password
    )
    .is_err());
    assert_eq!(before, std::fs::read(&file).unwrap());
    let vault = VaultDirectory::open_or_create(&upgraded, &password).unwrap();
    assert_eq!(
        server_password,
        vault.load_profile_password("server").unwrap()
    );
}
