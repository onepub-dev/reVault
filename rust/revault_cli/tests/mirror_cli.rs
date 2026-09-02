mod common;

use common::TestTempDir;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run(bin: &str, cwd: &Path, args: &[&str]) -> Output {
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("LOCKBOX_KEY", "mirror-test-content-key")
        .env("LOCKBOX_VAULT_DIR", cwd.join("vault"))
        .env("LOCKBOX_VAULT_PASSWORD", "mirror-test-vault-password")
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
fn mirror_create_rules_status_update_and_ownership() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("mirror-flow");
    let source = temp.path().join("project");
    let lockbox = temp.path().join("backup.lbox");
    fs::create_dir_all(source.join("src")).unwrap();
    fs::create_dir(source.join("empty")).unwrap();
    fs::write(source.join("README.md"), "one\n").unwrap();
    fs::write(source.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(source.join("ignored.tmp"), "ignored\n").unwrap();

    success(&run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "create"],
    ));
    for args in [
        vec![
            lockbox.to_str().unwrap(),
            "mirror",
            "create",
            "home",
            "--from",
            "project",
            "--to",
            "/home",
        ],
        vec![
            lockbox.to_str().unwrap(),
            "mirror",
            "home",
            "create",
            "home",
            "--from",
            "project",
            "--to",
            "/home",
        ],
    ] {
        let misplaced = run(bin, temp.path(), &args);
        assert!(!misplaced.status.success());
        assert!(String::from_utf8_lossy(&misplaced.stderr)
            .contains("lbx mirror home create --from <HOST_DIRECTORY>"));
    }
    let created = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "project",
            "create",
            "--from",
            "project",
            "--to",
            "/projects/project",
            "--strict",
        ],
    );
    success(&created);
    assert!(String::from_utf8_lossy(&created.stdout).contains("No files were copied"));
    assert!(String::from_utf8_lossy(&created.stdout).contains(&format!(
        "lbx {} mirror project update",
        lockbox.to_string_lossy()
    )));

    let projects = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "projects",
            "--format",
            "json",
        ],
    );
    success(&projects);
    assert!(String::from_utf8_lossy(&projects.stdout).contains("project"));
    let info = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "project",
            "info",
            "--format",
            "json",
        ],
    );
    success(&info);
    let info: serde_json::Value = serde_json::from_slice(&info.stdout).unwrap();
    assert_eq!(info["destination"], "/projects/project");
    assert_eq!(info["strict"], true);

    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "project",
            "configure",
            "--no-strict",
        ],
    ));
    let info = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "project",
            "info",
            "--format",
            "json",
        ],
    );
    success(&info);
    let info: serde_json::Value = serde_json::from_slice(&info.stdout).unwrap();
    assert_eq!(info["strict"], false);

    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "project",
            "rule",
            "add",
            "exclude",
            "*.tmp",
        ],
    ));
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "project",
            "rule",
            "remove",
            "exclude",
            "*.tmp",
        ],
    ));
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "project",
            "rule",
            "add",
            "exclude",
            "*.tmp",
        ],
    ));
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "project",
            "rule",
            "clear",
            "include",
        ],
    ));
    let rules = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "rule",
            "list",
            "exclude",
            "--format",
            "json",
        ],
    );
    success(&rules);
    assert!(String::from_utf8_lossy(&rules.stdout).contains("*.tmp"));

    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "project",
            "rebind",
            "--from",
            "project",
            "--force",
        ],
    ));

    let status = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "status",
            "--format",
            "json",
        ],
    );
    success(&status);
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(status_stdout.contains("\"add\":["));
    assert!(status_stdout.contains("/projects/project/README.md"));
    assert!(status_stdout.contains("/projects/project/src/main.rs"));
    let status_stderr = String::from_utf8_lossy(&status.stderr);
    assert!(status_stderr.contains("Mirror: scanning source"));
    assert!(status_stderr.contains("Mirror: comparison complete."));

    let quiet_status = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "status",
            "--quiet",
            "--format",
            "json",
        ],
    );
    success(&quiet_status);
    assert!(String::from_utf8_lossy(&quiet_status.stderr).is_empty());
    assert!(String::from_utf8_lossy(&quiet_status.stdout).contains("\"add\":["));

    let before = run(bin, temp.path(), &[lockbox.to_str().unwrap(), "list", "-R"]);
    success(&before);
    assert!(!String::from_utf8_lossy(&before.stdout).contains("README.md"));

    let update = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "update",
            "--force",
            "--allow-empty",
            "--allow-large-delete",
        ],
    );
    success(&update);
    let update_stderr = String::from_utf8_lossy(&update.stderr);
    assert!(update_stderr.contains("Mirror: applying"));
    assert!(update_stderr.contains("Mirror: committing the encrypted update."));
    assert!(update_stderr.contains("Mirror: update complete."));

    let quiet_update = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "update",
            "--force",
            "--quiet",
        ],
    );
    success(&quiet_update);
    assert!(String::from_utf8_lossy(&quiet_update.stderr).is_empty());
    assert!(String::from_utf8_lossy(&quiet_update.stdout).contains("is up to date"));
    let listing = run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "list", "-R"],
    );
    success(&listing);
    let listing = String::from_utf8_lossy(&listing.stdout);
    assert!(listing.contains("README.md"));
    assert!(listing.contains("src/main.rs"));
    assert!(listing.contains("empty"));
    assert!(!listing.contains("ignored.tmp"));

    fs::write(temp.path().join("manual.txt"), "manual\n").unwrap();
    let blocked = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "add",
            "manual.txt",
            "--to",
            "/projects/project/manual.txt",
        ],
    );
    assert!(!blocked.status.success());
    assert!(String::from_utf8_lossy(&blocked.stderr).contains("managed by mirror"));

    let scoped = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "add",
            "manual.txt",
            "--to",
            "manual.txt",
        ],
    );
    success(&scoped);
    let diverged = run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "status"],
    );
    success(&diverged);
    assert!(String::from_utf8_lossy(&diverged.stdout).contains("remove:    1 files"));

    let rejected_dry_run = run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "update", "--dry-run"],
    );
    assert!(!rejected_dry_run.status.success());
}

#[test]
fn populated_mirror_update_commits_without_recovery() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("mirror-populated-update");
    let source = temp.path().join("house");
    let lockbox = temp.path().join("system-api-keys.lbox");
    fs::create_dir_all(&source).unwrap();
    for directory in 0..4 {
        let child = source.join(format!("directory-{directory}"));
        fs::create_dir_all(&child).unwrap();
        for file in 0..23 {
            fs::write(
                child.join(format!("file-{file:02}.txt")),
                format!("directory {directory}, file {file}\n"),
            )
            .unwrap();
        }
    }

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
            "mirror",
            "house",
            "create",
            "--from",
            source.to_str().unwrap(),
            "--to",
            "house",
        ],
    ));

    let update = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "house",
            "update",
            "--force",
        ],
    );
    success(&update);
    assert!(!String::from_utf8_lossy(&update.stderr).contains("recovery"));
    assert!(String::from_utf8_lossy(&update.stdout).contains("92 added"));

    let first_listing = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "house",
            "list",
            "--recursive",
            "--format",
            "json",
        ],
    );
    success(&first_listing);
    assert!(String::from_utf8_lossy(&first_listing.stdout).contains("file-00.txt"));
    let first_stored = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "house",
            "cat",
            "directory-0/file-00.txt",
        ],
    );
    success(&first_stored);
    assert_eq!(
        first_stored.stdout,
        fs::read(source.join("directory-0/file-00.txt")).unwrap()
    );

    let status = run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "house", "status"],
    );
    success(&status);
    assert!(String::from_utf8_lossy(&status.stdout).contains("unchanged: 92 files"));

    fs::write(
        source.join("directory-0/file-00.txt"),
        b"replacement content\n",
    )
    .unwrap();
    fs::remove_file(source.join("directory-0/file-01.txt")).unwrap();
    fs::write(source.join("directory-0/new.txt"), b"new content\n").unwrap();
    let changed = run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "house", "status"],
    );
    success(&changed);
    let changed_stdout = String::from_utf8_lossy(&changed.stdout);
    assert!(changed_stdout.contains("add:       1 files"));
    assert!(changed_stdout.contains("replace:   1 files"));
    assert!(changed_stdout.contains("remove:    1 files"));
    assert!(changed_stdout.contains("unchanged: 90 files"));
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "house",
            "update",
            "--force",
        ],
    ));
    for (stored_path, expected) in [
        (
            "directory-0/file-00.txt",
            b"replacement content\n".as_slice(),
        ),
        ("directory-0/new.txt", b"new content\n".as_slice()),
    ] {
        let stored = run(
            bin,
            temp.path(),
            &[
                lockbox.to_str().unwrap(),
                "mirror",
                "house",
                "cat",
                stored_path,
            ],
        );
        success(&stored);
        assert_eq!(stored.stdout, expected);
    }
    let removed = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "house",
            "cat",
            "directory-0/file-01.txt",
        ],
    );
    assert!(!removed.status.success());
    let repeated = run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "house", "status"],
    );
    success(&repeated);
    assert!(String::from_utf8_lossy(&repeated.stdout).contains("unchanged: 92 files"));

    fs::remove_dir_all(source.join("directory-0")).unwrap();
    fs::remove_dir_all(source.join("directory-1")).unwrap();
    fs::remove_file(source.join("directory-2/file-00.txt")).unwrap();
    let large_delete = run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "house", "status"],
    );
    success(&large_delete);
    let large_delete_stdout = String::from_utf8_lossy(&large_delete.stdout);
    assert!(large_delete_stdout.contains("remove:    47 files"));
    assert!(large_delete_stdout.contains("unchanged: 45 files"));
    let refused = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "house",
            "update",
            "--force",
        ],
    );
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("--allow-large-delete"));
    let still_stored = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "house",
            "cat",
            "directory-1/file-00.txt",
        ],
    );
    success(&still_stored);
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "house",
            "update",
            "--force",
            "--allow-large-delete",
        ],
    ));
    let after_large_delete = run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "house", "status"],
    );
    success(&after_large_delete);
    assert!(String::from_utf8_lossy(&after_large_delete.stdout).contains("unchanged: 45 files"));

    for entry in fs::read_dir(&source).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            fs::remove_dir_all(path).unwrap();
        } else {
            fs::remove_file(path).unwrap();
        }
    }
    let refused_empty = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "house",
            "update",
            "--force",
            "--allow-large-delete",
        ],
    );
    assert!(!refused_empty.status.success());
    assert!(String::from_utf8_lossy(&refused_empty.stderr).contains("--allow-empty"));
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "house",
            "update",
            "--force",
            "--allow-empty",
            "--allow-large-delete",
        ],
    ));
    let empty_listing = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "house",
            "list",
            "--recursive",
        ],
    );
    success(&empty_listing);
    assert_eq!(
        String::from_utf8_lossy(&empty_listing.stdout).trim(),
        "empty"
    );
}

#[test]
fn mirror_excludes_the_lockbox_and_its_active_lock_file_from_its_source() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("mirror-self-exclusion");
    let source = temp.path().join("house");
    let lockbox = source.join("house.lbox");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("ordinary.txt"), b"ordinary content\n").unwrap();

    success(&run(bin, temp.path(), &["house/house.lbox", "create"]));
    success(&run(
        bin,
        temp.path(),
        &[
            "house/house.lbox",
            "mirror",
            "home",
            "create",
            "--from",
            "house",
            "--to",
            "home",
        ],
    ));

    let status = run(
        bin,
        temp.path(),
        &[
            "house/house.lbox",
            "mirror",
            "home",
            "status",
            "--format",
            "json",
        ],
    );
    success(&status);
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(status_stdout.contains("ordinary.txt"));
    assert!(!status_stdout.contains("house.lbox"));
    assert!(!status_stdout.contains(".house.lbox.lock"));

    success(&run(
        bin,
        temp.path(),
        &["house/house.lbox", "mirror", "home", "update", "--force"],
    ));

    let listing = run(
        bin,
        temp.path(),
        &[
            "house/house.lbox",
            "mirror",
            "home",
            "list",
            "--recursive",
            "--format",
            "json",
        ],
    );
    success(&listing);
    let listing_stdout = String::from_utf8_lossy(&listing.stdout);
    assert!(listing_stdout.contains("ordinary.txt"));
    assert!(!listing_stdout.contains("house.lbox"));
    assert!(!listing_stdout.contains(".house.lbox.lock"));

    let stored = run(
        bin,
        temp.path(),
        &["house/house.lbox", "mirror", "home", "cat", "ordinary.txt"],
    );
    success(&stored);
    assert_eq!(
        stored.stdout,
        fs::read(source.join("ordinary.txt")).unwrap()
    );

    let repeated = run(
        bin,
        temp.path(),
        &["house/house.lbox", "mirror", "home", "status"],
    );
    success(&repeated);
    assert!(String::from_utf8_lossy(&repeated.stdout).contains("unchanged: 1 files"));
    assert!(lockbox.exists());
}

#[test]
fn mirror_large_delete_persists_after_reopen() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("mirror-large-delete");
    let source = temp.path().join("source");
    let lockbox = temp.path().join("backup.lbox");
    fs::create_dir_all(&source).unwrap();
    for name in ["a.txt", "b.txt", "c.txt", "keep.txt"] {
        fs::write(source.join(name), format!("{name}\n")).unwrap();
    }
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
            "mirror",
            "source",
            "create",
            "--from",
            source.to_str().unwrap(),
            "--to",
            "source",
        ],
    ));
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "source",
            "update",
            "--force",
        ],
    ));
    for name in ["a.txt", "b.txt", "c.txt"] {
        fs::remove_file(source.join(name)).unwrap();
    }
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "source",
            "update",
            "--force",
            "--allow-large-delete",
        ],
    ));

    let listing = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "source",
            "list",
            "--recursive",
            "--format",
            "json",
        ],
    );
    success(&listing);
    let listing = String::from_utf8_lossy(&listing.stdout);
    assert!(listing.contains("keep.txt"));
    assert!(!listing.contains("a.txt"));
    assert!(!listing.contains("b.txt"));
    assert!(!listing.contains("c.txt"));

    fs::remove_file(source.join("keep.txt")).unwrap();
    let refused_empty = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "source",
            "update",
            "--force",
            "--allow-large-delete",
        ],
    );
    assert!(!refused_empty.status.success());
    let refused_empty_stderr = String::from_utf8_lossy(&refused_empty.stderr);
    assert!(
        refused_empty_stderr.contains("--allow-empty"),
        "unexpected empty-source refusal: {refused_empty_stderr}"
    );

    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "source",
            "update",
            "--force",
            "--allow-empty",
            "--allow-large-delete",
        ],
    ));
    let empty = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "source",
            "list",
            "--recursive",
        ],
    );
    success(&empty);
    assert_eq!(String::from_utf8_lossy(&empty.stdout).trim(), "empty");
}

#[test]
fn multiple_mirrors_require_name_and_destinations_cannot_overlap() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("mirror-multiple");
    let lockbox = temp.path().join("backup.lbox");
    fs::create_dir_all(temp.path().join("one")).unwrap();
    fs::create_dir_all(temp.path().join("two")).unwrap();
    fs::create_dir_all(temp.path().join("adopt-source")).unwrap();
    fs::write(temp.path().join("existing.txt"), "existing\n").unwrap();
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
            "add",
            "existing.txt",
            "--to",
            "/adopt/existing.txt",
        ],
    ));
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "adopted",
            "create",
            "--from",
            "adopt-source",
            "--to",
            "/adopt",
            "--adopt",
        ],
    ));
    for (name, source, destination) in [("one", "one", "/one"), ("two", "two", "/two")] {
        success(&run(
            bin,
            temp.path(),
            &[
                lockbox.to_str().unwrap(),
                "mirror",
                name,
                "create",
                "--from",
                source,
                "--to",
                destination,
            ],
        ));
    }
    let ambiguous = run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "status"],
    );
    assert!(!ambiguous.status.success());
    assert!(String::from_utf8_lossy(&ambiguous.stderr).contains("more than one"));

    let overlap = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "nested",
            "create",
            "--from",
            "one",
            "--to",
            "/one/nested",
        ],
    );
    assert!(!overlap.status.success());
    assert!(String::from_utf8_lossy(&overlap.stderr).contains("overlaps"));
}

#[test]
fn mirror_retain_forget_and_delete_have_distinct_results() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("mirror-lifecycle");
    let lockbox = temp.path().join("backup.lbox");
    fs::create_dir_all(temp.path().join("source")).unwrap();
    fs::write(temp.path().join("source/a.txt"), "a").unwrap();
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
            "mirror",
            "docs",
            "create",
            "--from",
            "source",
            "--to",
            "/docs",
        ],
    ));
    success(&run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "update", "--force"],
    ));
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "configure",
            "--missing-files",
            "retain",
            "--strict",
        ],
    ));
    let configured = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "info",
            "--format",
            "json",
        ],
    );
    success(&configured);
    let configured: serde_json::Value = serde_json::from_slice(&configured.stdout).unwrap();
    assert_eq!(configured["missing_files"], "retain");
    assert_eq!(configured["strict"], true);
    fs::remove_file(temp.path().join("source/a.txt")).unwrap();
    success(&run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "update", "--force"],
    ));
    let retained = run(bin, temp.path(), &[lockbox.to_str().unwrap(), "list", "-R"]);
    success(&retained);
    assert!(String::from_utf8_lossy(&retained.stdout).contains("/docs/a.txt"));

    success(&run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "forget", "--force"],
    ));
    let unmanaged = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "remove",
            "--force",
            "/docs/a.txt",
        ],
    );
    success(&unmanaged);
}

#[test]
fn mirror_file_commands_match_archive_file_command_shapes_and_destroy_removes_root() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("mirror-file-commands");
    let lockbox = temp.path().join("backup.lbox");
    fs::create_dir_all(temp.path().join("source")).unwrap();
    fs::write(temp.path().join("manual.txt"), "manual\n").unwrap();
    fs::create_dir_all(temp.path().join("bulk/nested")).unwrap();
    fs::write(temp.path().join("bulk/nested/keep.txt"), "keep\n").unwrap();
    fs::write(temp.path().join("bulk/nested/skip.tmp"), "skip\n").unwrap();

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
            "mirror",
            "docs",
            "create",
            "--from",
            "source",
            "--to",
            "/docs",
        ],
    ));
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "--verbose",
            "mirror",
            "add",
            "bulk",
            "--recursive",
            "--to",
            "bulk",
            "--include",
            "*.txt",
            "--exclude",
            "*.tmp",
            "--jobs",
            "1",
            "--overwrite",
            "--quiet",
        ],
    ));
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "add",
            "manual.txt",
            "--to",
            "notes/manual.txt",
        ],
    ));
    let cat = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "cat",
            "notes/manual.txt",
        ],
    );
    success(&cat);
    assert_eq!(String::from_utf8_lossy(&cat.stdout), "manual\n");

    let json_listing = run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "list",
            "bulk",
            "--recursive",
            "--format",
            "json",
        ],
    );
    success(&json_listing);
    assert!(String::from_utf8_lossy(&json_listing.stdout).contains("keep.txt"));

    let selected_file = temp.path().join("selected-file.txt");
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "extract",
            "notes/manual.txt",
            "--to",
            selected_file.to_str().unwrap(),
        ],
    ));
    assert_eq!(fs::read_to_string(selected_file).unwrap(), "manual\n");

    let selected_directory = temp.path().join("selected-directory");
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "extract",
            "bulk/nested",
            "--to",
            selected_directory.to_str().unwrap(),
        ],
    ));
    assert_eq!(
        fs::read_to_string(selected_directory.join("keep.txt")).unwrap(),
        "keep\n"
    );
    assert!(!selected_directory.join("skip.tmp").exists());

    let archive_directory = temp.path().join("archive-directory");
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "extract",
            "/docs/bulk/nested",
            "--to",
            archive_directory.to_str().unwrap(),
        ],
    ));
    assert_eq!(
        fs::read_to_string(archive_directory.join("keep.txt")).unwrap(),
        "keep\n"
    );

    let extracted_tree = temp.path().join("extracted-tree");
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "extract",
            "--to",
            extracted_tree.to_str().unwrap(),
            "--overwrite",
            "--restore-permissions",
            "--restore-symlinks",
        ],
    ));
    assert_eq!(
        fs::read_to_string(extracted_tree.join("bulk/nested/keep.txt")).unwrap(),
        "keep\n"
    );

    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "mv",
            "notes/manual.txt",
            "notes/moved.txt",
        ],
    ));
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "remove",
            "--force",
            "--recursive",
            "bulk",
        ],
    ));
    let extracted = temp.path().join("extracted.txt");
    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "extract",
            "notes/moved.txt",
            extracted.to_str().unwrap(),
        ],
    ));
    assert_eq!(fs::read_to_string(extracted).unwrap(), "manual\n");

    success(&run(
        bin,
        temp.path(),
        &[
            lockbox.to_str().unwrap(),
            "mirror",
            "rm",
            "--force",
            "notes/moved.txt",
        ],
    ));
    success(&run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "destroy", "--force"],
    ));
    let listing = run(bin, temp.path(), &[lockbox.to_str().unwrap(), "list", "-R"]);
    success(&listing);
    assert!(!String::from_utf8_lossy(&listing.stdout).contains("/docs"));
}
