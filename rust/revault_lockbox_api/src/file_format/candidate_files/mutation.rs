//! Atomic file updates with whole-pack retirement. No public CLI emits C yet.
use super::*;
use crate::file_format::allocation_map::Extent;
use std::collections::BTreeSet;
use std::io::{Seek, SeekFrom};

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
        Self::update(
            storage,
            archive,
            mode,
            authority,
            signer,
            key,
            MAX_LOGICAL,
            Vec::<Input<std::io::Cursor<Vec<u8>>>>::new(),
            paths,
        )
    }

    /// Inputs are seekable streams owned by the caller, not buffered file contents.
    /// Preflight reads them before preparation; unchanged inputs cause no writes.
    /// Changed streams are rewound and checked again during staging, so a changed
    /// source cannot publish a snapshot different from the preflight selection.
    /// A path cannot be both explicitly removed and supplied in the same update.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update<R: Read + Seek>(
        storage: S,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        signer: Option<&OwnerSigningKeyPair>,
        key: Option<&[u8]>,
        unit: usize,
        mut inputs: Vec<Input<R>>,
        paths: impl IntoIterator<Item = Vec<u8>>,
    ) -> Result<S> {
        if inputs.len() > MAX_FILES {
            return Err(Error::SecurityLimitExceeded(
                "candidate update file limit".into(),
            ));
        }
        let files = Self::open(storage, archive, mode, authority, key)?;
        let logical_unit = files.codec.logical_unit(unit)?;
        let mut removed = BTreeSet::new();
        let mut names = BTreeSet::new();
        for (count, path) in paths.into_iter().enumerate() {
            let path = Zeroizing::new(path);
            if count >= MAX_FILES || !valid_path(&path) {
                return Err(Error::InvalidInput(
                    "invalid or excessive removal paths".into(),
                ));
            }
            names.insert(<[u8; 32]>::from(Sha256::digest(&path)));
            if let Some(info) = files.info(&path)? {
                removed.insert(info.id);
            }
        }
        let mut plans = Vec::new();
        let mut bytes = crate::page_buffer::ZeroizingBytes::new(vec![0; logical_unit]);
        for (number, input) in inputs.iter_mut().enumerate() {
            if !valid_path(&input.path)
                || !names.insert(<[u8; 32]>::from(Sha256::digest(&input.path)))
            {
                return Err(Error::InvalidInput(
                    "invalid or duplicate update path".into(),
                ));
            }
            let start = input
                .reader
                .stream_position()
                .map_err(|e| Error::Io(e.to_string()))?;
            let (len, digest) = hash_source(&mut input.reader, &mut bytes)?;
            input
                .reader
                .seek(SeekFrom::Start(start))
                .map_err(|e| Error::Io(e.to_string()))?;
            let old = files.info(&input.path)?;
            if old
                .as_ref()
                .is_some_and(|info| info.len == len && info.digest == digest)
            {
                continue;
            }
            let id = if let Some(info) = old {
                removed.insert(info.id);
                info.id
            } else {
                let mut id = [0; 16];
                getrandom::fill(&mut id).map_err(|e| Error::Io(e.to_string()))?;
                id
            };
            let info = FileInfo {
                id,
                len,
                unit: logical_unit as u32,
                digest,
            };
            if id == [0; 16] || info.count() > MAX_ENTRIES as u64 {
                return Err(Error::SecurityLimitExceeded(
                    "candidate update size limit".into(),
                ));
            }
            plans.push((number, info));
        }
        if removed.is_empty() && plans.is_empty() {
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
            // Additions must not collide with retained identities; replacements
            // intentionally retain their object's identity and replace every chunk.
            let mut ids = BTreeSet::new();
            let mut file_count = 0;
            let mut path_bytes = 0usize;
            for entry in &entries {
                if entry.namespace == FILE {
                    let record = OwnedRecord::decode(&entry.value)?;
                    ids.insert(FileInfo::decode(&record.metadata, &codec)?.id);
                    file_count += 1;
                    path_bytes += entry.key.len();
                }
            }
            let mut packs = Builder::new(&codec, logical_unit);
            for (number, info) in plans {
                if file_count == MAX_FILES || !ids.insert(info.id) {
                    return Err(Error::SecurityLimitExceeded(
                        "candidate update identity limit".into(),
                    ));
                }
                file_count += 1;
                let input = &mut inputs[number];
                path_bytes = path_bytes
                    .checked_add(input.path.len())
                    .ok_or(Error::CorruptRecord)?;
                if path_bytes > MAX_PATH_BYTES {
                    return Err(Error::SecurityLimitExceeded(
                        "candidate path metadata limit".into(),
                    ));
                }
                let mut hash = Sha256::new();
                let mut position = 0u64;
                for ordinal in 0..info.count() {
                    let length = (info.len - position).min(logical_unit as u64) as usize;
                    input
                        .reader
                        .read_exact(&mut bytes[..length])
                        .map_err(|e| Error::Io(e.to_string()))?;
                    if entries.len() + packs.pending() >= MAX_ENTRIES - 1 {
                        return Err(Error::SecurityLimitExceeded(
                            "candidate update index limit".into(),
                        ));
                    }
                    packs.push(
                        (info.id, ordinal, position),
                        &bytes[..length],
                        &mut tx,
                        &mut entries,
                    )?;
                    hash.update(&bytes[..length]);
                    position += length as u64;
                }
                // Read to EOF as well as comparing the hash: growth, shortening,
                // replacement and a late I/O error all invalidate the preparation.
                let extra = loop {
                    match input.reader.read(&mut bytes[..1]) {
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        result => break result.map_err(|e| Error::Io(e.to_string()))?,
                    }
                };
                if extra != 0 || <[u8; 32]>::from(hash.finalize()) != info.digest {
                    return Err(Error::InvalidInput(
                        "candidate update source changed during staging".into(),
                    ));
                }
                if entries.len() + packs.pending() >= MAX_ENTRIES {
                    return Err(Error::SecurityLimitExceeded(
                        "candidate update index limit".into(),
                    ));
                }
                entries.push(Entry::new(
                    FILE,
                    &input.path,
                    &OwnedRecord::encode(&info.encode(), &[])?,
                )?);
            }
            packs.flush(&mut tx, &mut entries)?;
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

fn hash_source(reader: &mut impl Read, bytes: &mut [u8]) -> Result<(u64, [u8; 32])> {
    let mut hash = Sha256::new();
    let mut len = 0u64;
    loop {
        let n = match reader.read(bytes) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(Error::Io(e.to_string())),
        };
        len = len.checked_add(n as u64).ok_or(Error::CorruptRecord)?;
        if len > MAX_ENTRIES as u64 * MAX_LOGICAL as u64 {
            return Err(Error::SecurityLimitExceeded(
                "candidate source size limit".into(),
            ));
        }
        hash.update(&bytes[..n]);
    }
    Ok((len, hash.finalize().into()))
}
