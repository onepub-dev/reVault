//! Inline transition executor for the bounded metadata-update experiment. A
//! caller must prove reservations against the selected ownership graph. This
//! does not implement the separate overflow-arena rotation protocol.
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
}
impl InlineSession {
    pub(crate) fn open(
        storage: &impl Storage,
        archive: LockboxId,
        mode: FormatMode,
        key: Option<&[u8]>,
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
        if !matches!(newest.contents, Contents::Inline(_)) {
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
        self.mirror(storage)?;
        let first = 1 - self.selected.slot;
        storage.write_at(first as u64 * FAILURE_REGION + STUB_OFFSET, &encoded.stub)?;
        storage.sync()?;
        storage.write_at(
            self.selected.slot as u64 * FAILURE_REGION + STUB_OFFSET,
            &encoded.stub,
        )?;
        storage.sync()?;
        self.selected = InlineSelected {
            record,
            encoded: encoded.stub,
            slot: first,
            copies: 3,
        };
        Ok(())
    }
}
