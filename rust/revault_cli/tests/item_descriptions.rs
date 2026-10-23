mod common;
use common::CommandTestExt;
use std::process::{Command, Output};

#[test]
fn item_description_cli_persisted_lifecycle_and_multiline_output() {
    let dir = tempfile::tempdir().unwrap();
    let archive = dir.path().join("descriptions.lbox");
    let invoke = |args: &[&str], input: &[u8]| -> Output {
        Command::new(env!("CARGO_BIN_EXE_lockbox"))
            .arg(&archive)
            .args(args)
            .env_remove("LOCKBOX_KEY")
            .env("LOCKBOX_PASSWORD", "synthetic-description-password")
            .env("LOCKBOX_VAULT_DIR", dir.path().join("vault"))
            .env("LOCKBOX_SESSION_AGENT_DIR", dir.path().join("agent"))
            .test_output_with_input(input)
            .unwrap()
    };
    let run = |args: &[&str], input: &[u8]| -> String {
        let output = invoke(args, input);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    run(&["create", "--password", "--signing", "none"], b"");
    run(
        &["variables", "set", "TOKEN", "--secret", "--stdin"],
        b"synthetic-secret",
    );
    let description = "Purpose\nSecond line\twith tab\\slash";
    run(
        &["variables", "description", "set", "TOKEN", "--stdin"],
        description.as_bytes(),
    );
    assert_eq!(
        run(&["variables", "description", "get", "TOKEN"], b""),
        description
    );
    let listed = run(&["variables", "ls", "--format", "json"], b"");
    let row: serde_json::Value = serde_json::from_str(listed.trim()).unwrap();
    assert_eq!(row["description"], description);
    assert!(!listed.contains("synthetic-secret"));
    let table = run(&["variables", "ls"], b"");
    assert!(table.contains("Purpose…"));
    assert!(!table.contains("Second line"));
    let tsv = run(&["variables", "ls", "--format", "tsv"], b"");
    assert_eq!(tsv.lines().count(), 1);
    assert!(tsv.contains("Purpose\\nSecond line\\twith tab\\\\slash"));
    assert!(!run(&["variables", "export"], b"").contains("Purpose"));
    run(&["variables", "move", "TOKEN", "NEW_TOKEN"], b"");
    assert_eq!(
        run(&["variables", "description", "get", "NEW_TOKEN"], b""),
        description
    );
    assert_eq!(
        run(&["variables", "get", "NEW_TOKEN", "--secret"], b"").trim(),
        "synthetic-secret"
    );
    run(&["variables", "description", "clear", "NEW_TOKEN"], b"");
    run(&["variables", "description", "clear", "NEW_TOKEN"], b"");
    assert!(
        !invoke(&["variables", "description", "get", "NEW_TOKEN"], b"")
            .status
            .success()
    );
    assert!(
        !invoke(&["variables", "description", "set", "MISSING", "text"], b"")
            .status
            .success()
    );

    run(
        &[
            "forms",
            "define",
            "login",
            "--field",
            "password:secret:required:Password",
        ],
        b"",
    );
    run(&["forms", "add", "/staging", "--type", "login"], b"");
    run(
        &[
            "forms",
            "definition-description",
            "set",
            "login",
            "Login instructions\nMore guidance",
        ],
        b"",
    );
    run(
        &[
            "forms",
            "field-description",
            "set",
            "login",
            "password",
            "App password\nKeep it private",
        ],
        b"",
    );
    run(
        &[
            "forms",
            "description",
            "set",
            "/staging",
            "Staging\nRecreated nightly",
        ],
        b"",
    );
    let shown = run(&["forms", "show", "/staging"], b"");
    for text in [
        "Login instructions",
        "More guidance",
        "App password",
        "Keep it private",
        "Staging",
        "Recreated nightly",
        "guidance_revision",
    ] {
        assert!(shown.contains(text), "{shown}");
    }
    run(&["forms", "move", "/staging", "/renamed"], b"");
    assert_eq!(
        run(&["forms", "description", "get", "/renamed/staging"], b""),
        "Staging\nRecreated nightly"
    );
    assert!(run(&["forms", "ls"], b"").contains("Staging…"));
    run(
        &["forms", "field-description", "clear", "login", "password"],
        b"",
    );
    assert!(!invoke(
        &["forms", "field-description", "get", "login", "password"],
        b""
    )
    .status
    .success());
    run(&["forms", "remove", "/renamed/staging"], b"");
    run(
        &["forms", "add", "/renamed/staging", "--type", "login"],
        b"",
    );
    assert!(
        !invoke(&["forms", "description", "get", "/renamed/staging"], b"")
            .status
            .success()
    );
}
