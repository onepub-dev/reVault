//! Candidate physical ownership derived from authenticated index reachability.
//! All records use a bounded ownership envelope; public record/codec integration
//! remains separate. This module is not activated in production archives.
use super::authenticated_index::{Entry, Index, Visit};
use super::preparation_journal::{
    self as journal, PreparedStore, Reservation, DATA_START, FREE, PENDING,
};
use super::publication_anchor::{self as publication, Anchor, Authority, RootRef, FAILURE_REGION};
use crate::creation_options::FormatMode;
use crate::crypto::strong_checksum;
use crate::storage::Storage;
use crate::{Error, LockboxId, OwnerSigningKeyPair, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;

const MAGIC: &[u8; 8] = b"RV4OWN01";
const MAX_EXTENTS: usize = 512;
const MAX_RECORD: usize = 49152;
const MAX_CLAIMS: usize = 2_000_000;
const ARENA: u8 = 242;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Extent {
    pub start: u64,
    pub len: u64,
    pub digest: [u8; 32],
}
impl Extent {
    fn end(self) -> Result<u64> {
        self.start.checked_add(self.len).ok_or(Error::CorruptRecord)
    }
}
/// This envelope accounts for whole stored allocations, including a shared pack.
/// Slice/codec/file relationships belong to the typed record inside metadata.
/// The same physical pack may be named by several records only with identical
/// bounds and stored-byte commitment. Index/control pages cannot be shared.
pub(crate) struct OwnedRecord {
    pub metadata: Zeroizing<Vec<u8>>,
    pub extents: Vec<Extent>,
}
impl OwnedRecord {
    pub(crate) fn encode(metadata: &[u8], extents: &[Extent]) -> Result<Zeroizing<Vec<u8>>> {
        let len = extents
            .len()
            .checked_mul(48)
            .and_then(|n| n.checked_add(metadata.len()))
            .and_then(|n| n.checked_add(14))
            .ok_or(Error::CorruptRecord)?;
        if extents.len() > MAX_EXTENTS || len > MAX_RECORD {
            return Err(Error::SecurityLimitExceeded(
                "ownership envelope limit".into(),
            ));
        }
        let mut out = Zeroizing::new(Vec::with_capacity(len));
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&(extents.len() as u16).to_le_bytes());
        out.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
        for extent in extents {
            if extent.start < DATA_START || extent.len == 0 {
                return Err(Error::CorruptRecord);
            }
            extent.end()?;
            out.extend_from_slice(&extent.start.to_le_bytes());
            out.extend_from_slice(&extent.len.to_le_bytes());
            out.extend_from_slice(&extent.digest);
        }
        out.extend_from_slice(metadata);
        Ok(out)
    }
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 14 || bytes.len() > MAX_RECORD || &bytes[..8] != MAGIC {
            return Err(Error::CorruptRecord);
        }
        let count = u16::from_le_bytes(bytes[8..10].try_into().unwrap()) as usize;
        let metadata_len = u32::from_le_bytes(bytes[10..14].try_into().unwrap()) as usize;
        if count > MAX_EXTENTS
            || metadata_len > MAX_RECORD
            || 14 + 48 * count + metadata_len != bytes.len()
        {
            return Err(Error::CorruptRecord);
        }
        let mut extents = Vec::with_capacity(count);
        for chunk in bytes[14..14 + 48 * count].chunks_exact(48) {
            let extent = Extent {
                start: u64::from_le_bytes(chunk[..8].try_into().unwrap()),
                len: u64::from_le_bytes(chunk[8..16].try_into().unwrap()),
                digest: chunk[16..].try_into().unwrap(),
            };
            if extent.start < DATA_START || extent.len == 0 {
                return Err(Error::CorruptRecord);
            }
            extent.end()?;
            extents.push(extent);
        }
        Ok(Self {
            metadata: Zeroizing::new(bytes[14 + 48 * count..].to_vec()),
            extents,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Fixed,
    Payload,
    Index,
    Keys,
    Allocation,
    Reserve,
    Free,
    Pending,
}
#[derive(Clone, Copy, Debug)]
struct Claim {
    extent: Extent,
    kind: Kind,
}
#[derive(Default)]
struct Claims(BTreeMap<u64, Claim>);
impl Claims {
    fn insert(&mut self, claim: Claim, sealed: u64) -> Result<()> {
        let extent = claim.extent;
        let end = extent.end()?;
        if extent.len == 0
            || end > sealed
            || (claim.kind != Kind::Fixed && extent.start < DATA_START)
        {
            return Err(Error::CorruptRecord);
        }
        if let Some((_, previous)) = self.0.range(..=extent.start).next_back() {
            if previous.extent == extent
                && previous.kind == Kind::Payload
                && claim.kind == Kind::Payload
            {
                return Ok(());
            }
            if previous.extent.end()? > extent.start {
                return Err(Error::CorruptRecord);
            }
        }
        if self.0.range(extent.start..end).next().is_some() {
            return Err(Error::CorruptRecord);
        }
        if self.0.len() == MAX_CLAIMS {
            return Err(Error::SecurityLimitExceeded(
                "physical ownership limit".into(),
            ));
        }
        self.0.insert(extent.start, claim);
        Ok(())
    }
    fn page(&mut self, reference: RootRef, kind: Kind, sealed: u64) -> Result<()> {
        separated(reference)?;
        for start in [reference.primary, reference.mirror] {
            self.insert(
                Claim {
                    extent: Extent {
                        start,
                        len: reference.len,
                        digest: reference.digest,
                    },
                    kind,
                },
                sealed,
            )?;
        }
        Ok(())
    }
    fn covering(&self, start: u64, end: u64) -> Option<&Claim> {
        self.0
            .range(..=start)
            .next_back()
            .map(|(_, c)| c)
            .filter(|c| c.extent.end().is_ok_and(|limit| limit >= end))
    }
    fn complete(&self, sealed: u64) -> Result<()> {
        let mut position = 0;
        for claim in self.0.values() {
            if claim.extent.start != position {
                return Err(Error::CorruptRecord);
            }
            position = claim.extent.end()?;
        }
        if position != sealed {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Accounting {
    pub fixed: u64,
    pub payload: u64,
    pub index: u64,
    pub keys: u64,
    pub allocation: u64,
    pub reserve: u64,
    pub free: u64,
    pub pending: u64,
    pub total: u64,
}
pub(crate) struct Snapshot {
    claims: Claims,
    available: Vec<Reservation>,
    pub accounting: Accounting,
}
impl Snapshot {
    fn live(storage: &impl Storage, anchor: &Anchor, index: &Index) -> Result<Claims> {
        if anchor.sealed_len < DATA_START
            || anchor.sealed_len > storage.len()?
            || anchor.object_root != anchor.index.digest
        {
            return Err(Error::CorruptRecord);
        }
        let mut claims = Claims::default();
        claims.insert(
            Claim {
                extent: Extent {
                    start: 0,
                    len: DATA_START,
                    digest: [0; 32],
                },
                kind: Kind::Fixed,
            },
            anchor.sealed_len,
        )?;
        for (root, kind) in [(anchor.index, Kind::Index), (anchor.keys, Kind::Keys)] {
            if root == RootRef::default() {
                if kind == Kind::Index {
                    return Err(Error::CorruptRecord);
                }
                continue;
            }
            index.visit_owned(storage, root, anchor.sealed_len, |event| {
                match event {
                    Visit::Page(reference) => claims.page(reference, kind, anchor.sealed_len)?,
                    Visit::Entry(entry) => {
                        for extent in OwnedRecord::decode(&entry.value)?.extents {
                            claims.insert(
                                Claim {
                                    extent,
                                    kind: Kind::Payload,
                                },
                                anchor.sealed_len,
                            )?;
                        }
                    }
                }
                Ok(())
            })?;
        }
        Ok(claims)
    }
    /// Inspect every reachable index/key/allocation page and every record's
    /// physical ownership. This validates coverage, not payload decoding or the
    /// semantic relationships within typed record metadata.
    pub(crate) fn inspect(storage: &impl Storage, anchor: &Anchor, index: &Index) -> Result<Self> {
        Self::inspect_graph(storage, anchor, index, true)
    }
    fn inspect_graph(
        storage: &impl Storage,
        anchor: &Anchor,
        index: &Index,
        check_reserve: bool,
    ) -> Result<Self> {
        let mut claims = Self::live(storage, anchor, index)?;
        let mut available = Vec::new();
        let mut pages = Claims::default();
        let mut arena = None;
        index.visit_owned(storage, anchor.allocation, anchor.sealed_len, |event| {
            match event {
                Visit::Page(reference) => {
                    pages.page(reference, Kind::Allocation, anchor.sealed_len)?
                }
                Visit::Entry(entry) => {
                    if !matches!(entry.namespace, FREE | PENDING | ARENA)
                        || entry.key.len() != 8
                        || entry.value.len() != 8
                    {
                        return Err(Error::CorruptRecord);
                    }
                    let start = u64::from_be_bytes(entry.key.as_slice().try_into().unwrap());
                    let len = u64::from_le_bytes(entry.value.as_slice().try_into().unwrap());
                    if entry.namespace == ARENA {
                        let extent = Extent {
                            start,
                            len,
                            digest: [0; 32],
                        };
                        if arena.replace(extent).is_some()
                            || len == 0
                            || start < DATA_START
                            || extent.end()? > anchor.sealed_len
                        {
                            return Err(Error::CorruptRecord);
                        }
                        return Ok(());
                    }
                    let kind = if entry.namespace == FREE {
                        Kind::Free
                    } else {
                        Kind::Pending
                    };
                    claims.insert(
                        Claim {
                            extent: Extent {
                                start,
                                len,
                                digest: [0; 32],
                            },
                            kind,
                        },
                        anchor.sealed_len,
                    )?;
                    available.push(Reservation {
                        namespace: entry.namespace,
                        base: start,
                        start,
                        len,
                    });
                }
            }
            Ok(())
        })?;
        if let Some(arena) = arena {
            let mut position = arena.start;
            let end = arena.end()?;
            for page in pages.0.values() {
                if page.extent.start < position || page.extent.end()? > end {
                    return Err(Error::CorruptRecord);
                }
                if page.extent.start > position {
                    let extent = Extent {
                        start: position,
                        len: page.extent.start - position,
                        digest: [0; 32],
                    };
                    if check_reserve {
                        check_zero(storage, extent)?;
                    }
                    claims.insert(
                        Claim {
                            extent,
                            kind: Kind::Reserve,
                        },
                        anchor.sealed_len,
                    )?;
                }
                claims.insert(*page, anchor.sealed_len)?;
                position = page.extent.end()?;
            }
            if position < end {
                let extent = Extent {
                    start: position,
                    len: end - position,
                    digest: [0; 32],
                };
                if check_reserve {
                    check_zero(storage, extent)?;
                }
                claims.insert(
                    Claim {
                        extent,
                        kind: Kind::Reserve,
                    },
                    anchor.sealed_len,
                )?;
            }
        } else {
            for page in pages.0.values() {
                claims.insert(*page, anchor.sealed_len)?;
            }
        }
        claims.complete(anchor.sealed_len)?;
        let mut accounting = Accounting::default();
        for claim in claims.0.values() {
            let destination = match claim.kind {
                Kind::Fixed => &mut accounting.fixed,
                Kind::Payload => &mut accounting.payload,
                Kind::Index => &mut accounting.index,
                Kind::Keys => &mut accounting.keys,
                Kind::Allocation => &mut accounting.allocation,
                Kind::Reserve => &mut accounting.reserve,
                Kind::Free => &mut accounting.free,
                Kind::Pending => &mut accounting.pending,
            };
            *destination += claim.extent.len;
            accounting.total += claim.extent.len;
        }
        Ok(Self {
            claims,
            available,
            accounting,
        })
    }
    pub(crate) fn verify_reclaimed(&self, storage: &impl Storage) -> Result<()> {
        for range in &self.available {
            let mut position = range.start;
            let mut left = range.len;
            while left > 0 {
                let n = left.min(65536) as usize;
                let bytes = Zeroizing::new(storage.read_at(position, n)?);
                if bytes.iter().any(|b| *b != 0) {
                    return Err(Error::CorruptRecord);
                }
                position += n as u64;
                left -= n as u64;
            }
        }
        Ok(())
    }
    fn owns_payload(&self, extent: Extent) -> bool {
        self.claims
            .0
            .get(&extent.start)
            .is_some_and(|c| c.kind == Kind::Payload && c.extent == extent)
    }
}

fn check_zero(storage: &impl Storage, extent: Extent) -> Result<()> {
    let mut position = extent.start;
    let mut remaining = extent.len;
    while remaining > 0 {
        let n = remaining.min(65536) as usize;
        let bytes = Zeroizing::new(storage.read_at(position, n)?);
        if bytes.iter().any(|b| *b != 0) {
            return Err(Error::CorruptRecord);
        }
        position += n as u64;
        remaining -= n as u64;
    }
    Ok(())
}

struct AllocatorState<S: Storage> {
    prepared: PreparedStore<S>,
    available: BTreeMap<u64, Reservation>,
    sizes: BTreeSet<(u64, u64)>,
    written: Claims,
}
#[derive(Clone)]
struct Allocator<S: Storage>(Arc<Mutex<AllocatorState<S>>>);
impl<S: Storage> std::fmt::Debug for Allocator<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CandidateAllocator").finish_non_exhaustive()
    }
}
#[derive(Clone, Copy)]
enum Placement {
    Any,
    Node(Option<u64>),
    Aligned,
}
fn align_region(value: u64) -> Result<u64> {
    value
        .checked_add(FAILURE_REGION - 1)
        .map(|n| n / FAILURE_REGION * FAILURE_REGION)
        .ok_or(Error::CorruptRecord)
}
fn place(start: u64, len: u64, policy: Placement) -> Result<u64> {
    if len == 0 {
        return Err(Error::CorruptRecord);
    }
    let mut start = start;
    match policy {
        Placement::Any => {}
        Placement::Aligned => start = align_region(start)?,
        Placement::Node(excluded) => {
            if len > FAILURE_REGION {
                return Err(Error::CorruptRecord);
            }
            if start % FAILURE_REGION + len > FAILURE_REGION
                || excluded == Some(start / FAILURE_REGION)
            {
                start = align_region(start.checked_add(1).ok_or(Error::CorruptRecord)?)?;
            }
            if excluded == Some(start / FAILURE_REGION) {
                start = start
                    .checked_add(FAILURE_REGION)
                    .ok_or(Error::CorruptRecord)?;
            }
        }
    }
    start.checked_add(len).ok_or(Error::CorruptRecord)?;
    Ok(start)
}
fn separated(reference: RootRef) -> Result<()> {
    if reference.len == 0
        || reference.len > FAILURE_REGION
        || reference.primary / FAILURE_REGION == reference.mirror / FAILURE_REGION
    {
        return Err(Error::CorruptRecord);
    }
    for start in [reference.primary, reference.mirror] {
        if start % FAILURE_REGION + reference.len > FAILURE_REGION {
            return Err(Error::CorruptRecord);
        }
    }
    Ok(())
}
fn append_zeros(storage: &mut impl Storage, len: u64) -> Result<()> {
    let zeros = [0; 65536];
    let mut left = len;
    while left > 0 {
        let n = left.min(zeros.len() as u64) as usize;
        storage.append(&zeros[..n])?;
        left -= n as u64;
    }
    Ok(())
}
impl<S: Storage> AllocatorState<S> {
    fn take_available(&mut self, len: u64, policy: Placement) -> Result<Option<Reservation>> {
        let mut chosen = None;
        for &(_, original) in self.sizes.range((len, 0)..) {
            let range = self.available[&original];
            let start = place(range.start, len, policy)?;
            if start
                .checked_add(len)
                .is_some_and(|end| end <= range.start + range.len)
            {
                chosen = Some((range, start));
                break;
            }
        }
        let Some((range, start)) = chosen else {
            return Ok(None);
        };
        self.available.remove(&range.start);
        self.sizes.remove(&(range.len, range.start));
        for (part_start, part_len) in [
            (range.start, start - range.start),
            (start + len, range.start + range.len - start - len),
        ] {
            if part_len > 0 {
                let remainder = Reservation {
                    start: part_start,
                    len: part_len,
                    ..range
                };
                self.available.insert(part_start, remainder);
                self.sizes.insert((part_len, part_start));
            }
        }
        Ok(Some(Reservation {
            start,
            len,
            ..range
        }))
    }
    fn write_at_plan(&mut self, start: u64, bytes: &[u8]) -> Result<()> {
        let current = self.prepared.len()?;
        if start >= current {
            append_zeros(&mut self.prepared, start - current)?;
            let actual = self.prepared.append(bytes)?;
            if actual != start {
                return Err(Error::CorruptRecord);
            }
        } else {
            self.prepared.write_at(start, bytes)?;
        }
        let sealed = self.prepared.len()?;
        self.written.insert(
            Claim {
                extent: Extent {
                    start,
                    len: bytes.len() as u64,
                    digest: strong_checksum(bytes),
                },
                kind: Kind::Index,
            },
            sealed,
        )
    }
}

impl<S: Storage> Allocator<S> {
    fn new(prepared: PreparedStore<S>, reservations: &[Reservation]) -> Self {
        Self(Arc::new(Mutex::new(AllocatorState {
            prepared,
            available: reservations.iter().map(|r| (r.start, *r)).collect(),
            sizes: reservations.iter().map(|r| (r.len, r.start)).collect(),
            written: Claims::default(),
        })))
    }
    fn reserve_arena(&mut self, len: u64) -> Result<Extent> {
        let mut state = self.0.lock().unwrap();
        let start = if let Some(lease) = state.take_available(len, Placement::Aligned)? {
            state.prepared.reserve(&[lease])?;
            lease.start
        } else {
            let current = state.prepared.len()?;
            let start = place(current, len, Placement::Aligned)?;
            append_zeros(&mut state.prepared, start - current + len)?;
            start
        };
        let extent = Extent {
            start,
            len,
            digest: [0; 32],
        };
        let sealed = state.prepared.len()?;
        state.written.insert(
            Claim {
                extent,
                kind: Kind::Allocation,
            },
            sealed,
        )?;
        Ok(extent)
    }
    fn prepared(&self) -> PreparedStore<S> {
        self.0.lock().unwrap().prepared.clone()
    }
    fn into_prepared(self) -> Result<PreparedStore<S>> {
        Arc::try_unwrap(self.0)
            .map_err(|_| Error::InvalidInput("allocator handles still exist".into()))?
            .into_inner()
            .map(|s| s.prepared)
            .map_err(|_| Error::Io("allocator lock poisoned".into()))
    }
}
impl<S: Storage> Storage for Allocator<S> {
    fn len(&self) -> Result<u64> {
        self.0.lock().unwrap().prepared.len()
    }
    fn read_at(&self, o: u64, n: usize) -> Result<Vec<u8>> {
        self.0.lock().unwrap().prepared.read_at(o, n)
    }
    fn read_at_into(&self, o: u64, b: &mut [u8]) -> Result<()> {
        self.0.lock().unwrap().prepared.read_at_into(o, b)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        if bytes.is_empty() {
            return Err(Error::InvalidInput("empty physical allocation".into()));
        }
        let mut state = self.0.lock().unwrap();
        let len = bytes.len() as u64;
        let start = if let Some(lease) = state.take_available(len, Placement::Any)? {
            state.prepared.reserve(&[lease])?;
            lease.start
        } else {
            state.prepared.len()?
        };
        state.write_at_plan(start, bytes)?;
        Ok(start)
    }
    fn append_pair(&mut self, bytes: &[u8]) -> Result<(u64, u64)> {
        let len = bytes.len() as u64;
        let mut state = self.0.lock().unwrap();
        let mut tail = state.prepared.len()?;
        let mut leases = Vec::new();
        let primary = if let Some(lease) = state.take_available(len, Placement::Node(None))? {
            leases.push(lease);
            lease.start
        } else {
            let start = place(tail, len, Placement::Node(None))?;
            tail = start + len;
            start
        };
        let policy = Placement::Node(Some(primary / FAILURE_REGION));
        let mirror = if let Some(lease) = state.take_available(len, policy)? {
            leases.push(lease);
            lease.start
        } else {
            place(tail, len, policy)?
        };
        // Both reused copies are reserved by one durable journal transition.
        state.prepared.reserve(&leases)?;
        state.write_at_plan(primary, bytes)?;
        state.write_at_plan(mirror, bytes)?;
        Ok((primary, mirror))
    }
    fn write_at(&mut self, _o: u64, _b: &[u8]) -> Result<()> {
        Err(Error::InvalidInput(
            "allocated extents are immutable".into(),
        ))
    }
    fn truncate(&mut self, _n: u64) -> Result<()> {
        Err(Error::InvalidInput(
            "allocator cannot truncate directly".into(),
        ))
    }
    fn sync(&self) -> Result<()> {
        self.0.lock().unwrap().prepared.sync()
    }
}

#[derive(Clone)]
struct ArenaWriter<S: Storage> {
    storage: S,
    arena: Extent,
    position: Arc<Mutex<u64>>,
}
impl<S: Storage> std::fmt::Debug for ArenaWriter<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SeparatedMapBanks").finish_non_exhaustive()
    }
}
impl<S: Storage> Storage for ArenaWriter<S> {
    fn len(&self) -> Result<u64> {
        self.storage.len()
    }
    fn read_at(&self, o: u64, n: usize) -> Result<Vec<u8>> {
        self.storage.read_at(o, n)
    }
    fn read_at_into(&self, o: u64, b: &mut [u8]) -> Result<()> {
        self.storage.read_at_into(o, b)
    }
    fn append(&mut self, _b: &[u8]) -> Result<u64> {
        Err(Error::InvalidInput(
            "map arena requires paired nodes".into(),
        ))
    }
    fn append_pair(&mut self, bytes: &[u8]) -> Result<(u64, u64)> {
        if self.arena.start % FAILURE_REGION != 0 || self.arena.len % (2 * FAILURE_REGION) != 0 {
            return Err(Error::CorruptRecord);
        }
        let half = self.arena.len / 2;
        let mut position = self.position.lock().unwrap();
        let primary = place(*position, bytes.len() as u64, Placement::Node(None))?;
        if primary + bytes.len() as u64 > self.arena.start + half {
            return Err(Error::SecurityLimitExceeded(
                "allocation map bank exhausted".into(),
            ));
        }
        let mirror = primary + half;
        self.storage.write_at(primary, bytes)?;
        self.storage.write_at(mirror, bytes)?;
        *position = primary + bytes.len() as u64;
        Ok((primary, mirror))
    }
    fn write_at(&mut self, _o: u64, _b: &[u8]) -> Result<()> {
        Err(Error::InvalidInput("map nodes are immutable".into()))
    }
    fn truncate(&mut self, _n: u64) -> Result<()> {
        Err(Error::InvalidInput("map arena cannot truncate".into()))
    }
    fn sync(&self) -> Result<()> {
        self.storage.sync()
    }
}
/// Used only while creating an empty backend. Runtime allocation uses reuse-aware
/// paired placement; this bootstrap writer owns all appended alignment gaps.
#[derive(Clone, Debug)]
struct InitialPairs<S: Storage>(S);
impl<S: Storage> Storage for InitialPairs<S> {
    fn len(&self) -> Result<u64> {
        self.0.len()
    }
    fn read_at(&self, o: u64, n: usize) -> Result<Vec<u8>> {
        self.0.read_at(o, n)
    }
    fn read_at_into(&self, o: u64, b: &mut [u8]) -> Result<()> {
        self.0.read_at_into(o, b)
    }
    fn append(&mut self, b: &[u8]) -> Result<u64> {
        self.0.append(b)
    }
    fn append_pair(&mut self, b: &[u8]) -> Result<(u64, u64)> {
        let current = self.0.len()?;
        let primary = place(current, b.len() as u64, Placement::Node(None))?;
        append_zeros(&mut self.0, primary - current)?;
        self.0.append(b)?;
        let current = self.0.len()?;
        let mirror = place(
            current,
            b.len() as u64,
            Placement::Node(Some(primary / FAILURE_REGION)),
        )?;
        append_zeros(&mut self.0, mirror - current)?;
        self.0.append(b)?;
        Ok((primary, mirror))
    }
    fn write_at(&mut self, o: u64, b: &[u8]) -> Result<()> {
        self.0.write_at(o, b)
    }
    fn truncate(&mut self, n: u64) -> Result<()> {
        self.0.truncate(n)
    }
    fn sync(&self) -> Result<()> {
        self.0.sync()
    }
}

pub(crate) struct Transaction<S: Storage> {
    storage: Allocator<S>,
    index: Index,
    base: Anchor,
    snapshot: Snapshot,
    logical: RootRef,
    keys: RootRef,
    payloads: BTreeMap<u64, Extent>,
}
impl<S: Storage> Transaction<S> {
    /// Caller holds the exclusive archive writer lock throughout this transaction.
    pub(crate) fn begin(
        storage: S,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        key: Option<&[u8]>,
    ) -> Result<Self> {
        // Validate the complete selected graph before journal recovery can erase
        // any PENDING bytes. A direct-root-only check cannot protect descendants.
        let base = publication::select(&storage, archive, mode, authority)?.anchor;
        let index = Index::new(archive, mode, key)?;
        let snapshot = Snapshot::inspect(&storage, &base, &index)?;
        let prepared = PreparedStore::begin(storage, archive, mode, authority, key)?;
        let storage = Allocator::new(prepared, &snapshot.available);
        let logical = base.index;
        let keys = base.keys;
        Ok(Self {
            storage,
            index,
            base,
            snapshot,
            logical,
            keys,
            payloads: BTreeMap::new(),
        })
    }
    /// Read the selected base's immutable payloads while constructing replacements.
    /// The allocator never reuses live base extents before publication.
    pub(crate) fn read_storage(&self) -> &impl Storage {
        &self.storage
    }
    /// The caller supplies already encoded/protected bytes. Codec/frame semantics
    /// are outside this allocator; raw plaintext is not an encrypted file codec.
    pub(crate) fn append_encoded_extent(&mut self, bytes: &[u8]) -> Result<Extent> {
        let start = self.storage.append(bytes)?;
        let extent = Extent {
            start,
            len: bytes.len() as u64,
            digest: strong_checksum(bytes),
        };
        self.payloads.insert(start, extent);
        Ok(extent)
    }
    pub(crate) fn put(
        &mut self,
        namespace: u8,
        name: &[u8],
        metadata: &[u8],
        extents: &[Extent],
    ) -> Result<()> {
        self.put_record(false, namespace, name, metadata, extents)
    }
    pub(crate) fn put_key_record(
        &mut self,
        namespace: u8,
        name: &[u8],
        metadata: &[u8],
        extents: &[Extent],
    ) -> Result<()> {
        self.put_record(true, namespace, name, metadata, extents)
    }
    fn put_record(
        &mut self,
        keys: bool,
        namespace: u8,
        name: &[u8],
        metadata: &[u8],
        extents: &[Extent],
    ) -> Result<()> {
        for extent in extents {
            if !self.snapshot.owns_payload(*extent)
                && self.payloads.get(&extent.start) != Some(extent)
            {
                return Err(Error::CorruptRecord);
            }
        }
        let value = OwnedRecord::encode(metadata, extents)?;
        let mut root = if keys { self.keys } else { self.logical };
        if root == RootRef::default() {
            root = self.index.empty(&mut self.storage)?;
        }
        let sealed = self.storage.len()?;
        if self
            .index
            .get(&self.storage, root, sealed, namespace, name)?
            .is_some_and(|entry| entry.value.as_slice() == value.as_slice())
        {
            return Ok(());
        }
        let change = self.index.put(
            &mut self.storage,
            root,
            sealed,
            Entry::new(namespace, name, &value)?,
        )?;
        if keys {
            self.keys = change.root;
        } else {
            self.logical = change.root;
        }
        Ok(())
    }
    /// Stream a complete sorted logical-record replacement without building
    /// throwaway paths for each record. Input order/record limits are enforced by
    /// the index builder; every physical reference must already be owned here.
    pub(crate) fn replace_all_sorted(
        &mut self,
        records: impl IntoIterator<Item = Result<Entry>>,
    ) -> Result<()> {
        let Self {
            storage,
            index,
            snapshot,
            payloads,
            ..
        } = self;
        let records = records.into_iter().map(|entry| {
            let entry = entry?;
            for extent in OwnedRecord::decode(&entry.value)?.extents {
                if !snapshot.owns_payload(extent) && payloads.get(&extent.start) != Some(&extent) {
                    return Err(Error::CorruptRecord);
                }
            }
            Ok(entry)
        });
        self.logical = index.build_sorted(storage, records)?.root;
        Ok(())
    }
    pub(crate) fn remove(&mut self, namespace: u8, name: &[u8]) -> Result<()> {
        let sealed = self.storage.len()?;
        self.logical = self
            .index
            .remove(&mut self.storage, self.logical, sealed, namespace, name)?
            .root;
        Ok(())
    }
    pub(crate) fn abort(self, authority: &Authority<'_>) -> Result<S> {
        let mut prepared = self.storage.into_prepared()?;
        prepared.abort(authority)?;
        prepared.into_inner()
    }
    pub(crate) fn commit(
        mut self,
        authority: &Authority<'_>,
        signer: Option<&OwnerSigningKeyPair>,
    ) -> Result<(S, Anchor)> {
        if self.logical == self.base.index && self.keys == self.base.keys {
            let anchor = self.base.clone();
            return self.abort(authority).map(|storage| (storage, anchor));
        }
        let mut next = Anchor {
            generation: self
                .base
                .generation
                .checked_add(1)
                .ok_or(Error::CorruptRecord)?,
            previous: self.base.commitment()?,
            index: self.logical,
            keys: self.keys,
            object_root: self.logical.digest,
            sealed_len: self.storage.len()?,
            ..self.base.clone()
        };
        let mut live = Snapshot::live(&self.storage, &next, &self.index)?;
        let initial = {
            let state = self.storage.0.lock().unwrap();
            gap_records(&live, &self.snapshot, &state.written, next.sealed_len)?
        };
        // A single reserved arena can split one reusable range into two; its
        // own declaration adds one record. Three extra records bound that change.
        let capacity = self
            .index
            .fixed_record_node_bound(initial.len() as u64 + 3, 8, 8)?
            * 2
            * FAILURE_REGION;
        let arena = self.storage.reserve_arena(capacity)?;
        next.sealed_len = self.storage.len()?;
        live.insert(
            Claim {
                extent: arena,
                kind: Kind::Reserve,
            },
            next.sealed_len,
        )?;
        let mut records = {
            let state = self.storage.0.lock().unwrap();
            gap_records(&live, &self.snapshot, &state.written, next.sealed_len)?
        };
        records.push(Entry::new(
            ARENA,
            &arena.start.to_be_bytes(),
            &arena.len.to_le_bytes(),
        )?);
        records
            .sort_by(|a, b| (a.namespace, a.key.as_slice()).cmp(&(b.namespace, b.key.as_slice())));
        let mut writer = ArenaWriter {
            storage: self.storage.prepared(),
            arena,
            position: Arc::new(Mutex::new(arena.start)),
        };
        let allocation = self
            .index
            .build_sorted(&mut writer, records.into_iter().map(Ok))?;
        next.allocation = allocation.root;
        Snapshot::inspect(&writer, &next, &self.index)?;
        writer.storage.commit(&next, authority, signer)?;
        drop(writer);
        let prepared = self.storage.into_prepared()?;
        Ok((prepared.into_inner()?, next))
    }
}
fn gap_records(
    live: &Claims,
    base: &Snapshot,
    written: &Claims,
    sealed: u64,
) -> Result<Vec<Entry>> {
    let mut cuts = vec![0, sealed];
    for claims in [live, &base.claims, written] {
        for claim in claims.0.values() {
            cuts.push(claim.extent.start);
            cuts.push(claim.extent.end()?);
        }
    }
    cuts.sort_unstable();
    cuts.dedup();
    let mut ranges: Vec<(u8, u64, u64)> = Vec::new();
    for pair in cuts.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        if end > sealed {
            return Err(Error::CorruptRecord);
        }
        if live.covering(start, end).is_some() {
            continue;
        }
        let was_free = base
            .claims
            .covering(start, end)
            .is_some_and(|c| matches!(c.kind, Kind::Free | Kind::Pending));
        let namespace = if was_free && written.covering(start, end).is_none() {
            FREE
        } else {
            PENDING
        };
        if let Some(last) = ranges
            .last_mut()
            .filter(|r| r.0 == namespace && r.1 + r.2 == start)
        {
            last.2 += end - start;
        } else {
            ranges.push((namespace, start, end - start));
        }
    }
    ranges.sort_by_key(|r| (r.0, r.1));
    ranges
        .into_iter()
        .map(|(namespace, start, len)| {
            Entry::new(namespace, &start.to_be_bytes(), &len.to_le_bytes())
        })
        .collect()
}
#[derive(Debug, Default)]
pub(crate) struct MetadataRepair {
    pub copies: u64,
    pub zeroed_unused_bytes: u64,
}
/// Explicit metadata repair under the caller's exclusive writer lock. Audit
/// ownership first; reconstruct only authenticated page bytes or declared zeros.
/// Payload availability and content decoding remain separate checks.
pub(crate) fn repair_metadata(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<(Anchor, MetadataRepair)> {
    let selected = publication::select(storage, archive, mode, authority)?;
    let anchor = selected.anchor;
    let index = Index::new(archive, mode, key)?;
    let snapshot = Snapshot::inspect_graph(storage, &anchor, &index, false)?;
    let mut pages = Vec::new();
    for root in [anchor.index, anchor.keys, anchor.allocation] {
        if root != RootRef::default() {
            index.visit_owned(storage, root, anchor.sealed_len, |event| {
                if let Visit::Page(page) = event {
                    pages.push(page);
                }
                Ok(())
            })?;
        }
    }
    let mut report = MetadataRepair::default();
    for page in pages {
        report.copies += u64::from(page.repair(storage)?);
    }
    // Metadata copies are synced by this barrier before publication is repaired.
    // Do not clear any reserved/retired bytes before both publication copies hold.
    publication::ensure_mirrored(storage, archive, mode, authority, selected.commitment)?;
    let mut zeros = snapshot
        .claims
        .0
        .values()
        .filter(|c| matches!(c.kind, Kind::Reserve | Kind::Free | Kind::Pending))
        .map(|c| c.extent)
        .collect::<Vec<_>>();
    for slot in 0..2 {
        zeros.push(Extent {
            start: (slot * publication::SLOT_STRIDE + publication::SLOT_LEN) as u64,
            len: (publication::SLOT_STRIDE - publication::SLOT_LEN) as u64,
            digest: [0; 32],
        });
    }
    for extent in zeros {
        match check_zero(storage, extent) {
            Ok(()) => {}
            Err(Error::CorruptRecord) => {
                let mut position = extent.start;
                let mut left = extent.len;
                let bytes = [0; 65536];
                while left > 0 {
                    let n = left.min(bytes.len() as u64) as usize;
                    storage.write_at(position, &bytes[..n])?;
                    position += n as u64;
                    left -= n as u64;
                }
                report.zeroed_unused_bytes += extent.len;
            }
            Err(error) => return Err(error),
        }
    }
    storage.sync()?;
    journal::recover(storage, archive, mode, authority, key)?;
    Snapshot::inspect(storage, &anchor, &index)?;
    Ok((anchor, report))
}

/// Archive-level transaction recovery must audit the selected graph before
/// granting the lower-level journal authority to reclaim any physical range.
pub(crate) fn recover(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<Anchor> {
    let anchor = publication::select(storage, archive, mode, authority)?.anchor;
    Snapshot::inspect(storage, &anchor, &Index::new(archive, mode, key)?)?;
    journal::recover(storage, archive, mode, authority, key)?;
    Ok(anchor)
}
pub(crate) fn create_empty(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
) -> Result<Anchor> {
    if storage.len()? != 0 {
        return Err(Error::InvalidInput(
            "candidate creation requires empty storage".into(),
        ));
    }
    // Cloning is limited to a verified-empty backend, never an existing archive.
    let mut writer = InitialPairs(storage.clone());
    writer.append(&vec![0; DATA_START as usize])?;
    let index = Index::new(archive, mode, key)?;
    let root = index.empty(&mut writer)?;
    let old_len = writer.len()?;
    let arena_start = align_region(old_len)?;
    let arena = Extent {
        start: arena_start,
        len: 2 * FAILURE_REGION,
        digest: [0; 32],
    };
    append_zeros(&mut writer, arena_start - old_len + arena.len)?;
    let mut entries = Vec::new();
    for (start, end) in [
        (root.primary + root.len, root.mirror),
        (root.mirror + root.len, arena_start),
    ] {
        if end > start {
            entries.push(Entry::new(
                FREE,
                &start.to_be_bytes(),
                &(end - start).to_le_bytes(),
            )?);
        }
    }
    entries.push(Entry::new(
        ARENA,
        &arena.start.to_be_bytes(),
        &arena.len.to_le_bytes(),
    )?);
    let mut writer = ArenaWriter {
        storage: writer.0,
        arena,
        position: Arc::new(Mutex::new(arena.start)),
    };
    let allocation = index
        .build_sorted(&mut writer, entries.into_iter().map(Ok))?
        .root;
    let anchor = Anchor {
        archive,
        generation: 1,
        mode,
        sealed_len: writer.len()?,
        object_root: root.digest,
        previous: [0; 32],
        index: root,
        allocation,
        keys: RootRef::default(),
    };
    Snapshot::inspect(&writer, &anchor, &index)?;
    publication::publish(&mut writer.storage, &anchor, authority, signer, None)?;
    journal::initialize(&mut writer.storage, archive, mode, authority, key)?;
    *storage = writer.storage;
    Ok(anchor)
}

#[cfg(test)]
#[path = "allocation_map_tests.rs"]
mod tests;
