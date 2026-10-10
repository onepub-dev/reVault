mod common;

use common::{CommandTestExt, TestTempDir};
use std::process::{Command, Output};

struct Fixture(TestTempDir);

impl Fixture {
    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lockbox"));
        command
            .args(args)
            .current_dir(self.0.path())
            .env("LOCKBOX_VAULT_DIR", self.0.path().join("vault"))
            .env("LOCKBOX_SESSION_AGENT_DIR", self.0.path().join("agent"))
            .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
            .env("LOCKBOX_VAULT_PASSWORD", "collection test vault password")
            .env("LOCKBOX_PASSWORD", "collection test archive password")
            .env("LOCKBOX_KEY", "collection-test-key")
            .env_remove("COMPLETE");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        let output = self.command(args).test_output().unwrap();
        assert!(output.status.success(), "{args:?}: {output:?}");
        output
    }

    fn text(&self, args: &[&str]) -> String {
        let output = self.run(args);
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

#[test]
fn canonical_collections_and_hidden_aliases_share_persisted_state() {
    let fixture = Fixture(TestTempDir::new("collection-names"));
    fixture.run(&["vault", "init"]);
    fixture.run(&["vault", "profiles", "create", "collection-owner"]);
    assert!(fixture
        .text(&["vault", "profile", "list"])
        .contains("collection-owner"));
    fixture.run(&[
        "vault",
        "profiles",
        "export",
        "owner.pub",
        "--name",
        "collection-owner",
    ]);
    // No public CLI command computes the raw key-file import fingerprint.
    // Derive only that verification value from the publicly exported key.
    let public_key = revault_vault_api::import_public_key(
        &std::fs::read(fixture.0.path().join("owner.pub")).unwrap(),
    )
    .unwrap();
    let fingerprint =
        revault_vault_api::encode_hex(&revault_vault_api::public_key_fingerprint(&public_key));
    fixture.run(&[
        "vault",
        "contacts",
        "import",
        "collection-contact",
        "owner.pub",
        "--fingerprint",
        &fingerprint,
        "--fingerprint-channel",
        "in-person",
    ]);
    assert!(fixture
        .text(&["vault", "contact", "list"])
        .contains("collection-contact"));
    fixture.run(&["data.lbox", "create"]);
    fixture.run(&[
        "vault",
        "lockboxes",
        "aliases",
        "set",
        "collection-archive",
        "data.lbox",
    ]);
    assert!(fixture
        .text(&["vault", "lockbox", "alias", "list"])
        .contains("collection-archive"));
    fixture.run(&[
        "a@collection-archive",
        "variables",
        "set",
        "GREETING",
        "persisted collection bytes",
    ]);
    for name in ["variables", "variable", "var"] {
        assert_eq!(
            fixture.run(&["data.lbox", name, "get", "GREETING"]).stdout,
            b"persisted collection bytes\n"
        );
    }
    fixture.run(&[
        "vault",
        "forms",
        "define",
        "login",
        "--field",
        "username:text",
    ]);
    assert!(fixture.text(&["vault", "form", "list"]).contains("login"));
    fixture.run(&["data.lbox", "forms", "use", "login"]);
    fixture.run(&[
        "data.lbox",
        "forms",
        "add",
        "/account",
        "--type",
        "login",
        "--set",
        "username=alice",
    ]);
    assert_eq!(
        fixture
            .run(&["data.lbox", "form", "get", "/account@username"])
            .stdout,
        b"alice\n"
    );
    std::fs::create_dir(fixture.0.path().join("source")).unwrap();
    std::fs::write(
        fixture.0.path().join("source/keep.txt"),
        b"mirror persisted bytes",
    )
    .unwrap();
    fixture.run(&[
        "data.lbox",
        "mirrors",
        "project",
        "create",
        "--from",
        "source",
        "--to",
        "/project",
    ]);
    fixture.run(&[
        "data.lbox",
        "mirrors",
        "project",
        "rules",
        "add",
        "exclude",
        "*.tmp",
    ]);
    assert!(fixture
        .text(&["data.lbox", "mirror", "project", "rule", "list"])
        .contains("*.tmp"));
    fixture.run(&["data.lbox", "mirrors", "project", "update", "--force"]);
    assert_eq!(
        fixture
            .run(&["data.lbox", "mirror", "project", "cat", "keep.txt"])
            .stdout,
        b"mirror persisted bytes"
    );
}

#[test]
fn help_and_completion_advertise_only_canonical_collection_names() {
    let fixture = Fixture(TestTempDir::new("collection-help"));
    type CanonicalNameCase<'a> = (&'a [&'a str], &'a [(&'a str, &'a str)]);
    let cases: &[CanonicalNameCase<'_>] = &[
        (
            &[],
            &[
                ("forms", "form"),
                ("variables", "variable"),
                ("mirrors", "mirror"),
            ],
        ),
        (
            &["vault"],
            &[
                ("profiles", "profile"),
                ("contacts", "contact"),
                ("lockboxes", "lockbox"),
                ("forms", "form"),
            ],
        ),
        (&["vault", "lockboxes"], &[("aliases", "alias")]),
        (&["mirrors"], &[("rules", "rule")]),
    ];
    for (path, names) in cases {
        for verbose in [false, true] {
            let mut args = path.to_vec();
            args.push("--help");
            if verbose {
                args.push("--verbose");
            }
            let text = fixture.text(&args);
            let rows: Vec<_> = text
                .lines()
                .filter_map(|line| line.split_whitespace().next())
                .collect();
            for (plural, singular) in *names {
                assert!(rows.contains(plural), "{args:?}: {text}");
                assert!(!rows.contains(singular), "{args:?}: {text}");
                assert!(!text.contains(&format!("[aliases: {singular}")), "{text}");
                let mut legacy = path.to_vec();
                legacy.extend([singular, "--help"]);
                fixture.run(&legacy);
            }
            assert!(!text.contains("[aliases: var"), "{text}");
        }
        let mut words = vec!["--", "lockbox"];
        words.extend_from_slice(path);
        words.push("");
        let output = fixture
            .command(&words)
            .env("COMPLETE", "bash")
            .env("_CLAP_COMPLETE_INDEX", (path.len() + 1).to_string())
            .env("_CLAP_COMPLETE_COMP_TYPE", "9")
            .env("_CLAP_COMPLETE_SPACE", "true")
            .test_output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let text = String::from_utf8_lossy(&output.stdout);
        let completions: Vec<_> = text.lines().map(str::trim).collect();
        for (plural, singular) in *names {
            assert!(completions.contains(plural), "{path:?}: {text}");
            assert!(!completions.contains(singular), "{path:?}: {text}");
        }
        assert!(!completions.contains(&"var"), "{text}");
    }
    // Migration selects one artifact; these words are intentionally singular.
    for kind in ["vault", "lockbox"] {
        fixture.run(&["doctor", "migrate", kind, "--help"]);
    }
}
