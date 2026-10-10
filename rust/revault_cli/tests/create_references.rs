mod common;

use common::CommandTestExt;
use std::process::{Command, Output};

const LBX: &str = env!("CARGO_BIN_EXE_lockbox");

struct Fixture(tempfile::TempDir);

impl Fixture {
    fn new() -> Self {
        let fixture = Self(tempfile::tempdir().unwrap());
        fixture.ok(&["vault", "init"]);
        fixture.ok(&["original.lbox", "create"]);
        fixture.ok(&["original.lbox", "open"]);
        fixture.ok(&[
            "original.lbox",
            "variable",
            "set",
            "TOKEN",
            "original bytes",
        ]);
        fixture.ok(&["vault", "lockbox", "alias", "set", "first", "original.lbox"]);
        fixture.ok(&[
            "vault",
            "lockbox",
            "alias",
            "set",
            "second",
            "original.lbox",
        ]);
        fixture.ok(&["session", "default", "a@first"]);
        fixture
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(LBX)
            .current_dir(self.0.path())
            .args(args)
            .env("LOCKBOX_VAULT_DIR", self.0.path().join("vault"))
            .env("LOCKBOX_SESSION_AGENT_DIR", self.0.path().join("agent"))
            .env("LOCKBOX_SESSION_AGENT_LOG", self.0.path().join("agent.log"))
            .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
            .env("LOCKBOX_VAULT_PASSWORD", "synthetic-vault-password")
            .env_remove("LOCKBOX_KEY")
            .env_remove("LOCKBOX_PASSWORD")
            .env_remove("COMPLETE")
            .test_output()
            .unwrap()
    }

    fn ok(&self, args: &[&str]) -> Output {
        let output = self.run(args);
        assert!(output.status.success(), "{args:?}: {output:?}");
        output
    }

    fn identities(&self) -> Vec<String> {
        let output = self.ok(&["vault", "lockbox", "alias", "list", "--format", "json"]);
        serde_json::Deserializer::from_slice(&output.stdout)
            .into_iter::<serde_json::Value>()
            .map(|row| row.unwrap()["lockbox_id"].as_str().unwrap().to_string())
            .collect()
    }

    fn assert_value(&self, value: &[u8]) {
        for args in [
            vec!["a@first", "variable", "get", "TOKEN"],
            vec!["a@second", "variable", "get", "TOKEN"],
            vec!["variable", "get", "TOKEN"],
        ] {
            assert_eq!(self.ok(&args).stdout, value);
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.run(&["session", "stop"]);
    }
}

#[test]
fn create_alias_default_and_path_recreate_with_new_identity_and_all_aliases() {
    for selector in [Some("a@first"), None, Some("original.lbox")] {
        for configured in [false, true] {
            let f = Fixture::new();
            let path = if selector == Some("original.lbox") {
                "original.lbox"
            } else {
                f.ok(&["vault", "lockbox", "move", "a@first", "./extensionless"]);
                "extensionless"
            };
            f.ok(&["a@first", "open"]);
            f.assert_value(b"original bytes\n");
            let old = f.identities();
            // There is no public CLI operation for externally deleting an
            // archive while preserving its remembered aliases and default.
            std::fs::remove_file(f.0.path().join(path)).unwrap();
            let mut args = selector.into_iter().collect::<Vec<_>>();
            args.push("create");
            if configured {
                args.extend(["--compression", "none"]);
            }
            f.ok(&args);
            assert!(f.0.path().join(path).is_file());
            assert!(!f.0.path().join("extensionless.lbox").exists());
            let new = f.identities();
            assert_ne!(old, new);
            assert_eq!(new[0], new[1]);
            f.ok(&["a@first", "open"]);
            assert!(!f
                .run(&["a@first", "variable", "get", "TOKEN"])
                .status
                .success());
            f.ok(&["variable", "set", "TOKEN", "replacement bytes"]);
            f.assert_value(b"replacement bytes\n");
            let repeat = f.run(&args);
            assert!(!repeat.status.success(), "{repeat:?}");
            assert!(String::from_utf8_lossy(&repeat.stderr).contains("already exists"));
            f.assert_value(b"replacement bytes\n");
        }
    }
}

#[test]
fn create_refuses_existing_targets_before_archive_inspection() {
    let f = Fixture::new();
    for selector in [Some("a@first"), None, Some("original.lbox")] {
        for configured in [false, true] {
            let mut args = selector.into_iter().collect::<Vec<_>>();
            args.push("create");
            if configured {
                args.extend(["--encryption", "none", "--signing", "none"]);
            }
            let output = f.run(&args);
            assert!(!output.status.success(), "{output:?}");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("already exists"),
                "{output:?}"
            );
            f.assert_value(b"original bytes\n");
        }
    }
    // Simulate an external replacement with a non-archive. Create must refuse
    // the directory entry without trying to inspect/open its contents.
    std::fs::write(f.0.path().join("original.lbox"), b"external replacement").unwrap();
    for args in [vec!["a@first", "create"], vec!["create"]] {
        let output = f.run(&args);
        assert!(!output.status.success(), "{output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("already exists"),
            "{output:?}"
        );
    }
    assert_eq!(
        std::fs::read(f.0.path().join("original.lbox")).unwrap(),
        b"external replacement"
    );
}

#[test]
fn plaintext_recreation_rebinds_aliases_even_without_key_directory_mirroring() {
    let f = Fixture::new();
    let old = f.identities();
    // External archive deletion has no equivalent public CLI operation.
    std::fs::remove_file(f.0.path().join("original.lbox")).unwrap();
    f.ok(&[
        "a@first",
        "create",
        "--encryption",
        "none",
        "--signing",
        "none",
    ]);
    let new = f.identities();
    assert_ne!(old, new);
    assert_eq!(new[0], new[1]);
    f.ok(&[
        "a@first",
        "variable",
        "set",
        "TOKEN",
        "plaintext replacement",
    ]);
    f.assert_value(b"plaintext replacement\n");
}

#[cfg(unix)]
#[test]
fn create_refuses_dangling_symlink_reference() {
    let f = Fixture::new();
    // No CLI operation creates a dangling host symlink; simulate external
    // replacement to check that create does not follow it to a new archive.
    std::fs::remove_file(f.0.path().join("original.lbox")).unwrap();
    std::os::unix::fs::symlink("absent-target", f.0.path().join("original.lbox")).unwrap();
    let output = f.run(&["a@first", "create"]);
    assert!(!output.status.success(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("already exists"),
        "{output:?}"
    );
    assert!(!f.0.path().join("absent-target").exists());
    assert_eq!(
        std::fs::read_link(f.0.path().join("original.lbox")).unwrap(),
        std::path::Path::new("absent-target")
    );
}
