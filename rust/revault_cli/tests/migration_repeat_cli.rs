//! Repeat migration authenticates current archives without requiring an open
//! session. All Vault/archive state is created through the public CLI.
mod common;

use common::{CommandTestExt, TestTempDir};
use std::path::Path;
use std::process::{Command, Output};

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lockbox"))
        .args(args)
        .env("LOCKBOX_VAULT_DIR", root.join("vault"))
        .env(
            "LOCKBOX_SESSION_AGENT_DIR",
            common::agent_socket_dir(&root.join("agent")),
        )
        .env("LOCKBOX_SESSION_AGENT_LOG", root.join("agent.log"))
        .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
        .env("LOCKBOX_VAULT_PASSWORD", "repeat migration test password")
        .env_remove("LOCKBOX_PASSWORD")
        .env_remove("LOCKBOX_CONTENT_KEY")
        .test_output()
        .unwrap()
}

fn ok(root: &Path, args: &[&str]) -> Output {
    let result = run(root, args);
    assert!(
        result.status.success(),
        "{args:?}: {}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    result
}

#[test]
fn repeated_migration_of_closed_current_archives_authenticates_without_mutation() {
    let temp = TestTempDir::new("migration-repeat");
    let root = temp.path();
    let vault = root.join("vault");
    let agent = common::agent_socket_dir(&root.join("agent"));
    let _guard = common::TestAgentGuard::new(&agent, Some(&vault));
    ok(root, &["vault", "init"]);
    let source = root.join("original.lbox");
    let source = source.to_str().unwrap();
    let migrated = root.join("migrated.lbox");
    let migrated = migrated.to_str().unwrap();
    let input = root.join("payload.bin");
    let expected = b"migration repeat\0\xff\n";
    std::fs::write(&input, expected).unwrap();
    ok(root, &[source, "create"]);
    ok(root, &[source, "open"]);
    ok(
        root,
        &[
            source,
            "add",
            input.to_str().unwrap(),
            "--to",
            "/payload.bin",
        ],
    );
    ok(
        root,
        &["doctor", "migrate", "lockbox", source, "--output", migrated],
    );
    ok(root, &["session", "stop"]);
    let before = std::fs::read(migrated).unwrap();
    for _ in 0..2 {
        let output = ok(root, &["doctor", "migrate", "all", "--replace"]);
        assert!(String::from_utf8_lossy(&output.stdout).contains("0 failed"));
        assert_eq!(std::fs::read(migrated).unwrap(), before);
        // Migration must not implicitly open the archive in the Session Agent.
        let output = ok(root, &["session"]);
        assert!(!String::from_utf8_lossy(&output.stdout).contains(migrated));
    }
    ok(root, &[migrated, "open"]);
    assert_eq!(
        ok(root, &[migrated, "cat", "/payload.bin"]).stdout,
        expected
    );
    ok(root, &["session", "stop"]);
    // Replacing a host file is possible outside the CLI; the remembered
    // identity must still prevent accepting the wrong archive as migrated.
    let replacement = root.join("replacement.lbox");
    ok(root, &[replacement.to_str().unwrap(), "create"]);
    let replacement_bytes = std::fs::read(&replacement).unwrap();
    std::fs::write(migrated, &replacement_bytes).unwrap();
    let rejected = run(root, &["doctor", "migrate", "all", "--replace"]);
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("identity changed"));
    assert_eq!(std::fs::read(migrated).unwrap(), replacement_bytes);
}

#[test]
fn current_format_noop_migration_still_requires_valid_credentials() {
    let owner = TestTempDir::new("migration-owner");
    let stranger = TestTempDir::new("migration-stranger");
    let owner_root = owner.path();
    let other_root = stranger.path();
    let owner_vault = owner_root.join("vault");
    let other_vault = other_root.join("vault");
    let _owner_guard = common::TestAgentGuard::new(
        &common::agent_socket_dir(&owner_root.join("agent")),
        Some(&owner_vault),
    );
    let _other_guard = common::TestAgentGuard::new(
        &common::agent_socket_dir(&other_root.join("agent")),
        Some(&other_vault),
    );
    ok(owner_root, &["vault", "init"]);
    ok(other_root, &["vault", "init"]);
    let archive = owner_root.join("private.lbox");
    let archive = archive.to_str().unwrap();
    ok(owner_root, &[archive, "create"]);
    ok(owner_root, &["session", "stop"]);
    let before = std::fs::read(archive).unwrap();
    let rejected = run(
        other_root,
        &["doctor", "migrate", "lockbox", archive, "--replace"],
    );
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr).contains("credential"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
    );
    assert_eq!(std::fs::read(archive).unwrap(), before);
}
