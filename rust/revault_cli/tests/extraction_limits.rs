mod common;
use common::{CommandTestExt, TestTempDir};
use std::fs;
use std::process::{Command, Output};

struct Fixture {
    dir: TestTempDir,
}
impl Fixture {
    fn new() -> Self {
        Self::with_create_options(&[])
    }
    fn with_create_options(options: &[&str]) -> Self {
        let f = Self {
            dir: TestTempDir::new("extraction-limits"),
        };
        fs::create_dir_all(f.dir.path().join("source/sub")).unwrap();
        fs::write(f.dir.path().join("source/a"), b"12345").unwrap();
        fs::write(f.dir.path().join("source/sub/b"), b"1234567").unwrap();
        let mut args = vec!["box.lbox", "create"];
        args.extend_from_slice(options);
        f.ok(&args);
        f
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_lockbox"))
            .current_dir(self.dir.path())
            .args(args)
            .env("LOCKBOX_KEY", "synthetic-extraction-key")
            .env("LOCKBOX_VAULT_DIR", self.dir.path().join("vault"))
            .env(
                "LOCKBOX_VAULT_PASSWORD",
                "synthetic extraction vault password",
            )
            .env(
                "LOCKBOX_SESSION_AGENT_DIR",
                common::agent_socket_dir(&self.dir.path().join("agent")),
            )
            .env("LOCKBOX_ADD_PROGRESS", "off")
            .test_output()
            .unwrap()
    }
    fn ok(&self, args: &[&str]) -> Output {
        let out = self.run(args);
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        out
    }
    fn refused(&self, args: &[&str]) {
        let out = self.run(args);
        assert!(!out.status.success(), "unexpected success: {args:?}");
    }
    fn verify_tree(&self, destination: &str) {
        assert_eq!(
            fs::read(self.dir.path().join(destination).join("a")).unwrap(),
            b"12345"
        );
        assert_eq!(
            fs::read(self.dir.path().join(destination).join("sub/b")).unwrap(),
            b"1234567"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(self.dir.path().join(destination).join("a"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
    }
}

#[test]
fn file_directory_and_archive_extraction_obey_numeric_limits() {
    let f = Fixture::new();
    f.ok(&["box.lbox", "add", "source", "--recursive", "--to", "/docs"]);
    for (flag, limit) in [
        ("--max-file-bytes", "6"),
        ("--max-total-bytes", "11"),
        ("--max-entries", "3"),
    ] {
        f.refused(&[
            "box.lbox", "extract", "/docs", "--to", "refused", flag, limit,
        ]);
        assert!(!f.dir.path().join("refused").exists());
    }
    f.ok(&[
        "box.lbox",
        "extract",
        "/docs",
        "--to",
        "out",
        "--max-file-bytes",
        "7",
        "--max-total-bytes",
        "12",
        "--max-entries",
        "4",
    ]);
    f.verify_tree("out");
    f.refused(&[
        "box.lbox",
        "extract",
        "/docs",
        "--to",
        "out",
        "--max-total-bytes",
        "10M",
    ]);
    f.verify_tree("out");
    f.ok(&[
        "box.lbox",
        "extract",
        "/docs",
        "--to",
        "out",
        "--overwrite",
        "--max-total-bytes",
        "10M",
    ]);
    f.verify_tree("out");
    for (flag, limit) in [
        ("--max-file-bytes", "4"),
        ("--max-total-bytes", "4"),
        ("--max-entries", "0"),
    ] {
        f.refused(&["box.lbox", "extract", "/docs/a", "single", flag, limit]);
        assert!(!f.dir.path().join("single").exists());
    }
    f.ok(&[
        "box.lbox",
        "extract",
        "/docs/a",
        "single",
        "--max-file-bytes",
        "5",
        "--max-total-bytes",
        "5",
        "--max-entries",
        "1",
    ]);
    assert_eq!(fs::read(f.dir.path().join("single")).unwrap(), b"12345");
    f.refused(&["box.lbox", "extract", "--to", "all", "--max-entries", "1"]);
    f.ok(&[
        "box.lbox",
        "extract",
        "--to",
        "all",
        "--max-file-bytes",
        "1K",
        "--max-total-bytes",
        "10M",
        "--max-entries",
        "100",
    ]);
    f.verify_tree("all/docs");
    fs::write(f.dir.path().join("replacement"), b"abcde").unwrap();
    f.ok(&[
        "box.lbox",
        "add",
        "replacement",
        "--to",
        "/docs/a",
        "--overwrite",
    ]);
    f.ok(&[
        "box.lbox",
        "extract",
        "/docs/a",
        "single",
        "--overwrite",
        "--max-file-bytes",
        "5",
    ]);
    assert_eq!(fs::read(f.dir.path().join("single")).unwrap(), b"abcde");
    f.ok(&["box.lbox", "remove", "/docs/a", "--force"]);
    f.refused(&["box.lbox", "cat", "/docs/a"]);
    f.ok(&[
        "box.lbox",
        "extract",
        "/docs",
        "--to",
        "after-remove",
        "--max-total-bytes",
        "7",
    ]);
    assert!(!f.dir.path().join("after-remove/a").exists());
    assert_eq!(
        fs::read(f.dir.path().join("after-remove/sub/b")).unwrap(),
        b"1234567"
    );
}

#[test]
fn mirror_extraction_uses_the_same_limits() {
    let f = Fixture::new();
    f.ok(&[
        "box.lbox", "mirrors", "project", "create", "--from", "source", "--to", "/mirror",
    ]);
    f.ok(&["box.lbox", "mirrors", "project", "update", "--force"]);
    for (flag, limit) in [
        ("--max-file-bytes", "6"),
        ("--max-total-bytes", "11"),
        ("--max-entries", "3"),
    ] {
        f.refused(&[
            "box.lbox", "mirrors", "project", "extract", "--to", "out", flag, limit,
        ]);
        assert!(!f.dir.path().join("out").exists());
    }
    f.ok(&[
        "box.lbox",
        "mirrors",
        "project",
        "extract",
        "--to",
        "out",
        "--max-file-bytes",
        "7",
        "--max-total-bytes",
        "12",
        "--max-entries",
        "4",
    ]);
    f.verify_tree("out");
    f.refused(&[
        "box.lbox",
        "mirrors",
        "project",
        "extract",
        "a",
        "single",
        "--max-total-bytes",
        "4",
    ]);
    f.ok(&[
        "box.lbox",
        "mirrors",
        "project",
        "extract",
        "a",
        "single",
        "--max-file-bytes",
        "1KiB",
        "--max-total-bytes",
        "5",
        "--max-entries",
        "1",
    ]);
    assert_eq!(fs::read(f.dir.path().join("single")).unwrap(), b"12345");
}

#[test]
fn extraction_sizes_accept_short_units_and_reject_invalid_or_overflowing_values() {
    let f = Fixture::new();
    fs::write(f.dir.path().join("kilobyte"), vec![42; 1024]).unwrap();
    f.ok(&["box.lbox", "add", "kilobyte", "--to", "/k"]);
    for unit in [
        "1K", "1k", "1KiB", "1M", "10M", "1MiB", "1G", "1GiB", "1024", "1024B",
    ] {
        f.ok(&[
            "box.lbox",
            "extract",
            "/k",
            "unit",
            "--overwrite",
            "--max-file-bytes",
            unit,
        ]);
        assert_eq!(fs::read(f.dir.path().join("unit")).unwrap(), vec![42; 1024]);
    }
    f.refused(&[
        "box.lbox",
        "extract",
        "/k",
        "too-small",
        "--max-file-bytes",
        "1023",
    ]);
    for value in [
        "-1",
        "+1",
        "1.5M",
        "1MB",
        "unlimited",
        "18446744073709551616",
        "18446744073709551615G",
        "10 M",
        "M",
    ] {
        f.refused(&[
            "box.lbox",
            "extract",
            "/k",
            "invalid",
            "--max-file-bytes",
            value,
        ]);
    }
    for value in ["-1", "1K", "1.5", "18446744073709551616"] {
        f.refused(&[
            "box.lbox",
            "extract",
            "/k",
            "invalid",
            "--max-entries",
            value,
        ]);
    }
    f.refused(&[
        "box.lbox",
        "extract",
        "/k",
        "invalid",
        "--max-file-bytes",
        "1K",
        "--max-file-bytes",
        "2K",
    ]);
    assert!(!f.dir.path().join("invalid").exists());
    fs::write(f.dir.path().join("empty"), []).unwrap();
    f.ok(&["box.lbox", "add", "empty", "--to", "/empty"]);
    f.ok(&[
        "box.lbox",
        "extract",
        "/empty",
        "empty-out",
        "--max-file-bytes",
        "0",
        "--max-total-bytes",
        "0",
        "--max-entries",
        "1",
    ]);
    assert_eq!(fs::read(f.dir.path().join("empty-out")).unwrap(), b"");
    f.refused(&[
        "box.lbox",
        "extract",
        "/k",
        "no-bytes",
        "--max-total-bytes",
        "0",
    ]);
}

#[cfg(unix)]
#[test]
fn extraction_entry_limit_counts_symlinks_and_keeps_restore_opt_in() {
    let f = Fixture::with_create_options(&["--signing", "none"]);
    f.ok(&["box.lbox", "add", "source", "--recursive", "--to", "/docs"]);
    // The CLI cannot create archived symlinks (recursive add skips host links).
    // Use the API only for this otherwise-unrepresentable fixture entry.
    {
        use revault_lockbox_api::SecretVec;
        use revault_lockbox_api::{Lockbox, LockboxOpen, LockboxPath, Signing};
        let key = SecretVec::try_from_vec(b"synthetic-extraction-key".to_vec()).unwrap();
        let mut archive = Lockbox::open_for_write(
            &f.dir.path().join("box.lbox"),
            LockboxOpen::ContentKey(key),
            Signing::None,
        )
        .unwrap();
        archive
            .add_symlink(
                &LockboxPath::new("/link").unwrap(),
                &LockboxPath::new("/docs/a").unwrap(),
                false,
            )
            .unwrap();
        archive.commit().unwrap();
    }
    f.refused(&[
        "box.lbox",
        "extract",
        "--to",
        "refused",
        "--max-entries",
        "4",
        "--restore-symlinks",
    ]);
    f.ok(&[
        "box.lbox",
        "extract",
        "--to",
        "links",
        "--max-entries",
        "5",
        "--restore-symlinks",
    ]);
    assert_eq!(
        fs::read_link(f.dir.path().join("links/link")).unwrap(),
        std::path::Path::new("docs/a")
    );
    f.verify_tree("links/docs");
    assert_eq!(fs::read(f.dir.path().join("links/link")).unwrap(), b"12345");
    f.ok(&[
        "box.lbox",
        "extract",
        "--to",
        "no-links",
        "--max-entries",
        "5",
    ]);
    assert!(!f.dir.path().join("no-links/link").exists());
}
