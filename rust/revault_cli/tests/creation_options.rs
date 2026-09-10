mod common;
use common::{CommandTestExt, TestTempDir};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(dir: &Path, encrypted: bool, args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lockbox"));
    command
        .current_dir(dir)
        .args(args)
        .env("LOCKBOX_VAULT_DIR", dir.join("vault"))
        .env("LOCKBOX_VAULT_PASSWORD", "test vault password")
        .env(
            "LOCKBOX_SESSION_AGENT_DIR",
            common::agent_socket_dir(&dir.join("agent")),
        )
        .env("LOCKBOX_SESSION_AGENT_LOG", dir.join("agent.log"))
        .env("LOCKBOX_ADD_PROGRESS", "off")
        .env_remove("LOCKBOX_KEY");
    if encrypted {
        command.env("LOCKBOX_KEY", "test raw content key");
    }
    command.test_output().unwrap()
}

fn success(output: Output) -> Vec<u8> {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

#[test]
fn independent_creation_choices_persist_through_cli_lifecycle() {
    std::thread::scope(|scope| {
        for encrypted in [false, true] {
            for signed in [false, true] {
                for compressed in [false, true] {
                    scope.spawn(move || {
                        let temp = TestTempDir::new("creation-options");
                        let dir = temp.path();
                        if signed {
                            success(run(dir, false, &["vault", "init"]));
                        }
                        let archive = format!("{encrypted}-{signed}-{compressed}.lbox");
                        let encryption = if encrypted {
                            "chacha20-poly1305"
                        } else {
                            "none"
                        };
                        let signing = if signed { "owner" } else { "none" };
                        let compression = if compressed { "zstd" } else { "none" };
                        let mut create = vec![
                            archive.as_str(),
                            "create",
                            "--encryption",
                            encryption,
                            "--signing",
                            signing,
                            "--compression",
                            compression,
                        ];
                        if compressed {
                            create.extend(["--compression-level", "6"]);
                        }
                        success(run(dir, encrypted, &create));
                        if !encrypted {
                            success(run(dir, false, &[&archive, "open"]));
                        }
                        // Creating again must refuse the existing archive.
                        assert!(!run(dir, encrypted, &create).status.success());
                        let status =
                            String::from_utf8(success(run(dir, encrypted, &[&archive, "doctor"])))
                                .unwrap();
                        assert!(status.contains(&format!("encryption: {encryption}")));
                        assert!(status.contains(&format!("signing: {signing}")));
                        assert!(status.contains(&format!("compression: {compression}")));
                        if compressed {
                            assert!(status.contains("level 6"));
                        }
                        let original = b"compressible example line\n".repeat(12_000);
                        fs::write(dir.join("source.txt"), &original).unwrap();
                        success(run(
                            dir,
                            encrypted,
                            &[&archive, "add", "source.txt", "--to", "/notes.txt"],
                        ));
                        assert_eq!(
                            success(run(dir, encrypted, &[&archive, "cat", "/notes.txt"])),
                            original
                        );
                        // Repeating the add without replacement must preserve the bytes.
                        assert!(!run(
                            dir,
                            encrypted,
                            &[&archive, "add", "source.txt", "--to", "/notes.txt"]
                        )
                        .status
                        .success());
                        assert_eq!(
                            success(run(dir, encrypted, &[&archive, "cat", "/notes.txt"])),
                            original
                        );
                        fs::write(dir.join("source.txt"), b"replacement bytes").unwrap();
                        success(run(
                            dir,
                            encrypted,
                            &[
                                &archive,
                                "add",
                                "--overwrite",
                                "source.txt",
                                "--to",
                                "/notes.txt",
                            ],
                        ));
                        assert_eq!(
                            success(run(dir, encrypted, &[&archive, "cat", "/notes.txt"])),
                            b"replacement bytes"
                        );
                        success(run(
                            dir,
                            encrypted,
                            &[&archive, "add", "source.txt", "--to", "/second.txt"],
                        ));
                        assert_eq!(
                            success(run(dir, encrypted, &[&archive, "cat", "/second.txt"])),
                            b"replacement bytes"
                        );
                        success(run(dir, encrypted, &[&archive, "remove", "/notes.txt"]));
                        assert_eq!(
                            success(run(dir, encrypted, &[&archive, "cat", "/notes.txt"])),
                            b"replacement bytes"
                        );
                        success(run(
                            dir,
                            encrypted,
                            &[&archive, "remove", "--force", "/notes.txt"],
                        ));
                        assert!(!run(dir, encrypted, &[&archive, "cat", "/notes.txt"])
                            .status
                            .success());
                        let listed =
                            String::from_utf8(success(run(dir, encrypted, &[&archive, "list"])))
                                .unwrap();
                        assert!(!listed.contains("notes.txt"));
                        assert!(listed.contains("second.txt"));
                        assert_eq!(
                            success(run(dir, encrypted, &[&archive, "cat", "/second.txt"])),
                            b"replacement bytes"
                        );
                        if !signed {
                            assert!(!dir.join("vault").exists());
                        }
                    });
                }
            }
        }
    });
}

#[test]
fn invalid_creation_combinations_are_rejected_before_creation() {
    let temp = TestTempDir::new("creation-invalid");
    for args in [
        vec!["invalid.lbox", "create", "--compression-level", "0"],
        vec!["invalid.lbox", "create", "--compression-level", "23"],
        vec![
            "invalid.lbox",
            "create",
            "--compression",
            "none",
            "--compression-level",
            "3",
        ],
        vec![
            "invalid.lbox",
            "create",
            "--encryption",
            "none",
            "--password",
        ],
    ] {
        assert!(!run(temp.path(), false, &args).status.success());
        assert!(!temp.path().join("invalid.lbox").exists());
    }
}
