mod common;

#[allow(dead_code)]
#[path = "../src/commands/migration_exporters.rs"]
mod migration_exporters;

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

// No public writer emits future formats. Keep synthetic refusal fixtures
// checksummed so a damaged current header cannot masquerade as a future one.
fn checksum_future_header(slot: &mut [u8]) {
    use sha2::{Digest, Sha256};
    let mut checksum = Sha256::new();
    checksum.update(b"lockbox-v1-public-checksum/sha256");
    checksum.update(160_u64.to_le_bytes());
    checksum.update(&slot[..160]);
    slot[160..192].copy_from_slice(&checksum.finalize());
}

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
    let payload = b"ordinary migration persisted content";
    let input = fixture.root.join("input.txt");
    std::fs::write(&input, payload).unwrap();
    fixture.success(&[
        path(&advanced_source),
        "add",
        path(&input),
        "--to",
        "/content.txt",
    ]);

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
    assert_eq!(
        fixture
            .success(&[path(&imported), "cat", "/content.txt"])
            .stdout,
        payload
    );

    let direct_source = fixture.create_archive("direct.lbox");
    fixture.success(&[
        path(&direct_source),
        "add",
        path(&input),
        "--to",
        "/content.txt",
    ]);
    let missing_exporter = fixture.root.join("missing-exporter");
    let refused_output = fixture.root.join("refused-migration.lbox");
    let source_before = std::fs::read(&direct_source).unwrap();
    let refused = fixture.run(&[
        "doctor",
        "migrate",
        "lockbox",
        path(&direct_source),
        "--output",
        path(&refused_output),
        "--exporter",
        path(&missing_exporter),
    ]);
    assert!(
        !refused.status.success(),
        "an explicit missing exporter must not be ignored"
    );
    assert!(!refused_output.exists());
    assert_eq!(std::fs::read(&direct_source).unwrap(), source_before);
    let direct_output = fixture.root.join("direct-migrated.lbox");
    fixture.success(&[
        "doctor",
        "migrate",
        "lockbox",
        path(&direct_source),
        "--output",
        path(&direct_output),
    ]);
    assert_eq!(
        fixture
            .success(&[path(&direct_output), "cat", "/content.txt"])
            .stdout,
        payload
    );

    let replace_source = fixture.create_archive("replace.lbox");
    fixture.success(&[
        path(&replace_source),
        "add",
        path(&input),
        "--to",
        "/content.txt",
    ]);
    let already_current = fixture.run(&[
        "doctor",
        "migrate",
        "lockbox",
        path(&replace_source),
        "--replace",
    ]);
    assert_success(&already_current);
    assert!(String::from_utf8_lossy(&already_current.stdout).contains("No migration needed"));
    assert_eq!(
        fixture
            .success(&[path(&replace_source), "cat", "/content.txt"])
            .stdout,
        payload
    );
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
    assert!(
        report.contains("structure version: not read (unsupported container)"),
        "{report}"
    );
    assert!(
        report.contains("status: local vault upgrade required; migrate the Vault and all known Lockboxes: lbx doctor migrate all --replace"),
        "{report}"
    );
    assert!(
        report.contains(path(&container)) && report.contains("migrate vault"),
        "{report}"
    );
    assert!(!report.contains("migrate lockbox"), "{report}");
    assert_eq!(std::fs::read(&container).unwrap(), before_bytes);
    let exporter = build_selected_exporter("revault_migrate_v2", "revault-migrate-vault-v2");
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
    // Migration selects this Vault's default signer, so its rightful owner
    // may still write after opening through a different contact profile.
    let owner_write = fixture
        .command(&[path(&source), "variable", "set", "OWNER_WRITE", "allowed"])
        .env_remove("LOCKBOX_KEY")
        .test_output()
        .unwrap();
    assert_success(&owner_write);
    let backup = fixture.root.join("recipient-profile.backup");
    fixture.success(&[
        "vault",
        "profile",
        "backup",
        path(&backup),
        "--name",
        "legacy",
    ]);
    fixture.success(&["session", "stop"]);
    // A separate Vault receives only the contact profile, not the migration
    // owner's default signing key. Public backup/restore models a recipient.
    let recipient = Fixture::new("migration-contact-recipient");
    recipient.init_current_vault();
    recipient.success(&["vault", "profile", "restore", path(&backup)]);
    let opened = recipient
        .command(&[path(&source), "open"])
        .env_remove("LOCKBOX_KEY")
        .test_output()
        .unwrap();
    assert_success(&opened);
    let read = recipient
        .command(&[path(&source), "cat", "/contact.txt"])
        .env_remove("LOCKBOX_KEY")
        .test_output()
        .unwrap();
    assert_success(&read);
    assert_eq!(read.stdout, b"opened by a migrated profile key");
    let before_write = std::fs::read(&source).unwrap();
    let denied = recipient
        .command(&[
            path(&source),
            "variable",
            "set",
            "RECIPIENT_WRITE",
            "denied",
        ])
        .env_remove("LOCKBOX_KEY")
        .test_output()
        .unwrap();
    assert_failure_contains(&denied, "owner");
    assert_eq!(std::fs::read(&source).unwrap(), before_write);
}

#[test]
fn automatic_historical_exporter_install_hides_cargo_output() {
    let fixture = Fixture::new("migration-v1-install-e2e");
    fixture.init_v1_vault();
    let exporter = build_historical_vault_exporter();
    let fake_cargo = build_fake_cargo(&fixture.root);
    let home = fixture.root.join("installer-home");

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
    _agent_guard: common::TestAgentGuard,
    _temp: TestTempDir,
    root: PathBuf,
    vault: PathBuf,
    agent: PathBuf,
}

impl Fixture {
    fn new(prefix: &str) -> Self {
        let temp = TestTempDir::new(prefix);
        let root = temp.path().canonicalize().unwrap();
        prepare_exporter_cache(&root);
        let vault = root.join("vault");
        let agent = common::agent_socket_dir(&root.join("agent"));
        let agent_guard = common::TestAgentGuard::new(&agent, Some(&vault));
        Self {
            _agent_guard: agent_guard,
            vault,
            agent,
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

    fn historical_v2_signer(&self) -> revault_lockbox_api_structure_v2::OwnerSigningKeyPair {
        // Export through the public CLI, then use the frozen writer only to
        // create an old-format fixture owned by this same Vault profile.
        let backup = self.root.join("historical-owner-backup.txt");
        self.success(&["vault", "profile", "backup", path(&backup), "--overwrite"]);
        let text = zeroize::Zeroizing::new(std::fs::read_to_string(backup).unwrap());
        let encoded = zeroize::Zeroizing::new(
            text.split_once("Owner signing private key record (hex):")
                .unwrap()
                .1
                .split_whitespace()
                .collect::<String>(),
        );
        revault_lockbox_api_structure_v2::OwnerSigningKeyPair::from_private_key_record(
            revault_lockbox_api_structure_v2::SecretVec::try_from_vec(
                revault_vault_api::decode_hex(&encoded).unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    }

    #[track_caller]
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
            .env("HOME", self.root.join("home"))
            .env("LOCKBOX_TEST_IGNORE_INSTALLED_EXPORTER", "1")
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

fn exporter_binaries() -> &'static std::collections::BTreeMap<String, PathBuf> {
    static BINARIES: std::sync::OnceLock<std::collections::BTreeMap<String, PathBuf>> =
        std::sync::OnceLock::new();
    BINARIES.get_or_init(|| {
        if let Some(root) = std::env::var_os("REVAULT_TEST_INSTALLED_EXPORTERS") {
            return (1..=3)
                .flat_map(|v| {
                    ["archive", "vault"].map(move |kind| format!("revault-migrate-{kind}-v{v}"))
                })
                .map(|name| {
                    let path = PathBuf::from(&root)
                        .join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
                    assert!(
                        path.is_file(),
                        "installed exporter missing: {}",
                        path.display()
                    );
                    (name, path)
                })
                .collect();
        }
        let output = Command::new(env!("CARGO"))
            .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap())
            .args([
                "build",
                "--locked",
                "--offline",
                "--bins",
                "-p",
                "revault_migrate_v1",
                "-p",
                "revault_migrate_v2",
                "-p",
                "revault_migrate_v3",
                "--message-format=json",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "exporter build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let binaries: std::collections::BTreeMap<String, PathBuf> =
            String::from_utf8(output.stdout)
                .unwrap()
                .lines()
                .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                .filter(|value| value["reason"] == "compiler-artifact")
                .filter_map(|value| {
                    Some((
                        value["target"]["name"].as_str()?.to_owned(),
                        PathBuf::from(value["executable"].as_str()?),
                    ))
                })
                .filter(|(name, _)| name.starts_with("revault-migrate-"))
                .collect();
        assert_eq!(binaries.len(), 6);
        binaries
    })
}

fn build_selected_exporter(_package: &str, binary: &str) -> PathBuf {
    exporter_binaries()[binary].clone()
}

fn build_historical_vault_exporter() -> PathBuf {
    build_selected_exporter("revault_migrate_v1", "revault-migrate-vault-v1")
}

fn build_historical_archive_exporter() -> PathBuf {
    build_selected_exporter("revault_migrate_v1", "revault-migrate-archive-v1")
}

fn prepare_exporter_cache(root: &Path) {
    // Isolate installation state from the developer's home and registry. This
    // is executable setup, not archive/Vault test-state manipulation. Tests
    // still use the CLI's normal validated-cache lookup for automatic selection.
    for (name, source) in exporter_binaries() {
        let release = migration_exporters::EXPORTERS
            .iter()
            .find(|release| release.binary == name)
            .unwrap();
        let bin = root
            .join("home/.cache/revault/exporters")
            .join(format!("{}-{}/bin", release.package, release.version));
        std::fs::create_dir_all(&bin).unwrap();
        let destination = bin.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
        std::fs::hard_link(source, &destination)
            .or_else(|_| std::fs::copy(source, &destination).map(|_| ()))
            .unwrap();
    }
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
    let verification = TestTempDir::new("migration-public-verification");
    let agent = common::agent_socket_dir(&verification.path().join("agent"));
    let _agent_guard = common::TestAgentGuard::new(&agent, None);
    let run = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_lockbox"))
            .args(args)
            .env("LOCKBOX_VAULT_DIR", root)
            .env("LOCKBOX_VAULT_PASSWORD", VAULT_PASSWORD)
            .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
            .env("LOCKBOX_SESSION_AGENT_DIR", &agent)
            .test_output()
            .unwrap();
        assert_success(&output);
        output
    };
    let before = std::fs::read(root.join("local-vault.lbox")).unwrap();
    let current = run(&["doctor", "migrate", "vault", "--replace"]);
    assert!(String::from_utf8_lossy(&current.stdout).contains(&format!(
        "Vault already uses container format {} and structure version {}. No migration needed.",
        revault_lockbox_api::LOCKBOX_FORMAT_VERSION,
        revault_vault_api::CURRENT_VAULT_STRUCTURE_VERSION
    )));
    assert_eq!(
        std::fs::read(root.join("local-vault.lbox")).unwrap(),
        before
    );
    let listed = run(&["vault", "profile", "list", "--format", "json"]);
    assert!(String::from_utf8_lossy(&listed.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .any(|value| value["name"] == "default"));
    let first = verification.path().join("first.pub");
    let second = verification.path().join("second.pub");
    run(&[
        "vault",
        "profile",
        "export",
        path(&first),
        "--format",
        "raw",
    ]);
    run(&[
        "vault",
        "profile",
        "export",
        path(&second),
        "--format",
        "raw",
    ]);
    let public = std::fs::read(first).unwrap();
    assert!(!public.is_empty());
    assert_eq!(std::fs::read(second).unwrap(), public);
}

#[track_caller]
fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed: status={}, stdout_bytes={}, stderr={}",
        output.status,
        output.stdout.len(),
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
    assert!(
        rendered.contains(expected),
        "expected diagnostic {expected:?} absent; status={}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
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
        let mut archive =
            old::Lockbox::create_in_memory(protection, &fixture.historical_v2_signer()).unwrap();
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
        assert_failure_contains(&migrated, "incomplete fragments separately");
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
            // A password recipient has opening credentials but none of the
            // source owner's keys. Scanning must work without signing authority;
            // publishing a recovered copy must still preserve the owner.
            let recipient = Fixture::new("historical-recovery-recipient");
            recipient.init_current_vault();
            let scan = recipient
                .command(&[path(&source), "doctor", "recover", "--dry-run"])
                .env_remove("LOCKBOX_KEY")
                .test_output()
                .unwrap();
            assert_success(&scan);
            let unauthorized = recipient
                .command(&[
                    path(&source),
                    "doctor",
                    "recover",
                    "--output",
                    path(&recovered),
                ])
                .env_remove("LOCKBOX_KEY")
                .test_output()
                .unwrap();
            assert_failure_contains(
                &unauthorized,
                "established archive owner's signing key is unavailable",
            );
            assert!(!recovered.exists());
            assert_eq!(std::fs::read(&source).unwrap(), damaged);
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
        let dry_run = run(&[path(&source), "doctor", "recover", "--dry-run"]);
        assert_success(&dry_run);
        if !contact_only {
            assert_success(&dry_run);
            let rendered = String::from_utf8_lossy(&dry_run.stdout);
            assert!(
                rendered.contains("Planned partial-file directory:"),
                "{rendered}"
            );
            assert!(rendered.contains("/damaged.bin"), "{rendered}");
            assert!(rendered.contains("reason"), "{rendered}");
            let json = run(&[
                path(&source),
                "doctor",
                "recover",
                "--dry-run",
                "--format",
                "json",
            ]);
            assert_success(&json);
            let rows: Vec<serde_json::Value> = String::from_utf8_lossy(&json.stdout)
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            let destination = rows
                .iter()
                .find(|row| row["field"] == "recovery_directory")
                .unwrap();
            let directory = std::path::PathBuf::from(destination["value"].as_str().unwrap());
            assert!(directory.is_absolute());
            assert!(!directory.exists());
            let preview = run(&[
                path(&source),
                "doctor",
                "recover",
                "--directory",
                path(&directory),
                "--dry-run",
                "--format",
                "json",
            ]);
            assert_success(&preview);
            assert!(!directory.exists());
            let exported = run(&[path(&source), "doctor", "recover", "--format", "json"]);
            assert_success(&exported);
            let manifest: serde_json::Value =
                serde_json::from_slice(&std::fs::read(directory.join("manifest.json")).unwrap())
                    .unwrap();
            let files = manifest["files"].as_array().unwrap();
            assert!(!files.iter().any(|file| file["path"] == "/intact.bin"));
            let default_archive = source.with_extension("recovered.lbox");
            assert_success(&run(&[path(&default_archive), "open"]));
            let intact = run(&[path(&default_archive), "cat", "/intact.bin"]);
            assert_success(&intact);
            assert_eq!(intact.stdout, payload);
            assert!(!run(&[path(&default_archive), "cat", "/damaged.bin"])
                .status
                .success());
            let missing = files
                .iter()
                .find(|file| file["path"] == "/damaged.bin")
                .unwrap();
            assert_eq!(missing["status"], "unrecoverable");
            assert!(missing.get("output").is_none());
            assert!(!run(&[
                path(&source),
                "doctor",
                "recover",
                "--directory",
                path(&directory)
            ])
            .status
            .success());
            let repeated_archive = fixture.root.join("another-recovered.lbox");
            let repeated = run(&[
                path(&source),
                "doctor",
                "recover",
                "--output",
                path(&repeated_archive),
                "--format",
                "tsv",
            ]);
            assert_success(&repeated);
            assert!(String::from_utf8_lossy(&repeated.stdout)
                .contains("damaged.lbox.recovered-files.1"));
            assert!(fixture
                .root
                .join("damaged.lbox.recovered-files.1/manifest.json")
                .exists());
        }
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
    bytes[..8].copy_from_slice(b"LBX5HDR\0");
    bytes[8..10].copy_from_slice(&6_u16.to_le_bytes());
    checksum_future_header(&mut bytes[..192]);
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
fn recovery_restores_verified_alternatives_to_archive_and_only_partial_ranges_to_sidecar() {
    for partial in [false, true] {
        let fixture = Fixture::new("verified-recovery");
        fixture.init_current_vault();
        fixture.success(&["session", "auto-open", "disable", "--yes"]);
        let signer = fixture.historical_v2_signer();
        let record = signer.private_key_record().unwrap();
        // No public CLI can write an authenticated inconsistent historical
        // index. This narrow test-support fixture creates that failure only.
        let record = record
            .with_bytes(|record| zeroize::Zeroizing::new(record.to_vec()))
            .unwrap();
        let source_bytes = revault_migrate_v2::stale_reference_recovery_fixture(
            b"migration-test-content-key",
            &record,
            partial,
            None,
        )
        .unwrap();
        let source = fixture.root.join("source.lbox");
        std::fs::write(&source, &source_bytes).unwrap();
        let run = |args: &[&str]| fixture.command(args).test_output().unwrap();
        let preview = run(&[
            path(&source),
            "doctor",
            "recover",
            "--dry-run",
            "--format",
            "json",
        ]);
        assert_success(&preview);
        let recovered = source.with_extension("recovered.lbox");
        let sidecar = fixture.root.join("source.lbox.recovered-files");
        assert!(!recovered.exists() && !sidecar.exists());
        let result = run(&[path(&source), "doctor", "recover", "--format", "json"]);
        assert_success(&result);
        let rows: Vec<serde_json::Value> = String::from_utf8_lossy(&result.stdout)
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        for (field, expected) in [
            ("recovered_files", if partial { "0" } else { "1" }),
            ("partial_files", if partial { "1" } else { "0" }),
            ("not_recovered_files", "0"),
        ] {
            assert!(
                rows.iter()
                    .any(|row| row["field"] == field && row["value"] == expected),
                "{rows:?}"
            );
        }
        assert_eq!(std::fs::read(&source).unwrap(), source_bytes);
        let metadata = run(&[path(&recovered), "variable", "get", "KEPT"]);
        assert_success(&metadata);
        assert_eq!(metadata.stdout, b"preserved metadata\n");
        let content = run(&[path(&recovered), "cat", "/restored.bin"]);
        if partial {
            assert!(!content.status.success());
            let manifest: serde_json::Value =
                serde_json::from_slice(&std::fs::read(sidecar.join("manifest.json")).unwrap())
                    .unwrap();
            let file = &manifest["files"][0];
            assert_eq!(file["status"], "partial");
            assert_eq!(file["permissions"], 0o640);
            assert_eq!(file["missing_ranges"][0]["offset"], 2 * 1024 * 1024);
            let range = &file["ranges"][0];
            assert_eq!(range["offset"], 0);
            assert_eq!(
                range["output"],
                "files/restored.bin.part-offset-00000000000000000000"
            );
            assert_eq!(
                std::fs::read(sidecar.join(range["output"].as_str().unwrap())).unwrap(),
                vec![42; 2 * 1024 * 1024]
            );
            let second = fixture.root.join("second.lbox");
            assert!(!run(&[
                path(&source),
                "doctor",
                "recover",
                "--output",
                path(&second),
                "--directory",
                path(&sidecar)
            ])
            .status
            .success());
            assert!(!second.exists());
            let chosen = fixture.root.join("chosen-partials");
            assert_success(&run(&[
                path(&source),
                "doctor",
                "recover",
                "--output",
                path(&second),
                "--directory",
                path(&chosen),
            ]));
            assert!(chosen.join("manifest.json").exists());
        } else {
            assert_success(&content);
            assert_eq!(content.stdout, vec![42; 3 * 1024 * 1024]);
            assert!(!sidecar.exists());
        }
    }
}

#[test]
fn historical_recovery_restores_root_and_nested_mirrors_without_disabling_protection() {
    for root in ["/", "/managed"] {
        let fixture = Fixture::new("mirror-recovery");
        fixture.init_current_vault();
        let signer = fixture.historical_v2_signer();
        let record = signer
            .private_key_record()
            .unwrap()
            .with_bytes(|bytes| zeroize::Zeroizing::new(bytes.to_vec()))
            .unwrap();
        // The current CLI cannot create an authenticated stale historical
        // index; only fixture construction uses the frozen test-support writer.
        let bytes = revault_migrate_v2::stale_reference_recovery_fixture(
            b"migration-test-content-key",
            &record,
            false,
            Some(root),
        )
        .unwrap();
        let source = fixture.root.join("mirror.lbox");
        std::fs::write(&source, &bytes).unwrap();
        let diagnostic = fixture.root.join("diagnostic.json");
        let result = fixture.run(&[
            path(&source),
            "doctor",
            "recover",
            "--diagnostics",
            path(&diagnostic),
            "--quiet",
        ]);
        assert_success(&result);
        assert!(
            result.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let diagnostic_text = std::fs::read_to_string(&diagnostic).unwrap();
        let diagnostic: serde_json::Value = serde_json::from_str(&diagnostic_text).unwrap();
        assert_eq!(diagnostic["status"], "complete");
        assert!(diagnostic["helper_version"].is_string());
        assert!(!diagnostic_text.contains(".home.lbx.lock"));
        assert!(!diagnostic_text.contains("restored.bin"));
        assert!(!diagnostic_text.contains("migration-test-content-key"));
        let recovered = source.with_extension("recovered.lbox");
        let stored_path = format!("{}/restored.bin", root.trim_end_matches('/'));
        let content = fixture.run(&[path(&recovered), "cat", &stored_path]);
        assert_success(&content);
        assert_eq!(content.stdout, vec![42; 3 * 1024 * 1024]);
        let lock_path = format!("{}/.home.lbx.lock", root.trim_end_matches('/'));
        let lock_content = fixture.run(&[path(&recovered), "cat", &lock_path]);
        assert_success(&lock_content);
        assert!(lock_content.stdout.is_empty());
        let mirrors = fixture.run(&[path(&recovered), "mirror", "projects"]);
        assert_success(&mirrors);
        assert!(String::from_utf8_lossy(&mirrors.stdout).contains("home"));
        let input = fixture.root.join("forbidden.txt");
        std::fs::write(&input, b"ordinary mutation").unwrap();
        let destination = format!("{}/forbidden.txt", root.trim_end_matches('/'));
        let refused = fixture.run(&[path(&recovered), "add", path(&input), "--to", &destination]);
        assert_failure_contains(&refused, "managed by mirror");
        let retained = fixture.run(&[path(&recovered), "cat", &stored_path]);
        assert_success(&retained);
        assert_eq!(retained.stdout, content.stdout);
        assert_eq!(std::fs::read(&source).unwrap(), bytes);
        assert!(!fixture.root.join("mirror.lbox.recovered-files").exists());
    }
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
    // Only fixture creation needs the historical API: current CLI writes v5.
    let mut archive = old::Lockbox::create_in_memory(
        old::LockboxProtection::ContentKey(
            old::SecretVec::try_from_slice(b"migration-test-content-key").unwrap(),
        ),
        &fixture.historical_v2_signer(),
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
    // Current CLI cannot inspect an old-format header to remember its identity.
    // Seed the record that a migrated historical Vault would already contain.
    let password = SecretString::try_from_slice(VAULT_PASSWORD.as_bytes()).unwrap();
    let vault = VaultDirectory::open_or_create(&fixture.vault, &password).unwrap();
    vault
        .remember_known_lockbox(
            revault_lockbox_api::LockboxId::from_bytes(*archive.lockbox_id().as_bytes()),
            &source,
        )
        .unwrap();
    drop(vault);
    let current_bytes = std::fs::read(&current).unwrap();
    assert_failure_contains(&fixture.run(&["doctor", "migrate", "all"]), "--replace");
    let result = fixture.run(&["doctor", "migrate", "all", "--replace"]);
    assert_failure_contains(&result, "known Lockbox is missing");
    assert!(
        String::from_utf8_lossy(&result.stdout).contains("2 Lockboxes complete; 1 failed"),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        fixture.success(&[path(&source), "cat", "/payload"]).stdout,
        b"bulk historical payload"
    );
    assert_eq!(
        std::fs::read(source.with_file_name(format!(
            "historical.lbox.v2-v{LOCKBOX_FORMAT_VERSION}.pre-migration"
        )))
        .unwrap(),
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
        let target_version = if kind == ArtifactKind::Vault {
            revault_vault_api::CURRENT_VAULT_STRUCTURE_VERSION
        } else {
            u32::from(LOCKBOX_FORMAT_VERSION)
        };
        let backup = source.with_file_name(format!(
            "{}.v2-v{}.pre-migration",
            source.file_name().unwrap().to_str().unwrap(),
            target_version
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
            target_format_version: target_version,
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
        let mut archive =
            old::Lockbox::create_in_memory(protection, &fixture.historical_v2_signer()).unwrap();
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
            std::fs::read(source.with_file_name(format!(
                "closed.lbox.v2-v{LOCKBOX_FORMAT_VERSION}.pre-migration"
            )))
            .unwrap(),
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
    let signer = fixture.historical_v2_signer();
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
    // The current CLI cannot create an interrupted format-3 migration. Inject
    // only that encrypted historical state; migration and persisted reads below
    // exercise the public CLI. Opaque snapshots verify preservation.
    use revault_migration::{ArtifactKind, MigrationJournal, MigrationStage};
    use sha2::{Digest, Sha256};
    let fingerprint: [u8; 32] = Sha256::digest(&original).into();
    let mut digest = Sha256::new();
    digest.update(b"archive");
    digest.update(source.as_os_str().as_encoded_bytes());
    digest.update([0]);
    digest.update(output.as_os_str().as_encoded_bytes());
    digest.update(fingerprint);
    let mut operation_id = [0; 16];
    operation_id.copy_from_slice(&digest.finalize()[..16]);
    let id: String = operation_id
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let work = fixture.root.join(format!(".revault-migration-{id}"));
    std::fs::create_dir(&work).unwrap();
    let artifact = work.join("archive.migration");
    std::fs::write(&artifact, b"old encrypted migration artifact").unwrap();
    std::fs::write(&output, b"old incomplete output").unwrap();
    let journal_path = work.join("archive.migration-state");
    let journal = MigrationJournal {
        operation_id,
        artifact_kind: ArtifactKind::Archive,
        source_path: source.clone(),
        source_format_version: 2,
        source_fingerprint: fingerprint,
        target_format_version: 3,
        current_stage: MigrationStage::Import,
        temporary_paths: vec![
            artifact,
            work.join("archive.latest.migration"),
            output.clone(),
        ],
        exporter_version: None,
        artifact_key: SecretVec::try_from_slice(&[42; 32]).unwrap(),
    };
    journal
        .save(&journal_path, VAULT_PASSWORD.as_bytes())
        .unwrap();
    let journal_bytes = std::fs::read(&journal_path).unwrap();
    let migrated = fixture.run(&[
        source_arg,
        "doctor",
        "migrate",
        "lockbox",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert_success(&migrated);
    assert!(
        String::from_utf8_lossy(&migrated.stdout).contains("restarting from the unchanged source")
    );
    let preserved = std::fs::read_dir(&fixture.root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(".revault-preserved-migration-")
        })
        .unwrap();
    assert_eq!(
        std::fs::read(preserved.join("archive.migration-state")).unwrap(),
        journal_bytes
    );
    assert_eq!(
        std::fs::read(preserved.join("archive.migration")).unwrap(),
        b"old encrypted migration artifact"
    );
    assert_eq!(
        std::fs::read(preserved.join("previous-output")).unwrap(),
        b"old incomplete output"
    );
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
        report.contains("Local vault\n  container format version: 5\n"),
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
        report.starts_with("Lockbox\n  format version: 5\n"),
        "{report}"
    );
    assert!(
        report.contains("Local vault\n  container format version: 5\n"),
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
    for (version, slot) in [(6_u16, 0_usize), (7, 192)] {
        let mut future = original.clone();
        future[..384].fill(0);
        future[slot..slot + 8].copy_from_slice(b"LBX5HDR\0");
        future[slot + 8..slot + 10].copy_from_slice(&version.to_le_bytes());
        checksum_future_header(&mut future[slot..slot + 192]);
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
            selected.starts_with("Lockbox\n  format version: 5\n"),
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

#[test]
fn vault_v3_container_upgrade_preserves_profiles_and_enables_compaction() {
    let fixture = Fixture::new("migration-v3-container-v4");
    // Exception: only the frozen reader can create a historical container.
    // All migration, compaction, and persisted verification use the public CLI.
    let password = vault_v3::SecretString::try_from_slice(VAULT_PASSWORD.as_bytes()).unwrap();
    let vault = vault_v3::VaultDirectory::replace(&fixture.vault, &password).unwrap();
    let key = archive_v3::ContactKeyPair::generate().unwrap();
    let original_public = key.public_key().to_bytes();
    vault.store_private_key("default", &key).unwrap();
    let profile_password =
        vault_v3::SecretString::try_from_slice(b"retained profile password").unwrap();
    vault
        .store_password_profile("password-profile", &profile_password, false)
        .unwrap();
    vault
        .store_private_key(
            "archive-owner",
            &archive_v3::ContactKeyPair::generate().unwrap(),
        )
        .unwrap();
    let archive = fixture.root.join("legacy-owned.lbox");
    let signer = vault.load_owner_signing_key("archive-owner").unwrap();
    let mut legacy = archive_v3::Lockbox::create_file(
        &archive,
        archive_v3::LockboxProtection::ContentKey(
            archive_v3::SecretVec::try_from_slice(b"migration-test-content-key").unwrap(),
        ),
        &signer,
    )
    .unwrap();
    legacy.add_contact(&key.public_key()).unwrap();
    legacy.commit().unwrap();
    legacy
        .add_file(
            &archive_v3::LockboxPath::new("/notes.txt").unwrap(),
            b"migration content",
            false,
        )
        .unwrap();
    legacy.commit().unwrap();
    let archive_id = legacy.lockbox_id();
    vault.remember_known_lockbox(archive_id, &archive).unwrap();
    drop(legacy);
    let original_archive = std::fs::read(&archive).unwrap();
    let container_signer = vault.load_owner_signing_key("default").unwrap();
    drop(vault);
    // No current CLI can add records to a historical Vault. Seed the exact
    // released alias record plus an unknown binary record through its frozen
    // API; verify both through separate CLI reads after migration.
    let mut old_container = archive_v3::Lockbox::open_with_signer(
        &fixture.vault.join("local-vault.lbox"),
        archive_v3::LockboxOpen::Password(&password),
        |_| Ok(container_signer),
    )
    .unwrap();
    let mut alias = b"LBLA".to_vec();
    alias.extend_from_slice(&1_u16.to_be_bytes());
    alias.extend_from_slice(archive_id.as_bytes());
    old_container
        .add_file(
            &archive_v3::LockboxPath::new("/lockbox_aliases/retained.lbla").unwrap(),
            &alias,
            false,
        )
        .unwrap();
    let opaque = b"opaque Vault extension\0\xff".repeat(8193);
    old_container
        .add_file(
            &archive_v3::LockboxPath::new("/future-extension/payload").unwrap(),
            &opaque,
            false,
        )
        .unwrap();
    old_container.commit().unwrap();
    drop(old_container);
    let original = std::fs::read(fixture.vault.join("local-vault.lbox")).unwrap();
    let exporter = build_selected_exporter("revault_migrate_v3", "revault-migrate-vault-v3");
    fixture.success(&[
        "doctor",
        "migrate",
        "vault",
        "--replace",
        "--exporter",
        path(&exporter),
    ]);
    let backup = fixture.vault.with_file_name("vault.v3-v3.pre-migration");
    assert_eq!(
        std::fs::read(backup.join("local-vault.lbox")).unwrap(),
        original
    );
    let aliases = fixture.success(&["vault", "lockbox", "alias", "list"]);
    assert!(String::from_utf8_lossy(&aliases.stdout).contains("retained"));
    // Use a separate credential Vault/session so the target container is not
    // also held open as this command's own credential store.
    let verification = Fixture::new("raw-vault-public-verification");
    verification.init_current_vault();
    let opaque_open = verification
        .command(&[path(&fixture.vault.join("local-vault.lbox")), "open"])
        .env_remove("LOCKBOX_KEY")
        .env("LOCKBOX_PASSWORD", VAULT_PASSWORD)
        .test_output()
        .unwrap();
    assert_success(&opaque_open);
    let opaque_read = verification
        .command(&[
            path(&fixture.vault.join("local-vault.lbox")),
            "cat",
            "/future-extension/payload",
        ])
        .env_remove("LOCKBOX_KEY")
        .env("LOCKBOX_PASSWORD", VAULT_PASSWORD)
        .test_output()
        .unwrap();
    assert_success(&opaque_read);
    assert_eq!(opaque_read.stdout, opaque);
    let opaque_close = verification
        .command(&[path(&fixture.vault.join("local-vault.lbox")), "close"])
        .env_remove("LOCKBOX_KEY")
        .test_output()
        .unwrap();
    assert_success(&opaque_close);
    drop(verification);
    fixture.success(&[
        "vault",
        "profile",
        "export",
        path(&fixture.root.join("profile.pub")),
        "--format",
        "raw",
    ]);
    assert_eq!(
        revault_vault_api::import_public_key(
            &std::fs::read(fixture.root.join("profile.pub")).unwrap()
        )
        .unwrap()
        .to_bytes(),
        original_public
    );
    let profiles = fixture.success(&["vault", "profile", "list"]);
    assert!(String::from_utf8_lossy(&profiles.stdout).contains("password-profile"));
    let credential = fixture.root.join("retained-password");
    fixture.success(&[
        "vault",
        "profile",
        "password",
        "password-profile",
        "--output",
        path(&credential),
    ]);
    assert_eq!(
        std::fs::read(credential).unwrap(),
        b"retained profile password"
    );
    let archive_exporter =
        build_selected_exporter("revault_migrate_v3", "revault-migrate-archive-v3");
    fixture.success(&[
        "doctor",
        "migrate",
        "lockbox",
        path(&archive),
        "--replace",
        "--exporter",
        path(&archive_exporter),
    ]);
    assert_eq!(
        std::fs::read(archive.with_file_name(format!(
            "legacy-owned.lbox.v3-v{LOCKBOX_FORMAT_VERSION}.pre-migration"
        )))
        .unwrap(),
        original_archive
    );
    fixture.success(&[path(&archive), "doctor", "compact"]);
    let content = fixture.success(&[path(&archive), "cat", "/notes.txt"]);
    assert_eq!(content.stdout, b"migration content");
    assert_eq!(
        fixture.success(&["a@retained", "cat", "/notes.txt"]).stdout,
        b"migration content"
    );
    let deep = fixture.success(&[path(&archive), "doctor", "--deep"]);
    assert!(String::from_utf8_lossy(&deep.stdout)
        .contains("Deep storage check: all allocations accounted for"));
    fixture.success(&["doctor", "migrate", "vault", "--replace"]);
}

#[test]
fn automatic_exporter_discovery_skips_obsolete_protocol_but_explicit_selection_refuses_it() {
    let fixture = Fixture::new("obsolete-exporter-discovery");
    // Historical setup requires the frozen writer; replacement and persisted
    // profile verification below use separate public CLI invocations.
    let password = vault_v3::SecretString::try_from_slice(VAULT_PASSWORD.as_bytes()).unwrap();
    let vault = vault_v3::VaultDirectory::replace(&fixture.vault, &password).unwrap();
    vault
        .store_private_key("default", &archive_v3::ContactKeyPair::generate().unwrap())
        .unwrap();
    drop(vault);
    let container = fixture.vault.join("local-vault.lbox");
    let original = std::fs::read(&container).unwrap();
    let bin = fixture.root.join("older-install");
    std::fs::create_dir(&bin).unwrap();
    let lockbox = bin.join(format!("lockbox{}", std::env::consts::EXE_SUFFIX));
    std::fs::hard_link(env!("CARGO_BIN_EXE_lockbox"), &lockbox)
        .or_else(|_| std::fs::copy(env!("CARGO_BIN_EXE_lockbox"), &lockbox).map(|_| ()))
        .unwrap();
    let obsolete = bin.join(format!(
        "revault-migrate-vault-v3{}",
        std::env::consts::EXE_SUFFIX
    ));
    assert!(Command::new("rustc")
        .args(["--edition=2021"])
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/legacy_vault_exporter.rs"))
        .arg("-o")
        .arg(&obsolete)
        .status()
        .unwrap()
        .success());
    let selected = fixture.run(&[
        "doctor",
        "migrate",
        "vault",
        "--replace",
        "--exporter",
        path(&obsolete),
    ]);
    assert_failure_contains(&selected, "capabilities do not match");
    assert_eq!(std::fs::read(&container).unwrap(), original);
    let template = fixture.command(&[]);
    let mut command = Command::new(&lockbox);
    for (name, value) in template.get_envs() {
        if let Some(value) = value {
            command.env(name, value);
        }
    }
    let migrated = command
        .args(["doctor", "migrate", "vault", "--replace"])
        .env_remove("LOCKBOX_TEST_IGNORE_INSTALLED_EXPORTER")
        .test_output()
        .unwrap();
    assert_success(&migrated);
    assert_eq!(
        std::fs::read(
            fixture
                .vault
                .with_file_name("vault.v3-v3.pre-migration")
                .join("local-vault.lbox")
        )
        .unwrap(),
        original
    );
    let profiles = fixture.success(&["vault", "profile", "list"]);
    assert!(String::from_utf8_lossy(&profiles.stdout).contains("default"));
    assert_current_vault(&fixture.vault);
}
