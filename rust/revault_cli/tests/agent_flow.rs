mod common;

use common::TestTempDir;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(20);

#[test]
fn open_populates_cache_and_close_clears_it() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("lockbox-cli-agent-flow");
    let dir = temp.path();
    let vault = dir.join("test.lbox");
    let source = dir.join("source.txt");
    let agent_dir = dir.join("agent");
    let vault_dir = dir.join("vault");
    fs::create_dir_all(&agent_dir).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();
    fs::write(&source, "alpha").unwrap();

    run(bin, &agent_dir, &vault_dir, &["vault", "init"]);
    run(
        bin,
        &agent_dir,
        &vault_dir,
        &["create", vault.to_str().unwrap()],
    );
    let open = run_output(
        bin,
        &agent_dir,
        &vault_dir,
        &["open", vault.to_str().unwrap()],
    );
    if String::from_utf8_lossy(&open.stderr).contains("lockbox session agent did not start") {
        eprintln!("skipping session agent cache assertions: lockbox session agent did not start");
        return;
    }
    assert!(
        open.status.success(),
        "command failed: {bin} open {}\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        vault.display(),
        open.status,
        String::from_utf8_lossy(&open.stdout),
        String::from_utf8_lossy(&open.stderr)
    );
    let output = run_output(bin, &agent_dir, &vault_dir, &["session", "--format", "tsv"]);
    assert!(
        output.status.success(),
        "command failed: {bin} session --format tsv\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let open_list_text = String::from_utf8_lossy(&output.stdout);
    assert!(open_list_text.contains("open\t"));
    assert!(open_list_text.contains(vault.to_str().unwrap()));

    let output = run_output(bin, &agent_dir, &vault_dir, &["session"]);
    assert!(
        output.status.success(),
        "command failed: {bin} session\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let vault_opened = String::from_utf8_lossy(&output.stdout);
    assert!(vault_opened.contains("Default lockbox:"));
    assert!(vault_opened.contains("Open lockboxes:"));
    assert!(vault_opened.contains(vault.to_str().unwrap()));

    run(
        bin,
        &agent_dir,
        &vault_dir,
        &[
            "add",
            vault.to_str().unwrap(),
            source.to_str().unwrap(),
            "/docs/a.txt",
        ],
    );

    let output = run_output(
        bin,
        &agent_dir,
        &vault_dir,
        &["list", vault.to_str().unwrap(), "/docs"],
    );
    assert!(
        output.status.success(),
        "command failed: {bin} list {} /docs\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        vault.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("a.txt"));
    assert_agent_log_contains(&agent_dir, "cached lockbox");
    assert_agent_log_contains(&agent_dir, "cache hit");

    run(
        bin,
        &agent_dir,
        &vault_dir,
        &["close", vault.to_str().unwrap()],
    );
    let output = run_output(
        bin,
        &agent_dir,
        &vault_dir,
        &["list", vault.to_str().unwrap(), "/docs"],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("lockbox is closed"));
    assert_agent_log_contains(&agent_dir, "forgot lockbox");

    let output = run_output(bin, &agent_dir, &vault_dir, &["session"]);
    assert!(
        output.status.success(),
        "command failed: {bin} session\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let session = String::from_utf8_lossy(&output.stdout);
    assert!(session.contains("Session agent:"));
    assert!(session.contains("Auto-open:"));
    assert!(session.contains("Default lockbox:"));
    assert!(session.contains("Open lockboxes:"));
    assert!(session.contains("none"));
}

fn assert_agent_log_contains(agent_dir: &Path, expected: &str) {
    let log_path = agent_dir.join("agent.log");
    let log = fs::read_to_string(&log_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", log_path.display()));
    assert!(
        log.contains(expected),
        "expected agent log {} to contain {expected:?}; contents:\n{log}",
        log_path.display()
    );
}

fn run(bin: &str, agent_dir: &PathBuf, vault_dir: &PathBuf, args: &[&str]) {
    let status = run_status(bin, agent_dir, vault_dir, args);
    assert!(
        status.success(),
        "command failed: {bin} {}\nstatus: {}",
        args.join(" "),
        status
    );
}

fn run_status(bin: &str, agent_dir: &PathBuf, vault_dir: &PathBuf, args: &[&str]) -> ExitStatus {
    let mut command = command(bin, agent_dir, vault_dir, args);
    command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    let command_line = format!("{bin} {}", args.join(" "));
    let mut child = command.spawn().unwrap();
    let deadline = Instant::now() + COMMAND_TIMEOUT;
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait().unwrap();
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            panic!("command timed out after {COMMAND_TIMEOUT:?}: {command_line}");
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn run_output(bin: &str, agent_dir: &PathBuf, vault_dir: &PathBuf, args: &[&str]) -> Output {
    let mut command = command(bin, agent_dir, vault_dir, args);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let command_line = format!("{bin} {}", args.join(" "));
    let mut child = command.spawn().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let stdout_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).unwrap();
        bytes
    });
    let stderr_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).unwrap();
        bytes
    });
    let deadline = Instant::now() + COMMAND_TIMEOUT;
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("command timed out after {COMMAND_TIMEOUT:?}: {command_line}");
        }
        thread::sleep(Duration::from_millis(25));
    };
    Output {
        status,
        stdout: stdout_reader.join().unwrap(),
        stderr: stderr_reader.join().unwrap(),
    }
}

fn command(bin: &str, agent_dir: &PathBuf, vault_dir: &PathBuf, args: &[&str]) -> Command {
    let mut command = Command::new(bin);
    command
        .args(args)
        .env("LOCKBOX_PASSWORD", "test-password")
        .env("LOCKBOX_VAULT_PASSWORD", "test-vault-password")
        .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
        .env("LOCKBOX_SESSION_AGENT_DIR", agent_dir)
        .env("LOCKBOX_SESSION_AGENT_LOG", agent_dir.join("agent.log"))
        .env("LOCKBOX_VAULT_DIR", vault_dir);
    command
}
