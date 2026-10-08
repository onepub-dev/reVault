//! Bounded record transition writer. Raw rewrites preserve payload/key ownership;
//! a typed adapter may stage new payloads and retire exact old payload extents.
use super::*;
use crate::file_format::preparation_journal::compact::session::{InlineSession, OverflowSession};

pub(super) const MAX_APPEND: u64 =
    (2 * MAX_PAGES as u64 + 5) * FAILURE_REGION + MAX_PAYLOAD_BYTES as u64;
const MAX_RECORD_BYTES: usize = 64 * 1024 * 1024;
const MAX_PAYLOAD_BYTES: usize = 64 * 1024 * 1024;
type RebindRecords = Box<dyn FnOnce(&[Extent]) -> Result<Vec<Entry>>>;
/// Internal bounded staging plan. Rebinding runs before any persistent write;
/// physical ownership is independently checked against the selected old graph.
pub(crate) struct PayloadPlan {
    pub base: [u8; 32],
    pub retired: Vec<Extent>,
    pub bytes: Vec<crate::page_buffer::ZeroizingBytes>,
    pub rebind: RebindRecords,
}

/// Guarded staging for payloads which can contain plaintext secret values.
pub(crate) struct SecurePayloadPlan {
    pub base: [u8; 32],
    pub retired: Vec<Extent>,
    pub bytes: Vec<crate::secret_vec::SecureVec>,
    pub rebind: RebindRecords,
}
/// Prepared page commitments admit every extent and rebound record before any
/// persistent write. The callback regenerates raw guarded payload only; private
/// prepared-page state verifies its fingerprint before reproducing ciphertext.
pub(crate) struct PreparedSecurePayloadPlan<'a> {
    pub base: [u8; 32],
    pub retired: Vec<Extent>,
    pub pages: crate::file_format::page::secure_storage::PreparedSecurePages,
    pub payload: Box<dyn FnMut(usize) -> Result<crate::secret_vec::SecureVec> + 'a>,
    pub rebind: RebindRecords,
}
enum StagingBytes<'a> {
    Ready(Vec<PayloadBytes>),
    Prepared {
        pages: crate::file_format::page::secure_storage::PreparedSecurePages,
        payload: Box<dyn FnMut(usize) -> Result<crate::secret_vec::SecureVec> + 'a>,
    },
}
impl StagingBytes<'_> {
    fn count(&self) -> usize {
        match self {
            Self::Ready(bytes) => bytes.len(),
            Self::Prepared { pages, .. } => pages.len(),
        }
    }
    fn size(&self, index: usize) -> Result<usize> {
        match self {
            Self::Ready(bytes) => Ok(bytes.get(index).ok_or(Error::CorruptRecord)?.len()),
            Self::Prepared { pages, .. } => pages.size(index),
        }
    }
    fn digest(&self, index: usize) -> Result<[u8; 32]> {
        match self {
            Self::Ready(bytes) => bytes
                .get(index)
                .ok_or(Error::CorruptRecord)?
                .with_bytes(strong_checksum),
            Self::Prepared { pages, .. } => pages.digest(index),
        }
    }
    fn with_page(
        &mut self,
        index: usize,
        f: impl FnOnce(&PayloadBytes) -> Result<()>,
    ) -> Result<()> {
        match self {
            Self::Ready(bytes) => f(bytes.get(index).ok_or(Error::CorruptRecord)?),
            Self::Prepared { pages, payload } => {
                let raw = payload(index)?;
                let encoded = pages.render(index, &raw)?;
                drop(raw);
                f(&PayloadBytes::Secure(encoded))
            }
        }
    }
}
enum PayloadBytes {
    Ordinary(crate::page_buffer::ZeroizingBytes),
    Secure(crate::secret_vec::SecureVec),
}
impl PayloadBytes {
    fn len(&self) -> usize {
        match self {
            Self::Ordinary(v) => v.len(),
            Self::Secure(v) => v.len(),
        }
    }
    fn with_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Result<R> {
        match self {
            Self::Ordinary(v) => Ok(f(v)),
            Self::Secure(v) => Ok(v.with_bytes(f)?),
        }
    }
    fn verify(&self, storage: &impl Storage, extent: Extent) -> Result<()> {
        let digest = match self {
            Self::Ordinary(_) => {
                let read = crate::page_buffer::ZeroizingBytes::new(
                    storage.read_at(extent.start, self.len())?,
                );
                if read.len() != self.len() {
                    return Err(Error::CorruptRecord);
                }
                strong_checksum(&read)
            }
            Self::Secure(_) => {
                let read = storage.read_at_secure(extent.start, self.len())?;
                if read.len() != self.len() {
                    return Err(Error::CorruptRecord);
                }
                read.with_bytes(strong_checksum)?
            }
        };
        if digest != extent.digest {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }
}
struct StagingPlan<'a> {
    base: [u8; 32],
    retired: Vec<Extent>,
    bytes: StagingBytes<'a>,
    rebind: RebindRecords,
}
impl From<PayloadPlan> for StagingPlan<'_> {
    fn from(plan: PayloadPlan) -> Self {
        Self {
            base: plan.base,
            retired: plan.retired,
            bytes: StagingBytes::Ready(
                plan.bytes.into_iter().map(PayloadBytes::Ordinary).collect(),
            ),
            rebind: plan.rebind,
        }
    }
}
impl From<SecurePayloadPlan> for StagingPlan<'_> {
    fn from(plan: SecurePayloadPlan) -> Self {
        Self {
            base: plan.base,
            retired: plan.retired,
            bytes: StagingBytes::Ready(plan.bytes.into_iter().map(PayloadBytes::Secure).collect()),
            rebind: plan.rebind,
        }
    }
}
impl<'a> From<PreparedSecurePayloadPlan<'a>> for StagingPlan<'a> {
    fn from(plan: PreparedSecurePayloadPlan<'a>) -> Self {
        Self {
            base: plan.base,
            retired: plan.retired,
            bytes: StagingBytes::Prepared {
                pages: plan.pages,
                payload: plan.payload,
            },
            rebind: plan.rebind,
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn rewrite_prepared_secure_payload_records(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    records: Vec<Entry>,
    payloads: PreparedSecurePayloadPlan<'_>,
) -> Result<bool> {
    rewrite_inner(
        storage,
        archive,
        mode,
        authority,
        signer,
        key,
        records,
        false,
        true,
        false,
        Some(payloads.into()),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn rewrite_secure_payload_records(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    records: Vec<Entry>,
    payloads: SecurePayloadPlan,
) -> Result<bool> {
    rewrite_inner(
        storage,
        archive,
        mode,
        authority,
        signer,
        key,
        records,
        false,
        true,
        false,
        Some(payloads.into()),
    )
}

/// Resume the actual selected generation, never a caller-selected old snapshot.
pub(crate) fn recover_update(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<()> {
    let (anchor, _) = snapshot(storage, archive, mode, authority, key)?;
    let session = OverflowSession::open(storage, archive, mode, key)?;
    if session.active() && session.base() == commitment(&anchor)? {
        super::abort::recover_writer_abort(storage, archive, mode, authority, key)
    } else {
        super::commit::recover_writer_commit(storage, archive, mode, authority, key)
    }
}

/// Replace the ordered raw application record set. Caller retains exclusive
/// access and recovers any interrupted attempt before calling again. The 64 MiB
/// input bound is experimental admission, not a changed public value capacity.
#[allow(clippy::too_many_arguments)]
pub(crate) fn rewrite_records(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    records: Vec<Entry>,
) -> Result<bool> {
    rewrite_inner(
        storage, archive, mode, authority, signer, key, records, false, false, false, None,
    )
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn rewrite_payload_records(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    records: Vec<Entry>,
    payloads: PayloadPlan,
) -> Result<bool> {
    rewrite_inner(
        storage,
        archive,
        mode,
        authority,
        signer,
        key,
        records,
        false,
        true,
        false,
        Some(payloads.into()),
    )
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn grow_dense_payload_records(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    records: Vec<Entry>,
    payloads: PayloadPlan,
) -> Result<bool> {
    rewrite_inner(
        storage,
        archive,
        mode,
        authority,
        signer,
        key,
        records,
        true,
        true,
        false,
        Some(payloads.into()),
    )
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn grow_dense_records(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    records: Vec<Entry>,
) -> Result<bool> {
    rewrite_inner(
        storage, archive, mode, authority, signer, key, records, true, true, false, None,
    )
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn relocate_records(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    records: Vec<Entry>,
) -> Result<bool> {
    rewrite_inner(
        storage, archive, mode, authority, signer, key, records, false, true, true, None,
    )
}
#[allow(clippy::too_many_arguments)]
fn rewrite_inner(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    mut records: Vec<Entry>,
    dense: bool,
    force: bool,
    append_private: bool,
    payloads: Option<StagingPlan<'_>>,
) -> Result<bool> {
    let (old, dense_records) = if dense {
        let (anchor, graph) = crate::file_format::candidate_files::tree_image::dense_base(
            storage, archive, mode, authority, key,
        )?;
        (
            Tree {
                anchor,
                graph,
                index: Index::new(archive, mode, key)?,
                root: RootRef::default(),
                pages: Vec::new(),
            },
            Some(Vec::new()),
        )
    } else {
        (Tree::open(storage, archive, mode, authority, key)?, None)
    };
    let sealed = old.anchor.sealed_len;
    let aligned = sealed
        .checked_add(FAILURE_REGION - 1)
        .ok_or(Error::CorruptRecord)?
        / FAILURE_REGION
        * FAILURE_REGION;
    if let Some(plan) = &payloads {
        let live = old.graph.payloads();
        let mut starts = std::collections::BTreeSet::new();
        let mut total = 0usize;
        if plan.base != commitment(&old.anchor)? {
            return Err(Error::CorruptRecord);
        }
        for extent in &plan.retired {
            if !starts.insert(extent.start) || !live.contains(extent) {
                return Err(Error::CorruptRecord);
            }
        }
        for index in 0..plan.bytes.count() {
            let length = plan.bytes.size(index)?;
            total = total.checked_add(length).ok_or(Error::CorruptRecord)?;
            if length == 0 || length > 320 * 1024 || total > MAX_PAYLOAD_BYTES {
                return Err(Error::SecurityLimitExceeded(
                    "bounded payload staging".into(),
                ));
            }
        }
    }
    let mut size = 0usize;
    for entry in &records {
        Entry::new(entry.namespace, &entry.key, &entry.value)?;
        size = size
            .checked_add(entry.key.len() + entry.value.len() + 7)
            .ok_or(Error::CorruptRecord)?;
        if entry.namespace == OWNERSHIP || size > MAX_RECORD_BYTES {
            return Err(Error::InvalidInput(
                "bounded raw application records required".into(),
            ));
        }
    }
    if records.windows(2).any(|pair| {
        (pair[0].namespace, pair[0].key.as_slice()) >= (pair[1].namespace, pair[1].key.as_slice())
    }) {
        return Err(Error::InvalidInput(
            "records must be strictly ordered".into(),
        ));
    }
    // Validate signing authority before even repairing an existing copy.
    encode_in(&old.anchor, authority, signer, Layout::Shared)?;
    let mut position = 0usize;
    let mut equal = true;
    let mut compare = |entry: &Entry| {
        equal &= records.get(position).is_some_and(|next| {
            next.namespace == entry.namespace && next.key == entry.key && next.value == entry.value
        });
        position += 1;
        Ok(())
    };
    if let Some(entries) = dense_records {
        for entry in &entries {
            compare(entry)?;
        }
    } else {
        old.visit(storage, |entry| compare(&entry))?;
    }
    if !force && equal && position == records.len() {
        return Ok(false);
    }
    // Every build needs at least this many nodes regardless of ownership rows.
    // Reserve only a guaranteed-used prefix, avoiding ownership/allocation cycles.
    let leaves = size
        .div_ceil(FAILURE_REGION as usize)
        .max(records.len().div_ceil(1024))
        .max(1);
    let minimum_nodes = leaves + usize::from(leaves > 1);
    let reserved_nodes = 2 * minimum_nodes + if append_private { 0 } else { 2 };
    let (mut reused, mut reservations) = old.graph.reusable_regions(reserved_nodes)?;
    let payload_slots = if let Some(plan) = &payloads {
        old.graph.reserve_payloads(
            &(0..plan.bytes.count())
                .map(|index| plan.bytes.size(index))
                .collect::<Result<Vec<_>>>()?,
            &mut reused,
            &mut reservations,
        )?
    } else {
        Vec::new()
    };
    let payload_spans: Vec<_> = payload_slots.iter().flatten().copied().collect();
    old.graph.validate_reservations(&reservations)?;
    let overflow = crate::file_format::preparation_journal::compact::requires_overflow(
        archive,
        mode,
        key,
        reservations.len(),
    )?;
    let mut arena = None;
    if overflow {
        let (candidates, ranges) = old.graph.reusable_regions(2048)?;
        let copies: Vec<_> = candidates
            .iter()
            .filter(|span| {
                payloads.is_none()
                    || reused.iter().all(|used| {
                        span.start + span.len <= used.start || used.start + used.len <= span.start
                    })
            })
            .filter_map(|span| {
                ranges
                    .iter()
                    .find(|range| range.start == span.start && range.len == span.len)
                    .copied()
            })
            .take(2)
            .collect();
        if copies.len() == 2 && (payloads.is_none() || reservations.len() <= 2046) {
            arena = Some([copies[0], copies[1]]);
            if payloads.is_none() {
                reused = candidates
                    .into_iter()
                    .filter(|span| copies.iter().all(|range| range.start != span.start))
                    .take(reserved_nodes)
                    .collect();
            }
            reused.extend(copies.iter().map(|range| Span {
                start: range.start,
                len: range.len,
            }));
            reused.sort_by_key(|span| span.start);
            if payloads.is_none() {
                reservations = ranges
                    .into_iter()
                    .filter(|range| {
                        reused.iter().any(|span| {
                            range.start >= span.start
                                && range.start + range.len <= span.start + span.len
                        })
                    })
                    .collect();
            } else {
                reservations.extend(copies);
            }
        }
    }
    old.graph.validate_reservations(&reservations)?;
    let mut ownership = old.graph.rewrite_metadata_records(&reused)?;
    if let Some(plan) = &payloads {
        for extent in &plan.retired {
            let key = [vec![2], extent.start.to_be_bytes().to_vec()].concat();
            let row = ownership
                .iter_mut()
                .find(|row| row.key.as_slice() == key)
                .ok_or(Error::CorruptRecord)?;
            row.key[0] = 1;
            row.value.truncate(8);
        }
    }
    if aligned > sealed {
        ownership.push(Entry::new(
            OWNERSHIP,
            &[vec![0], sealed.to_be_bytes().to_vec()].concat(),
            &(aligned - sealed).to_le_bytes(),
        )?);
    }
    if ownership.len() + 2 + if overflow { 2 } else { 0 } > MAX_OWNERSHIP_RECORDS {
        return Err(Error::SecurityLimitExceeded(
            "rewrite ownership bound".into(),
        ));
    }
    let mut tail = aligned
        .checked_add(if overflow && arena.is_none() {
            2 * FAILURE_REGION
        } else {
            0
        })
        .ok_or(Error::CorruptRecord)?;
    let payload_start = tail;
    let (staged, mut staged_bytes) = if let Some(plan) = payloads {
        let mut extents = Vec::new();
        for (index, slot) in payload_slots.iter().enumerate() {
            let extent = Extent {
                start: slot.map_or(tail, |span| span.start),
                len: plan.bytes.size(index)? as u64,
                digest: plan.bytes.digest(index)?,
            };
            if slot.is_none() {
                tail = tail.checked_add(extent.len).ok_or(Error::CorruptRecord)?;
            }
            ownership.push(Entry::new(
                OWNERSHIP,
                &[vec![2], extent.start.to_be_bytes().to_vec()].concat(),
                &[extent.len.to_le_bytes().to_vec(), extent.digest.to_vec()].concat(),
            )?);
            extents.push(extent);
        }
        let rebound = (plan.rebind)(&extents)?;
        for row in &rebound {
            Entry::new(row.namespace, &row.key, &row.value)?;
        }
        let rebound_size = rebound.iter().try_fold(0usize, |sum, row| {
            sum.checked_add(row.key.len() + row.value.len() + 7)
                .ok_or(Error::CorruptRecord)
        })?;
        if rebound.len() != records.len()
            || rebound_size != size
            || rebound.iter().any(|row| row.namespace == OWNERSHIP)
            || rebound.windows(2).any(|pair| {
                (pair[0].namespace, pair[0].key.as_slice())
                    >= (pair[1].namespace, pair[1].key.as_slice())
            })
        {
            return Err(Error::CorruptRecord);
        }
        records = rebound;
        (extents, plan.bytes)
    } else {
        (Vec::new(), StagingBytes::Ready(Vec::new()))
    };
    let payload_end = tail;
    tail = tail
        .checked_add(FAILURE_REGION - 1)
        .ok_or(Error::CorruptRecord)?
        / FAILURE_REGION
        * FAILURE_REGION;
    if tail > payload_end {
        ownership.push(Entry::new(
            OWNERSHIP,
            &[vec![0], payload_end.to_be_bytes().to_vec()].concat(),
            &(tail - payload_end).to_le_bytes(),
        )?);
    }
    if ownership.len() + 2 + if overflow { 2 } else { 0 } > MAX_OWNERSHIP_RECORDS {
        return Err(Error::SecurityLimitExceeded(
            "payload ownership bound".into(),
        ));
    }
    let mut slots: std::collections::VecDeque<u64> = reused
        .iter()
        .filter(|span| {
            !payload_spans.contains(span)
                && arena.is_none_or(|copies| copies.iter().all(|range| range.start != span.start))
        })
        .map(|span| span.start)
        .collect();
    let mut private_slot = || -> Result<u64> {
        if !append_private {
            if let Some(start) = slots.pop_front() {
                return Ok(start);
            }
        }
        let start = tail;
        tail = tail
            .checked_add(FAILURE_REGION)
            .ok_or(Error::CorruptRecord)?;
        Ok(start)
    };
    let primary = private_slot()?;
    let mirror = private_slot()?;
    let bound = sealed.checked_add(MAX_APPEND).ok_or(Error::CorruptRecord)?;
    if overflow {
        for start in arena.map_or([aligned, aligned + FAILURE_REGION], |copies| {
            [copies[0].start, copies[1].start]
        }) {
            ownership.push(Entry::new(
                OWNERSHIP,
                &[vec![1], start.to_be_bytes().to_vec()].concat(),
                &FAILURE_REGION.to_le_bytes(),
            )?);
        }
    }
    for start in [primary, mirror] {
        ownership.push(Entry::new(
            OWNERSHIP,
            &[
                vec![0],
                (start + PRIVATE_BYTES as u64).to_be_bytes().to_vec(),
            ]
            .concat(),
            &(FAILURE_REGION - PRIVATE_BYTES as u64).to_le_bytes(),
        )?);
    }
    ownership.sort_by(|a, b| a.key.cmp(&b.key));
    old.mirror_pages(storage)?;
    let base = commitment(&old.anchor)?;
    ensure_mirrored(storage, archive, mode, authority, base)?;
    if overflow {
        let mut journal = OverflowSession::open(storage, archive, mode, key)?;
        if let Some(arena) = arena {
            journal.begin_reused(storage, base, sealed, arena, reservations)?;
        } else {
            journal.begin(storage, base, sealed, reservations)?;
        }
    } else {
        InlineSession::open(storage, archive, mode, key)?.begin(storage, base, reservations)?;
    }
    let change = {
        let mut append = Append {
            storage: std::rc::Rc::new(std::cell::RefCell::new(&mut *storage)),
            bound,
            reused: std::rc::Rc::new(std::cell::RefCell::new(slots)),
        };
        if append.len()? < aligned {
            append.append(&vec![0; (aligned - append.len()?) as usize])?;
        }
        if append.len()? != payload_start {
            return Err(Error::CorruptRecord);
        }
        for (index, extent) in staged.iter().enumerate() {
            staged_bytes.with_page(index, |bytes| {
                // Verify source reproduction before touching its reserved extent.
                if bytes.len() as u64 != extent.len
                    || bytes.with_bytes(strong_checksum)? != extent.digest
                {
                    return Err(Error::CorruptRecord);
                }
                bytes.with_bytes(|slice| -> Result<()> {
                    if extent.start < sealed {
                        append.storage.borrow_mut().write_at(extent.start, slice)?;
                    } else if append.append(slice)? != extent.start {
                        return Err(Error::CorruptRecord);
                    }
                    Ok(())
                })??;
                bytes.verify(&append, *extent)
            })?;
        }
        let padded = payload_end.div_ceil(FAILURE_REGION) * FAILURE_REGION;
        if padded > payload_end {
            append.append(&vec![0; (padded - payload_end) as usize])?;
        }
        for expected in [primary, mirror] {
            if expected >= sealed && append.append(&vec![0; FAILURE_REGION as usize])? != expected {
                return Err(Error::CorruptRecord);
            }
        }
        let change = old
            .index
            .build_sorted(&mut append, ownership.into_iter().chain(records).map(Ok))?;
        if !append.reused.borrow().is_empty() {
            return Err(Error::CorruptRecord);
        }
        change
    };
    let body = manifest(change.root);
    let private = encode_private(archive, mode, key, &body)?;
    for start in [primary, mirror] {
        storage.write_at(start, &private)?;
    }
    let root = RootRef {
        primary,
        mirror,
        len: private.len() as u64,
        digest: strong_checksum(&private),
    };
    let next = Anchor {
        generation: old
            .anchor
            .generation
            .checked_add(1)
            .ok_or(Error::CorruptRecord)?,
        previous: base,
        sealed_len: storage.len()?,
        object_root: root.digest,
        index: root,
        ..old.anchor.clone()
    };
    let selected = Tree::from_snapshot(storage, archive, mode, key, next.clone(), &body)?;
    old.graph.transition_to(&selected.graph)?;
    selected.graph.verify_free(storage)?;
    selected.mirror_pages(storage)?;
    let prepared = prepare(&next, authority, signer)?;
    publish(storage, &prepared, authority, base)?;
    super::commit::recover_writer_commit(storage, archive, mode, authority, key)?;
    Ok(true)
}

#[derive(Clone, Debug)]
struct Append<'a, S> {
    storage: std::rc::Rc<std::cell::RefCell<&'a mut S>>,
    bound: u64,
    reused: std::rc::Rc<std::cell::RefCell<std::collections::VecDeque<u64>>>,
}
impl<S: Storage> Storage for Append<'_, S> {
    fn len(&self) -> Result<u64> {
        self.storage.borrow().len()
    }
    fn read_at(&self, at: u64, len: usize) -> Result<Vec<u8>> {
        self.storage.borrow().read_at(at, len)
    }
    fn read_at_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        self.storage.borrow().read_at_into(at, out)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        if self
            .len()?
            .checked_add(bytes.len() as u64)
            .is_none_or(|end| end > self.bound)
        {
            return Err(Error::SecurityLimitExceeded(
                "bounded rewrite append tail".into(),
            ));
        }
        self.storage.borrow_mut().append(bytes)
    }
    fn append_pair(&mut self, bytes: &[u8]) -> Result<(u64, u64)> {
        if bytes.len() > FAILURE_REGION as usize || self.len()? % FAILURE_REGION != 0 {
            return Err(Error::CorruptRecord);
        }
        let mut offsets = [0; 2];
        for offset in &mut offsets {
            let reused = self.reused.borrow_mut().pop_front();
            *offset = if let Some(start) = reused {
                self.storage.borrow_mut().write_at(start, bytes)?;
                start
            } else {
                let start = self.append(bytes)?;
                self.append(&vec![0; FAILURE_REGION as usize - bytes.len()])?;
                start
            };
        }
        Ok((offsets[0], offsets[1]))
    }
    fn write_at(&mut self, _: u64, _: &[u8]) -> Result<()> {
        Err(Error::InvalidOperation(
            "append-only rewrite staging".into(),
        ))
    }
    fn truncate(&mut self, _: u64) -> Result<()> {
        Err(Error::InvalidOperation(
            "append-only rewrite staging".into(),
        ))
    }
    fn sync(&self) -> Result<()> {
        self.storage.borrow().sync()
    }
}
