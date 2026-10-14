use super::{
    agent_is_compatible, encode_control_err_response, encode_control_ok_response,
    encode_err_response, encode_forget_all, encode_forget_identifier, encode_get_identifier,
    encode_info, encode_info_response, encode_key_response, encode_list, encode_list_response,
    encode_miss_response, encode_ok_response, encode_put_identifier,
    encode_register_secret_activity, encode_registered_response, encode_stop,
    encode_unregister_secret_activity, frame_header_len, frame_message_type, frame_payload_len,
    is_control_message_type, max_message_bytes, parse_control_request, parse_control_response,
    parse_request, parse_response, AgentRequest, AgentResponse, CachedLockbox, ControlRequest,
    ControlResponse, SecretActivityKind, SecretVec, DEFAULT_TTL_SECONDS,
};
use crate::active_secret::ActiveSecretRegistry;
use crate::agent_config::AgentConfig;
use crate::agent_log::log_agent_event;
use crate::sleep_watcher::{SleepEvent, SleepWatcher};
use revault_lockbox_api::LockboxId;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const IDLE_EXIT_SECONDS: u64 = 10 * 60;

struct CacheEntry {
    key: SecretVec,
    path: Option<String>,
    expires_at: Instant,
}

pub(crate) fn serve_agent() -> io::Result<()> {
    log_agent_event("agent starting");
    let socket = socket_path();
    prepare_socket_dir()?;
    remove_stale_socket(&socket)?;
    let listener = UnixListener::bind(&socket)?;
    listener.set_nonblocking(true)?;

    let cache = Arc::new(Mutex::new(BTreeMap::<String, CacheEntry>::new()));
    let config = AgentConfig::load();
    log_agent_event(format!(
        "agent config prevent_sleep={} terminate_on_suspend={}",
        config.prevent_sleep, config.terminate_on_suspend
    ));
    let active = Arc::new(Mutex::new(ActiveSecretRegistry::new(config)));
    let mut last_activity = Instant::now();
    start_sleep_cache_clearer(cache.clone(), active.clone());
    loop {
        {
            let mut cache = lock_cache(&cache)?;
            log_pruned_expired(&mut cache);
        }
        match listener.accept() {
            Ok((stream, _)) => {
                last_activity = Instant::now();
                let stop = handle_client(stream, &cache, &active).unwrap_or(false);
                if stop {
                    let _ = fs::remove_file(&socket);
                    log_agent_event("agent stopped by request");
                    return Ok(());
                }
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                let cache_empty = lock_cache(&cache)?.is_empty();
                let active_empty = lock_active(&active)?.is_empty();
                if cache_empty
                    && active_empty
                    && last_activity.elapsed() > Duration::from_secs(IDLE_EXIT_SECONDS)
                {
                    let _ = fs::remove_file(&socket);
                    log_agent_event("agent exiting after idle timeout");
                    return Ok(());
                }
                wait_for_agent_connection(&listener, Instant::now() + Duration::from_millis(100))?;
            }
            Err(err) => return Err(err),
        }
    }
}
// Keep the existing maintenance cadence, but wake immediately for a queued
// connection instead of delaying every sequential control request by 100 ms.
fn wait_for_agent_connection(listener: &UnixListener, deadline: Instant) -> io::Result<()> {
    use std::os::fd::AsRawFd;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(());
        }
        let timeout = remaining
            .as_millis()
            .saturating_add(u128::from(remaining.subsec_nanos() % 1_000_000 != 0))
            .min(i32::MAX as u128) as i32;
        let mut descriptor = libc::pollfd {
            fd: listener.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: the listener owns this live descriptor throughout the call;
        // poll receives one initialized pollfd and retains no pointer to it.
        let result = unsafe { libc::poll(&mut descriptor, 1, timeout) };
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        if result == 0 {
            return Ok(());
        }
        if descriptor.revents & libc::POLLNVAL != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid Session Agent listener",
            ));
        }
        if descriptor.revents & (libc::POLLERR | libc::POLLHUP) != 0 {
            return Err(io::Error::other("Session Agent listener unavailable"));
        }
        if descriptor.revents & libc::POLLIN != 0 {
            return Ok(());
        }
        return Err(io::Error::other(
            "unexpected Session Agent listener readiness",
        ));
    }
}

fn start_sleep_cache_clearer(
    cache: Arc<Mutex<BTreeMap<String, CacheEntry>>>,
    active: Arc<Mutex<ActiveSecretRegistry>>,
) {
    crate::sleep_watcher::set_cache_monitor_active(true);
    let result = SleepWatcher::start_handler(move |event| match event {
        SleepEvent::SuspendRequested | SleepEvent::Unavailable => {
            if event == SleepEvent::Unavailable {
                crate::sleep_watcher::set_cache_monitor_active(false);
            }
            if let Ok(mut cache) = cache.lock() {
                let count = cache.len();
                cache.clear();
                log_agent_event(format!(
                    "suspend requested; cleared {count} cached lockboxes"
                ));
            }
            if let Ok(mut active) = active.lock() {
                active.suspend_requested();
            }
        }
        SleepEvent::Resumed => {
            log_agent_event("resume observed");
            if let Ok(mut active) = active.lock() {
                if let Err(err) = active.resumed() {
                    log_agent_event(format!("resume sleep inhibition unavailable: {err}"));
                }
            }
        }
    });
    match result {
        Ok(()) => log_agent_event("sleep watcher started"),
        Err(err) => {
            crate::sleep_watcher::set_cache_monitor_active(false);
            log_agent_event(format!("sleep watcher unavailable: {err}"));
        }
    }
}

fn lock_cache(
    cache: &Arc<Mutex<BTreeMap<String, CacheEntry>>>,
) -> io::Result<std::sync::MutexGuard<'_, BTreeMap<String, CacheEntry>>> {
    cache
        .lock()
        .map_err(|_| io::Error::other("Session Agent cache lock was poisoned"))
}

fn lock_active(
    active: &Arc<Mutex<ActiveSecretRegistry>>,
) -> io::Result<std::sync::MutexGuard<'_, ActiveSecretRegistry>> {
    active
        .lock()
        .map_err(|_| io::Error::other("Session Agent activity lock was poisoned"))
}

#[cfg(test)]
fn sleep_watcher_has_suspend(watcher: &Option<SleepWatcher>) -> bool {
    watcher
        .as_ref()
        .is_some_and(|watcher| watcher.drain().contains(&SleepEvent::SuspendRequested))
}

pub(crate) fn verify_agent_transport_security() -> io::Result<()> {
    prepare_socket_dir()
}

pub(crate) fn suspend_cache_clearing_active() -> io::Result<bool> {
    if !existing_agent_is_reachable()? {
        return Ok(false);
    }
    let response = request_started_agent(&encode_info()?)?;
    match response {
        AgentResponse::Info(protocol, implementation, active)
            if agent_is_compatible(protocol, &implementation) =>
        {
            Ok(active)
        }
        _ => Err(io::Error::other(
            "Session Agent capability response unavailable or incompatible",
        )),
    }
}

pub(crate) fn get(lockbox_id: LockboxId) -> io::Result<Option<SecretVec>> {
    get_named(&lockbox_id.to_string())
}

pub(crate) fn get_named(identifier: &str) -> io::Result<Option<SecretVec>> {
    if !existing_agent_is_reachable()? {
        return Ok(None);
    }
    match request(&encode_get_identifier(identifier)?)? {
        AgentResponse::Key(key) => Ok(Some(key)),
        AgentResponse::Miss => Ok(None),
        response => invalid_agent_response(response),
    }
}

pub(crate) fn put(
    lockbox_id: LockboxId,
    key: &SecretVec,
    path: Option<&str>,
    ttl_seconds: Option<u64>,
) -> io::Result<()> {
    expect_ok(request(&encode_put_identifier(
        &lockbox_id.to_string(),
        key,
        path,
        ttl_seconds,
    )?)?)
}

pub(crate) fn put_named(
    identifier: &str,
    key: &SecretVec,
    path: Option<&str>,
    ttl_seconds: Option<u64>,
) -> io::Result<()> {
    if !existing_agent_is_reachable()? {
        return Ok(());
    }
    expect_ok(request(&encode_put_identifier(
        identifier,
        key,
        path,
        ttl_seconds,
    )?)?)
}

pub(crate) fn forget(lockbox_id: LockboxId) -> io::Result<()> {
    forget_named(&lockbox_id.to_string())
}

pub(crate) fn forget_named(identifier: &str) -> io::Result<()> {
    if !existing_agent_is_reachable()? {
        return Ok(());
    }
    expect_ok(request(&encode_forget_identifier(identifier)?)?)
}

pub(crate) fn forget_all() -> io::Result<()> {
    if !existing_agent_is_reachable()? {
        return Ok(());
    }
    expect_ok(request(&encode_forget_all()?)?)
}

pub(crate) fn stop() -> io::Result<()> {
    if !existing_agent_is_reachable()? {
        return Ok(());
    }
    match request_started_agent(&encode_stop()?) {
        Ok(response) => expect_ok(response),
        Err(error) if stop_disconnect(error.kind()) => {
            let deadline = Instant::now() + Duration::from_secs(1);
            while Instant::now() < deadline {
                if !existing_agent_is_reachable()? {
                    return Ok(());
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(error)
        }
        Err(error) => Err(error),
    }
}

fn stop_disconnect(kind: io::ErrorKind) -> bool {
    matches!(
        kind,
        io::ErrorKind::BrokenPipe
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::NotConnected
            | io::ErrorKind::UnexpectedEof
    )
}

pub(crate) fn list() -> io::Result<Vec<CachedLockbox>> {
    if !existing_agent_is_reachable()? {
        return Ok(Vec::new());
    }
    match request(&encode_list()?)? {
        AgentResponse::List(ids) => Ok(ids),
        response => invalid_agent_response(response),
    }
}

pub(crate) fn register_secret_activity(kind: SecretActivityKind) -> io::Result<(u64, bool)> {
    match request_control(&encode_register_secret_activity(std::process::id(), kind)?)? {
        ControlResponse::Registered(token, terminate) => Ok((token, terminate)),
        ControlResponse::Err(message) => Err(io::Error::other(message)),
        response => invalid_control_response(response),
    }
}

pub(crate) fn unregister_secret_activity(pid: u32, token: u64) -> io::Result<()> {
    if !existing_agent_is_reachable()? {
        return Ok(());
    }
    expect_control_ok(request_control(&encode_unregister_secret_activity(
        pid, token,
    )?)?)
}

pub(crate) fn is_running() -> bool {
    existing_agent_is_compatible().unwrap_or(false)
}

pub(crate) fn start() -> io::Result<()> {
    ensure_agent()
}

fn existing_agent_is_reachable() -> io::Result<bool> {
    if !socket_dir().exists() {
        return Ok(false);
    }
    validate_socket_dir(&socket_dir())?;
    Ok(connect_socket().is_ok())
}

fn request(message: &SecretVec) -> io::Result<AgentResponse> {
    ensure_agent()?;
    request_started_agent(message)
}

fn request_started_agent(message: &SecretVec) -> io::Result<AgentResponse> {
    let mut stream = DeadlineStream::new(connect_started_agent()?)?;
    message
        .with_bytes(|message| stream.write_all(message))
        .map_err(io::Error::other)??;
    stream
        .stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(|e| transport_context("request write shutdown", e))?;
    parse_response(read_secure_frame(stream, max_message_bytes())?)
}

fn request_control(message: &[u8]) -> io::Result<ControlResponse> {
    ensure_agent()?;
    let mut stream = DeadlineStream::new(connect_started_agent()?)?;
    stream.write_all(message)?;
    stream
        .stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(|e| transport_context("request write shutdown", e))?;
    parse_control_response(&read_plain_frame(stream, max_message_bytes())?)
}

fn connect_started_agent() -> io::Result<UnixStream> {
    connect_socket().map_err(|err| {
        if matches!(
            err.kind(),
            io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
        ) {
            agent_start_timeout_error()
        } else {
            err
        }
    })
}

// Operation names contain no endpoint, credential or message data. Preserve
// ErrorKind because retry/refusal logic uses it; retain the OS error in Display.
fn transport_context(operation: &'static str, error: io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("Session Agent {operation}: {error}"))
}

fn connect_socket() -> io::Result<UnixStream> {
    let address = socket2::SockAddr::unix(socket_path())
        .map_err(|e| transport_context("socket address", e))?;
    let socket = socket2::Socket::new(socket2::Domain::UNIX, socket2::Type::STREAM, None)
        .map_err(|e| transport_context("socket creation", e))?;
    socket
        .connect_timeout(&address, Duration::from_secs(3))
        .map_err(|e| transport_context("connect", e))?;
    let descriptor: std::os::fd::OwnedFd = socket.into();
    Ok(descriptor.into())
}

fn ensure_agent() -> io::Result<()> {
    prepare_socket_dir()?;
    if connect_socket().is_ok() {
        if existing_agent_is_compatible()? {
            return Ok(());
        }
        stop_incompatible_agent()?;
    }
    let exe = env::current_exe()?;
    Command::new(exe)
        .arg("__agent")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if connect_socket().is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(25));
    }
    Err(agent_start_timeout_error())
}

fn existing_agent_is_compatible() -> io::Result<bool> {
    if !existing_agent_is_reachable()? {
        return Ok(false);
    }
    let request = encode_info()?;
    Ok(matches!(
        request_started_agent(&request),
        Ok(AgentResponse::Info(protocol, implementation, _))
            if agent_is_compatible(protocol, &implementation)
    ))
}

fn stop_incompatible_agent() -> io::Result<()> {
    let _ = request_started_agent(&encode_stop()?);
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if !existing_agent_is_reachable()? {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(25));
    }
    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "incompatible Session Agent did not stop",
    ))
}

fn agent_start_timeout_error() -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, "Session Agent did not start")
}

fn handle_client(
    stream: UnixStream,
    cache: &Arc<Mutex<BTreeMap<String, CacheEntry>>>,
    active: &Arc<Mutex<ActiveSecretRegistry>>,
) -> io::Result<bool> {
    if !client_matches_current_user(&stream)? {
        log_agent_event("rejected agent request from a different user or group");
        return Ok(false);
    }
    let peer_pid = peer_credentials(&stream)?.pid;
    let mut stream = DeadlineStream::new(stream)?;
    let request = read_agent_frame(&mut stream, max_message_bytes())?;
    let (stop, response) = match request {
        AgentFrame::Cache(request) => {
            // Transport must never hold the cache lock needed by suspend wiping.
            let (stop, response) = {
                let mut cache = lock_cache(cache)?;
                handle_agent_request(&request, &mut cache)?
            };
            (stop, AgentReply::Cache(response))
        }
        AgentFrame::Control(request) => {
            let response = handle_control_request(&request, active, peer_pid, &stream.stream)?;
            (false, AgentReply::Control(response))
        }
    };
    write_agent_reply(&mut stream, response)?;
    Ok(stop)
}

fn client_matches_current_user(stream: &UnixStream) -> io::Result<bool> {
    let peer = peer_credentials(stream)?;
    Ok(peer.uid == current_effective_uid() && peer.gid == current_effective_gid())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PeerCredentials {
    uid: u32,
    gid: u32,
    pid: Option<u32>,
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn peer_credentials(stream: &UnixStream) -> io::Result<PeerCredentials> {
    use std::os::unix::io::AsRawFd;

    let mut credential = std::mem::MaybeUninit::<libc::ucred>::uninit();
    let mut credential_len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: `stream.as_raw_fd()` is a valid Unix socket descriptor, and the
    // output buffer points to initialized storage large enough for `ucred`.
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            credential.as_mut_ptr().cast::<libc::c_void>(),
            &mut credential_len,
        )
    };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    if credential_len < std::mem::size_of::<libc::ucred>() as libc::socklen_t {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "peer credential response was truncated",
        ));
    }
    // SAFETY: `getsockopt` succeeded and reported a full `ucred` payload.
    let credential = unsafe { credential.assume_init() };
    Ok(PeerCredentials {
        uid: credential.uid,
        gid: credential.gid,
        pid: u32::try_from(credential.pid).ok().filter(|pid| *pid != 0),
    })
}

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
fn peer_credentials(stream: &UnixStream) -> io::Result<PeerCredentials> {
    use std::os::unix::io::AsRawFd;

    let mut euid = 0 as libc::uid_t;
    let mut egid = 0 as libc::gid_t;
    // SAFETY: `stream.as_raw_fd()` is a valid Unix socket descriptor, and the
    // uid/gid output pointers are valid for writes during this call.
    let result = unsafe { libc::getpeereid(stream.as_raw_fd(), &mut euid, &mut egid) };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(PeerCredentials {
        uid: euid as u32,
        gid: egid as u32,
        pid: None,
    })
}

#[cfg(not(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
fn peer_credentials(_stream: &UnixStream) -> io::Result<PeerCredentials> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "authenticated peer credentials unavailable",
    ))
}

fn current_effective_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and returns the current process euid.
    unsafe { libc::geteuid() as u32 }
}

fn current_effective_gid() -> u32 {
    // SAFETY: `getegid` has no preconditions and returns the current process egid.
    unsafe { libc::getegid() as u32 }
}

fn handle_agent_request(
    request: &SecretVec,
    cache: &mut BTreeMap<String, CacheEntry>,
) -> io::Result<(bool, SecretVec)> {
    let mut stop = false;
    let response = match parse_request(request) {
        Ok(AgentRequest::Get(lockbox_id)) => {
            let now = Instant::now();
            match cache.get_mut(&lockbox_id) {
                Some(entry) if entry.expires_at > now => {
                    log_agent_event(format!("cache hit {lockbox_id}"));
                    encode_key_response(&entry.key)?
                }
                _ => {
                    log_agent_event(format!("cache miss {lockbox_id}"));
                    encode_miss_response()?
                }
            }
        }
        Ok(AgentRequest::Put(lockbox_id, key, path, ttl_seconds)) => {
            let ttl_seconds = ttl_seconds.unwrap_or(DEFAULT_TTL_SECONDS);
            let expires_at = Instant::now()
                .checked_add(Duration::from_secs(ttl_seconds))
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "cache TTL exceeds clock range")
                })?;
            log_agent_event(format!(
                "cached lockbox {lockbox_id} ttl_seconds={ttl_seconds} path={}",
                path.as_deref().unwrap_or("")
            ));
            cache.insert(
                lockbox_id.clone(),
                CacheEntry {
                    key,
                    path,
                    expires_at,
                },
            );
            encode_ok_response()?
        }
        Ok(AgentRequest::Forget(lockbox_id)) => {
            cache.remove(&lockbox_id);
            log_agent_event(format!("forgot lockbox {lockbox_id}"));
            encode_ok_response()?
        }
        Ok(AgentRequest::ForgetAll) => {
            let count = cache.len();
            cache.clear();
            log_agent_event(format!("forgot all cached lockboxes count={count}"));
            encode_ok_response()?
        }
        Ok(AgentRequest::Stop) => {
            let count = cache.len();
            cache.clear();
            stop = true;
            log_agent_event(format!("stop requested; cleared {count} cached lockboxes"));
            encode_ok_response()?
        }
        Ok(AgentRequest::List) => {
            let visible = cache
                .iter()
                .filter(|(id, _)| {
                    !id.starts_with("vault-unlock:") && !id.starts_with("owner-signing:")
                })
                .collect::<Vec<_>>();
            log_agent_event(format!("listed cached lockboxes count={}", visible.len()));
            encode_list_response(visible.into_iter().map(|(id, entry)| CachedLockbox {
                id: id.clone(),
                path: entry.path.clone(),
            }))?
        }
        Ok(AgentRequest::Info) => encode_info_response()?,
        Err(_) => encode_err_response("invalid request")?,
    };
    Ok((stop, response))
}

fn handle_control_request(
    request: &[u8],
    active: &Arc<Mutex<ActiveSecretRegistry>>,
    peer_pid: Option<u32>,
    _stream: &UnixStream,
) -> io::Result<Vec<u8>> {
    match parse_control_request(request) {
        Ok(ControlRequest::RegisterSecretActivity(pid, kind)) => {
            if peer_pid.is_some_and(|peer| peer != pid) {
                return encode_control_err_response(
                    "process identity does not match authenticated peer",
                );
            }
            let mut registry = lock_active(active)?;
            let token = match registry.register(pid, kind) {
                Ok(token) => token,
                Err(error) => return encode_control_err_response(&error.to_string()),
            };
            encode_registered_response(token, registry.terminate_on_suspend())
        }
        Ok(ControlRequest::UnregisterSecretActivity(pid, token)) => {
            if peer_pid.is_some_and(|peer| peer != pid) {
                return encode_control_err_response(
                    "process identity does not match authenticated peer",
                );
            }
            lock_active(active)?.unregister(pid, token);
            encode_control_ok_response()
        }
        Err(_) => encode_control_err_response("invalid control request"),
    }
}

fn prune_expired(cache: &mut BTreeMap<String, CacheEntry>) {
    let now = Instant::now();
    cache.retain(|_, entry| entry.expires_at > now);
}

fn log_pruned_expired(cache: &mut BTreeMap<String, CacheEntry>) {
    let before = cache.len();
    prune_expired(cache);
    let pruned = before.saturating_sub(cache.len());
    if pruned != 0 {
        log_agent_event(format!("pruned expired cached lockboxes count={pruned}"));
    }
}

fn expect_ok(response: AgentResponse) -> io::Result<()> {
    match response {
        AgentResponse::Ok => Ok(()),
        AgentResponse::Err(message) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("agent rejected request: {message}"),
        )),
        other => invalid_agent_response(other),
    }
}

fn expect_control_ok(response: ControlResponse) -> io::Result<()> {
    match response {
        ControlResponse::Ok => Ok(()),
        ControlResponse::Err(message) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("agent rejected control request: {message}"),
        )),
        other => invalid_control_response(other),
    }
}

fn invalid_agent_response<T>(response: AgentResponse) -> io::Result<T> {
    let label = match response {
        AgentResponse::Ok => "OK",
        AgentResponse::Miss => "MISS",
        AgentResponse::Key(_) => "KEY",
        AgentResponse::List(_) => "LIST",
        AgentResponse::Info(_, _, _) => "INFO",
        AgentResponse::Err(_) => "ERR",
    };
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("unexpected agent response: {label}"),
    ))
}

fn invalid_control_response<T>(response: ControlResponse) -> io::Result<T> {
    let label = match response {
        ControlResponse::Ok => "OK",
        ControlResponse::Registered(_, _) => "REGISTERED",
        ControlResponse::Err(_) => "ERR",
    };
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("unexpected agent control response: {label}"),
    ))
}

enum AgentFrame {
    Cache(SecretVec),
    Control(Vec<u8>),
}

enum AgentReply {
    Cache(SecretVec),
    Control(Vec<u8>),
}

fn read_agent_frame(mut reader: impl Read, max_bytes: usize) -> io::Result<AgentFrame> {
    let mut header = vec![0u8; frame_header_len()];
    reader.read_exact(&mut header)?;
    let payload_len = frame_payload_len(&header)?;
    let frame_len = checked_frame_len(header.len(), payload_len, max_bytes)?;
    let message_type = frame_message_type(&header)?;
    if is_control_message_type(message_type) {
        let mut frame = header;
        frame.resize(frame_len, 0);
        reader.read_exact(&mut frame[frame_header_len()..])?;
        Ok(AgentFrame::Control(frame))
    } else {
        let mut frame = SecretVec::new();
        frame
            .try_extend_from_slice(&header)
            .map_err(io::Error::other)?;
        read_secure_payload(reader, payload_len, &mut frame)?;
        Ok(AgentFrame::Cache(frame))
    }
}

fn read_secure_frame(mut reader: impl Read, max_bytes: usize) -> io::Result<SecretVec> {
    let mut header = vec![0u8; frame_header_len()];
    reader.read_exact(&mut header)?;
    let payload_len = frame_payload_len(&header)?;
    checked_frame_len(header.len(), payload_len, max_bytes)?;
    let mut frame = SecretVec::new();
    frame
        .try_extend_from_slice(&header)
        .map_err(io::Error::other)?;
    read_secure_payload(reader, payload_len, &mut frame)?;
    Ok(frame)
}

fn read_plain_frame(mut reader: impl Read, max_bytes: usize) -> io::Result<Vec<u8>> {
    let mut header = vec![0u8; frame_header_len()];
    reader.read_exact(&mut header)?;
    let payload_len = frame_payload_len(&header)?;
    let frame_len = checked_frame_len(header.len(), payload_len, max_bytes)?;
    let mut frame = header;
    frame.resize(frame_len, 0);
    reader.read_exact(&mut frame[frame_header_len()..])?;
    Ok(frame)
}

fn checked_frame_len(header_len: usize, payload_len: usize, max_bytes: usize) -> io::Result<usize> {
    let frame_len = header_len
        .checked_add(payload_len)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "message too large"))?;
    if frame_len > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "message too large",
        ));
    }
    Ok(frame_len)
}

fn write_agent_reply(stream: &mut impl Write, response: AgentReply) -> io::Result<()> {
    match response {
        AgentReply::Cache(response) => response
            .with_bytes(|response| stream.write_all(response))
            .map_err(io::Error::other)?,
        AgentReply::Control(response) => stream.write_all(&response),
    }
}

// One deadline covers both directions and every partial read/write. Socket
// timeouts alone would restart the allowance whenever a peer makes progress.
struct DeadlineStream {
    stream: UnixStream,
    deadline: Instant,
}

impl DeadlineStream {
    fn new(stream: UnixStream) -> io::Result<Self> {
        stream
            .set_nonblocking(true)
            .map_err(|e| transport_context("nonblocking mode", e))?;
        Ok(Self {
            stream,
            deadline: Instant::now() + Duration::from_secs(10),
        })
    }

    fn remaining(&self) -> io::Result<Duration> {
        self.deadline
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::TimedOut, "agent request deadline expired")
            })
    }

    fn wait_ready(&self, events: libc::c_short) -> io::Result<()> {
        use std::os::fd::AsRawFd;
        loop {
            let remaining = self.remaining()?;
            // Round upward, then recheck the absolute deadline before I/O.
            let millis =
                remaining.as_millis() + u128::from(remaining.subsec_nanos() % 1_000_000 != 0);
            let timeout = millis.min(libc::c_int::MAX as u128) as libc::c_int;
            let mut descriptor = libc::pollfd {
                fd: self.stream.as_raw_fd(),
                events,
                revents: 0,
            };
            // SAFETY: descriptor is live for this call; poll borrows exactly one
            // initialized pollfd and does not retain its address.
            let result = unsafe { libc::poll(&mut descriptor, 1, timeout) };
            if result < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(transport_context("readiness wait", error));
            }
            if result == 0 {
                continue;
            }
            if descriptor.revents & libc::POLLNVAL != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Session Agent readiness descriptor invalid",
                ));
            }
            // HUP/ERR also permit an I/O attempt: drain buffered replies or
            // obtain the socket's actual EOF/error without changing options.
            return Ok(());
        }
    }
}

impl Read for DeadlineStream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        loop {
            self.remaining()?;
            match self.stream.read(buffer) {
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    self.wait_ready(libc::POLLIN)?
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                result => return result.map_err(|e| transport_context("read", e)),
            }
        }
    }
}

impl Write for DeadlineStream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        loop {
            self.remaining()?;
            match self.stream.write(buffer) {
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    self.wait_ready(libc::POLLOUT)?
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                result => return result.map_err(|e| transport_context("write", e)),
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.remaining()?;
        self.stream.flush()
    }
}

fn read_secure_payload(
    mut reader: impl Read,
    mut remaining: usize,
    out: &mut SecretVec,
) -> io::Result<()> {
    let mut buffer = zeroize::Zeroizing::new([0u8; 4096]);
    while remaining != 0 {
        let read = reader.read(&mut buffer[..remaining.min(4096)])?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "message ended before frame body",
            ));
        }
        out.try_extend_from_slice(&buffer[..read])
            .map_err(io::Error::other)?;
        zeroize::Zeroize::zeroize(&mut buffer[..read]);
        remaining -= read;
    }
    Ok(())
}

fn prepare_socket_dir() -> io::Result<()> {
    let dir = socket_dir();
    match fs::symlink_metadata(&dir) {
        Ok(_) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => fs::create_dir_all(&dir)?,
        Err(err) => return Err(err),
    }
    validate_socket_dir_owner(&dir)?;
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
    validate_socket_dir(&dir)
}

fn validate_socket_dir_owner(dir: &Path) -> io::Result<()> {
    let metadata = socket_dir_metadata(dir)?;
    if metadata.uid() != current_effective_uid() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "Session Agent directory is not owned by the current user: {}",
                dir.display()
            ),
        ));
    }
    Ok(())
}

fn validate_socket_dir(dir: &Path) -> io::Result<()> {
    let metadata = socket_dir_metadata(dir)?;
    if metadata.uid() != current_effective_uid() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "Session Agent directory is not owned by the current user: {}",
                dir.display()
            ),
        ));
    }
    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "Session Agent directory is accessible by other users: {} (mode {mode:o})",
                dir.display()
            ),
        ));
    }
    Ok(())
}

fn socket_dir_metadata(dir: &Path) -> io::Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(dir)?;
    if metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "Session Agent directory must not be a symlink: {}",
                dir.display()
            ),
        ));
    }
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Session Agent directory path is not a directory: {}",
                dir.display()
            ),
        ));
    }
    Ok(metadata)
}

fn remove_stale_socket(socket: &Path) -> io::Result<()> {
    let metadata = match fs::symlink_metadata(socket) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "refusing to remove symlink at Session Agent socket path: {}",
                socket.display()
            ),
        ));
    }
    if !file_type.is_socket() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "Session Agent socket path exists and is not a socket: {}",
                socket.display()
            ),
        ));
    }
    fs::remove_file(socket)
}

fn socket_path() -> PathBuf {
    socket_dir().join("agent.sock")
}

fn socket_dir() -> PathBuf {
    if let Ok(dir) = env::var("LOCKBOX_SESSION_AGENT_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(dir) = env::var("XDG_RUNTIME_DIR") {
        let dir = PathBuf::from(dir);
        if runtime_dir_belongs_to_current_user(&dir) {
            return dir.join("lockbox");
        }
    }
    #[cfg(target_os = "linux")]
    {
        let dir = PathBuf::from(format!("/run/user/{}", current_effective_uid()));
        if dir.is_dir() {
            return dir.join("lockbox");
        }
    }
    env::temp_dir().join(format!("lockbox-{}", current_effective_uid()))
}

pub(crate) fn session_agent_dir() -> PathBuf {
    socket_dir()
}

fn runtime_dir_belongs_to_current_user(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => metadata.uid() == current_effective_uid(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ListenerTestDirectory(PathBuf);
    impl ListenerTestDirectory {
        fn new() -> Self {
            let path = unique_test_dir("listener");
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for ListenerTestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn listener_readiness_accepts_a_connection_without_changing_nonblocking_mode() {
        let directory = ListenerTestDirectory::new();
        let path = directory.path().join("listener.sock");
        let listener = UnixListener::bind(&path).unwrap();
        listener.set_nonblocking(true).unwrap();
        let peer = UnixStream::connect(&path).unwrap();
        wait_for_agent_connection(&listener, Instant::now() + Duration::from_secs(1)).unwrap();
        let (accepted, _) = listener.accept().unwrap();
        drop((accepted, peer));
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
    }

    #[test]
    fn listener_readiness_timeout_returns_for_maintenance() {
        let directory = ListenerTestDirectory::new();
        let listener = UnixListener::bind(directory.path().join("listener.sock")).unwrap();
        listener.set_nonblocking(true).unwrap();
        let deadline = Instant::now() + Duration::from_millis(5);
        wait_for_agent_connection(&listener, deadline).unwrap();
        assert!(Instant::now() >= deadline);
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        wait_for_agent_connection(&listener, Instant::now() - Duration::from_secs(1)).unwrap();
    }

    #[test]
    fn request_deadline_is_shared_by_reads_and_writes() {
        let (left, mut right) = UnixStream::pair().unwrap();
        let mut stream = DeadlineStream::new(left).unwrap();
        right.write_all(b"a").unwrap();
        let mut byte = [0];
        stream.read_exact(&mut byte).unwrap();
        assert_eq!(&byte, b"a");
        right.write_all(b"b").unwrap();
        stream.deadline = Instant::now() - Duration::from_secs(1);
        assert_eq!(
            stream.read(&mut byte).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        assert_eq!(
            stream.write(b"b").unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
    }

    #[test]
    fn buffered_framed_reply_is_read_after_peer_closes() {
        let (left, mut right) = UnixStream::pair().unwrap();
        let response = encode_control_ok_response().unwrap();
        right.write_all(&response).unwrap();
        drop(right);
        let stream = DeadlineStream::new(left).unwrap();
        let frame = read_plain_frame(stream, max_message_bytes()).unwrap();
        assert_eq!(frame, response);
        assert!(matches!(
            parse_control_response(&frame).unwrap(),
            ControlResponse::Ok
        ));
    }

    #[test]
    fn idle_read_obeys_absolute_deadline() {
        let (left, _right) = UnixStream::pair().unwrap();
        let mut stream = DeadlineStream::new(left).unwrap();
        stream.deadline = Instant::now() + Duration::from_millis(20);
        assert_eq!(
            stream.read(&mut [0]).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
    }

    #[test]
    fn stalled_write_obeys_absolute_deadline() {
        let (left, _right) = UnixStream::pair().unwrap();
        let mut stream = DeadlineStream::new(left).unwrap();
        stream.deadline = Instant::now() + Duration::from_millis(20);
        let bytes = [7u8; 4096];
        let error = loop {
            match stream.write_all(&bytes) {
                Ok(()) => continue,
                Err(error) => break error,
            }
        };
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    }

    #[test]
    fn cache_ttl_admission_preserves_existing_entry() {
        let mut cache = BTreeMap::new();
        let id = LockboxId::from_bytes([19; 16]);
        let key = SecretVec::try_from_slice(&[7u8; 32]).unwrap();
        let valid = encode_put(id, &key, None, Some(60)).unwrap();
        handle_agent_request(&valid, &mut cache).unwrap();
        let old_expiry = cache.values().next().unwrap().expires_at;
        let excessive = encode_put(id, &key, None, Some(u64::MAX)).unwrap();
        assert_eq!(
            handle_agent_request(&excessive, &mut cache)
                .err()
                .unwrap()
                .kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.values().next().unwrap().expires_at, old_expiry);
    }
    use crate::agent_protocol::{
        encode_forget, encode_get, encode_put, AGENT_IMPLEMENTATION_VERSION, AGENT_PROTOCOL_VERSION,
    };

    #[test]
    fn stop_accepts_only_transport_disconnect_errors() {
        assert!(stop_disconnect(io::ErrorKind::NotConnected));
        assert!(stop_disconnect(io::ErrorKind::ConnectionReset));
        assert!(stop_disconnect(io::ErrorKind::UnexpectedEof));
        assert!(!stop_disconnect(io::ErrorKind::PermissionDenied));
        assert!(!stop_disconnect(io::ErrorKind::InvalidData));
    }

    #[test]
    fn sleep_watcher_suspend_event_is_detected() {
        let watcher = SleepWatcher::from_events([SleepEvent::Resumed]);
        assert!(!sleep_watcher_has_suspend(&Some(watcher)));

        let watcher = SleepWatcher::from_events([SleepEvent::SuspendRequested]);
        assert!(sleep_watcher_has_suspend(&Some(watcher)));
        assert!(!sleep_watcher_has_suspend(&None));
    }

    #[test]
    fn cached_requests_work_without_sleep_watcher() {
        let mut cache = BTreeMap::<String, CacheEntry>::new();
        let lockbox_id = LockboxId::from_bytes([7; 16]);
        let key = SecretVec::try_from_slice(b"0123456789abcdef0123456789abcdef").unwrap();

        let (stop, response) = round_trip(
            &mut cache,
            encode_put(lockbox_id, &key, Some("/tmp/test.lbox"), Some(60)).unwrap(),
        );
        assert!(!stop);
        assert_ok(response);

        let (stop, response) = round_trip(&mut cache, encode_get(lockbox_id).unwrap());
        assert!(!stop);
        match response {
            AgentResponse::Key(stored_key) => assert_secret_eq(&stored_key, &key),
            _ => panic!("expected cached key response"),
        }

        let (_, response) = round_trip(&mut cache, encode_list().unwrap());
        match response {
            AgentResponse::List(lockboxes) => {
                assert_eq!(lockboxes.len(), 1);
                assert_eq!(lockboxes[0].id, lockbox_id.to_string());
                assert_eq!(lockboxes[0].path.as_deref(), Some("/tmp/test.lbox"));
            }
            _ => panic!("expected list response"),
        }

        let (_, response) = round_trip(&mut cache, encode_info().unwrap());
        assert!(matches!(
            response,
            AgentResponse::Info(AGENT_PROTOCOL_VERSION, ref implementation, _)
                if implementation == AGENT_IMPLEMENTATION_VERSION
        ));

        let (_, response) = round_trip(&mut cache, encode_forget(lockbox_id).unwrap());
        assert_ok(response);

        let (_, response) = round_trip(&mut cache, encode_get(lockbox_id).unwrap());
        assert!(matches!(response, AgentResponse::Miss));

        let (_, response) = round_trip(
            &mut cache,
            encode_put(lockbox_id, &key, Some("/tmp/test.lbox"), Some(60)).unwrap(),
        );
        assert_ok(response);

        let (_, response) = round_trip(&mut cache, encode_forget_all().unwrap());
        assert_ok(response);

        let (stop, response) = round_trip(&mut cache, encode_stop().unwrap());
        assert!(stop);
        assert_ok(response);
    }

    #[test]
    fn typed_named_entries_have_ttl_and_are_hidden_from_archive_listing() {
        let mut cache = BTreeMap::<String, CacheEntry>::new();
        let key = SecretVec::try_from_slice(b"owner-signing-record").unwrap();
        let identifier = "owner-signing:/vault\0default";

        let (_, response) = round_trip(
            &mut cache,
            super::encode_put_identifier(identifier, &key, None, Some(1)).unwrap(),
        );
        assert_ok(response);
        let (_, response) = round_trip(
            &mut cache,
            super::encode_get_identifier(identifier).unwrap(),
        );
        match response {
            AgentResponse::Key(stored) => assert_secret_eq(&stored, &key),
            _ => panic!("expected typed cache hit"),
        }

        let (_, response) = round_trip(&mut cache, encode_list().unwrap());
        assert!(matches!(response, AgentResponse::List(entries) if entries.is_empty()));

        cache
            .get_mut(identifier)
            .expect("typed cache entry")
            .expires_at = Instant::now() - Duration::from_secs(1);
        let (_, response) = round_trip(
            &mut cache,
            super::encode_get_identifier(identifier).unwrap(),
        );
        assert!(matches!(response, AgentResponse::Miss));
    }

    #[test]
    fn suspend_event_clears_cached_lockboxes() {
        let mut cache = BTreeMap::<String, CacheEntry>::new();
        cache.insert(
            LockboxId::from_bytes([9; 16]).to_string(),
            CacheEntry {
                key: SecretVec::try_from_slice(b"cached-key").unwrap(),
                path: Some("/tmp/test.lbox".to_string()),
                expires_at: Instant::now() + Duration::from_secs(60),
            },
        );

        let sleep_watcher = Some(SleepWatcher::from_events([SleepEvent::SuspendRequested]));
        if sleep_watcher_has_suspend(&sleep_watcher) {
            cache.clear();
        }

        assert!(cache.is_empty());
    }

    #[test]
    fn socket_dir_validation_rejects_unsafe_paths() {
        let dir = unique_test_dir("socket-dir");
        fs::create_dir_all(&dir).unwrap();

        fs::set_permissions(&dir, fs::Permissions::from_mode(0o770)).unwrap();
        let err = validate_socket_dir(&dir).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);

        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
        validate_socket_dir(&dir).unwrap();

        let not_dir = dir.join("not-a-dir");
        fs::write(&not_dir, b"not a directory").unwrap();
        let err = validate_socket_dir(&not_dir).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);

        let link = dir.join("link");
        std::os::unix::fs::symlink(&dir, &link).unwrap();
        let err = validate_socket_dir(&link).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn stale_socket_cleanup_rejects_non_socket_paths() {
        let dir = unique_test_dir("socket-path");
        fs::create_dir_all(&dir).unwrap();
        let socket = dir.join("agent.sock");
        fs::write(&socket, b"not a socket").unwrap();

        let err = remove_stale_socket(&socket).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn peer_owner_check_accepts_current_user_socket() {
        let dir = unique_test_dir("peer");
        fs::create_dir_all(&dir).unwrap();
        let socket = dir.join("agent.sock");
        let listener = match UnixListener::bind(&socket) {
            Ok(listener) => listener,
            Err(err) if err.kind() == io::ErrorKind::PermissionDenied => {
                let _ = fs::remove_dir_all(&dir);
                return;
            }
            Err(err) => panic!("unable to bind local agent test socket: {err}"),
        };
        let client = match UnixStream::connect(&socket) {
            Ok(client) => client,
            Err(err) if err.kind() == io::ErrorKind::PermissionDenied => {
                let _ = fs::remove_dir_all(&dir);
                return;
            }
            Err(err) => panic!("unable to connect local agent test socket: {err}"),
        };
        let (server, _) = listener.accept().unwrap();

        assert_eq!(
            peer_credentials(&server).unwrap(),
            PeerCredentials {
                uid: current_effective_uid(),
                gid: current_effective_gid(),
                pid: if cfg!(any(target_os = "linux", target_os = "android")) {
                    Some(std::process::id())
                } else {
                    None
                },
            }
        );
        assert!(client_matches_current_user(&server).unwrap());

        drop(client);
        let _ = fs::remove_dir_all(dir);
    }

    fn unique_test_dir(label: &str) -> PathBuf {
        PathBuf::from("/tmp").join(format!(
            "lockbox-agent-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn round_trip(
        cache: &mut BTreeMap<String, CacheEntry>,
        request: SecretVec,
    ) -> (bool, AgentResponse) {
        let (stop, response) = handle_agent_request(&request, cache).unwrap();
        (stop, parse_response(response).unwrap())
    }

    fn assert_ok(response: AgentResponse) {
        match response {
            AgentResponse::Ok => {}
            _ => panic!("expected OK response"),
        }
    }

    fn assert_secret_eq(left: &SecretVec, right: &SecretVec) {
        left.with_bytes(|left| {
            right.with_bytes(|right| {
                assert_eq!(left, right);
            })
        })
        .unwrap()
        .unwrap();
    }
}
