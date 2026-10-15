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
    f.ok(&["other.lbox", "variable", "set", "TOKEN", "replacement"]);
    f.ok(&["vault", "lockbox", "alias", "set", "dev", "other.lbox"]);
    assert_eq!(f.read("TOKEN"), b"replacement");
    f.ok(&["vault", "lockbox", "alias", "remove", "dev"]);
    f.ok(&["vault", "lockbox", "alias", "remove", "dev"]);
    assert!(!f.run(VALUE, &["a@dev", "TOKEN"]).status.success());
    assert!(f.run(VALUE, &["other.lbox", "TOKEN"]).status.success());
    for name in ["a@b", "../bad", "", "bad name"] {
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
fn format_three_reader_refuses_unicode_format_four_vault_without_mutation() {
    let Ok(baseline) = std::env::var("REVAULT_ALIAS_BASELINE_BIN") else {
        return;
    };
    let f = Fixture::new();
    f.ok(&["vault", "lockbox", "alias", "set", "東京.dév", "dev.lbox"]);
    let vault_path = f.root.path().join("vault/local-vault.lbox");
    let before = std::fs::read(&vault_path).unwrap();
    let output = f.run(&baseline, &["vault", "lockbox", "alias", "list"]);
    assert!(!output.status.success(), "{output:?}");
    let diagnostic = String::from_utf8_lossy(&output.stderr).to_lowercase();
    assert_eq!(output.status.code(), Some(14), "{diagnostic}");
    assert!(
        diagnostic.contains("unsupported") && diagnostic.contains("format"),
        "{diagnostic}"
    );
    assert!(
        diagnostic.contains(
            "found lockbox container version 4; this revault build supports container version 3"
        ),
        "{diagnostic}"
    );
    assert!(
        diagnostic.contains("use a revault build that supports this newer format"),
        "{diagnostic}"
    );
    assert!(
        diagnostic.contains("automatic downgrade is not supported"),
        "{diagnostic}"
    );
    assert!(!diagnostic.contains("migrate"), "{diagnostic}");
    assert!(!diagnostic.contains("invalid record name"), "{diagnostic}");
    assert_eq!(std::fs::read(vault_path).unwrap(), before);
    assert_eq!(
        f.run(VALUE, &["a@東京.dév", "TOKEN"]).stdout,
        b"synthetic-token"
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
        "bad name",
        "bad name",
        &"a".repeat(129),
    ] {
        let existing = f.run(LBX, &["vault", "lockbox", "alias", "set", name, "dev.lbox"]);
        let failed = f.run(LBX, &["invalid.lbox", "create", "--alias", name]);
        assert!(!failed.status.success(), "{failed:?}");
        assert_eq!(failed.stderr, existing.stderr);
        assert!(!f.root.path().join("invalid.lbox").exists());
    }
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
    // Standalone unsigned creation must not initialize a Vault for auto-aliases.
    let created = f.ok(&[
        "plain.lbox",
        "create",
        "--encryption",
        "none",
        "--signing",
        "none",
    ]);
    assert!(String::from_utf8_lossy(&created.stderr).contains("no unlocked Vault"));
    assert!(!f.root.path().join("vault").exists());
    f.ok(&[
        "plain.lbox",
        "variable",
        "set",
        "VALUE",
        "synthetic-plain-value",
    ]);
    assert_eq!(
        f.run(VALUE, &["plain.lbox", "VALUE"]).stdout,
        b"synthetic-plain-value"
    );
    f.ok(&["vault", "init"]);
    let failed = f
        .command(
            LBX,
            &[
                "new.lbox",
                "create",
                "--alias",
                "explicit",
                "--encryption",
                "none",
                "--signing",
                "none",
            ],
        )
        .env("LOCKBOX_VAULT_PASSWORD", "wrong-synthetic-password")
        .test_output()
        .unwrap();
    assert!(!failed.status.success(), "{failed:?}");
    assert!(!f.root.path().join("new.lbox").exists());
}

#[test]
fn unicode_alias_lifecycle_normalizes_and_preserves_persisted_values() {
    let f = Fixture::new();
    let decomposed = "cafe\u{0301}.東京";
    let canonical = "café.東京";
    f.ok(&["vault", "lockbox", "alias", "set", decomposed, "dev.lbox"]);
    let first = f
        .ok(&["vault", "lockbox", "alias", "list", "--format", "json"])
        .stdout;
    f.ok(&["vault", "lockbox", "alias", "set", canonical, "dev.lbox"]);
    assert_eq!(
        first,
        f.ok(&["vault", "lockbox", "alias", "list", "--format", "json"])
            .stdout
    );
    assert!(String::from_utf8_lossy(&first).contains(canonical));
    assert!(!String::from_utf8_lossy(&first).contains(decomposed));
    assert_eq!(
        f.run(VALUE, &[&format!("a@{decomposed}"), "TOKEN"]).stdout,
        b"synthetic-token"
    );
    f.ok(&["other.lbox", "variable", "set", "TOKEN", "replacement"]);
    f.ok(&["vault", "lockbox", "alias", "set", decomposed, "other.lbox"]);
    assert_eq!(
        f.run(VALUE, &[&format!("a@{canonical}"), "TOKEN"]).stdout,
        b"replacement"
    );
    f.ok(&["vault", "backup", "unicode.backup"]);
    f.ok(&["vault", "lockbox", "alias", "remove", canonical]);
    f.ok(&["vault", "restore", "unicode.backup", "--overwrite"]);
    assert_eq!(
        f.run(VALUE, &[&format!("a@{decomposed}"), "TOKEN"]).stdout,
        b"replacement"
    );
    f.ok(&["vault", "lockbox", "alias", "remove", decomposed]);
    f.ok(&["vault", "lockbox", "alias", "remove", canonical]);
    assert!(!f
        .run(VALUE, &[&format!("a@{canonical}"), "TOKEN"])
        .status
        .success());
    assert_eq!(
        f.run(VALUE, &["other.lbox", "TOKEN"]).stdout,
        b"replacement"
    );
}

#[test]
fn unicode_creation_derives_names_and_preserves_normalization_collisions() {
    let f = Fixture::new();
    for (filename, alias) in [
        ("cafe\u{0301}.東京.lbox", "café.東京"),
        ("space  name.lbox", "space__name"),
        (".hidden.lbox", "_hidden"),
    ] {
        f.ok(&[filename, "create"]);
        let selector = format!("a@{alias}");
        f.ok(&[&selector, "variable", "set", "VALUE", alias]);
        assert_eq!(f.run(VALUE, &[&selector, "VALUE"]).stdout, alias.as_bytes());
        assert_eq!(f.run(VALUE, &[filename, "VALUE"]).stdout, alias.as_bytes());
    }
    let collision = f.ok(&["another.lbox", "create", "--alias", "cafe\u{0301}.東京"]);
    assert!(String::from_utf8_lossy(&collision.stderr).contains("already exists"));
    assert_eq!(
        f.run(VALUE, &["a@café.東京", "VALUE"]).stdout,
        "café.東京".as_bytes()
    );
    f.ok(&["another.lbox", "variable", "set", "VALUE", "new archive"]);
    assert_eq!(
        f.run(VALUE, &["another.lbox", "VALUE"]).stdout,
        b"new archive"
    );
    let long_filename = format!("{}.lbox", "é".repeat(65));
    let created = f.ok(&[&long_filename, "create"]);
    assert!(String::from_utf8_lossy(&created.stderr).contains("128 UTF-8 bytes"));
    f.ok(&[&long_filename, "variable", "set", "VALUE", "kept"]);
    assert_eq!(f.run(VALUE, &[&long_filename, "VALUE"]).stdout, b"kept");
}

#[test]
fn unicode_alias_helpers_normalize_lookup_and_completion_prefixes() {
    let f = Fixture::new();
    f.form();
    f.ok(&["vault", "lockbox", "alias", "set", "café", "dev.lbox"]);
    for shell in ["bash", "fish"] {
        for (binary, program) in [(VALUE, "lbxv"), (EXEC, "lbxx")] {
            let output = f
                .command(binary, &["--", program, "a@cafe\u{0301}"])
                .env("COMPLETE", shell)
                .env("_CLAP_COMPLETE_INDEX", "1")
                .env("_CLAP_COMPLETE_COMP_TYPE", "9")
                .env("_CLAP_COMPLETE_SPACE", "true")
                .test_output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
            let expected = if shell == "fish" {
                "a@cafe\u{0301}"
            } else {
                "a@café"
            };
            assert!(
                String::from_utf8_lossy(&output.stdout).contains(expected),
                "{output:?}"
            );
            assert!(!String::from_utf8_lossy(&output.stdout).contains("synthetic"));
        }
    }
    assert_eq!(
        f.run(VALUE, &["a@cafe\u{0301}", "TOKEN"]).stdout,
        b"synthetic-token"
    );
    let child = std::env::current_exe().unwrap();
    let output = f
        .command(
            EXEC,
            &[
                "a@cafe\u{0301}",
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
