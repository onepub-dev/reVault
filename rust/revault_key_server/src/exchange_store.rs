//! Durable single-server invitations. Active records are never LRU-evicted.
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
            let record: Record = decode(&bytes).map_err(std::io::Error::other)?;
            let id = &record.offer.id;
            if !valid_token(id) || path.file_stem().and_then(|v| v.to_str()) != Some(id) {
                return Err(std::io::Error::other("invalid invitation record path"));
            }
            if record.offer.expires_ms <= now {
                std::fs::remove_file(path)?;
                continue;
            }
            record.offer.validate(now).map_err(std::io::Error::other)?;
            if let Some(acceptance) = &record.acceptance {
                acceptance
                    .validate(&record.offer)
                    .map_err(std::io::Error::other)?;
            }
            index.insert(
                id.clone(),
                IndexEntry {
                    expiry: record.offer.expires_ms,
                    identity: token_hash(&revault_publish_protocol::exchange::hex(
                        &record.offer.inviter.signing_key,
                    )),
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

    fn read(&self, id: &str) -> std::result::Result<Record, String> {
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
        let bytes = encode(record).map_err(|_| "invitation too large")?;
        let mut file =
            tempfile::NamedTempFile::new_in(&self.directory).map_err(|_| "storage unavailable")?;
        file.write_all(&bytes)
            .and_then(|()| file.as_file().sync_all())
            .map_err(|_| "storage write failed")?;
        file.persist(self.path(&record.offer.id))
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
        if let Request::Create { offer, owner_token } = request {
            offer.validate(now).map_err(|e| e.to_string())?;
            if !valid_token(&owner_token) {
                return Err("invalid capability".to_owned());
            }
            if index.contains_key(&offer.id) {
                let record = self.read(&offer.id)?;
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
        let mut record = self.read(id)?;
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
                // Keep an authenticated receipt until expiry so a lost response
                // can be retried. Payload cleanup is performed by expiry.
                self.write(&record)?;
                Ok(reply(&record, true))
            }
            Request::Cancel { owner_token, .. } => {
                if !record.is_owner(&owner_token) {
                    return Err("invalid capability".to_owned());
                }
                if record.acceptance.is_some() {
                    return Err("accepted invitations cannot be cancelled".to_owned());
                }
                std::fs::remove_file(self.path(&record.offer.id))
                    .map_err(|_| "storage cleanup failed")?;
                index.remove(&record.offer.id);
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
