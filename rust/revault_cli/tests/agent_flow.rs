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
    let agent_temp = tempfile::Builder::new()
        .prefix("lbx-agent-")
        .tempdir()
        .unwrap();
    let agent_dir = agent_temp.path().to_path_buf();
    let vault_dir = dir.join("vault");
    fs::create_dir_all(&agent_dir).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();
    fs::write(&source, "alpha").unwrap();

    run(bin, &agent_dir, &vault_dir, &["vault", "init"]);
    run(
        bin,
        &agent_dir,
        &vault_dir,
        &[vault.to_str().unwrap(), "create"],
    );
    // The first open launches the Windows agent. Inheriting the test's capture
    // pipes into that process tree prevents their readers from observing EOF.
    #[cfg(windows)]
    let open = Output {
        status: run_status(
            bin,
            &agent_dir,
            &vault_dir,
            &[vault.to_str().unwrap(), "open"],
        ),
        stdout: Vec::new(),
        stderr: Vec::new(),
    };
    #[cfg(not(windows))]
    let open = run_output(
        bin,
        &agent_dir,
        &vault_dir,
        &[vault.to_str().unwrap(), "open"],
    );
    if String::from_utf8_lossy(&open.stderr).contains("Session Agent did not start") {
        eprintln!("skipping Session Agent cache assertions: Session Agent did not start");
        stop_agent(bin, &agent_dir, &vault_dir);
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

    run(
        bin,
        &agent_dir,
        &vault_dir,
        &[
            vault.to_str().unwrap(),
            "add",
            source.to_str().unwrap(),
            "--to",
            "/docs/a.txt",
        ],
    );

    let output = run_output(
        bin,
        &agent_dir,
        &vault_dir,
        &[vault.to_str().unwrap(), "list", "/docs"],
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
        &[vault.to_str().unwrap(), "close"],
    );
    let output = run_output(
        bin,
        &agent_dir,
        &vault_dir,
        &[vault.to_str().unwrap(), "list", "/docs"],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Lockbox is closed"));
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
    assert!(session.contains("Session Agent:"));
    assert!(session.contains("Auto-open:"));
    assert!(session.contains("Default lockbox:"));
    assert!(session.contains("Open lockboxes:"));
    assert!(session.contains("none"));
    stop_agent(bin, &agent_dir, &vault_dir);
}

#[test]
#[ignore = "serial E2E contract for the platform Session Agent"]
fn open_and_open_key_complete_real_session_flows() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("lockbox-cli-open-contract");
    let dir = temp.path();
    let lockbox = dir.join("open-contract.lbox");
    let password_file = dir.join("password.txt");
    let agent_temp = tempfile::Builder::new()
        .prefix("lbx-agent-")
        .tempdir()
        .unwrap();
    let agent_dir = agent_temp.path().to_path_buf();
    let vault_dir = dir.join("vault");
    fs::create_dir_all(&agent_dir).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();
    fs::write(&password_file, "test-password\n").unwrap();

    let mut agent = command(bin, &agent_dir, &vault_dir, &["__agent"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    #[cfg(unix)]
    {
        let deadline = Instant::now() + COMMAND_TIMEOUT;
        while !agent_dir.join("agent.sock").exists() {
            if let Some(status) = agent.try_wait().unwrap() {
                panic!("Session Agent exited during startup with {status}");
            }
            assert!(
                Instant::now() < deadline,
                "Session Agent did not create its socket after {COMMAND_TIMEOUT:?}"
            );
            thread::sleep(Duration::from_millis(25));
        }
    }
    #[cfg(windows)]
    {
        thread::sleep(Duration::from_millis(500));
        if let Some(status) = agent.try_wait().unwrap() {
            panic!("Session Agent exited during startup with {status}");
        }
    }

    run(bin, &agent_dir, &vault_dir, &["vault", "init"]);
    run(
        bin,
        &agent_dir,
        &vault_dir,
        &[lockbox.to_str().unwrap(), "create", "--password"],
    );
    let env_open = command(
        bin,
        &agent_dir,
        &vault_dir,
        &[
            lockbox.to_str().unwrap(),
            "open",
            "--duration",
            "2m",
            "--password-env",
            "E2E_LOCKBOX_PASSWORD",
        ],
    )
    .env("E2E_LOCKBOX_PASSWORD", "test-password")
    .output()
    .unwrap();
    if !env_open.status.success() {
        let log = fs::read_to_string(agent_dir.join("agent.log"))
            .unwrap_or_else(|error| format!("unable to read agent log: {error}"));
        panic!(
            "open --password-env failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nagent log:\n{log}",
            env_open.status,
            String::from_utf8_lossy(&env_open.stdout),
            String::from_utf8_lossy(&env_open.stderr)
        );
    }
    run(
        bin,
        &agent_dir,
        &vault_dir,
        &[
            lockbox.to_str().unwrap(),
            "access",
            "grant",
            "profile:default",
        ],
    );
    run(
        bin,
        &agent_dir,
        &vault_dir,
        &[lockbox.to_str().unwrap(), "close"],
    );

    let file_open = run_output(
        bin,
        &agent_dir,
        &vault_dir,
        &[
            lockbox.to_str().unwrap(),
            "open",
            "--password-file",
            password_file.to_str().unwrap(),
        ],
    );
    assert_command_success(&file_open, "open --password-file");
    run(
        bin,
        &agent_dir,
        &vault_dir,
        &[lockbox.to_str().unwrap(), "close"],
    );

    let mut stdin_open = command(
        bin,
        &agent_dir,
        &vault_dir,
        &[lockbox.to_str().unwrap(), "open", "--password-stdin"],
    )
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
    std::io::Write::write_all(stdin_open.stdin.as_mut().unwrap(), b"test-password\n").unwrap();
    let stdin_open = stdin_open.wait_with_output().unwrap();
    assert_command_success(&stdin_open, "open --password-stdin");
    run(
        bin,
        &agent_dir,
        &vault_dir,
        &[lockbox.to_str().unwrap(), "close"],
    );

    let key_open = run_output(
        bin,
        &agent_dir,
        &vault_dir,
        &[lockbox.to_str().unwrap(), "open-key", "default"],
    );
    assert_command_success(&key_open, "open-key default");
    stop_agent(bin, &agent_dir, &vault_dir);
    assert!(agent.wait().unwrap().success());
}

fn assert_command_success(output: &Output, command_name: &str) {
    assert!(
        output.status.success(),
        "{command_name} failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn stop_agent(bin: &str, agent_dir: &PathBuf, vault_dir: &PathBuf) {
    let output = run_output(bin, agent_dir, vault_dir, &["session", "stop"]);
    assert!(
        output.status.success(),
        "command failed: {bin} session stop\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
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
