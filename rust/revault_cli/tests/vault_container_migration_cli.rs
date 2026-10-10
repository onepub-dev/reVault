mod common;

use common::{agent_socket_dir, CommandTestExt, TestTempDir};
use revault_lockbox_api::{
    ListOptions, Lockbox, LockboxEntryKind, LockboxOpen, LockboxPath, SecretString,
    VariableSensitivity,
};
use revault_lockbox_api_structure_v2 as old;
use revault_vault_api::VaultDirectory;
use std::path::Path;
use std::process::{Command, Output};

const PASSWORD: &str = "container migration test password";

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lbx"))
        .args(args)
        .env("LOCKBOX_VAULT_DIR", root.join("vault"))
        .env("LOCKBOX_VAULT_PASSWORD", PASSWORD)
        .env_remove("LOCKBOX_PASSWORD")
        .env("LOCKBOX_KEY", "container-migration-fixture-key")
        .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
        .env(
            "LOCKBOX_SESSION_AGENT_DIR",
            agent_socket_dir(&root.join("agent")),
        )
        .env("LOCKBOX_SESSION_AGENT_LOG", root.join("agent.log"))
        .test_output()
        .unwrap()
}

fn success(root: &Path, args: &[&str]) -> Output {
    let output = run(root, args);
    assert!(
        output.status.success(),
        "{args:?}: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

// The public CLI cannot write historical containers. Create all Vault records
// through CLI commands first, then copy them unchanged into a format-2 container
// using the pinned historical API solely to reproduce this mixed-version state.
fn rewrite_as_container_two(root: &Path) -> Vec<u8> {
    let path = root.join("vault/local-vault.lbox");
    let password = SecretString::try_from_slice(PASSWORD.as_bytes()).unwrap();
    let source = Lockbox::open(&path, LockboxOpen::Password(&password)).unwrap();
    let signing_record = VaultDirectory::migration_container_signing_key(&source)
        .unwrap()
        .private_key_record()
        .unwrap();
    let signer = old::OwnerSigningKeyPair::from_private_key_record(
        signing_record
            .with_bytes(old::SecretVec::try_from_slice)
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    let old_password = old::SecretString::try_from_slice(PASSWORD.as_bytes()).unwrap();
    let mut destination =
        old::Lockbox::create_in_memory(old::LockboxProtection::Password(&old_password), &signer)
            .unwrap();
    let root_path = LockboxPath::new("/").unwrap();
    let mut options = ListOptions::new(&root_path);
    options.recursive = true;
    for entry in source.list(options).unwrap() {
        let entry = entry.unwrap();
        let old_path = old::LockboxPath::new(entry.path.as_str()).unwrap();
        match entry.kind {
            LockboxEntryKind::Directory => destination.create_dir(&old_path, true).unwrap(),
            LockboxEntryKind::File => destination
                .add_file(&old_path, &source.get_file(&entry.path).unwrap(), false)
                .unwrap(),
            LockboxEntryKind::Symlink => panic!("unexpected symlink in Vault fixture"),
        }
    }
    for (name, sensitivity) in source.list_variables().unwrap() {
        let old_name = old::VariableName::new(name.as_str()).unwrap();
        match sensitivity {
            VariableSensitivity::Normal => destination
                .set_variable(&old_name, &source.get_variable(&name).unwrap().unwrap())
                .unwrap(),
            VariableSensitivity::Secret => {
                let secret = source
                    .with_secret_variable(&name, |value| {
                        value.with_bytes(old::SecretString::try_from_slice)
                    })
                    .unwrap()
                    .unwrap()
                    .unwrap()
                    .unwrap();
                destination.set_secret_variable(&old_name, &secret).unwrap();
            }
        }
    }
    destination.commit().unwrap();
    let bytes = destination.try_to_bytes().unwrap();
    assert_eq!(
        revault_lockbox_api::probe_lockbox_format_version(&bytes).unwrap(),
        2
    );
    drop(source);
    std::fs::write(&path, &bytes).unwrap();
    assert_eq!(
        VaultDirectory::probe_structure_version(root.join("vault"), &password).unwrap(),
        3
    );
    bytes
}

#[test]
fn current_vault_structure_in_old_container_migrates_losslessly() {
    let temp = TestTempDir::new("vault-container-only-migration");
    let root = temp.path();
    let archive = root.join("personal.lbox");
    let archive_path = archive.to_str().unwrap();
    success(root, &["vault", "init"]);
    success(root, &[archive_path, "create"]);
    let payload = root.join("payload.txt");
    std::fs::write(&payload, b"preserved alias target bytes\0\xff").unwrap();
    success(
        root,
        &[
            archive_path,
            "add",
            payload.to_str().unwrap(),
            "--to",
            "/payload",
        ],
    );
    success(
        root,
        &[
            "vault",
            "lockboxes",
            "aliases",
            "set",
            "personal",
            archive_path,
        ],
    );
    success(
        root,
        &["vault", "profiles", "create", "server", "--password"],
    );
    success(root, &["vault", "profiles", "rotate", "default"]);
    let queries: &[&[&str]] = &[
        &["vault", "lockboxes", "aliases", "list", "--format", "json"],
        &["vault", "profiles", "list", "--format", "json"],
        &[
            "vault", "profiles", "history", "default", "--format", "json",
        ],
        &["vault", "profiles", "password", "server"],
    ];
    let before: Vec<_> = queries
        .iter()
        .map(|query| success(root, query).stdout)
        .collect();
    let old_bytes = rewrite_as_container_two(root);
    let refused = run(root, &["vault", "lockboxes", "aliases", "list"]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("migrate vault"));
    let report = success(root, &["doctor"]);
    let report = String::from_utf8_lossy(&report.stdout);
    assert!(report.contains("container format version: 2"), "{report}");
    assert!(report.contains("structure version: 3"), "{report}");
    assert!(report.contains("status: upgrade required"), "{report}");
    let selected = success(root, &[archive_path, "doctor"]);
    let selected = String::from_utf8_lossy(&selected.stdout);
    assert!(
        selected.contains("Local vault\n  container format version: 2"),
        "{selected}"
    );
    assert!(selected.contains("structure version: 3"), "{selected}");
    assert!(selected.contains("status: upgrade required"), "{selected}");
    assert!(
        !selected.contains("key-directory backup: error"),
        "{selected}"
    );
    assert_eq!(
        std::fs::read(root.join("vault/local-vault.lbox")).unwrap(),
        old_bytes
    );

    success(root, &["doctor", "migrate", "vault", "--replace"]);
    let backup = root.join("vault.v3-v3.pre-migration/local-vault.lbox");
    assert_eq!(std::fs::read(&backup).unwrap(), old_bytes);
    let migrated = std::fs::read(root.join("vault/local-vault.lbox")).unwrap();
    assert_eq!(
        revault_lockbox_api::probe_lockbox_format_version(&migrated).unwrap(),
        3
    );
    for (query, expected) in queries.iter().zip(&before) {
        assert_eq!(&success(root, query).stdout, expected, "{query:?}");
    }
    // Separate public invocations verify alias resolution and a usable retained
    // signing/profile history, including after another persistent mutation.
    assert_eq!(
        success(root, &["a@personal", "cat", "/payload"]).stdout,
        b"preserved alias target bytes\0\xff"
    );
    success(
        root,
        &[
            "vault",
            "lockboxes",
            "aliases",
            "set",
            "second",
            archive_path,
        ],
    );
    let aliases = success(root, &["vault", "lockboxes", "aliases", "list"]);
    assert!(String::from_utf8_lossy(&aliases.stdout).contains("second"));
    success(root, &["vault", "lockboxes", "aliases", "remove", "second"]);
    assert_eq!(success(root, queries[0]).stdout, before[0]);
    let current_bytes = std::fs::read(root.join("vault/local-vault.lbox")).unwrap();
    let repeated = success(root, &["doctor", "migrate", "vault", "--replace"]);
    assert!(String::from_utf8_lossy(&repeated.stdout).contains("No migration needed"));
    assert_eq!(
        std::fs::read(root.join("vault/local-vault.lbox")).unwrap(),
        current_bytes
    );
    assert_eq!(std::fs::read(&backup).unwrap(), old_bytes);
    assert_eq!(success(root, queries[3]).stdout, before[3]);

    let copy_root = root.join("copy");
    let copy_vault = copy_root.join("vault");
    success(
        root,
        &[
            "doctor",
            "migrate",
            "vault",
            "--output",
            copy_vault.to_str().unwrap(),
        ],
    );
    for (query, expected) in queries.iter().zip(&before) {
        assert_eq!(
            &success(&copy_root, query).stdout,
            expected,
            "copied {query:?}"
        );
    }
    assert_eq!(
        std::fs::read(root.join("vault/local-vault.lbox")).unwrap(),
        current_bytes
    );
    assert_eq!(std::fs::read(&backup).unwrap(), old_bytes);
}
