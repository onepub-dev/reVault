use crate::command::TaskResult;

#[cfg(not(windows))]
pub fn run(args: &[String]) -> TaskResult {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!("Usage: xtask.exe agent-sleep-windows-vm --bin <lockbox.exe> [--work-dir DIR] [--no-sleep]");
        Ok(())
    } else {
        Err("agent-sleep-windows-vm must run on Windows".to_owned())
    }
}

#[cfg(windows)]
pub fn run(args: &[String]) -> TaskResult {
    use crate::command;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    let mut lockbox = None;
    let mut work_dir = env::temp_dir().join("lockbox-agent-sleep-test");
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
            "--no-sleep" => no_sleep = true,
            "-h" | "--help" => {
                println!("Usage: xtask.exe agent-sleep-windows-vm --bin <lockbox.exe> [--work-dir DIR] [--no-sleep]");
                return Ok(());
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        index += 1;
    }
    let lockbox = lockbox.ok_or_else(|| "--bin is required".to_owned())?;
    if work_dir.exists() {
        fs::remove_dir_all(&work_dir)
            .map_err(|error| format!("cannot clear {}: {error}", work_dir.display()))?;
    }
    let agent_dir = work_dir.join("agent");
    let vault_dir = work_dir.join("vault");
    let agent_log = work_dir.join("agent.log");
    fs::create_dir_all(&agent_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&vault_dir).map_err(|error| error.to_string())?;

    let process_name = lockbox
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("lockbox");
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", &format!("{process_name}.exe")])
        .status();
    if !no_sleep {
        command::run(Command::new("powercfg").args(["/hibernate", "off"]))?;
        let states = command::output_lossy(Command::new("powercfg").arg("/a"))?;
        if !(states.contains("Standby (S0")
            || states.contains("Standby (S1")
            || states.contains("Standby (S2")
            || states.contains("Standby (S3"))
        {
            return Err("this Windows VM does not expose a usable standby sleep state".to_owned());
        }
    }

    let invoke = |arguments: &[&str]| -> TaskResult<String> {
        let mut process = Command::new(&lockbox);
        process
            .args(arguments)
            .env("LOCKBOX_PASSWORD", "test-lockbox-password")
            .env("LOCKBOX_VAULT_PASSWORD", "test-vault-password")
            .env("LOCKBOX_SESSION_AGENT_DIR", &agent_dir)
            .env("LOCKBOX_SESSION_AGENT_LOG", &agent_log)
            .env("LOCKBOX_VAULT_DIR", &vault_dir);
        command::output_lossy(&mut process)
    };
    invoke(&["vault", "init"])?;
    let lockbox_path = work_dir.join("test.lbox");
    invoke(&["create", &lockbox_path.to_string_lossy()])?;
    invoke(&["open", &lockbox_path.to_string_lossy()])?;
    let before = invoke(&["vault", "sessions", "--format", "tsv"])?;
    if !before.contains("open") {
        return Err("expected lockbox to be cached before sleep".to_owned());
    }
    let log = fs::read_to_string(&agent_log).map_err(|error| error.to_string())?;
    if !log.contains("sleep watcher started") {
        return Err("agent log did not show a running sleep watcher".to_owned());
    }
    if no_sleep {
        println!("prepared: cache is populated and sleep watcher is active");
        println!("log: {}", agent_log.display());
        return Ok(());
    }
    println!("sleeping now; resume the VM if the hypervisor does not do it automatically");
    command::run(
        Command::new("rundll32.exe")
            .arg("powrprof.dll,SetSuspendState")
            .args(["0", "1", "0"]),
    )?;
    thread::sleep(Duration::from_secs(8));
    let after = invoke(&["vault", "sessions", "--format", "tsv"])?;
    if after.trim() != "empty" {
        return Err(format!(
            "expected cache to be empty after resume; sessions output: {after}"
        ));
    }
    let log = fs::read_to_string(&agent_log).map_err(|error| error.to_string())?;
    if !log.contains("suspend requested; cleared") {
        return Err("agent log did not show suspend cache clearing".to_owned());
    }
    println!("pass: cache cleared on sleep");
    println!("log: {}", agent_log.display());
    Ok(())
}
