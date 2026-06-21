use crate::{Error, Result};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DEFAULT_LOCK_TIMEOUT: Duration = Duration::from_secs(30);
const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileLockScope {
    Lockbox,
    Vault,
}

impl FileLockScope {
    fn as_str(self) -> &'static str {
        match self {
            Self::Lockbox => "lockbox",
            Self::Vault => "vault",
        }
    }
}

#[derive(Debug)]
pub struct ScopedFileLock {
    lock_path: PathBuf,
    #[cfg(unix)]
    file: Option<File>,
    #[cfg(not(unix))]
    owns_lock_file: bool,
}

impl ScopedFileLock {
    pub fn acquire(target: &Path, scope: FileLockScope) -> Result<Self> {
        let lock_path = lock_path_for(target);
        if enter_thread_lock(&lock_path) {
            return Ok(Self {
                lock_path,
                #[cfg(unix)]
                file: None,
                #[cfg(not(unix))]
                owns_lock_file: false,
            });
        }
        let timeout = lock_timeout();
        let started = Instant::now();
        loop {
            match try_acquire(target, &lock_path, scope) {
                Ok(lock) => return Ok(lock),
                Err(AcquireFailure::Busy(owner)) => {
                    if started.elapsed() >= timeout {
                        leave_thread_lock(&lock_path);
                        return Err(timeout_error(target, scope, timeout, owner.as_deref()));
                    }
                    thread::sleep(LOCK_POLL_INTERVAL);
                }
                Err(AcquireFailure::Io(err)) => {
                    leave_thread_lock(&lock_path);
                    return Err(Error::Io(err));
                }
            }
        }
    }
}

#[cfg(unix)]
impl Drop for ScopedFileLock {
    fn drop(&mut self) {
        if !leave_thread_lock(&self.lock_path) {
            return;
        }
        if let Some(file) = &self.file {
            use std::os::fd::AsRawFd;

            // SAFETY: this releases the same valid descriptor locked in
            // `try_acquire`.
            let _ = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_UN) };
        }
    }
}

#[cfg(not(unix))]
impl Drop for ScopedFileLock {
    fn drop(&mut self) {
        if leave_thread_lock(&self.lock_path) && self.owns_lock_file {
            let _ = fs::remove_file(&self.lock_path);
        }
    }
}

enum AcquireFailure {
    Busy(Option<String>),
    Io(String),
}

#[cfg(unix)]
fn try_acquire(
    target: &Path,
    lock_path: &Path,
    scope: FileLockScope,
) -> std::result::Result<ScopedFileLock, AcquireFailure> {
    use std::os::fd::AsRawFd;

    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent).map_err(|err| AcquireFailure::Io(err.to_string()))?;
    }
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|err| AcquireFailure::Io(format!("open {}: {err}", lock_path.display())))?;
    // SAFETY: flock operates on a valid file descriptor owned by `file`.
    let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc == 0 {
        write_owner_metadata(&mut file, target, scope)
            .map_err(|err| AcquireFailure::Io(format!("write {}: {err}", lock_path.display())))?;
        return Ok(ScopedFileLock {
            lock_path: lock_path.to_path_buf(),
            file: Some(file),
        });
    }
    let err = std::io::Error::last_os_error();
    if err.raw_os_error() == Some(libc::EWOULDBLOCK) || err.raw_os_error() == Some(libc::EAGAIN) {
        return Err(AcquireFailure::Busy(read_owner_metadata(&lock_path)));
    }
    Err(AcquireFailure::Io(err.to_string()))
}

#[cfg(not(unix))]
fn try_acquire(
    target: &Path,
    lock_path: &Path,
    scope: FileLockScope,
) -> std::result::Result<ScopedFileLock, AcquireFailure> {
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent).map_err(|err| AcquireFailure::Io(err.to_string()))?;
    }
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
    {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            if lock_file_is_stale(&lock_path) {
                let _ = fs::remove_file(&lock_path);
                return Err(AcquireFailure::Busy(None));
            }
            return Err(AcquireFailure::Busy(read_owner_metadata(&lock_path)));
        }
        Err(err) => {
            return Err(AcquireFailure::Io(format!(
                "create {}: {err}",
                lock_path.display()
            )));
        }
    };
    write_owner_metadata(&mut file, target, scope)
        .map_err(|err| AcquireFailure::Io(format!("write {}: {err}", lock_path.display())))?;
    Ok(ScopedFileLock {
        lock_path: lock_path.to_path_buf(),
        owns_lock_file: true,
    })
}

thread_local! {
    static THREAD_LOCKS: RefCell<BTreeMap<PathBuf, usize>> = const {
        RefCell::new(BTreeMap::new())
    };
}

fn enter_thread_lock(lock_path: &Path) -> bool {
    THREAD_LOCKS.with(|locks| {
        let mut locks = locks.borrow_mut();
        let count = locks.entry(lock_path.to_path_buf()).or_insert(0);
        let nested = *count > 0;
        *count = count.saturating_add(1);
        nested
    })
}

fn leave_thread_lock(lock_path: &Path) -> bool {
    THREAD_LOCKS.with(|locks| {
        let mut locks = locks.borrow_mut();
        let Some(count) = locks.get_mut(lock_path) else {
            return true;
        };
        *count = count.saturating_sub(1);
        if *count == 0 {
            locks.remove(lock_path);
            true
        } else {
            false
        }
    })
}

fn lock_path_for(target: &Path) -> PathBuf {
    let mut path = OsString::from(target.as_os_str());
    path.push(".lock");
    PathBuf::from(path)
}

fn lock_timeout() -> Duration {
    std::env::var("LOCKBOX_LOCK_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(DEFAULT_LOCK_TIMEOUT)
}

fn write_owner_metadata(
    file: &mut File,
    target: &Path,
    scope: FileLockScope,
) -> std::io::Result<()> {
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let exe = std::env::current_exe()
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    writeln!(file, "scope={}", scope.as_str())?;
    writeln!(file, "target={}", target.display())?;
    writeln!(file, "pid={}", std::process::id())?;
    writeln!(file, "user={user}")?;
    writeln!(file, "exe={exe}")?;
    writeln!(file, "created_unix_ms={now_ms}")?;
    file.sync_data()
}

fn read_owner_metadata(path: &Path) -> Option<String> {
    let mut text = String::new();
    File::open(path).ok()?.read_to_string(&mut text).ok()?;
    let pid = metadata_value(&text, "pid").unwrap_or("unknown");
    let exe = metadata_value(&text, "exe").unwrap_or("unknown");
    let created = metadata_value(&text, "created_unix_ms").unwrap_or("unknown");
    Some(format!(" by pid {pid} ({exe}) since {created}"))
}

fn timeout_error(
    target: &Path,
    scope: FileLockScope,
    timeout: Duration,
    owner: Option<&str>,
) -> Error {
    Error::LockUnavailable(format!(
        "{} {} is locked{}; timed out after {}s",
        scope.as_str(),
        target.display(),
        owner.unwrap_or(""),
        timeout.as_secs()
    ))
}

fn metadata_value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
}

#[cfg(not(unix))]
fn lock_file_is_stale(path: &Path) -> bool {
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    let Some(pid) = metadata_value(&text, "pid").and_then(|pid| pid.parse::<u32>().ok()) else {
        return false;
    };
    !process_exists(pid)
}

#[cfg(not(unix))]
fn process_exists(pid: u32) -> bool {
    let pid = sysinfo::Pid::from_u32(pid);
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
    system.process(pid).is_some()
}
