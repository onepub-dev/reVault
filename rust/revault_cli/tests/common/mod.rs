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
}

impl CommandTestExt for std::process::Command {
    fn test_output(&mut self) -> std::io::Result<std::process::Output> {
        use std::io::{Read, Seek};
        use std::process::Stdio;
        use std::time::{Duration, Instant};
        let mut stdout = tempfile::tempfile()?;
        let mut stderr = tempfile::tempfile()?;
        self.stdout(Stdio::from(stdout.try_clone()?));
        self.stderr(Stdio::from(stderr.try_clone()?));
        self.stdin(Stdio::null());
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
