mod common;

use common::TestTempDir;
use std::fs;
use std::process::Command;

fn run(bin: &str, args: &[&str], vault_dir: &std::path::Path) -> std::process::Output {
    Command::new(bin)
        .args(args)
        .env("LOCKBOX_VAULT_DIR", vault_dir)
        .env_remove("LOCKBOX_VAULT_PASSWORD")
        .env_remove("COMPLETE")
        .output()
        .unwrap()
}

#[test]
fn completion_generation_supports_shell_override_and_install_uninstall() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("completion-install");
    let vault_dir = temp.path().join("vault");
    let output_path = temp.path().join("completion").join("lockbox.bash");

    let generated = run(
        bin,
        &[
            "completion",
            "generate",
            "--shell",
            "bash",
            "--output",
            output_path.to_str().unwrap(),
        ],
        &vault_dir,
    );
    assert!(generated.status.success(), "{generated:?}");
    let script = fs::read_to_string(&output_path).unwrap();
    assert!(script.contains("_clap_complete_lockbox"));
    assert!(script.contains("COMPLETE=\"bash\""));

    let installed = temp.path().join("custom").join("lockbox.fish");
    let output = run(
        bin,
        &[
            "completion",
            "install",
            "--shell",
            "fish",
            "--path",
            installed.to_str().unwrap(),
        ],
        &vault_dir,
    );
    assert!(output.status.success(), "{output:?}");
    assert!(installed.exists());

    let output = run(
        bin,
        &[
            "completion",
            "uninstall",
            "--shell",
            "fish",
            "--path",
            installed.to_str().unwrap(),
        ],
        &vault_dir,
    );
    assert!(output.status.success(), "{output:?}");
    assert!(!installed.exists());
}

#[test]
fn locked_vault_completion_falls_back_without_prompt_or_diagnostics() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("completion-locked");
    let vault_dir = temp.path().join("vault");

    let output = Command::new(bin)
        .env("LOCKBOX_VAULT_DIR", &vault_dir)
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "4")
        .env("_CLAP_COMPLETE_COMP_TYPE", "9")
        .env("_CLAP_COMPLETE_SPACE", "true")
        .args(["--", "lockbox", "vault", "profile", "create", ""])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
}

#[test]
fn dynamic_completion_reads_vault_names_without_exposing_signing_material() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("completion-dynamic");
    let vault_dir = temp.path().join("vault");
    let agent_dir = temp.path().join("agent");
    let agent_log = temp.path().join("agent.log");
    let password = "completion-test-vault-password";

    let init = Command::new(bin)
        .args(["vault", "init"])
        .env("LOCKBOX_VAULT_DIR", &vault_dir)
        .env("LOCKBOX_VAULT_PASSWORD", password)
        .env("LOCKBOX_SESSION_AGENT_DIR", &agent_dir)
        .env("LOCKBOX_SESSION_AGENT_LOG", &agent_log)
        .output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");

    let create = Command::new(bin)
        .args(["vault", "profile", "create", "alice"])
        .env("LOCKBOX_VAULT_DIR", &vault_dir)
        .env("LOCKBOX_VAULT_PASSWORD", password)
        .env("LOCKBOX_SESSION_AGENT_DIR", &agent_dir)
        .env("LOCKBOX_SESSION_AGENT_LOG", &agent_log)
        .output()
        .unwrap();
    assert!(create.status.success(), "{create:?}");

    let log_len_before_completion = fs::metadata(&agent_log).map(|m| m.len()).unwrap_or(0);

    let output = Command::new(bin)
        .env("LOCKBOX_VAULT_DIR", &vault_dir)
        .env("LOCKBOX_VAULT_PASSWORD", password)
        .env("LOCKBOX_SESSION_AGENT_DIR", &agent_dir)
        .env("LOCKBOX_SESSION_AGENT_LOG", &agent_log)
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "4")
        .env("_CLAP_COMPLETE_COMP_TYPE", "9")
        .env("_CLAP_COMPLETE_SPACE", "true")
        .args(["--", "lockbox", "vault", "profile", "history", "al"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("alice"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("LBX1SPRV"));
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);

    let agent_log_bytes = fs::read(&agent_log).unwrap_or_default();
    let completion_log = &agent_log_bytes[log_len_before_completion as usize..];
    assert!(
        !String::from_utf8_lossy(completion_log).contains("owner-signing:"),
        "completion must not request an owner-signing key from the agent"
    );
}
