//! Whole-pack file retirement for the experimental shared tree. All source
//! membership and bytes are checked before preparation; only affected packs are
//! copied. Encoded staging is wipeable and bounded, not a public streaming API.
use super::super::dense_catalogue::{File, Fragment, Pack};
use super::*;
use crate::crypto::strong_checksum;
use crate::page_buffer::ZeroizingBytes;
use std::collections::BTreeSet;
use std::io::{Seek, SeekFrom};

fn checked_name(path: &[u8]) -> Result<[u8; 32]> {
    let path = std::str::from_utf8(path)
        .map_err(|_| Error::InvalidInput("canonical file path required".into()))?;
    let canonical = Zeroizing::new(crate::lockbox_path::canonicalize_stored_path(path, false)?);
    if canonical.as_str() != path {
        return Err(Error::InvalidInput("canonical file path required".into()));
    }
    Ok(Sha256::digest(path.as_bytes()).into())
}

#[allow(clippy::too_many_arguments)]
pub(in crate::file_format::candidate_files) fn remove_files(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    paths: &[Vec<u8>],
) -> Result<bool> {
    update_files(
        storage,
        archive,
        mode,
        authority,
        signer,
        key,
        Vec::<Input<std::io::Cursor<Vec<u8>>>>::new(),
        paths,
    )
}
/// Changed seekable sources are read twice before persistence. New files receive
/// private default permissions; replacements retain identity and permissions.
/// This bounded adapter accepts at most 64 MiB of changed logical source bytes.
#[allow(clippy::too_many_arguments)]
pub(in crate::file_format::candidate_files) fn update_files<R: Read + Seek>(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    mut inputs: Vec<Input<R>>,
    paths: &[Vec<u8>],
) -> Result<bool> {
    if paths.len() > MAX_FILES {
        return Err(Error::SecurityLimitExceeded("file removal count".into()));
    }
    let mut names = BTreeSet::new();
    for path in paths {
        if !names.insert(checked_name(path)?) {
            return Err(Error::InvalidInput("distinct file paths required".into()));
        }
    }
    let (_, body) = shared::open_private(storage, archive, mode, authority, key)?;
    let dense = !body.starts_with(b"RV4TRE01");
    let mut image = if dense {
        Image::open(
            allocation::compaction::View(&*storage),
            archive,
            mode,
            authority,
            key,
        )?
    } else {
        TreeImage::open(
            allocation::compaction::View(&*storage),
            archive,
            mode,
            authority,
            key,
        )?
        .image
    };
    let anchor = image.anchor.clone();
    shared::prepare(&anchor, authority, signer)?;
    if dense {
        image.catalogue.upgrade(&anchor)?;
    }
    if inputs.len() > MAX_FILES {
        return Err(Error::SecurityLimitExceeded("file update count".into()));
    }
    let unit = image.codec.logical_unit(65536)?;
    let mut scratch = ZeroizingBytes::new(vec![0; unit]);
    let mut plans = Vec::new();
    let mut all_names = names.clone();
    let mut total_source = 0usize;
    for (number, input) in inputs.iter_mut().enumerate() {
        let name = checked_name(&input.path)?;
        if !all_names.insert(name) {
            return Err(Error::InvalidInput("conflicting file update paths".into()));
        }
        let start = input
            .reader
            .stream_position()
            .map_err(|e| Error::Io(e.to_string()))?;
        let mut hash = Sha256::new();
        let mut len = 0u64;
        loop {
            let count = match input.reader.read(&mut scratch) {
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                result => result.map_err(|e| Error::Io(e.to_string()))?,
            };
            if count == 0 {
                break;
            }
            len = len.checked_add(count as u64).ok_or(Error::CorruptRecord)?;
            if len > 64 * 1024 * 1024 {
                return Err(Error::SecurityLimitExceeded("bounded file source".into()));
            }
            hash.update(&scratch[..count]);
        }
        let digest: [u8; 32] = hash.finalize().into();
        input
            .reader
            .seek(SeekFrom::Start(start))
            .map_err(|e| Error::Io(e.to_string()))?;
        let old = image
            .catalogue
            .files
            .iter()
            .find(|file| file.path.as_slice() == input.path);
        if old.is_some_and(|file| file.info.len == len && file.info.digest == digest) {
            continue;
        }
        total_source = total_source
            .checked_add(len as usize)
            .ok_or(Error::CorruptRecord)?;
        if total_source > 64 * 1024 * 1024 {
            return Err(Error::SecurityLimitExceeded(
                "bounded aggregate file source".into(),
            ));
        }
        let (id, permissions) = if let Some(file) = old {
            names.insert(name);
            (file.info.id, file.permissions)
        } else {
            let mut id = [0; 16];
            getrandom::fill(&mut id).map_err(|e| Error::Io(e.to_string()))?;
            (id, 0o600)
        };
        let info = FileInfo {
            id,
            len,
            unit: unit as u32,
            digest,
        };
        FileInfo::decode(&info.encode(), &image.codec)?;
        plans.push((number, info, permissions));
    }
    let removed = |path: &[u8]| names.contains(&<[u8; 32]>::from(Sha256::digest(path)));
    if plans.is_empty() && !image.catalogue.files.iter().any(|file| removed(&file.path)) {
        return Ok(false);
    }
    image.verify_all()?;
    let base = shared::commitment(&anchor)?;
    let mut affected = BTreeSet::new();
    for file in &image.catalogue.files {
        if removed(&file.path) {
            for fragment in &file.fragments {
                affected.insert(fragment.pack);
            }
        }
    }
    let mut catalogue = image.catalogue;
    catalogue.files.retain(|file| !removed(&file.path));
    let mut retired = Vec::new();
    let mut retained = Vec::new();
    let mut staged = Vec::new();
    let mut replacements = Vec::new();
    let mut mapping = vec![None; catalogue.packs.len()];
    let mut total = 0usize;
    for (index, pack) in catalogue.packs.into_iter().enumerate() {
        if !affected.contains(&index) {
            mapping[index] = Some(retained.len());
            retained.push(pack);
            continue;
        }
        retired.push(pack.extent);
        let source =
            ZeroizingBytes::new(storage.read_at(pack.extent.start, pack.extent.len as usize)?);
        if strong_checksum(&source) != pack.extent.digest {
            return Err(Error::CorruptRecord);
        }
        let mut members = Vec::new();
        for (file_index, file) in catalogue.files.iter().enumerate() {
            for (fragment_index, fragment) in file.fragments.iter().enumerate() {
                if fragment.pack == index {
                    members.push((fragment.relative, file_index, fragment_index));
                }
            }
        }
        members.sort_unstable();
        if members.is_empty() {
            continue;
        }
        let mut bytes = ZeroizingBytes::new(Vec::with_capacity(pack.extent.len as usize));
        for (_, file, fragment) in members {
            let fragment = &mut catalogue.files[file].fragments[fragment];
            let end = fragment
                .relative
                .checked_add(fragment.descriptor.stored_len())
                .ok_or(Error::CorruptRecord)?;
            let slice = source
                .get(fragment.relative..end)
                .ok_or(Error::CorruptRecord)?;
            if strong_checksum(slice) != fragment.digest {
                return Err(Error::CorruptRecord);
            }
            fragment.relative = bytes.len();
            bytes.extend_from_slice(slice);
        }
        let used = bytes.len();
        let padded = image.codec.pack_padded_len(used)?;
        let padding = image.codec.pack_padding(padded - used)?;
        bytes.extend_from_slice(&padding);
        total = total.checked_add(bytes.len()).ok_or(Error::CorruptRecord)?;
        if total > 64 * 1024 * 1024 {
            return Err(Error::SecurityLimitExceeded(
                "bounded file removal staging".into(),
            ));
        }
        replacements.push((
            index,
            super::super::dense_catalogue::Pack {
                extent: allocation::Extent {
                    start: 0,
                    len: bytes.len() as u64,
                    digest: strong_checksum(&bytes),
                },
                padding_digest: strong_checksum(&padding),
                used,
            },
        ));
        staged.push(bytes);
    }
    let first = retained.len();
    let mut placeholder = anchor.sealed_len;
    for (old_index, mut pack) in replacements {
        pack.extent.start = placeholder;
        placeholder = placeholder
            .checked_add(pack.extent.len)
            .ok_or(Error::CorruptRecord)?;
        mapping[old_index] = Some(retained.len());
        retained.push(pack);
    }
    for file in &mut catalogue.files {
        for fragment in &mut file.fragments {
            fragment.pack = mapping[fragment.pack].ok_or(Error::CorruptRecord)?;
        }
    }
    catalogue.packs = retained;
    let mut pending = NewPacks {
        packs: catalogue.packs,
        bytes: staged,
        buffer: ZeroizingBytes::new(Vec::with_capacity(unit + 65536)),
        logical: 0,
        unit,
        total,
        next: placeholder,
    };
    for (number, info, permissions) in plans {
        let input = &mut inputs[number];
        let mut fragments = Vec::new();
        let mut hash = Sha256::new();
        for ordinal in 0..info.count() {
            let offset = ordinal * u64::from(info.unit);
            let len = (info.len - offset).min(u64::from(info.unit)) as usize;
            input
                .reader
                .read_exact(&mut scratch[..len])
                .map_err(|e| Error::Io(e.to_string()))?;
            hash.update(&scratch[..len]);
            let (descriptor, encoded) =
                image
                    .codec
                    .encode(info.id, ordinal, offset, &scratch[..len])?;
            fragments.push(pending.push(&image.codec, descriptor, &encoded)?);
        }
        let extra = loop {
            match input.reader.read(&mut scratch[..1]) {
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                result => break result.map_err(|e| Error::Io(e.to_string()))?,
            }
        };
        if extra != 0 || <[u8; 32]>::from(hash.finalize()) != info.digest {
            return Err(Error::InvalidInput(
                "file source changed during staging".into(),
            ));
        }
        catalogue.files.push(File {
            path: Zeroizing::new(input.path.clone()),
            permissions,
            info,
            fragments,
        });
    }
    pending.flush(&image.codec)?;
    catalogue.packs = pending.packs;
    let staged = pending.bytes;
    catalogue
        .files
        .sort_by(|a, b| a.path.as_slice().cmp(&b.path));
    let metadata = catalogue.filesystem_metadata()?;
    catalogue.set_tree_metadata(&metadata)?;
    let mut ids = BTreeSet::new();
    let mut fragments = 0usize;
    for file in &catalogue.files {
        fragments += file.fragments.len();
        if !ids.insert(file.info.id) || fragments > 4096 || catalogue.packs.len() > 4096 {
            return Err(Error::SecurityLimitExceeded(
                "bounded typed file catalogue".into(),
            ));
        }
    }
    let records = catalogue.tree_records()?;
    let codec = image.codec;
    let plan = tree::PayloadPlan {
        base,
        retired,
        bytes: staged,
        rebind: Box::new(move |extents| {
            if extents.len() != catalogue.packs.len() - first {
                return Err(Error::CorruptRecord);
            }
            for (pack, extent) in catalogue.packs[first..].iter_mut().zip(extents) {
                if pack.extent.len != extent.len || pack.extent.digest != extent.digest {
                    return Err(Error::CorruptRecord);
                }
                pack.extent = *extent;
            }
            let mut ordered: Vec<_> = catalogue.packs.into_iter().enumerate().collect();
            ordered.sort_by_key(|(_, pack)| pack.extent.start);
            let mut mapping = vec![0; ordered.len()];
            for (new, (old, _)) in ordered.iter().enumerate() {
                mapping[*old] = new;
            }
            catalogue.packs = ordered.into_iter().map(|(_, pack)| pack).collect();
            for file in &mut catalogue.files {
                for fragment in &mut file.fragments {
                    fragment.pack = mapping[fragment.pack];
                }
            }
            let sealed = catalogue.packs.last().map_or(REGION_LEN as u64, |pack| {
                pack.extent.start + pack.extent.len
            });
            catalogue.validate_fragments(&codec, sealed)?;
            catalogue.tree_records()
        }),
    };
    if dense {
        tree::grow_dense_payload_records(
            storage, archive, mode, authority, signer, key, records, plan,
        )?;
        // Small results retain the dense path. Capacity is checked before any
        // return relocation; oversized results remain an authenticated tree.
        if super::can_return_inline(storage, archive, mode, authority, key)? {
            super::return_inline(storage, archive, mode, authority, signer, key)?;
        }
        Ok(true)
    } else {
        tree::rewrite_payload_records(
            storage, archive, mode, authority, signer, key, records, plan,
        )
    }
}

struct NewPacks {
    packs: Vec<Pack>,
    bytes: Vec<ZeroizingBytes>,
    buffer: ZeroizingBytes,
    logical: usize,
    unit: usize,
    total: usize,
    next: u64,
}
impl NewPacks {
    fn push(&mut self, codec: &Codec, descriptor: Descriptor, encoded: &[u8]) -> Result<Fragment> {
        if self.buffer.len() + encoded.len() > self.unit
            || self.logical + descriptor.logical_len as usize > self.unit
        {
            self.flush(codec)?;
        }
        let fragment = Fragment {
            descriptor,
            pack: self.packs.len(),
            relative: self.buffer.len(),
            digest: strong_checksum(encoded),
        };
        self.logical += fragment.descriptor.logical_len as usize;
        self.buffer.extend_from_slice(encoded);
        if self.logical >= self.unit || self.buffer.len() >= self.unit {
            self.flush(codec)?;
        }
        Ok(fragment)
    }
    fn flush(&mut self, codec: &Codec) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let used = self.buffer.len();
        let padded = codec.pack_padded_len(used)?;
        self.total = self.total.checked_add(padded).ok_or(Error::CorruptRecord)?;
        if self.total > 64 * 1024 * 1024 {
            return Err(Error::SecurityLimitExceeded(
                "bounded encoded file staging".into(),
            ));
        }
        let padding = codec.pack_padding(padded - used)?;
        self.buffer.extend_from_slice(&padding);
        let extent = allocation::Extent {
            start: self.next,
            len: self.buffer.len() as u64,
            digest: strong_checksum(&self.buffer),
        };
        self.next = self
            .next
            .checked_add(extent.len)
            .ok_or(Error::CorruptRecord)?;
        self.packs.push(Pack {
            extent,
            padding_digest: strong_checksum(&padding),
            used,
        });
        self.bytes.push(std::mem::replace(
            &mut self.buffer,
            ZeroizingBytes::new(Vec::with_capacity(self.unit + 65536)),
        ));
        self.logical = 0;
        Ok(())
    }
}
