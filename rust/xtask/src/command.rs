use std::env;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

pub type TaskResult<T = ()> = Result<T, String>;

pub fn workspace_root() -> TaskResult<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask has no workspace parent directory".to_owned())
}

pub fn repo_root() -> TaskResult<PathBuf> {
    workspace_root()?
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "Rust workspace has no repository parent directory".to_owned())
}

pub fn command(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    if let Ok(root) = workspace_root() {
        command.current_dir(root);
    }
    command
}

pub fn run(command: &mut Command) -> TaskResult {
    eprintln!("+ {}", display(command));
    let status = command
        .status()
        .map_err(|error| format!("failed to start {}: {error}", display(command)))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited with {status}", display(command)))
    }
}

pub fn output(command: &mut Command) -> TaskResult<Output> {
    eprintln!("+ {}", display(command));
    let output = command
        .output()
        .map_err(|error| format!("failed to start {}: {error}", display(command)))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(format!(
            "{} exited with {}: {}",
            display(command),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

pub fn output_lossy(command: &mut Command) -> TaskResult<String> {
    output(command).map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

pub fn exists_on_path(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

pub fn require_commands(programs: &[&str]) -> TaskResult {
    for program in programs {
        if !exists_on_path(program) {
            return Err(format!("missing required command: {program}"));
        }
    }
    Ok(())
}

pub fn require_file(path: &Path) -> TaskResult {
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("missing required file: {}", path.display()))
    }
}

pub fn display(command: &Command) -> String {
    let mut rendered = command.get_program().to_string_lossy().into_owned();
    for argument in command.get_args() {
        rendered.push(' ');
        rendered.push_str(&argument.to_string_lossy());
    }
    rendered
}

pub fn option_value(args: &[String], index: &mut usize, option: &str) -> TaskResult<String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("missing value for {option}"))
}

pub fn run_and_tee(command: &mut Command, log_path: &Path) -> TaskResult {
    eprintln!("+ {}", display(command));
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to start {}: {error}", display(command)))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout was not piped".to_owned())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "stderr was not piped".to_owned())?;
    let log = Arc::new(Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map_err(|error| format!("cannot open {}: {error}", log_path.display()))?,
    ));

    let pump = |reader: Box<dyn std::io::Read + Send>, to_stderr: bool, log: Arc<Mutex<File>>| {
        thread::spawn(move || -> TaskResult {
            for line in BufReader::new(reader).lines() {
                let line = line.map_err(|error| format!("cannot read command output: {error}"))?;
                if to_stderr {
                    eprintln!("{line}");
                } else {
                    println!("{line}");
                }
                let mut log = log
                    .lock()
                    .map_err(|_| "performance log lock poisoned".to_owned())?;
                writeln!(log, "{line}").map_err(|error| format!("cannot write log: {error}"))?;
            }
            Ok(())
        })
    };
    let out_thread = pump(Box::new(stdout), false, Arc::clone(&log));
    let err_thread = pump(Box::new(stderr), true, log);
    let status = child
        .wait()
        .map_err(|error| format!("cannot wait for command: {error}"))?;
    out_thread
        .join()
        .map_err(|_| "stdout pump panicked".to_owned())??;
    err_thread
        .join()
        .map_err(|_| "stderr pump panicked".to_owned())??;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited with {status}", display(command)))
    }
}
