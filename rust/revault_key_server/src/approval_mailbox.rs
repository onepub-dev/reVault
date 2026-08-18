use revault_approval_protocol::{MAX_ENVELOPE_BYTES, MAX_REQUEST_LIFETIME_MS};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

const MAX_PENDING_PER_SOURCE: usize = 3;
const MAX_PENDING_PER_DEVICE: usize = 10;
const REQUESTS_PER_MINUTE: usize = 6;
const REQUESTS_PER_HOUR: usize = 30;
const PUSHES_PER_MINUTE: usize = 3;
const PUSHES_PER_HOUR: usize = 20;

type CapabilityHash = [u8; 32];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MailboxError {
    Invalid,
    TooLarge,
    Conflict,
    NotFound,
    NotReady,
    Gone,
    RateLimited,
    Capacity,
}

#[derive(Debug)]
pub(crate) struct PendingRequest {
    pub(crate) id: String,
    pub(crate) envelope: Vec<u8>,
}

#[derive(Debug)]
struct RequestRecord {
    id: String,
    device: CapabilityHash,
    source: CapabilityHash,
    reply: CapabilityHash,
    request: Vec<u8>,
    response: Option<Vec<u8>>,
    created_at_unix_ms: u64,
    expires_at_unix_ms: u64,
    consumed: bool,
}

#[derive(Default)]
struct MailboxState {
    requests: HashMap<String, RequestRecord>,
    request_windows: HashMap<CapabilityHash, VecDeque<u64>>,
    push_windows: HashMap<CapabilityHash, VecDeque<u64>>,
}

#[derive(Default)]
pub(crate) struct ApprovalMailboxStore {
    state: Mutex<MailboxState>,
}

impl ApprovalMailboxStore {
    pub(crate) fn upload_request(
        &self,
        id: &str,
        mailbox_capability: &str,
        source_capability: &str,
        reply_capability: &str,
        envelope: &[u8],
        now_unix_ms: u64,
    ) -> Result<(), MailboxError> {
        validate_id(id)?;
        validate_capability(mailbox_capability)?;
        validate_capability(source_capability)?;
        validate_capability(reply_capability)?;
        if envelope.is_empty() || envelope.len() > MAX_ENVELOPE_BYTES {
            return Err(MailboxError::TooLarge);
        }
        let device = capability_hash(mailbox_capability);
        let source = capability_hash(source_capability);
        let reply = capability_hash(reply_capability);
        let mut state = self.state.lock().map_err(|_| MailboxError::NotReady)?;
        purge(&mut state, now_unix_ms);
        enforce_window(
            state.request_windows.entry(source).or_default(),
            now_unix_ms,
            REQUESTS_PER_MINUTE,
            60_000,
            REQUESTS_PER_HOUR,
            3_600_000,
        )?;
        if state.requests.contains_key(id) {
            return Err(MailboxError::Conflict);
        }
        let source_pending = state
            .requests
            .values()
            .filter(|record| record.source == source && !record.consumed)
            .count();
        let device_pending = state
            .requests
            .values()
            .filter(|record| record.device == device && !record.consumed)
            .count();
        if source_pending >= MAX_PENDING_PER_SOURCE || device_pending >= MAX_PENDING_PER_DEVICE {
            return Err(MailboxError::Capacity);
        }
        state.requests.insert(
            id.to_string(),
            RequestRecord {
                id: id.to_string(),
                device,
                source,
                reply,
                request: envelope.to_vec(),
                response: None,
                created_at_unix_ms: now_unix_ms,
                expires_at_unix_ms: now_unix_ms.saturating_add(MAX_REQUEST_LIFETIME_MS),
                consumed: false,
            },
        );
        Ok(())
    }

    pub(crate) fn poll_request(
        &self,
        mailbox_capability: &str,
        now_unix_ms: u64,
    ) -> Result<PendingRequest, MailboxError> {
        validate_capability(mailbox_capability)?;
        let device = capability_hash(mailbox_capability);
        let mut state = self.state.lock().map_err(|_| MailboxError::NotReady)?;
        purge(&mut state, now_unix_ms);
        state
            .requests
            .values()
            .filter(|record| {
                record.device == device && record.response.is_none() && !record.consumed
            })
            .min_by_key(|record| record.created_at_unix_ms)
            .map(|record| PendingRequest {
                id: record.id.clone(),
                envelope: record.request.clone(),
            })
            .ok_or(MailboxError::NotFound)
    }

    pub(crate) fn upload_response(
        &self,
        id: &str,
        mailbox_capability: &str,
        envelope: &[u8],
        now_unix_ms: u64,
    ) -> Result<(), MailboxError> {
        validate_id(id)?;
        validate_capability(mailbox_capability)?;
        if envelope.is_empty() || envelope.len() > MAX_ENVELOPE_BYTES {
            return Err(MailboxError::TooLarge);
        }
        let device = capability_hash(mailbox_capability);
        let mut state = self.state.lock().map_err(|_| MailboxError::NotReady)?;
        purge(&mut state, now_unix_ms);
        let record = state.requests.get_mut(id).ok_or(MailboxError::NotFound)?;
        if record.device != device {
            return Err(MailboxError::NotFound);
        }
        if record.consumed || record.response.is_some() {
            return Err(MailboxError::Conflict);
        }
        record.response = Some(envelope.to_vec());
        record.request.clear();
        Ok(())
    }

    pub(crate) fn consume_response(
        &self,
        id: &str,
        reply_capability: &str,
        now_unix_ms: u64,
    ) -> Result<Vec<u8>, MailboxError> {
        validate_id(id)?;
        validate_capability(reply_capability)?;
        let reply = capability_hash(reply_capability);
        let mut state = self.state.lock().map_err(|_| MailboxError::NotReady)?;
        purge(&mut state, now_unix_ms);
        let record = state.requests.get_mut(id).ok_or(MailboxError::NotFound)?;
        if record.reply != reply {
            return Err(MailboxError::NotFound);
        }
        if record.consumed {
            return Err(MailboxError::Gone);
        }
        let response = record.response.take().ok_or(MailboxError::NotReady)?;
        record.consumed = true;
        Ok(response)
    }

    pub(crate) fn note_push(
        &self,
        mailbox_capability: &str,
        now_unix_ms: u64,
    ) -> Result<(), MailboxError> {
        validate_capability(mailbox_capability)?;
        let device = capability_hash(mailbox_capability);
        let mut state = self.state.lock().map_err(|_| MailboxError::NotReady)?;
        enforce_window(
            state.push_windows.entry(device).or_default(),
            now_unix_ms,
            PUSHES_PER_MINUTE,
            60_000,
            PUSHES_PER_HOUR,
            3_600_000,
        )
    }
}

fn purge(state: &mut MailboxState, now_unix_ms: u64) {
    state
        .requests
        .retain(|_, record| record.expires_at_unix_ms > now_unix_ms);
    state.request_windows.retain(|_, entries| {
        trim_window(entries, now_unix_ms.saturating_sub(3_600_000));
        !entries.is_empty()
    });
    state.push_windows.retain(|_, entries| {
        trim_window(entries, now_unix_ms.saturating_sub(3_600_000));
        !entries.is_empty()
    });
}

fn enforce_window(
    entries: &mut VecDeque<u64>,
    now_unix_ms: u64,
    short_limit: usize,
    short_window_ms: u64,
    long_limit: usize,
    long_window_ms: u64,
) -> Result<(), MailboxError> {
    trim_window(entries, now_unix_ms.saturating_sub(long_window_ms));
    let short_count = entries
        .iter()
        .filter(|time| **time > now_unix_ms.saturating_sub(short_window_ms))
        .count();
    if short_count >= short_limit || entries.len() >= long_limit {
        return Err(MailboxError::RateLimited);
    }
    entries.push_back(now_unix_ms);
    Ok(())
}

fn trim_window(entries: &mut VecDeque<u64>, earliest: u64) {
    while entries.front().is_some_and(|time| *time <= earliest) {
        entries.pop_front();
    }
}

fn validate_id(value: &str) -> Result<(), MailboxError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(MailboxError::Invalid);
    }
    Ok(())
}

fn validate_capability(value: &str) -> Result<(), MailboxError> {
    if !(32..=256).contains(&value.len()) || value.chars().any(char::is_whitespace) {
        return Err(MailboxError::Invalid);
    }
    Ok(())
}

fn capability_hash(value: &str) -> CapabilityHash {
    Sha256::digest(value.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEVICE: &str = "device-capability-0000000000000000";
    const SOURCE: &str = "source-capability-0000000000000000";
    const REPLY: &str = "reply-capability-00000000000000000";
    const ID: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn response_is_consumed_exactly_once() {
        let store = ApprovalMailboxStore::default();
        store
            .upload_request(ID, DEVICE, SOURCE, REPLY, b"request", 1_000)
            .unwrap();
        assert_eq!(store.poll_request(DEVICE, 2_000).unwrap().envelope, b"request");
        store.upload_response(ID, DEVICE, b"response", 3_000).unwrap();
        assert_eq!(
            store.consume_response(ID, REPLY, 4_000).unwrap(),
            b"response"
        );
        assert_eq!(
            store.consume_response(ID, REPLY, 5_000),
            Err(MailboxError::Gone)
        );
    }

    #[test]
    fn source_pending_limit_is_enforced() {
        let store = ApprovalMailboxStore::default();
        for index in 0..MAX_PENDING_PER_SOURCE {
            let id = format!("{index:064x}");
            store
                .upload_request(&id, DEVICE, SOURCE, &format!("{REPLY}{index}"), b"x", 1_000)
                .unwrap();
        }
        assert_eq!(
            store.upload_request(&format!("{:064x}", 99), DEVICE, SOURCE, REPLY, b"x", 1_000),
            Err(MailboxError::Capacity)
        );
    }
}
