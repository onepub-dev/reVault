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
/// Diagnostic scope recorded by a cross-process write lock.
pub enum FileLockScope {
    /// Lock protects a `.lbox` archive.
    Lockbox,
    /// Lock protects a local vault directory.
    Vault,
    /// Lock exclusively owns mandatory transaction recovery.
    Recovery,
}

impl FileLockScope {
    fn as_str(self) -> &'static str {
        match self {
            Self::Lockbox => "lockbox",
            Self::Vault => "vault",
            Self::Recovery => "transaction recovery",
        }
    }
}

#[derive(Debug)]
/// RAII write lock shared by threads and cooperating processes.
///
/// Lockbox and recovery scopes lock the archive inode. Vault scope retains
/// its separate coordination sidecar. Archive locks are released on close.
pub struct ScopedFileLock {
    lock_path: PathBuf,
    archive: Option<File>,
    #[cfg(unix)]
    file: Option<std::sync::Arc<File>>,
    #[cfg(not(unix))]
    owns_lock_file: bool,
}

impl ScopedFileLock {
    /// Holds a shared lock on an existing archive without requiring write access.
    pub fn read_archive(path: &Path) -> Result<Self> {
        Ok(Self::archive_guard(super::archive_lock::open(path, false)?))
    }

    /// Reads archive bytes through the locked handle, including after a rename.
    pub fn read_archive_bytes(&self) -> Result<Vec<u8>> {
        let mut file = self
            .archive
            .as_ref()
            .ok_or_else(|| Error::InvalidOperation("not an archive lock".into()))?;
        let mut bytes = Vec::new();
        file.seek(SeekFrom::Start(0))
            .map_err(|err| Error::Io(err.to_string()))?;
        file.read_to_end(&mut bytes)
            .map_err(|err| Error::Io(err.to_string()))?;
        Ok(bytes)
    }

    /// Exclusively locks a privately staged archive before publishing its name.
    /// Keep this guard alive through publication and all associated updates.
    pub fn lock_archive(file: File) -> Result<Self> {
        super::archive_lock::acquire(&file, Path::new("<staged archive>"), true, Instant::now())?;
        Ok(Self::archive_guard(file))
    }

    fn archive_guard(file: File) -> Self {
        Self {
            lock_path: PathBuf::new(),
            archive: Some(file),
            #[cfg(unix)]
            file: None,
            #[cfg(not(unix))]
            owns_lock_file: false,
        }
    }

    /// Acquires the lock for `target`, waiting up to the configured timeout.
    pub fn acquire(target: &Path, scope: FileLockScope) -> Result<Self> {
        if scope != FileLockScope::Vault {
            return Ok(Self::archive_guard(super::archive_lock::open(
                target, true,
            )?));
        }
        let lock_path = lock_path_for(target);
        #[cfg(unix)]
        if let Some(file) = VAULT_HANDLES.with(|handles| {
            handles
                .borrow()
                .get(&lock_path)
                .and_then(std::sync::Weak::upgrade)
        }) {
            return Ok(Self {
                lock_path,
                archive: None,
                file: Some(file),
            });
        }
        #[cfg(not(unix))]
        if enter_thread_lock(&lock_path) {
            return Ok(Self {
                lock_path,
                archive: None,
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
                Ok(lock) => {
                    #[cfg(unix)]
                    if let Some(file) = &lock.file {
                        VAULT_HANDLES.with(|handles| {
                            handles
                                .borrow_mut()
                                .insert(lock_path.clone(), std::sync::Arc::downgrade(file));
                        });
                    }
                    return Ok(lock);
                }
                Err(AcquireFailure::Busy(owner)) => {
                    if started.elapsed() >= timeout {
                        #[cfg(not(unix))]
                        leave_thread_lock(&lock_path);
                        return Err(timeout_error(target, scope, timeout, owner.as_deref()));
                    }
                    thread::sleep(LOCK_POLL_INTERVAL);
                }
                Err(AcquireFailure::Io(err)) => {
                    #[cfg(not(unix))]
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
        if self
            .file
            .as_ref()
            .is_some_and(|file| std::sync::Arc::strong_count(file) == 1)
        {
            VAULT_HANDLES.with(|handles| {
                handles.borrow_mut().remove(&self.lock_path);
            });
        }
        // Closing the last Arc<File> releases the kernel lock. Never explicitly
        // unlock here: a nested guard may still own the same open description.
    }
}

#[cfg(not(unix))]
impl Drop for ScopedFileLock {
    fn drop(&mut self) {
        if self.archive.is_some() {
            return;
        }
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
        .open(lock_path)
        .map_err(|err| AcquireFailure::Io(format!("open {}: {err}", lock_path.display())))?;
    // SAFETY: flock operates on a valid file descriptor owned by `file`.
    let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc == 0 {
        write_owner_metadata(&mut file, target, scope)
            .map_err(|err| AcquireFailure::Io(format!("write {}: {err}", lock_path.display())))?;
        return Ok(ScopedFileLock {
            lock_path: lock_path.to_path_buf(),
            archive: None,
            file: Some(std::sync::Arc::new(file)),
        });
    }
    let err = std::io::Error::last_os_error();
    if err.raw_os_error() == Some(libc::EWOULDBLOCK) || err.raw_os_error() == Some(libc::EAGAIN) {
        return Err(AcquireFailure::Busy(read_owner_metadata(lock_path)));
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
        .open(lock_path)
    {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            if lock_file_is_stale(lock_path) {
                let _ = fs::remove_file(lock_path);
                return Err(AcquireFailure::Busy(None));
            }
            return Err(AcquireFailure::Busy(read_owner_metadata(lock_path)));
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
        archive: None,
        owns_lock_file: true,
    })
}

#[cfg(unix)]
thread_local! {
    static VAULT_HANDLES: RefCell<BTreeMap<PathBuf, std::sync::Weak<File>>> = const { RefCell::new(BTreeMap::new()) };
}

#[cfg(not(unix))]
thread_local! {
    static THREAD_LOCKS: RefCell<BTreeMap<PathBuf, usize>> = const {
        RefCell::new(BTreeMap::new())
    };
}

#[cfg(not(unix))]
fn enter_thread_lock(lock_path: &Path) -> bool {
    THREAD_LOCKS.with(|locks| {
        let mut locks = locks.borrow_mut();
        let count = locks.entry(lock_path.to_path_buf()).or_insert(0);
        let nested = *count > 0;
        *count = count.saturating_add(1);
        nested
    })
}

#[cfg(not(unix))]
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

/// Returns the hidden sidecar path used to coordinate writes to `target`.
pub fn lock_path_for(target: &Path) -> PathBuf {
    let Some(file_name) = target.file_name() else {
        let mut path = OsString::from(target.as_os_str());
        path.push(".lock");
        return PathBuf::from(path);
    };
    let mut lock_name = OsString::from(".");
    lock_name.push(file_name);
    lock_name.push(".lock");
    target.with_file_name(lock_name)
}

pub(crate) fn lock_timeout() -> Duration {
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
    let created = metadata_value(&text, "created_unix_ms")
        .and_then(|value| value.parse::<u128>().ok())
        .map(format_unix_millis_datetime)
        .or_else(|| metadata_value(&text, "created_unix_ms").map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string());
    Some(format!(" by pid {pid} since {created}"))
}

fn timeout_error(
    target: &Path,
    scope: FileLockScope,
    timeout: Duration,
    owner: Option<&str>,
) -> Error {
    let message = format!(
        "{} {} is locked{}; timed out after {}s",
        scope.as_str(),
        target.display(),
        owner.unwrap_or(""),
        timeout.as_secs()
    );
    if scope == FileLockScope::Recovery {
        Error::RecoveryInProgress(message)
    } else {
        Error::LockUnavailable(message)
    }
}

fn metadata_value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
}

fn format_unix_millis_datetime(unix_ms: u128) -> String {
    let rounded_seconds = ((unix_ms + 500) / 1000).min(i64::MAX as u128) as i64;
    let days = rounded_seconds.div_euclid(86_400);
    let seconds_of_day = rounded_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year, month as u32, day as u32)
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

#[cfg(windows)]
fn process_exists(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_INVALID_PARAMETER};
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    // SAFETY: `OpenProcess` receives a numeric PID and returns an owned handle
    // that is closed below when successful.
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if !handle.is_null() {
        // SAFETY: `handle` was returned successfully by `OpenProcess` and is
        // closed exactly once here.
        unsafe { CloseHandle(handle) };
        return true;
    }

    // Access can be denied for protected processes that still exist. Treat
    // every failure except the documented invalid-PID case conservatively as
    // evidence that the process may still be alive.
    // SAFETY: `GetLastError` reads the calling thread's last-error value and
    // does not dereference pointers or require additional invariants.
    (unsafe { GetLastError() }) != ERROR_INVALID_PARAMETER
}

#[cfg(not(any(unix, windows)))]
fn process_exists(_pid: u32) -> bool {
    // Avoid deleting another process's lock on platforms where no reliable
    // native process query is implemented.
    true
}

#[cfg(test)]
mod tests {
    use super::{format_unix_millis_datetime, lock_path_for};
    use std::path::Path;

    #[test]
    fn lock_file_is_hidden_beside_target() {
        assert_eq!(
            lock_path_for(Path::new("/tmp/secrets.lbox")),
            Path::new("/tmp/.secrets.lbox.lock")
        );
        assert_eq!(
            lock_path_for(Path::new("relative.lbox")),
            Path::new(".relative.lbox.lock")
        );
    }

    #[test]
    fn unix_millis_format_is_human_readable_datetime() {
        assert_eq!(format_unix_millis_datetime(0), "1970-01-01 00:00:00");
        assert_eq!(
            format_unix_millis_datetime(1_704_067_201_234),
            "2024-01-01 00:00:01"
        );
        assert_eq!(
            format_unix_millis_datetime(1_704_067_201_500),
            "2024-01-01 00:00:02"
        );
    }

    #[cfg(windows)]
    #[test]
    fn current_process_exists() {
        assert!(super::process_exists(std::process::id()));
    }
}
