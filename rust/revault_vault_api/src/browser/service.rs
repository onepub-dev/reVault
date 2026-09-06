use super::{
    approval, now,
    pairing::{digest, pair_id, read_pair, secure_directory},
    Pairing,
};
use revault_browser_protocol::{
    browser_request::Operation, crypto::seal, response, validation::verify_signed, BrowserRequest,
    BrowserResponse, Error, Message, UnlockEnvelope, MAX_PROTO_BYTES, VERSION,
};
use revault_lockbox_api::{vault_integration::VaultOpen, Lockbox, LockboxOpen, SecretVec};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{Read, Write},
    os::unix::{
        fs::OpenOptionsExt,
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

pub(super) static EPOCH: AtomicU64 = AtomicU64::new(0);
type Approval = dyn Fn(&str, &AtomicBool, u64) -> Result<(), Error> + Send + Sync;
type KeySource = dyn Fn(&Pairing) -> Result<SecretVec, Error> + Send + Sync;
struct Pending {
    cancel: Arc<AtomicBool>,
}
struct State {
    root: PathBuf,
    pending: Mutex<BTreeMap<String, Pending>>,
    cancelled: Mutex<BTreeMap<String, u64>>,
    approval: Arc<Approval>,
    replay: Mutex<Replay>,
    keys: Arc<KeySource>,
    stopped: Arc<AtomicBool>,
}
#[derive(Default, Serialize, Deserialize)]
struct Replay {
    high_water: u64,
    used: BTreeMap<String, u64>,
}
impl Replay {
    fn consume(
        &mut self,
        root: &Path,
        pair: &Pairing,
        request: &revault_browser_protocol::UnlockRequest,
        time: u64,
    ) -> Result<(), Error> {
        if time < self.high_water {
            return Err(Error::Expired);
        }
        self.high_water = time;
        self.used.retain(|_, expiry| *expiry > time);
        // Both request ID and challenge are single-use within a pairing, even
        // if the server mistakenly signs them again with other metadata.
        let pair_id = pair.id();
        let id = digest(&[pair_id.as_bytes(), b"id", request.request_id.as_bytes()]);
        let challenge = digest(&[pair_id.as_bytes(), b"challenge", &request.challenge]);
        if self.used.contains_key(&id) || self.used.contains_key(&challenge) {
            return Err(Error::Replayed);
        }
        if self.used.len() >= 4096 {
            return Err(Error::Busy);
        }
        self.used.insert(id, request.expires_at);
        self.used.insert(challenge, request.expires_at);
        let bytes = serde_json::to_vec(self).map_err(|_| Error::Internal)?;
        let target = root.join("replay.state");
        let temporary = root.join("replay.pending");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&temporary)
            .map_err(|_| Error::Internal)?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|_| Error::Internal)?;
        fs::rename(temporary, target).map_err(|_| Error::Internal)?;
        fs::File::open(root)
            .and_then(|f| f.sync_all())
            .map_err(|_| Error::Internal)
    }
}
/// Lifetime of the browser listener is owned by the existing Session Agent.
pub(crate) struct Service {
    stop: Arc<AtomicBool>,
    path: PathBuf,
}
impl Drop for Service {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        EPOCH.fetch_add(1, Ordering::SeqCst);
        let _ = fs::remove_file(&self.path);
    }
}
pub(crate) fn start(keys: Arc<KeySource>) -> std::io::Result<Service> {
    let root = super::state_directory().map_err(std::io::Error::other)?;
    secure_directory(&root).map_err(std::io::Error::other)?;
    let replay = match fs::read(root.join("replay.state")) {
        Ok(bytes) if bytes.len() <= 1024 * 1024 => {
            serde_json::from_slice(&bytes).map_err(std::io::Error::other)?
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Replay::default(),
        _ => return Err(std::io::Error::other("invalid browser replay state")),
    };
    let path = crate::unix::socket_dir().join("browser.sock");
    crate::unix::remove_stale_socket(&path)?;
    let listener = UnixListener::bind(&path)?;
    listener.set_nonblocking(true)?;
    let stop = Arc::new(AtomicBool::new(false));
    let state = Arc::new(State {
        root,
        pending: Mutex::new(BTreeMap::new()),
        cancelled: Mutex::new(BTreeMap::new()),
        approval: Arc::new(approval::approve),
        replay: Mutex::new(replay),
        keys,
        stopped: stop.clone(),
    });
    let connections = Arc::new(AtomicUsize::new(0));
    let running = stop.clone();
    thread::spawn(move || {
        while !running.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    if !crate::unix::client_matches_current_user(&stream).unwrap_or(false) {
                        continue;
                    }
                    if connections.load(Ordering::SeqCst) >= 12 {
                        continue;
                    }
                    connections.fetch_add(1, Ordering::SeqCst);
                    let state = state.clone();
                    let connections = connections.clone();
                    thread::spawn(move || {
                        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                        let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
                        let result = read_message(&mut stream).and_then(|bytes| {
                            let request = BrowserRequest::decode(bytes.as_slice())
                                .map_err(|_| Error::InvalidRequest)?;
                            state.handle(request)
                        });
                        let _ = write_message(&mut stream, &response(result));
                        connections.fetch_sub(1, Ordering::SeqCst);
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(25))
                }
                Err(_) => break,
            }
        }
    });
    Ok(Service { stop, path })
}
impl State {
    fn handle(&self, browser: BrowserRequest) -> Result<Option<UnlockEnvelope>, Error> {
        if browser.protocol_version != VERSION {
            return Err(Error::UpgradeRequired);
        }
        if browser.frame_id != 0
            || !revault_browser_protocol::validation::identifier(&browser.extension_id)
        {
            return Err(Error::InvalidRequest);
        }
        revault_browser_protocol::validation::validate_origin(&browser.origin)?;
        if self.stopped.load(Ordering::SeqCst) {
            return Err(Error::Cancelled);
        }
        match browser.operation.as_ref().ok_or(Error::InvalidRequest)? {
            Operation::GetCapabilities(true) => Ok(None),
            Operation::CancelRequest(id) => {
                if !revault_browser_protocol::validation::identifier(id) {
                    return Err(Error::InvalidRequest);
                }
                let key = digest(&[
                    browser.origin.as_bytes(),
                    browser.extension_id.as_bytes(),
                    id.as_bytes(),
                ]);
                let pending = self.pending.lock().map_err(|_| Error::Internal)?;
                if let Some(entry) = pending.get(&key) {
                    entry.cancel.store(true, Ordering::SeqCst);
                }
                let time = now()?;
                let mut cancelled = self.cancelled.lock().map_err(|_| Error::Internal)?;
                cancelled.retain(|_, expiry| *expiry > time);
                if cancelled.len() >= 1024 {
                    return Err(Error::Busy);
                }
                cancelled.insert(key, time + 120);
                drop(pending);
                Ok(None)
            }
            Operation::RequestUnlock(signed) => {
                let request = signed.request.as_ref().ok_or(Error::InvalidRequest)?;
                let id = pair_id(
                    &browser.origin,
                    &request.application_id,
                    &request.lockbox_id,
                    &browser.extension_id,
                );
                let pair = read_pair(&self.root, &id)?;
                let request = verify_signed(signed, &pair.server_signing_key, now()?)?;
                if request.origin != pair.origin {
                    return Err(Error::UnpairedRecipient);
                }
                let key = digest(&[
                    browser.origin.as_bytes(),
                    browser.extension_id.as_bytes(),
                    request.request_id.as_bytes(),
                ]);
                let cancel = Arc::new(AtomicBool::new(false));
                if self
                    .cancelled
                    .lock()
                    .map_err(|_| Error::Internal)?
                    .get(&key)
                    .is_some_and(|expiry| *expiry > now().unwrap_or(0))
                {
                    return Err(Error::Cancelled);
                }
                {
                    let mut pending = self.pending.lock().map_err(|_| Error::Internal)?;
                    if pending.contains_key(&key) {
                        return Err(Error::Replayed);
                    }
                    if pending.len() >= 8 {
                        return Err(Error::Busy);
                    }
                    self.replay.lock().map_err(|_| Error::Internal)?.consume(
                        &self.root,
                        &pair,
                        request,
                        now()?,
                    )?;
                    pending.insert(
                        key.clone(),
                        Pending {
                            cancel: cancel.clone(),
                        },
                    );
                    if self
                        .cancelled
                        .lock()
                        .map_err(|_| Error::Internal)?
                        .get(&key)
                        .is_some_and(|expiry| *expiry > now().unwrap_or(0))
                    {
                        cancel.store(true, Ordering::SeqCst);
                    }
                }
                let result = self.unlock(&pair, request, &cancel);
                self.pending
                    .lock()
                    .map_err(|_| Error::Internal)?
                    .remove(&key);
                result.map(Some)
            }
            _ => Err(Error::InvalidRequest),
        }
    }
    fn unlock(
        &self,
        pair: &Pairing,
        request: &revault_browser_protocol::UnlockRequest,
        cancel: &AtomicBool,
    ) -> Result<UnlockEnvelope, Error> {
        let epoch = EPOCH.load(Ordering::SeqCst);
        (self.approval)(
            &format!(
                "Unlock for this server?\n\n{}\nRecipient key ID: {}\nRequest: {}",
                pair.prompt(),
                request.recipient_key_id,
                request.request_id
            ),
            cancel,
            request.expires_at,
        )?;
        let revalidate = || {
            if cancel.load(Ordering::SeqCst)
                || epoch != EPOCH.load(Ordering::SeqCst)
                || self.stopped.load(Ordering::SeqCst)
            {
                return Err(Error::Cancelled);
            }
            revault_browser_protocol::validation::validate_request(request, now()?)?;
            if read_pair(&self.root, &pair.id())? != *pair {
                return Err(Error::UnpairedRecipient);
            }
            Ok(())
        };
        revalidate()?;
        // Use the existing secret-activity/suspend policy while this agent holds
        // decrypted profile or content material. Unit policy tests inject sources.
        #[cfg(not(test))]
        let _activity = crate::begin_secret_activity(crate::SecretActivityKind::Open)
            .map_err(|_| Error::Internal)?;
        revalidate()?;
        let key = (self.keys)(pair)?;
        revalidate()?;
        let envelope = seal(&key, request)?;
        revalidate()?;
        Ok(envelope)
    }
}
/// Forward only typed browser operations over the same-user restricted socket.
/// Does not start an agent or retrieve any plaintext key.
pub fn request(message: &BrowserRequest) -> Result<BrowserResponse, Error> {
    crate::unix::verify_agent_transport_security().map_err(|_| Error::NativeHelperMissing)?;
    let mut stream = UnixStream::connect(crate::unix::socket_dir().join("browser.sock"))
        .map_err(|_| Error::NativeHelperMissing)?;
    if !crate::unix::client_matches_current_user(&stream).map_err(|_| Error::NativeHelperMissing)? {
        return Err(Error::NativeHelperMissing);
    }
    stream
        .set_read_timeout(Some(Duration::from_secs(125)))
        .map_err(|_| Error::Internal)?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|_| Error::Internal)?;
    write_message(&mut stream, message)?;
    BrowserResponse::decode(read_message(&mut stream)?.as_slice())
        .map_err(|_| Error::InvalidRequest)
}
fn read_message(stream: &mut UnixStream) -> Result<Vec<u8>, Error> {
    let mut header = [0; 4];
    stream
        .read_exact(&mut header)
        .map_err(|_| Error::InvalidRequest)?;
    let len = u32::from_le_bytes(header) as usize;
    if len == 0 || len > MAX_PROTO_BYTES {
        return Err(Error::InvalidRequest);
    }
    let mut bytes = vec![0; len];
    stream
        .read_exact(&mut bytes)
        .map_err(|_| Error::InvalidRequest)?;
    Ok(bytes)
}
fn write_message(stream: &mut UnixStream, message: &impl Message) -> Result<(), Error> {
    let bytes = message.encode_to_vec();
    if bytes.len() > MAX_PROTO_BYTES {
        return Err(Error::InvalidRequest);
    }
    stream
        .write_all(&(bytes.len() as u32).to_le_bytes())
        .and_then(|()| stream.write_all(&bytes))
        .map_err(|_| Error::Internal)
}
/// Resolve only the local pairing's profile grant, using existing Auto Open policy.
pub(crate) fn profile_key(pair: &Pairing) -> Result<SecretVec, Error> {
    if VaultOpen::read_lockbox_id(&pair.lockbox_path)
        .map_err(|_| Error::LockboxUnavailable)?
        .to_string()
        != pair.lockbox_id
    {
        return Err(Error::LockboxUnavailable);
    }
    if crate::auto_open_scope().map_err(|_| Error::VaultLocked)? == crate::AutoOpenScope::Off {
        return Err(Error::VaultLocked);
    }
    let vault_path = crate::default_vault_path().map_err(|_| Error::VaultLocked)?;
    let password = match crate::get_vault_unlock_key(&vault_path.to_string_lossy())
        .map_err(|_| Error::VaultLocked)?
    {
        Some(password) => password,
        None => crate::get_platform_vault_password()
            .map_err(|_| Error::SecureStoreUnavailable)?
            .ok_or(Error::VaultLocked)?,
    };
    let vault = crate::VaultDirectory::open_file(
        crate::default_vault_path().map_err(|_| Error::VaultLocked)?,
        &password,
    )
    .map_err(|_| Error::VaultLocked)?;
    let profile = vault
        .load_private_key(&pair.profile)
        .map_err(|_| Error::LockboxUnavailable)?;
    let opened = VaultOpen::path_with_contact(&pair.lockbox_path, &profile)
        .map_err(|_| Error::LockboxUnavailable)?;
    if opened.lockbox_id.to_string() != pair.lockbox_id {
        return Err(Error::LockboxUnavailable);
    }
    opened.try_clone_key().map_err(|_| Error::Internal)
}
pub(crate) fn validate_cached_key(pair: &Pairing, key: SecretVec) -> Result<SecretVec, Error> {
    let lockbox = Lockbox::open(
        &pair.lockbox_path,
        LockboxOpen::ContentKey(key.try_clone().map_err(|_| Error::Internal)?),
    )
    .map_err(|_| Error::LockboxUnavailable)?;
    if lockbox.lockbox_id().to_string() != pair.lockbox_id {
        return Err(Error::LockboxUnavailable);
    }
    Ok(key)
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
