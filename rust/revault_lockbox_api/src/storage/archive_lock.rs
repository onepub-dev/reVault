//! Archive locks follow the open handle. Openers must validate identity after locking.
use crate::{Error, Result};
use std::{fs::File, path::Path, time::Instant};

pub(super) fn acquire(file: &File, path: &Path, exclusive: bool, started: Instant) -> Result<()> {
    loop {
        match try_lock(file, exclusive) {
            Ok(true) => return Ok(()),
            Ok(false) => {
                if started.elapsed() >= super::file_lock::lock_timeout() {
                    return Err(Error::LockUnavailable(format!(
                        "timed out locking archive {}",
                        path.display()
                    )));
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(err) => return Err(Error::Io(format!("lock {}: {err}", path.display()))),
        }
    }
}

#[cfg(unix)]
fn try_lock(file: &File, exclusive: bool) -> std::io::Result<bool> {
    use std::os::fd::AsRawFd;
    let operation = if exclusive {
        libc::LOCK_EX
    } else {
        libc::LOCK_SH
    };
    // SAFETY: file owns a live descriptor throughout the call.
    if unsafe { libc::flock(file.as_raw_fd(), operation | libc::LOCK_NB) } == 0 {
        return Ok(true);
    }
    let err = std::io::Error::last_os_error();
    if err.kind() == std::io::ErrorKind::WouldBlock {
        Ok(false)
    } else {
        Err(err)
    }
}

#[cfg(windows)]
fn try_lock(file: &File, exclusive: bool) -> std::io::Result<bool> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        LockFileEx, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY,
    };
    let flags = LOCKFILE_FAIL_IMMEDIATELY
        | if exclusive {
            LOCKFILE_EXCLUSIVE_LOCK
        } else {
            0
        };
    // SAFETY: the synchronous handle and initialized offset structure are valid.
    let mut overlapped = unsafe { std::mem::zeroed() };
    if unsafe {
        LockFileEx(
            file.as_raw_handle(),
            flags,
            0,
            u32::MAX,
            u32::MAX,
            &mut overlapped,
        )
    } != 0
    {
        return Ok(true);
    }
    let err = std::io::Error::last_os_error();
    if err.raw_os_error() == Some(33) {
        Ok(false)
    } else {
        Err(err)
    }
}

#[cfg(not(any(unix, windows)))]
fn try_lock(_: &File, _: bool) -> std::io::Result<bool> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "archive file locking unsupported on this platform",
    ))
}

pub(super) fn is_current(file: &File, path: &Path) -> Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let held = file.metadata().map_err(|err| Error::Io(err.to_string()))?;
        let current = match std::fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(err) => return Err(Error::Io(err.to_string())),
        };
        Ok(held.dev() == current.dev() && held.ino() == current.ino())
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::GetFileInformationByHandle;
        fn identity(file: &File) -> Result<(u32, u32, u32)> {
            // SAFETY: the API initializes the output on success.
            let mut info = unsafe { std::mem::zeroed() };
            if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
                return Err(Error::Io(std::io::Error::last_os_error().to_string()));
            }
            Ok((
                info.dwVolumeSerialNumber,
                info.nFileIndexHigh,
                info.nFileIndexLow,
            ))
        }
        let current = match File::open(path) {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(err) => return Err(Error::Io(err.to_string())),
        };
        Ok(identity(file)? == identity(&current)?)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (file, path);
        Err(Error::Io(
            "archive identity checks unsupported on this platform".into(),
        ))
    }
}

pub(super) fn open(path: &Path, writable: bool) -> Result<File> {
    let started = Instant::now();
    loop {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(writable)
            .open(path)
            .map_err(|err| Error::Io(format!("open {}: {err}", path.display())))?;
        acquire(&file, path, writable, started)?;
        if is_current(&file, path)? {
            return Ok(file);
        }
        if started.elapsed() >= super::file_lock::lock_timeout() {
            return Err(Error::LockUnavailable(format!(
                "archive repeatedly replaced: {}",
                path.display()
            )));
        }
    }
}
