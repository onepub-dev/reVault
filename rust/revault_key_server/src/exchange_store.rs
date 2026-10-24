//! Durable single-server invitations. Active records are never LRU-evicted.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use revault_publish_protocol::exchange::{
    decode, encode, token_hash, valid_token, Record, Request, Response, MAX_MESSAGE_BYTES,
};

/// Capacity limits apply to admitted invitations, not trusted local contacts.
#[derive(Clone, Copy)]
pub struct ExchangeLimits {
    /// Maximum retained invitations.
    pub invitations: usize,
    /// Maximum serialized payload bytes on disk.
    pub bytes: usize,
    /// Maximum pending invitations per signing identity.
    pub per_identity: usize,
}

impl Default for ExchangeLimits {
    fn default() -> Self {
        Self {
            invitations: 1_000_000,
            bytes: 1024 * 1024 * 1024,
            per_identity: 100,
        }
    }
}

struct IndexEntry {
    expiry: u64,
    identity: String,
}

/// Terminal state retains only capability and transcript hashes until expiry.
/// This makes retries idempotent without retaining public identity payloads.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    id: String,
    expiry: u64,
    identity: String,
    owner_hash: String,
    recipient_hash: Option<String>,
    offer_hash: String,
    acceptance_hash: Option<String>,
    cancelled: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum Stored {
    Active(Box<Record>),
    Receipt(Receipt),
}

fn digest<T: Serialize>(value: &T) -> std::result::Result<String, String> {
    let bytes = encode(value).map_err(|_| "invalid exchange state")?;
    Ok(token_hash(&revault_publish_protocol::exchange::hex(&bytes)))
}

impl Stored {
    fn id(&self) -> &str {
        match self {
            Self::Active(r) => &r.offer.id,
            Self::Receipt(r) => &r.id,
        }
    }
    fn expiry(&self) -> u64 {
        match self {
            Self::Active(r) => r.offer.expires_ms,
            Self::Receipt(r) => r.expiry,
        }
    }
    fn identity(&self) -> String {
        match self {
            Self::Active(r) => token_hash(&revault_publish_protocol::exchange::hex(
                &r.offer.inviter.signing_key,
            )),
            Self::Receipt(r) => r.identity.clone(),
        }
    }
}

impl Receipt {
    fn from_record(record: &Record, cancelled: bool) -> std::result::Result<Self, String> {
        Ok(Self {
            id: record.offer.id.clone(),
            expiry: record.offer.expires_ms,
            identity: Stored::Active(Box::new(record.clone())).identity(),
            owner_hash: record.owner_hash.clone(),
            recipient_hash: record.recipient_hash.clone(),
            offer_hash: digest(&record.offer)?,
            acceptance_hash: record.acceptance.as_ref().map(digest).transpose()?,
            cancelled,
        })
    }

    fn reply(&self, request: &Request) -> std::result::Result<Response, String> {
        let authorized = match request {
            Request::Create { offer, owner_token } => {
                !self.cancelled
                    && valid_token(owner_token)
                    && token_hash(owner_token) == self.owner_hash
                    && digest(offer)? == self.offer_hash
            }
            Request::Accept {
                acceptance,
                recipient_token,
                ..
            } => {
                !self.cancelled
                    && valid_token(recipient_token)
                    && self.recipient_hash.as_ref() == Some(&token_hash(recipient_token))
                    && self.acceptance_hash.as_ref() == Some(&digest(acceptance)?)
            }
            Request::Complete { token, .. } | Request::Poll { token, .. } => {
                !self.cancelled
                    && valid_token(token)
                    && (token_hash(token) == self.owner_hash
                        || self.recipient_hash.as_ref() == Some(&token_hash(token)))
            }
            Request::Cancel { owner_token, .. } => {
                self.cancelled
                    && valid_token(owner_token)
                    && token_hash(owner_token) == self.owner_hash
            }
            Request::Inspect { .. } => false,
        };
        if !authorized {
            return Err("invitation unavailable or expired".to_owned());
        }
        Ok(Response {
            offer: None,
            acceptance: None,
            complete: true,
            error: None,
        })
    }
}

#[derive(Default)]
struct Index {
    records: BTreeMap<String, IndexEntry>,
    expiries: BTreeMap<(u64, String), ()>,
    identities: BTreeMap<String, usize>,
}

impl Index {
    fn insert(&mut self, id: String, entry: IndexEntry) {
        self.expiries.insert((entry.expiry, id.clone()), ());
        *self.identities.entry(entry.identity.clone()).or_default() += 1;
        self.records.insert(id, entry);
    }

    fn remove(&mut self, id: &str) {
        if let Some(entry) = self.records.remove(id) {
            self.expiries.remove(&(entry.expiry, id.to_owned()));
            if let Some(count) = self.identities.get_mut(&entry.identity) {
                *count -= 1;
                if *count == 0 {
                    self.identities.remove(&entry.identity);
                }
            }
        }
    }

    fn len(&self) -> usize {
        self.records.len()
    }
    fn contains_key(&self, id: &str) -> bool {
        self.records.contains_key(id)
    }
}

/// Persistent invitation relay. Payloads are read from disk on demand.
pub struct ExchangeStore {
    directory: PathBuf,
    limits: ExchangeLimits,
    index: Mutex<Index>,
}

impl ExchangeStore {
    /// Returns durable state for authenticated server-to-server replication.
    pub fn snapshot(&self, id: &str) -> std::result::Result<Vec<u8>, String> {
        if !valid_token(id) {
            return Err("invalid invitation ID".to_owned());
        }
        let _index = self.index.lock().map_err(|_| "storage unavailable")?;
        encode(&self.read(id)?).map_err(|_| "invalid invitation state".to_owned())
    }

    /// Returns a consistent inventory for explicit peer resynchronization.
    pub fn snapshot_ids(&self) -> std::result::Result<Vec<String>, String> {
        let index = self.index.lock().map_err(|_| "storage unavailable")?;
        Ok(index.records.keys().cloned().collect())
    }

    /// Applies an authenticated replica snapshot monotonically. A receipt cannot
    /// be replaced by an earlier payload, and a pinned acceptance cannot change.
    pub fn apply_snapshot(&self, bytes: &[u8], now: u64) -> std::result::Result<(), String> {
        let mut incoming: Stored = decode(bytes).map_err(|_| "invalid exchange snapshot")?;
        if !valid_token(incoming.id())
            || incoming.expiry()
                > now.saturating_add(revault_publish_protocol::exchange::MAX_LIFETIME_MS + 60_000)
        {
            return Err("invalid exchange snapshot".to_owned());
        }
        if incoming.expiry() <= now {
            return Ok(());
        }
        match &incoming {
            Stored::Active(record) => {
                record.offer.validate(now).map_err(|e| e.to_string())?;
                if !valid_token(&record.owner_hash)
                    || record
                        .recipient_hash
                        .as_ref()
                        .is_some_and(|hash| !valid_token(hash))
                    || record.acceptance.is_some() != record.recipient_hash.is_some()
                    || ((record.owner_done || record.recipient_done) && record.acceptance.is_none())
                {
                    return Err("invalid exchange state".to_owned());
                }
                if let Some(acceptance) = &record.acceptance {
                    acceptance
                        .validate(&record.offer)
                        .map_err(|e| e.to_string())?;
                }
            }
            Stored::Receipt(receipt) => {
                if !valid_token(&receipt.owner_hash)
                    || !valid_token(&receipt.offer_hash)
                    || !valid_token(&receipt.identity)
                    || receipt
                        .recipient_hash
                        .as_ref()
                        .is_some_and(|v| !valid_token(v))
                    || receipt
                        .acceptance_hash
                        .as_ref()
                        .is_some_and(|v| !valid_token(v))
                    || (receipt.cancelled
                        && (receipt.recipient_hash.is_some() || receipt.acceptance_hash.is_some()))
                    || (!receipt.cancelled
                        && (receipt.recipient_hash.is_none() || receipt.acceptance_hash.is_none()))
                {
                    return Err("invalid exchange receipt".to_owned());
                }
            }
        }
        let mut index = self.index.lock().map_err(|_| "storage unavailable")?;
        self.purge(&mut index, now)?;
        if index.contains_key(incoming.id()) {
            let previous = self.read(incoming.id())?;
            if previous.expiry() != incoming.expiry() || previous.identity() != incoming.identity()
            {
                return Err("conflicting exchange snapshot".to_owned());
            }
            match (&previous, &mut incoming) {
                (Stored::Receipt(old), Stored::Receipt(new)) => {
                    if digest(old)? != digest(new)? {
                        return Err("conflicting exchange receipt".to_owned());
                    }
                    return Ok(());
                }
                (Stored::Receipt(_), Stored::Active(_)) => return Ok(()),
                (Stored::Active(old), Stored::Receipt(new)) => {
                    if new.offer_hash != digest(&old.offer)?
                        || new.owner_hash != old.owner_hash
                        || (old.acceptance.is_some()
                            && (new.cancelled
                                || new.acceptance_hash
                                    != old.acceptance.as_ref().map(digest).transpose()?
                                || new.recipient_hash != old.recipient_hash))
                    {
                        return Err("conflicting exchange receipt".to_owned());
                    }
                }
                (Stored::Active(old), Stored::Active(new)) => {
                    if old.offer != new.offer
                        || old.owner_hash != new.owner_hash
                        || (old.acceptance.is_some()
                            && new.acceptance.is_some()
                            && (old.acceptance != new.acceptance
                                || old.recipient_hash != new.recipient_hash))
                    {
                        return Err("conflicting exchange snapshot".to_owned());
                    }
                    if new.acceptance.is_none() {
                        new.acceptance = old.acceptance.clone();
                        new.recipient_hash = old.recipient_hash.clone();
                    }
                    new.owner_done |= old.owner_done;
                    new.recipient_done |= old.recipient_done;
                }
            }
        } else if index.len() >= self.limits.invitations
            || index
                .len()
                .saturating_add(1)
                .saturating_mul(MAX_MESSAGE_BYTES)
                > self.limits.bytes
        {
            return Err("replica invitation capacity exhausted".to_owned());
        }
        self.write_stored(&incoming)?;
        index.remove(incoming.id());
        index.insert(
            incoming.id().to_owned(),
            IndexEntry {
                expiry: incoming.expiry(),
                identity: incoming.identity(),
            },
        );
        Ok(())
    }

    /// Opens a store and checks all retained records. Corruption fails startup.
    pub fn open(directory: &Path, limits: ExchangeLimits, now: u64) -> std::io::Result<Self> {
        std::fs::create_dir_all(directory)?;
        let mut index = Index::default();
        for item in std::fs::read_dir(directory)? {
            let item = item?;
            let path = item.path();
            if path.extension().and_then(|v| v.to_str()) != Some("json") {
                continue;
            }
            if item.metadata()?.len() > MAX_MESSAGE_BYTES as u64 {
                return Err(std::io::Error::other("oversized invitation record"));
            }
            let bytes = std::fs::read(&path)?;
            let record: Stored = decode(&bytes).map_err(std::io::Error::other)?;
            let id = record.id();
            if !valid_token(id) || path.file_stem().and_then(|v| v.to_str()) != Some(id) {
                return Err(std::io::Error::other("invalid invitation record path"));
            }
            if record.expiry() <= now {
                std::fs::remove_file(path)?;
                continue;
            }
            if let Stored::Active(active) = &record {
                active.offer.validate(now).map_err(std::io::Error::other)?;
                if let Some(acceptance) = &active.acceptance {
                    acceptance
                        .validate(&active.offer)
                        .map_err(std::io::Error::other)?;
                }
            }
            index.insert(
                id.to_owned(),
                IndexEntry {
                    expiry: record.expiry(),
                    identity: record.identity(),
                },
            );
        }
        Ok(Self {
            directory: directory.to_owned(),
            limits,
            index: Mutex::new(index),
        })
    }

    fn path(&self, id: &str) -> PathBuf {
        self.directory.join(format!("{id}.json"))
    }

    fn read(&self, id: &str) -> std::result::Result<Stored, String> {
        if std::fs::metadata(self.path(id))
            .map_err(|_| "invitation unavailable")?
            .len()
            > MAX_MESSAGE_BYTES as u64
        {
            return Err("oversized invitation record".to_owned());
        }
        let bytes = std::fs::read(self.path(id)).map_err(|_| "invitation unavailable")?;
        decode(&bytes).map_err(|_| "invalid stored invitation".to_owned())
    }

    fn write(&self, record: &Record) -> std::result::Result<(), String> {
        self.write_stored(&Stored::Active(Box::new(record.clone())))
    }

    fn write_stored(&self, record: &Stored) -> std::result::Result<(), String> {
        let bytes = encode(record).map_err(|_| "invitation too large")?;
        let mut file =
            tempfile::NamedTempFile::new_in(&self.directory).map_err(|_| "storage unavailable")?;
        file.write_all(&bytes)
            .and_then(|()| file.as_file().sync_all())
            .map_err(|_| "storage write failed")?;
        file.persist(self.path(record.id()))
            .map_err(|_| "storage commit failed")?;
        #[cfg(unix)]
        std::fs::File::open(&self.directory)
            .and_then(|dir| dir.sync_all())
            .map_err(|_| "storage sync failed")?;
        Ok(())
    }

    /// Processes a request atomically, including retries and capability checks.
    pub fn handle(&self, request: Request, now: u64) -> Response {
        match self.apply(request, now) {
            Ok(response) => response,
            Err(error) => Response {
                offer: None,
                acceptance: None,
                complete: false,
                error: Some(error),
            },
        }
    }

    fn apply(&self, request: Request, now: u64) -> std::result::Result<Response, String> {
        let mut index = self.index.lock().map_err(|_| "storage unavailable")?;
        self.purge(&mut index, now)?;
        let request_id = match &request {
            Request::Create { offer, .. } => &offer.id,
            Request::Inspect { id }
            | Request::Accept { id, .. }
            | Request::Poll { id, .. }
            | Request::Complete { id, .. }
            | Request::Cancel { id, .. } => id,
        };
        if !valid_token(request_id) {
            return Err("invalid invitation ID".to_owned());
        }
        if index.contains_key(request_id) {
            if let Stored::Receipt(receipt) = self.read(request_id)? {
                return receipt.reply(&request);
            }
        }
        if let Request::Create { offer, owner_token } = request {
            offer.validate(now).map_err(|e| e.to_string())?;
            if !valid_token(&owner_token) {
                return Err("invalid capability".to_owned());
            }
            if index.contains_key(&offer.id) {
                let Stored::Active(record) = self.read(&offer.id)? else {
                    unreachable!()
                };
                if record.is_owner(&owner_token) && record.offer == offer {
                    return Ok(reply(&record, true));
                }
                return Err("invitation already exists".to_owned());
            }
            // Reserve a complete record's maximum size at admission so accepting
            // an invitation cannot later fail due to other invitations filling it.
            if index.len() >= self.limits.invitations
                || index
                    .len()
                    .saturating_add(1)
                    .saturating_mul(MAX_MESSAGE_BYTES)
                    > self.limits.bytes
            {
                return Err("invitation capacity exhausted; retry later".to_owned());
            }
            let identity = token_hash(&revault_publish_protocol::exchange::hex(
                &offer.inviter.signing_key,
            ));
            if index.identities.get(&identity).copied().unwrap_or(0) >= self.limits.per_identity {
                return Err("too many invitations for this identity".to_owned());
            }
            let record = Record {
                offer,
                owner_hash: token_hash(&owner_token),
                recipient_hash: None,
                acceptance: None,
                owner_done: false,
                recipient_done: false,
            };
            self.write(&record)?;
            index.insert(
                record.offer.id.clone(),
                IndexEntry {
                    expiry: record.offer.expires_ms,
                    identity,
                },
            );
            return Ok(reply(&record, true));
        }
        let id = match &request {
            Request::Inspect { id }
            | Request::Accept { id, .. }
            | Request::Poll { id, .. }
            | Request::Complete { id, .. }
            | Request::Cancel { id, .. } => id,
            Request::Create { .. } => return Err("invalid operation".to_owned()),
        };
        if !valid_token(id) || !index.contains_key(id) {
            return Err("invitation unavailable or expired".to_owned());
        }
        let Stored::Active(mut record) = self.read(id)? else {
            unreachable!()
        };
        match request {
            Request::Inspect { .. } => Ok(reply(&record, false)),
            Request::Accept {
                recipient_token,
                acceptance,
                ..
            } => {
                if !valid_token(&recipient_token) {
                    return Err("invalid capability".to_owned());
                }
                acceptance
                    .validate(&record.offer)
                    .map_err(|e| e.to_string())?;
                if let Some(existing) = &record.acceptance {
                    if record.is_recipient(&recipient_token) && existing == &acceptance {
                        return Ok(reply(&record, true));
                    }
                    return Err("invitation already accepted".to_owned());
                }
                record.acceptance = Some(acceptance);
                record.recipient_hash = Some(token_hash(&recipient_token));
                self.write(&record)?;
                Ok(reply(&record, true))
            }
            Request::Poll { token, .. } => {
                if !record.is_owner(&token) && !record.is_recipient(&token) {
                    return Err("invalid capability".to_owned());
                }
                Ok(reply(&record, true))
            }
            Request::Complete { token, .. } => {
                if record.acceptance.is_none() {
                    return Err("invitation has not been accepted".to_owned());
                }
                if record.is_owner(&token) {
                    record.owner_done = true;
                } else if record.is_recipient(&token) {
                    record.recipient_done = true;
                } else {
                    return Err("invalid capability".to_owned());
                }
                if record.owner_done && record.recipient_done {
                    let receipt = Receipt::from_record(&record, false)?;
                    self.write_stored(&Stored::Receipt(receipt))?;
                    Ok(Response {
                        offer: None,
                        acceptance: None,
                        complete: true,
                        error: None,
                    })
                } else {
                    self.write(&record)?;
                    Ok(reply(&record, true))
                }
            }
            Request::Cancel { owner_token, .. } => {
                if !record.is_owner(&owner_token) {
                    return Err("invalid capability".to_owned());
                }
                if record.acceptance.is_some() {
                    return Err("accepted invitations cannot be cancelled".to_owned());
                }
                self.write_stored(&Stored::Receipt(Receipt::from_record(&record, true)?))?;
                Ok(Response {
                    offer: None,
                    acceptance: None,
                    complete: true,
                    error: None,
                })
            }
            Request::Create { .. } => Err("invalid operation".to_owned()),
        }
    }

    /// Removes expired records without scanning every retained invitation.
    pub fn purge_expired(&self, now: u64) -> std::io::Result<()> {
        let mut index = self
            .index
            .lock()
            .map_err(|_| std::io::Error::other("storage unavailable"))?;
        self.purge(&mut index, now).map_err(std::io::Error::other)
    }

    fn purge(&self, index: &mut Index, now: u64) -> std::result::Result<(), String> {
        while let Some(((expiry, id), ())) = index.expiries.first_key_value() {
            if *expiry > now {
                break;
            }
            let id = id.clone();
            match std::fs::remove_file(self.path(&id)) {
                Ok(()) => (),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                Err(_) => return Err("expiry cleanup failed".to_owned()),
            }
            index.remove(&id);
        }
        Ok(())
    }
}

fn reply(record: &Record, private: bool) -> Response {
    Response {
        offer: Some(record.offer.clone()),
        acceptance: private.then(|| record.acceptance.clone()).flatten(),
        complete: record.owner_done && record.recipient_done,
        error: None,
    }
}
