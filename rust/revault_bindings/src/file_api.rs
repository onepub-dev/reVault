//! Native file operations shared by the language facades.
use super::*;

/// Opens, creates, or atomically replaces a native lockbox and retains its lock.
///
/// `mode` is UTF-8 `open`, `create`, or `replace`. `credential` is `password`,
/// `content-key`, `contact`, `vault`, or `unencrypted`. `secret` contains UTF-8 password bytes or the raw
/// content key, and is empty for contact and unencrypted credentials.
/// Unencrypted mode only opens existing plaintext files read-only; signer and
/// contact must be null and no secret is accepted. `contact` is a public key
/// for creation, a private keypair for contact opening, or an open VaultDirectory
/// for vault credential resolution. Otherwise it must be null. Vault resolution
/// is process-local, uses remembered passwords/profile keys, and never uses the agent.
/// A null signer on open selects shared read-only access; a supplied signer
/// selects exclusive write access. Creation generates a signer when null.
/// Runtime tuning has the same contract as `lockbox_open_with_options`.
/// Close with `lockbox_free`; mutations persist through `lockbox_commit`.
///
/// # Safety
/// Pointer/length inputs must be readable for this call. Non-null opaque keys
/// must be live handles of the documented type. The returned handle is owned.
#[no_mangle]
pub unsafe extern "C" fn lockbox_file(
    path: *const c_char,
    path_len: usize,
    mode: *const c_char,
    mode_len: usize,
    credential: *const c_char,
    credential_len: usize,
    secret: *const u8,
    secret_len: usize,
    contact: *const c_void,
    signer: *const c_void,
    cache_mode: *const c_char,
    cache_len: usize,
    cache_bytes: u64,
    workload: *const c_char,
    workload_len: usize,
    worker: *const c_char,
    worker_len: usize,
    jobs: usize,
) -> *mut c_void {
    let result = (|| -> LockboxResult<Lockbox> {
        let invalid = || {
            revault_lockbox_api::Error::InvalidOperation("invalid file operation arguments".into())
        };
        let path = unsafe { input_str(path, path_len) }.ok_or_else(invalid)?;
        let mode = unsafe { input_str(mode, mode_len) }.ok_or_else(invalid)?;
        let credential = unsafe { input_str(credential, credential_len) }.ok_or_else(invalid)?;
        let secret = unsafe { input(secret, secret_len) }.ok_or_else(invalid)?;
        if path.is_empty() || path.contains('\0') || !matches!(mode, "open" | "create" | "replace")
        {
            return Err(invalid());
        }
        if matches!(credential, "contact" | "vault") == contact.is_null()
            || (matches!(credential, "contact" | "vault") && !secret.is_empty())
            || (credential == "vault" && mode != "open")
            || (credential == "unencrypted"
                && (mode != "open"
                    || !secret.is_empty()
                    || !signer.is_null()
                    || !contact.is_null()))
        {
            return Err(invalid());
        }
        let options = lockbox_options(
            cache_mode,
            cache_len,
            cache_bytes,
            workload,
            workload_len,
            worker,
            worker_len,
            jobs,
        )
        .map_err(revault_lockbox_api::Error::InvalidOperation)?;
        let signer = if signer.is_null() {
            None
        } else {
            Some(unsafe { &*signer.cast::<OwnerSigningKeyPair>() })
        };
        let password = if credential == "password" {
            Some(SecretString::try_from_slice(secret)?)
        } else {
            None
        };
        let path = std::path::Path::new(path);
        if mode == "open" {
            if credential == "vault" {
                return open_from_vault(
                    path,
                    unsafe { &*contact.cast::<VaultDirectoryHandle>() },
                    signer,
                    options,
                );
            }
            let open = match credential {
                "unencrypted" => LockboxOpen::Unencrypted,
                "password" => LockboxOpen::Password(password.as_ref().unwrap()),
                "content-key" => {
                    LockboxOpen::ContentKey(revault_lockbox_api::SecretVec::try_from_slice(secret)?)
                }
                "contact" => LockboxOpen::ContactKeyPair(ContactKeyPair::from_private_key_record(
                    unsafe { &*contact.cast::<ContactKeyPair>() }.private_key_record()?,
                )?),
                _ => return Err(invalid()),
            };
            Lockbox::open_file_handle(path, open, signer, options)
        } else {
            let protection = match credential {
                "password" => LockboxProtection::Password(password.as_ref().unwrap()),
                "content-key" => LockboxProtection::ContentKey(
                    revault_lockbox_api::SecretVec::try_from_slice(secret)?,
                ),
                "contact" => LockboxProtection::ContactPublicKey {
                    name: None,
                    contact: unsafe { &*contact.cast::<ContactPublicKey>() }.clone(),
                },
                _ => return Err(invalid()),
            };
            let generated;
            let signer = match signer {
                Some(signer) => signer,
                None => {
                    generated = OwnerSigningKeyPair::generate()?;
                    &generated
                }
            };
            Lockbox::create_file_handle(path, protection, signer, options, mode == "replace")
        }
    })();
    match result {
        Ok(lockbox) => {
            clear_error();
            Box::into_raw(Box::new(lockbox)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

// Credential selection stays in Rust; only authentication misses try another key.
// Lock, I/O, corruption and recovery errors must reach the caller unchanged.
fn open_from_vault(
    path: &std::path::Path,
    vault: &VaultDirectoryHandle,
    signer: Option<&OwnerSigningKeyPair>,
    options: revault_lockbox_api::LockboxOptions,
) -> LockboxResult<Lockbox> {
    let id = Lockbox::inspect_file(path)?.lockbox_id;
    let verify_identity = |result: LockboxResult<Lockbox>| {
        result.and_then(|lockbox| {
            if lockbox.lockbox_id() != id {
                return Err(revault_lockbox_api::Error::LockUnavailable(
                    "archive changed during vault credential resolution; retry the open".into(),
                ));
            }
            Ok(lockbox)
        })
    };
    if let Some(password) = vault.remembered_lockbox_password(id)? {
        match verify_identity(Lockbox::open_file_handle(
            path,
            LockboxOpen::Password(&password),
            signer,
            options,
        )) {
            Err(revault_lockbox_api::Error::InvalidKey) => {}
            result => return result,
        }
    }
    for profile in vault.list_private_keys()? {
        let contact = vault.load_private_key(&profile)?;
        match verify_identity(Lockbox::open_file_handle(
            path,
            LockboxOpen::ContactKeyPair(contact),
            signer,
            options,
        )) {
            Err(revault_lockbox_api::Error::InvalidKey) => {}
            result => return result,
        }
    }
    Err(revault_lockbox_api::Error::InvalidKey)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{ffi::CStr, path::Path, process::Command};

    struct Handle(*mut c_void);
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe { lockbox_free(self.0) };
        }
    }
    fn diagnostic() -> String {
        unsafe { CStr::from_ptr(buffer_last_error()) }
            .to_string_lossy()
            .into_owned()
    }
    fn file(path: &Path, mode: &str, signer: *const c_void) -> *mut c_void {
        let path = path.to_str().unwrap();
        unsafe {
            lockbox_file(
                path.as_ptr().cast(),
                path.len(),
                mode.as_ptr().cast(),
                mode.len(),
                b"content-key".as_ptr().cast(),
                11,
                b"KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK".as_ptr(),
                32,
                ptr::null(),
                signer,
                b"disabled".as_ptr().cast(),
                8,
                0,
                b"read-mostly".as_ptr().cast(),
                11,
                b"single".as_ptr().cast(),
                6,
                0,
            )
        }
    }
    fn opened(path: &Path, mode: &str, signer: *const c_void) -> Handle {
        let handle = file(path, mode, signer);
        assert!(!handle.is_null(), "{}", diagnostic());
        Handle(handle)
    }
    fn contents(handle: &Handle, expected: &[u8]) {
        let result = unsafe { lockbox_get_file(handle.0, b"/hello".as_ptr().cast(), 6) };
        assert!(!result.ptr.is_null(), "{}", diagnostic());
        let bytes = unsafe { std::slice::from_raw_parts(result.ptr, result.len) };
        assert_eq!(bytes, expected);
        unsafe { buffer_free(result) };
    }
    fn probe(path: &Path, mode: &str, blocked: bool) {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "file_api::tests::file_lock_probe", "--nocapture"])
            .env("REVAULT_FILE_PROBE_PATH", path)
            .env("REVAULT_FILE_PROBE_MODE", mode)
            .env(
                "REVAULT_FILE_PROBE_BLOCKED",
                if blocked { "yes" } else { "no" },
            )
            .env("LOCKBOX_LOCK_TIMEOUT_MS", "30")
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
    fn unencrypted_file_open_is_read_only_and_preserves_bytes() {
        use revault_lockbox_api::{Encryption, LockboxCreateOptions, LockboxPath, Signing};
        let root =
            std::env::temp_dir().join(format!("revault-ffi-plaintext-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("docs.lbox");
        {
            // Plaintext creation is not exposed by this C file API; use the
            // public Rust API to create its fixture without archive internals.
            let mut writer = Lockbox::create_file_with_options(
                &path,
                LockboxCreateOptions::new(Encryption::None, Signing::None),
            )
            .unwrap();
            writer
                .add_file(
                    &LockboxPath::new("/hello").unwrap(),
                    b"plaintext bytes",
                    false,
                )
                .unwrap();
            writer.commit().unwrap();
        }
        let before = std::fs::read(&path).unwrap();
        let name = path.to_str().unwrap();
        let open = |mode: &str, secret: &[u8]| unsafe {
            lockbox_file(
                name.as_ptr().cast(),
                name.len(),
                mode.as_ptr().cast(),
                mode.len(),
                b"unencrypted".as_ptr().cast(),
                11,
                secret.as_ptr(),
                secret.len(),
                ptr::null(),
                ptr::null(),
                b"disabled".as_ptr().cast(),
                8,
                0,
                b"read-mostly".as_ptr().cast(),
                11,
                b"single".as_ptr().cast(),
                6,
                0,
            )
        };
        assert!(open("replace", &[]).is_null());
        assert!(open("open", b"unexpected credential").is_null());
        let raw = open("open", &[]);
        assert!(!raw.is_null(), "{}", diagnostic());
        let reader = Handle(raw);
        contents(&reader, b"plaintext bytes");
        let bytes = unsafe { lockbox_read_range(reader.0, b"/hello".as_ptr().cast(), 6, 2, 5) };
        assert!(!bytes.ptr.is_null(), "{}", diagnostic());
        assert_eq!(
            unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) },
            b"ainte"
        );
        unsafe {
            buffer_free(bytes);
        }
        assert!(!unsafe {
            lockbox_add_file(
                reader.0,
                b"/new".as_ptr().cast(),
                4,
                b"x".as_ptr(),
                1,
                false,
            )
        });
        assert!(!unsafe { lockbox_commit(reader.0) });
        drop(reader);
        assert_eq!(before, std::fs::read(&path).unwrap());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn file_lock_probe() {
        let Some(path) = std::env::var_os("REVAULT_FILE_PROBE_PATH") else {
            return;
        };
        let mode = std::env::var("REVAULT_FILE_PROBE_MODE").unwrap();
        let signer = key_signing_generate();
        let handle = file(
            Path::new(&path),
            if mode == "replace" { "replace" } else { "open" },
            if mode == "read" { ptr::null() } else { signer },
        );
        if std::env::var("REVAULT_FILE_PROBE_BLOCKED").unwrap() == "yes" {
            assert!(handle.is_null());
            assert!(diagnostic().contains("lock"), "{}", diagnostic());
        } else {
            assert!(!handle.is_null(), "{}", diagnostic());
        }
        unsafe {
            lockbox_free(handle);
            key_signing_free(signer);
        }
    }

    #[test]
    fn native_file_handles_preserve_locks_persistence_and_replacement() {
        let root = std::env::temp_dir().join(format!("revault-ffi-files-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("archive.lbox");
        let signer = key_signing_generate();
        assert!(!signer.is_null());
        assert!(file(&path, "invalid", signer).is_null());
        assert!(!path.exists());
        let writer = opened(&path, "create", signer);
        let payload = b"exact file bytes\0\xff";
        assert!(unsafe {
            lockbox_add_file(
                writer.0,
                b"/hello".as_ptr().cast(),
                6,
                payload.as_ptr(),
                payload.len(),
                false,
            )
        });
        assert!(unsafe { lockbox_commit(writer.0) });
        probe(&path, "read", true);
        drop(writer);
        let reader = opened(&path, "open", ptr::null());
        let second = opened(&path, "open", ptr::null());
        contents(&reader, payload);
        assert!(!unsafe { lockbox_set_owner_signing_key(reader.0, signer) });
        assert!(!unsafe {
            lockbox_add_file(
                reader.0,
                b"/bad".as_ptr().cast(),
                4,
                payload.as_ptr(),
                payload.len(),
                false,
            )
        });
        assert!(!unsafe { lockbox_commit(reader.0) });
        assert!(unsafe {
            lockbox_set_workload_profile(reader.0, b"interactive".as_ptr().cast(), 11)
        });
        probe(&path, "read", false);
        probe(&path, "write", true);
        probe(&path, "replace", true);
        drop(reader);
        probe(&path, "write", true);
        drop(second);
        let writer = opened(&path, "open", signer);
        assert!(unsafe {
            lockbox_add_file(
                writer.0,
                b"/hello".as_ptr().cast(),
                6,
                b"updated".as_ptr(),
                7,
                true,
            )
        });
        assert!(unsafe { lockbox_commit(writer.0) });
        drop(writer);
        let reader = opened(&path, "open", ptr::null());
        contents(&reader, b"updated");
        drop(reader);
        assert!(file(&path, "create", signer).is_null());
        let replacement = opened(&path, "replace", signer);
        assert!(!unsafe { lockbox_exists(replacement.0, b"/hello".as_ptr().cast(), 6) });
        probe(&path, "read", true);
        drop(replacement);
        probe(&path, "read", false);
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        unsafe { key_signing_free(signer) };
        std::fs::remove_dir_all(root).unwrap();
    }
}
