mod common;

use common::CommandTestExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Fixture(tempfile::TempDir);

impl Fixture {
    fn run(&self, binary: &Path, args: &[&str]) -> Output {
        Command::new(binary)
            .current_dir(self.0.path())
            .args(args)
            .env("LOCKBOX_VAULT_DIR", self.0.path().join("vault"))
            .env("LOCKBOX_SESSION_AGENT_DIR", self.0.path().join("agent"))
            .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
            .env("LOCKBOX_VAULT_PASSWORD", "synthetic-compat-vault-password")
            .env("LOCKBOX_KEY", "synthetic-compat-content-key")
            .env_remove("LOCKBOX_PASSWORD")
            .env_remove("COMPLETE")
            .test_output()
            .unwrap()
    }

    fn ok(&self, binary: &Path, args: &[&str]) -> Vec<u8> {
        let output = self.run(binary, args);
        assert!(
            output.status.success(),
            "{} {args:?}: {output:?}",
            binary.display()
        );
        output.stdout
    }

    fn verify(&self, binary: &Path, content: &[u8], value: &str) {
        assert_eq!(
            self.ok(binary, &["compat.lbox", "cat", "/payload"]),
            content
        );
        assert_eq!(
            self.ok(binary, &["compat.lbox", "variable", "get", "VALUE"]),
            format!("{value}\n").as_bytes()
        );
        let mut form_args = vec!["compat.lbox", "form", "get"];
        form_args.extend(form_selector(binary));
        assert_eq!(self.ok(binary, &form_args), format!("{value}\n").as_bytes());
    }
}

fn form_selector(binary: &Path) -> Vec<&'static str> {
    if binary == Path::new(env!("CARGO_BIN_EXE_lockbox")) {
        vec!["/record@text"]
    } else {
        vec!["/record", "text"]
    }
}

fn required_binary(variable: &str) -> PathBuf {
    let path = PathBuf::from(std::env::var_os(variable).unwrap_or_else(|| panic!("{variable} must point to the independently built historical CLI; this test never silently skips")));
    assert!(
        path.is_absolute() && path.is_file(),
        "{variable} must be an absolute executable path"
    );
    path
}

/// Run explicitly with REVAULT_FORMAT3_BASELINE_BIN pointing to released CLI 0.4.0.
/// No native archive or Vault internals are used for setup or content assertions.
#[test]
#[ignore = "requires independently built released CLI 0.4.0; set REVAULT_FORMAT3_BASELINE_BIN"]
fn released_and_restored_cli_mutually_read_and_write_format3_and_vault3() {
    let old = required_binary("REVAULT_FORMAT3_BASELINE_BIN");
    let new = Path::new(env!("CARGO_BIN_EXE_lockbox"));
    let version = Command::new(&old).arg("--version").test_output().unwrap();
    assert!(version.status.success());
    assert!(
        String::from_utf8_lossy(&version.stdout).contains("0.4.0"),
        "{version:?}"
    );
    for (creator, updater) in [(&*old, new), (new, &*old)] {
        let f = Fixture(tempfile::tempdir().unwrap());
        f.ok(creator, &["vault", "init"]);
        f.ok(creator, &["compat.lbox", "create"]);
        let original: Vec<u8> = (0..131_073).map(|i| ((i * 37) % 251) as u8).collect();
        std::fs::write(f.0.path().join("source"), &original).unwrap();
        f.ok(
            creator,
            &["compat.lbox", "add", "source", "--to", "/payload"],
        );
        f.ok(
            creator,
            &["compat.lbox", "variable", "set", "VALUE", "original"],
        );
        f.ok(
            creator,
            &[
                "compat.lbox",
                "form",
                "define",
                "record",
                "--name",
                "Record",
                "--field",
                "text:text",
            ],
        );
        f.ok(
            creator,
            &[
                "compat.lbox",
                "form",
                "add",
                "/record",
                "--type",
                "record",
                "--name",
                "Record",
                "--set",
                "text=original",
            ],
        );
        for reader in [&*old, new] {
            f.verify(reader, &original, "original");
        }

        // No-change operations must also remain readable by either release.
        f.ok(
            updater,
            &["compat.lbox", "variable", "set", "VALUE", "original"],
        );
        assert!(!f
            .run(
                updater,
                &["compat.lbox", "add", "source", "--to", "/payload"]
            )
            .status
            .success());
        for reader in [&*old, new] {
            f.verify(reader, &original, "original");
        }

        let replacement = b"replacement with binary bytes\0\xff\r\n".repeat(8193);
        std::fs::write(f.0.path().join("source"), &replacement).unwrap();
        f.ok(
            updater,
            &[
                "compat.lbox",
                "add",
                "--overwrite",
                "source",
                "--to",
                "/payload",
            ],
        );
        f.ok(
            updater,
            &["compat.lbox", "variable", "set", "VALUE", "replacement"],
        );
        let mut form_args = vec!["compat.lbox", "form", "set", "--value", "replacement"];
        form_args.extend(form_selector(updater));
        f.ok(updater, &form_args);
        f.ok(
            updater,
            &["compat.lbox", "add", "source", "--to", "/temporary"],
        );
        for reader in [&*old, new] {
            f.verify(reader, &replacement, "replacement");
            assert_eq!(
                f.ok(reader, &["compat.lbox", "cat", "/temporary"]),
                replacement
            );
        }
        f.ok(creator, &["compat.lbox", "remove", "--force", "/temporary"]);
        for reader in [&*old, new] {
            assert!(!f
                .run(reader, &["compat.lbox", "cat", "/temporary"])
                .status
                .success());
            f.verify(reader, &replacement, "replacement");
        }

        // New additive Vault alias records must survive an old writer and backup.
        f.ok(
            new,
            &[
                "vault",
                "lockbox",
                "alias",
                "set",
                "compatible",
                "compat.lbox",
            ],
        );
        f.ok(&old, &["vault", "profile", "create", "older-writer"]);
        f.ok(&old, &["vault", "backup", "older.backup"]);
        f.ok(&old, &["vault", "restore", "older.backup", "--overwrite"]);
        assert_eq!(f.ok(new, &["a@compatible", "cat", "/payload"]), replacement);
        f.ok(new, &["vault", "profile", "create", "newer-writer"]);
        let profiles = f.ok(&old, &["vault", "profile", "list"]);
        assert!(String::from_utf8_lossy(&profiles).contains("newer-writer"));
    }
}

#[test]
#[ignore = "requires preserved format-4 CLI; set REVAULT_FORMAT4_BASELINE_BIN"]
fn future_archive_and_vault_are_refused_without_modification() {
    let future = required_binary("REVAULT_FORMAT4_BASELINE_BIN");
    let new = Path::new(env!("CARGO_BIN_EXE_lockbox"));
    let f = Fixture(tempfile::tempdir().unwrap());
    f.ok(&future, &["vault", "init"]);
    f.ok(&future, &["future.lbox", "create"]);
    f.ok(
        &future,
        &["future.lbox", "variable", "set", "VALUE", "future-value"],
    );
    // There is no CLI byte-snapshot command; compare opaque file bytes solely
    // to prove refusal does not mutate the future archive or Vault container.
    let archive = f.0.path().join("future.lbox");
    let vault = f.0.path().join("vault").join("local-vault.lbox");
    let before_archive = std::fs::read(&archive).unwrap();
    let before_vault = std::fs::read(&vault).unwrap();
    for args in [
        vec!["future.lbox", "variable", "get", "VALUE"],
        vec!["future.lbox", "variable", "set", "VALUE", "wrong"],
        vec!["vault", "profile", "list"],
        vec!["vault", "profile", "create", "must-not-exist"],
    ] {
        let output = f.run(new, &args);
        assert!(!output.status.success(), "{args:?}: {output:?}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains("format") && error.contains('4'),
            "{args:?}: {output:?}"
        );
        assert!(
            error.contains("newer") && error.contains("downgrade"),
            "{args:?}: {output:?}"
        );
        assert_eq!(std::fs::read(&archive).unwrap(), before_archive);
        assert_eq!(std::fs::read(&vault).unwrap(), before_vault);
    }
    assert_eq!(
        f.ok(&future, &["future.lbox", "variable", "get", "VALUE"]),
        b"future-value\n"
    );
}
