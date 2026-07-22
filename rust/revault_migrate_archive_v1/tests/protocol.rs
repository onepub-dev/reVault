use revault_lockbox_api::{
    ContactKeyPair, Lockbox, LockboxOpen, LockboxPath, LockboxProtection, OwnerSigningKeyPair,
    SecretString,
};
use revault_migration_format::{ArtifactKind, ArtifactReader};
use revault_vault_api::VaultDirectory;
use std::fs::File;
use std::io::{BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn reports_exact_archive_v1_capabilities() {
    let output = Command::new(env!("CARGO_BIN_EXE_revault-migrate-archive-v1"))
        .arg("capabilities")
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["protocol"], 2);
    assert_eq!(value["artifact"], "archive");
    assert_eq!(value["native_version"], 1);
    assert_eq!(value["migration_schema"], 1);
}

#[test]
fn child_process_streams_archive_v1_and_preserves_source() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.lbox");
    let output = temp.path().join("archive.migration");
    let vault_root = temp.path().join("vault");
    let vault_password = SecretString::try_from_slice(b"vault password").unwrap();
    VaultDirectory::open_or_create(&vault_root, &vault_password).unwrap();
    let password = SecretString::try_from_slice(b"archive password").unwrap();
    let signing = OwnerSigningKeyPair::generate().unwrap();
    let mut lockbox =
        Lockbox::create_in_memory(LockboxProtection::Password(&password), &signing).unwrap();
    lockbox
        .add_file(
            &LockboxPath::new("/large.bin").unwrap(),
            &vec![7u8; 9 * 1024 * 1024],
            false,
        )
        .unwrap();
    lockbox.commit().unwrap();
    lockbox.write_to_path(&source).unwrap();
    let before = std::fs::read(&source).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_revault-migrate-archive-v1"))
        .args(["migrate", "archive", "export"])
        .arg(&source)
        .arg("--output")
        .arg(&output)
        .env("LOCKBOX_VAULT_DIR", &vault_root)
        .env("LOCKBOX_PASSWORD", "archive password")
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    write_secret_frame(&mut stdin, &[b"vault password", b"artifact password"]);
    drop(stdin);
    assert!(child.wait().unwrap().success());
    assert_eq!(std::fs::read(&source).unwrap(), before);

    let reader = ArtifactReader::new(
        BufReader::new(File::open(output).unwrap()),
        b"artifact password",
    )
    .unwrap();
    assert_eq!(reader.header().artifact_kind, ArtifactKind::Archive);
    assert_eq!(reader.header().source_native_version, 1);
    assert!(Lockbox::open(&source, LockboxOpen::Password(&password)).is_ok());
}

#[test]
fn child_process_opens_archive_with_a_migrated_vault_profile_key() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.lbox");
    let output = temp.path().join("archive.migration");
    let vault_root = temp.path().join("vault");
    let vault_password = SecretString::try_from_slice(b"vault password").unwrap();
    let contact = ContactKeyPair::generate().unwrap();
    let vault = VaultDirectory::open_or_create(&vault_root, &vault_password).unwrap();
    vault.store_private_key("legacy", &contact).unwrap();

    let signing = OwnerSigningKeyPair::generate().unwrap();
    let mut lockbox = Lockbox::create_in_memory(
        LockboxProtection::ContactPublicKey {
            name: Some("legacy".to_string()),
            contact: contact.public_key(),
        },
        &signing,
    )
    .unwrap();
    lockbox
        .add_file(
            &LockboxPath::new("/from-vault.txt").unwrap(),
            b"vault profile opened this archive",
            false,
        )
        .unwrap();
    lockbox.commit().unwrap();
    lockbox.write_to_path(&source).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_revault-migrate-archive-v1"))
        .args(["migrate", "archive", "export"])
        .arg(&source)
        .arg("--output")
        .arg(&output)
        .env("LOCKBOX_VAULT_DIR", &vault_root)
        .env_remove("LOCKBOX_PASSWORD")
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    write_secret_frame(&mut stdin, &[b"vault password", b"artifact password"]);
    drop(stdin);
    assert!(child.wait().unwrap().success());

    let reader = ArtifactReader::new(
        BufReader::new(File::open(output).unwrap()),
        b"artifact password",
    )
    .unwrap();
    assert_eq!(reader.header().artifact_kind, ArtifactKind::Archive);
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
