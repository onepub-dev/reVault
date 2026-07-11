#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

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
            "{prefix}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
}

pub fn unique_dir_path(prefix: &str, label: &str) -> PathBuf {
    let counter = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../target/test-tmp")
        .join(format!("{prefix}-{label}-{}-{counter}", std::process::id()))
}

pub fn short_dir_path(label: &str) -> PathBuf {
    let counter = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../target/t")
        .join(format!("lb-{label}-{}-{counter}", std::process::id()))
}
