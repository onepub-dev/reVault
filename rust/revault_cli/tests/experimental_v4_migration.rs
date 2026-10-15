mod common;

use common::{CommandTestExt, TestTempDir};
use std::path::Path;
use std::process::{Command, Output};

fn successful(output: Output) -> Vec<u8> {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

#[test]
#[ignore = "requires the pinned isolated exporters and historical CLI; set REVAULT_EXPERIMENTAL_V4_BUILD"]
fn historical_v4_vault_and_password_archive_migrate_through_public_cli() {
    let build = std::path::PathBuf::from(
        std::env::var_os("REVAULT_EXPERIMENTAL_V4_BUILD").expect("isolated exporter build path"),
    );
    let old_cli = build.join("adapter/target/release/lockbox");
    let current_cli = Path::new(env!("CARGO_BIN_EXE_lockbox"));
    let root = TestTempDir::new("historical-v4-security");
    let old_vault = root.path().join("old-vault");
    let new_vault = root.path().join("new-vault");
    let old_agent = common::agent_socket_dir(&root.path().join("old-agent"));
    let new_agent = common::agent_socket_dir(&root.path().join("new-agent"));
    let run = |binary: &Path, vault: &Path, args: &[&str]| {
        Command::new(binary)
            .current_dir(root.path())
            .args(args)
            .env("LOCKBOX_VAULT_DIR", vault)
            .env("LOCKBOX_VAULT_PASSWORD", "historical vault fixture")
            .env("LOCKBOX_PASSWORD", "historical archive fixture")
            .env_remove("LOCKBOX_KEY")
            .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
            .env(
                "LOCKBOX_SESSION_AGENT_DIR",
                if binary == old_cli.as_path() {
                    &old_agent
                } else {
                    &new_agent
                },
            )
            .env("LOCKBOX_SESSION_AGENT_LOG", root.path().join("agent.log"))
            .test_output()
            .unwrap()
    };
    successful(run(&old_cli, &old_vault, &["vault", "init"]));
    successful(run(
        &old_cli,
        &old_vault,
        &[
            "vault",
            "profile",
            "export",
            "old-profile.pub",
            "--format",
            "raw",
        ],
    ));
    successful(run(
        &old_cli,
        &old_vault,
        &["old.lbox", "create", "--password"],
    ));
    // Exercise historical compressed content with varied repeated Huffman symbols.
    let alphabet = b"aaabbbccccdddddeeeeeeffffffgghijklmno";
    let payload = (0..800_000)
        .map(|index| alphabet[index % alphabet.len()])
        .collect::<Vec<_>>();
    std::fs::write(root.path().join("input"), &payload).unwrap();
    successful(run(
        &old_cli,
        &old_vault,
        &["old.lbox", "add", "input", "--to", "/kept"],
    ));
    successful(run(
        &old_cli,
        &old_vault,
        &["old.lbox", "variable", "set", "/state", "retained value"],
    ));
    assert_eq!(
        successful(run(&old_cli, &old_vault, &["old.lbox", "cat", "/kept"])),
        payload
    );
    successful(run(&old_cli, &old_vault, &["session", "stop"]));
    let old_archive = std::fs::read(root.path().join("old.lbox")).unwrap();
    let old_vault_bytes = std::fs::read(old_vault.join("local-vault.lbox")).unwrap();
    let vault_exporter = build.join("revault-migrate-vault-experimental-v4");
    successful(run(
        current_cli,
        &old_vault,
        &[
            "doctor",
            "migrate",
            "vault",
            "--output",
            new_vault.to_str().unwrap(),
            "--exporter",
            vault_exporter.to_str().unwrap(),
        ],
    ));
    assert_eq!(
        std::fs::read(old_vault.join("local-vault.lbox")).unwrap(),
        old_vault_bytes
    );
    successful(run(
        current_cli,
        &new_vault,
        &[
            "vault",
            "profile",
            "export",
            "new-profile.pub",
            "--format",
            "raw",
        ],
    ));
    assert_eq!(
        std::fs::read(root.path().join("new-profile.pub")).unwrap(),
        std::fs::read(root.path().join("old-profile.pub")).unwrap()
    );
    assert!(!run(
        current_cli,
        &new_vault,
        &["doctor", "migrate", "lockbox", "old.lbox", "--replace"]
    )
    .status
    .success());
    assert_eq!(
        std::fs::read(root.path().join("old.lbox")).unwrap(),
        old_archive
    );
    let archive_exporter = build.join("revault-migrate-archive-experimental-v4");
    successful(run(
        current_cli,
        &new_vault,
        &[
            "doctor",
            "migrate",
            "lockbox",
            "old.lbox",
            "--output",
            "new.lbox",
            "--exporter",
            archive_exporter.to_str().unwrap(),
        ],
    ));
    assert_eq!(
        std::fs::read(root.path().join("old.lbox")).unwrap(),
        old_archive
    );
    successful(run(
        current_cli,
        &new_vault,
        &["new.lbox", "open", "--password-env", "LOCKBOX_PASSWORD"],
    ));
    assert!(String::from_utf8(successful(run(
        current_cli,
        &new_vault,
        &["new.lbox", "doctor"]
    )))
    .unwrap()
    .contains("signing: owner"));
    assert_eq!(
        successful(run(current_cli, &new_vault, &["new.lbox", "cat", "/kept"])),
        payload
    );
    assert!(String::from_utf8(successful(run(
        current_cli,
        &new_vault,
        &["new.lbox", "variable", "get", "/state"]
    )))
    .unwrap()
    .contains("retained value"));
    let current_bytes = std::fs::read(root.path().join("new.lbox")).unwrap();
    successful(run(
        current_cli,
        &new_vault,
        &["doctor", "migrate", "lockbox", "new.lbox", "--replace"],
    ));
    assert_eq!(
        std::fs::read(root.path().join("new.lbox")).unwrap(),
        current_bytes
    );
    successful(run(current_cli, &new_vault, &["session", "stop"]));
}
