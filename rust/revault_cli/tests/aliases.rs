mod common;
use common::CommandTestExt;
use std::process::{Command, Output};

const LBX: &str = env!("CARGO_BIN_EXE_lockbox");
const VALUE: &str = env!("CARGO_BIN_EXE_lbxv");
const EXEC: &str = env!("CARGO_BIN_EXE_lbxx");

struct Fixture {
    root: tempfile::TempDir,
    raw_key: bool,
}
impl Fixture {
    fn new() -> Self {
        Self::with_raw_key(true)
    }
    fn with_raw_key(raw_key: bool) -> Self {
        let fixture = Self {
            root: tempfile::tempdir().unwrap(),
            raw_key,
        };
        fixture.ok(&["vault", "init"]);
        if !raw_key {
            fixture.ok(&["dev.lbox", "create"]);
            fixture.ok(&["dev.lbox", "open"]);
        }
        fixture.ok(&["dev.lbox", "variable", "set", "TOKEN", "synthetic-token"]);
        fixture.ok(&["vault", "lockbox", "alias", "set", "dev", "dev.lbox"]);
        fixture
    }
    fn command(&self, binary: &str, args: &[&str]) -> Command {
        let mut command = Command::new(binary);
        command
            .current_dir(self.root.path())
            .args(args)
            .env("LOCKBOX_VAULT_DIR", self.root.path().join("vault"))
            .env("LOCKBOX_SESSION_AGENT_DIR", self.root.path().join("agent"))
            .env(
                "LOCKBOX_SESSION_AGENT_LOG",
                self.root.path().join("agent.log"),
            )
            .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
            .env("LOCKBOX_VAULT_PASSWORD", "synthetic-vault-password")
            .env_remove("LOCKBOX_PASSWORD")
            .env_remove("COMPLETE");
        if self.raw_key {
            command.env("LOCKBOX_KEY", "synthetic-content-key");
        } else {
            command.env_remove("LOCKBOX_KEY");
        }
        command
    }
    fn run(&self, binary: &str, args: &[&str]) -> Output {
        self.command(binary, args).test_output().unwrap()
    }
    fn ok(&self, args: &[&str]) -> Output {
        let output = self.run(LBX, args);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }
    fn read(&self, selector: &str) -> Vec<u8> {
        let output = self.run(VALUE, &["a@dev", selector]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    }
    fn form(&self) {
        self.ok(&[
            "a@dev",
            "form",
            "define",
            "login",
            "--name",
            "Login",
            "--field",
            "username:text",
            "--field",
            "token:secret",
        ]);
        self.ok(&[
            "a@dev",
            "form",
            "add",
            "/work/github",
            "--type",
            "login",
            "--name",
            "GitHub",
            "--set",
            "username=alice",
        ]);
        let output = self
            .command(
                LBX,
                &[
                    "a@dev",
                    "form",
                    "set",
                    "/work/github@token",
                    "--secret",
                    "--stdin",
                ],
            )
            .test_output_with_input(b"synthetic form secret\nsecond line")
            .unwrap();
        assert!(output.status.success(), "{output:?}");
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.run(LBX, &["session", "stop"]);
    }
}

#[test]
fn alias_lifecycle_persists_and_tracks_identity() {
    let f = Fixture::new();
    assert_eq!(f.read("TOKEN"), b"synthetic-token");
    f.ok(&["vault", "lockbox", "alias", "set", "dev", "dev.lbox"]);
    let listed = f.ok(&["vault", "lockbox", "alias", "list", "--format", "json"]);
    assert!(String::from_utf8_lossy(&listed.stdout).contains("dev"));
    f.ok(&["vault", "lockbox", "move", "dev.lbox", "./moved.lbox"]);
    assert_eq!(f.read("TOKEN"), b"synthetic-token");
    f.ok(&["other.lbox", "variable", "set", "TOKEN", "replacement"]);
    f.ok(&["vault", "lockbox", "alias", "set", "dev", "other.lbox"]);
    assert_eq!(f.read("TOKEN"), b"replacement");
    f.ok(&["vault", "lockbox", "alias", "remove", "dev"]);
    f.ok(&["vault", "lockbox", "alias", "remove", "dev"]);
    assert!(!f.run(VALUE, &["a@dev", "TOKEN"]).status.success());
    assert!(f.run(VALUE, &["other.lbox", "TOKEN"]).status.success());
    for name in ["a@b", "../bad", "", "bad.name"] {
        assert!(!f
            .run(
                LBX,
                &["vault", "lockbox", "alias", "set", name, "other.lbox"]
            )
            .status
            .success());
    }
}

#[test]
fn aliases_never_fall_back_and_reject_replaced_or_forgotten_targets() {
    let f = Fixture::new();
    f.ok(&["unknown.lbox", "variable", "set", "TOKEN", "file-value"]);
    f.ok(&["vault", "lockbox", "move", "unknown.lbox", "./a@unknown"]);
    assert!(!f.run(VALUE, &["a@unknown", "TOKEN"]).status.success());
    assert_eq!(
        f.run(VALUE, &["./a@unknown", "TOKEN"]).stdout,
        b"file-value"
    );
    f.ok(&[
        "other.lbox",
        "variable",
        "set",
        "TOKEN",
        "different-identity",
    ]);
    // There is deliberately no CLI operation that replaces an archive while
    // retaining a stale alias: simulate an external file replacement here.
    std::fs::rename(
        f.root.path().join("other.lbox"),
        f.root.path().join("dev.lbox"),
    )
    .unwrap();
    assert!(!f.run(VALUE, &["a@dev", "TOKEN"]).status.success());
    assert!(!f
        .run(LBX, &["a@dev", "variable", "get", "TOKEN"])
        .status
        .success());
    f.ok(&["vault", "lockbox", "forget", "dev.lbox"]);
    assert!(!f.run(VALUE, &["a@dev", "TOKEN"]).status.success());
}

#[test]
fn helpers_read_normal_and_secret_fields_and_reject_invalid_destinations() {
    let f = Fixture::new();
    f.form();
    assert_eq!(f.read("/work/github@username"), b"alice");
    assert_eq!(
        f.read("/work/github@token"),
        b"synthetic form secret\nsecond line"
    );
    let got = f.ok(&["a@dev", "form", "get", "/work/github@username"]);
    assert_eq!(got.stdout, b"alice\n");
    assert!(!f
        .run(LBX, &["a@dev", "form", "get", "/work/github", "username"])
        .status
        .success());
    assert!(!f.run(VALUE, &["a@dev", "X=TOKEN"]).status.success());
    for selectors in [
        &["BAD-NAME=TOKEN"][..],
        &["X=TOKEN", "X=TOKEN"],
        &["TOKEN", "MISSING"],
        &["TOKEN", "/work/github@missing"],
    ] {
        let mut args = vec!["a@dev"];
        args.extend_from_slice(selectors);
        args.extend(["--", "program-must-not-be-spawned"]);
        let failed = f.run(EXEC, &args);
        assert!(!failed.status.success());
        assert!(
            !String::from_utf8_lossy(&failed.stderr).contains("No such file"),
            "{failed:?}"
        );
        assert!(failed.stdout.is_empty());
    }
    let secret = f
        .command(
            LBX,
            &["a@dev", "variable", "set", "SECRET", "--secret", "--stdin"],
        )
        .test_output_with_input(b"synthetic-variable-secret")
        .unwrap();
    assert!(secret.status.success(), "{secret:?}");
    assert_eq!(f.read("SECRET"), b"synthetic-variable-secret");
    let child = std::env::current_exe().unwrap();
    let output = f
        .command(
            EXEC,
            &[
                "a@dev",
                "TOKEN",
                "/work/github@username",
                "RENAMED=/work/github@token",
                "--",
                child.to_str().unwrap(),
                "--exact",
                "environment_child",
                "--nocapture",
            ],
        )
        .env("REVAULT_ALIAS_TEST_CHILD", "1")
        .test_output()
        .unwrap();
    assert_eq!(output.status.code(), Some(37), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("environment verified"));
}

#[test]
fn environment_child() {
    if std::env::var("REVAULT_ALIAS_TEST_CHILD").as_deref() != Ok("1") {
        return;
    }
    assert_eq!(std::env::var("TOKEN").unwrap(), "synthetic-token");
    assert_eq!(std::env::var("username").unwrap(), "alice");
    assert_eq!(
        std::env::var("RENAMED").unwrap(),
        "synthetic form secret\nsecond line"
    );
    println!("environment verified");
    std::process::exit(37);
}

#[test]
fn aliases_survive_backup_and_restore_and_locked_helpers_do_not_prompt() {
    let f = Fixture::new();
    f.ok(&["vault", "backup", "saved.backup"]);
    f.ok(&["vault", "lockbox", "alias", "remove", "dev"]);
    f.ok(&["vault", "restore", "saved.backup", "--overwrite"]);
    assert_eq!(f.read("TOKEN"), b"synthetic-token");
    let closed = f
        .command(VALUE, &["a@dev", "TOKEN"])
        .env_remove("LOCKBOX_VAULT_PASSWORD")
        .env("LOCKBOX_SESSION_AGENT_DIR", f.root.path().join("no-agent"))
        .test_output()
        .unwrap();
    assert!(!closed.status.success());
    assert!(closed.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&closed.stderr).contains("passphrase:"));
}

#[test]
fn new_names_reserve_at_but_host_paths_do_not() {
    let f = Fixture::new();
    assert!(!f
        .run(LBX, &["a@dev", "variable", "set", "bad@name", "value"])
        .status
        .success());
    std::fs::write(f.root.path().join("source"), b"data").unwrap();
    assert!(!f
        .run(LBX, &["a@dev", "add", "source", "--to", "/bad@name"])
        .status
        .success());
    f.form();
    assert!(!f
        .run(
            LBX,
            &[
                "a@dev",
                "form",
                "add",
                "/bad@name",
                "--type",
                "login",
                "--name",
                "Bad"
            ]
        )
        .status
        .success());
}

#[test]
fn older_cli_preserves_additive_alias_records() {
    let Ok(baseline) = std::env::var("REVAULT_ALIAS_BASELINE_BIN") else {
        return;
    };
    let f = Fixture::new();
    for args in [
        vec!["vault", "lockbox", "list"],
        vec!["vault", "profile", "create", "older-writer"],
        vec!["vault", "backup", "older.backup"],
        vec!["vault", "restore", "older.backup", "--overwrite"],
    ] {
        let output = f.run(&baseline, &args);
        assert!(output.status.success(), "{args:?}: {output:?}");
    }
    assert_eq!(f.read("TOKEN"), b"synthetic-token");
    std::fs::write(f.root.path().join("historical-input"), b"historical-file").unwrap();
    let old_file = f.run(
        &baseline,
        &["dev.lbox", "add", "historical-input", "--to", "/old@file"],
    );
    assert!(old_file.status.success(), "{old_file:?}");
    assert_eq!(
        f.ok(&["dev.lbox", "cat", "/old@file"]).stdout,
        b"historical-file"
    );
}

#[test]
fn completion_exposes_names_only_and_retains_assignment_destination() {
    let f = Fixture::with_raw_key(false);
    f.form();
    f.ok(&["a@dev", "open"]);
    for (binary, words, expected) in [
        (LBX, vec!["lockbox", "a@"], "a@dev"),
        (VALUE, vec!["lbxv", "a@dev", "TO"], "TOKEN"),
        (
            VALUE,
            vec!["lbxv", "a@dev", "/work/github@t"],
            "/work/github@token",
        ),
        (
            EXEC,
            vec!["lbxx", "a@dev", "GH=/work/github@t"],
            "GH=/work/github@token",
        ),
        (
            LBX,
            vec!["lockbox", "a@dev", "form", "get", "/work/github@u"],
            "/work/github@username",
        ),
    ] {
        let mut args = vec!["--"];
        args.extend_from_slice(&words);
        let output = f
            .command(binary, &args)
            .env("COMPLETE", "bash")
            .env("_CLAP_COMPLETE_INDEX", (words.len() - 1).to_string())
            .env("_CLAP_COMPLETE_COMP_TYPE", "9")
            .env("_CLAP_COMPLETE_SPACE", "true")
            .test_output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains(expected), "{words:?}: {text:?}");
        assert!(!text.contains("synthetic"));
        assert!(!text.contains("alice"));
    }
    for binary in [VALUE, EXEC] {
        let output = f.run(binary, &["completion", "generate", "--shell", "bash"]);
        assert!(output.status.success(), "{output:?}");
        assert!(String::from_utf8_lossy(&output.stdout).contains("_clap_complete_"));
    }
}
