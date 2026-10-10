mod common;

use common::{CommandTestExt, TestTempDir};
use std::process::Command;

#[test]
fn alias_ls_lists_the_same_persisted_records_as_list() {
    let root = TestTempDir::new("alias-ls");
    let run = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_lockbox"))
            .args(args)
            .current_dir(root.path())
            .env("LOCKBOX_VAULT_DIR", root.path().join("vault"))
            .env("LOCKBOX_SESSION_AGENT_DIR", root.path().join("agent"))
            .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
            .env(
                "LOCKBOX_VAULT_PASSWORD",
                "alias list synthetic vault password",
            )
            .env("LOCKBOX_KEY", "alias-list-synthetic-content-key")
            .env_remove("LOCKBOX_PASSWORD")
            .env_remove("LOCKBOX_KEY_FILE")
            .test_output()
            .unwrap();
        assert!(output.status.success(), "{args:?}: {output:?}");
        output.stdout
    };
    run(&["vault", "init"]);
    run(&["./listed.lbox", "create"]);
    run(&[
        "vault",
        "lockboxes",
        "aliases",
        "set",
        "listed",
        "./listed.lbox",
    ]);
    let listed = run(&["vault", "lockboxes", "aliases", "list", "--format", "json"]);
    let shortened = run(&["vault", "lockboxes", "aliases", "ls", "--format", "json"]);
    assert_eq!(shortened, listed);
    let row: serde_json::Value = serde_json::from_slice(&listed).unwrap();
    assert_eq!(row["alias"], "listed");
    assert!(!row["lockbox_id"].as_str().unwrap().is_empty());
    let legacy = run(&["vault", "lockbox", "alias", "ls", "--format", "json"]);
    assert_eq!(legacy, listed);
}
