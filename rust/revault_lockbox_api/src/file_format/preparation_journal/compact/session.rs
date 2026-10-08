//! Preparation transition executors. Callers must prove reservations and cleanup
//! against the selected authenticated ownership graph. OverflowSession stages a
//! tail arena and unlinks it before caller-authorized erasure; archive mutation
//! and committed-graph cleanup remain the integrating caller's responsibility.
use super::*;
pub(crate) struct InlineSession {
    context: Context,
    selected: InlineSelected,
}
struct InlineSelected {
    record: Record,
    encoded: Vec<u8>,
    slot: usize,
    copies: u8,
    overflow: Option<RootRef>,
}
impl InlineSession {
    pub(crate) fn open(
        storage: &impl Storage,
        archive: LockboxId,
        mode: FormatMode,
        key: Option<&[u8]>,
    ) -> Result<Self> {
        Self::open_inner(storage, archive, mode, key, false)
    }
    fn open_inner(
        storage: &impl Storage,
        archive: LockboxId,
        mode: FormatMode,
        key: Option<&[u8]>,
        allow_overflow: bool,
    ) -> Result<Self> {
        let context = Context::new(archive, mode, key)?;
        let mut candidates = Vec::with_capacity(2);
        for slot in 0..2 {
            let encoded =
                storage.read_at(slot as u64 * FAILURE_REGION + STUB_OFFSET, STUB_BYTES)?;
            if let Ok(stub) = decode(&context, &encoded) {
                candidates.push((slot, encoded, stub));
            }
        }
        candidates.sort_by_key(|(_, _, stub)| stub.sequence);
        let (slot, encoded, newest) = candidates.pop().ok_or(Error::CorruptRecord)?;
        let mut copies = 1 << slot;
        if let Some((old_slot, _, old)) = candidates.pop() {
            if old.sequence == newest.sequence {
                if old.digest != newest.digest {
                    return Err(Error::CorruptRecord);
                }
                copies |= 1 << old_slot;
            } else if old.sequence.checked_add(1) != Some(newest.sequence)
                || newest.previous != old.digest
            {
                return Err(Error::CorruptRecord);
            }
        }
        let overflow = match &newest.contents {
            Contents::Inline(_) => None,
            Contents::Overflow(reference) => Some(*reference),
        };
        if overflow.is_some() && !allow_overflow {
            return Err(Error::SecurityLimitExceeded(
                "shared update needs overflow-arena integration".into(),
            ));
        }
        let record = materialize(&context, storage, newest)?;
        if record.cleanup_commit != [0; 32] || record.cleanup_bytes != 0 {
            return Err(Error::CorruptRecord);
        }
        Ok(Self {
            context,
            selected: InlineSelected {
                record,
                encoded,
                slot,
                copies,
                overflow,
            },
        })
    }
    pub(crate) fn base(&self) -> [u8; 32] {
        self.selected.record.base
    }
    pub(crate) fn active(&self) -> bool {
        self.selected.record.active
    }
    pub(crate) fn reservations(&self) -> &[Reservation] {
        &self.selected.record.reservations
    }
    pub(crate) fn require_idle(&self, base: [u8; 32]) -> Result<()> {
        if self.active() || self.base() != base {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }
    pub(crate) fn mirror(&self, storage: &mut impl Storage) -> Result<()> {
        if let Some(reference) = self.selected.overflow {
            // A journal reference alone cannot authorize writing its arena.
            // Validate availability, but leave physical repair to a caller with
            // selected-graph ownership proof. Only fixed control stubs are mirrored.
            reference.read_verified(storage)?;
        }
        storage.sync()?;
        if self.selected.copies != 3 {
            storage.write_at(
                (1 - self.selected.slot) as u64 * FAILURE_REGION + STUB_OFFSET,
                &self.selected.encoded,
            )?;
            storage.sync()?;
        }
        Ok(())
    }
    pub(crate) fn begin(
        &mut self,
        storage: &mut impl Storage,
        base: [u8; 32],
        reservations: Vec<Reservation>,
    ) -> Result<()> {
        self.require_idle(base)?;
        self.replace(storage, base, true, reservations)
    }
    pub(crate) fn finish(&mut self, storage: &mut impl Storage, base: [u8; 32]) -> Result<()> {
        if !self.active() {
            self.require_idle(base)?;
            return self.mirror(storage);
        }
        self.replace(storage, base, false, Vec::new())
    }
    fn replace(
        &mut self,
        storage: &mut impl Storage,
        base: [u8; 32],
        active: bool,
        reservations: Vec<Reservation>,
    ) -> Result<()> {
        if self.selected.overflow.is_some() {
            return Err(Error::InvalidOperation(
                "unlink preparation overflow before finishing".into(),
            ));
        }
        let record = Record {
            sequence: self
                .selected
                .record
                .sequence
                .checked_add(1)
                .ok_or(Error::CorruptRecord)?,
            previous: strong_checksum(&self.selected.encoded),
            base,
            active,
            cleanup_commit: [0; 32],
            cleanup_bytes: 0,
            reservations,
        };
        let encoded = encode(&self.context, &record, None)?;
        if encoded.overflow.is_some() {
            return Err(Error::CorruptRecord);
        }
        self.publish_encoded(storage, record, encoded.stub, None)
    }
    fn publish_encoded(
        &mut self,
        storage: &mut impl Storage,
        record: Record,
        encoded: Vec<u8>,
        overflow: Option<RootRef>,
    ) -> Result<()> {
        self.mirror(storage)?;
        let first = 1 - self.selected.slot;
        storage.write_at(first as u64 * FAILURE_REGION + STUB_OFFSET, &encoded)?;
        storage.sync()?;
        storage.write_at(
            self.selected.slot as u64 * FAILURE_REGION + STUB_OFFSET,
            &encoded,
        )?;
        storage.sync()?;
        self.selected = InlineSelected {
            record,
            encoded,
            slot: first,
            copies: 3,
            overflow,
        };
        Ok(())
    }
}

/// Opt-in executor: inline-only archive recovery continues to reject overflow.
/// Callers hold exclusive ownership and authenticate the base graph themselves.
pub(crate) struct OverflowSession(InlineSession);
impl OverflowSession {
    pub(crate) fn open(
        storage: &impl Storage,
        archive: LockboxId,
        mode: FormatMode,
        key: Option<&[u8]>,
    ) -> Result<Self> {
        Ok(Self(InlineSession::open_inner(
            storage, archive, mode, key, true,
        )?))
    }
    pub(crate) fn active(&self) -> bool {
        self.0.active()
    }
    pub(crate) fn base(&self) -> [u8; 32] {
        self.0.base()
    }
    pub(crate) fn reservations(&self) -> &[Reservation] {
        self.0.reservations()
    }
    pub(crate) fn arena(&self) -> Option<RootRef> {
        self.0.selected.overflow
    }
    pub(crate) fn mirror(&self, storage: &mut impl Storage) -> Result<()> {
        self.0.mirror(storage)
    }

    /// The caller has authenticated this sealed base and validated every reusable
    /// reservation against its graph. An initial durable inline active stub owns
    /// the unpublished append tail before either overflow copy is written.
    pub(crate) fn begin(
        &mut self,
        storage: &mut impl Storage,
        base: [u8; 32],
        sealed: u64,
        reservations: Vec<Reservation>,
    ) -> Result<()> {
        self.0.require_idle(base)?;
        if storage.len()? != sealed || sealed < REGION_LEN as u64 {
            return Err(Error::CorruptRecord);
        }
        let primary = sealed
            .checked_add(FAILURE_REGION - 1)
            .ok_or(Error::CorruptRecord)?
            / FAILURE_REGION
            * FAILURE_REGION;
        let mirror = primary
            .checked_add(FAILURE_REGION)
            .ok_or(Error::CorruptRecord)?;
        mirror
            .checked_add(FAILURE_REGION)
            .ok_or(Error::CorruptRecord)?;
        // Validate capacity before touching storage. This explicit entry point
        // does not silently allocate arenas for records that fit inline.
        let mut record = Record {
            sequence: self
                .0
                .selected
                .record
                .sequence
                .checked_add(2)
                .ok_or(Error::CorruptRecord)?,
            previous: [1; 32],
            base,
            active: true,
            cleanup_commit: [0; 32],
            cleanup_bytes: 0,
            reservations,
        };
        encode(&self.0.context, &record, Some((primary, mirror)))?;
        self.0.begin(storage, base, Vec::new())?;
        record.previous = strong_checksum(&self.0.selected.encoded);
        let encoded = encode(&self.0.context, &record, Some((primary, mirror)))?;
        let bytes = encoded.overflow.ok_or(Error::CorruptRecord)?;
        if storage.len()? != sealed {
            return Err(Error::CorruptRecord);
        }
        if primary > sealed {
            storage.append(&vec![0; (primary - sealed) as usize])?;
        }
        if storage.append(&bytes)? != primary || storage.append(&bytes)? != mirror {
            return Err(Error::CorruptRecord);
        }
        storage.sync()?;
        let reference = RootRef {
            primary,
            mirror,
            len: bytes.len() as u64,
            digest: strong_checksum(&bytes),
        };
        for offset in [primary, mirror] {
            if strong_checksum(&storage.read_at(offset, bytes.len())?) != reference.digest {
                return Err(Error::CorruptRecord);
            }
        }
        self.0
            .publish_encoded(storage, record, encoded.stub, Some(reference))
    }

    /// The caller has completed all graph-authorized cleanup except the arena.
    /// For abort, reservations must already be erased; for committed changes,
    /// selected pending claims must be erased except these arena copies. The
    /// arena must remain owned by the unsealed tail or selected pending claims.
    /// Publish BOTH empty active stubs before the caller erases the old arena.
    pub(crate) fn unlink_after_cleanup(&mut self, storage: &mut impl Storage) -> Result<RootRef> {
        self.unlink_with_reservations(storage, Vec::new())
    }
    /// Reused arena allocations remain reserved in the small inline record after
    /// unlink. Interrupted erasure must not lose their in-sealed cleanup authority.
    pub(crate) fn unlink_reused_after_cleanup(
        &mut self,
        storage: &mut impl Storage,
        arena: Vec<Reservation>,
    ) -> Result<RootRef> {
        let reference = self.arena().ok_or(Error::CorruptRecord)?;
        if arena.len() != 2
            || ![reference.primary, reference.mirror]
                .into_iter()
                .all(|start| {
                    arena.iter().any(|range| {
                        range.start == start
                            && range.len == reference.len
                            && self.reservations().contains(range)
                    })
                })
        {
            return Err(Error::CorruptRecord);
        }
        self.unlink_with_reservations(storage, arena)
    }
    fn unlink_with_reservations(
        &mut self,
        storage: &mut impl Storage,
        reservations: Vec<Reservation>,
    ) -> Result<RootRef> {
        let reference = self
            .arena()
            .ok_or_else(|| Error::InvalidOperation("no preparation overflow arena".into()))?;
        if !self.active() {
            return Err(Error::CorruptRecord);
        }
        let record = Record {
            sequence: self
                .0
                .selected
                .record
                .sequence
                .checked_add(1)
                .ok_or(Error::CorruptRecord)?,
            previous: strong_checksum(&self.0.selected.encoded),
            base: self.base(),
            active: true,
            cleanup_commit: [0; 32],
            cleanup_bytes: 0,
            reservations,
        };
        let encoded = encode(&self.0.context, &record, None)?;
        self.0
            .publish_encoded(storage, record, encoded.stub, None)?;
        Ok(reference)
    }
    /// Caller proves both arena reservations and the full reservation set against
    /// the selected graph. Publish a SMALL inline preparation for the arena first;
    /// thus even a torn first arena write has durable cleanup ownership.
    pub(crate) fn begin_reused(
        &mut self,
        storage: &mut impl Storage,
        base: [u8; 32],
        sealed: u64,
        arena: [Reservation; 2],
        reservations: Vec<Reservation>,
    ) -> Result<()> {
        self.0.require_idle(base)?;
        if storage.len()? != sealed
            || arena[0].start == arena[1].start
            || arena.iter().any(|range| {
                range.len != FAILURE_REGION
                    || range
                        .start
                        .checked_add(range.len)
                        .is_none_or(|end| end > sealed)
                    || !reservations.contains(range)
            })
        {
            return Err(Error::CorruptRecord);
        }
        let mut record = Record {
            sequence: self
                .0
                .selected
                .record
                .sequence
                .checked_add(2)
                .ok_or(Error::CorruptRecord)?,
            previous: [1; 32],
            base,
            active: true,
            cleanup_commit: [0; 32],
            cleanup_bytes: 0,
            reservations,
        };
        let locations = (arena[0].start, arena[1].start);
        encode(&self.0.context, &record, Some(locations))?;
        self.0.begin(storage, base, arena.to_vec())?;
        record.previous = strong_checksum(&self.0.selected.encoded);
        let encoded = encode(&self.0.context, &record, Some(locations))?;
        let bytes = encoded.overflow.ok_or(Error::CorruptRecord)?;
        let reference = RootRef {
            primary: locations.0,
            mirror: locations.1,
            len: bytes.len() as u64,
            digest: strong_checksum(&bytes),
        };
        for start in [reference.primary, reference.mirror] {
            storage.write_at(start, &bytes)?;
        }
        storage.sync()?;
        for start in [reference.primary, reference.mirror] {
            if strong_checksum(&storage.read_at(start, bytes.len())?) != reference.digest {
                return Err(Error::CorruptRecord);
            }
        }
        self.0
            .publish_encoded(storage, record, encoded.stub, Some(reference))
    }
    /// Caller has completed erasure/truncation against the selected graph.
    pub(crate) fn finish(&mut self, storage: &mut impl Storage, base: [u8; 32]) -> Result<()> {
        self.0.finish(storage, base)
    }
}
