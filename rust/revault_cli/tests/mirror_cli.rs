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
        ],
    );
    success(&created);
    assert!(String::from_utf8_lossy(&created.stdout).contains("No files were copied"));

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
    let rules = run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "rule", "list"],
    );
    success(&rules);
    assert!(String::from_utf8_lossy(&rules.stdout).contains("*.tmp"));

    let status = run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "status"],
    );
    success(&status);
    assert!(String::from_utf8_lossy(&status.stdout).contains("add:       2 files"));

    let before = run(bin, temp.path(), &[lockbox.to_str().unwrap(), "list", "-R"]);
    success(&before);
    assert!(!String::from_utf8_lossy(&before.stdout).contains("README.md"));

    success(&run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "mirror", "update", "--force"],
    ));
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
fn multiple_mirrors_require_name_and_destinations_cannot_overlap() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("mirror-multiple");
    let lockbox = temp.path().join("backup.lbox");
    fs::create_dir_all(temp.path().join("one")).unwrap();
    fs::create_dir_all(temp.path().join("two")).unwrap();
    success(&run(
        bin,
        temp.path(),
        &[lockbox.to_str().unwrap(), "create"],
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
        ],
    ));
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
