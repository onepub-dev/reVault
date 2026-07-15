use revault_lockbox_api_v1::{
    ContactKeyPair, Lockbox, LockboxOpen, LockboxPath, SecretString, VariableName,
};
use revault_migrate_vault_v1::export_vault_v1;
use revault_migration_format::{ArtifactKind, ArtifactReader, MigrationRecord, VaultRecord};
use revault_vault_api_v1::VaultDirectory;
use std::fs::File;
use std::io::{BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn reports_exact_vault_v1_capabilities() {
    let output = Command::new(env!("CARGO_BIN_EXE_revault-migrate-vault-v1"))
        .arg("capabilities")
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["protocol"], 1);
    assert_eq!(value["artifact"], "vault");
    assert_eq!(value["native_version"], 1);
    assert_eq!(value["migration_schema"], 1);
}

#[test]
fn child_process_exports_published_api_v1_without_modifying_source() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("vault-v1");
    let output = temp.path().join("vault.migration");
    let password = SecretString::try_from_slice(b"vault password").unwrap();
    let vault = VaultDirectory::replace(&root, &password).unwrap();
    vault
        .store_private_key("default", &ContactKeyPair::generate().unwrap())
        .unwrap();
    drop(vault);
    let vault_path = root.join("local-vault.lbox");
    let before = std::fs::read(&vault_path).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_revault-migrate-vault-v1"))
        .args(["migrate", "vault", "export", "--source"])
        .arg(&root)
        .arg("--output")
        .arg(&output)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    write_secret_frame(&mut stdin, &[b"vault password", b"artifact password"]);
    drop(stdin);
    assert!(child.wait().unwrap().success());
    assert_eq!(std::fs::read(&vault_path).unwrap(), before);

    let reader = ArtifactReader::new(
        BufReader::new(File::open(output).unwrap()),
        b"artifact password",
    )
    .unwrap();
    assert_eq!(reader.header().artifact_kind, ArtifactKind::Vault);
    assert_eq!(reader.header().source_native_version, 1);
}

fn write_secret_frame(output: &mut impl Write, secrets: &[&[u8]]) {
    output.write_all(b"LBXMIPC1").unwrap();
    output
        .write_all(&(secrets.len() as u32).to_le_bytes())
        .unwrap();
    for secret in secrets {
        output
            .write_all(&(secret.len() as u32).to_le_bytes())
            .unwrap();
        output.write_all(secret).unwrap();
    }
}

#[test]
fn prehistory_identity_is_exported_without_mutating_the_v1_vault() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("vault-v1");
    let output = temp.path().join("vault.migration");
    let password = SecretString::try_from_slice(b"vault password").unwrap();
    let vault = VaultDirectory::replace(&root, &password).unwrap();
    vault
        .store_private_key("default", &ContactKeyPair::generate().unwrap())
        .unwrap();
    let signing = vault.load_owner_signing_key("default").unwrap();
    drop(vault);

    let path = root.join("local-vault.lbox");
    let mut lockbox =
        Lockbox::open_for_write(&path, LockboxOpen::Password(&password), &signing).unwrap();
    lockbox
        .delete(&LockboxPath::new("/identity_histories/default.lbih").unwrap())
        .unwrap();
    lockbox
        .delete_variable(
            &VariableName::new("LOCKBOX_VAULT_PRIVATE_KEY_64656661756C74_GEN_0001").unwrap(),
        )
        .unwrap();
    lockbox
        .delete_variable(
            &VariableName::new("LOCKBOX_VAULT_SIGNING_KEY_64656661756C74_GEN_0001").unwrap(),
        )
        .unwrap();
    lockbox.commit().unwrap();
    drop(lockbox);
    let before = std::fs::read(&path).unwrap();

    export_vault_v1(
        &root,
        b"vault password",
        &output,
        b"artifact password",
        [8; 16],
    )
    .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), before);

    let mut reader = ArtifactReader::new(
        BufReader::new(File::open(output).unwrap()),
        b"artifact password",
    )
    .unwrap();
    let mut found = false;
    while let Some(record) = reader.next_json::<MigrationRecord>().unwrap() {
        if let MigrationRecord::Vault(VaultRecord::Profile(profile)) = record {
            assert_eq!(profile.name, "default");
            assert_eq!(profile.active_generation, 1);
            assert_eq!(profile.generations.len(), 1);
            found = true;
        }
    }
    assert!(found);
}
