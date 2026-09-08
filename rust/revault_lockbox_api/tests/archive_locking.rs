mod common;
use common::{add_file, p, password, public_api_unique_dir, signing_key};
use revault_lockbox_api::{Error, Lockbox, LockboxOpen, LockboxProtection, SecretVec};
use std::{fs, path::Path, process::Command};
const KEY: &[u8] = b"archive locking test key";
fn key() -> SecretVec {
    SecretVec::try_from_slice(KEY).unwrap()
}
fn create(path: &Path) -> Lockbox {
    let mut archive =
        Lockbox::create_file(path, LockboxProtection::ContentKey(key()), &signing_key()).unwrap();
    add_file(
        &mut archive,
        &p("/secret"),
        b"exact secret bytes\0\xff",
        false,
    )
    .unwrap();
    archive.commit().unwrap();
    archive
}
fn child(path: &Path, operation: &str, blocked: bool) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "archive_lock_child", "--nocapture"])
        .env("ARCHIVE_LOCK_TEST_PATH", path)
        .env("ARCHIVE_LOCK_TEST_OP", operation)
        .env(
            "ARCHIVE_LOCK_TEST_BLOCKED",
            if blocked { "yes" } else { "no" },
        )
        .env("LOCKBOX_LOCK_TIMEOUT_MS", "50")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn archive_lock_child() {
    let Some(path) = std::env::var_os("ARCHIVE_LOCK_TEST_PATH") else {
        return;
    };
    let path = Path::new(&path);
    let operation = std::env::var("ARCHIVE_LOCK_TEST_OP").unwrap();
    let result = if operation == "inspect" {
        Lockbox::inspect_file(path).map(|_| ())
    } else if operation == "write" {
        Lockbox::open_for_write(path, LockboxOpen::ContentKey(key()), &signing_key()).map(|_| ())
    } else if operation == "password" {
        let password = password("replacement-password");
        Lockbox::open(path, LockboxOpen::Password(&password)).map(|archive| {
            assert_eq!(
                archive.get_file(&p("/secret")).unwrap(),
                b"exact secret bytes\0\xff"
            );
        })
    } else {
        Lockbox::open(path, LockboxOpen::ContentKey(key())).map(|archive| {
            assert_eq!(
                archive.get_file(&p("/secret")).unwrap(),
                b"exact secret bytes\0\xff"
            );
        })
    };
    if std::env::var("ARCHIVE_LOCK_TEST_BLOCKED").unwrap() == "yes" {
        assert!(
            matches!(result, Err(Error::LockUnavailable(_))),
            "{result:?}"
        );
    } else {
        result.unwrap();
    }
}
#[test]
fn readers_share_locks_and_exclude_writers_across_processes() {
    let root = public_api_unique_dir("archive-shared-locks");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("archive.lbox");
    let writer = create(&path);
    child(&path, "read", true);
    child(&path, "inspect", true);
    child(&path, "write", true);
    drop(writer);
    let reader = Lockbox::open(&path, LockboxOpen::ContentKey(key())).unwrap();
    let clone = Lockbox::open(&path, LockboxOpen::ContentKey(key())).unwrap();
    child(&path, "read", false);
    child(&path, "inspect", false);
    child(&path, "write", true);
    drop(reader);
    child(&path, "write", true);
    drop(clone);
    child(&path, "write", false);
    assert_eq!(
        fs::read_dir(&root).unwrap().count(),
        1,
        "no sidecars or temporary files"
    );
    fs::remove_dir_all(root).unwrap();
}
#[cfg(unix)]
#[test]
fn read_only_file_and_directory_support_inspection_and_decryption() {
    use std::os::unix::fs::PermissionsExt;
    let root = public_api_unique_dir("archive-readonly");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("archive.lbox");
    let password = password("readonly-password");
    let mut writer = Lockbox::create_file(
        &path,
        LockboxProtection::Password(&password),
        &signing_key(),
    )
    .unwrap();
    add_file(
        &mut writer,
        &p("/secret"),
        b"exact secret bytes\0\xff",
        false,
    )
    .unwrap();
    writer.commit().unwrap();
    drop(writer);
    let before = fs::read(&path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o555)).unwrap();
    // Permissions are a host condition, not a lockbox API operation. Restore
    // them before assertions so a regression does not leave an undeletable fixture.
    let inspection = Lockbox::inspect_file(&path);
    let opened = Lockbox::open(&path, LockboxOpen::Password(&password));
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
    inspection.unwrap();
    assert_eq!(
        opened.unwrap().get_file(&p("/secret")).unwrap(),
        b"exact secret bytes\0\xff"
    );
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn replacement_stays_locked_and_remains_writable() {
    let root = public_api_unique_dir("archive-replacement-lock");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("archive.lbox");
    let password = password("replacement-password");
    let mut writer = create(&path);
    writer
        .replace_content_key_with_access(&[], &[("retained".into(), password)])
        .unwrap();
    child(&path, "read", true);
    child(&path, "write", true);
    add_file(
        &mut writer,
        &p("/after"),
        b"written after replacement",
        false,
    )
    .unwrap();
    writer.commit().unwrap();
    drop(writer);
    let password = common::password("replacement-password");
    let reader = Lockbox::open(&path, LockboxOpen::Password(&password)).unwrap();
    assert_eq!(
        reader.get_file(&p("/secret")).unwrap(),
        b"exact secret bytes\0\xff"
    );
    assert_eq!(
        reader.get_file(&p("/after")).unwrap(),
        b"written after replacement"
    );
    drop(reader);
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn waiting_reader_retries_after_inode_replacement() {
    use std::{
        os::unix::fs::MetadataExt,
        time::{Duration, Instant},
    };
    let root = public_api_unique_dir("archive-waiter-replacement");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("archive.lbox");
    let mut writer = create(&path);
    let original = fs::metadata(&path).unwrap();
    let mut waiting = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "archive_lock_child", "--nocapture"])
        .env("ARCHIVE_LOCK_TEST_PATH", &path)
        .env("ARCHIVE_LOCK_TEST_OP", "password")
        .env("ARCHIVE_LOCK_TEST_BLOCKED", "no")
        .env("LOCKBOX_LOCK_TIMEOUT_MS", "30000")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    // Observe the child's actual descriptor, not a sleep-based guess, to prove
    // it opened the old inode before publication. /proc is Linux-specific.
    let started = Instant::now();
    loop {
        let opened_old_inode = fs::read_dir(format!("/proc/{}/fd", waiting.id()))
            .unwrap()
            .any(|entry| {
                entry
                    .ok()
                    .and_then(|entry| fs::metadata(entry.path()).ok())
                    .is_some_and(|m| m.dev() == original.dev() && m.ino() == original.ino())
            });
        if opened_old_inode {
            break;
        }
        if started.elapsed() > Duration::from_secs(10) {
            let _ = waiting.kill();
            let _ = waiting.wait();
            panic!("child did not open the original archive");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let password = password("replacement-password");
    writer
        .replace_content_key_with_access(&[], &[("retained".into(), password)])
        .unwrap();
    drop(writer);
    let output = waiting.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).unwrap();
}
