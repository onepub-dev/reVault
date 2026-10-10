mod common;
use common::CommandTestExt;
use std::process::{Command, Output};

struct Fixture(tempfile::TempDir);

impl Fixture {
    fn new() -> Self {
        let fixture = Self(tempfile::tempdir().unwrap());
        fixture.ok(&["vault", "init"]);
        fixture.ok(&["first.lbox", "create"]);
        fixture.ok(&["first.lbox", "variable", "set", "TOKEN", "first-value"]);
        fixture.ok(&["session", "default", "first.lbox"]);
        fixture
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lockbox"));
        command
            .current_dir(self.0.path())
            .args(args)
            .env("LOCKBOX_VAULT_DIR", self.0.path().join("vault"))
            .env("LOCKBOX_SESSION_AGENT_DIR", self.0.path().join("agent"))
            .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
            .env("LOCKBOX_VAULT_PASSWORD", "synthetic-vault-password")
            .env("LOCKBOX_MIGRATION_PASSWORD", "synthetic-migration-password")
            .env("LOCKBOX_KEY", "synthetic-content-key")
            .env_remove("LOCKBOX_PASSWORD")
            .env_remove("COMPLETE");
        command
    }

    fn ok(&self, args: &[&str]) -> Output {
        let output = self.command(args).test_output().unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn value(&self, selector: Option<&str>, expected: &[u8]) {
        let mut args = Vec::new();
        args.extend(selector);
        args.extend(["variable", "get", "TOKEN"]);
        assert_eq!(self.ok(&args).stdout, expected);
    }
}

#[test]
fn maintenance_uses_default_and_explicit_selection_wins() {
    let fixture = Fixture::new();
    let diagnostic = fixture.ok(&["doctor"]);
    assert!(String::from_utf8_lossy(&diagnostic.stdout).contains("first.lbox"));
    fixture.ok(&["doctor", "recover", "--dry-run"]);
    fixture.value(None, b"first-value\n");
    fixture.ok(&["doctor", "recover", "--dry-run"]);
    fixture.value(None, b"first-value\n");
    fixture.ok(&["second.lbox", "create"]);
    fixture.ok(&["second.lbox", "variable", "set", "TOKEN", "second-value"]);
    fixture.ok(&["second.lbox", "doctor", "recover", "--dry-run"]);
    fixture.value(Some("second.lbox"), b"second-value\n");
    fixture.value(None, b"first-value\n");

    let exported = fixture
        .command(&[
            "doctor",
            "migrate",
            "lockbox",
            "export",
            "--output",
            "default.migration",
            "--migration-password-stdin",
        ])
        .test_output_with_input(b"synthetic-migration-password\n")
        .unwrap();
    assert!(exported.status.success(), "{exported:?}");
    fixture.ok(&[
        "doctor",
        "migrate",
        "lockbox",
        "verify",
        "default.migration",
    ]);
    fixture.ok(&[
        "doctor",
        "migrate",
        "lockbox",
        "import",
        "default.migration",
        "--output",
        "imported.lbox",
    ]);
    fixture.value(Some("imported.lbox"), b"first-value\n");
}

#[test]
fn vault_operands_default_and_move_preserves_alias_and_content() {
    let fixture = Fixture::new();
    fixture.ok(&["vault", "lockbox", "remember"]);
    fixture.ok(&["vault", "lockbox", "remember"]);
    fixture.ok(&["vault", "lockbox", "alias", "set", "chosen"]);
    fixture.value(Some("a@chosen"), b"first-value\n");
    fixture.ok(&["vault", "lockbox", "move", "moved.lbox"]);
    fixture.value(None, b"first-value\n");
    fixture.value(Some("a@chosen"), b"first-value\n");
    let refused = fixture
        .command(&["vault", "lockbox", "move", "moved.lbox"])
        .test_output()
        .unwrap();
    assert!(!refused.status.success());
    fixture.value(None, b"first-value\n");
    fixture.ok(&["vault", "lockbox", "forget"]);
    fixture.ok(&["vault", "lockbox", "forget"]);
    let records = fixture.ok(&["vault", "lockbox", "list", "--format", "json"]);
    assert!(!String::from_utf8_lossy(&records.stdout).contains("moved.lbox"));
    fixture.ok(&["vault", "lockbox", "remember"]);
    fixture.value(Some("a@chosen"), b"first-value\n");
}

#[test]
fn alias_identity_and_missing_default_refusals_do_not_create_files() {
    let fixture = Fixture::new();
    fixture.ok(&["vault", "lockbox", "alias", "set", "chosen"]);
    fixture.ok(&["second.lbox", "create"]);
    // No CLI operation can replace a file while deliberately retaining stale
    // alias metadata. Simulate external replacement for the identity guard.
    std::fs::copy(
        fixture.0.path().join("second.lbox"),
        fixture.0.path().join("first.lbox"),
    )
    .unwrap();
    let rejected = fixture
        .command(&["a@chosen", "doctor", "recover", "--dry-run"])
        .test_output()
        .unwrap();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("identity changed"));
    // External deletion is likewise needed to exercise a stale default path.
    std::fs::remove_file(fixture.0.path().join("first.lbox")).unwrap();
    for args in [
        vec!["doctor", "recover", "--dry-run"],
        vec!["doctor", "recover", "--dry-run"],
        vec!["variable", "set", "TOKEN", "unexpected"],
    ] {
        assert!(!fixture
            .command(&args)
            .test_output()
            .unwrap()
            .status
            .success());
        assert!(!fixture.0.path().join("first.lbox").exists());
    }
    fixture.ok(&["vault", "lockbox", "forget"]);
    let records = fixture.ok(&["vault", "lockbox", "list", "--format", "json"]);
    assert!(!String::from_utf8_lossy(&records.stdout).contains("first.lbox"));
}
