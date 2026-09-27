//! Candidate write-ahead preparation journal. Fixed mirrored slots own an
//! unpublished append tail and explicitly reserved reusable ranges. Reuse/erasure
//! authority comes from the selected owner-authenticated allocation index, never
//! from the journal itself. Not activated in production archives.
use super::authenticated_index::Index;
use super::publication_anchor::{self as publication, Anchor, Authority, Published, REGION_LEN};
use crate::creation_options::FormatMode;
use crate::crypto::{open_with_nonce, seal_with_random_nonce, strong_checksum};
use crate::storage::Storage;
use crate::{Error, LockboxId, OwnerSigningKeyPair, Result};
use sha2::Sha256;
use std::sync::{Arc, Mutex, MutexGuard};
use zeroize::Zeroizing;

pub(crate) const SLOT_BYTES: usize = 65536;
pub(crate) const DATA_START: u64 = (REGION_LEN + 2 * SLOT_BYTES) as u64;
pub(crate) const FREE: u8 = 240;
pub(crate) const PENDING: u8 = 241;
const MAX_RESERVATIONS: usize = 2048;
const CLEANUP_CHECKPOINT: u64 = 8 * 1024 * 1024;
const MAGIC: &[u8; 8] = b"RV4PRE02";
const HEADER: usize = 48;
const CHECKSUM: usize = SLOT_BYTES - 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Reservation {
    pub namespace: u8,
    pub base: u64,
    pub start: u64,
    pub len: u64,
}
impl Reservation {
    fn end(self) -> Result<u64> {
        self.start.checked_add(self.len).ok_or(Error::CorruptRecord)
    }
}
#[derive(Clone, PartialEq, Eq)]
struct Record {
    sequence: u64,
    previous: [u8; 32],
    base: [u8; 32],
    active: bool,
    cleanup_commit: [u8; 32],
    cleanup_bytes: u64,
    reservations: Vec<Reservation>,
}
struct Selected {
    record: Record,
    digest: [u8; 32],
    encoded: Vec<u8>,
    slot: usize,
    copies: u8,
}
struct Context {
    archive: LockboxId,
    mode: FormatMode,
    key: Option<Zeroizing<[u8; 32]>>,
    index: Index,
}
impl Context {
    fn new(archive: LockboxId, mode: FormatMode, key: Option<&[u8]>) -> Result<Self> {
        let index = Index::new(archive, mode, key)?;
        let key = key.map(|key| {
            let mut derived = Zeroizing::new([0; 32]);
            hkdf::Hkdf::<Sha256>::new(Some(archive.as_bytes()), key)
                .expand(b"revault-candidate-preparation-key-v2\0", &mut *derived)
                .expect("fixed key size");
            derived
        });
        Ok(Self {
            archive,
            mode,
            key,
            index,
        })
    }
    fn encode(&self, record: &Record) -> Result<Vec<u8>> {
        let body_size = CHECKSUM - HEADER - if self.key.is_some() { 16 } else { 0 };
        let body = encode_body(record, body_size)?;
        let mut out = Vec::with_capacity(SLOT_BYTES);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&self.mode.0.to_le_bytes());
        out.extend_from_slice(self.archive.as_bytes());
        out.extend_from_slice(&(SLOT_BYTES as u32).to_le_bytes());
        if let Some(key) = &self.key {
            let (nonce, encrypted) = seal_with_random_nonce(&body, key.as_slice(), &out)?;
            out.extend_from_slice(&nonce);
            out.extend_from_slice(&(encrypted.len() as u32).to_le_bytes());
            out.extend_from_slice(&encrypted);
        } else {
            out.extend_from_slice(&[0; 12]);
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(&body);
        }
        debug_assert_eq!(out.len(), CHECKSUM);
        let checksum = strong_checksum(&out);
        out.extend_from_slice(&checksum);
        Ok(out)
    }
    fn decode(&self, bytes: &[u8]) -> Result<Record> {
        if bytes.len() != SLOT_BYTES
            || &bytes[..8] != MAGIC
            || bytes[8..10] != 2u16.to_le_bytes()
            || bytes[10..12] != self.mode.0.to_le_bytes()
            || bytes[12..28] != *self.archive.as_bytes()
            || bytes[28..32] != (SLOT_BYTES as u32).to_le_bytes()
            || u32::from_le_bytes(bytes[44..48].try_into().unwrap()) as usize != CHECKSUM - HEADER
            || strong_checksum(&bytes[..CHECKSUM]) != bytes[CHECKSUM..]
        {
            return Err(Error::CorruptRecord);
        }
        let body = if let Some(key) = &self.key {
            Zeroizing::new(open_with_nonce(
                &bytes[HEADER..CHECKSUM],
                key.as_slice(),
                &bytes[32..44],
                &bytes[..32],
            )?)
        } else {
            if bytes[32..44] != [0; 12] {
                return Err(Error::CorruptRecord);
            }
            Zeroizing::new(bytes[HEADER..CHECKSUM].to_vec())
        };
        decode_body(&body)
    }
    fn select(&self, storage: &impl Storage) -> Result<Selected> {
        let mut candidates = Vec::new();
        for slot in 0..2 {
            let encoded = storage.read_at((REGION_LEN + slot * SLOT_BYTES) as u64, SLOT_BYTES)?;
            if let Ok(record) = self.decode(&encoded) {
                candidates.push(Selected {
                    record,
                    digest: strong_checksum(&encoded),
                    encoded,
                    slot,
                    copies: 1 << slot,
                });
            }
        }
        candidates.sort_by_key(|c| c.record.sequence);
        let mut best = candidates.pop().ok_or(Error::CorruptRecord)?;
        if let Some(old) = candidates.pop() {
            if old.record.sequence == best.record.sequence {
                if old.digest != best.digest {
                    return Err(Error::CorruptRecord);
                }
                best.copies |= old.copies;
            } else if old.record.sequence.checked_add(1) != Some(best.record.sequence)
                || best.record.previous != old.digest
            {
                return Err(Error::CorruptRecord);
            }
        }
        Ok(best)
    }
    fn transition(
        &self,
        storage: &mut impl Storage,
        old: &Selected,
        base: [u8; 32],
        active: bool,
        reservations: Vec<Reservation>,
    ) -> Result<Selected> {
        let record = Record {
            sequence: 1,
            previous: [0; 32],
            base,
            active,
            cleanup_commit: [0; 32],
            cleanup_bytes: 0,
            reservations,
        };
        self.replace(storage, old, record)
    }
    fn replace(
        &self,
        storage: &mut impl Storage,
        old: &Selected,
        mut record: Record,
    ) -> Result<Selected> {
        record.sequence = old
            .record
            .sequence
            .checked_add(1)
            .ok_or(Error::CorruptRecord)?;
        record.previous = old.digest;
        let encoded = self.encode(&record)?;
        // Repair/sync the prior record before replacing either copy. Even two
        // readable copies may come from a failed final synchronization.
        mirror(storage, old)?;
        let first = 1 - old.slot;
        storage.write_at((REGION_LEN + first * SLOT_BYTES) as u64, &encoded)?;
        storage.sync()?;
        storage.write_at((REGION_LEN + old.slot * SLOT_BYTES) as u64, &encoded)?;
        storage.sync()?;
        Ok(Selected {
            record,
            digest: strong_checksum(&encoded),
            encoded,
            slot: first,
            copies: 3,
        })
    }
    fn validate_reservations(
        &self,
        storage: &impl Storage,
        anchor: &Anchor,
        reservations: &[Reservation],
    ) -> Result<()> {
        if reservations.len() > MAX_RESERVATIONS {
            return Err(Error::SecurityLimitExceeded(
                "preparation reservation limit".into(),
            ));
        }
        let mut sorted = reservations.to_vec();
        sorted.sort_by_key(|r| r.start);
        for r in &sorted {
            validate_range(anchor, *r)?;
            if !matches!(r.namespace, FREE | PENDING) {
                return Err(Error::CorruptRecord);
            }
            let entry = self
                .index
                .get(
                    storage,
                    anchor.allocation,
                    anchor.sealed_len,
                    r.namespace,
                    &r.base.to_be_bytes(),
                )?
                .ok_or(Error::CorruptRecord)?;
            if entry.value.len() != 8 {
                return Err(Error::CorruptRecord);
            }
            let len = u64::from_le_bytes(entry.value.as_slice().try_into().unwrap());
            let end = r.base.checked_add(len).ok_or(Error::CorruptRecord)?;
            if r.start < r.base || r.end()? > end || end > anchor.sealed_len {
                return Err(Error::CorruptRecord);
            }
        }
        for pair in sorted.windows(2) {
            if pair[0].end()? > pair[1].start {
                return Err(Error::CorruptRecord);
            }
        }
        Ok(())
    }
    fn pending(&self, storage: &impl Storage, anchor: &Anchor) -> Result<Vec<Reservation>> {
        let mut ranges = Vec::new();
        self.index.visit_range(
            storage,
            anchor.allocation,
            anchor.sealed_len,
            Some((PENDING, b"")),
            Some((PENDING + 1, b"")),
            |entry| {
                if entry.namespace != PENDING || entry.key.len() != 8 || entry.value.len() != 8 {
                    return Err(Error::CorruptRecord);
                }
                let start = u64::from_be_bytes(entry.key.as_slice().try_into().unwrap());
                let len = u64::from_le_bytes(entry.value.as_slice().try_into().unwrap());
                let range = Reservation {
                    namespace: PENDING,
                    base: start,
                    start,
                    len,
                };
                validate_range(anchor, range)?;
                ranges.push(range);
                Ok(())
            },
        )?;
        for pair in ranges.windows(2) {
            if pair[0].end()? > pair[1].start {
                return Err(Error::CorruptRecord);
            }
        }
        Ok(ranges)
    }
    fn clean(
        &self,
        storage: &mut impl Storage,
        journal: &mut Selected,
        commit: [u8; 32],
        ranges: &[Reservation],
    ) -> Result<()> {
        if journal.record.cleanup_commit != [0; 32] && journal.record.cleanup_commit != commit {
            return Err(Error::CorruptRecord);
        }
        let total = ranges.iter().try_fold(0u64, |total, r| {
            total.checked_add(r.len).ok_or(Error::CorruptRecord)
        })?;
        if journal.record.cleanup_bytes > total {
            return Err(Error::CorruptRecord);
        }
        let mut skip = journal.record.cleanup_bytes;
        let mut completed = skip;
        let mut checkpoint = skip;
        let zeros = [0; 65536];
        for range in ranges {
            if skip >= range.len {
                skip -= range.len;
                continue;
            }
            let mut position = range.start + skip;
            let mut remaining = range.len - skip;
            skip = 0;
            while remaining > 0 {
                let chunk = remaining.min(zeros.len() as u64);
                storage.write_at(position, &zeros[..chunk as usize])?;
                position += chunk;
                remaining -= chunk;
                completed += chunk;
                if completed - checkpoint >= CLEANUP_CHECKPOINT {
                    // Never publish progress before the corresponding zeros are durable.
                    storage.sync()?;
                    let mut record = journal.record.clone();
                    record.cleanup_commit = commit;
                    record.cleanup_bytes = completed;
                    *journal = self.replace(storage, journal, record)?;
                    checkpoint = completed;
                }
            }
        }
        storage.sync()
    }
    fn recover(&self, storage: &mut impl Storage, authority: &Authority<'_>) -> Result<()> {
        let selected = publication::select(storage, self.archive, self.mode, authority)?;
        validate_anchor(&selected.anchor)?;
        let mut journal = self.select(storage)?;
        if journal.record.base != selected.commitment
            && (!journal.record.active || journal.record.base != selected.anchor.previous)
        {
            return Err(Error::CorruptRecord);
        }
        // The durability result is obtained here; an untrusted caller cannot
        // authorize erasure by supplying a merely readable publication pair.
        let published = publication::ensure_mirrored(
            storage,
            self.archive,
            self.mode,
            authority,
            selected.commitment,
        )?;
        if !journal.record.active {
            if storage.len()? != published.anchor.sealed_len {
                return Err(Error::CorruptRecord);
            }
            return mirror(storage, &journal);
        }
        if journal.record.base == published.commitment {
            self.validate_reservations(storage, &published.anchor, &journal.record.reservations)?;
            let ranges = journal.record.reservations.clone();
            self.clean(storage, &mut journal, published.commitment, &ranges)?;
        } else {
            // New owner publication won. Never erase the old preparation's reused
            // ranges: they may now contain live data. Only its new pending map
            // grants authority to retire old data and obsolete control pages.
            let ranges = self.pending(storage, &published.anchor)?;
            self.clean(storage, &mut journal, published.commitment, &ranges)?;
        }
        if storage.len()? < published.anchor.sealed_len {
            return Err(Error::Truncated);
        }
        storage.truncate(published.anchor.sealed_len)?;
        storage.sync()?;
        self.transition(storage, &journal, published.commitment, false, Vec::new())?;
        Ok(())
    }
}

fn mirror(storage: &mut impl Storage, selected: &Selected) -> Result<()> {
    storage.sync()?;
    if selected.copies != 3 {
        storage.write_at(
            (REGION_LEN + (1 - selected.slot) * SLOT_BYTES) as u64,
            &selected.encoded,
        )?;
        storage.sync()?;
    }
    Ok(())
}
fn validate_anchor(anchor: &Anchor) -> Result<()> {
    if anchor.sealed_len < DATA_START || anchor.allocation == Default::default() {
        return Err(Error::CorruptHeader);
    }
    for root in [anchor.index, anchor.allocation, anchor.keys] {
        if root != Default::default() && (root.primary < DATA_START || root.mirror < DATA_START) {
            return Err(Error::CorruptHeader);
        }
    }
    Ok(())
}
fn validate_range(anchor: &Anchor, range: Reservation) -> Result<()> {
    let end = range.end()?;
    if range.len == 0 || range.start < DATA_START || end > anchor.sealed_len {
        return Err(Error::CorruptRecord);
    }
    for root in [anchor.index, anchor.allocation, anchor.keys] {
        for offset in [root.primary, root.mirror] {
            let root_end = offset.checked_add(root.len).ok_or(Error::CorruptRecord)?;
            if range.start < root_end && offset < end {
                return Err(Error::CorruptRecord);
            }
        }
    }
    Ok(())
}
fn zero(storage: &mut impl Storage, mut start: u64, mut len: u64) -> Result<()> {
    let bytes = [0; 65536];
    while len > 0 {
        let chunk = len.min(bytes.len() as u64);
        storage.write_at(start, &bytes[..chunk as usize])?;
        start += chunk;
        len -= chunk;
    }
    Ok(())
}
fn require_zero(storage: &impl Storage, range: Reservation) -> Result<()> {
    let mut start = range.start;
    let mut left = range.len;
    while left > 0 {
        let chunk = left.min(65536);
        if Zeroizing::new(storage.read_at(start, chunk as usize)?)
            .iter()
            .any(|b| *b != 0)
        {
            return Err(Error::CorruptRecord);
        }
        start += chunk;
        left -= chunk;
    }
    Ok(())
}

/// Initialize only a newly reserved, zeroed journal region after first publication.
/// The caller must already hold exclusive ownership of the storage.
pub(crate) fn initialize(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<()> {
    let context = Context::new(archive, mode, key)?;
    let selected = publication::select(storage, archive, mode, authority)?;
    validate_anchor(&selected.anchor)?;
    if storage.len()? != selected.anchor.sealed_len
        || storage
            .read_at(REGION_LEN as u64, 2 * SLOT_BYTES)?
            .iter()
            .any(|b| *b != 0)
    {
        return Err(Error::CorruptRecord);
    }
    publication::ensure_mirrored(storage, archive, mode, authority, selected.commitment)?;
    for range in context.pending(storage, &selected.anchor)? {
        zero(storage, range.start, range.len)?;
    }
    storage.sync()?;
    let record = Record {
        sequence: 1,
        previous: [0; 32],
        base: selected.commitment,
        active: false,
        cleanup_commit: [0; 32],
        cleanup_bytes: 0,
        reservations: Vec::new(),
    };
    let encoded = context.encode(&record)?;
    storage.write_at(REGION_LEN as u64, &encoded)?;
    storage.sync()?;
    storage.write_at((REGION_LEN + SLOT_BYTES) as u64, &encoded)?;
    storage.sync()
}
pub(crate) fn recover(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<()> {
    Context::new(archive, mode, key)?.recover(storage, authority)
}

struct State<S> {
    storage: S,
    context: Context,
    base: Anchor,
    commitment: [u8; 32],
    journal: Selected,
    active: bool,
}
#[derive(Clone)]
pub(crate) struct PreparedStore<S: Storage> {
    state: Arc<Mutex<State<S>>>,
}
impl<S: Storage> std::fmt::Debug for PreparedStore<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedStore").finish_non_exhaustive()
    }
}
impl<S: Storage> PreparedStore<S> {
    /// Caller holds the exclusive archive writer lock through recovery and the
    /// transaction. Clone the returned handle to share it; separate begin calls
    /// are not serialized by this component.
    pub(crate) fn begin(
        mut storage: S,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        key: Option<&[u8]>,
    ) -> Result<Self> {
        let context = Context::new(archive, mode, key)?;
        context.recover(&mut storage, authority)?;
        let selected = publication::select(&storage, archive, mode, authority)?;
        let old = context.select(&storage)?;
        let journal =
            context.transition(&mut storage, &old, selected.commitment, true, Vec::new())?;
        Ok(Self {
            state: Arc::new(Mutex::new(State {
                storage,
                context,
                base: selected.anchor,
                commitment: selected.commitment,
                journal,
                active: true,
            })),
        })
    }
    fn lock(&self) -> Result<MutexGuard<'_, State<S>>> {
        self.state
            .lock()
            .map_err(|_| Error::Io("preparation lock poisoned".into()))
    }
    pub(crate) fn reserve(&mut self, requests: &[Reservation]) -> Result<()> {
        let mut state = self.lock()?;
        if !state.active {
            return Err(Error::CorruptRecord);
        }
        if requests.is_empty() {
            return Ok(());
        }
        let result = (|| {
            let State {
                storage,
                context,
                base,
                commitment,
                journal,
                ..
            } = &mut *state;
            let mut reservations = journal.record.reservations.clone();
            reservations.extend_from_slice(requests);
            reservations.sort_by_key(|r| r.start);
            let mut merged: Vec<Reservation> = Vec::with_capacity(reservations.len());
            for range in reservations {
                if let Some(last) = merged.last_mut() {
                    let end = last.end()?;
                    if end > range.start {
                        return Err(Error::CorruptRecord);
                    }
                    if end == range.start
                        && last.base == range.base
                        && last.namespace == range.namespace
                    {
                        last.len = last
                            .len
                            .checked_add(range.len)
                            .ok_or(Error::CorruptRecord)?;
                        continue;
                    }
                }
                merged.push(range);
            }
            let reservations = merged;
            context.validate_reservations(storage, base, &reservations)?;
            for range in requests {
                require_zero(storage, *range)?;
            }
            *journal = context.transition(storage, journal, *commitment, true, reservations)?;
            Ok(())
        })();
        if result.is_err() {
            state.active = false;
        }
        result
    }
    pub(crate) fn abort(&mut self, authority: &Authority<'_>) -> Result<()> {
        let mut state = self.lock()?;
        if !state.active {
            return Err(Error::CorruptRecord);
        }
        state.active = false;
        let State {
            storage, context, ..
        } = &mut *state;
        context.recover(storage, authority)
    }
    /// Any error after publication starts has an uncertain commit outcome. Reopen
    /// through recovery to select the durable generation; never assume rollback.
    pub(crate) fn commit(
        &mut self,
        next: &Anchor,
        authority: &Authority<'_>,
        signer: Option<&OwnerSigningKeyPair>,
    ) -> Result<Published> {
        let mut state = self.lock()?;
        if !state.active {
            return Err(Error::CorruptRecord);
        }
        state.active = false;
        validate_anchor(next)?;
        let State {
            storage,
            context,
            commitment,
            ..
        } = &mut *state;
        let result = publication::publish(storage, next, authority, signer, Some(*commitment))?;
        context.recover(storage, authority)?;
        Ok(result)
    }
    pub(crate) fn into_inner(self) -> Result<S> {
        Arc::try_unwrap(self.state)
            .map_err(|_| Error::InvalidInput("preparation handles still exist".into()))?
            .into_inner()
            .map(|state| state.storage)
            .map_err(|_| Error::Io("preparation lock poisoned".into()))
    }
}
impl<S: Storage> Storage for PreparedStore<S> {
    fn len(&self) -> Result<u64> {
        self.lock()?.storage.len()
    }
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        self.lock()?.storage.read_at(offset, len)
    }
    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        self.lock()?.storage.read_at_into(offset, out)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        let mut state = self.lock()?;
        if !state.active {
            return Err(Error::CorruptRecord);
        }
        if state.storage.len()? < state.base.sealed_len {
            state.active = false;
            return Err(Error::Truncated);
        }
        let result = state.storage.append(bytes);
        if result.is_err() {
            state.active = false;
        }
        result
    }
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        let mut state = self.lock()?;
        if !state.active {
            return Err(Error::CorruptRecord);
        }
        let end = offset
            .checked_add(bytes.len() as u64)
            .ok_or(Error::CorruptRecord)?;
        let len = state.storage.len()?;
        if len < state.base.sealed_len {
            state.active = false;
            return Err(Error::Truncated);
        }
        let in_tail = offset >= state.base.sealed_len && end <= len;
        let reserved = state
            .journal
            .record
            .reservations
            .iter()
            .any(|r| offset >= r.start && r.end().is_ok_and(|limit| end <= limit));
        if bytes.is_empty() || (!in_tail && !reserved) {
            return Err(Error::CorruptRecord);
        }
        let result = state.storage.write_at(offset, bytes);
        if result.is_err() {
            state.active = false;
        }
        result
    }
    fn truncate(&mut self, _len: u64) -> Result<()> {
        Err(Error::InvalidInput(
            "preparation storage cannot truncate directly".into(),
        ))
    }
    fn sync(&self) -> Result<()> {
        let mut state = self.lock()?;
        if !state.active {
            return Err(Error::CorruptRecord);
        }
        let result = state.storage.sync();
        if result.is_err() {
            state.active = false;
        }
        result
    }
}
struct Cursor<'a>(&'a [u8]);
impl<'a> Cursor<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8]> {
        if len > self.0.len() {
            return Err(Error::CorruptRecord);
        }
        let (head, tail) = self.0.split_at(len);
        self.0 = tail;
        Ok(head)
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
}

#[cfg(test)]
#[path = "preparation_journal_tests.rs"]
pub(super) mod tests;

fn encode_body(record: &Record, body_size: usize) -> Result<Zeroizing<Vec<u8>>> {
    if record.sequence == 0
        || ((record.sequence == 1) != (record.previous == [0; 32]))
        || record.reservations.len() > MAX_RESERVATIONS
        || (!record.active && (!record.reservations.is_empty() || record.cleanup_bytes != 0))
        || ((record.cleanup_bytes == 0) != (record.cleanup_commit == [0; 32]))
    {
        return Err(Error::CorruptRecord);
    }
    if 117 + record.reservations.len() * 25 > body_size {
        return Err(Error::SecurityLimitExceeded(
            "preparation record needs overflow".into(),
        ));
    }
    let mut body = Zeroizing::new(Vec::with_capacity(body_size));
    body.extend_from_slice(&record.sequence.to_le_bytes());
    body.extend_from_slice(&record.previous);
    body.extend_from_slice(&record.base);
    body.push(u8::from(record.active));
    body.extend_from_slice(&record.cleanup_commit);
    body.extend_from_slice(&record.cleanup_bytes.to_le_bytes());
    body.extend_from_slice(&(record.reservations.len() as u32).to_le_bytes());
    for r in &record.reservations {
        body.push(r.namespace);
        body.extend_from_slice(&r.base.to_le_bytes());
        body.extend_from_slice(&r.start.to_le_bytes());
        body.extend_from_slice(&r.len.to_le_bytes());
    }
    body.resize(body_size, 0);
    Ok(body)
}

fn decode_body(body: &[u8]) -> Result<Record> {
    let mut cursor = Cursor(body);
    let sequence = cursor.u64()?;
    let previous = cursor.take(32)?.try_into().unwrap();
    let base = cursor.take(32)?.try_into().unwrap();
    let active = match cursor.take(1)?[0] {
        0 => false,
        1 => true,
        _ => return Err(Error::CorruptRecord),
    };
    let cleanup_commit = cursor.take(32)?.try_into().unwrap();
    let cleanup_bytes = cursor.u64()?;
    let count = u32::from_le_bytes(cursor.take(4)?.try_into().unwrap()) as usize;
    if sequence == 0
        || ((sequence == 1) != (previous == [0; 32]))
        || count > MAX_RESERVATIONS
        || count > cursor.0.len() / 25
        || (!active && (count != 0 || cleanup_bytes != 0))
        || ((cleanup_bytes == 0) != (cleanup_commit == [0; 32]))
    {
        return Err(Error::CorruptRecord);
    }
    let mut reservations = Vec::with_capacity(count);
    for _ in 0..count {
        reservations.push(Reservation {
            namespace: cursor.take(1)?[0],
            base: cursor.u64()?,
            start: cursor.u64()?,
            len: cursor.u64()?,
        });
    }
    if cursor.0.iter().any(|byte| *byte != 0) {
        return Err(Error::CorruptRecord);
    }
    Ok(Record {
        sequence,
        previous,
        base,
        active,
        cleanup_commit,
        cleanup_bytes,
        reservations,
    })
}

mod compact;
