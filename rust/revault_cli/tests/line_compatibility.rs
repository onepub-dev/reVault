//! Bidirectional persisted-format compatibility with an already released CLI.
//! Run explicitly with REVAULT_COMPAT_BASELINE_BIN pointing to the release binary.
mod common;

use common::{CommandTestExt, TestAgentGuard, TestTempDir};
use std::path::Path;
use std::process::{Command, Output};

#[test]
#[ignore = "requires the published compatibility-line baseline CLI"]
fn published_cli_and_candidate_read_and_update_each_others_archives_and_vaults() {
    let baseline = std::env::var_os("REVAULT_COMPAT_BASELINE_BIN")
        .expect("REVAULT_COMPAT_BASELINE_BIN must identify the published baseline CLI");
    let baseline = Path::new(&baseline);
    let candidate = Path::new(env!("CARGO_BIN_EXE_lockbox"));
    let declaration: serde_json::Value =
        serde_json::from_str(include_str!("../../release-compatibility.json")).unwrap();
    let expected_baseline = declaration["baseline_cli_version"].as_str().unwrap();
    let line: Vec<&str> = declaration["public_line"]
        .as_str()
        .unwrap()
        .split('.')
        .collect();

    for (origin, peer, name) in [
        (baseline, candidate, "baseline-origin"),
        (candidate, baseline, "candidate-origin"),
    ] {
        let root = TestTempDir::new(name);
        let vault = root.path().join("vault");
        let agent = common::agent_socket_dir(&root.path().join("agent"));
        let _guard = TestAgentGuard::new(&agent, Some(&vault));
        let run = |binary: &Path, args: &[&str]| -> Output {
            Command::new(binary)
                .current_dir(root.path())
                .args(args)
                .env("LOCKBOX_VAULT_DIR", &vault)
                .env(
                    "LOCKBOX_VAULT_PASSWORD",
                    "compatibility fixture vault password",
                )
                .env("LOCKBOX_KEY", "compatibility fixture archive content key")
                .env_remove("LOCKBOX_PASSWORD")
                .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
                .env("LOCKBOX_SESSION_AGENT_DIR", &agent)
                .env("LOCKBOX_SESSION_AGENT_LOG", root.path().join("agent.log"))
                .test_output()
                .unwrap()
        };
        let success = |binary: &Path, args: &[&str]| -> Vec<u8> {
            let output = run(binary, args);
            assert!(
                output.status.success(),
                "{} {args:?}\nstdout: {}\nstderr: {}",
                binary.display(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            output.stdout
        };
        for binary in [origin, peer] {
            let version = success(binary, &["--version"]);
            let output = String::from_utf8_lossy(&version);
            let actual = output
                .split_whitespace()
                .find(|word| word.as_bytes().first().is_some_and(u8::is_ascii_digit))
                .expect("CLI reports a semantic version");
            let parts: Vec<&str> = actual.split('.').collect();
            assert!(
                parts.len() == 3 && parts[0] == line[0] && (line[0] != "0" || parts[1] == line[1]),
                "wrong compatibility line: {output}"
            );
            if binary == baseline {
                assert_eq!(
                    actual, expected_baseline,
                    "test must use the declared released baseline"
                );
            }
        }
        success(origin, &["vault", "init"]);
        success(origin, &["vault", "profile", "create", "origin-profile"]);
        success(
            origin,
            &[
                "vault",
                "profile",
                "export",
                "origin.pub",
                "--format",
                "raw",
            ],
        );
        let profile = std::fs::read(root.path().join("origin.pub")).unwrap();
        success(origin, &["shared.lbox", "create", "--signing", "none"]);
        let original = b"first persisted payload\0\xff".repeat(4097);
        std::fs::write(root.path().join("input"), &original).unwrap();
        success(origin, &["shared.lbox", "add", "input", "--to", "/payload"]);
        success(
            origin,
            &["shared.lbox", "variable", "set", "VALUE", "original"],
        );
        success(origin, &["vault", "lockbox", "remember", "shared.lbox"]);
        success(origin, &["session", "stop"]);

        assert_eq!(success(peer, &["shared.lbox", "cat", "/payload"]), original);
        assert_eq!(
            success(peer, &["shared.lbox", "variable", "get", "VALUE"]),
            b"original\n"
        );
        assert!(
            String::from_utf8_lossy(&success(peer, &["vault", "profile", "list"]))
                .contains("origin-profile")
        );
        assert!(
            String::from_utf8_lossy(&success(peer, &["vault", "lockbox", "list"]))
                .contains("shared.lbox")
        );
        success(
            peer,
            &["vault", "profile", "export", "peer.pub", "--format", "raw"],
        );
        assert_eq!(
            std::fs::read(root.path().join("peer.pub")).unwrap(),
            profile
        );
        success(peer, &["vault", "profile", "create", "peer-profile"]);
        let replacement = b"peer replacement persisted payload\0\xfe".repeat(8193);
        std::fs::write(root.path().join("input"), &replacement).unwrap();
        success(
            peer,
            &[
                "shared.lbox",
                "add",
                "input",
                "--to",
                "/payload",
                "--overwrite",
            ],
        );
        success(peer, &["shared.lbox", "variable", "set", "VALUE", "peer"]);
        success(peer, &["session", "stop"]);

        assert_eq!(
            success(origin, &["shared.lbox", "cat", "/payload"]),
            replacement
        );
        assert_eq!(
            success(origin, &["shared.lbox", "variable", "get", "VALUE"]),
            b"peer\n"
        );
        assert!(
            String::from_utf8_lossy(&success(origin, &["vault", "profile", "list"]))
                .contains("peer-profile")
        );
        assert!(
            String::from_utf8_lossy(&success(origin, &["vault", "lockbox", "list"]))
                .contains("shared.lbox")
        );
        success(origin, &["shared.lbox", "remove", "/payload", "--force"]);
        success(origin, &["session", "stop"]);
        assert!(!run(peer, &["shared.lbox", "cat", "/payload"])
            .status
            .success());
        success(peer, &["session", "stop"]);

        // New descriptions use format-4 encrypted metadata. Released readers
        // must preserve it through ordinary writes, regardless of the creator.
        success(
            candidate,
            &[
                "shared.lbox",
                "variables",
                "description",
                "set",
                "VALUE",
                "Purpose\nSecond line",
            ],
        );
        success(
            candidate,
            &[
                "shared.lbox",
                "forms",
                "define",
                "login",
                "--field",
                "username:text",
            ],
        );
        success(
            candidate,
            &["shared.lbox", "forms", "add", "/account", "--type", "login"],
        );
        success(
            candidate,
            &[
                "shared.lbox",
                "forms",
                "field-description",
                "set",
                "login",
                "username",
                "Full email\nInclude domain",
            ],
        );
        success(
            candidate,
            &[
                "shared.lbox",
                "forms",
                "description",
                "set",
                "/account",
                "Company account",
            ],
        );
        success(
            candidate,
            &[
                "vault",
                "forms",
                "define",
                "login",
                "--field",
                "username:text",
            ],
        );
        success(
            candidate,
            &[
                "vault",
                "forms",
                "field-description",
                "set",
                "login",
                "username",
                "Vault guidance",
            ],
        );
        success(candidate, &["session", "stop"]);
        success(
            baseline,
            &["shared.lbox", "variables", "set", "VALUE", "legacy update"],
        );
        success(
            baseline,
            &["vault", "profile", "create", "description-compat"],
        );
        success(baseline, &["session", "stop"]);
        assert_eq!(
            success(
                candidate,
                &["shared.lbox", "variables", "description", "get", "VALUE"]
            ),
            b"Purpose\nSecond line"
        );
        assert_eq!(
            success(
                candidate,
                &[
                    "shared.lbox",
                    "forms",
                    "field-description",
                    "get",
                    "login",
                    "username"
                ]
            ),
            b"Full email\nInclude domain"
        );
        assert_eq!(
            success(
                candidate,
                &["shared.lbox", "forms", "description", "get", "/account"]
            ),
            b"Company account"
        );
        assert_eq!(
            success(
                candidate,
                &[
                    "vault",
                    "forms",
                    "field-description",
                    "get",
                    "login",
                    "username"
                ]
            ),
            b"Vault guidance"
        );
        success(candidate, &["session", "stop"]);
    }
}
