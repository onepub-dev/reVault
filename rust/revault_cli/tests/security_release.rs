//! Security-boundary E2E coverage uses public CLI setup and independent reads.
//! Malformed backup bytes and hostile host filesystem entries are intentional
//! exceptions: the CLI cannot produce these invalid inputs.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Fixture {
    root: tempfile::TempDir,
    vault: PathBuf,
    agent: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let fixture = Self {
            vault: root.path().join("vault"),
            agent: root.path().join("agent"),
            root,
        };
        fixture.ok(&["vault", "init"]);
        fixture
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_lockbox"))
            .args(args)
            .env("LOCKBOX_VAULT_DIR", &self.vault)
            .env("LOCKBOX_SESSION_AGENT_DIR", &self.agent)
            .env("LOCKBOX_SESSION_AGENT_LOG", self.agent.join("agent.log"))
            .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
            .env(
                "LOCKBOX_VAULT_PASSWORD",
                "synthetic security regression passphrase",
            )
            .env("LOCKBOX_KEY", "synthetic archive regression key")
            .env_remove("LOCKBOX_PASSWORD")
            .env_remove("LOCKBOX_CONTENT_KEY")
            .output()
            .unwrap()
    }
    fn ok(&self, args: &[&str]) -> Output {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }
    fn fails(&self, args: &[&str]) {
        let output = self.run(args);
        assert!(!output.status.success(), "unexpected success: {args:?}");
    }
    fn backup(&self, path: &Path) {
        self.ok(&["vault", "backup", path.to_str().unwrap(), "--overwrite"]);
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.run(&["session", "stop"]);
    }
}

#[test]
fn profile_backup_streaming_preserves_previous_output_on_failure_and_bounds_restore() {
    let fixture = Fixture::new();
    let backup = fixture.root.path().join("profile.backup");
    let path = backup.to_str().unwrap();
    fixture.ok(&["vault", "profile", "backup", path, "--name", "default"]);
    let original = fs::read(&backup).unwrap();
    fixture.fails(&[
        "vault",
        "profile",
        "backup",
        path,
        "--name",
        "missing-profile",
        "--overwrite",
    ]);
    assert_eq!(fs::read(&backup).unwrap(), original);
    fixture.ok(&["vault", "profile", "restore", path, "--name", "restored"]);
    let restored = fixture.ok(&["vault", "profile", "list", "--format", "tsv"]);
    assert!(String::from_utf8_lossy(&restored.stdout).contains("restored"));
    let invalid = fixture.root.path().join("invalid-utf8.backup");
    fs::write(&invalid, [0xff, 0xfe, 0x80]).unwrap();
    fixture.fails(&["vault", "profile", "restore", invalid.to_str().unwrap()]);
    let oversized = fixture.root.path().join("oversized.backup");
    fs::File::create(&oversized)
        .unwrap()
        .set_len(8 * 1024 * 1024 + 1)
        .unwrap();
    let refused = fixture.run(&["vault", "profile", "restore", oversized.to_str().unwrap()]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("8 MiB"));
    assert_eq!(fs::read(&backup).unwrap(), original);
}

#[test]
fn malformed_vault_backups_leave_existing_vault_unchanged() {
    let fixture = Fixture::new();
    fixture.ok(&["vault", "profile", "create", "retained"]);
    let backup = fixture.root.path().join("valid.backup");
    fixture.backup(&backup);
    let valid = fs::read(&backup).unwrap();
    let original_vault = fs::read(fixture.vault.join("local-vault.lbox")).unwrap();
    let mut trailing = valid.clone();
    trailing.push(42);
    let mut checksum = valid.clone();
    *checksum.last_mut().unwrap() ^= 1;
    let mut bad_size = valid.clone();
    let manifest_len = u64::from_be_bytes(valid[8..16].try_into().unwrap()) as usize;
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&valid[16..16 + manifest_len]).unwrap();
    manifest["vault_size"] = serde_json::json!(u64::MAX);
    let encoded = serde_json::to_vec(&manifest).unwrap();
    bad_size.truncate(8);
    bad_size.extend_from_slice(&(encoded.len() as u64).to_be_bytes());
    bad_size.extend_from_slice(&encoded);
    bad_size.extend_from_slice(&valid[16 + manifest_len..]);
    for (name, bytes) in [
        ("trailing", trailing),
        ("truncated", valid[..valid.len() - 1].to_vec()),
        ("checksum", checksum),
        ("overflow", bad_size),
    ] {
        let damaged = fixture.root.path().join(format!("{name}.backup"));
        fs::write(&damaged, bytes).unwrap();
        fixture.fails(&["vault", "restore", damaged.to_str().unwrap(), "--overwrite"]);
        assert_eq!(
            fs::read(fixture.vault.join("local-vault.lbox")).unwrap(),
            original_vault
        );
        let listed = fixture.ok(&["vault", "profile", "list", "--format", "tsv"]);
        assert!(String::from_utf8_lossy(&listed.stdout).contains("retained"));
    }
    fixture.ok(&["vault", "restore", backup.to_str().unwrap(), "--overwrite"]);
    let listed = fixture.ok(&["vault", "profile", "list", "--format", "tsv"]);
    assert!(String::from_utf8_lossy(&listed.stdout).contains("retained"));
}

#[cfg(unix)]
#[test]
fn vault_backup_is_private_and_refuses_output_links() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let fixture = Fixture::new();
    let backup = fixture.root.path().join("private.backup");
    fixture.backup(&backup);
    assert_eq!(
        fs::metadata(&backup).unwrap().permissions().mode() & 0o777,
        0o600
    );
    fs::set_permissions(&backup, fs::Permissions::from_mode(0o644)).unwrap();
    fixture.backup(&backup);
    assert_eq!(
        fs::metadata(&backup).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let sentinel = fixture.root.path().join("unrelated-file");
    let contents = b"synthetic host file must remain unchanged";
    fs::write(&sentinel, contents).unwrap();
    let linked = fixture.root.path().join("linked.backup");
    symlink(&sentinel, &linked).unwrap();
    fixture.fails(&["vault", "backup", linked.to_str().unwrap(), "--overwrite"]);
    assert_eq!(fs::read(&sentinel).unwrap(), contents);
    fs::remove_file(&linked).unwrap();
    fs::hard_link(&sentinel, &linked).unwrap();
    fixture.fails(&["vault", "backup", linked.to_str().unwrap(), "--overwrite"]);
    assert_eq!(fs::read(&sentinel).unwrap(), contents);
}

#[cfg(unix)]
#[test]
fn directory_import_skips_file_and_directory_symlinks() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let source = fixture.root.path().join("source");
    let outside = fixture.root.path().join("outside");
    fs::create_dir_all(source.join("nested")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let payload = b"allowed content\0\xff";
    fs::write(source.join("nested/allowed.bin"), payload).unwrap();
    fs::write(outside.join("private.bin"), b"synthetic out-of-tree secret").unwrap();
    symlink(outside.join("private.bin"), source.join("file-link")).unwrap();
    symlink(&outside, source.join("directory-link")).unwrap();
    let archive = fixture.root.path().join("import.lbox");
    let archive = archive.to_str().unwrap();
    fixture.ok(&[archive, "create"]);
    fixture.ok(&[archive, "add", "--recursive", source.to_str().unwrap()]);
    let listing = fixture.ok(&[archive, "ls", "--recursive"]);
    let listing = String::from_utf8_lossy(&listing.stdout);
    assert!(!listing.contains("file-link"));
    assert!(!listing.contains("directory-link"));
    assert!(!listing.contains("private.bin"));
    let stored = fixture.ok(&[archive, "cat", "/nested/allowed.bin"]);
    assert_eq!(stored.stdout, payload);
}
