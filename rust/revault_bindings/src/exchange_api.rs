//! Shared native exchange workflow. The session owns credentials, not a Vault lock.
use super::*;
use std::path::PathBuf;

struct ExchangeSession {
    path: PathBuf,
    password: SecretString,
}

/// Opens a process-local exchange session for an existing Vault file. The
/// passphrase is copied into protected memory. A brief Vault open validates it;
/// the returned session retains no Vault/file lock and never uses the Agent.
/// Release with contact_exchange_free.
///
/// # Safety
/// Path and password must be readable for their respective lengths.
#[no_mangle]
pub unsafe extern "C" fn contact_exchange_open(
    path: *const c_char,
    path_len: usize,
    password: *const u8,
    password_len: usize,
) -> *mut c_void {
    let result = (|| -> LockboxResult<ExchangeSession> {
        let path = unsafe { input_str(path, path_len) }
            .ok_or_else(|| revault_lockbox_api::Error::InvalidInput("invalid Vault path".into()))?;
        let password = unsafe { input(password, password_len) }.ok_or_else(|| {
            revault_lockbox_api::Error::InvalidInput("invalid Vault passphrase".into())
        })?;
        let password = SecretString::try_from_slice(password)?;
        drop(revault_vault_api::VaultDirectory::open_file(
            path, &password,
        )?);
        Ok(ExchangeSession {
            path: PathBuf::from(path),
            password,
        })
    })();
    match result {
        Ok(session) => {
            clear_error();
            Box::into_raw(Box::new(session)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

/// Executes a typed facade operation. The private wire request is bounded UTF-8
/// JSON matching ContactExchangeCommand. Returns ContactExchangeStatusList as a
/// FlatBuffer; release it with buffer_free. No management capabilities are returned.
/// No Vault handle is held across HTTP requests or application interaction.
///
/// # Safety
/// Handle must be a live contact_exchange_open result; request must be readable
/// for request_len bytes. Do not free the session concurrently with this call.
#[no_mangle]
pub unsafe extern "C" fn contact_exchange_execute(
    handle: *const c_void,
    request: *const c_char,
    request_len: usize,
) -> RevaultBuffer {
    let result = (|| -> LockboxResult<Vec<revault_vault_api::ContactExchangeStatus>> {
        let session = unsafe { handle.cast::<ExchangeSession>().as_ref() }.ok_or_else(|| {
            revault_lockbox_api::Error::InvalidInput("null exchange session".into())
        })?;
        if request_len > 16 * 1024 {
            return Err(revault_lockbox_api::Error::InvalidInput(
                "exchange request too large".into(),
            ));
        }
        let request = unsafe { input(request.cast(), request_len) }.ok_or_else(|| {
            revault_lockbox_api::Error::InvalidInput("invalid exchange request".into())
        })?;
        let command = serde_json::from_slice(request).map_err(|_| {
            revault_lockbox_api::Error::InvalidInput("invalid exchange operation or fields".into())
        })?;
        revault_vault_api::contact_exchange(&session.path, &session.password, command)
    })();
    match result {
        Ok(values) => {
            let data = bindings_transport::ContactExchangeStatusListT {
                values: Some(
                    values
                        .into_iter()
                        .map(|value| bindings_transport::ContactExchangeStatusT {
                            id: Some(value.id),
                            relay: Some(value.relay),
                            invitation: Some(value.invitation),
                            inviter_email: Some(value.inviter_email),
                            recipient_email: Some(value.recipient_email),
                            fingerprint: Some(value.fingerprint),
                            expires_ms: value.expires_ms,
                            accepted: value.accepted,
                            verified: value.verified,
                            contact_name: Some(value.contact_name),
                            relay_pending: value.relay_pending,
                        })
                        .collect(),
                ),
            };
            let mut builder = flatbuffers::FlatBufferBuilder::new();
            let root = data.pack(&mut builder);
            builder.finish(root, None);
            clear_error();
            buffer(builder.finished_data().to_vec())
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

/// Releases an exchange session and its protected passphrase. It does not cancel
/// or delete saved exchanges, contacts, or server invitations.
///
/// # Safety
/// Handle must be null or a live contact_exchange_open result released exactly once.
#[no_mangle]
pub unsafe extern "C" fn contact_exchange_free(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle.cast::<ExchangeSession>())) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use revault_key_server::store::{PublishStore, ServerConfig};
    use revault_vault_api::VaultDirectory;
    use serde_json::json;
    use std::sync::Arc;

    const PASSWORD: &[u8] = b"synthetic exchange passphrase";

    struct Session(*mut c_void);
    impl Session {
        fn open(path: &std::path::Path) -> Self {
            let path = path.to_str().unwrap();
            // All buffers are live and readable; the constructor copies them.
            let handle = unsafe {
                contact_exchange_open(
                    path.as_ptr().cast(),
                    path.len(),
                    PASSWORD.as_ptr(),
                    PASSWORD.len(),
                )
            };
            assert!(!handle.is_null());
            Self(handle)
        }
        fn execute(
            &self,
            request: serde_json::Value,
        ) -> Option<Vec<bindings_transport::ContactExchangeStatusT>> {
            let request = request.to_string();
            // Session and request remain valid throughout the synchronous call.
            let result =
                unsafe { contact_exchange_execute(self.0, request.as_ptr().cast(), request.len()) };
            if result.ptr.is_null() {
                return None;
            }
            // Unpack into owned Rust values before releasing the native buffer.
            let bytes = unsafe { std::slice::from_raw_parts(result.ptr, result.len) };
            let decoded =
                flatbuffers::root::<bindings_transport::ContactExchangeStatusList<'_>>(bytes)
                    .unwrap()
                    .unpack();
            unsafe { buffer_free(result) };
            Some(decoded.values.unwrap_or_default())
        }
    }
    impl Drop for Session {
        fn drop(&mut self) {
            unsafe { contact_exchange_free(self.0) };
        }
    }

    fn setup(path: &std::path::Path, email: &str) -> (Vec<u8>, Vec<u8>) {
        let password = SecretString::try_from_slice(PASSWORD).unwrap();
        let vault = VaultDirectory::create_file(path, &password).unwrap();
        vault
            .store_private_key("default", &ContactKeyPair::generate().unwrap())
            .unwrap();
        vault.store_profile_email("default", email).unwrap();
        (
            vault
                .load_private_key("default")
                .unwrap()
                .public_key()
                .to_bytes(),
            vault
                .load_owner_signing_key("default")
                .unwrap()
                .public_key()
                .to_bytes(),
        )
    }

    #[test]
    fn native_exchange_sessions_complete_both_directions_and_preserve_contacts() {
        let root = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let alice_path = root.path().join("alice.lbox");
        let bob_path = root.path().join("bob.lbox");
        let alice_keys = setup(&alice_path, "alice@example.test");
        let bob_keys = setup(&bob_path, "bob@example.test");
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let relay = format!("http://{}", listener.local_addr().unwrap());
        let store = Arc::new(
            PublishStore::open(ServerConfig {
                state_dir: root.path().join("relay"),
                ..Default::default()
            })
            .unwrap(),
        );
        std::thread::spawn(move || {
            revault_key_server::server::run_listener(listener, store).unwrap()
        });
        let alice = Session::open(&alice_path);
        let bob = Session::open(&bob_path);
        // A session retains credentials, not an open Vault. A different thread
        // can acquire the writable lock while both native sessions stay alive.
        let path = alice_path.clone();
        std::thread::spawn(move || {
            let password = SecretString::try_from_slice(PASSWORD).unwrap();
            drop(VaultDirectory::open_file(path, &password).unwrap());
        })
        .join()
        .unwrap();
        let offered = alice
            .execute(
                json!({"operation":"invite","relay":relay,"profile":"default",
            "recipient_email":"bob@example.test","ttl_hours":24}),
            )
            .unwrap()
            .remove(0);
        assert!(!offered.relay_pending);
        assert!(!offered.verified);
        let id = offered.id.unwrap();
        let accepted = bob
            .execute(
                json!({"operation":"accept","invitation":offered.invitation.unwrap(),
            "profile":"default"}),
            )
            .unwrap()
            .remove(0);
        assert!(!accepted.relay_pending);
        let fetched = alice
            .execute(json!({"operation":"list","offline":false}))
            .unwrap()
            .remove(0);
        assert_eq!(fetched.fingerprint, accepted.fingerprint);
        assert!(alice
            .execute(json!({"operation":"verify","id":id,"contact_name":"bob",
            "fingerprint":"0000","channel":"in-person"}))
            .is_none());
        let verified = alice
            .execute(json!({"operation":"verify","id":id,"contact_name":"bob",
            "fingerprint":fetched.fingerprint,"channel":"in-person"}))
            .unwrap()
            .remove(0);
        assert!(verified.verified);
        assert!(!verified.relay_pending);
        assert!(
            bob.execute(json!({"operation":"verify","id":id,"contact_name":"alice",
            "fingerprint":accepted.fingerprint,"channel":"known-phone-call"}))
                .unwrap()[0]
                .verified
        );
        assert!(
            !alice
                .execute(json!({"operation":"list","offline":false}))
                .unwrap()[0]
                .relay_pending
        );
        alice
            .execute(json!({"operation":"forget","id":id}))
            .unwrap();
        assert!(alice
            .execute(json!({"operation":"list","offline":true}))
            .unwrap()
            .is_empty());
        drop(alice);
        drop(bob);
        let password = SecretString::try_from_slice(PASSWORD).unwrap();
        for (path, name, keys) in [
            (&alice_path, "bob", bob_keys),
            (&bob_path, "alice", alice_keys),
        ] {
            let vault = VaultDirectory::open_file(path, &password).unwrap();
            assert_eq!(vault.load_contact(name).unwrap().to_bytes(), keys.0);
            assert_eq!(
                vault.load_contact_signing_key(name).unwrap().to_bytes(),
                keys.1
            );
        }
    }
}
