use crate::command::{self, TaskResult};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

struct Options {
    lockbox: PathBuf,
    work_dir: PathBuf,
    sleep_command: Option<String>,
    no_sleep: bool,
}

pub fn run(args: &[String]) -> TaskResult {
    let Some(options) = parse(args)? else {
        return Ok(());
    };
    let platform = env::consts::OS;
    if platform != "linux" && platform != "macos" {
        return Err(format!("unsupported Unix platform: {platform}"));
    }
    let sleep_command = options.sleep_command.unwrap_or_else(|| {
        if platform == "linux" {
            "sudo systemctl suspend".to_owned()
        } else {
            "pmset sleepnow".to_owned()
        }
    });

    if options.work_dir.exists() {
        fs::remove_dir_all(&options.work_dir)
            .map_err(|error| format!("cannot remove {}: {error}", options.work_dir.display()))?;
    }
    let agent_dir = options.work_dir.join("agent");
    let vault_dir = options.work_dir.join("vault");
    let agent_log = options.work_dir.join("agent.log");
    fs::create_dir_all(&agent_dir)
        .map_err(|error| format!("cannot create work directory: {error}"))?;
    fs::create_dir_all(&vault_dir)
        .map_err(|error| format!("cannot create vault directory: {error}"))?;

    let run_lockbox = |arguments: &[&str]| -> TaskResult<String> {
        let mut process = Command::new(&options.lockbox);
        process
            .args(arguments)
            .env("LOCKBOX_PASSWORD", "test-lockbox-password")
            .env("LOCKBOX_VAULT_PASSWORD", "test-vault-password")
            .env("LOCKBOX_SESSION_AGENT_DIR", &agent_dir)
            .env("LOCKBOX_SESSION_AGENT_LOG", &agent_log)
            .env("LOCKBOX_VAULT_DIR", &vault_dir);
        command::output_lossy(&mut process)
    };

    run_lockbox(&["vault", "init"])?;
    let lockbox_path = options.work_dir.join("test.lbox");
    run_lockbox(&["create", &lockbox_path.to_string_lossy()])?;
    run_lockbox(&["open", &lockbox_path.to_string_lossy()])?;
    let before = run_lockbox(&["vault", "sessions", "--format", "tsv"])?;
    if !before.lines().any(|line| line.starts_with("open")) {
        return Err(format!(
            "expected lockbox to be cached before sleep; sessions output:\n{before}"
        ));
    }
    require_log(&agent_log, "sleep watcher started")?;

    if platform == "linux" && command::exists_on_path("systemd-inhibit") {
        let inhibitors = command::output_lossy(
            command::command("systemd-inhibit").args(["--list", "--no-pager"]),
        )?;
        if !(inhibitors.contains("lockbox")
            && inhibitors.contains("sleep")
            && inhibitors.contains("Clear cached lockbox keys"))
        {
            return Err("logind delay inhibitor is not registered".to_owned());
        }
    }

    if options.no_sleep {
        println!("prepared: cache is populated and sleep watcher is active");
        println!("log: {}", agent_log.display());
        return Ok(());
    }

    println!("sleeping now; resume the VM if the hypervisor does not do it automatically");
    command::run(Command::new("sh").args(["-c", &sleep_command]))?;
    thread::sleep(Duration::from_secs(5));
    let after = run_lockbox(&["vault", "sessions", "--format", "tsv"])?;
    if after.trim() != "empty" {
        return Err(format!(
            "expected cache to be empty after resume; sessions output:\n{after}"
        ));
    }
    require_log(&agent_log, "suspend requested; cleared")?;
    println!("pass: cache cleared on sleep");
    println!("log: {}", agent_log.display());
    Ok(())
}

fn require_log(path: &Path, needle: &str) -> TaskResult {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("cannot read agent log {}: {error}", path.display()))?;
    if contents.contains(needle) {
        Ok(())
    } else {
        Err(format!("agent log did not contain {needle:?}:\n{contents}"))
    }
}

fn parse(args: &[String]) -> TaskResult<Option<Options>> {
    let mut lockbox = None;
    let mut work_dir = env::temp_dir().join("lockbox-agent-sleep-test");
    let mut sleep_command = None;
    let mut no_sleep = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--bin" => {
                lockbox = Some(PathBuf::from(command::option_value(
                    args, &mut index, "--bin",
                )?))
            }
            "--work-dir" => {
                work_dir = PathBuf::from(command::option_value(args, &mut index, "--work-dir")?)
            }
            "--sleep-command" => {
                sleep_command = Some(command::option_value(args, &mut index, "--sleep-command")?)
            }
            "--no-sleep" => no_sleep = true,
            "-h" | "--help" => {
                println!("Usage: cargo xtask agent-sleep-unix --bin <lockbox> [--work-dir DIR] [--sleep-command CMD] [--no-sleep]");
                return Ok(None);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        index += 1;
    }
    Ok(Some(Options {
        lockbox: lockbox.ok_or_else(|| "--bin is required".to_owned())?,
        work_dir,
        sleep_command,
        no_sleep,
    }))
}
