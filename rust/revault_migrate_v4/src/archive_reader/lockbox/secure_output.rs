//! Capability-relative, private extraction staging.
#[cfg(any(unix, windows))]
mod native {
    use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
    use cap_std::fs::{Dir, OpenOptions};
    use std::collections::BTreeMap;
    use std::ffi::{OsStr, OsString};
    use std::fs::File;
    use std::io;
    use std::path::{Component, Path, PathBuf};
    use std::sync::Mutex;

    pub(crate) struct OutputRoot {
        dir: Dir,
        final_directories: Mutex<BTreeMap<PathBuf, (Dir, std::fs::Permissions)>>,
    }

    impl OutputRoot {
        // Only the caller-selected destination is ambient authority. Archive paths
        // below this point are resolved one no-follow component at a time.
        pub(crate) fn from_dir(dir: Dir) -> Self {
            Self {
                dir,
                final_directories: Mutex::new(BTreeMap::new()),
            }
        }

        pub(crate) fn open(destination: &Path) -> io::Result<Self> {
            // Ambient authority is restricted to the explicitly supplied root.
            // Descendants are never resolved using this path again.
            let dir = match destination.file_name() {
                Some(name) => {
                    let parent = destination
                        .parent()
                        .filter(|p| !p.as_os_str().is_empty())
                        .unwrap_or(Path::new("."));
                    let parent = Dir::open_ambient_dir(parent, cap_std::ambient_authority())?;
                    parent.open_dir_nofollow(name)?
                }
                None => Dir::open_ambient_dir(destination, cap_std::ambient_authority())?,
            };
            Ok(Self::from_dir(dir))
        }

        pub(crate) fn symlink(
            &self,
            relative: &Path,
            target: &Path,
            overwrite: bool,
        ) -> io::Result<()> {
            let name = relative.file_name().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "missing output name")
            })?;
            let parent = self.directory(relative.parent().unwrap_or(Path::new("")), true)?;
            let stage = PrivateDirectory::new(parent)?;
            let dir = stage.dir.as_ref().unwrap();
            #[cfg(unix)]
            dir.symlink(target, "payload")?;
            #[cfg(windows)]
            dir.symlink_file(target, "payload")?;
            if overwrite {
                dir.rename("payload", &stage.parent, name)?;
            } else {
                publish_noreplace(dir, OsStr::new("payload"), &stage.parent, name)?;
            }
            Ok(())
        }

        pub(crate) fn directory(&self, relative: &Path, create: bool) -> io::Result<Dir> {
            let mut current = self.dir.try_clone()?;
            let mut prefix = PathBuf::new();
            for component in relative.components() {
                let Component::Normal(name) = component else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "unsafe output component",
                    ));
                };
                prefix.push(name);
                current = match current.open_dir_nofollow(name) {
                    Ok(dir) => dir,
                    Err(error) if create && error.kind() == io::ErrorKind::NotFound => {
                        match create_private_directory(&current, name) {
                            Ok(()) => {}
                            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                                current = current.open_dir_nofollow(name)?;
                                continue;
                            }
                            Err(error) => return Err(error),
                        }
                        let child = current.open_dir_nofollow(name)?;
                        let mode = default_directory_permissions(&child)?;
                        self.final_directories
                            .lock()
                            .map_err(|_| io::Error::other("output directory state poisoned"))?
                            .insert(prefix.clone(), (child.try_clone()?, mode));
                        child
                    }
                    Err(error) => return Err(error),
                };
            }
            Ok(current)
        }

        pub(crate) fn restore_directory(&self, relative: &Path, mode: u32) -> io::Result<()> {
            let dir = self.directory(relative, true)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if mode & !0o777 != 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "invalid stored mode",
                    ));
                }
                self.final_directories
                    .lock()
                    .map_err(|_| io::Error::other("output directory state poisoned"))?
                    .insert(
                        relative.to_owned(),
                        (dir, std::fs::Permissions::from_mode(mode)),
                    );
            }
            #[cfg(windows)]
            {
                let _ = (dir, mode);
            }
            Ok(())
        }

        pub(crate) fn finish_directories(&self) -> io::Result<()> {
            let mut directories = std::mem::take(
                &mut *self
                    .final_directories
                    .lock()
                    .map_err(|_| io::Error::other("output directory state poisoned"))?,
            )
            .into_iter()
            .collect::<Vec<_>>();
            directories.sort_by_key(|(path, _)| std::cmp::Reverse(path.components().count()));
            for (_, (dir, permissions)) in directories {
                dir.set_permissions(".", cap_std::fs::Permissions::from_std(permissions))?;
            }
            Ok(())
        }

        pub(crate) fn stage_file(&self, relative: &Path) -> io::Result<StagedFile> {
            let name = relative.file_name().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "output has no filename")
            })?;
            let parent = self.directory(relative.parent().unwrap_or(Path::new("")), true)?;
            StagedFile::new(parent, name.to_owned())
        }
    }

    fn default_directory_permissions(private: &Dir) -> io::Result<std::fs::Permissions> {
        private.create_dir(".mode-probe")?;
        let probe = private.open_dir_nofollow(".mode-probe")?;
        let permissions = probe.into_std_file().metadata()?.permissions();
        private.remove_dir(".mode-probe")?;
        Ok(permissions)
    }

    pub(crate) struct StagedFile {
        stage: PrivateDirectory,
        destination: OsString,
        file: Option<File>,
        default_permissions: std::fs::Permissions,
    }

    pub(crate) struct StagedDirectory {
        stage: PrivateDirectory,
        destination: OsString,
        default_permissions: std::fs::Permissions,
    }

    impl StagedDirectory {
        pub(crate) fn at(destination: &Path) -> io::Result<(Self, OutputRoot)> {
            let name = destination.file_name().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "destination must name a directory",
                )
            })?;
            let parent = destination
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            std::fs::create_dir_all(parent)?;
            let parent = Dir::open_ambient_dir(parent, cap_std::ambient_authority())?;
            Self::new(parent, name.to_owned())
        }
        pub(crate) fn new(parent: Dir, destination: OsString) -> io::Result<(Self, OutputRoot)> {
            let stage = PrivateDirectory::new(parent)?;
            let dir = stage.dir.as_ref().unwrap();
            create_private_directory(dir, OsStr::new("payload"))?;
            let payload = dir.open_dir_nofollow("payload")?;
            let default_permissions = default_directory_permissions(&payload)?;
            let root = OutputRoot::from_dir(payload);
            Ok((
                Self {
                    stage,
                    destination,
                    default_permissions,
                },
                root,
            ))
        }

        pub(crate) fn publish(self, root: OutputRoot) -> io::Result<()> {
            root.dir.set_permissions(
                ".",
                cap_std::fs::Permissions::from_std(self.default_permissions.clone()),
            )?;
            root.finish_directories()?;
            // Windows directory handles intentionally deny deletion. Close all
            // descendant capabilities before the exclusive directory rename.
            drop(root);
            publish_noreplace(
                self.stage.dir.as_ref().unwrap(),
                OsStr::new("payload"),
                &self.stage.parent,
                &self.destination,
            )
        }
    }

    struct PrivateDirectory {
        parent: Dir,
        name: OsString,
        dir: Option<Dir>,
    }

    impl PrivateDirectory {
        fn new(parent: Dir) -> io::Result<Self> {
            for _ in 0..32 {
                let mut nonce = [0u8; 16];
                getrandom::fill(&mut nonce).map_err(io::Error::other)?;
                let name = OsString::from(format!(
                    ".lockbox-private-{:032x}",
                    u128::from_le_bytes(nonce)
                ));
                match create_private_directory(&parent, &name) {
                    Ok(()) => {}
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => return Err(error),
                }
                let mut stage = Self {
                    parent,
                    name,
                    dir: None,
                };
                stage.dir = Some(stage.parent.open_dir_nofollow(&stage.name)?);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    let metadata = stage
                        .dir
                        .as_ref()
                        .unwrap()
                        .try_clone()?
                        .into_std_file()
                        .metadata()?;
                    // SAFETY: geteuid has no pointer arguments or preconditions.
                    let effective_uid = unsafe { libc::geteuid() };
                    if metadata.uid() != effective_uid || metadata.mode() & 0o777 != 0o700 {
                        return Err(io::Error::new(
                            io::ErrorKind::PermissionDenied,
                            "private stage ownership or mode changed",
                        ));
                    }
                }
                return Ok(stage);
            }
            Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "private staging namespace unavailable",
            ))
        }
    }

    impl Drop for PrivateDirectory {
        fn drop(&mut self) {
            if let Some(dir) = self.dir.take() {
                if dir.remove_file("payload").is_err() {
                    // cap-std resolves cleanup beneath the held capability; it
                    // does not follow an archive-controlled ancestor pathname.
                    if let Ok(payload) = dir.open_dir_nofollow("payload") {
                        let _ = make_cleanup_writable(&payload);
                        drop(payload);
                    }
                    let _ = dir.remove_dir_all("payload");
                }
                let _ = dir.remove_file("mode-probe");
                drop(dir);
            }
            let _ = self.parent.remove_dir(&self.name);
        }
    }

    fn make_cleanup_writable(dir: &Dir) -> io::Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            dir.set_permissions(
                ".",
                cap_std::fs::Permissions::from_std(std::fs::Permissions::from_mode(0o700)),
            )?;
        }
        for entry in dir.entries()? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let child = dir.open_dir_nofollow(entry.file_name())?;
                make_cleanup_writable(&child)?;
            }
        }
        Ok(())
    }

    // Publish a completed same-directory archive without requiring hard links.
    // Only initial caller-provided parent acquisition uses ambient authority; the
    // exclusive publication itself is relative to the held parent capability.
    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        windows
    ))]
    pub(crate) fn publish_file_noreplace(source: &Path, destination: &Path) -> io::Result<()> {
        let parent_of = |path: &Path| {
            path.parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."))
                .to_path_buf()
        };
        let parent = parent_of(source);
        if parent != parent_of(destination) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "archive publication requires the same parent",
            ));
        }
        let old = source
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "source has no filename"))?;
        let new = destination.file_name().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "destination has no filename")
        })?;
        let parent = Dir::open_ambient_dir(parent, cap_std::ambient_authority())?;
        publish_noreplace(&parent, old, &parent, new)
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    fn publish_noreplace(from: &Dir, old: &OsStr, to: &Dir, new: &OsStr) -> io::Result<()> {
        use std::os::unix::{ffi::OsStrExt, io::AsRawFd};
        let old = std::ffi::CString::new(old.as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid output name"))?;
        let new = std::ffi::CString::new(new.as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid output name"))?;
        // SAFETY: both directory descriptors are held live by the borrowed Dir
        // handles, and both C strings remain valid for the duration of this call.
        #[cfg(target_os = "linux")]
        let result = unsafe {
            libc::renameat2(
                from.as_raw_fd(),
                old.as_ptr(),
                to.as_raw_fd(),
                new.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        // Bionic only exports renameat2 from API 30. Use the kernel ABI on
        // Android so the carrier can load on API 23. Preserve exclusive atomic
        // publication; kernels without this syscall fail rather than clobber.
        // SAFETY: the descriptors and C strings satisfy the same lifetimes
        // above, and these are the documented renameat2 syscall arguments.
        #[cfg(target_os = "android")]
        let result = unsafe {
            libc::syscall(
                libc::SYS_renameat2,
                from.as_raw_fd(),
                old.as_ptr(),
                to.as_raw_fd(),
                new.as_ptr(),
                libc::RENAME_NOREPLACE as libc::c_uint,
            )
        };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    fn publish_noreplace(from: &Dir, old: &OsStr, to: &Dir, new: &OsStr) -> io::Result<()> {
        use std::os::unix::{ffi::OsStrExt, io::AsRawFd};
        let old = std::ffi::CString::new(old.as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid output name"))?;
        let new = std::ffi::CString::new(new.as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid output name"))?;
        if unsafe {
            libc::renameatx_np(
                from.as_raw_fd(),
                old.as_ptr(),
                to.as_raw_fd(),
                new.as_ptr(),
                libc::RENAME_EXCL,
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    #[cfg(windows)]
    fn publish_noreplace(from: &Dir, old: &OsStr, to: &Dir, new: &OsStr) -> io::Result<()> {
        let old = windows_child_path(from, old)?;
        let new = windows_child_path(to, new)?;
        // Like cap-std's Windows rename implementation, both parent capabilities
        // remain open without FILE_SHARE_DELETE for the complete OS operation.
        // No REPLACE_EXISTING and no COPY_ALLOWED: fail rather than clobber/copy.
        if unsafe {
            windows_sys::Win32::Storage::FileSystem::MoveFileExW(old.as_ptr(), new.as_ptr(), 0)
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        windows
    )))]
    fn publish_noreplace(_from: &Dir, _old: &OsStr, _to: &Dir, _new: &OsStr) -> io::Result<()> {
        // Never silently replace a destination or fall back to a non-atomic copy.
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "exclusive directory/symlink publication unavailable on this platform",
        ))
    }

    impl StagedFile {
        pub(crate) fn at(destination: &Path, require_existing: bool) -> io::Result<Self> {
            let name = destination.file_name().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "output has no filename")
            })?;
            let parent = destination
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let parent = Dir::open_ambient_dir(parent, cap_std::ambient_authority())?;
            if require_existing {
                let metadata = parent.symlink_metadata(name)?;
                if metadata.is_dir() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "output is a directory",
                    ));
                }
            }
            Self::new(parent, name.to_owned())
        }

        fn new(parent: Dir, destination: OsString) -> io::Result<Self> {
            for _ in 0..32 {
                let mut nonce = [0u8; 16];
                getrandom::fill(&mut nonce).map_err(io::Error::other)?;
                let stage_name = OsString::from(format!(
                    ".lockbox-private-{:032x}",
                    u128::from_le_bytes(nonce)
                ));
                match create_private_directory(&parent, &stage_name) {
                    Ok(()) => {}
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => return Err(error),
                }
                // Install cleanup ownership immediately after creation, including
                // failures in opening the directory, metadata, or permission setup.
                let mut stage = PrivateDirectory {
                    parent,
                    name: stage_name,
                    dir: None,
                };
                stage.dir = Some(stage.parent.open_dir_nofollow(&stage.name)?);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    let metadata = stage
                        .dir
                        .as_ref()
                        .unwrap()
                        .try_clone()?
                        .into_std_file()
                        .metadata()?;
                    // SAFETY: geteuid has no pointer arguments or preconditions.
                    let effective_uid = unsafe { libc::geteuid() };
                    if metadata.uid() != effective_uid || metadata.mode() & 0o777 != 0o700 {
                        return Err(io::Error::new(
                            io::ErrorKind::PermissionDenied,
                            "private stage ownership or mode changed",
                        ));
                    }
                }
                // This distinct empty probe never contains archive bytes. The
                // actual payload is independently private at atomic creation.
                let mut options = OpenOptions::new();
                options
                    .write(true)
                    .create_new(true)
                    .follow(FollowSymlinks::No);
                let probe = stage
                    .dir
                    .as_ref()
                    .unwrap()
                    .open_with("mode-probe", &options)?
                    .into_std();
                let default_permissions = probe.metadata()?.permissions();
                drop(probe);
                stage.dir.as_ref().unwrap().remove_file("mode-probe")?;
                #[cfg(unix)]
                let file = {
                    use cap_std::fs::OpenOptionsExt;
                    options.mode(0o600);
                    stage
                        .dir
                        .as_ref()
                        .unwrap()
                        .open_with("payload", &options)?
                        .into_std()
                };
                #[cfg(windows)]
                let file = create_private_file(stage.dir.as_ref().unwrap(), OsStr::new("payload"))?;
                return Ok(Self {
                    stage,
                    destination,
                    file: Some(file),
                    default_permissions,
                });
            }
            Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "private staging namespace unavailable",
            ))
        }

        pub(crate) fn writer(&mut self) -> &mut File {
            self.file.as_mut().unwrap()
        }

        pub(crate) fn publish(
            mut self,
            overwrite: bool,
            stored_permissions: Option<u32>,
        ) -> io::Result<()> {
            let permissions = match stored_permissions {
                #[cfg(unix)]
                Some(mode) => {
                    use std::os::unix::fs::PermissionsExt;
                    if mode & !0o777 != 0 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "invalid stored mode",
                        ));
                    }
                    std::fs::Permissions::from_mode(mode)
                }
                _ => self.default_permissions.clone(),
            };
            self.file.as_ref().unwrap().set_permissions(permissions)?;
            // In particular, Windows cleanup/publication must not retain an open
            // payload handle with incompatible sharing permissions.
            drop(self.file.take());
            let dir = self.stage.dir.as_ref().unwrap();
            if overwrite {
                // rename replaces the leaf itself; it never follows the leaf.
                dir.rename("payload", &self.stage.parent, &self.destination)?;
            } else {
                // Same-filesystem no-clobber publication, no check-then-rename and
                // no copy fallback that could truncate another destination.
                publish_noreplace(
                    dir,
                    OsStr::new("payload"),
                    &self.stage.parent,
                    &self.destination,
                )?;
            }
            Ok(())
        }
    }

    impl Drop for StagedFile {
        fn drop(&mut self) {
            drop(self.file.take());
            // PrivateDirectory closes its directory handle before removing it.
        }
    }

    #[cfg(unix)]
    fn create_private_directory(parent: &Dir, name: &OsStr) -> io::Result<()> {
        use cap_std::fs::{DirBuilder, DirBuilderExt};
        let mut builder = DirBuilder::new();
        builder.mode(0o700);
        parent.create_dir_with(name, &builder)
    }

    #[cfg(windows)]
    fn create_private_directory(parent: &Dir, name: &OsStr) -> io::Result<()> {
        with_private_windows_child(parent, name, |path, attributes| {
            if unsafe {
                windows_sys::Win32::Storage::FileSystem::CreateDirectoryW(path, attributes)
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        })
    }

    #[cfg(windows)]
    fn create_private_file(parent: &Dir, name: &OsStr) -> io::Result<File> {
        use std::os::windows::io::FromRawHandle;
        use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::Storage::FileSystem::{
            CreateFileW, CREATE_NEW, FILE_ATTRIBUTE_NORMAL,
        };
        with_private_windows_child(parent, name, |path, attributes| {
            let handle = unsafe {
                CreateFileW(
                    path,
                    GENERIC_READ | GENERIC_WRITE,
                    0,
                    attributes,
                    CREATE_NEW,
                    FILE_ATTRIBUTE_NORMAL,
                    std::ptr::null_mut(),
                )
            };
            if handle == INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error());
            }
            Ok(unsafe { File::from_raw_handle(handle) })
        })
    }

    #[cfg(windows)]
    fn with_private_windows_child<T>(
        parent: &Dir,
        name: &OsStr,
        operation: impl FnOnce(
            *const u16,
            *const windows_sys::Win32::Security::SECURITY_ATTRIBUTES,
        ) -> io::Result<T>,
    ) -> io::Result<T> {
        use windows_sys::Win32::Foundation::LocalFree;
        use windows_sys::Win32::Security::Authorization::{
            ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
        };
        use windows_sys::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
        let path = windows_child_path(parent, name)?;
        // Protected DACL; only the owner has access, inherited by files and dirs.
        let descriptor_text = "D:P(A;OICI;FA;;;OW)\0".encode_utf16().collect::<Vec<_>>();
        let mut descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                descriptor_text.as_ptr(),
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
        operation(path.as_ptr(), &attributes)
    }

    #[cfg(windows)]
    fn windows_child_path(parent: &Dir, name: &OsStr) -> io::Result<Vec<u16>> {
        use std::os::windows::{ffi::OsStrExt, io::AsRawHandle};
        use windows_sys::Win32::Storage::FileSystem::GetFinalPathNameByHandleW;
        if Path::new(name).components().count() != 1
            || !matches!(
                Path::new(name).components().next(),
                Some(Component::Normal(_))
            )
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid private directory name",
            ));
        }
        // cap-std opens Windows directories without delete sharing. Keep this
        // parent handle alive while resolving its OS name and creating its child.
        let needed = unsafe {
            GetFinalPathNameByHandleW(parent.as_raw_handle(), std::ptr::null_mut(), 0, 0)
        };
        if needed == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut path = vec![0u16; needed as usize + 1];
        let written = unsafe {
            GetFinalPathNameByHandleW(
                parent.as_raw_handle(),
                path.as_mut_ptr(),
                path.len() as u32,
                0,
            )
        };
        if written == 0 {
            return Err(io::Error::last_os_error());
        }
        if written as usize >= path.len() {
            return Err(io::Error::other("parent path changed during resolution"));
        }
        path.truncate(written as usize);
        path.push(b'\\' as u16);
        path.extend(name.encode_wide());
        path.push(0);
        Ok(path)
    }

    #[cfg(all(test, windows))]
    mod windows_security_tests {
        use super::*;
        use crate::archive_reader::storage::test_assert_owner_only;
        use std::io::Write;

        #[test]
        fn staged_output_has_private_dacl_before_bytes_and_after_publication() {
            let mut nonce = [0; 16];
            getrandom::fill(&mut nonce).unwrap();
            let parent = std::env::temp_dir().join(format!(
                "revault-private-output-{:032x}",
                u128::from_le_bytes(nonce)
            ));
            std::fs::create_dir(&parent).unwrap();
            let destination = parent.join("result");
            let mut staged = StagedFile::at(&destination, false).unwrap();
            test_assert_owner_only(staged.writer());
            let stage_dir = staged
                .stage
                .dir
                .as_ref()
                .unwrap()
                .try_clone()
                .unwrap()
                .into_std_file();
            test_assert_owner_only(&stage_dir);
            drop(stage_dir);
            staged.writer().write_all(b"private output bytes").unwrap();
            assert!(!destination.exists());
            staged.publish(false, Some(0o600)).unwrap();
            let published = File::open(&destination).unwrap();
            test_assert_owner_only(&published);
            assert_eq!(
                std::fs::read(&destination).unwrap(),
                b"private output bytes"
            );
            drop(published);
            {
                let mut abandoned = StagedFile::at(&destination, true).unwrap();
                test_assert_owner_only(abandoned.writer());
                abandoned.writer().write_all(b"unfinished").unwrap();
            }
            assert_eq!(
                std::fs::read(&destination).unwrap(),
                b"private output bytes"
            );
            assert_eq!(std::fs::read_dir(&parent).unwrap().count(), 1);
            std::fs::remove_dir_all(parent).unwrap();
        }
    }

    // Native helper tests; property/lifecycle checks, no race or attack reproduction.
    #[cfg(all(test, unix))]
    mod security_tests {
        use super::*;
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;

        fn root(label: &str) -> PathBuf {
            let mut nonce = [0u8; 16];
            getrandom::fill(&mut nonce).unwrap();
            let path = std::env::temp_dir().join(format!(
                "revault-{label}-{:032x}",
                u128::from_le_bytes(nonce)
            ));
            std::fs::create_dir(&path).unwrap();
            path
        }

        #[test]
        fn staged_payload_is_private_before_bytes_and_final_mode_precedes_publication() {
            let parent = root("private-output");
            let destination = parent.join("result");
            let mut staged = StagedFile::at(&destination, false).unwrap();
            assert_eq!(
                staged.writer().metadata().unwrap().permissions().mode() & 0o777,
                0o600
            );
            let stage_dir = staged
                .stage
                .dir
                .as_ref()
                .unwrap()
                .try_clone()
                .unwrap()
                .into_std_file();
            assert_eq!(
                stage_dir.metadata().unwrap().permissions().mode() & 0o777,
                0o700
            );
            staged
                .writer()
                .write_all(b"private archive content")
                .unwrap();
            assert!(!destination.exists());
            staged.publish(false, Some(0o640)).unwrap();
            assert_eq!(
                std::fs::read(&destination).unwrap(),
                b"private archive content"
            );
            assert_eq!(
                std::fs::metadata(&destination)
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o640
            );
            assert_eq!(std::fs::read_dir(&parent).unwrap().count(), 1);
            drop(stage_dir);
            std::fs::remove_dir_all(parent).unwrap();
        }

        #[test]
        fn abandoning_or_refusing_stage_preserves_existing_destination_and_cleans_private_files() {
            let parent = root("private-output-refusal");
            let destination = parent.join("result");
            std::fs::write(&destination, b"original").unwrap();
            {
                let mut staged = StagedFile::at(&destination, true).unwrap();
                staged.writer().write_all(b"unfinished").unwrap();
            }
            assert_eq!(std::fs::read(&destination).unwrap(), b"original");
            let mut staged = StagedFile::at(&destination, false).unwrap();
            staged.writer().write_all(b"completed but refused").unwrap();
            assert_eq!(
                staged.publish(false, Some(0o600)).unwrap_err().kind(),
                io::ErrorKind::AlreadyExists
            );
            assert_eq!(std::fs::read(&destination).unwrap(), b"original");
            assert_eq!(std::fs::read_dir(&parent).unwrap().count(), 1);
            std::fs::remove_dir_all(parent).unwrap();
        }

        #[test]
        fn readonly_directory_modes_are_deferred_until_children_are_complete() {
            let parent = root("private-directory-mode");
            let destination = parent.join("result");
            let (stage, root) = StagedDirectory::at(&destination).unwrap();
            root.restore_directory(Path::new("readonly"), 0o500)
                .unwrap();
            let mut file = root.stage_file(Path::new("readonly/content")).unwrap();
            file.writer().write_all(b"content").unwrap();
            file.publish(false, Some(0o400)).unwrap();
            stage.publish(root).unwrap();
            assert_eq!(
                std::fs::read(destination.join("readonly/content")).unwrap(),
                b"content"
            );
            assert_eq!(
                std::fs::metadata(destination.join("readonly"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o500
            );
            std::fs::set_permissions(
                destination.join("readonly"),
                std::fs::Permissions::from_mode(0o700),
            )
            .unwrap();
            std::fs::remove_dir_all(parent).unwrap();
        }
    }
}
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios",
    windows
))]
pub(crate) use native::publish_file_noreplace;
#[cfg(any(unix, windows))]
pub(crate) use native::{OutputRoot, StagedDirectory, StagedFile};
#[cfg(not(any(unix, windows)))]
mod unavailable {
    use std::{fs::File, io, path::Path};
    fn unavailable<T>() -> io::Result<T> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "secure filesystem extraction is unavailable on this platform",
        ))
    }
    pub(crate) struct OutputRoot;
    pub(crate) struct StagedDirectory;
    pub(crate) struct StagedFile;
    impl OutputRoot {
        pub(crate) fn open(_: &Path) -> io::Result<Self> {
            unavailable()
        }
        pub(crate) fn directory(&self, _: &Path, _: bool) -> io::Result<()> {
            unavailable()
        }
        pub(crate) fn restore_directory(&self, _: &Path, _: u32) -> io::Result<()> {
            unavailable()
        }
        pub(crate) fn finish_directories(&self) -> io::Result<()> {
            unavailable()
        }
        pub(crate) fn stage_file(&self, _: &Path) -> io::Result<StagedFile> {
            unavailable()
        }
        pub(crate) fn symlink(&self, _: &Path, _: &Path, _: bool) -> io::Result<()> {
            unavailable()
        }
    }
    impl StagedDirectory {
        pub(crate) fn at(_: &Path) -> io::Result<(Self, OutputRoot)> {
            unavailable()
        }
        pub(crate) fn publish(self, _: OutputRoot) -> io::Result<()> {
            unavailable()
        }
    }
    impl StagedFile {
        pub(crate) fn at(_: &Path, _: bool) -> io::Result<Self> {
            unavailable()
        }
        pub(crate) fn writer(&mut self) -> &mut File {
            unreachable!("no staged file can be constructed on this platform")
        }
        pub(crate) fn publish(self, _: bool, _: Option<u32>) -> io::Result<()> {
            unavailable()
        }
    }
}
#[cfg(not(any(unix, windows)))]
pub(crate) use unavailable::{OutputRoot, StagedDirectory, StagedFile};
