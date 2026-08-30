mod common;

use common::TestTempDir;
use revault_lockbox_api::{
    ContactKeyPair, Lockbox, LockboxOpen, SecretString, SecretVec, LOCKBOX_FORMAT_VERSION,
};
use revault_lockbox_api_v1::{
    Lockbox as V1Lockbox, LockboxPath as V1LockboxPath, LockboxProtection as V1LockboxProtection,
    OwnerSigningKeyPair as V1OwnerSigningKeyPair, SecretString as V1SecretString,
};
use revault_lockbox_api_vault_v1::ContactKeyPair as V1VaultContactKeyPair;
use revault_vault_api::VaultDirectory;
use revault_vault_api_v1::VaultDirectory as V1VaultDirectory;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const VAULT_PASSWORD: &str = "migration vault password";
const LOCKBOX_PASSWORD: &str = "migration lockbox password";
const ARTIFACT_PASSWORD: &str = "migration artifact password";

#[test]
fn vault_migration_commands_and_options_execute_end_to_end() {
    let fixture = Fixture::new("migration-vault-e2e");
    fixture.init_current_vault();

    let direct_output = fixture.root.join("direct-vault");
    fixture.success(&[
        "migrate",
        "vault",
        "--output",
        path(&direct_output),
        "--exporter",
        "unused-for-current-format",
    ]);
    assert_current_vault(&direct_output);

    let conflict = fixture.run(&[
        "migrate",
        "vault",
        "--replace",
        "--output",
        path(&fixture.root.join("conflict")),
    ]);
    assert_failure_contains(&conflict, "cannot be used with");

    let export = fixture.root.join("vault-export.migration");
    let exported = fixture.run_with_stdin(
        &[
            "migrate",
            "vault",
            "export",
            "-o",
            path(&export),
            "--vault-password-stdin",
            "--migration-password-stdin",
        ],
        &format!("{VAULT_PASSWORD}\n{ARTIFACT_PASSWORD}\n"),
        true,
    );
    assert_success(&exported);
    assert!(export.is_file());

    fixture.success(&["migrate", "vault", "verify", path(&export)]);
    let upgraded = fixture.root.join("vault-upgraded.migration");
    fixture.success(&[
        "migrate",
        "vault",
        "upgrade",
        path(&export),
        "--output",
        path(&upgraded),
    ]);
    fixture.success(&["migrate", "vault", "verify", path(&upgraded)]);

    let imported = fixture.root.join("vault-imported");
    fixture.success(&[
        "migrate",
        "vault",
        "import",
        path(&upgraded),
        "--output",
        path(&imported),
    ]);
    assert_current_vault(&imported);

    fixture.success(&["migrate", "vault", "--replace"]);
    assert_current_vault(&fixture.vault);
    assert_current_vault(&fixture.root.join("vault.v2.pre-migration"));
}

#[test]
fn archive_migration_commands_and_options_execute_end_to_end() {
    let fixture = Fixture::new("migration-archive-e2e");
    fixture.init_current_vault();
    let advanced_source = fixture.create_archive("advanced.lbox");

    let artifact = fixture.root.join("archive-export.migration");
    let exported = fixture.run_with_stdin(
        &[
            "migrate",
            "archive",
            "export",
            path(&advanced_source),
            "-o",
            path(&artifact),
            "--migration-password-stdin",
        ],
        &format!("{ARTIFACT_PASSWORD}\n"),
        false,
    );
    assert_success(&exported);
    fixture.success(&["migrate", "archive", "verify", path(&artifact)]);

    let upgraded = fixture.root.join("archive-upgraded.migration");
    fixture.success(&[
        "migrate",
        "archive",
        "upgrade",
        path(&artifact),
        "--output",
        path(&upgraded),
    ]);
    fixture.success(&["migrate", "archive", "verify", path(&upgraded)]);

    let imported = fixture.root.join("archive-imported.lbox");
    fixture.success(&[
        "migrate",
        "archive",
        "import",
        path(&upgraded),
        "--output",
        path(&imported),
    ]);
    Lockbox::inspect_file(&imported).unwrap();

    let direct_source = fixture.create_archive("direct.lbox");
    let direct_output = fixture.root.join("direct-migrated.lbox");
    fixture.success(&[
        "migrate",
        "archive",
        path(&direct_source),
        "--output",
        path(&direct_output),
        "--exporter",
        "unused-for-current-format",
    ]);
    Lockbox::inspect_file(&direct_output).unwrap();

    let replace_source = fixture.create_archive("replace.lbox");
    fixture.success(&["migrate", "archive", path(&replace_source), "--replace"]);
    Lockbox::inspect_file(&replace_source).unwrap();
    Lockbox::inspect_file(fixture.root.join(format!(
        "replace.lbox.v{LOCKBOX_FORMAT_VERSION}.pre-migration"
    )))
    .unwrap();

    let conflict = fixture.run(&[
        "migrate",
        "archive",
        path(&replace_source),
        "--replace",
        "--output",
        path(&fixture.root.join("archive-conflict.lbox")),
    ]);
    assert_failure_contains(&conflict, "cannot be used with");
}

#[test]
fn vault_v1_replace_uses_the_explicit_historical_exporter() {
    let fixture = Fixture::new("migration-v1-e2e");
    fixture.init_v1_vault();
    let exporter = build_historical_vault_exporter();

    let output = fixture.run(&[
        "migrate",
        "vault",
        "--replace",
        "--exporter",
        path(&exporter),
    ]);
    assert_success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("format version 2"));
    assert_current_vault(&fixture.vault);
    assert!(fixture.root.join("vault.v1.pre-migration").is_dir());
}

#[test]
fn archive_v1_migrates_end_to_end_with_the_historical_exporter() {
    let fixture = Fixture::new("migration-archive-v1-e2e");
    fixture.init_current_vault();
    let source = fixture.root.join("legacy.lbox");
    let password = V1SecretString::try_from_slice(LOCKBOX_PASSWORD.as_bytes()).unwrap();
    let signing = V1OwnerSigningKeyPair::generate().unwrap();
    let mut legacy =
        V1Lockbox::create_in_memory(V1LockboxProtection::Password(&password), &signing).unwrap();
    legacy
        .add_file(
            &V1LockboxPath::new("/legacy.txt").unwrap(),
            b"legacy archive",
            false,
        )
        .unwrap();
    legacy.commit().unwrap();
    std::fs::write(&source, legacy.try_to_bytes().unwrap()).unwrap();

    let output = fixture.root.join("migrated.lbox");
    let exporter = build_historical_archive_exporter();
    fixture.success(&[
        "migrate",
        "archive",
        path(&source),
        "--output",
        path(&output),
        "--exporter",
        path(&exporter),
    ]);
    let migrated = Lockbox::open(
        &output,
        LockboxOpen::Password(&SecretString::try_from_slice(LOCKBOX_PASSWORD.as_bytes()).unwrap()),
    )
    .unwrap();
    assert_eq!(
        migrated
            .get_file(&revault_lockbox_api::LockboxPath::new("/legacy.txt").unwrap())
            .unwrap(),
        b"legacy archive"
    );
}

#[test]
fn archive_v1_contact_only_migration_uses_keys_from_the_current_vault() {
    let fixture = Fixture::new("migration-contact-archive-v1-e2e");
    fixture.init_current_vault();
    let source = fixture.root.join("legacy-contact.lbox");
    let legacy_contact = revault_lockbox_api_v1::ContactKeyPair::generate().unwrap();
    let legacy_public = legacy_contact.public_key();
    let signing = V1OwnerSigningKeyPair::generate().unwrap();
    let mut legacy = V1Lockbox::create_in_memory(
        V1LockboxProtection::ContactPublicKey {
            name: Some("legacy".to_string()),
            contact: legacy_public,
        },
        &signing,
    )
    .unwrap();
    legacy
        .add_file(
            &V1LockboxPath::new("/contact.txt").unwrap(),
            b"opened by a migrated profile key",
            false,
        )
        .unwrap();
    legacy.commit().unwrap();
    std::fs::write(&source, legacy.try_to_bytes().unwrap()).unwrap();

    let legacy_record = legacy_contact.private_key_record().unwrap();
    let record_bytes = legacy_record.with_bytes(|bytes| bytes.to_vec()).unwrap();
    let current_contact =
        ContactKeyPair::from_private_key_record(SecretVec::try_from_vec(record_bytes).unwrap())
            .unwrap();
    let vault_password = SecretString::try_from_slice(VAULT_PASSWORD.as_bytes()).unwrap();
    let vault = VaultDirectory::open_or_create(&fixture.vault, &vault_password).unwrap();
    vault.store_private_key("legacy", &current_contact).unwrap();

    let output = fixture.root.join("migrated-contact.lbox");
    let exporter = build_historical_archive_exporter();
    fixture.success(&[
        "migrate",
        "archive",
        path(&source),
        "--output",
        path(&output),
        "--exporter",
        path(&exporter),
    ]);
    let migrated = Lockbox::open(&output, LockboxOpen::ContactKeyPair(current_contact)).unwrap();
    assert_eq!(
        migrated
            .get_file(&revault_lockbox_api::LockboxPath::new("/contact.txt").unwrap())
            .unwrap(),
        b"opened by a migrated profile key"
    );
}

#[test]
fn automatic_historical_exporter_install_hides_cargo_output() {
    let fixture = Fixture::new("migration-v1-install-e2e");
    fixture.init_v1_vault();
    let exporter = build_historical_vault_exporter();
    let fake_cargo = build_fake_cargo(&fixture.root);
    let home = fixture.root.join("home");

    let output = fixture
        .command(&["migrate", "vault", "--replace"])
        .env("CARGO", fake_cargo)
        .env("HOME", home)
        .env("FAKE_EXPORTER_SOURCE", exporter)
        .output()
        .unwrap();
    assert_success(&output);
    let rendered = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(rendered.contains("Installing historical reVault exporter"));
    assert!(rendered.contains("is ready"));
    assert!(!rendered.contains("Compiling revault"));
    assert!(!rendered.contains("Installed package"));
    assert!(!rendered.contains("add `"));
    assert!(!rendered.contains("to your PATH"));
    assert_current_vault(&fixture.vault);
}

struct Fixture {
    _temp: TestTempDir,
    root: PathBuf,
    vault: PathBuf,
    agent: PathBuf,
}

impl Fixture {
    fn new(prefix: &str) -> Self {
        let temp = TestTempDir::new(prefix);
        let root = temp.path().to_path_buf();
        Self {
            vault: root.join("vault"),
            agent: root.join("agent"),
            _temp: temp,
            root,
        }
    }

    fn init_current_vault(&self) {
        self.success(&["vault", "init"]);
    }

    fn init_v1_vault(&self) {
        let password = V1SecretString::try_from_slice(VAULT_PASSWORD.as_bytes()).unwrap();
        let vault = V1VaultDirectory::replace(&self.vault, &password).unwrap();
        vault
            .store_private_key("default", &V1VaultContactKeyPair::generate().unwrap())
            .unwrap();
        vault
            .store_identity_email("default", "migration@example.test")
            .unwrap();
    }

    fn create_archive(&self, name: &str) -> PathBuf {
        let archive = self.root.join(name);
        self.success(&[path(&archive), "create"]);
        archive
    }

    fn success(&self, args: &[&str]) -> Output {
        let output = self.run(args);
        assert_success(&output);
        output
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    fn run_with_stdin(
        &self,
        args: &[&str],
        stdin: &str,
        remove_vault_password_env: bool,
    ) -> Output {
        let mut command = self.command(args);
        command.env_remove("LOCKBOX_MIGRATION_PASSWORD");
        if remove_vault_password_env {
            command.env_remove("LOCKBOX_VAULT_PASSWORD");
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lockbox"));
        command
            .args(args)
            .env("LOCKBOX_VAULT_DIR", &self.vault)
            .env("LOCKBOX_VAULT_PASSWORD", VAULT_PASSWORD)
            .env("LOCKBOX_PASSWORD", LOCKBOX_PASSWORD)
            .env("LOCKBOX_KEY", "migration-test-content-key")
            .env("LOCKBOX_MIGRATION_PASSWORD", ARTIFACT_PASSWORD)
            .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
            .env("LOCKBOX_SESSION_AGENT_DIR", &self.agent)
            .env("LOCKBOX_SESSION_AGENT_LOG", self.agent.join("agent.log"));
        command
    }
}

fn build_historical_vault_exporter() -> PathBuf {
    let status = Command::new(env!("CARGO"))
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join(".."))
        .args([
            "build",
            "--quiet",
            "-p",
            "revault_migrate_vault_v1",
            "--bin",
            "revault-migrate-vault-v1",
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let extension = if cfg!(windows) { ".exe" } else { "" };
    std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../target"))
        .join("debug")
        .join(format!("revault-migrate-vault-v1{extension}"))
}

fn build_historical_archive_exporter() -> PathBuf {
    let status = Command::new(env!("CARGO"))
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join(".."))
        .args([
            "build",
            "--quiet",
            "-p",
            "revault_migrate_archive_v1",
            "--bin",
            "revault-migrate-archive-v1",
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let extension = if cfg!(windows) { ".exe" } else { "" };
    std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../target"))
        .join("debug")
        .join(format!("revault-migrate-archive-v1{extension}"))
}

fn build_fake_cargo(output_dir: &Path) -> PathBuf {
    let extension = if cfg!(windows) { ".exe" } else { "" };
    let output = output_dir.join(format!("fake-cargo{extension}"));
    let status = Command::new("rustc")
        .arg("--edition=2021")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/fake_cargo.rs"))
        .arg("-o")
        .arg(&output)
        .status()
        .unwrap();
    assert!(status.success());
    output
}

fn assert_current_vault(root: &Path) {
    assert!(root.join("local-vault.lbox").is_file());
    let password =
        revault_vault_api::SecretString::try_from_slice(VAULT_PASSWORD.as_bytes()).unwrap();
    let vault = VaultDirectory::open_or_create(root, &password).unwrap();
    assert_eq!(
        vault.structure_version().unwrap(),
        revault_vault_api::CURRENT_VAULT_STRUCTURE_VERSION
    );
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure_contains(output: &Output, expected: &str) {
    assert!(!output.status.success());
    let rendered = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(rendered.contains(expected), "output was:\n{rendered}");
}

fn path(value: &Path) -> &str {
    value.to_str().unwrap()
}
