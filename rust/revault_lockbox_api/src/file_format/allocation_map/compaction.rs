//! Source-preserving relocation into an empty, private replacement. This entry
//! point currently accepts only candidate file records (no key/access tree).
use super::*;
use crate::page_buffer::ZeroizingBytes;

/// The caller holds the source stable and validates typed file/codec semantics.
/// Relative slice metadata survives relocation; only ownership-envelope physical
/// addresses change. Every unique pack is copied once after digest verification.
/// No publication from the source is overwritten. On error, clear the temporary
/// backend; failure of that cleanup is reported, never treated as successful copy.
pub(crate) fn relocate<S: Storage>(
    source: &impl Storage,
    anchor: &Anchor,
    mut destination: S,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
) -> Result<S> {
    if destination.len()? != 0 {
        return Err(Error::InvalidInput(
            "compaction requires an empty replacement".into(),
        ));
    }
    if anchor.keys != RootRef::default() {
        return Err(Error::InvalidInput(
            "candidate file compaction does not yet support access records".into(),
        ));
    }
    let result = (|| {
        let selected = publication::select(source, anchor.archive, anchor.mode, authority)?;
        if selected.anchor != *anchor {
            return Err(Error::CorruptRecord);
        }
        let index = Index::new(anchor.archive, anchor.mode, key)?;
        Snapshot::inspect(source, anchor, &index)?.verify_reclaimed(source)?;
        // As in create_empty, cloning is limited to the verified-empty backend.
        let mut writer = InitialPairs(destination.clone());
        append_zeros(&mut writer, DATA_START)?;
        let mut packs = BTreeMap::<u64, (Extent, Extent)>::new();
        let mut entries = Vec::new();
        index.visit(source, anchor.index, anchor.sealed_len, |entry| {
            let mut record = OwnedRecord::decode(&entry.value)?;
            for extent in &mut record.extents {
                if let Some((old, copied)) = packs.get(&extent.start) {
                    if old != extent {
                        return Err(Error::CorruptRecord);
                    }
                    *extent = *copied;
                    continue;
                }
                // Candidate file packs are bounded by the codec's largest padded
                // physical allocation. Refuse a forged size before allocating.
                if extent.len > 320 * 1024 {
                    return Err(Error::CorruptRecord);
                }
                let stored =
                    ZeroizingBytes::new(source.read_at(extent.start, extent.len as usize)?);
                if stored.len() != extent.len as usize || strong_checksum(&stored) != extent.digest
                {
                    return Err(Error::CorruptRecord);
                }
                let copied = Extent {
                    start: writer.append(&stored)?,
                    ..*extent
                };
                packs.insert(extent.start, (*extent, copied));
                *extent = copied;
            }
            entries.push(Entry::new(
                entry.namespace,
                &entry.key,
                &OwnedRecord::encode(&record.metadata, &record.extents)?,
            )?);
            Ok(())
        })?;
        super::super::candidate_files::compaction::checkpoint("payloads");
        let root = index
            .build_sorted(&mut writer, entries.into_iter().map(Ok))?
            .root;
        let mut next = Anchor {
            generation: anchor
                .generation
                .checked_add(1)
                .ok_or(Error::CorruptRecord)?,
            previous: anchor.commitment()?,
            index: root,
            object_root: root.digest,
            allocation: RootRef::default(),
            sealed_len: writer.len()?,
            ..anchor.clone()
        };
        let mut live = Snapshot::live(&writer, &next, &index)?;
        let mut free = fresh_gaps(&live, next.sealed_len)?;
        let arena_start = align_region(next.sealed_len)?;
        let capacity =
            index.fixed_record_node_bound(free.len() as u64 + 2, 8, 8)? * 2 * FAILURE_REGION;
        let arena = Extent {
            start: arena_start,
            len: capacity,
            digest: [0; 32],
        };
        append_zeros(&mut writer, arena_start - next.sealed_len + capacity)?;
        next.sealed_len = writer.len()?;
        live.insert(
            Claim {
                extent: arena,
                kind: Kind::Reserve,
            },
            next.sealed_len,
        )?;
        free = fresh_gaps(&live, next.sealed_len)?;
        free.push(Entry::new(
            ARENA,
            &arena.start.to_be_bytes(),
            &arena.len.to_le_bytes(),
        )?);
        free.sort_by(|a, b| (a.namespace, a.key.as_slice()).cmp(&(b.namespace, b.key.as_slice())));
        let mut writer = ArenaWriter {
            storage: writer.0,
            arena,
            position: Arc::new(Mutex::new(arena.start)),
        };
        next.allocation = index
            .build_sorted(&mut writer, free.into_iter().map(Ok))?
            .root;
        Snapshot::inspect(&writer, &next, &index)?.verify_reclaimed(&writer)?;
        super::super::candidate_files::compaction::checkpoint("dependencies");
        publication::publish_relocation(source, &mut writer.storage, &next, authority, signer)?;
        journal::initialize(
            &mut writer.storage,
            anchor.archive,
            anchor.mode,
            authority,
            key,
        )?;
        Ok(writer.storage)
    })();
    if let Err(error) = result {
        if let Err(cleanup) = discard(&mut destination) {
            return Err(Error::Io(format!(
                "compaction failed ({error}); replacement cleanup failed ({cleanup})"
            )));
        }
        return Err(error);
    }
    result
}
fn fresh_gaps(live: &Claims, sealed: u64) -> Result<Vec<Entry>> {
    let mut records = Vec::new();
    let mut position = 0;
    for claim in live.0.values() {
        if claim.extent.start < position {
            return Err(Error::CorruptRecord);
        }
        if claim.extent.start > position {
            records.push(Entry::new(
                FREE,
                &position.to_be_bytes(),
                &(claim.extent.start - position).to_le_bytes(),
            )?);
        }
        position = claim.extent.end()?;
    }
    if position > sealed {
        return Err(Error::CorruptRecord);
    }
    if position < sealed {
        records.push(Entry::new(
            FREE,
            &position.to_be_bytes(),
            &(sealed - position).to_le_bytes(),
        )?);
    }
    Ok(records)
}
pub(crate) fn discard(storage: &mut impl Storage) -> Result<()> {
    let len = storage.len()?;
    let zeros = [0u8; 65536];
    let mut position = 0;
    while position < len {
        let n = (len - position).min(zeros.len() as u64) as usize;
        storage.write_at(position, &zeros[..n])?;
        position += n as u64;
    }
    storage.sync()?;
    storage.truncate(0)?;
    storage.sync()
}

/// Fresh read-only component validation without cloning an entire memory archive.
#[derive(Clone, Debug)]
pub(crate) struct View<'a, S: Storage>(pub &'a S);
impl<S: Storage> Storage for View<'_, S> {
    fn len(&self) -> Result<u64> {
        self.0.len()
    }
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        self.0.read_at(offset, len)
    }
    fn read_at_into(&self, offset: u64, bytes: &mut [u8]) -> Result<()> {
        self.0.read_at_into(offset, bytes)
    }
    fn append(&mut self, _: &[u8]) -> Result<u64> {
        Err(Error::InvalidOperation("read-only validation view".into()))
    }
    fn write_at(&mut self, _: u64, _: &[u8]) -> Result<()> {
        Err(Error::InvalidOperation("read-only validation view".into()))
    }
    fn truncate(&mut self, _: u64) -> Result<()> {
        Err(Error::InvalidOperation("read-only validation view".into()))
    }
    fn sync(&self) -> Result<()> {
        Err(Error::InvalidOperation("read-only validation view".into()))
    }
}
