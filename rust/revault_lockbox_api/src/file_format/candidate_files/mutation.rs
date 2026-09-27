//! Whole-pack retirement for logical file deletion. No public CLI emits C yet.
use super::*;
use crate::file_format::allocation_map::Extent;
use std::collections::BTreeSet;

struct Affected {
    extent: Extent,
    survivors: Vec<(Entry, Slice)>,
}
impl<S: Storage> Files<S> {
    /// Delete files as one transaction. Shared packs are rewritten from only
    /// surviving slices; their complete old allocations are then reclaimed.
    /// Missing paths are a no-change repeat, with no publication or allocation.
    pub(crate) fn remove(
        storage: S,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        signer: Option<&OwnerSigningKeyPair>,
        key: Option<&[u8]>,
        paths: impl IntoIterator<Item = Vec<u8>>,
    ) -> Result<S> {
        let files = Self::open(storage, archive, mode, authority, key)?;
        let mut removed = BTreeSet::new();
        for (count, path) in paths.into_iter().enumerate() {
            let path = Zeroizing::new(path);
            if count >= MAX_FILES || !valid_path(&path) {
                return Err(Error::InvalidInput(
                    "invalid or excessive removal paths".into(),
                ));
            }
            if let Some(info) = files.info(&path)? {
                removed.insert(info.id);
            }
        }
        if removed.is_empty() {
            return Ok(files.into_storage());
        }
        let Self {
            storage,
            anchor,
            index,
            codec,
        } = files;
        let mut tx = Transaction::begin(storage, archive, mode, authority, key)?;
        let result = (|| {
            let mut affected = BTreeMap::<u64, Affected>::new();
            index.visit_range(
                tx.read_storage(),
                anchor.index,
                anchor.sealed_len,
                Some((CHUNK, &[])),
                Some((CHUNK + 1, &[])),
                |entry| {
                    if entry.key.len() != 24 {
                        return Err(Error::CorruptRecord);
                    }
                    let id: [u8; 16] = entry.key[..16].try_into().unwrap();
                    if removed.contains(&id) {
                        let record = OwnedRecord::decode(&entry.value)?;
                        if record.extents.len() != 1 {
                            return Err(Error::CorruptRecord);
                        }
                        Slice::decode(&record.metadata)?;
                        let extent = record.extents[0];
                        affected.entry(extent.start).or_insert(Affected {
                            extent,
                            survivors: Vec::new(),
                        });
                    }
                    Ok(())
                },
            )?;
            let mut entries = Vec::new();
            index.visit(
                tx.read_storage(),
                anchor.index,
                anchor.sealed_len,
                |entry| {
                    let record = OwnedRecord::decode(&entry.value)?;
                    match entry.namespace {
                        FILE => {
                            if !removed.contains(&FileInfo::decode(&record.metadata, &codec)?.id) {
                                entries.push(entry);
                            }
                        }
                        CHUNK => {
                            if entry.key.len() != 24 || record.extents.len() != 1 {
                                return Err(Error::CorruptRecord);
                            }
                            let id: [u8; 16] = entry.key[..16].try_into().unwrap();
                            if !removed.contains(&id) {
                                if let Some(pack) = affected.get_mut(&record.extents[0].start) {
                                    let slice = Slice::decode(&record.metadata)?;
                                    if pack.extent != record.extents[0] {
                                        return Err(Error::CorruptRecord);
                                    }
                                    pack.survivors.push((entry, slice));
                                } else {
                                    entries.push(entry);
                                }
                            }
                        }
                        _ => return Err(Error::CorruptRecord),
                    }
                    Ok(())
                },
            )?;
            // Verify and copy independently encoded fragments one at a time. Their
            // logical AAD is unchanged by physical relocation; no recompression
            // or decrypt/re-encrypt is needed. Retire the whole old allocation.
            for mut pack in affected.into_values() {
                if pack.survivors.is_empty() {
                    continue;
                }
                pack.survivors.sort_by_key(|(_, slice)| slice.start);
                let mut builder = Builder::new(&codec, codec.logical_unit(MAX_LOGICAL)?);
                for (entry, slice) in pack.survivors {
                    if entry.key.as_slice()
                        != chunk_key(slice.descriptor.object, slice.descriptor.ordinal)
                    {
                        return Err(Error::CorruptRecord);
                    }
                    let fragment = slice.physical(pack.extent)?;
                    let stored = crate::page_buffer::ZeroizingBytes::new(
                        tx.read_storage()
                            .read_at(fragment.start, fragment.len as usize)?,
                    );
                    if stored.len() != fragment.len as usize
                        || crate::crypto::strong_checksum(&stored) != fragment.digest
                    {
                        return Err(Error::CorruptRecord);
                    }
                    builder.push_encoded(slice.descriptor, &stored, &mut tx, &mut entries)?;
                }
                builder.flush(&mut tx, &mut entries)?;
            }
            entries.sort_by(|a, b| {
                (a.namespace, a.key.as_slice()).cmp(&(b.namespace, b.key.as_slice()))
            });
            tx.replace_all_sorted(entries.into_iter().map(Ok))?;
            Ok(())
        })();
        if let Err(error) = result {
            tx.abort(authority)?;
            return Err(error);
        }
        tx.commit(authority, signer).map(|(storage, _)| storage)
    }
}
