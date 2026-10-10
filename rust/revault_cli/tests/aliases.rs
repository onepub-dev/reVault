mod common;
use common::CommandTestExt;
use std::process::{Command, Output};

const LBX: &str = env!("CARGO_BIN_EXE_lockbox");
const VALUE: &str = env!("CARGO_BIN_EXE_lbxv");
const EXEC: &str = env!("CARGO_BIN_EXE_lbxx");

#[test]
fn bare_names_select_aliases_local_files_and_reject_conflicts() {
    let f = Fixture::new();
    // dev and its local .lbox file have the same identity.
    f.ok(&["dev", "variables", "set", "TOKEN", "same-identity"]);
    assert_eq!(f.read("TOKEN"), b"same-identity");
    f.ok(&[
        "vault",
        "lockboxes",
        "aliases",
        "set",
        "remote",
        "./dev.lbox",
    ]);
    f.ok(&["remote", "variables", "set", "TOKEN", "alias-only"]);
    assert_eq!(f.read("TOKEN"), b"alias-only");
    f.ok(&["session", "default", "remote"]);
    let default = f.run(VALUE, &["TOKEN"]);
    assert!(default.status.success(), "{default:?}");
    assert_eq!(default.stdout, b"alias-only");
    let bare_helper = f.run(VALUE, &["remote", "TOKEN"]);
    assert!(bare_helper.status.success(), "{bare_helper:?}");
    assert_eq!(bare_helper.stdout, b"alias-only");

    f.ok(&["./remote.lbox", "create"]);
    f.ok(&["./remote.lbox", "variables", "set", "TOKEN", "local-only"]);
    for args in [
        vec!["remote", "variables", "set", "TOKEN", "wrong"],
        vec!["remote", "create"],
        vec!["session", "default", "remote"],
    ] {
        let refused = f.run(LBX, &args);
        assert!(!refused.status.success(), "{refused:?}");
        let error = String::from_utf8_lossy(&refused.stderr);
        for guidance in ["ambiguous", "a@remote", "./", ".lbox"] {
            assert!(error.contains(guidance), "{refused:?}");
        }
    }
    for (binary, args) in [
        (VALUE, vec!["remote", "TOKEN"]),
        (
            EXEC,
            vec!["--lockbox", "remote", "TOKEN", "--", "must-not-run"],
        ),
        (EXEC, vec!["remote", "TOKEN", "--", "must-not-run"]),
    ] {
        let refused = f.run(binary, &args);
        assert!(!refused.status.success(), "{refused:?}");
        assert!(
            String::from_utf8_lossy(&refused.stderr).contains("ambiguous"),
            "{refused:?}"
        );
    }
    assert_eq!(f.read("TOKEN"), b"alias-only");
    for selector in ["./remote.lbox", "remote.lbox"] {
        let local = f.run(VALUE, &[selector, "TOKEN"]);
        assert!(local.status.success(), "{local:?}");
        assert_eq!(local.stdout, b"local-only");
    }
    f.ok(&["vault", "lockboxes", "aliases", "remove", "remote"]);
    f.ok(&["remote", "variables", "set", "TOKEN", "local-updated"]);
    let local = f.run(VALUE, &["remote.lbox", "TOKEN"]);
    assert!(local.status.success(), "{local:?}");
    assert_eq!(local.stdout, b"local-updated");
}

#[test]
fn bare_creation_refuses_existing_targets_and_explicit_filenames_bypass_aliases() {
    let f = Fixture::new();
    f.ok(&["fresh", "create"]);
    f.ok(&["fresh", "variables", "set", "TOKEN", "created-value"]);
    let refused = f.run(LBX, &["fresh", "create"]);
    assert!(!refused.status.success(), "{refused:?}");
    let fresh = f.run(VALUE, &["fresh.lbox", "TOKEN"]);
    assert!(fresh.status.success(), "{fresh:?}");
    assert_eq!(fresh.stdout, b"created-value");
    let unknown = f.run(LBX, &["unknown", "variables", "set", "TOKEN", "wrong"]);
    assert!(!unknown.status.success(), "{unknown:?}");
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown lockbox alias"));
    assert!(!f.root.path().join("unknown.lbox").exists());
    f.ok(&["./with.dot", "create", "--alias", "dot-file"]);
    f.ok(&["with.dot", "variables", "set", "TOKEN", "dot-file"]);
    let dotted = f.run(VALUE, &["with.dot", "TOKEN"]);
    assert!(dotted.status.success(), "{dotted:?}");
    assert_eq!(dotted.stdout, b"dot-file");
    f.ok(&[
        "vault",
        "lockboxes",
        "move",
        "./fresh.lbox",
        "./extensionless",
    ]);
    f.ok(&["extensionless", "variables", "set", "TOKEN", "exact-file"]);
    let exact = f.run(VALUE, &["./extensionless", "TOKEN"]);
    assert!(exact.status.success(), "{exact:?}");
    assert_eq!(exact.stdout, b"exact-file");
}

#[test]
fn bare_alias_same_identity_accepts_local_copy_and_missing_remembered_path() {
    let f = Fixture::new();
    f.ok(&["vault", "lockboxes", "aliases", "set", "copy", "./dev.lbox"]);
    // Public CLI moves refresh remembered locations. External copies/removal
    // are required to exercise identical archives at different paths and a
    // stale remembered location without changing the encrypted Vault records.
    std::fs::copy(
        f.root.path().join("dev.lbox"),
        f.root.path().join("copy.lbox"),
    )
    .unwrap();
    let copy = f.run(VALUE, &["copy", "TOKEN"]);
    assert!(copy.status.success(), "{copy:?}");
    assert_eq!(copy.stdout, b"synthetic-token");
    std::fs::remove_file(f.root.path().join("dev.lbox")).unwrap();
    let copy = f.run(VALUE, &["copy", "TOKEN"]);
    assert!(copy.status.success(), "{copy:?}");
    assert_eq!(copy.stdout, b"synthetic-token");
    f.ok(&["copy", "variables", "set", "TOKEN", "local-copy"]);
    let local = f.run(VALUE, &["./copy.lbox", "TOKEN"]);
    assert!(local.status.success(), "{local:?}");
    assert_eq!(local.stdout, b"local-copy");
}

#[test]
fn bare_local_selection_checks_vault_availability_but_explicit_paths_do_not() {
    let f = Fixture::new();
    for (binary, args) in [
        (LBX, vec!["dev", "variables", "get", "TOKEN"]),
        (VALUE, vec!["dev", "TOKEN"]),
    ] {
        let locked = f
            .command(binary, &args)
            .env("LOCKBOX_VAULT_PASSWORD", "wrong-password")
            .test_output()
            .unwrap();
        assert!(!locked.status.success(), "{locked:?}");
        assert!(locked.stdout.is_empty(), "{locked:?}");
    }
    let explicit = f
        .command(VALUE, &["./dev.lbox", "TOKEN"])
        .env("LOCKBOX_VAULT_PASSWORD", "wrong-password")
        .test_output()
        .unwrap();
    assert!(explicit.status.success(), "{explicit:?}");
    assert_eq!(explicit.stdout, b"synthetic-token");
    // Point at an uninitialized Vault using the public configuration option.
    let headless = f
        .command(VALUE, &["dev", "TOKEN"])
        .env("LOCKBOX_VAULT_DIR", f.root.path().join("no-vault"))
        .env_remove("LOCKBOX_VAULT_PASSWORD")
        .test_output()
        .unwrap();
    assert!(headless.status.success(), "{headless:?}");
    assert_eq!(headless.stdout, b"synthetic-token");
}

#[test]
fn exec_helper_accepts_bare_alias_and_explicit_bare_selection() {
    let f = Fixture::new();
    f.form();
    for selections in [
        vec![
            "dev",
            "TOKEN",
            "/work/github@username",
            "RENAMED=/work/github@token",
        ],
        vec![
            "--lockbox",
            "dev",
            "TOKEN",
            "/work/github@username",
            "RENAMED=/work/github@token",
        ],
    ] {
        assert_helper_environment(&f, &selections);
    }
}

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
        fixture.ok(&["dev.lbox", "create"]);
        if !raw_key {
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
            .env("LOCKBOX_MIGRATION_PASSWORD", "synthetic-migration-password")
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
    fn input(&self, binary: &str, args: &[&str], input: &[u8]) -> Output {
        self.command(binary, args)
            .test_output_with_input(input)
            .unwrap()
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
    f.ok(&["other.lbox", "create"]);
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
fn session_default_alias_reads_follow_lockbox_move() {
    let f = Fixture::new();
    f.ok(&[
        "dev.lbox",
        "variable",
        "set",
        "DEFAULT_ONLY",
        "default-value",
    ]);
    let default = f.ok(&["session", "default", "a@dev"]);
    assert!(String::from_utf8_lossy(&default.stdout).contains("Default lockbox:"));
    let before = f.ok(&["variable", "get", "DEFAULT_ONLY"]);
    assert_eq!(before.stdout, b"default-value\n");

    f.ok(&["vault", "lockbox", "move", "a@dev", "./moved.lbox"]);
    assert_eq!(f.read("TOKEN"), b"synthetic-token");
    let after = f.ok(&["variable", "get", "DEFAULT_ONLY"]);
    assert_eq!(after.stdout, b"default-value\n");
    assert!(f.root.path().join("moved.lbox").is_file());
}

#[test]
fn alias_forget_handles_externally_removed_file_and_move_refusal_preserves_destination() {
    let f = Fixture::new();
    f.ok(&["vault", "lockbox", "remember", "a@dev"]);
    let destination = f.root.path().join("occupied.lbox");
    std::fs::write(&destination, b"keep these destination bytes").unwrap();
    let refused = f.run(LBX, &["vault", "lockbox", "move", "a@dev", "occupied.lbox"]);
    assert!(!refused.status.success(), "{refused:?}");
    assert_eq!(
        std::fs::read(&destination).unwrap(),
        b"keep these destination bytes"
    );
    assert_eq!(f.read("TOKEN"), b"synthetic-token");

    // The CLI has no operation that removes an archive behind a remembered
    // path. Simulate that external filesystem change, then exercise public forget.
    std::fs::remove_file(f.root.path().join("dev.lbox")).unwrap();
    let forgotten = f.ok(&["vault", "lockbox", "forget", "a@dev"]);
    assert!(String::from_utf8_lossy(&forgotten.stdout).contains("Forgot known lockbox"));
    assert!(!f
        .run(LBX, &["vault", "lockbox", "forget", "a@dev"])
        .status
        .success());
}

#[test]
fn alias_destinations_are_refused_without_overwriting_existing_files() {
    let f = Fixture::new();
    let destination = f.root.path().join("existing.lbox");
    f.ok(&["existing.lbox", "create"]);
    f.ok(&[
        "existing.lbox",
        "variable",
        "set",
        "SENTINEL",
        "destination sentinel",
    ]);
    f.ok(&[
        "vault",
        "lockbox",
        "alias",
        "set",
        "existing",
        "existing.lbox",
    ]);
    let destination_contents = std::fs::read(&destination).unwrap();

    let moved = f.run(LBX, &["vault", "lockbox", "move", "a@dev", "a@existing"]);
    assert!(!moved.status.success(), "{moved:?}");
    assert!(String::from_utf8_lossy(&moved.stderr).contains("destination already exists"));
    assert_eq!(std::fs::read(&destination).unwrap(), destination_contents);
    assert_destination_sentinel(&f);

    let recovered = f.run(
        LBX,
        &["a@dev", "doctor", "recover", "--output", "a@existing"],
    );
    assert!(!recovered.status.success(), "{recovered:?}");
    assert!(
        String::from_utf8_lossy(&recovered.stderr).contains("already exists"),
        "{recovered:?}"
    );
    assert!(!f.root.path().join("a@existing").exists());
    assert_eq!(std::fs::read(&destination).unwrap(), destination_contents);
    assert_destination_sentinel(&f);
    assert_eq!(f.read("TOKEN"), b"synthetic-token");

    let artifact = f.root.path().join("archive.migration");
    let exported = f.input(
        LBX,
        &[
            "a@dev",
            "doctor",
            "migrate",
            "lockbox",
            "export",
            "--output",
            artifact.to_str().unwrap(),
            "--migration-password-stdin",
        ],
        b"synthetic-migration-password\n",
    );
    assert!(exported.status.success(), "{exported:?}");
    let refused = f.run(
        LBX,
        &[
            "doctor",
            "migrate",
            "lockbox",
            "import",
            artifact.to_str().unwrap(),
            "--output",
            "a@existing",
        ],
    );
    assert!(!refused.status.success(), "{refused:?}");
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains("destination already exists"),
        "{refused:?}"
    );
    assert!(!f.root.path().join("a@existing").exists());
    assert_destination_sentinel(&f);

    let direct_output = f.root.path().join("extracted-value");
    std::fs::write(&direct_output, b"keep direct output").unwrap();
    let refused = f.run(
        LBX,
        &[
            "a@dev",
            "variable",
            "get",
            "TOKEN",
            "--output",
            direct_output.to_str().unwrap(),
        ],
    );
    assert!(!refused.status.success(), "{refused:?}");
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains("already exists"),
        "{refused:?}"
    );
    assert_eq!(std::fs::read(direct_output).unwrap(), b"keep direct output");
}

fn assert_destination_sentinel(f: &Fixture) {
    let sentinel = f.run(VALUE, &["existing.lbox", "SENTINEL"]);
    assert!(sentinel.status.success(), "{sentinel:?}");
    assert_eq!(sentinel.stdout, b"destination sentinel");
}

#[test]
fn lockbox_alias_creation_refuses_existing_target_including_extensionless_name() {
    let f = Fixture::new();
    let original = std::fs::read(f.root.path().join("dev.lbox")).unwrap();
    let refused = f.run(LBX, &["dev.lbox", "create"]);
    assert!(!refused.status.success(), "{refused:?}");
    assert_eq!(
        std::fs::read(f.root.path().join("dev.lbox")).unwrap(),
        original
    );

    f.ok(&["extensionless.lbox", "create"]);
    std::fs::rename(
        f.root.path().join("extensionless.lbox"),
        f.root.path().join("extensionless"),
    )
    .unwrap();
    f.ok(&[
        "vault",
        "lockbox",
        "alias",
        "set",
        "extless",
        "extensionless",
    ]);
    let target = std::fs::read(f.root.path().join("extensionless")).unwrap();
    let refused = f.run(LBX, &["a@extless", "create"]);
    assert!(!refused.status.success(), "{refused:?}");
    assert_eq!(
        std::fs::read(f.root.path().join("extensionless")).unwrap(),
        target
    );
    assert!(!f.root.path().join("extensionless.lbox").exists());
}

#[test]
fn representative_archive_routes_accept_alias_selector() {
    let f = Fixture::new();
    f.ok(&["a@dev", "doctor"]);
    f.ok(&["a@dev", "list"]);
    f.ok(&["a@dev", "access", "list"]);
    let variables = f.ok(&["a@dev", "variable", "list"]);
    assert!(String::from_utf8_lossy(&variables.stdout).contains("TOKEN"));
    f.ok(&["a@dev", "form", "list"]);
}

#[test]
fn migration_alias_selector_exports_and_imports_with_independent_readback() {
    let f = Fixture::new();
    let artifact = f.root.path().join("archive.migration");
    let exported = f.input(
        LBX,
        &[
            "a@dev",
            "doctor",
            "migrate",
            "lockbox",
            "export",
            "--output",
            artifact.to_str().unwrap(),
            "--migration-password-stdin",
        ],
        b"synthetic-migration-password\n",
    );
    assert!(exported.status.success(), "{exported:?}");
    f.ok(&[
        "doctor",
        "migrate",
        "lockbox",
        "verify",
        artifact.to_str().unwrap(),
    ]);

    let explicit_artifact = f.root.path().join("explicit-source.migration");
    let exported = f.input(
        LBX,
        &[
            "doctor",
            "migrate",
            "lockbox",
            "export",
            "a@dev",
            "--output",
            explicit_artifact.to_str().unwrap(),
            "--migration-password-stdin",
        ],
        b"synthetic-migration-password\n",
    );
    assert!(exported.status.success(), "{exported:?}");
    f.ok(&[
        "doctor",
        "migrate",
        "lockbox",
        "verify",
        explicit_artifact.to_str().unwrap(),
    ]);

    let imported = f.root.path().join("imported.lbox");
    let result = f.input(
        LBX,
        &[
            "doctor",
            "migrate",
            "lockbox",
            "import",
            explicit_artifact.to_str().unwrap(),
            "--output",
            imported.to_str().unwrap(),
        ],
        b"",
    );
    assert!(result.status.success(), "{result:?}");
    let reread = f.run(VALUE, &[imported.to_str().unwrap(), "TOKEN"]);
    assert!(reread.status.success(), "{reread:?}");
    assert_eq!(reread.stdout, b"synthetic-token");
}

#[test]
fn aliases_never_fall_back_and_reject_replaced_or_forgotten_targets() {
    let f = Fixture::new();
    f.ok(&["unknown.lbox", "create", "--alias", "literal-file"]);
    f.ok(&["unknown.lbox", "variable", "set", "TOKEN", "file-value"]);
    f.ok(&["vault", "lockbox", "move", "unknown.lbox", "./a@unknown"]);
    assert!(!f.run(VALUE, &["a@unknown", "TOKEN"]).status.success());
    assert_eq!(
        f.run(VALUE, &["./a@unknown", "TOKEN"]).stdout,
        b"file-value"
    );
    f.ok(&["other.lbox", "create"]);
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
fn helper_help_is_useful_without_a_vault_or_session() {
    let root = tempfile::tempdir().unwrap();
    for binary in [VALUE, EXEC] {
        for args in [vec![], vec!["--help"]] {
            let output = Command::new(binary)
                .args(args)
                .env("LOCKBOX_VAULT_DIR", root.path().join("no-vault"))
                .env("LOCKBOX_SESSION_AGENT_DIR", root.path().join("no-session"))
                .env_remove("COMPLETE")
                .test_output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            let help = String::from_utf8_lossy(&output.stdout);
            for expected in [
                "Usage:",
                "session default",
                "a@NAME",
                "--lockbox NAME",
                "/full/path/dev.lbox",
                "never prompt",
                "Examples:",
            ] {
                assert!(help.contains(expected), "missing {expected}: {help}");
            }
            assert!(!root.path().join("no-vault").exists());
            assert!(!root.path().join("no-session").exists());
        }
    }
}

#[test]
fn helpers_use_defaults_and_explicit_paths_or_aliases_override_them() {
    let f = Fixture::new();
    f.form();
    f.ok(&["other.lbox", "create"]);
    f.ok(&["other.lbox", "variable", "set", "TOKEN", "other-value"]);
    f.ok(&["session", "default", "other.lbox"]);
    let default = f.run(VALUE, &["TOKEN"]);
    assert!(default.status.success(), "{default:?}");
    assert_eq!(default.stdout, b"other-value");
    let path = f.root.path().join("dev.lbox");
    for explicit in ["a@dev", path.to_str().unwrap()] {
        let value = f.run(VALUE, &[explicit, "TOKEN"]);
        assert!(value.status.success(), "{value:?}");
        assert_eq!(value.stdout, b"synthetic-token");
        assert_helper_environment(
            &f,
            &[
                explicit,
                "TOKEN",
                "/work/github@username",
                "RENAMED=/work/github@token",
            ],
        );
    }
    f.ok(&["session", "default", "a@dev"]);
    let secret = f.input(
        LBX,
        &["a@dev", "variable", "set", "SECRET", "--secret", "--stdin"],
        b"synthetic variable secret\nsecond line",
    );
    assert!(secret.status.success(), "{secret:?}");
    for (selector, expected) in [
        ("TOKEN", &b"synthetic-token"[..]),
        ("SECRET", &b"synthetic variable secret\nsecond line"[..]),
        (
            "/work/github@token",
            &b"synthetic form secret\nsecond line"[..],
        ),
    ] {
        let value = f.run(VALUE, &[selector]);
        assert!(value.status.success(), "{value:?}");
        assert_eq!(value.stdout, expected);
        assert!(value.stderr.is_empty(), "{value:?}");
    }
    assert_helper_environment(
        &f,
        &[
            "TOKEN",
            "/work/github@username",
            "RENAMED=/work/github@token",
        ],
    );
    // Archive variable paths must not be confused with absolute host paths.
    f.ok(&[
        "a@dev",
        "variable",
        "set",
        "/group/TOKEN",
        "synthetic-token",
    ]);
    assert_helper_environment(
        &f,
        &[
            "/group/TOKEN",
            "/work/github@username",
            "RENAMED=/work/github@token",
        ],
    );
    // An assignment ending in .lbox is parsed as a selector, then rejected as
    // an invalid field identifier, rather than treated as a host path.
    let missing_field = f.run(EXEC, &["X=/work/github@missing.lbox", "--", "must-not-run"]);
    assert!(!missing_field.status.success(), "{missing_field:?}");
    assert!(
        String::from_utf8_lossy(&missing_field.stderr).contains("invalid form field id"),
        "{missing_field:?}"
    );
    f.ok(&["vault", "lockbox", "move", "a@dev", "./extensionless"]);
    for prefix in [
        vec!["./extensionless"],
        vec!["--lockbox", "./extensionless"],
    ] {
        let mut args = prefix.clone();
        args.push("TOKEN");
        let value = f.run(VALUE, &args);
        assert!(value.status.success(), "{value:?}");
        assert_eq!(value.stdout, b"synthetic-token");
        let mut selections = prefix;
        selections.extend([
            "TOKEN",
            "/work/github@username",
            "RENAMED=/work/github@token",
        ]);
        assert_helper_environment(&f, &selections);
    }
    assert_eq!(f.run(VALUE, &["TOKEN"]).stdout, b"synthetic-token");
}

fn missing_lockbox_error(output: &Output) -> bool {
    assert!(!output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty());
    String::from_utf8_lossy(&output.stderr).contains("missing lockbox")
}

fn assert_helper_environment(f: &Fixture, selections: &[&str]) {
    let child = std::env::current_exe().unwrap();
    let mut args = selections.to_vec();
    args.extend([
        "--",
        child.to_str().unwrap(),
        "--exact",
        "environment_child",
        "--nocapture",
    ]);
    let output = f
        .command(EXEC, &args)
        .env("REVAULT_ALIAS_TEST_CHILD", "1")
        .test_output()
        .unwrap();
    assert_eq!(output.status.code(), Some(37), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("environment verified"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("synthetic"));
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn helpers_reject_missing_defaults_and_invalid_explicit_targets_without_fallback() {
    let f = Fixture::new();
    for (binary, args) in [
        (VALUE, vec!["TOKEN"]),
        (EXEC, vec!["TOKEN", "OTHER", "--", "must-not-run"]),
    ] {
        let output = f.run(binary, &args);
        assert!(missing_lockbox_error(&output), "{output:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("lbx session default"));
    }
    f.ok(&["session", "default", "a@dev"]);
    let absent = f.root.path().join("missing.lbox");
    for explicit in ["a@missing", absent.to_str().unwrap()] {
        for (binary, args) in [
            (VALUE, vec![explicit, "TOKEN"]),
            (EXEC, vec![explicit, "TOKEN", "--", "must-not-run"]),
        ] {
            let failed = f.run(binary, &args);
            assert!(!failed.status.success(), "{failed:?}");
            assert!(failed.stdout.is_empty());
            assert!(!String::from_utf8_lossy(&failed.stderr).contains("must-not-run"));
            if explicit == absent.to_str().unwrap() {
                assert_missing_target_guidance(&failed, explicit);
            } else {
                assert!(
                    !String::from_utf8_lossy(&failed.stderr).contains("missing lockbox"),
                    "{failed:?}"
                );
            }
        }
    }
    for (binary, args) in [
        (VALUE, vec!["a@dev"]),
        (EXEC, vec!["a@dev", "--", "must-not-run"]),
    ] {
        let failed = f.run(binary, &args);
        assert!(!failed.status.success(), "{failed:?}");
        assert!(
            String::from_utf8_lossy(&failed.stderr).contains("expected a selector"),
            "{failed:?}"
        );
    }
    let too_many = f.run(VALUE, &["--lockbox", "a@dev", "dev.lbox", "TOKEN"]);
    assert!(!too_many.status.success(), "{too_many:?}");
    assert!(
        String::from_utf8_lossy(&too_many.stderr).contains("expected a selector"),
        "{too_many:?}"
    );
}

#[test]
fn mutations_never_create_missing_explicit_alias_or_default_lockboxes() {
    for selector in [Some("dev.lbox"), Some("a@dev"), None] {
        let f = Fixture::new();
        f.ok(&["session", "default", "a@dev"]);
        std::fs::write(f.root.path().join("payload.txt"), b"preserve source bytes").unwrap();
        // No CLI command removes the host archive while retaining its alias and
        // session default. Simulate external deletion to test stale references.
        std::fs::remove_file(f.root.path().join("dev.lbox")).unwrap();
        for operation in [
            vec!["variable", "set", "TOKEN", "must not create"],
            vec!["add", "payload.txt", "--to", "/payload.txt"],
            vec!["form", "define", "login", "--field", "username:text"],
            vec![
                "form",
                "add",
                "/login",
                "--type",
                "login",
                "--name",
                "Login",
                "--set",
                "username=alice",
            ],
        ] {
            let mut args = selector.into_iter().collect::<Vec<_>>();
            args.extend(operation);
            let failed = f.run(LBX, &args);
            assert!(!failed.status.success(), "{args:?}: {failed:?}");
            assert!(
                !String::from_utf8_lossy(&failed.stdout).contains("created"),
                "{failed:?}"
            );
            assert_missing_target_guidance(&failed, "dev.lbox");
            // A separate public CLI read must also refuse the missing archive.
            let verify = f.run(LBX, &["dev.lbox", "list"]);
            assert!(!verify.status.success(), "{verify:?}");
            // The CLI cannot enumerate absent host files; check that the failed
            // write did not leave an empty archive or a literal alias filename.
            assert!(!f.root.path().join("dev.lbox").exists());
            assert!(!f.root.path().join("a@dev").exists());
            assert_eq!(
                std::fs::read(f.root.path().join("payload.txt")).unwrap(),
                b"preserve source bytes"
            );
        }
        for (binary, operation) in [
            (VALUE, vec!["TOKEN"]),
            (EXEC, vec!["TOKEN", "--", "must-not-run"]),
        ] {
            let mut args = selector.into_iter().collect::<Vec<_>>();
            args.extend(operation);
            let failed = f.run(binary, &args);
            assert!(!failed.status.success(), "{failed:?}");
            assert_missing_target_guidance(&failed, "dev.lbox");
            assert!(failed.stdout.is_empty());
        }
    }
}

fn assert_missing_target_guidance(output: &Output, path: &str) {
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("lockbox not found:"), "{output:?}");
    assert!(error.contains(path), "{output:?}");
    assert!(
        error.contains("lbx ") && error.contains(" create"),
        "{output:?}"
    );
    assert!(!error.contains("os error 2"), "{output:?}");
}

#[test]
fn default_helpers_use_cached_access_and_never_prompt_when_closed() {
    let f = Fixture::with_raw_key(false);
    f.form();
    f.ok(&["session", "default", "a@dev"]);
    assert_eq!(f.run(VALUE, &["TOKEN"]).stdout, b"synthetic-token");
    assert_helper_environment(
        &f,
        &[
            "TOKEN",
            "/work/github@username",
            "RENAMED=/work/github@token",
        ],
    );
    f.ok(&["a@dev", "close"]);
    // Closing clears the default; setting it again does not unlock the archive.
    f.ok(&["session", "default", "a@dev"]);
    for (binary, args) in [
        (VALUE, vec!["TOKEN"]),
        (EXEC, vec!["TOKEN", "--", "must-not-run"]),
    ] {
        let output = f
            .command(binary, &args)
            .env_remove("LOCKBOX_VAULT_PASSWORD")
            .test_output_with_input(b"must not be consumed as a password\n")
            .unwrap();
        assert!(!output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("closed"), "{output:?}");
        assert!(!error.contains("passphrase:"), "{output:?}");
        assert!(!error.contains("must-not-run"), "{output:?}");
    }
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
    f.ok(&["session", "default", "a@dev"]);
    for (binary, words, expected) in [
        (LBX, vec!["lockbox", "a@"], "a@dev"),
        (VALUE, vec!["lbxv", "a@dev", "TO"], "TOKEN"),
        (VALUE, vec!["lbxv", "TO"], "TOKEN"),
        (VALUE, vec!["lbxv", "a@d"], "a@dev"),
        (VALUE, vec!["lbxv", "--lockbox", "a@dev", "TO"], "TOKEN"),
        (VALUE, vec!["lbxv", "--lockbox=a@dev", "TO"], "TOKEN"),
        (
            EXEC,
            vec!["lbxx", "TOKEN", "GH=/work/github@t"],
            "GH=/work/github@token",
        ),
        (VALUE, vec!["lbxv", "/work/github@t"], "/work/github@token"),
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
        (
            LBX,
            vec!["lockbox", "vault", "lockbox", "move", "a@d"],
            "a@dev",
        ),
        (
            LBX,
            vec!["lockbox", "vault", "lockbox", "move", "a@dev", "a@"],
            "a@dev",
        ),
        (LBX, vec!["lockbox", "session", "default", "a@d"], "a@dev"),
        (
            LBX,
            vec!["lockbox", "doctor", "migrate", "lockbox", "export", "a@d"],
            "a@dev",
        ),
        (
            LBX,
            vec![
                "lockbox",
                "doctor",
                "migrate",
                "lockbox",
                "import",
                "archive.migration",
                "--output",
                "a@",
            ],
            "a@dev",
        ),
        (
            LBX,
            vec!["lockbox", "a@dev", "doctor", "recover", "--output", "a@"],
            "a@dev",
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

#[test]
fn create_registers_derived_and_explicit_aliases_with_readback() {
    let f = Fixture::new();
    std::fs::create_dir(f.root.path().join("directory.with.dots")).unwrap();
    for (path, alias, options) in [
        (
            "directory.with.dots/project-secrets.lbox",
            "project-secrets",
            vec![],
        ),
        (
            "directory.with.dots/project.prod.lbox",
            "production",
            vec!["--alias", "production"],
        ),
        ("extensionless", "extensionless", vec![]),
        (
            "configured.lbox",
            "configured-alias",
            vec!["--alias", "configured-alias", "--compression", "none"],
        ),
    ] {
        let mut args = vec![path, "create"];
        args.extend(options);
        f.ok(&args);
        let selector = format!("a@{alias}");
        f.ok(&[
            &selector,
            "variable",
            "set",
            "CREATED",
            "synthetic-created-value",
        ]);
        let value = f.run(VALUE, &[&selector, "CREATED"]);
        assert!(value.status.success(), "{value:?}");
        assert_eq!(value.stdout, b"synthetic-created-value");
        let disk_path = if path == "extensionless" {
            "extensionless.lbox"
        } else {
            path
        };
        let direct = f.run(VALUE, &[disk_path, "CREATED"]);
        assert!(direct.status.success(), "{direct:?}");
        assert_eq!(direct.stdout, b"synthetic-created-value");
    }
}

#[test]
fn create_alias_collisions_warn_and_preserve_both_lockboxes() {
    let f = Fixture::new();
    std::fs::create_dir(f.root.path().join("nested")).unwrap();
    for (path, options) in [
        ("explicit.lbox", vec!["--alias", "dev"]),
        ("nested/dev.lbox", vec![]),
        (
            "configured.lbox",
            vec!["--alias", "dev", "--compression", "none"],
        ),
    ] {
        let mut args = vec![path, "create"];
        args.extend(options);
        let output = f.ok(&args);
        let warning = String::from_utf8_lossy(&output.stderr);
        assert!(warning.contains("no alias was created"), "{output:?}");
        assert!(warning.contains("'dev' already exists"), "{output:?}");
        assert_eq!(f.read("TOKEN"), b"synthetic-token");
        f.ok(&[path, "variable", "set", "TOKEN", "new-lockbox-value"]);
        let value = f.run(VALUE, &[path, "TOKEN"]);
        assert!(value.status.success(), "{value:?}");
        assert_eq!(value.stdout, b"new-lockbox-value");
        assert_eq!(f.read("TOKEN"), b"synthetic-token");
    }
}

#[test]
fn create_rejects_invalid_aliases_and_does_not_register_failed_creations() {
    let f = Fixture::new();
    let before = f
        .ok(&["vault", "lockbox", "alias", "list", "--format", "json"])
        .stdout;
    for name in [
        "",
        "a@bad",
        "../bad",
        "bad.name",
        "bad name",
        &"a".repeat(129),
    ] {
        let existing = f.run(LBX, &["vault", "lockbox", "alias", "set", name, "dev.lbox"]);
        let failed = f.run(LBX, &["invalid.lbox", "create", "--alias", name]);
        assert!(!failed.status.success(), "{failed:?}");
        assert_eq!(failed.stderr, existing.stderr);
        assert!(!f.root.path().join("invalid.lbox").exists());
    }
    let dotted = f.run(LBX, &["project.prod.lbox", "create"]);
    assert!(!dotted.status.success(), "{dotted:?}");
    assert!(!f.root.path().join("project.prod.lbox").exists());
    for args in [
        vec!["dev.lbox", "create", "--alias", "should-not-exist"],
        vec!["dev.lbox/new.lbox", "create", "--alias", "not-a-directory"],
        vec![
            "bad-options.lbox",
            "create",
            "--alias",
            "bad-options",
            "--compression",
            "none",
            "--compression-level",
            "3",
        ],
        vec![
            "missing-contact.lbox",
            "create",
            "--alias",
            "missing-contact",
            "--for",
            "unknown-contact",
        ],
    ] {
        let failed = f.run(LBX, &args);
        assert!(!failed.status.success(), "{args:?}: {failed:?}");
    }
    let after = f
        .ok(&["vault", "lockbox", "alias", "list", "--format", "json"])
        .stdout;
    assert_eq!(before, after);
    assert_eq!(f.read("TOKEN"), b"synthetic-token");
}

#[test]
fn create_validates_options_and_vault_before_creating_an_archive() {
    let f = Fixture {
        root: tempfile::tempdir().unwrap(),
        raw_key: false,
    };
    for (options, diagnostic) in [
        (
            vec!["--compression", "none", "--compression-level", "3"],
            "--compression-level requires --compression zstd",
        ),
        (
            vec!["--encryption", "none", "--password"],
            "--encryption none does not accept decryption credentials",
        ),
    ] {
        let mut args = vec!["new.lbox", "create"];
        args.extend(options);
        let result = f.run(LBX, &args);
        assert!(!result.status.success(), "{result:?}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(diagnostic),
            "{result:?}"
        );
        assert!(!f.root.path().join("new.lbox").exists());
    }
    // An explicit alias requests Vault access, including initialization when
    // the Vault credentials are supplied.
    f.ok(&[
        "plain.lbox",
        "create",
        "--encryption",
        "none",
        "--signing",
        "none",
        "--alias",
        "plain",
    ]);
    f.ok(&[
        "a@plain",
        "variable",
        "set",
        "VALUE",
        "synthetic-plain-value",
    ]);
    let value = f.run(VALUE, &["a@plain", "VALUE"]);
    assert!(value.status.success(), "{value:?}");
    assert_eq!(value.stdout, b"synthetic-plain-value");
    let failed = f
        .command(
            LBX,
            &[
                "new.lbox",
                "create",
                "--encryption",
                "none",
                "--signing",
                "none",
                "--alias",
                "new",
            ],
        )
        .env("LOCKBOX_VAULT_PASSWORD", "wrong-synthetic-password")
        .test_output()
        .unwrap();
    assert!(!failed.status.success(), "{failed:?}");
    assert!(!f.root.path().join("new.lbox").exists());
    let listed = f.ok(&["vault", "lockbox", "alias", "list", "--format", "json"]);
    assert!(!String::from_utf8_lossy(&listed.stdout).contains("new"));
}

#[test]
fn automatic_alias_preserves_standalone_creation_without_an_unlocked_vault() {
    for encrypted in [false, true] {
        let f = Fixture {
            root: tempfile::tempdir().unwrap(),
            raw_key: encrypted,
        };
        let encryption = if encrypted {
            "chacha20-poly1305"
        } else {
            "none"
        };
        let options = [
            "standalone.lbox",
            "create",
            "--signing",
            "none",
            "--encryption",
            encryption,
        ];
        let created = f
            .command(LBX, &options)
            .env_remove("LOCKBOX_VAULT_PASSWORD")
            .test_output()
            .unwrap();
        assert!(created.status.success(), "{created:?}");
        assert!(
            String::from_utf8_lossy(&created.stderr).contains("no unlocked Vault"),
            "{created:?}"
        );
        assert!(!f.root.path().join("vault").exists());
        f.ok(&[
            "standalone.lbox",
            "variables",
            "set",
            "TOKEN",
            "standalone-value",
        ]);
        let value = f.run(VALUE, &["standalone.lbox", "TOKEN"]);
        assert!(value.status.success(), "{value:?}");
        assert_eq!(value.stdout, b"standalone-value");
        assert!(!f.root.path().join("vault").exists());
    }
    let f = Fixture::new();
    let created = f
        .command(LBX, &["locked-vault.lbox", "create", "--signing", "none"])
        .env("LOCKBOX_VAULT_PASSWORD", "wrong-password")
        .test_output()
        .unwrap();
    assert!(created.status.success(), "{created:?}");
    assert!(
        String::from_utf8_lossy(&created.stderr).contains("no unlocked Vault"),
        "{created:?}"
    );
    f.ok(&[
        "locked-vault.lbox",
        "variables",
        "set",
        "TOKEN",
        "locked-vault-value",
    ]);
    let value = f.run(VALUE, &["locked-vault.lbox", "TOKEN"]);
    assert!(value.status.success(), "{value:?}");
    assert_eq!(value.stdout, b"locked-vault-value");
    assert!(!f.run(VALUE, &["a@locked-vault", "TOKEN"]).status.success());
}
