//! Ordinary read sessions authenticate only the selected root and lookup paths.
//! They are not whole-archive audits or allocation authority. The caller holds a
//! stable snapshot/read lock; every newly fetched page and payload is verified.
use super::*;
use crate::archive_reader::file_format::allocation_map::Extent;
use crate::archive_reader::file_format::authenticated_index::{Entry, PageCache};
use crate::archive_reader::file_format::publication_anchor::RootRef;
use crate::archive_reader::{LockboxEntry, LockboxEntryKind, LockboxPath};
use std::cell::RefCell;
mod parallel;

struct Selected {
    extent: Extent,
    descriptor: Descriptor,
    first: usize,
    last: usize,
}

pub(in crate::archive_reader::file_format::candidate_files) struct TreeImage<S: Storage> {
    pub image: ReadImage<S>,
}
pub(in crate::archive_reader::file_format::candidate_files) struct ReadImage<S: Storage> {
    pub storage: S,
    pub anchor: Anchor,
    index: Index,
    root: RootRef,
    cache: RefCell<PageCache>,
    version: u8,
    codec: Codec,
    key: Option<Zeroizing<Vec<u8>>>,
    parallel: Option<parallel::Parallel>,
    automatic_workers: bool,
}
impl<S: Storage> TreeImage<S> {
    pub fn open(
        storage: S,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        key: Option<&[u8]>,
    ) -> Result<Self> {
        Self::open_observed(storage, archive, mode, authority, key, |_| {})
    }
    pub(in crate::archive_reader::file_format::candidate_files) fn open_observed(
        storage: S,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        key: Option<&[u8]>,
        mut stage: impl FnMut(&'static str),
    ) -> Result<Self> {
        // Bounded publication/journal admission remains mandatory. No global
        // allocation graph, unused padding, free-space or payload scan occurs.
        let (anchor, body) = shared::open_private(&storage, archive, mode, authority, key)?;
        stage("selected_publication");
        let root = tree::parse_manifest(&body)?;
        let mut image = ReadImage {
            storage,
            anchor,
            root,
            cache: RefCell::default(),
            version: 0,
            index: Index::new(archive, mode, key)?,
            codec: Codec::shared_packed(archive, mode, key)?,
            key: key.map(|v| Zeroizing::new(v.to_vec())),
            parallel: None,
            automatic_workers: false,
        };
        let format = image.required(5, b"format")?;
        if !matches!(
            format.value.as_slice(),
            b"RV4FS001" | b"RV4FS002" | b"RV4FS003"
        ) {
            return Err(Error::CorruptRecord);
        }
        image.version = match format.value.as_slice() {
            b"RV4FS001" => 1,
            b"RV4FS002" => 2,
            _ => 3,
        };
        stage("format_lookup");
        Ok(Self { image })
    }
    pub fn open_credential(
        storage: S,
        archive: LockboxId,
        mode: FormatMode,
        owner: Option<&crate::archive_reader::OwnerSigningPublicKey>,
        credential: publication::bootstrap::Credential<'_>,
        slot: Option<u64>,
    ) -> Result<Self> {
        let opened = shared::credential_open(&storage, archive, mode, owner, credential, slot)?;
        let mut key = Zeroizing::new([0u8; 32]);
        opened.key.with_bytes(|bytes| {
            if bytes.len() != key.len() {
                return Err(Error::CorruptRecord);
            }
            key.copy_from_slice(bytes);
            Ok(())
        })??;
        let authority = owner.map_or(Authority::Symmetric(key.as_ref()), Authority::Owner);
        let image = Self::open(storage, archive, mode, &authority, Some(key.as_ref()))?;
        if image.image.anchor != opened.anchor {
            return Err(Error::CorruptRecord);
        }
        Ok(image)
    }
}
impl<S: Storage> ReadImage<S> {
    /// Test-only policy: zero selects lazy automatic workers; one means the
    /// unchanged serial path and no thread pool.
    /// Reuse this session across files; even configured parallel sessions keep
    /// single-chunk ranges and requests below 128 KiB serial. The coordinator
    /// is additional to N workers.
    pub fn set_read_workers(&mut self, workers: usize) -> Result<()> {
        let parallel = match workers {
            0 | 1 => None,
            2 | 4 => Some(parallel::Parallel::new(
                self.anchor.archive,
                self.anchor.mode,
                self.key.as_deref().map(Vec::as_slice),
                workers,
            )?),
            _ => {
                return Err(Error::InvalidInput(
                    "read workers must be 0, 1, 2 or 4".into(),
                ))
            }
        };
        self.parallel = parallel;
        self.automatic_workers = workers == 0;
        Ok(())
    }
    pub(in crate::archive_reader::file_format::candidate_files) fn read_pool_workers(
        &self,
    ) -> usize {
        self.parallel.as_ref().map_or(0, |pool| pool.capacity() / 8)
    }
    fn get(&self, namespace: u8, key: &[u8]) -> Result<Option<Entry>> {
        self.index.get_cached(
            &self.storage,
            self.root,
            self.anchor.sealed_len,
            namespace,
            key,
            &mut self.cache.borrow_mut(),
        )
    }
    fn required(&self, namespace: u8, key: &[u8]) -> Result<Entry> {
        self.get(namespace, key)?.ok_or(Error::CorruptRecord)
    }
    fn file_info(&self, entry: &Entry) -> Result<(u32, FileInfo)> {
        super::super::dense_catalogue::nodes::canonical(&entry.key)?;
        if entry.value.len() != 76 {
            return Err(Error::CorruptRecord);
        }
        let permissions = u32::from_le_bytes(entry.value[..4].try_into().unwrap());
        crate::archive_reader::security::validate_permissions(permissions)
            .map_err(|_| Error::CorruptRecord)?;
        Ok((
            permissions,
            FileInfo::decode(&entry.value[4..], &self.codec)?,
        ))
    }
    pub fn info(&self, path: &[u8]) -> Result<Option<FileInfo>> {
        super::super::dense_catalogue::nodes::canonical(path)?;
        self.get(1, path)?
            .map(|entry| self.file_info(&entry).map(|(_, info)| info))
            .transpose()
    }
    /// Untimed benchmark fixture census from authenticated chunk references.
    #[cfg(test)]
    pub(in crate::archive_reader::file_format::candidate_files) fn codec_census(
        &self,
        path: &[u8],
    ) -> Result<(u64, u64, u64, u64)> {
        let info = self.info(path)?.ok_or(Error::CorruptRecord)?;
        let mut census = (0u64, 0u64, 0u64, 0u64);
        for ordinal in 0..info.count() {
            let selected = self.select_chunk(&info, ordinal, 0, info.len)?;
            let (raw, encoded_len) = selected.descriptor.benchmark_encoding();
            if raw {
                census.0 += 1;
            } else {
                census.1 += 1;
            }
            census.2 += u64::from(selected.descriptor.logical_len);
            census.3 += u64::from(encoded_len);
        }
        Ok(census)
    }

    pub fn read_range(
        &mut self,
        path: &[u8],
        offset: u64,
        len: u64,
        mut visitor: impl FnMut(&[u8]) -> Result<()>,
    ) -> Result<()> {
        let info = self
            .info(path)?
            .ok_or(Error::InvalidInput("file not found".into()))?;
        let end = offset
            .checked_add(len)
            .ok_or(Error::InvalidInput("range overflow".into()))?;
        if end > info.len {
            return Err(Error::InvalidInput("range outside file".into()));
        }
        if len == 0 {
            return Ok(());
        }
        let first_ordinal = offset / info.unit as u64;
        let limit = end.div_ceil(info.unit as u64);
        if self.automatic_workers {
            return self.read_automatic(&info, first_ordinal, limit, offset, end, &mut visitor);
        }
        if self.parallel.is_some() && len >= 128 * 1024 && limit - first_ordinal > 1 {
            let capacity = self.parallel.as_ref().unwrap().capacity();
            let mut next = first_ordinal;
            while next < limit {
                let mut jobs = Vec::with_capacity(capacity);
                while next < limit && jobs.len() < capacity {
                    let selected = self.select_chunk(&info, next, offset, end)?;
                    // Validate before allocation/I/O; workers revalidate and run
                    // the identical commitment/decryption/decoding path.
                    self.codec.validate_extent(
                        selected.extent,
                        self.anchor.sealed_len,
                        &selected.descriptor,
                    )?;
                    let mut stored = crate::archive_reader::page_buffer::ZeroizingBytes::new(vec![
                        0;
                        selected.extent.len
                            as usize
                    ]);
                    self.storage
                        .read_at_into(selected.extent.start, &mut stored)?;
                    jobs.push(parallel::Job {
                        selected,
                        stored,
                        decoded: None,
                    });
                    next += 1;
                }
                self.parallel
                    .as_mut()
                    .unwrap()
                    .decode(&mut jobs, self.anchor.sealed_len);
                // Success exposes only authenticated bytes in logical order. A
                // later failure may leave an already verified prefix delivered,
                // exactly as the serial API; callers must discard partial sinks.
                for mut job in jobs {
                    let bytes = job.decoded.take().ok_or(Error::CorruptRecord)??;
                    visitor(&bytes[job.selected.first..job.selected.last])?;
                }
            }
        } else {
            for ordinal in first_ordinal..limit {
                let selected = self.select_chunk(&info, ordinal, offset, end)?;
                self.codec.with_loaded(
                    &self.storage,
                    selected.extent,
                    self.anchor.sealed_len,
                    &selected.descriptor,
                    |bytes| visitor(&bytes[selected.first..selected.last]),
                )?;
            }
        }
        Ok(())
    }

    fn read_automatic(
        &mut self,
        info: &FileInfo,
        mut next: u64,
        limit: u64,
        offset: u64,
        end: u64,
        visitor: &mut impl FnMut(&[u8]) -> Result<()>,
    ) -> Result<()> {
        // Qualification is per request, even when a prior request left a pool.
        // A raw chunk resets the run. No upfront file scan or mode-flag guess.
        const QUALIFY_BYTES: u64 = 16 * 1024 * 1024;
        const BATCH_CHUNKS: usize = 32;
        let mut compressed_run = 0u64;
        let mut pending = None;
        while next < limit || pending.is_some() {
            let selected = match pending.take() {
                Some(selected) => selected,
                None => {
                    let selected = self.select_chunk(info, next, offset, end)?;
                    next += 1;
                    selected
                }
            };
            let compressed = selected.descriptor.is_compressed();
            if !compressed || compressed_run < QUALIFY_BYTES {
                self.visit_selected(&selected, visitor)?;
                compressed_run = if compressed {
                    compressed_run + (selected.last - selected.first) as u64
                } else {
                    0
                };
                continue;
            }
            let mut selected_batch = Vec::with_capacity(BATCH_CHUNKS);
            selected_batch.push(selected);
            while next < limit && selected_batch.len() < BATCH_CHUNKS {
                let selected = self.select_chunk(info, next, offset, end)?;
                next += 1;
                if !selected.descriptor.is_compressed() {
                    pending = Some(selected);
                    break;
                }
                selected_batch.push(selected);
            }
            if selected_batch.len() == 1 {
                self.visit_selected(&selected_batch[0], visitor)?;
                continue;
            }
            if self.parallel.is_none() {
                self.parallel = Some(parallel::Parallel::new(
                    self.anchor.archive,
                    self.anchor.mode,
                    self.key.as_deref().map(Vec::as_slice),
                    4,
                )?);
            }
            let mut jobs = Vec::with_capacity(selected_batch.len());
            for selected in selected_batch {
                self.codec.validate_extent(
                    selected.extent,
                    self.anchor.sealed_len,
                    &selected.descriptor,
                )?;
                let mut stored = crate::archive_reader::page_buffer::ZeroizingBytes::new(vec![
                        0;
                        selected.extent.len
                            as usize
                    ]);
                self.storage
                    .read_at_into(selected.extent.start, &mut stored)?;
                jobs.push(parallel::Job {
                    selected,
                    stored,
                    decoded: None,
                });
            }
            self.parallel
                .as_mut()
                .unwrap()
                .decode(&mut jobs, self.anchor.sealed_len);
            for mut job in jobs {
                let bytes = job.decoded.take().ok_or(Error::CorruptRecord)??;
                visitor(&bytes[job.selected.first..job.selected.last])?;
            }
        }
        Ok(())
    }

    fn visit_selected(
        &mut self,
        selected: &Selected,
        visitor: &mut impl FnMut(&[u8]) -> Result<()>,
    ) -> Result<()> {
        self.codec.with_loaded(
            &self.storage,
            selected.extent,
            self.anchor.sealed_len,
            &selected.descriptor,
            |bytes| visitor(&bytes[selected.first..selected.last]),
        )
    }

    fn select_chunk(
        &self,
        info: &FileInfo,
        ordinal: u64,
        offset: u64,
        end: u64,
    ) -> Result<Selected> {
        let row = self.required(2, &chunk_key(info.id, ordinal))?;
        if row.value.len() != 112 {
            return Err(Error::CorruptRecord);
        }
        let descriptor = Descriptor::decode(&row.value[..64])?;
        let logical = ordinal
            .checked_mul(info.unit as u64)
            .ok_or(Error::CorruptRecord)?;
        if descriptor.object != info.id
            || descriptor.ordinal != ordinal
            || descriptor.offset != logical
            || descriptor.logical_len as u64 != (info.len - logical).min(info.unit as u64)
        {
            return Err(Error::CorruptRecord);
        }
        let start = u64::from_le_bytes(row.value[64..72].try_into().unwrap());
        let relative = u64::from_le_bytes(row.value[72..80].try_into().unwrap());
        let pack = self.required(3, &start.to_be_bytes())?;
        if pack.value.len() != 72 {
            return Err(Error::CorruptRecord);
        }
        let pack_len = u64::from_le_bytes(pack.value[..8].try_into().unwrap());
        let stored = descriptor.stored_len() as u64;
        if start < REGION_LEN as u64
            || pack_len == 0
            || pack_len
                > self
                    .codec
                    .pack_padded_len(self.codec.logical_unit(MAX_LOGICAL)? + 28)?
                    as u64
            || start
                .checked_add(pack_len)
                .is_none_or(|v| v > self.anchor.sealed_len)
            || relative.checked_add(stored).is_none_or(|v| v > pack_len)
        {
            return Err(Error::CorruptRecord);
        }
        let extent = Extent {
            start: start.checked_add(relative).ok_or(Error::CorruptRecord)?,
            len: stored,
            digest: row.value[80..].try_into().unwrap(),
        };
        let first = offset.saturating_sub(logical) as usize;
        let last = (end - logical).min(descriptor.logical_len as u64) as usize;
        Ok(Selected {
            extent,
            descriptor,
            first,
            last,
        })
    }
    /// Listing intentionally traverses filesystem metadata, not fragment,
    /// variable or form-value records and never their payloads or spare space.
    pub fn filesystem_metadata(&self) -> Result<Vec<Metadata>> {
        let mut entries = Vec::new();
        let mut path_bytes = 0usize;
        for namespace in [1u8, 4] {
            self.index.visit_range(
                &self.storage,
                self.root,
                self.anchor.sealed_len,
                Some((namespace, &[])),
                Some((namespace + 1, &[])),
                |row| {
                    let path = super::super::dense_catalogue::nodes::canonical(&row.key)?;
                    path_bytes = path_bytes
                        .checked_add(row.key.len())
                        .ok_or(Error::CorruptRecord)?;
                    if entries.len() >= MAX_FILES || path_bytes > MAX_PATH_BYTES {
                        return Err(Error::SecurityLimitExceeded(
                            "filesystem listing budget".into(),
                        ));
                    }
                    let (kind, len, permissions, target) = if namespace == 1 {
                        let (permissions, info) = self.file_info(&row)?;
                        (LockboxEntryKind::File, info.len, permissions, None)
                    } else {
                        if row.value.len() < 5 {
                            return Err(Error::CorruptRecord);
                        }
                        let permissions = u32::from_le_bytes(row.value[1..5].try_into().unwrap());
                        crate::archive_reader::security::validate_permissions(permissions)
                            .map_err(|_| Error::CorruptRecord)?;
                        match row.value[0] {
                            3 if row.value.len() == 5 => {
                                (LockboxEntryKind::Directory, 0, permissions, None)
                            }
                            2 if row.value.len() > 5 => {
                                let target = super::super::dense_catalogue::nodes::canonical(
                                    &row.value[5..],
                                )?;
                                (
                                    LockboxEntryKind::Symlink,
                                    0,
                                    permissions,
                                    Some(LockboxPath::new(target)?),
                                )
                            }
                            _ => return Err(Error::CorruptRecord),
                        }
                    };
                    entries.push(Metadata {
                        entry: LockboxEntry {
                            path: LockboxPath::new(path)?,
                            kind,
                            len,
                            permissions,
                        },
                        target,
                    });
                    Ok(())
                },
            )?;
        }
        entries.sort_by(|a, b| a.entry.path.cmp(&b.entry.path));
        if entries
            .windows(2)
            .any(|p| p[0].entry.path == p[1].entry.path)
        {
            return Err(Error::CorruptRecord);
        }
        Ok(entries)
    }
    /// Explicit full verification retains graph, padding, erased-space and
    /// complete content checks. It is never called implicitly by a read.
    pub fn verify_all(&mut self) -> Result<()> {
        let codec = Codec::shared_packed(
            self.anchor.archive,
            self.anchor.mode,
            self.key.as_deref().map(Vec::as_slice),
        )?;
        let (catalogue, tree) = Catalogue::with_tree_borrowed(&codec, |visitor| {
            Tree::from_snapshot_visit(
                &self.storage,
                self.anchor.archive,
                self.anchor.mode,
                self.key.as_deref().map(Vec::as_slice),
                self.anchor.clone(),
                &tree::manifest(self.root),
                visitor,
                &mut |_| {},
            )
        })?;
        tree.graph.verify_reclaimed(&self.storage)?;
        catalogue.verify_padding(&self.storage, &codec)?;
        let mut audit = Image {
            storage: allocation::compaction::View(&self.storage),
            anchor: self.anchor.clone(),
            value_key: super::super::dense_image::value_key_for(
                &catalogue,
                self.anchor.mode,
                self.key.as_deref().map(Vec::as_slice),
            )?,
            catalogue,
            codec,
        };
        audit.verify_all()
    }
    fn value_key(&self) -> Result<Zeroizing<[u8; 32]>> {
        super::super::dense_image::value_key(
            self.anchor.mode,
            self.key.as_deref().map(Vec::as_slice),
        )
    }
}

impl<S: Storage> TreeImage<S> {
    fn variable(
        &self,
        name: &crate::archive_reader::VariableName,
    ) -> Result<Option<crate::archive_reader::file_format::secure_segments::Layout>> {
        self.image
            .get(6, name.as_str().as_bytes())?
            .map(|row| {
                if self.image.version < 2 {
                    return Err(Error::CorruptRecord);
                }
                crate::archive_reader::file_format::secure_segments::Layout::decode_metadata(
                    &row.value,
                    self.image.anchor.mode,
                    self.image.anchor.sealed_len,
                )
            })
            .transpose()
    }
    pub fn get_variable(
        &self,
        name: &crate::archive_reader::VariableName,
    ) -> Result<Option<String>> {
        let Some(layout) = self.variable(name)? else {
            return Ok(None);
        };
        if layout.sensitivity == crate::archive_reader::VariableSensitivity::Secret {
            return Err(Error::InvalidOperation(
                "variable is secret; use secret access".into(),
            ));
        }
        let bytes = layout.read(
            &self.image.storage,
            self.image.anchor.archive,
            &*self.image.value_key()?,
            self.image.anchor.sealed_len,
        )?;
        bytes.with_bytes(|bytes| {
            Ok(Some(
                std::str::from_utf8(bytes)
                    .map_err(|_| Error::CorruptRecord)?
                    .to_owned(),
            ))
        })?
    }
    pub fn with_secret_variable<R>(
        &self,
        name: &crate::archive_reader::VariableName,
        f: impl FnOnce(&crate::archive_reader::SecretString) -> R,
    ) -> Result<Option<R>> {
        let Some(layout) = self.variable(name)? else {
            return Ok(None);
        };
        if layout.sensitivity != crate::archive_reader::VariableSensitivity::Secret {
            return Err(Error::InvalidOperation("variable is not secret".into()));
        }
        let bytes = layout.read(
            &self.image.storage,
            self.image.anchor.archive,
            &*self.image.value_key()?,
            self.image.anchor.sealed_len,
        )?;
        let value =
            std::sync::Arc::new(crate::archive_reader::SecretString::from_secure_vec(bytes));
        Ok(Some(f(&value)))
    }
    pub fn get_form_definition(
        &self,
        id: &crate::archive_reader::FormTypeId,
        revision: u32,
    ) -> Result<Option<crate::archive_reader::FormDefinition>> {
        let forms = super::super::dense_catalogue::forms::Forms::selected_definition(
            id,
            revision,
            self.image.anchor.mode,
            self.image.anchor.sealed_len,
            |n, k| self.image.get(n, k),
        )?;
        if self.image.version < 3 && !forms.is_empty() {
            return Err(Error::CorruptRecord);
        }
        forms
            .definitions
            .first()
            .map(|d| {
                d.read(
                    &self.image.storage,
                    self.image.anchor.archive,
                    &*self.image.value_key()?,
                    self.image.anchor.sealed_len,
                )
            })
            .transpose()
    }
    fn selected_form(
        &self,
        path: &LockboxPath,
    ) -> Result<super::super::dense_catalogue::forms::Forms> {
        let forms = super::super::dense_catalogue::forms::Forms::selected_record(
            path,
            self.image.anchor.mode,
            self.image.anchor.sealed_len,
            |n, k| self.image.get(n, k),
        )?;
        if self.image.version < 3 && !forms.is_empty() {
            return Err(Error::CorruptRecord);
        }
        Ok(forms)
    }
    pub fn get_form_record(
        &self,
        path: &LockboxPath,
    ) -> Result<Option<crate::archive_reader::FormRecord>> {
        let forms = self.selected_form(path)?;
        forms
            .records
            .first()
            .map(|r| {
                r.read(
                    &self.image.storage,
                    self.image.anchor.archive,
                    &*self.image.value_key()?,
                    self.image.anchor.sealed_len,
                )
            })
            .transpose()
    }
    pub fn with_form_field_value<R>(
        &self,
        path: &LockboxPath,
        id: &str,
        f: impl FnOnce(&crate::archive_reader::FormValue) -> R,
    ) -> Result<Option<R>> {
        let forms = self.selected_form(path)?;
        let Some(field) = forms
            .records
            .first()
            .and_then(|r| r.fields.iter().find(|f| f.id == id))
        else {
            return Ok(None);
        };
        let value = field.read_value(
            &self.image.storage,
            self.image.anchor.archive,
            &*self.image.value_key()?,
            self.image.anchor.sealed_len,
        )?;
        Ok(Some(f(&value)))
    }
}
