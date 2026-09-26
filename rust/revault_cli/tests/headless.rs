#![cfg(target_os = "linux")]
mod common;
use common::{CommandTestExt, TestTempDir};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
    time::{Duration, Instant},
};

fn run(dir: &Path, credential: bool, args: &[&str]) -> Output {
    let binary = std::env::var_os("REVAULT_HEADLESS_CLI")
        .or_else(|| option_env!("CARGO_BIN_EXE_lockbox").map(Into::into))
        .expect("set REVAULT_HEADLESS_CLI when testing an installed CLI");
    let mut command = Command::new(binary);
    command
        .current_dir(dir)
        .args(args)
        .env("LOCKBOX_VAULT_DIR", dir.join("vault"))
        .env(
            "LOCKBOX_SESSION_AGENT_DIR",
            common::agent_socket_dir(&dir.join("agent")),
        )
        .env("LOCKBOX_SESSION_AGENT_LOG", dir.join("agent.log"))
        .env("LOCKBOX_PLATFORM_SECRET_STORE", "auto")
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path={}", dir.join("missing-session-bus").display()),
        )
        .env(
            "DBUS_SYSTEM_BUS_ADDRESS",
            format!("unix:path={}", dir.join("missing-system-bus").display()),
        )
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("LOCKBOX_KEY")
        .env_remove("LOCKBOX_PASSWORD")
        .env_remove("LOCKBOX_VAULT_PASSWORD");
    if credential {
        command.env(
            "LOCKBOX_VAULT_PASSWORD",
            "synthetic headless test passphrase",
        );
    }
    let start = Instant::now();
    let output = command.test_output().unwrap();
    assert!(
        start.elapsed() < Duration::from_secs(30),
        "headless command exceeded time budget: {args:?}"
    );
    output
}

fn success(output: Output) -> Vec<u8> {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

#[test]
fn explicit_credentials_work_without_desktop_or_bus_and_missing_credentials_fail() {
    let temp = TestTempDir::new("headless");
    let dir = temp.path();
    success(run(dir, true, &["vault", "init"]));
    success(run(dir, true, &["headless.lbox", "create"]));
    success(run(dir, true, &["headless.lbox", "open"]));
    let original = b"headless persisted content\0\xff";
    fs::write(dir.join("input.bin"), original).unwrap();
    success(run(
        dir,
        true,
        &["headless.lbox", "add", "input.bin", "--to", "/data"],
    ));
    success(run(dir, true, &["session", "stop"]));
    success(run(dir, true, &["headless.lbox", "open"]));
    assert_eq!(
        success(run(dir, true, &["headless.lbox", "cat", "/data"])),
        original
    );
    fs::write(dir.join("input.bin"), b"replacement").unwrap();
    success(run(
        dir,
        true,
        &[
            "headless.lbox",
            "add",
            "input.bin",
            "--to",
            "/data",
            "--overwrite",
        ],
    ));
    success(run(
        dir,
        true,
        &[
            "headless.lbox",
            "add",
            "input.bin",
            "--to",
            "/data",
            "--overwrite",
        ],
    ));
    success(run(dir, true, &["session", "stop"]));
    success(run(dir, true, &["headless.lbox", "open"]));
    assert_eq!(
        success(run(dir, true, &["headless.lbox", "cat", "/data"])),
        b"replacement"
    );
    let unavailable = run(dir, true, &["session", "auto-open", "vault"]);
    assert!(!unavailable.status.success());
    assert!(String::from_utf8_lossy(&unavailable.stderr).contains("credential store"));
    success(run(dir, true, &["session", "stop"]));
    let no_credentials = run(dir, false, &["headless.lbox", "open"]);
    assert!(!no_credentials.status.success());
    assert!(!no_credentials.stderr.is_empty());
    success(run(dir, true, &["headless.lbox", "open"]));
    assert_eq!(
        success(run(dir, true, &["headless.lbox", "cat", "/data"])),
        b"replacement"
    );
    success(run(
        dir,
        true,
        &["headless.lbox", "remove", "--force", "/data"],
    ));
    success(run(dir, true, &["session", "stop"]));
    success(run(dir, true, &["headless.lbox", "open"]));
    assert!(!run(dir, true, &["headless.lbox", "cat", "/data"])
        .status
        .success());
    success(run(dir, true, &["session", "stop"]));
}
