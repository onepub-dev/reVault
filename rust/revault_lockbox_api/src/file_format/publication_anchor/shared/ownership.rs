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
    pub(crate) fn payloads(&self) -> Vec<Extent> {
        self.claims
            .values()
            .filter(|claim| claim.kind == Kind::Payload)
            .map(|claim| Extent {
                start: claim.span.start,
                len: claim.span.len,
                digest: claim.digest,
            })
            .collect()
    }
    /// Dense return-to-inline retains actual vacancy and retires every selected
    /// descendant. The caller separately retires/replaces the private root.
    pub(crate) fn catalogue_vacant(&self) -> Vec<Vacant> {
        self.claims
            .values()
            .filter_map(|claim| {
                let kind = match claim.kind {
                    Kind::Free => VacantKind::Free,
                    Kind::Pending | Kind::Descendant => VacantKind::Pending,
                    _ => return None,
                };
                Some(Vacant {
                    span: claim.span,
                    kind,
                })
            })
            .collect()
    }
    /// Build authenticated ownership records for a metadata-only rewrite. The
    /// caller first verifies reclaimed bytes. Payload and keys stay live; old
    /// private/descendant metadata becomes pending, never immediately reusable.
    pub(crate) fn rewrite_metadata_records(
        &self,
        reused: &[Span],
    ) -> Result<Vec<crate::file_format::authenticated_index::Entry>> {
        use crate::file_format::authenticated_index::Entry;
        let mut records = Vec::new();
        for claim in self.claims.values() {
            let kind = match claim.kind {
                Kind::Fixed | Kind::Keys => continue,
                Kind::Free
                    if claim.span.start < REGION_LEN as u64
                        && claim.span.start % FAILURE_REGION == KEYS_START =>
                {
                    continue
                }
                Kind::Free | Kind::Pending => 0,
                Kind::Private | Kind::Descendant => 1,
                Kind::Payload => 2,
            };
            let mut position = claim.span.start;
            let end = claim.span.end()?;
            let mut emit = |start: u64, limit: u64| -> Result<()> {
                if start < limit {
                    let key = [vec![kind], start.to_be_bytes().to_vec()].concat();
                    let mut value = (limit - start).to_le_bytes().to_vec();
                    if kind == 2 {
                        value.extend_from_slice(&claim.digest);
                    }
                    records.push(Entry::new(0, &key, &value)?);
                }
                Ok(())
            };
            for span in reused {
                let limit = span.end()?;
                if span.start >= end || limit <= position {
                    continue;
                }
                if !claim.kind.vacant() {
                    return Err(Error::CorruptRecord);
                }
                emit(position, span.start.min(end))?;
                position = limit.min(end);
            }
            emit(position, end)?;
        }
        Ok(records)
    }
    /// Select full failure regions whose every byte is authenticated vacant
    /// space. Keep original claim boundaries in the preparation reservations.
    pub(crate) fn reusable_regions(
        &self,
        maximum: usize,
    ) -> Result<(
        Vec<Span>,
        Vec<crate::file_format::preparation_journal::Reservation>,
    )> {
        use crate::file_format::preparation_journal::{Reservation, FREE, PENDING};
        let mut runs: Vec<Span> = Vec::new();
        for claim in self.claims.values().filter(|claim| claim.kind.vacant()) {
            if let Some(last) = runs
                .last_mut()
                .filter(|last| last.end().ok() == Some(claim.span.start))
            {
                last.len += claim.span.len;
            } else {
                runs.push(claim.span);
            }
        }
        let mut regions = Vec::new();
        let mut reservations = Vec::new();
        for run in runs {
            let Some(rounded) = run
                .start
                .max(REGION_LEN as u64)
                .checked_add(FAILURE_REGION - 1)
            else {
                continue;
            };
            let mut at = rounded / FAILURE_REGION * FAILURE_REGION;
            while at
                .checked_add(FAILURE_REGION)
                .is_some_and(|end| end <= run.start + run.len)
                && regions.len() < maximum
            {
                let mut position = at;
                let mut pieces = Vec::new();
                while position < at + FAILURE_REGION {
                    let claim = self.covering(Span {
                        start: position,
                        len: 1,
                    })?;
                    let end = claim.span.end()?.min(at + FAILURE_REGION);
                    pieces.push(Reservation {
                        namespace: if claim.kind == Kind::Pending {
                            PENDING
                        } else {
                            FREE
                        },
                        base: claim.span.start,
                        start: position,
                        len: end - position,
                    });
                    position = end;
                }
                if reservations.len() + pieces.len() > 2048 {
                    return Ok((regions, reservations));
                }
                reservations.extend(pieces);
                regions.push(Span {
                    start: at,
                    len: FAILURE_REGION,
                });
                at += FAILURE_REGION;
            }
        }
        self.validate_reservations(&reservations)?;
        Ok((regions, reservations))
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
    /// Reserve bounded payload slices within individual authenticated vacant
    /// claims. Do not merge ownership boundaries or consume metadata/arena slots.
    /// The selected graph must have independently verified these bytes erased.
    pub(crate) fn reserve_payloads(
        &self,
        lengths: &[usize],
        excluded: &mut Vec<Span>,
        reservations: &mut Vec<crate::file_format::preparation_journal::Reservation>,
    ) -> Result<Vec<Option<Span>>> {
        use crate::file_format::preparation_journal::{Reservation, FREE, PENDING};
        let mut result = Vec::with_capacity(lengths.len());
        excluded.sort_by_key(|span| span.start);
        for &length in lengths {
            if length == 0 {
                return Err(Error::CorruptRecord);
            }
            let length = length as u64;
            let mut best: Option<(u64, u64, &Claim)> = None;
            if reservations.len() < 2046 {
                for claim in self
                    .claims
                    .values()
                    .filter(|claim| claim.kind.vacant() && claim.span.start >= REGION_LEN as u64)
                {
                    let end = claim.span.end()?;
                    let mut position = claim.span.start;
                    let mut consider = |start: u64, limit: u64| {
                        if limit >= start && limit - start >= length {
                            let candidate = (limit - start - length, start, claim);
                            if best
                                .as_ref()
                                .is_none_or(|old| (candidate.0, candidate.1) < (old.0, old.1))
                            {
                                best = Some(candidate);
                            }
                        }
                    };
                    for span in excluded.iter() {
                        let limit = span.end()?;
                        if span.start >= end || limit <= position {
                            continue;
                        }
                        consider(position, span.start.min(end));
                        position = limit.min(end);
                    }
                    consider(position, end);
                }
            }
            if let Some((_, start, claim)) = best {
                let span = Span { start, len: length };
                reservations.push(Reservation {
                    namespace: if claim.kind == Kind::Pending {
                        PENDING
                    } else {
                        FREE
                    },
                    base: claim.span.start,
                    start,
                    len: length,
                });
                excluded.push(span);
                excluded.sort_by_key(|span| span.start);
                result.push(Some(span));
            } else {
                result.push(None);
            }
        }
        self.validate_reservations(reservations)?;
        Ok(result)
    }
    /// Authenticate every exception before checking reclaimed bytes. Only these
    /// reservations may contain abandoned writes; unrelated dirty vacancies are
    /// corruption, not additional erase authority.
    pub(crate) fn verify_abort_reservations(
        &self,
        storage: &impl Storage,
        reservations: &[crate::file_format::preparation_journal::Reservation],
    ) -> Result<Vec<Span>> {
        let spans = self.validate_reservations(reservations)?;
        for claim in self.claims.values().filter(|claim| claim.kind.vacant()) {
            let end = claim.span.end()?;
            let mut position = claim.span.start;
            for span in spans
                .iter()
                .filter(|span| span.start >= claim.span.start && span.start < end)
            {
                verify_zero(storage, position, span.start)?;
                position = span.end()?;
            }
            verify_zero(storage, position, end)?;
        }
        Ok(spans)
    }
    pub(crate) fn transition_to(&self, next: &Self) -> Result<Transition> {
        self.transition(next, false, false)
    }
    /// Narrow return-to-inline proof; payload and public-key claims cannot change.
    pub(crate) fn metadata_tail_transition_to(&self, next: &Self) -> Result<Transition> {
        self.metadata_tail_transition(next, false)
    }
    /// Explicit complete-tree retirement. Both graphs must be derived from
    /// authenticated membership; the replacement has no descendant tree.
    pub(crate) fn tree_tail_transition_to(&self, next: &Self) -> Result<Transition> {
        if next
            .claims
            .values()
            .any(|claim| claim.kind == Kind::Descendant)
        {
            return Err(Error::CorruptRecord);
        }
        self.metadata_tail_transition(next, true)
    }
    fn metadata_tail_transition(
        &self,
        next: &Self,
        retire_descendants: bool,
    ) -> Result<Transition> {
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
                .filter(|claim| {
                    matches!(claim.kind, Kind::Payload | Kind::Keys)
                        || (!retire_descendants && claim.kind == Kind::Descendant)
                })
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
            if !old.kind.vacant()
                && old.kind != Kind::Private
                && !(retire_descendants && old.kind == Kind::Descendant)
            {
                return Err(Error::CorruptRecord);
            }
        }
        self.transition(next, true, retire_descendants)
    }
    fn transition(
        &self,
        next: &Self,
        metadata_tail: bool,
        retire_descendants: bool,
    ) -> Result<Transition> {
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
                let end = claim.span.end()?;
                if end > self.anchor.sealed_len {
                    return Err(Error::CorruptRecord);
                }
                let mut position = claim.span.start;
                while position < end {
                    let previous = self.covering(Span {
                        start: position,
                        len: 1,
                    })?;
                    if !previous.kind.vacant() {
                        return Err(Error::CorruptRecord);
                    }
                    let limit = previous.span.end()?.min(end);
                    if previous.kind == Kind::Pending {
                        result.erase_before_write.push(Span {
                            start: position,
                            len: limit - position,
                        });
                    }
                    position = limit;
                }
            }
            result.writes.push(claim.span);
        }
        for old in self.claims.values().filter(|claim| claim.kind.live()) {
            if next.claims.get(&old.span.start) == Some(old) {
                continue;
            }
            if metadata_tail
                && (old.kind == Kind::Private
                    || (retire_descendants && old.kind == Kind::Descendant))
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
fn verify_zero(storage: &impl Storage, mut position: u64, end: u64) -> Result<()> {
    while position < end {
        let n = (end - position).min(65536) as usize;
        let bytes = ZeroizingBytes::new(storage.read_at(position, n)?);
        if bytes.len() != n || bytes.iter().any(|byte| *byte != 0) {
            return Err(Error::CorruptRecord);
        }
        position += n as u64;
    }
    Ok(())
}
#[cfg(test)]
mod tests;
