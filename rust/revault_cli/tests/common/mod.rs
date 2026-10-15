#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Capture output in files so descendants retaining pipe handles cannot hang
/// the test after the CLI exits. Bound the CLI itself independently of CI.
pub trait CommandTestExt {
    fn test_output(&mut self) -> std::io::Result<std::process::Output>;
    fn test_output_with_input(&mut self, input: &[u8]) -> std::io::Result<std::process::Output>;
}

impl CommandTestExt for std::process::Command {
    fn test_output(&mut self) -> std::io::Result<std::process::Output> {
        self.test_output_with_input(&[])
    }

    fn test_output_with_input(&mut self, input: &[u8]) -> std::io::Result<std::process::Output> {
        use std::io::{Read, Seek, Write};
        use std::process::Stdio;
        use std::time::{Duration, Instant};
        // Persistence E2Es run without assuming a desktop/system bus. Tests
        // of suspend capability explicitly override these policy values.
        for key in [
            "LOCKBOX_AGENT_TERMINATE_ON_SUSPEND",
            "LOCKBOX_AGENT_PREVENT_SLEEP",
        ] {
            if !self
                .get_envs()
                .any(|(name, _)| name == std::ffi::OsStr::new(key))
            {
                self.env(key, "false");
            }
        }
        let mut stdout = tempfile::tempfile()?;
        let mut stderr = tempfile::tempfile()?;
        self.stdout(Stdio::from(stdout.try_clone()?));
        self.stderr(Stdio::from(stderr.try_clone()?));
        let mut stdin = tempfile::tempfile()?;
        stdin.write_all(input)?;
        stdin.rewind()?;
        self.stdin(Stdio::from(stdin));
        let started = Instant::now();
        let mut child = self.spawn()?;
        let thread = std::thread::current();
        let test = thread.name().unwrap_or("unnamed test");
        eprintln!("CLI start: test={test} pid={}", child.id());
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if started.elapsed() >= Duration::from_secs(180) {
                let _ = child.kill();
                let _ = child.wait();
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("CLI timed out after 180s: test={test} pid={}", child.id()),
                ));
            }
            std::thread::sleep(Duration::from_millis(25));
        };
        eprintln!(
            "CLI end: test={test} pid={} elapsed={:?} status={status}",
            child.id(),
            started.elapsed()
        );
        stdout.rewind()?;
        stderr.rewind()?;
        let mut out = Vec::new();
        let mut err = Vec::new();
        stdout.read_to_end(&mut out)?;
        stderr.read_to_end(&mut err)?;
        Ok(std::process::Output {
            status,
            stdout: out,
            stderr: err,
        })
    }
}
static TEST_RUN_ID: LazyLock<u128> = LazyLock::new(|| {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after the Unix epoch")
        .as_nanos()
});

pub struct TestTempDir {
    dir: tempfile::TempDir,
}

impl TestTempDir {
    pub fn new(prefix: &str) -> Self {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/test-tmp");
        std::fs::create_dir_all(&base).unwrap();
        let dir = tempfile::Builder::new()
            .prefix(&format!("{prefix}-{}-", std::process::id()))
            .tempdir_in(base)
            .unwrap();
        Self { dir }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }
}

pub fn unique_thread_dir_path(prefix: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../target/test-tmp")
        .join(format!(
            "{prefix}-{}-{:x}-{:?}",
            std::process::id(),
            *TEST_RUN_ID,
            std::thread::current().id()
        ))
}

pub fn unique_dir_path(prefix: &str, label: &str) -> PathBuf {
    let counter = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../target/test-tmp")
        .join(format!(
            "{prefix}-{label}-{}-{:x}-{counter}",
            std::process::id(),
            *TEST_RUN_ID
        ))
}

pub fn short_dir_path(label: &str) -> PathBuf {
    let counter = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../target/t")
        .join(format!(
            "lb-{label}-{}-{:x}-{counter}",
            std::process::id(),
            *TEST_RUN_ID
        ))
}
// Unix-domain socket paths have a small OS limit. Keep sockets outside long
// checkout/test names while retaining each test's agent log beside its vault.
pub fn agent_socket_dir(root: &Path) -> PathBuf {
    #[cfg(unix)]
    {
        use std::sync::{LazyLock, Mutex};
        static DIRECTORIES: LazyLock<
            Mutex<std::collections::BTreeMap<PathBuf, tempfile::TempDir>>,
        > = LazyLock::new(|| Mutex::new(std::collections::BTreeMap::new()));
        std::fs::create_dir_all(root).unwrap();
        let mut directories = DIRECTORIES.lock().unwrap();
        directories
            .entry(root.to_owned())
            .or_insert_with(|| {
                tempfile::Builder::new()
                    .prefix("rv-agent-")
                    .tempdir_in("/tmp")
                    .unwrap()
            })
            .path()
            .to_owned()
    }
    #[cfg(not(unix))]
    root.to_owned()
}

/// Owns only a test's private agent namespace and optional synthetic Vault.
/// Install before initialization so assertion failures also stop the agent.
pub struct TestAgentGuard {
    binary: PathBuf,
    agent: PathBuf,
    vault: Option<PathBuf>,
}

impl TestAgentGuard {
    pub fn new(agent: &Path, vault: Option<&Path>) -> Self {
        eprintln!("Owned test agent namespace: {}", agent.display());
        Self {
            binary: std::env::var_os("REVAULT_HEADLESS_CLI")
                .or_else(|| option_env!("CARGO_BIN_EXE_lockbox").map(Into::into))
                .map(PathBuf::from)
                .expect("owned agent cleanup requires the supplied installed or Cargo test CLI"),
            agent: agent.to_owned(),
            vault: vault.map(Path::to_owned),
        }
    }

    fn cleanup(&self) -> std::io::Result<()> {
        use std::process::{Command, Stdio};
        use std::time::{Duration, Instant};
        let mut command = Command::new(&self.binary);
        command
            .args(["session", "stop"])
            .env("LOCKBOX_SESSION_AGENT_DIR", &self.agent)
            .env("LOCKBOX_AGENT_TERMINATE_ON_SUSPEND", "false")
            .env("LOCKBOX_AGENT_PREVENT_SLEEP", "false")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(vault) = &self.vault {
            command.env("LOCKBOX_VAULT_DIR", vault);
        }
        let mut child = command.spawn()?;
        let started = Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                if !status.success() {
                    return Err(std::io::Error::other(format!(
                        "owned test agent stop failed: {status}"
                    )));
                }
                break;
            }
            if started.elapsed() >= Duration::from_secs(15) {
                // This is the child we just spawned, never the agent or an
                // unrelated PID. Keep the socket endpoint if shutdown was
                // unconfirmed; an enclosing TempDir may still remove its Vault.
                let _ = child.kill();
                let _ = child.wait();
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "owned test agent stop timed out",
                ));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let mut cleanup_error = None;
        for path in std::iter::once(&self.agent).chain(self.vault.iter()) {
            match std::fs::remove_dir_all(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    if cleanup_error.is_none() {
                        cleanup_error = Some(error);
                    }
                }
            }
        }
        if let Some(error) = cleanup_error {
            return Err(error);
        }
        eprintln!("Owned test agent stopped and fixture directories removed");
        Ok(())
    }
}

impl Drop for TestAgentGuard {
    fn drop(&mut self) {
        if let Err(error) = self.cleanup() {
            if std::thread::panicking() {
                eprintln!(
                    "Owned test agent cleanup failed during unwind: {}, endpoint={}",
                    error.kind(),
                    self.agent.display()
                );
            } else {
                panic!(
                    "owned test agent cleanup failed: {}, endpoint={}",
                    error.kind(),
                    self.agent.display()
                );
            }
        }
    }
}
