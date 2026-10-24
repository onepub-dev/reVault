//! Private temporary archive creation and held-handle permission preservation.
use std::fs::File;
use std::io;
use std::path::Path;

#[cfg(unix)]
pub(super) fn create(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

#[cfg(windows)]
pub(super) fn create(path: &Path) -> io::Result<File> {
    use std::os::windows::{ffi::OsStrExt, io::FromRawHandle};
    use windows_sys::Win32::Foundation::{
        LocalFree, GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::Security::Authorization::{
        ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
    };
    use windows_sys::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_DELETE, FILE_SHARE_READ,
        FILE_SHARE_WRITE, WRITE_DAC,
    };
    let wide = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    if wide[..wide.len() - 1].contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "archive path contains NUL",
        ));
    }
    // Same owner-only protected DACL used by extraction staging. Installed at
    // creation, before the first archive byte; there is no broad-access interval.
    let sddl = "D:P(A;;FA;;;OW)\0".encode_utf16().collect::<Vec<_>>();
    let mut descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    struct Descriptor(PSECURITY_DESCRIPTOR);
    impl Drop for Descriptor {
        fn drop(&mut self) {
            unsafe {
                LocalFree(self.0.cast());
            }
        }
    }
    let descriptor = Descriptor(descriptor);
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    };
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ | GENERIC_WRITE | WRITE_DAC,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            &attributes,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { File::from_raw_handle(handle) })
}

#[cfg(not(any(unix, windows)))]
pub(super) fn create(_path: &Path) -> io::Result<File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "private archive file creation is unavailable on this platform",
    ))
}

pub(super) fn copy_permissions(source: &File, destination: &File) -> io::Result<()> {
    destination.set_permissions(source.metadata()?.permissions())?;
    #[cfg(windows)]
    copy_windows_dacl(source, destination)?;
    destination.sync_all()
}

#[cfg(windows)]
fn copy_windows_dacl(source: &File, destination: &File) -> io::Result<()> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Authorization::{
        GetSecurityInfo, SetSecurityInfo, SE_FILE_OBJECT,
    };
    use windows_sys::Win32::Security::{
        GetSecurityDescriptorControl, ACL, DACL_SECURITY_INFORMATION,
        PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, SE_DACL_PROTECTED,
        UNPROTECTED_DACL_SECURITY_INFORMATION,
    };
    let mut dacl: *mut ACL = std::ptr::null_mut();
    let mut descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
    let status = unsafe {
        GetSecurityInfo(
            source.as_raw_handle(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut dacl,
            std::ptr::null_mut(),
            &mut descriptor,
        )
    };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    struct Descriptor(PSECURITY_DESCRIPTOR);
    impl Drop for Descriptor {
        fn drop(&mut self) {
            unsafe {
                LocalFree(self.0.cast());
            }
        }
    }
    let descriptor = Descriptor(descriptor);
    let mut control = 0u16;
    let mut revision = 0u32;
    if unsafe { GetSecurityDescriptorControl(descriptor.0, &mut control, &mut revision) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let inheritance = if control & SE_DACL_PROTECTED != 0 {
        PROTECTED_DACL_SECURITY_INFORMATION
    } else {
        UNPROTECTED_DACL_SECURITY_INFORMATION
    };
    // A null DACL is copied only when returned for the existing held source;
    // it is never substituted for a failed descriptor query.
    let status = unsafe {
        SetSecurityInfo(
            destination.as_raw_handle(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | inheritance,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            dacl,
            std::ptr::null_mut(),
        )
    };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    Ok(())
}

#[cfg(all(test, windows))]
pub(crate) fn test_assert_owner_only(file: &File) -> Vec<u8> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Authorization::{GetSecurityInfo, SE_FILE_OBJECT};
    use windows_sys::Win32::Security::{
        GetAce, GetSecurityDescriptorControl, IsWellKnownSid, WinCreatorOwnerRightsSid,
        ACCESS_ALLOWED_ACE, ACL, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
        SE_DACL_PROTECTED,
    };
    let mut dacl: *mut ACL = std::ptr::null_mut();
    let mut descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
    let status = unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut dacl,
            std::ptr::null_mut(),
            &mut descriptor,
        )
    };
    assert_eq!(status, 0, "query actual held file security descriptor");
    struct Descriptor(PSECURITY_DESCRIPTOR);
    impl Drop for Descriptor {
        fn drop(&mut self) {
            unsafe {
                LocalFree(self.0.cast());
            }
        }
    }
    let descriptor = Descriptor(descriptor);
    let mut control = 0u16;
    let mut revision = 0u32;
    assert_ne!(
        unsafe { GetSecurityDescriptorControl(descriptor.0, &mut control, &mut revision) },
        0
    );
    assert_ne!(
        control & SE_DACL_PROTECTED,
        0,
        "private DACL must not inherit broad access"
    );
    assert!(!dacl.is_null(), "null DACL would grant everyone access");
    assert_eq!(
        unsafe { (*dacl).AceCount },
        1,
        "exactly one owner access entry"
    );
    let mut ace: *mut std::ffi::c_void = std::ptr::null_mut();
    assert_ne!(unsafe { GetAce(dacl, 0, &mut ace) }, 0);
    let allowed = unsafe { &*(ace as *const ACCESS_ALLOWED_ACE) };
    assert_eq!(allowed.Header.AceType, 0, "ordinary ACCESS_ALLOWED_ACE");
    assert_eq!(
        allowed.Header.AceFlags, 0,
        "no inherited or propagation entry"
    );
    assert_ne!(
        unsafe {
            IsWellKnownSid(
                std::ptr::addr_of!(allowed.SidStart).cast_mut().cast(),
                WinCreatorOwnerRightsSid,
            )
        },
        0,
        "only OWNER RIGHTS receives access"
    );
    unsafe { std::slice::from_raw_parts(dacl.cast::<u8>(), (*dacl).AclSize as usize).to_vec() }
}

#[cfg(all(test, windows))]
mod windows_security_tests {
    use super::*;
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::os::windows::io::AsRawHandle;

    #[test]
    fn private_archive_creation_and_replacement_preserve_held_dacl_and_bytes() {
        let mut nonce = [0; 16];
        getrandom::fill(&mut nonce).unwrap();
        let root = std::env::temp_dir().join(format!(
            "revault-private-archive-{:032x}",
            u128::from_le_bytes(nonce)
        ));
        std::fs::create_dir(&root).unwrap();
        let source_path = root.join("source");
        let replacement_path = root.join("replacement");
        let mut source = create(&source_path).unwrap();
        let original_acl = test_assert_owner_only(&source);
        source.write_all(b"original archive bytes").unwrap();
        // Give the held source a distinct, still owner-only DACL, so preservation
        // is not accidentally proved by comparing two identical defaults.
        use windows_sys::Win32::Foundation::LocalFree;
        use windows_sys::Win32::Security::Authorization::{
            ConvertStringSecurityDescriptorToSecurityDescriptorW, SetSecurityInfo, SDDL_REVISION_1,
            SE_FILE_OBJECT,
        };
        use windows_sys::Win32::Security::{
            GetSecurityDescriptorDacl, DACL_SECURITY_INFORMATION,
            PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
        };
        let sddl = "D:P(A;;FRFW;;;OW)\0".encode_utf16().collect::<Vec<_>>();
        let mut descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        assert_ne!(
            unsafe {
                ConvertStringSecurityDescriptorToSecurityDescriptorW(
                    sddl.as_ptr(),
                    SDDL_REVISION_1,
                    &mut descriptor,
                    std::ptr::null_mut(),
                )
            },
            0
        );
        struct Descriptor(PSECURITY_DESCRIPTOR);
        impl Drop for Descriptor {
            fn drop(&mut self) {
                unsafe {
                    LocalFree(self.0.cast());
                }
            }
        }
        let descriptor = Descriptor(descriptor);
        let mut present = 0;
        let mut defaulted = 0;
        let mut dacl = std::ptr::null_mut();
        assert_ne!(
            unsafe {
                GetSecurityDescriptorDacl(descriptor.0, &mut present, &mut dacl, &mut defaulted)
            },
            0
        );
        assert_ne!(present, 0);
        assert_eq!(
            unsafe {
                SetSecurityInfo(
                    source.as_raw_handle(),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    dacl,
                    std::ptr::null_mut(),
                )
            },
            0
        );
        let source_acl = test_assert_owner_only(&source);
        assert_ne!(source_acl, original_acl);
        let mut replacement = create(&replacement_path).unwrap();
        test_assert_owner_only(&replacement);
        replacement.write_all(b"replacement archive bytes").unwrap();
        copy_permissions(&source, &replacement).unwrap();
        assert_eq!(test_assert_owner_only(&replacement), source_acl);
        replacement.seek(SeekFrom::Start(0)).unwrap();
        let mut bytes = Vec::new();
        replacement.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"replacement archive bytes");
        drop(source);
        drop(replacement);
        std::fs::rename(&replacement_path, &source_path).unwrap();
        let reopened = File::open(&source_path).unwrap();
        assert_eq!(test_assert_owner_only(&reopened), source_acl);
        assert_eq!(
            std::fs::read(&source_path).unwrap(),
            b"replacement archive bytes"
        );
        drop(reopened);
        std::fs::remove_dir_all(root).unwrap();
    }
}
