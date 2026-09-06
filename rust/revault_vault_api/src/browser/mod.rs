//! Experimental native browser authorisation, separate from unrestricted local IPC.
//! Pairings are local-only. Every request needs a new trusted native approval.
mod approval;
mod pairing;
mod service;
pub use pairing::{list_pairings, pair, revoke, state_directory, Pairing};
pub use service::request;
pub(crate) use service::start;

fn now() -> Result<u64, revault_browser_protocol::Error> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| revault_browser_protocol::Error::Expired)
}
pub(crate) use service::profile_key;
pub(crate) use service::validate_cached_key;
pub(crate) fn suspend() {
    service::EPOCH.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}
