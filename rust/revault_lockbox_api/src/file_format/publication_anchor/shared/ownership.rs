//! Physical graph for the bounded shared-control experiment. Inputs must come
//! from the authenticated selected catalogue, not caller-supplied erase ranges.
//! A transition plan is a list of proof obligations, never write/erase authority:
//! the journal and both durable publications must authorize actual execution.
use super::*;
use crate::file_format::allocation_map::Extent;
use crate::page_buffer::ZeroizingBytes;
use std::collections::BTreeMap;
const MAX_CLAIMS: usize = 8192;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Span {
    pub start: u64,
    pub len: u64,
}
impl Span {
    fn end(self) -> Result<u64> {
        self.start.checked_add(self.len).ok_or(Error::CorruptRecord)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VacantKind {
    Free,
    Pending,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Vacant {
    pub span: Span,
    pub kind: VacantKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Fixed,
    Private,
    Descendant,
    Keys,
    Payload,
    Free,
    Pending,
}
impl Kind {
    fn live(self) -> bool {
        matches!(
            self,
            Self::Private | Self::Descendant | Self::Keys | Self::Payload
        )
    }
    fn vacant(self) -> bool {
        matches!(self, Self::Free | Self::Pending)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Claim {
    span: Span,
    digest: [u8; 32],
    kind: Kind,
}
pub(crate) struct Graph {
    anchor: Anchor,
    claims: BTreeMap<u64, Claim>,
}
#[derive(Debug)]
pub(crate) struct Transition {
    /// New allocations; ranges previously pending must be erased before reuse.
    pub writes: Vec<Span>,
    pub erase_before_write: Vec<Span>,
    /// Old live allocations: only erasable after BOTH new publications are durable.
    pub retire_after_publication: Vec<Span>,
    /// Pending-to-free ranges require independently verified completed erasure.
    pub prove_erased: Vec<Span>,
    /// Entire append interval needs durable preparation ownership before any write.
    pub append: Option<Span>,
    /// Metadata-only suffix: publish BOTH shorter anchors, wipe/sync, then truncate/sync.
    pub truncate: Option<Span>,
}
impl Graph {
    pub(crate) fn derive(anchor: &Anchor, packs: &[Extent], vacant: &[Vacant]) -> Result<Self> {
        Self::derive_with_descendants(anchor, packs, vacant, &[])
    }
    /// References must be enumerated from the authenticated selected catalogue
    /// traversal. This checks physical ownership, not reachability or page bytes.
    /// It does not authorize allocation or erasure without durable preparation.
    pub(crate) fn derive_with_descendants(
        anchor: &Anchor,
        packs: &[Extent],
        vacant: &[Vacant],
        descendants: &[RootRef],
    ) -> Result<Self> {
        anchor.validate_in(Layout::Shared)?;
        if anchor.index.len != PRIVATE_BYTES as u64
            || anchor.object_root != anchor.index.digest
            || !anchor.allocation.absent()
            || (!anchor.keys.absent() && anchor.keys.len != 4096)
        {
            return Err(Error::CorruptRecord);
        }
        // Descendant metadata has a distinct claim kind: the narrow private-root
        // tail-retirement proof must never silently erase descendant pages.
        // Journal-overflow arenas still require their own explicit graph type.
        let mut graph = Self {
            anchor: anchor.clone(),
            claims: BTreeMap::new(),
        };
        for bank in [0, FAILURE_REGION] {
            graph.insert(Claim {
                span: Span {
                    start: bank,
                    len: KEYS_START,
                },
                digest: [0; 32],
                kind: Kind::Fixed,
            })?;
        }
        for (root, kind) in [(anchor.index, Kind::Private), (anchor.keys, Kind::Keys)] {
            if root.absent() {
                continue;
            }
            for start in [root.primary, root.mirror] {
                graph.insert(Claim {
                    span: Span {
                        start,
                        len: root.len,
                    },
                    digest: root.digest,
                    kind,
                })?;
            }
        }
        for reference in descendants {
            if reference.len == 0 || reference.len > FAILURE_REGION {
                return Err(Error::CorruptRecord);
            }
            let mut regions = [0; 2];
            for (index, start) in [reference.primary, reference.mirror]
                .into_iter()
                .enumerate()
            {
                let end = start
                    .checked_add(reference.len)
                    .ok_or(Error::CorruptRecord)?;
                regions[index] = start / FAILURE_REGION;
                if start < REGION_LEN as u64 || regions[index] != (end - 1) / FAILURE_REGION {
                    return Err(Error::CorruptRecord);
                }
                graph.insert(Claim {
                    span: Span {
                        start,
                        len: reference.len,
                    },
                    digest: reference.digest,
                    kind: Kind::Descendant,
                })?;
            }
            if regions[0] == regions[1] {
                return Err(Error::CorruptRecord);
            }
        }
        for extent in packs {
            if extent.start < REGION_LEN as u64 {
                return Err(Error::CorruptRecord);
            }
            graph.insert(Claim {
                span: Span {
                    start: extent.start,
                    len: extent.len,
                },
                digest: extent.digest,
                kind: Kind::Payload,
            })?;
        }
        for entry in vacant {
            let end = entry.span.end()?;
            if entry.span.start < REGION_LEN as u64 {
                // Inline retirement owns exactly one typed slot; never publication,
                // preparation or a merged interval across private/public slots.
                let local = entry.span.start % FAILURE_REGION;
                if !((local == PRIVATE_START && entry.span.len == PRIVATE_BYTES as u64)
                    || (local == KEYS_START && entry.span.len == 4096))
                    || end > REGION_LEN as u64
                {
                    return Err(Error::CorruptRecord);
                }
            }
            graph.insert(Claim {
                span: entry.span,
                digest: [0; 32],
                kind: match entry.kind {
                    VacantKind::Free => Kind::Free,
                    VacantKind::Pending => Kind::Pending,
                },
            })?;
        }
        let mut position = 0;
        for claim in graph.claims.values() {
            if claim.span.start != position {
                return Err(Error::CorruptRecord);
            }
            position = claim.span.end()?;
        }
        if position != anchor.sealed_len {
            return Err(Error::CorruptRecord);
        }
        Ok(graph)
    }
    fn insert(&mut self, claim: Claim) -> Result<()> {
        let end = claim.span.end()?;
        if claim.span.len == 0 || end > self.anchor.sealed_len {
            return Err(Error::CorruptRecord);
        }
        if self.claims.len() == MAX_CLAIMS {
            return Err(Error::SecurityLimitExceeded(
                "bounded shared ownership graph".into(),
            ));
        }
        if self
            .claims
            .range(..=claim.span.start)
            .next_back()
            .is_some_and(|(_, old)| old.span.end().is_ok_and(|limit| limit > claim.span.start))
            || self.claims.range(claim.span.start..end).next().is_some()
        {
            return Err(Error::CorruptRecord);
        }
        self.claims.insert(claim.span.start, claim);
        Ok(())
    }
    fn covering(&self, span: Span) -> Result<&Claim> {
        self.claims
            .range(..=span.start)
            .next_back()
            .map(|(_, claim)| claim)
            .filter(|claim| {
                claim
                    .span
                    .end()
                    .is_ok_and(|end| span.end().is_ok_and(|limit| limit <= end))
            })
            .ok_or(Error::CorruptRecord)
    }
    pub(crate) fn fresh_files(anchor: &Anchor, packs: &[Extent]) -> Result<Self> {
        validate_fresh_shape(anchor)?;
        let vacant = if anchor.keys.absent() {
            [0, FAILURE_REGION]
                .into_iter()
                .map(|bank| Vacant {
                    span: Span {
                        start: bank + KEYS_START,
                        len: 4096,
                    },
                    kind: VacantKind::Free,
                })
                .collect()
        } else {
            Vec::new()
        };
        Self::derive(anchor, packs, &vacant)
    }
    /// Free means durably erased, not merely unreachable. Pending bytes may still
    /// contain an old secret and are deliberately not accepted by this check.
    pub(crate) fn verify_free(&self, storage: &impl Storage) -> Result<()> {
        self.verify_vacant(storage, false)
    }
    pub(crate) fn verify_reclaimed(&self, storage: &impl Storage) -> Result<()> {
        self.verify_vacant(storage, true)
    }
    fn verify_vacant(&self, storage: &impl Storage, pending: bool) -> Result<()> {
        for claim in self
            .claims
            .values()
            .filter(|claim| claim.kind == Kind::Free || (pending && claim.kind == Kind::Pending))
        {
            let mut start = claim.span.start;
            let end = claim.span.end()?;
            while start < end {
                let n = (end - start).min(65536) as usize;
                let bytes = ZeroizingBytes::new(storage.read_at(start, n)?);
                if bytes.len() != n || bytes.iter().any(|byte| *byte != 0) {
                    return Err(Error::CorruptRecord);
                }
                start += n as u64;
            }
        }
        Ok(())
    }
    pub(crate) fn pending(&self) -> Vec<Span> {
        self.claims
            .values()
            .filter(|claim| claim.kind == Kind::Pending)
            .map(|claim| claim.span)
            .collect()
    }
    pub(crate) fn validate_reservations(
        &self,
        reservations: &[crate::file_format::preparation_journal::Reservation],
    ) -> Result<Vec<Span>> {
        use crate::file_format::preparation_journal::{FREE, PENDING};
        let mut spans = Vec::new();
        for range in reservations {
            let span = Span {
                start: range.start,
                len: range.len,
            };
            if span.len == 0 {
                return Err(Error::CorruptRecord);
            }
            let claim = self.covering(span)?;
            let namespace = match claim.kind {
                Kind::Free => FREE,
                Kind::Pending => PENDING,
                _ => return Err(Error::CorruptRecord),
            };
            if namespace != range.namespace || range.base != claim.span.start {
                return Err(Error::CorruptRecord);
            }
            spans.push(span);
        }
        spans.sort_by_key(|span| span.start);
        for pair in spans.windows(2) {
            if pair[0].end()? > pair[1].start {
                return Err(Error::CorruptRecord);
            }
        }
        Ok(spans)
    }
    pub(crate) fn transition_to(&self, next: &Self) -> Result<Transition> {
        self.transition(next, false)
    }
    /// Narrow return-to-inline proof; payload and public-key claims cannot change.
    pub(crate) fn metadata_tail_transition_to(&self, next: &Self) -> Result<Transition> {
        if next.anchor.sealed_len >= self.anchor.sealed_len
            || next.anchor.index.primary != PRIVATE_START
            || next.anchor.index.mirror != FAILURE_REGION + PRIVATE_START
            || self.anchor.index.primary < next.anchor.sealed_len
            || self.anchor.index.mirror < next.anchor.sealed_len
            || self.anchor.keys != next.anchor.keys
        {
            return Err(Error::CorruptRecord);
        }
        let durable = |graph: &Self| {
            graph
                .claims
                .values()
                .filter(|claim| matches!(claim.kind, Kind::Payload | Kind::Keys | Kind::Descendant))
                .copied()
                .collect::<Vec<_>>()
        };
        if durable(self) != durable(next) {
            return Err(Error::CorruptRecord);
        }
        for old in self.claims.values().filter(|claim| {
            claim
                .span
                .end()
                .is_ok_and(|end| end > next.anchor.sealed_len)
        }) {
            if !old.kind.vacant() && old.kind != Kind::Private {
                return Err(Error::CorruptRecord);
            }
        }
        self.transition(next, true)
    }
    fn transition(&self, next: &Self, metadata_tail: bool) -> Result<Transition> {
        if next.anchor.archive != self.anchor.archive
            || next.anchor.mode != self.anchor.mode
            || self.anchor.generation.checked_add(1) != Some(next.anchor.generation)
            || next.anchor.previous != commitment(&self.anchor)?
            || (!metadata_tail && next.anchor.sealed_len < self.anchor.sealed_len)
        {
            return Err(Error::CorruptRecord);
        }
        let mut result = Transition {
            writes: Vec::new(),
            erase_before_write: Vec::new(),
            retire_after_publication: Vec::new(),
            prove_erased: Vec::new(),
            append: None,
            truncate: metadata_tail.then_some(Span {
                start: next.anchor.sealed_len,
                len: self
                    .anchor
                    .sealed_len
                    .saturating_sub(next.anchor.sealed_len),
            }),
        };
        if next.anchor.sealed_len > self.anchor.sealed_len {
            result.append = Some(Span {
                start: self.anchor.sealed_len,
                len: next.anchor.sealed_len - self.anchor.sealed_len,
            });
        }
        for claim in next.claims.values().filter(|claim| claim.kind.live()) {
            if self.claims.get(&claim.span.start) == Some(claim) {
                continue;
            }
            if claim.span.start < self.anchor.sealed_len {
                let previous = self.covering(claim.span)?;
                if !previous.kind.vacant() {
                    return Err(Error::CorruptRecord);
                }
                if previous.kind == Kind::Pending {
                    result.erase_before_write.push(claim.span);
                }
            }
            result.writes.push(claim.span);
        }
        for old in self.claims.values().filter(|claim| claim.kind.live()) {
            if next.claims.get(&old.span.start) == Some(old) {
                continue;
            }
            if metadata_tail
                && old.kind == Kind::Private
                && old.span.start >= next.anchor.sealed_len
            {
                result.retire_after_publication.push(old.span);
                continue;
            }
            if next.covering(old.span)?.kind != Kind::Pending {
                return Err(Error::CorruptRecord);
            }
            result.retire_after_publication.push(old.span);
        }
        // A free claim may merge several previous vacant claims, but cannot turn
        // live bytes directly into reusable storage or skip their erasure proof.
        for free in next
            .claims
            .values()
            .filter(|claim| claim.kind == Kind::Free)
        {
            let mut position = free.span.start;
            let end = free.span.end()?;
            while position < end.min(self.anchor.sealed_len) {
                let old = self.covering(Span {
                    start: position,
                    len: 1,
                })?;
                if !old.kind.vacant() {
                    return Err(Error::CorruptRecord);
                }
                let limit = old.span.end()?.min(end);
                if old.kind == Kind::Pending {
                    result.prove_erased.push(Span {
                        start: position,
                        len: limit - position,
                    });
                }
                position = limit;
            }
        }
        Ok(result)
    }
}
#[cfg(test)]
mod tests;
