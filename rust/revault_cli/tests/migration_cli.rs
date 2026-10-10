mod common;

use common::{CommandTestExt, TestTempDir};
use revault_lockbox_api::{
    ContactKeyPair, Lockbox, LockboxOpen, SecretString, SecretVec, LOCKBOX_FORMAT_VERSION,
};
use revault_lockbox_api_container_v1::{
    ContactKeyPair as StructureV2ContactKeyPair, Lockbox as ContainerV1Lockbox,
    LockboxProtection as ContainerV1LockboxProtection,
    OwnerSigningKeyPair as ContainerV1OwnerSigningKeyPair,
};
use revault_lockbox_api_v1::{
    Lockbox as V1Lockbox, LockboxPath as V1LockboxPath, LockboxProtection as V1LockboxProtection,
    OwnerSigningKeyPair as V1OwnerSigningKeyPair, SecretString as V1SecretString,
};
use revault_lockbox_api_vault_v1::ContactKeyPair as V1VaultContactKeyPair;
use revault_vault_api::VaultDirectory;
use revault_vault_api_container_v1::{
    SecretString as StructureV2SecretString, VaultDirectory as StructureV2VaultDirectory,
};
use revault_vault_api_v1::VaultDirectory as V1VaultDirectory;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const VAULT_PASSWORD: &str = "migration vault password";
const LOCKBOX_PASSWORD: &str = "migration lockbox password";
const ARTIFACT_PASSWORD: &str = "migration artifact password";

#[test]
fn migrated_mirror_preserves_configuration_and_update_lifecycle() {
    let fixture = Fixture::new("migration-mirror-lifecycle");
    fixture.init_current_vault();
    let archive = fixture.create_archive("source.lbox");
    let host = fixture.root.join("host");
    std::fs::create_dir(&host).unwrap();
    for (name, bytes) in [
        ("replace.txt", b"before".as_slice()),
        ("remove.txt", b"remove"),
        ("keep.txt", b"keep"),
    ] {
        std::fs::write(host.join(name), bytes).unwrap();
    }
    fixture.success(&[
        path(&archive),
        "mirror",
        "project",
        "create",
        "--from",
        path(&host),
        "--to",
        "/project",
        "--strict",
    ]);
    fixture.success(&[path(&archive), "mirror", "project", "update", "--force"]);
    let before = fixture.success(&[
        path(&archive),
        "mirror",
        "project",
        "info",
        "--format",
        "json",
    ]);
    let destination = fixture.root.join("migrated.lbox");
    fixture.success(&[
        "doctor",
        "migrate",
        "lockbox",
        path(&archive),
        "--output",
        path(&destination),
    ]);
    let after = fixture.success(&[
        path(&destination),
        "mirror",
        "project",
        "info",
        "--format",
        "json",
    ]);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&before.stdout).unwrap(),
        serde_json::from_slice::<serde_json::Value>(&after.stdout).unwrap()
    );
    fixture.success(&[path(&destination), "mirror", "project", "update"]);
    assert_eq!(
        fixture
            .success(&[path(&destination), "cat", "/project/replace.txt"])
            .stdout,
        b"before"
    );
    std::fs::write(host.join("replace.txt"), b"replacement bytes").unwrap();
    std::fs::write(host.join("added.txt"), b"added bytes").unwrap();
    std::fs::remove_file(host.join("remove.txt")).unwrap();
    fixture.success(&[path(&destination), "mirror", "project", "update", "--force"]);
    for (name, bytes) in [
        ("replace.txt", b"replacement bytes".as_slice()),
        ("added.txt", b"added bytes"),
        ("keep.txt", b"keep"),
    ] {
        assert_eq!(
            fixture
                .success(&[path(&destination), "cat", &format!("/project/{name}")])
                .stdout,
            bytes
        );
    }
    assert!(!fixture
        .run(&[path(&destination), "cat", "/project/remove.txt"])
        .status
        .success());
    fixture.success(&[path(&destination), "mirror", "project", "update"]);
    // An empty source must still trigger the public safety refusal after migration.
    for name in ["replace.txt", "added.txt", "keep.txt"] {
        std::fs::remove_file(host.join(name)).unwrap();
    }
    let refused = fixture.run(&[path(&destination), "mirror", "project", "update", "--force"]);
    assert_failure_contains(&refused, "--allow-empty");
    for (name, bytes) in [
        ("replace.txt", b"replacement bytes".as_slice()),
        ("added.txt", b"added bytes"),
        ("keep.txt", b"keep"),
    ] {
        assert_eq!(
            fixture
                .success(&[path(&destination), "cat", &format!("/project/{name}")])
                .stdout,
            bytes
        );
    }
    // Migration must not change the original archive.
    assert_eq!(
        fixture
            .success(&[path(&archive), "cat", "/project/replace.txt"])
            .stdout,
        b"before"
    );
    assert_eq!(
        fixture
            .success(&[path(&archive), "cat", "/project/remove.txt"])
            .stdout,
        b"remove"
    );
}

#[test]
fn vault_migration_commands_and_options_execute_end_to_end() {
    let fixture = Fixture::new("migration-vault-e2e");
    fixture.init_current_vault();

    let direct_output = fixture.root.join("direct-vault");
    fixture.success(&[
        "doctor",
        "migrate",
        "vault",
        "--output",
        path(&direct_output),
        "--exporter",
        "unused-for-current-format",
    ]);
    assert_current_vault(&direct_output);

    let conflict = fixture.run(&[
        "doctor",
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
            "doctor",
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

    fixture.success(&["doctor", "migrate", "vault", "verify", path(&export)]);
    let upgraded = fixture.root.join("vault-upgraded.migration");
    fixture.success(&[
        "doctor",
        "migrate",
        "vault",
        "upgrade",
        path(&export),
        "--output",
        path(&upgraded),
    ]);
    fixture.success(&["doctor", "migrate", "vault", "verify", path(&upgraded)]);

    let imported = fixture.root.join("vault-imported");
    fixture.success(&[
        "doctor",
        "migrate",
        "vault",
        "import",
        path(&upgraded),
        "--output",
        path(&imported),
    ]);
    assert_current_vault(&imported);

    let already_current = fixture.run(&["doctor", "migrate", "vault", "--replace"]);
    assert_success(&already_current);
    assert!(String::from_utf8_lossy(&already_current.stdout).contains("No migration needed"));
    assert_current_vault(&fixture.vault);
    assert!(!fixture.root.join("vault.v2-v3.pre-migration").exists());
}

#[test]
fn archive_migration_commands_and_options_execute_end_to_end() {
    let fixture = Fixture::new("migration-archive-e2e");
    fixture.init_current_vault();
    let advanced_source = fixture.create_archive("advanced.lbox");

    let artifact = fixture.root.join("archive-export.migration");
    let exported = fixture.run_with_stdin(
        &[
            "doctor",
            "migrate",
            "lockbox",
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
    fixture.success(&["doctor", "migrate", "lockbox", "verify", path(&artifact)]);

    let upgraded = fixture.root.join("archive-upgraded.migration");
    fixture.success(&[
        "doctor",
        "migrate",
        "lockbox",
        "upgrade",
        path(&artifact),
        "--output",
        path(&upgraded),
    ]);
    fixture.success(&["doctor", "migrate", "lockbox", "verify", path(&upgraded)]);

    let imported = fixture.root.join("archive-imported.lbox");
    fixture.success(&[
        "doctor",
        "migrate",
        "lockbox",
        "import",
        path(&upgraded),
        "--output",
        path(&imported),
    ]);
    Lockbox::inspect_file(&imported).unwrap();

    let direct_source = fixture.create_archive("direct.lbox");
    let direct_output = fixture.root.join("direct-migrated.lbox");
    fixture.success(&[
        "doctor",
        "migrate",
        "lockbox",
        path(&direct_source),
        "--output",
        path(&direct_output),
        "--exporter",
        "unused-for-current-format",
    ]);
    Lockbox::inspect_file(&direct_output).unwrap();

    let replace_source = fixture.create_archive("replace.lbox");
    let already_current = fixture.run(&[
        "doctor",
        "migrate",
        "lockbox",
        path(&replace_source),
        "--replace",
    ]);
    assert_success(&already_current);
    assert!(String::from_utf8_lossy(&already_current.stdout).contains("No migration needed"));
    Lockbox::inspect_file(&replace_source).unwrap();
    assert!(!fixture
        .root
        .join(format!(
            "replace.lbox.v{LOCKBOX_FORMAT_VERSION}-v{LOCKBOX_FORMAT_VERSION}.pre-migration"
        ))
        .exists());

    let conflict = fixture.run(&[
        "doctor",
        "migrate",
        "lockbox",
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
        "doctor",
        "migrate",
        "vault",
        "--replace",
        "--exporter",
        path(&exporter),
    ]);
    assert_success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).contains(&format!(
        "container format {} and structure version {}",
        revault_lockbox_api::LOCKBOX_FORMAT_VERSION,
        revault_vault_api::CURRENT_VAULT_STRUCTURE_VERSION
    )));
    assert_current_vault(&fixture.vault);
    assert!(fixture.root.join("vault.v1-v3.pre-migration").is_dir());
    let repeated = fixture.run(&["doctor", "migrate", "vault", "--replace"]);
    assert_success(&repeated);
    assert!(String::from_utf8_lossy(&repeated.stdout).contains("No migration needed"));
    assert!(!fixture.root.join("vault.v2-v3.pre-migration").exists());
}

#[test]
fn vault_v2_replace_preserves_profile_keys_and_supports_password_profiles() {
    let fixture = Fixture::new("migration-v2-v3");
    // The current public CLI cannot create a historical v2 vault. Use the
    // pinned v2 library only for this historical setup; migrate and verify
    // the result using separate public CLI invocations.
    let password =
        revault_vault_api_structure_v2::SecretString::try_from_slice(VAULT_PASSWORD.as_bytes())
            .unwrap();
    let vault =
        revault_vault_api_structure_v2::VaultDirectory::replace(&fixture.vault, &password).unwrap();
    let key = revault_lockbox_api_structure_v2::ContactKeyPair::generate().unwrap();
    let original = key.public_key().to_bytes();
    vault.store_private_key("default", &key).unwrap();
    drop(vault);
    let container = fixture.vault.join("local-vault.lbox");
    let before_bytes = std::fs::read(&container).unwrap();
    let refused = fixture.run(&["vault", "profile", "list"]);
    assert_failure_contains(&refused, "Unsupported Vault container format");
    assert_failure_contains(&refused, "migrate vault");
    assert_failure_contains(&refused, path(&container));
    let aliases = fixture.run(&["vault", "lockbox", "alias", "list"]);
    assert_failure_contains(&aliases, path(&container));
    assert_failure_contains(&aliases, "migrate vault");
    let doctor = fixture.run(&["doctor"]);
    assert_success(&doctor);
    let report = String::from_utf8_lossy(&doctor.stdout);
    assert!(report.contains("container format version: 2"), "{report}");
    assert!(report.contains("structure version: 2"), "{report}");
    assert!(
        report.contains("status: upgrade required; run: lbx doctor migrate vault --replace"),
        "{report}"
    );
    assert!(
        report.contains(path(&container)) && report.contains("migrate vault"),
        "{report}"
    );
    assert!(!report.contains("migrate lockbox"), "{report}");
    assert_eq!(std::fs::read(&container).unwrap(), before_bytes);
    let status = Command::new("cargo")
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap())
        .args(["build", "--offline", "-p", "revault_migrate_vault_v2"])
        .status()
        .unwrap();
    assert!(status.success());
    let exporter = Path::new(env!("CARGO_BIN_EXE_lockbox"))
        .parent()
        .unwrap()
        .join(format!(
            "revault-migrate-vault-v2{}",
            std::env::consts::EXE_SUFFIX
        ));
    fixture.success(&[
        "doctor",
        "migrate",
        "vault",
        "--replace",
        "--exporter",
        path(&exporter),
    ]);
    fixture.success(&[
        "vault",
        "profile",
        "export",
        path(&fixture.root.join("profile.pub")),
        "--format",
        "raw",
    ]);
    let exported = std::fs::read(fixture.root.join("profile.pub")).unwrap();
    assert_eq!(
        revault_vault_api::import_public_key(&exported)
            .unwrap()
            .to_bytes(),
        original
    );
    fixture.success(&["vault", "profile", "create", "server", "--password"]);
    let secret = fixture.run(&["vault", "profile", "password", "server"]);
    assert_success(&secret);
    assert_eq!(secret.stdout.len(), 65);
    fixture.success(&["doctor", "migrate", "vault", "--replace"]);
    let repeated = fixture.run(&["vault", "profile", "password", "server"]);
    assert_success(&repeated);
    assert_eq!(secret.stdout, repeated.stdout);
    assert!(fixture.root.join("vault.v2-v3.pre-migration").is_dir());
}

#[test]
fn vault_structure_v2_in_container_v1_migrates_end_to_end() {
    let fixture = Fixture::new("migration-v2-container-v1-e2e");
    fixture.init_structure_v2_container_v1_vault();
    let exporter = build_historical_vault_exporter();

    let before = fixture.run(&["vault", "profile", "list"]);
    assert_failure_contains(&before, "Unsupported Vault container format");
    assert_failure_contains(&before, "Found Lockbox container version 1");

    let output = fixture.run(&[
        "doctor",
        "migrate",
        "vault",
        "--replace",
        "--exporter",
        path(&exporter),
    ]);
    assert_success(&output);
    assert_current_vault(&fixture.vault);
    assert!(fixture.root.join("vault.v1-v3.pre-migration").is_dir());
    let password = SecretString::try_from_slice(VAULT_PASSWORD.as_bytes()).unwrap();
    let migrated = VaultDirectory::open_or_create(&fixture.vault, &password).unwrap();
    assert_eq!(
        migrated.profile_email("default").unwrap().as_deref(),
        Some("mixed@example.test")
    );
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
        "doctor",
        "migrate",
        "lockbox",
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
fn lockbox_v1_with_description_migrates_and_default_error_names_its_path() {
    let fixture = Fixture::new("migration-described-lockbox-v1-e2e");
    fixture.init_current_vault();
    let source = fixture.root.join("described.lbox");
    let password = StructureV2SecretString::try_from_slice(LOCKBOX_PASSWORD.as_bytes()).unwrap();
    let signing = ContainerV1OwnerSigningKeyPair::generate().unwrap();
    let mut legacy = ContainerV1Lockbox::create_in_memory(
        ContainerV1LockboxProtection::Password(&password),
        &signing,
    )
    .unwrap();
    legacy.set_description("Production API keys").unwrap();
    legacy.commit().unwrap();
    // Set a default through the current CLI, then install the historical
    // fixture to simulate a default selected by an earlier release. The
    // current CLI deliberately refuses selecting an unsupported archive, so
    // this stale-default condition cannot be created through a public command.
    fixture.success(&[path(&source), "create"]);
    fixture.success(&["session", "default", path(&source)]);
    let historical_bytes = legacy.try_to_bytes().unwrap();
    std::fs::write(&source, &historical_bytes).unwrap();
    let select_old = fixture.run(&["session", "default", path(&source)]);
    assert_failure_contains(&select_old, "doctor migrate lockbox");
    let before = fixture.run(&["list"]);
    assert_failure_contains(&before, "Unsupported lockbox format");
    assert_failure_contains(&before, path(&source.canonicalize().unwrap()));
    assert_failure_contains(&before, "doctor migrate lockbox");
    assert_eq!(std::fs::read(&source).unwrap(), historical_bytes);

    let exporter = build_historical_archive_exporter();
    fixture.success(&[
        "doctor",
        "migrate",
        "lockbox",
        path(&source),
        "--replace",
        "--exporter",
        path(&exporter),
    ]);
    let migrated = Lockbox::open(
        &source,
        LockboxOpen::Password(&SecretString::try_from_slice(LOCKBOX_PASSWORD.as_bytes()).unwrap()),
    )
    .unwrap();
    assert_eq!(
        migrated.description().unwrap().as_deref(),
        Some("Production API keys")
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
    // Current CLI cannot inspect a format-1 header to remember it. Preserve the
    // historical known record here, as a migrated Vault would provide it.
    vault
        .remember_known_lockbox(
            revault_lockbox_api::LockboxId::from_bytes(*legacy.lockbox_id().as_bytes()),
            &source,
        )
        .unwrap();
    // Release the fixture's vault lock before invoking the migration CLI.
    drop(vault);

    let output = fixture.root.join("migrated-contact.lbox");
    let exporter = build_historical_archive_exporter();
    fixture.success(&[
        "doctor",
        "migrate",
        "lockbox",
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
    drop(migrated);
    fixture.success(&["doctor", "migrate", "all", "--replace"]);
    let opened = fixture
        .command(&[path(&source), "open"])
        .env_remove("LOCKBOX_KEY")
        .test_output()
        .unwrap();
    assert_success(&opened);
    let content = fixture
        .command(&[path(&source), "cat", "/contact.txt"])
        .env_remove("LOCKBOX_KEY")
        .test_output()
        .unwrap();
    assert_success(&content);
    assert_eq!(content.stdout, b"opened by a migrated profile key");
}

#[test]
fn automatic_historical_exporter_install_hides_cargo_output() {
    let fixture = Fixture::new("migration-v1-install-e2e");
    fixture.init_v1_vault();
    let exporter = build_historical_vault_exporter();
    let fake_cargo = build_fake_cargo(&fixture.root);
    let home = fixture.root.join("home");

    let output = fixture
        .command(&["doctor", "migrate", "vault", "--replace"])
        .env("CARGO", fake_cargo)
        .env("HOME", home)
        .env("FAKE_EXPORTER_SOURCE", exporter)
        .env("LOCKBOX_TEST_IGNORE_INSTALLED_EXPORTER", "1")
        .test_output()
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
        let root = temp.path().canonicalize().unwrap();
        Self {
            vault: root.join("vault"),
            agent: common::agent_socket_dir(&root.join("agent")),
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

    fn init_structure_v2_container_v1_vault(&self) {
        let password = StructureV2SecretString::try_from_slice(VAULT_PASSWORD.as_bytes()).unwrap();
        let vault = StructureV2VaultDirectory::replace(&self.vault, &password).unwrap();
        vault
            .store_private_key("default", &StructureV2ContactKeyPair::generate().unwrap())
            .unwrap();
        vault
            .store_profile_email("default", "mixed@example.test")
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
        self.command(args).test_output().unwrap()
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
        command.test_output_with_input(stdin.as_bytes()).unwrap()
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
#[test]
fn damaged_format_two_recovers_with_current_cli_and_original_credentials() {
    use revault_lockbox_api_structure_v2 as old;
    for contact_only in [false, true] {
        let fixture = Fixture::new("historical-recovery");
        fixture.init_current_vault();
        fixture.success(&["session", "auto-open", "disable", "--yes"]);
        let public_path = fixture.root.join("owner.pub");
        fixture.success(&[
            "vault",
            "profiles",
            "export",
            path(&public_path),
            "--format",
            "raw",
        ]);
        let public =
            revault_vault_api::import_public_key(&std::fs::read(&public_path).unwrap()).unwrap();
        // Current CLI cannot create old-format or damaged archives. Only this
        // fixture setup uses the pinned historical writer and page inspection;
        // migration, recovery, unlocking and byte assertions use public CLI.
        let password = old::SecretString::try_from_slice(LOCKBOX_PASSWORD.as_bytes()).unwrap();
        let protection = if contact_only {
            old::LockboxProtection::ContactPublicKey {
                name: Some("default".into()),
                contact: old::ContactPublicKey::from_bytes(&public.to_bytes()).unwrap(),
            }
        } else {
            old::LockboxProtection::Password(&password)
        };
        let mut archive = old::Lockbox::create_in_memory(
            protection,
            &old::OwnerSigningKeyPair::generate().unwrap(),
        )
        .unwrap();
        archive
            .add_file(
                &old::LockboxPath::new("/damaged.bin").unwrap(),
                &vec![42; 256 * 1024],
                false,
            )
            .unwrap();
        archive.commit().unwrap();
        let payload = b"intact historical content\0\xff";
        archive
            .add_file(
                &old::LockboxPath::new("/intact.bin").unwrap(),
                payload,
                false,
            )
            .unwrap();
        archive.commit().unwrap();
        let pages = archive.inspector().inspect_pages().unwrap();
        let page = pages
            .iter()
            .filter(|page| {
                page.objects.iter().any(|object| {
                    if contact_only {
                        object.kind == "toc-leaf"
                    } else {
                        object.kind == "file-data" || object.kind == "packed-file-data"
                    }
                })
            })
            .max_by_key(|page| {
                if contact_only {
                    page.sequence
                } else {
                    u64::MAX - page.sequence
                }
            })
            .unwrap();
        let mut damaged = archive.try_to_bytes().unwrap();
        damaged[page.offset as usize + if contact_only { 64 } else { 96 }] ^= 0xff;
        let source = fixture.root.join("damaged.lbox");
        let recovered = fixture.root.join("recovered.lbox");
        std::fs::write(&source, &damaged).unwrap();
        let run = |args: &[&str]| {
            fixture
                .command(args)
                .env_remove("LOCKBOX_KEY")
                .test_output()
                .unwrap()
        };
        let migrated = run(&["doctor", "migrate", "lockbox", path(&source), "--replace"]);
        assert_failure_contains(&migrated, "damaged source Lockbox");
        assert_failure_contains(&migrated, "doctor recover --dry-run");
        assert_failure_contains(&migrated, "doctor recover --output");
        assert_failure_contains(
            &run(&[path(&source), "doctor", "recover"]),
            "separate --output",
        );
        assert_failure_contains(
            &run(&[
                path(&source),
                "doctor",
                "recover",
                "--output",
                path(&source),
                "--overwrite",
            ]),
            "separate output",
        );
        if !contact_only {
            let wrong = fixture
                .command(&[
                    path(&source),
                    "doctor",
                    "recover",
                    "--output",
                    path(&recovered),
                ])
                .env_remove("LOCKBOX_KEY")
                .env("LOCKBOX_PASSWORD", "incorrect-password")
                .test_output()
                .unwrap();
            assert!(!wrong.status.success());
            assert!(!recovered.exists());
        }
        assert_success(&run(&[path(&source), "doctor", "recover", "--dry-run"]));
        assert!(!recovered.exists());
        assert_eq!(std::fs::read(&source).unwrap(), damaged);
        assert_success(&run(&[
            path(&source),
            "doctor",
            "recover",
            "--output",
            path(&recovered),
        ]));
        assert_success(&run(&[path(&recovered), "open"]));
        let content = run(&[path(&recovered), "cat", "/intact.bin"]);
        assert_success(&content);
        assert_eq!(content.stdout, payload);
        let saved = std::fs::read(&recovered).unwrap();
        assert!(!run(&[
            path(&source),
            "doctor",
            "recover",
            "--output",
            path(&recovered)
        ])
        .status
        .success());
        assert_eq!(std::fs::read(&recovered).unwrap(), saved);
        assert_eq!(std::fs::read(&source).unwrap(), damaged);
        assert_success(&run(&[
            "doctor",
            "migrate",
            "lockbox",
            path(&recovered),
            "--replace",
        ]));
    }
}

#[test]
fn recovery_refuses_future_formats_without_writing_output() {
    let fixture = Fixture::new("recovery-future");
    let source = fixture.root.join("future.lbox");
    let output = fixture.root.join("recovered.lbox");
    // No current CLI writer can produce a future-format header.
    let mut bytes = vec![0; 384];
    bytes[..8].copy_from_slice(b"LBX4HDR\0");
    bytes[8..10].copy_from_slice(&4_u16.to_le_bytes());
    std::fs::write(&source, &bytes).unwrap();
    let result = fixture.run(&[
        path(&source),
        "doctor",
        "recover",
        "--output",
        path(&output),
    ]);
    assert_failure_contains(&result, "newer format");
    assert!(!output.exists());
    assert_eq!(std::fs::read(&source).unwrap(), bytes);
}

#[test]
fn all_migration_reports_partial_failures_and_can_be_repeated() {
    let fixture = Fixture::new("all-migration");
    fixture.init_current_vault();
    let current = fixture.create_archive("current.lbox");
    fixture.success(&["session", "default", path(&current)]);
    let missing = fixture.create_archive("missing.lbox");
    for archive in [&current, &missing] {
        fixture.success(&["vault", "lockboxes", "remember", path(archive)]);
    }
    // No public CLI command removes an archive while deliberately retaining its
    // known-path record; host removal models an unavailable remembered file.
    std::fs::remove_file(&missing).unwrap();
    use revault_lockbox_api_structure_v2 as old;
    // Only fixture creation needs the historical API: current CLI writes v3.
    let mut archive = old::Lockbox::create_in_memory(
        old::LockboxProtection::ContentKey(
            old::SecretVec::try_from_slice(b"migration-test-content-key").unwrap(),
        ),
        &old::OwnerSigningKeyPair::generate().unwrap(),
    )
    .unwrap();
    archive
        .add_file(
            &old::LockboxPath::new("/payload").unwrap(),
            b"bulk historical payload",
            false,
        )
        .unwrap();
    archive.commit().unwrap();
    let original = archive.try_to_bytes().unwrap();
    let source = fixture.root.join("historical.lbox");
    std::fs::write(&source, &original).unwrap();
    fixture.success(&["vault", "lockboxes", "remember", path(&source)]);
    let current_bytes = std::fs::read(&current).unwrap();
    assert_failure_contains(&fixture.run(&["doctor", "migrate", "all"]), "--replace");
    let result = fixture.run(&["doctor", "migrate", "all", "--replace"]);
    assert_failure_contains(&result, "known Lockbox is missing");
    assert!(String::from_utf8_lossy(&result.stdout).contains("2 Lockboxes complete; 1 failed"));
    assert_eq!(
        fixture.success(&[path(&source), "cat", "/payload"]).stdout,
        b"bulk historical payload"
    );
    assert_eq!(
        std::fs::read(source.with_file_name("historical.lbox.v2-v3.pre-migration")).unwrap(),
        original
    );
    assert_eq!(std::fs::read(&current).unwrap(), current_bytes);
    assert!(!missing.exists());
    fixture.success(&["vault", "lockboxes", "forget", path(&missing)]);
    let repeated = fixture.success(&["doctor", "migrate", "all", "--replace"]);
    assert!(String::from_utf8_lossy(&repeated.stdout).contains("2 Lockboxes complete; 0 failed"));
    assert_eq!(
        fixture.success(&[path(&source), "cat", "/payload"]).stdout,
        b"bulk historical payload"
    );
}

#[test]
fn all_migration_recovers_interrupted_replacements_before_missing_path_checks() {
    use revault_migration::{ArtifactKind, MigrationJournal, MigrationStage};
    for kind in [ArtifactKind::Vault, ArtifactKind::Archive] {
        let fixture = Fixture::new("all-migration-resume");
        fixture.init_current_vault();
        let archive = fixture.create_archive("resumed.lbox");
        fixture.success(&["vault", "lockboxes", "remember", path(&archive)]);
        fixture.success(&[path(&archive), "variables", "set", "PAYLOAD", "retained"]);
        let source = if kind == ArtifactKind::Vault {
            fixture.vault.clone()
        } else {
            archive.clone()
        };
        let output = fixture.root.join("pending-output");
        let backup = source.with_file_name(format!(
            "{}.v2-v3.pre-migration",
            source.file_name().unwrap().to_str().unwrap()
        ));
        // No CLI can stop exactly between replacement renames. Construct only
        // that crash boundary and its authenticated journal; all content was
        // created above through CLI and is verified through CLI after recovery.
        if kind == ArtifactKind::Vault {
            std::fs::create_dir(&backup).unwrap();
            std::fs::copy(
                source.join("local-vault.lbox"),
                backup.join("local-vault.lbox"),
            )
            .unwrap();
        } else {
            std::fs::copy(&source, &backup).unwrap();
        }
        std::fs::rename(&source, &output).unwrap();
        let work = fixture.root.join(".revault-migration-interrupted");
        std::fs::create_dir(&work).unwrap();
        let journal = MigrationJournal {
            operation_id: [1; 16],
            artifact_kind: kind,
            source_path: source.clone(),
            source_format_version: 2,
            source_fingerprint: [2; 32],
            target_format_version: 3,
            current_stage: MigrationStage::Replace,
            temporary_paths: vec![output],
            exporter_version: None,
            artifact_key: SecretVec::try_from_slice(&[3; 32]).unwrap(),
        };
        journal
            .save(
                &work.join(if kind == ArtifactKind::Vault {
                    "vault.migration-state"
                } else {
                    "archive.migration-state"
                }),
                VAULT_PASSWORD.as_bytes(),
            )
            .unwrap();
        fixture.success(&["doctor", "migrate", "all", "--replace"]);
        assert_eq!(
            fixture
                .success(&[path(&archive), "variables", "get", "PAYLOAD"])
                .stdout,
            b"retained\n"
        );
        assert!(backup.exists());
        assert!(!work.exists());
    }
}

#[test]
fn all_migration_refuses_a_replaced_known_identity() {
    let fixture = Fixture::new("all-migration-changed-identity");
    fixture.init_current_vault();
    let target = fixture.create_archive("target.lbox");
    let other = fixture.create_archive("other.lbox");
    for archive in [&target, &other] {
        fixture.success(&["vault", "lockboxes", "remember", path(archive)]);
    }
    let replacement = std::fs::read(&other).unwrap();
    // The public CLI intentionally keeps records consistent. Model an external
    // replacement to verify bulk migration never trusts the remembered path alone.
    std::fs::write(&target, &replacement).unwrap();
    let result = fixture.run(&["doctor", "migrate", "all", "--replace"]);
    assert_failure_contains(&result, "identity");
    assert!(String::from_utf8_lossy(&result.stdout).contains("1 Lockboxes complete; 1 failed"));
    assert_eq!(std::fs::read(&target).unwrap(), replacement);
}

#[test]
fn all_migration_requires_existing_vault_and_migrates_legacy_vault_first() {
    let absent = Fixture::new("all-migration-no-vault");
    assert_failure_contains(
        &absent.run(&["doctor", "migrate", "all", "--replace"]),
        "existing local Vault",
    );
    assert!(!absent.vault.exists());
    let fixture = Fixture::new("all-migration-legacy-vault");
    fixture.init_structure_v2_container_v1_vault();
    build_historical_vault_exporter();
    fixture.success(&["doctor", "migrate", "all", "--replace"]);
    fixture.success(&["vault", "profiles", "list"]);
    fixture.success(&["doctor", "migrate", "all", "--replace"]);
}

#[test]
fn closed_format_two_archive_migrates_without_normal_open_or_auto_open() {
    use revault_lockbox_api_structure_v2 as old;
    for contact_only in [false, true] {
        let fixture = Fixture::new("closed-archive2-migration");
        fixture.init_current_vault();
        fixture.success(&["session", "auto-open", "disable", "--yes"]);
        let public = fixture.root.join("owner.pub");
        fixture.success(&[
            "vault",
            "profile",
            "export",
            path(&public),
            "--format",
            "raw",
        ]);
        let public = revault_vault_api::import_public_key(&std::fs::read(public).unwrap()).unwrap();
        // The current CLI cannot write format 2. The pinned historical API is
        // used only for fixture creation, with a publicly exported vault key.
        let password = old::SecretString::try_from_slice(LOCKBOX_PASSWORD.as_bytes()).unwrap();
        let protection = if contact_only {
            old::LockboxProtection::ContactPublicKey {
                name: Some("default".into()),
                contact: old::ContactPublicKey::from_bytes(&public.to_bytes()).unwrap(),
            }
        } else {
            old::LockboxProtection::Password(&password)
        };
        let mut archive = old::Lockbox::create_in_memory(
            protection,
            &old::OwnerSigningKeyPair::generate().unwrap(),
        )
        .unwrap();
        let payload = b"closed historical payload\0\xff";
        archive
            .add_file(&old::LockboxPath::new("/payload").unwrap(), payload, false)
            .unwrap();
        archive.commit().unwrap();
        let original = archive.try_to_bytes().unwrap();
        let source = fixture.root.join("closed.lbox");
        std::fs::write(&source, &original).unwrap();
        let run = |args: &[&str]| {
            fixture
                .command(args)
                .env_remove("LOCKBOX_KEY")
                .test_output()
                .unwrap()
        };
        assert_failure_contains(&run(&[path(&source), "open"]), "migrate");
        if !contact_only {
            let wrong = fixture
                .command(&["doctor", "migrate", "lockbox", path(&source), "--replace"])
                .env_remove("LOCKBOX_KEY")
                .env("LOCKBOX_PASSWORD", "wrong-password")
                .test_output()
                .unwrap();
            assert!(!wrong.status.success());
            assert_eq!(std::fs::read(&source).unwrap(), original);
        }
        assert_success(&run(&[
            "doctor",
            "migrate",
            "lockbox",
            path(&source),
            "--replace",
        ]));
        assert_success(&run(&[path(&source), "open"]));
        let result = run(&[path(&source), "cat", "/payload"]);
        assert_success(&result);
        assert_eq!(result.stdout, payload);
        assert_eq!(
            std::fs::read(source.with_file_name("closed.lbox.v2-v3.pre-migration")).unwrap(),
            original
        );
        assert_success(&run(&[
            "doctor",
            "migrate",
            "lockbox",
            path(&source),
            "--replace",
        ]));
    }
}

#[test]
fn older_archive_is_refused_normally_then_explicitly_migrated() {
    let fixture = Fixture::new("archive2-explicit-migration");
    fixture.init_current_vault();
    // The current CLI deliberately has no older-format writer. Only historical
    // setup uses the pinned released API; all operations and content assertions
    // after setup use separate public CLI invocations.
    use revault_lockbox_api_structure_v2 as old;
    let signer = old::OwnerSigningKeyPair::generate().unwrap();
    let mut archive = old::Lockbox::create_in_memory(
        old::LockboxProtection::ContentKey(
            old::SecretVec::try_from_slice(b"migration-test-content-key").unwrap(),
        ),
        &signer,
    )
    .unwrap();
    let payload = b"older archive payload\0with binary bytes\xff".repeat(4097);
    archive
        .add_file(&old::LockboxPath::new("/payload").unwrap(), &payload, false)
        .unwrap();
    archive
        .set_variable(&old::VariableName::new("VALUE").unwrap(), "older-value")
        .unwrap();
    archive.commit().unwrap();
    let original = archive.try_to_bytes().unwrap();
    assert_eq!(
        revault_lockbox_api::probe_lockbox_format_version(&original).unwrap(),
        2
    );
    let source = fixture.root.join("older.lbox");
    let output = fixture.root.join("current.lbox");
    std::fs::write(&source, &original).unwrap();
    let source_arg = source.to_str().unwrap();
    for args in [
        vec![source_arg, "cat", "/payload"],
        vec![source_arg, "variable", "set", "VALUE", "must-not-write"],
    ] {
        let result = fixture.run(&args);
        assert!(!result.status.success(), "{args:?}: {result:?}");
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(
            error.contains('2') && error.contains("migrate"),
            "{result:?}"
        );
        // No CLI byte-snapshot operation exists; opaque bytes solely prove
        // refusal has not changed the historical source.
        assert_eq!(std::fs::read(&source).unwrap(), original);
    }
    let migrated = fixture.run(&[
        source_arg,
        "doctor",
        "migrate",
        "lockbox",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert_success(&migrated);
    let bytes = fixture.run(&[output.to_str().unwrap(), "cat", "/payload"]);
    assert_success(&bytes);
    assert_eq!(bytes.stdout, payload);
    let value = fixture.run(&[output.to_str().unwrap(), "variable", "get", "VALUE"]);
    assert_success(&value);
    assert_eq!(value.stdout, b"older-value\n");
    assert_eq!(std::fs::read(&source).unwrap(), original);
}

#[test]
fn doctor_shows_local_credential_names_only_after_unlocking() {
    let fixture = Fixture::new("doctor-credential-names");
    fixture.init_current_vault();
    fixture.success(&["vault", "profile", "create", "doctor-owner"]);
    let archive = fixture.create_archive("named.lbox");
    fixture.success(&[path(&archive), "access", "grant", "doctor-owner"]);
    let listed = fixture.success(&[path(&archive), "access", "list"]);
    assert!(String::from_utf8_lossy(&listed.stdout).contains("doctor-owner"));
    let before = std::fs::read(&archive).unwrap();
    let doctor = fixture.success(&[path(&archive), "doctor"]);
    let report = String::from_utf8_lossy(&doctor.stdout);
    let (public, encrypted) = report.split_once("Encrypted content\n").unwrap();
    assert!(!public.contains("doctor-owner"), "{report}");
    assert!(!public.contains("  slots:\n"), "{report}");
    assert!(
        encrypted.contains("credential names (local vault):\n    doctor-owner\n"),
        "{report}"
    );
    assert!(!report.contains(LOCKBOX_PASSWORD), "{report}");
    assert!(!report.contains(VAULT_PASSWORD), "{report}");
    assert_eq!(std::fs::read(&archive).unwrap(), before);
}

#[test]
fn doctor_reports_vault_versions_and_future_vault_errors_identify_the_path() {
    let fixture = Fixture::new("doctor-vault-format-diagnostics");
    fixture.init_current_vault();
    let archive = fixture.create_archive("doctor.lbox");
    let container = fixture.vault.join("local-vault.lbox");
    let original = std::fs::read(&container).unwrap();
    let doctor = fixture.run(&["doctor"]);
    assert_success(&doctor);
    let report = String::from_utf8_lossy(&doctor.stdout);
    assert!(
        report.contains("Local vault\n  container format version: 3\n"),
        "{report}"
    );
    assert!(report.contains("structure version: 3"), "{report}");
    assert_eq!(std::fs::read(&container).unwrap(), original);

    let closed = fixture
        .command(&[path(&archive), "doctor"])
        .env_remove("LOCKBOX_VAULT_PASSWORD")
        .test_output()
        .unwrap();
    assert_success(&closed);
    let report = String::from_utf8_lossy(&closed.stdout);
    assert!(
        report.starts_with("Lockbox\n  format version: 3\n"),
        "{report}"
    );
    assert!(
        report.contains("Local vault\n  container format version: 3\n"),
        "{report}"
    );
    assert!(
        report.contains("structure version: not read (vault is closed or absent)"),
        "{report}"
    );
    assert!(report.contains(path(&container)), "{report}");
    assert!(!report.contains("credential names"), "{report}");
    assert_eq!(std::fs::read(&container).unwrap(), original);

    // The current CLI cannot create a future container. Change only opaque
    // test header bytes to exercise refusal, then verify them unchanged.
    for (version, slot) in [(4_u16, 0_usize), (5, 192)] {
        let mut future = original.clone();
        future[..384].fill(0);
        future[slot..slot + 8].copy_from_slice(b"LBX4HDR\0");
        future[slot + 8..slot + 10].copy_from_slice(&version.to_le_bytes());
        std::fs::write(&container, &future).unwrap();
        let aliases = fixture.run(&["vault", "lockbox", "alias", "list"]);
        assert_failure_contains(&aliases, path(&container));
        assert_failure_contains(
            &aliases,
            &format!("Found Lockbox container version {version}"),
        );
        assert_failure_contains(&aliases, "Automatic downgrade is not supported");
        let doctor = fixture.run(&["doctor"]);
        assert_success(&doctor);
        let report = String::from_utf8_lossy(&doctor.stdout);
        assert!(
            report.contains(&format!("container format version: {version}")),
            "{report}"
        );
        assert!(
            report.contains("status: unsupported container; upgrade reVault"),
            "{report}"
        );
        assert!(
            report.contains("structure version: not read (unsupported container)"),
            "{report}"
        );
        assert!(
            report.contains(path(&container))
                && report.contains("Automatic downgrade is not supported"),
            "{report}"
        );
        assert!(!report.contains("migrate lockbox"), "{report}");
        let selected = fixture.success(&[path(&archive), "doctor"]);
        let selected = String::from_utf8_lossy(&selected.stdout);
        assert!(
            selected.starts_with("Lockbox\n  format version: 3\n"),
            "{selected}"
        );
        assert!(
            selected.contains(&format!(
                "Local vault\n  container format version: {version}\n"
            )),
            "{selected}"
        );
        assert_eq!(
            selected.matches("status: unsupported container").count(),
            1,
            "{selected}"
        );
        assert!(selected.contains(path(&container)), "{selected}");
        assert!(!selected.contains("key-directory backup:"), "{selected}");
        assert!(!selected.contains("credential names"), "{selected}");
        assert_eq!(std::fs::read(&container).unwrap(), future);
    }
}
