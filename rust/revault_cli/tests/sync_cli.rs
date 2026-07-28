mod common;

use common::TestTempDir;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run(bin: &str, cwd: &Path, args: &[&str]) -> Output {
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("LOCKBOX_KEY", "sync-test-content-key")
        .env("LOCKBOX_VAULT_DIR", cwd.join("vault"))
        .env("LOCKBOX_VAULT_PASSWORD", "sync-test-vault-password")
        .env("LOCKBOX_SESSION_AGENT_DIR", cwd.join("agent"))
        .env("LOCKBOX_SESSION_AGENT_LOG", cwd.join("agent.log"))
        .env("LOCKBOX_ADD_PROGRESS", "off")
        .output()
        .unwrap()
}

fn success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn sync_adds_replaces_deletes_and_persists_source_identity() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("sync-flow");
    let source = temp.path().join("project");
    let other = temp.path().join("other");
    let lockbox = temp.path().join("backup.lbox");
    fs::create_dir_all(source.join("src")).unwrap();
    fs::create_dir_all(&other).unwrap();
    fs::write(source.join("README.md"), "one\n").unwrap();
    fs::write(source.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(source.join("ignored.tmp"), "ignored\n").unwrap();

    success(&run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "create"],
    ));
    let dry_run = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "sync",
            "project",
            "--to",
            "/project",
            "--exclude",
            "*.tmp",
            "--dry-run",
            "--format",
            "json",
        ],
    );
    success(&dry_run);
    let dry_run = String::from_utf8_lossy(&dry_run.stdout);
    assert!(dry_run.contains("\"dry_run\":true"));
    assert!(dry_run.contains("/project/README.md"));
    assert!(!dry_run.contains("ignored.tmp"));

    let initial = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "sync",
            "project",
            "--to",
            "/project",
            "--exclude",
            "*.tmp",
            "--force",
        ],
    );
    success(&initial);
    assert!(String::from_utf8_lossy(&initial.stdout).contains("2 added"));

    fs::write(source.join("README.md"), "two\n").unwrap();
    fs::remove_file(source.join("src/main.rs")).unwrap();
    fs::write(source.join("new.txt"), "new\n").unwrap();
    let update = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "sync",
            "project",
            "--to",
            "/project",
            "--exclude",
            "*.tmp",
            "--delete",
            "--force",
        ],
    );
    success(&update);
    let update = String::from_utf8_lossy(&update.stdout);
    assert!(update.contains("1 added"));
    assert!(update.contains("1 replaced"));
    assert!(update.contains("1 removed"));

    let listing = run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "list", "-R", "/project"],
    );
    success(&listing);
    let listing = String::from_utf8_lossy(&listing.stdout);
    assert!(listing.contains("/project/README.md"));
    assert!(listing.contains("/project/new.txt"));
    assert!(!listing.contains("main.rs"));
    assert!(!listing.contains("ignored.tmp"));

    fs::remove_file(source.join("README.md")).unwrap();
    fs::create_dir(source.join("README.md")).unwrap();
    fs::write(source.join("README.md/nested.txt"), "nested\n").unwrap();
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "sync",
            "project",
            "--to",
            "/project",
            "--exclude",
            "*.tmp",
            "--delete",
            "--force",
        ],
    ));
    fs::remove_dir_all(source.join("README.md")).unwrap();
    fs::write(source.join("README.md"), "file again\n").unwrap();
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "sync",
            "project",
            "--to",
            "/project",
            "--exclude",
            "*.tmp",
            "--delete",
            "--force",
        ],
    ));

    fs::write(other.join("README.md"), "wrong source\n").unwrap();
    let wrong_source = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "sync",
            "other",
            "--to",
            "/project",
            "--exclude",
            "*.tmp",
            "--dry-run",
        ],
    );
    assert!(!wrong_source.status.success());
    assert!(String::from_utf8_lossy(&wrong_source.stderr).contains("--rebind-source"));
}

#[test]
fn sync_rejects_empty_and_large_deletion_without_overrides() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("sync-delete-guards");
    let source = temp.path().join("source");
    let lockbox = temp.path().join("backup.lbox");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("a"), "a").unwrap();
    fs::write(source.join("b"), "b").unwrap();

    success(&run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "create"],
    ));
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "sync",
            source.to_str().unwrap(),
            "--to",
            "/data",
            "--force",
        ],
    ));
    fs::remove_file(source.join("a")).unwrap();
    fs::remove_file(source.join("b")).unwrap();

    let empty = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "sync",
            source.to_str().unwrap(),
            "--to",
            "/data",
            "--delete",
            "--force",
        ],
    );
    assert!(!empty.status.success());
    assert!(String::from_utf8_lossy(&empty.stderr).contains("--allow-empty"));

    let large = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "sync",
            source.to_str().unwrap(),
            "--to",
            "/data",
            "--delete",
            "--allow-empty",
            "--force",
        ],
    );
    assert!(!large.status.success());
    assert!(String::from_utf8_lossy(&large.stderr).contains("--allow-large-delete"));
}
