use super::secure_output::{OutputRoot, StagedDirectory};
use super::Lockbox;
use crate::host_path::HostPath;
use crate::lockbox_path::{canonicalize_stored_path, validate_symlink_paths as validate_symlink};
use crate::node_kind::NodeKind;
use crate::toc_entry::TocEntry;
use crate::{Error, ExtractPolicy, Result};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

// Workers consume already authenticated entry references from the parent's plan.
// Keep the data-page reader, but do not duplicate unrelated archive metadata for
// every thread. This private wrapper deliberately exposes no Lockbox/Deref API:
// its empty metadata collections are not a general-purpose archive view.
struct ExtractionWorker {
    reader: Lockbox<super::ReadOnly>,
}

impl ExtractionWorker {
    fn new<State>(parent: &Lockbox<State>) -> Result<Self> {
        if !parent.pending_small_files.is_empty() {
            return Err(Error::InvalidOperation(
                "parallel extraction requires flushed small files".into(),
            ));
        }
        let source = &parent.staged;
        Ok(Self {
            reader: Lockbox {
                format_mode: parent.format_mode,
                storage: parent.storage.clone(),
                key: parent.key.try_clone()?,
                staged: super::StagedLockboxState {
                    control_reservations: Default::default(),
                    commit_chain: source.commit_chain.clone(),
                    published_sequence: source.published_sequence,
                    header_slot: source.header_slot,
                    header_generation: source.header_generation,
                    cleanup_sequence: source.cleanup_sequence,
                    cleanup_completed_ranges: source.cleanup_completed_ranges,
                    cleanup_completed_pages: source.cleanup_completed_pages,
                    cleanup_completed_bytes: source.cleanup_completed_bytes,
                    toc_tree: Default::default(),
                    toc_root_commitment: source.toc_root_commitment,
                    variable_root_offset: source.variable_root_offset,
                    variable_root_commitment: source.variable_root_commitment,
                    form_root_commitment: source.form_root_commitment,
                    free_index_offset: source.free_index_offset,
                    free_index_commitment: source.free_index_commitment,
                    post_cleanup_free_index_offset: source.post_cleanup_free_index_offset,
                    post_cleanup_free_index_commitment: source.post_cleanup_free_index_commitment,
                    redaction_manifest_offset: source.redaction_manifest_offset,
                    redaction_manifest_commitment: source.redaction_manifest_commitment,
                    redaction_range_count: source.redaction_range_count,
                    redaction_total_bytes: source.redaction_total_bytes,
                    key_directory: source.key_directory,
                    poisoned: source.poisoned.clone(),
                    key_slots: Vec::new(),
                    toc_entries: BTreeMap::new(),
                    variables: std::cell::RefCell::new(Some(BTreeMap::new())),
                    variable_root: None,
                    variable_leaves: Vec::new(),
                    dirty_variables: false,
                    forms: super::FormStore::loaded(),
                    free_space: Default::default(),
                    record_ref_counts: Default::default(),
                    pending_redactions: BTreeMap::new(),
                    pending_redaction_object_count: 0,
                    redacted_free_slots: Vec::new(),
                    pending_small_files: BTreeMap::new(),
                    pending_small_file_bytes: 0,
                    pending_symlinks: BTreeMap::new(),
                    mirror_mutation_root: None,
                    needs_packing: false,
                    access_widening_pending: false,
                },
                lockbox_id: parent.lockbox_id,
                read_only: true,
                owner_signing_key: None,
                // Preserve cached and uncommitted data pages. PageCache clones
                // retain shared page owners and independent eviction bookkeeping.
                page_manager: std::cell::RefCell::new(parent.page_manager.borrow().clone()),
                compression_frame_cache: Default::default(),
                import_stats: Default::default(),
                workload_profile: parent.workload_profile,
                worker_policy: parent.worker_policy,
                transaction_start_len: parent.transaction_start_len,
                trim_origin_len: parent.trim_origin_len,
                preparing: parent.preparing,
                rollback_required: parent.rollback_required,
                state: std::marker::PhantomData,
            },
        })
    }

    fn extract(
        &self,
        entry: &TocEntry,
        root: &OutputRoot,
        relative: &Path,
        policy: &ExtractPolicy,
    ) -> Result<()> {
        self.reader
            .extract_file_entry_to_root(entry, root, relative, policy)
    }

    fn retire(&self, last_use: &mut ExtractionLastUse, entry: &TocEntry, index: usize) {
        last_use.retire(&self.reader, entry, index);
    }
}

struct ExtractionLastUse {
    frames: BTreeMap<u64, usize>,
    pages: BTreeMap<u64, usize>,
}

impl ExtractionLastUse {
    fn new(job: &[(usize, &TocEntry)]) -> Self {
        let mut uses = Self {
            frames: BTreeMap::new(),
            pages: BTreeMap::new(),
        };
        for (index, (_, entry)) in job.iter().enumerate() {
            for chunk in &entry.chunks {
                uses.frames.insert(chunk.compression_frame_id, index);
                for segment in &chunk.segments {
                    uses.pages.insert(segment.page_offset, index);
                }
            }
        }
        uses
    }

    fn retire<State>(&mut self, worker: &Lockbox<State>, entry: &TocEntry, index: usize) {
        for chunk in &entry.chunks {
            if self.frames.get(&chunk.compression_frame_id) == Some(&index) {
                self.frames.remove(&chunk.compression_frame_id);
                let mut cache = worker.compression_frame_cache.borrow_mut();
                if let Some(old) = cache.entries.remove(&chunk.compression_frame_id) {
                    cache.used_bytes = cache.used_bytes.saturating_sub(old.data.len());
                }
            }
            for segment in &chunk.segments {
                if self.pages.get(&segment.page_offset) == Some(&index) {
                    self.pages.remove(&segment.page_offset);
                    worker
                        .page_manager
                        .borrow_mut()
                        .retire_clean_file_page(segment.page_offset);
                }
            }
        }
    }
}

impl<State: Send> Lockbox<State> {
    /// Extract all permitted entries into a destination directory.
    ///
    /// Returns `Error::Io` for host filesystem failures,
    /// `Error::SecurityLimitExceeded` when the extraction policy rejects the
    /// destination or size/count limits, and lockbox read errors for corrupt or
    /// missing stored entries.
    pub fn extract_to_directory(&self, destination: &Path, policy: &ExtractPolicy) -> Result<()> {
        let destination = HostPath::new(destination);
        let destination = destination.as_path();
        match fs::symlink_metadata(destination) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                let root = OutputRoot::open(destination).map_err(output_error)?;
                self.extract_entries_to_root(&root, policy)?;
                root.finish_directories().map_err(output_error)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.extract_to_new_directory(destination, policy)
            }
            Ok(_) => Err(Error::SecurityLimitExceeded(
                "destination is not a non-symlink directory".into(),
            )),
            Err(error) => Err(output_error(error)),
        }
    }

    /// Extract one stored directory and its contents to an exact host directory.
    ///
    /// Unlike [`Self::extract_to_directory`], paths below `source` are written
    /// relative to `destination`; the stored source directory name is not added
    /// automatically.
    pub fn extract_directory_to(
        &self,
        source: &crate::LockboxPath,
        destination: &Path,
        policy: &ExtractPolicy,
    ) -> Result<()> {
        let source = source.as_file_path()?;
        let source_entry = self
            .toc_entries
            .get(source)
            .filter(|entry| !entry.deleted)
            .ok_or_else(|| Error::NotFound(source.to_string()))?;
        if source_entry.node_kind != NodeKind::Directory {
            return Err(Error::InvalidOperation(format!(
                "{} is not a directory",
                source_entry.path
            )));
        }

        let prefix = format!("{}/", source.trim_end_matches('/'));
        let entries = self
            .toc_entries
            .values()
            .filter(|entry| {
                !entry.deleted
                    && (entry.path.as_str() == source || entry.path.as_str().starts_with(&prefix))
            })
            .collect::<Vec<_>>();
        self.validate_extract_entries(&entries, policy)?;

        let (stage, root) = match fs::symlink_metadata(destination) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                (None, OutputRoot::open(destination).map_err(output_error)?)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let (stage, root) = StagedDirectory::at(destination).map_err(output_error)?;
                (Some(stage), root)
            }
            Ok(_) => {
                return Err(Error::SecurityLimitExceeded(
                    "destination is not a non-symlink directory".into(),
                ))
            }
            Err(error) => return Err(output_error(error)),
        };
        for entry in entries {
            let relative = entry
                .path
                .as_str()
                .strip_prefix(source)
                .ok_or_else(|| {
                    Error::InvalidPath(format!(
                        "cannot extract entry {:?}: it is outside the selected source {:?}",
                        entry.path.as_str(),
                        source,
                    ))
                })?
                .trim_start_matches('/');
            self.extract_entry_to_root(entry, &root, Path::new(relative), policy)?;
        }
        if let Some(stage) = stage {
            stage.publish(root).map_err(output_error)
        } else {
            root.finish_directories().map_err(output_error)
        }
    }

    fn extract_to_new_directory(&self, destination: &Path, policy: &ExtractPolicy) -> Result<()> {
        let (stage, root) = StagedDirectory::at(destination).map_err(output_error)?;
        self.extract_entries_to_root(&root, policy)?;
        stage.publish(root).map_err(output_error)
    }

    fn extract_entries_to_root(&self, root: &OutputRoot, policy: &ExtractPolicy) -> Result<()> {
        let current_entries = self.validate_extract_plan(policy)?;
        if self.should_extract_files_in_parallel(&current_entries) {
            self.extract_entries_to_directory_parallel(root, policy, &current_entries)?;
        } else {
            for entry in current_entries {
                let relative = extraction_relative(&entry.path)?;
                self.extract_entry_to_root(entry, root, &relative, policy)?;
            }
        }
        Ok(())
    }

    fn extract_entry_to_root(
        &self,
        entry: &TocEntry,
        root: &OutputRoot,
        relative: &Path,
        policy: &ExtractPolicy,
    ) -> Result<()> {
        match entry.node_kind {
            NodeKind::Directory => {
                root.directory(relative, true).map_err(output_error)?;
                if policy.restore_permissions {
                    root.restore_directory(relative, entry.permissions)
                        .map_err(output_error)?;
                }
            }
            NodeKind::File => self.extract_file_entry_to_root(entry, root, relative, policy)?,
            NodeKind::Symlink if policy.restore_symlinks => {
                let target = self.get_symlink_target(&entry.path)?;
                validate_symlink(entry.path.as_str(), target.as_str())?;
                root.symlink(
                    relative,
                    Path::new(target.as_str().trim_start_matches('/')),
                    policy.overwrite,
                )
                .map_err(output_error)?;
            }
            NodeKind::Symlink => {}
        }
        Ok(())
    }

    fn should_extract_files_in_parallel(&self, current_entries: &[&TocEntry]) -> bool {
        matches!(self.storage, crate::storage::StorageBackend::File(_))
            && self.pending_small_files.is_empty()
            && current_entries
                .iter()
                .filter(|entry| entry.node_kind == NodeKind::File)
                .count()
                >= 256
            && std::thread::available_parallelism()
                .map(|count| count.get() > 1)
                .unwrap_or(false)
    }

    fn extract_entries_to_directory_parallel(
        &self,
        root: &OutputRoot,
        policy: &ExtractPolicy,
        current_entries: &[&TocEntry],
    ) -> Result<()> {
        for entry in current_entries
            .iter()
            .copied()
            .filter(|entry| entry.node_kind == NodeKind::Directory)
        {
            let relative = extraction_relative(&entry.path)?;
            self.extract_entry_to_root(entry, root, &relative, policy)?;
        }

        let file_entries: Vec<_> = current_entries
            .iter()
            .copied()
            .filter(|entry| entry.node_kind == NodeKind::File)
            .enumerate()
            .collect();
        let workers = std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
            .clamp(1, 4)
            .min(file_entries.len().max(1));
        let worker_jobs = group_parallel_extraction_jobs(file_entries, workers);

        let retire_after_last_use = !cfg!(target_arch = "wasm32")
            && self.workload_profile == crate::WorkloadProfile::ExtractMany
            && matches!(
                self.page_manager.borrow().configured_limit(),
                crate::CacheLimit::Auto
            );

        let parallel_result = std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for job in worker_jobs {
                let worker = ExtractionWorker::new(self)?;
                handles.push(scope.spawn(move || {
                    let extract_job = || {
                        let mut last_use =
                            retire_after_last_use.then(|| ExtractionLastUse::new(&job));
                        for (index, (_, entry)) in job.into_iter().enumerate() {
                            let relative = extraction_relative(&entry.path)?;
                            worker.extract(entry, root, &relative, policy)?;
                            if let Some(last_use) = &mut last_use {
                                worker.retire(last_use, entry, index);
                            }
                        }
                        Ok(())
                    };
                    if retire_after_last_use {
                        crate::compression::with_decode_workspace(extract_job)
                    } else {
                        extract_job()
                    }
                }));
            }

            for handle in handles {
                match handle.join() {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) => return Err(err),
                    Err(_) => {
                        return Err(Error::Io("parallel extraction worker panicked".to_string()))
                    }
                }
            }
            Ok(())
        });
        parallel_result?;

        for entry in current_entries
            .iter()
            .copied()
            .filter(|entry| entry.node_kind == NodeKind::Symlink)
        {
            if !policy.restore_symlinks {
                continue;
            }
            let relative = extraction_relative(&entry.path)?;
            self.extract_entry_to_root(entry, root, &relative, policy)?;
        }
        Ok(())
    }

    fn validate_extract_plan(&self, policy: &ExtractPolicy) -> Result<Vec<&TocEntry>> {
        let current_entries: Vec<_> = self
            .toc_entries
            .values()
            .filter(|entry| !entry.deleted)
            .collect();

        self.validate_extract_entries(&current_entries, policy)?;
        Ok(current_entries)
    }

    fn validate_extract_entries(
        &self,
        current_entries: &[&TocEntry],
        policy: &ExtractPolicy,
    ) -> Result<()> {
        if current_entries.len() > policy.max_files {
            return Err(Error::SecurityLimitExceeded(format!(
                "node count {} exceeds limit {}",
                current_entries.len(),
                policy.max_files
            )));
        }

        let mut total = 0u64;
        for entry in current_entries {
            match entry.node_kind {
                NodeKind::File => {
                    if entry.len > policy.max_file_bytes {
                        return Err(Error::SecurityLimitExceeded(format!(
                            "{} is {} bytes, limit is {}",
                            entry.path, entry.len, policy.max_file_bytes
                        )));
                    }
                    total = total.checked_add(entry.len).ok_or_else(|| {
                        Error::SecurityLimitExceeded("total extracted size overflow".to_string())
                    })?;
                    if total > policy.max_total_bytes {
                        return Err(Error::SecurityLimitExceeded(format!(
                            "total extracted bytes {total} exceeds limit {}",
                            policy.max_total_bytes
                        )));
                    }
                }
                NodeKind::Symlink => {
                    if policy.restore_symlinks {
                        let target = self.get_symlink_target(&entry.path)?;
                        validate_symlink(entry.path.as_str(), target.as_str())?;
                    }
                }
                NodeKind::Directory => {}
            }
        }

        Ok(())
    }

    fn extract_file_entry_to_root(
        &self,
        entry: &TocEntry,
        root: &OutputRoot,
        relative: &Path,
        policy: &ExtractPolicy,
    ) -> Result<()> {
        let mut staged = root.stage_file(relative).map_err(output_error)?;
        self.write_file_entry_cached(entry, staged.writer())?;
        staged
            .publish(
                policy.overwrite,
                policy.restore_permissions.then_some(entry.permissions),
            )
            .map_err(output_error)
    }

    fn write_file_entry_cached(&self, entry: &TocEntry, writer: &mut impl Write) -> Result<()> {
        if let Some(pending) = self.pending_small_files.get(&entry.path) {
            if pending.data.len() as u64 != entry.len {
                return Err(Error::CorruptRecord);
            }
            writer
                .write_all(&pending.data)
                .map_err(|err| Error::Io(err.to_string()))?;
            return Ok(());
        }

        if entry.chunks.is_empty() {
            write_zeroes(writer, entry.len)?;
            return Ok(());
        }

        let mut written = 0u64;
        let mut chunks = entry.chunks.clone();
        chunks.sort_by_key(|chunk| chunk.file_offset);
        for chunk in chunks {
            if chunk.file_offset < written || chunk.file_offset > entry.len {
                return Err(Error::CorruptRecord);
            }
            if chunk.file_offset > written {
                write_zeroes(writer, chunk.file_offset - written)?;
                written = chunk.file_offset;
            }
            if chunk.file_offset.saturating_add(chunk.len) > entry.len {
                return Err(Error::CorruptRecord);
            }
            let decoded = crate::page_buffer::SensitiveBytes::new(
                self.read_file_chunk_compression_frame(entry.len, &chunk)?,
                !self.format_mode.plaintext(),
            );
            writer
                .write_all(&decoded)
                .map_err(|err| Error::Io(err.to_string()))?;
            written += decoded.len() as u64;
        }

        if written < entry.len {
            write_zeroes(writer, entry.len - written)?;
        } else if written != entry.len {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }
}

fn output_error(error: std::io::Error) -> Error {
    #[cfg(unix)]
    let component_refused = matches!(
        error.raw_os_error(),
        Some(libc::ELOOP) | Some(libc::ENOTDIR)
    );
    #[cfg(not(unix))]
    let component_refused = false;
    if matches!(
        error.kind(),
        std::io::ErrorKind::AlreadyExists | std::io::ErrorKind::PermissionDenied
    ) || component_refused
    {
        Error::SecurityLimitExceeded(format!(
            "output refused (existing destination or symlink component): {error}"
        ))
    } else {
        Error::Io(error.to_string())
    }
}

fn extraction_relative(path: &crate::LockboxPath) -> Result<PathBuf> {
    canonicalize_stored_path(path.as_str(), false)?;
    let relative = PathBuf::from(path.as_str().trim_start_matches('/'));
    if relative
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(Error::InvalidPath(path.to_string()));
    }
    Ok(relative)
}

fn write_zeroes(writer: &mut impl Write, mut len: u64) -> Result<()> {
    const ZERO_BUF: [u8; 8192] = [0; 8192];
    while len > 0 {
        let write_len = ZERO_BUF.len().min(len as usize);
        writer
            .write_all(&ZERO_BUF[..write_len])
            .map_err(|err| Error::Io(err.to_string()))?;
        len -= write_len as u64;
    }
    Ok(())
}

fn group_parallel_extraction_jobs<'a>(
    file_entries: Vec<(usize, &'a TocEntry)>,
    workers: usize,
) -> Vec<Vec<(usize, &'a TocEntry)>> {
    let mut frame_groups: BTreeMap<u64, Vec<(usize, &'a TocEntry)>> = BTreeMap::new();
    for (path_id, entry) in file_entries {
        let frame_id = entry
            .chunks
            .first()
            .map(|chunk| chunk.compression_frame_id)
            .unwrap_or(u64::MAX);
        frame_groups
            .entry(frame_id)
            .or_default()
            .push((path_id, entry));
    }

    let mut jobs = vec![Vec::new(); workers];
    for (_, group) in frame_groups {
        let target = jobs
            .iter()
            .enumerate()
            .min_by_key(|(_, job)| job.len())
            .map(|(index, _)| index)
            .unwrap_or(0);
        jobs[target].extend(group);
    }
    jobs.into_iter().filter(|job| !job.is_empty()).collect()
}

#[cfg(test)]
fn checked_destination(root: &Path, lockbox_path: &str) -> Result<PathBuf> {
    canonicalize_stored_path(lockbox_path, false)?;
    let relative = lockbox_path.trim_start_matches('/');
    let mut out = root.to_path_buf();
    for component in Path::new(relative).components() {
        match component {
            Component::Normal(part) => out.push(part),
            _ => return Err(Error::InvalidPath(format!(
                "cannot extract lockbox path {lockbox_path:?}: component {component:?} is not a normal filename on this operating system",
            ))),
        }
    }
    if !out.starts_with(root) {
        return Err(Error::SecurityLimitExceeded(
            "extraction destination escaped root".to_string(),
        ));
    }
    reject_symlink_components(root, &out)?;
    Ok(out)
}

#[cfg(test)]
fn reject_symlink_components(root: &Path, destination: &Path) -> Result<()> {
    let relative = destination.strip_prefix(root).map_err(|_| {
        Error::SecurityLimitExceeded("extraction destination escaped root".to_string())
    })?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            return Err(Error::SecurityLimitExceeded(
                "extraction destination contains an unsafe component".to_string(),
            ));
        };
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(Error::SecurityLimitExceeded(format!(
                    "extraction destination contains a symlink: {}",
                    current.display()
                )));
            }
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(Error::Io(err.to_string())),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_test_root(label: &str) -> PathBuf {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("lockbox-{label}-{}-{suffix}", std::process::id()))
    }

    #[test]
    fn extraction_worker_keeps_dirty_payloads_without_archive_metadata() {
        let mut parent = Lockbox::create("worker dirty payload");
        let path = crate::LockboxPath::new("/large.bin").unwrap();
        parent.add_file(&path, b"old", false).unwrap();
        parent.commit().unwrap();
        let expected: Vec<u8> = (0..2 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
        parent.add_file(&path, &expected, true).unwrap();
        assert!(parent.pending_small_files.is_empty());
        assert!(parent.page_manager.borrow().has_dirty_pages());
        assert!(!parent.toc_entries.is_empty());
        let worker = ExtractionWorker::new(&parent).unwrap();
        assert!(worker.reader.toc_entries.is_empty());
        assert!(worker.reader.toc_tree.root.is_none());
        assert!(worker.reader.toc_tree.leaves.is_empty());
        assert!(worker.reader.record_ref_counts.is_empty());
        assert!(worker.reader.key_slots.is_empty());
        assert!(worker.reader.owner_signing_key.is_none());
        assert!(worker.reader.read_only);
        assert!(worker.reader.page_manager.borrow().has_dirty_pages());
        let entry = parent.toc_entries.get(path.as_str()).unwrap();
        let mut actual = Vec::new();
        worker
            .reader
            .write_file_entry_cached(entry, &mut actual)
            .unwrap();
        assert_eq!(actual, expected);
        drop(worker);
        assert!(parent.page_manager.borrow().has_dirty_pages());
        assert_eq!(
            parent
                .read_file_range(&path, 0, expected.len() as u64)
                .unwrap(),
            expected
        );
    }

    #[test]
    fn extraction_worker_rejects_changed_frame_digest_and_pending_small_files() {
        let mut parent = Lockbox::create("worker digest");
        let path = crate::LockboxPath::new("/file").unwrap();
        parent
            .add_file(&path, b"authenticated file", false)
            .unwrap();
        assert!(matches!(
            ExtractionWorker::new(&parent),
            Err(Error::InvalidOperation(_))
        ));
        parent.commit().unwrap();
        let mut entry = parent.toc_entries.get(path.as_str()).unwrap().clone();
        assert!(!entry.chunks.is_empty());
        let worker = ExtractionWorker::new(&parent).unwrap();
        let mut actual = Vec::new();
        worker
            .reader
            .write_file_entry_cached(&entry, &mut actual)
            .unwrap();
        assert_eq!(actual, b"authenticated file");
        // Keep every length, offset and slice valid; change only the committed
        // digest. A fresh worker must reject via the authenticated frame path.
        entry.chunks[0].compression_frame_digest[0] ^= 1;
        let worker = ExtractionWorker::new(&parent).unwrap();
        let mut rejected = Vec::new();
        assert!(matches!(
            worker.reader.write_file_entry_cached(&entry, &mut rejected),
            Err(Error::CorruptRecord)
        ));
        assert!(rejected.is_empty());
        assert_eq!(
            parent.read_file_range(&path, 0, 18).unwrap(),
            b"authenticated file"
        );
    }

    #[test]
    fn private_tree_publication_preserves_nested_files() {
        let parent = unique_test_root("private-tree-publication");
        fs::create_dir_all(&parent).unwrap();
        let destination = parent.join("output");
        let (stage, root) = StagedDirectory::at(&destination).unwrap();
        let mut file = root.stage_file(Path::new("nested/file.txt")).unwrap();
        file.writer().write_all(b"content").unwrap();
        file.publish(false, Some(0o600)).unwrap();
        assert!(!destination.exists());
        stage.publish(root).unwrap();
        assert_eq!(
            fs::read(destination.join("nested/file.txt")).unwrap(),
            b"content"
        );
        assert_eq!(fs::read_dir(&parent).unwrap().count(), 1);
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn extract_directory_to_writes_only_the_selected_subtree() {
        let root = unique_test_root("extract-selected-directory");
        let destination = root.join("destination");
        let mut lockbox = Lockbox::create("secret");
        for (path, contents) in [
            ("/docs/selected/nested/keep.txt", b"keep".as_slice()),
            ("/docs/outside.txt", b"outside".as_slice()),
        ] {
            let path = crate::LockboxPath::new(path).unwrap();
            lockbox.create_parent_dirs_for(&path).unwrap();
            lockbox.add_file(&path, contents, false).unwrap();
        }

        lockbox
            .extract_directory_to(
                &crate::LockboxPath::new("/docs/selected").unwrap(),
                &destination,
                &ExtractPolicy::default(),
            )
            .unwrap();

        assert_eq!(
            fs::read(destination.join("nested/keep.txt")).unwrap(),
            b"keep"
        );
        assert!(!destination.join("outside.txt").exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn checked_destination_rejects_existing_symlink_parent() {
        let root = unique_test_root("extract-symlink-helper");
        let outside = root.with_extension("outside");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, root.join("docs")).unwrap();

        assert!(matches!(
            checked_destination(&root, "/docs/secret.txt"),
            Err(Error::SecurityLimitExceeded(message)) if message.contains("symlink")
        ));

        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
    }

    #[cfg(unix)]
    #[test]
    fn sequential_extraction_rejects_existing_symlink_parent() {
        let root = unique_test_root("extract-symlink-sequential");
        let destination = root.join("destination");
        let outside = root.join("outside");
        fs::create_dir_all(&destination).unwrap();
        fs::create_dir_all(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, destination.join("docs")).unwrap();
        let mut lockbox = Lockbox::create("secret");
        let path = crate::LockboxPath::new("/docs/secret.txt").unwrap();
        lockbox.create_parent_dirs_for(&path).unwrap();
        lockbox.add_file(&path, b"secret", false).unwrap();

        assert!(matches!(
            lockbox.extract_to_directory(&destination, &ExtractPolicy::default()),
            Err(Error::SecurityLimitExceeded(message)) if message.contains("symlink")
        ));
        assert!(!outside.join("secret.txt").exists());

        let _ = fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn parallel_extraction_rejects_existing_symlink_parent() {
        let root = unique_test_root("extract-symlink-parallel");
        fs::create_dir_all(&root).unwrap();
        let archive = root.join("source.lbox");
        let destination = root.join("destination");
        let outside = root.join("outside");
        fs::create_dir_all(&destination).unwrap();
        fs::create_dir_all(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, destination.join("docs")).unwrap();
        let mut lockbox = Lockbox::create_path(&archive, "secret").unwrap();
        for index in 0..256 {
            let path = crate::LockboxPath::new(format!("/docs/file-{index:03}.txt")).unwrap();
            lockbox.create_parent_dirs_for(&path).unwrap();
            lockbox.add_file(&path, b"content", false).unwrap();
        }
        lockbox.commit().unwrap();

        assert!(lockbox.should_extract_files_in_parallel(
            &lockbox
                .validate_extract_plan(&ExtractPolicy::default())
                .unwrap()
        ));
        assert!(matches!(
            lockbox.extract_to_directory(&destination, &ExtractPolicy::default()),
            Err(Error::SecurityLimitExceeded(message)) if message.contains("symlink")
        ));
        assert!(fs::read_dir(&outside).unwrap().next().is_none());

        drop(lockbox);
        let _ = fs::remove_dir_all(&root);
    }
    #[test]
    fn last_use_retirement_preserves_later_shared_frames_pages_and_parent() {
        use crate::page::{DecodedPage, PageObject, PageObjectKind};
        use crate::page_cache::PageSecurity;
        use std::sync::Arc;
        let mut parent = Lockbox::create("last use");
        parent
            .add_file(&crate::LockboxPath::new("/a").unwrap(), b"a", false)
            .unwrap();
        parent.commit().unwrap();
        let template = parent.toc_entries.get("/a").unwrap().clone();
        assert!(!template.chunks.is_empty());
        // Synthetic references exercise planning only. Their page payload bytes are
        // observed below; no synthetic metadata is used as an authenticated archive.
        let chunk = |frame, page| {
            let mut value = template.chunks[0].clone();
            value.compression_frame_id = frame;
            value.segments[0].page_offset = page;
            value.segments.truncate(1);
            value
        };
        let mut first = template.clone();
        let mut second = template.clone();
        let mut third = template.clone();
        first.chunks = vec![chunk(10, 8192), chunk(12, 16384)];
        // Duplicate references in one file must not release a later file's page.
        let duplicate = first.chunks[1].segments[0].clone();
        first.chunks[1].segments.push(duplicate);
        second.chunks = vec![chunk(11, 8192)];
        third.chunks = vec![chunk(12, 16384)];
        parent.set_cache_limit(crate::CacheLimit::Bytes(1024 * 1024));
        for (offset, byte) in [(8192, 41), (16384, 73)] {
            parent.page_manager.borrow_mut().insert_page(
                offset,
                DecodedPage {
                    page_id: offset,
                    sequence: 1,
                    objects: vec![PageObject::new(
                        PageObjectKind::FileData,
                        offset,
                        vec![byte; 1024],
                    )],
                },
                4096,
            );
        }
        let worker = ExtractionWorker::new(&parent).unwrap().reader;
        let held = worker
            .page_manager
            .borrow_mut()
            .cached_page(8192, PageSecurity::Normal)
            .unwrap()
            .unwrap();
        let weak = Arc::downgrade(&held);
        for frame in [10, 11, 12] {
            let data = vec![frame as u8; 32];
            let mut cache = worker.compression_frame_cache.borrow_mut();
            cache.used_bytes += data.len();
            cache.entries.insert(
                frame,
                super::super::CachedCompressionFrame {
                    native_reference: None,
                    compression: crate::compression::COMPRESSION_NONE,
                    compression_frame_len: 32,
                    compressed_len: 32,
                    compression_frame_digest: [0; 32],
                    slices: vec![],
                    data,
                },
            );
        }
        let parent_bytes = parent.page_manager.borrow().stats().used_bytes;
        let job = vec![(0, &first), (1, &second), (2, &third)];
        let mut uses = ExtractionLastUse::new(&job);
        uses.retire(&worker, &first, 0);
        assert_eq!(worker.compression_frame_cache.borrow().used_bytes, 64);
        assert!(worker
            .page_manager
            .borrow_mut()
            .cached_page(8192, PageSecurity::Normal)
            .unwrap()
            .is_some());
        assert!(worker
            .page_manager
            .borrow_mut()
            .cached_page(16384, PageSecurity::Normal)
            .unwrap()
            .is_some());
        uses.retire(&worker, &second, 1);
        assert_eq!(worker.compression_frame_cache.borrow().used_bytes, 32);
        assert!(worker
            .page_manager
            .borrow_mut()
            .cached_page(8192, PageSecurity::Normal)
            .unwrap()
            .is_none());
        assert!(worker
            .page_manager
            .borrow_mut()
            .cached_page(16384, PageSecurity::Normal)
            .unwrap()
            .is_some());
        assert_eq!(
            held.objects[0].with_payload(|v| v.to_vec()).unwrap(),
            vec![41; 1024]
        );
        uses.retire(&worker, &third, 2);
        assert!(uses.frames.is_empty() && uses.pages.is_empty());
        assert_eq!(worker.compression_frame_cache.borrow().used_bytes, 0);
        assert!(worker
            .page_manager
            .borrow_mut()
            .cached_page(16384, PageSecurity::Normal)
            .unwrap()
            .is_none());
        assert_eq!(
            parent.page_manager.borrow().stats().used_bytes,
            parent_bytes
        );
        assert!(parent
            .page_manager
            .borrow_mut()
            .cached_page(8192, PageSecurity::Normal)
            .unwrap()
            .is_some());
        drop(worker);
        drop(parent);
        assert!(weak.upgrade().is_some());
        drop(held);
        assert!(weak.upgrade().is_none());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn protected_compressed_parallel_extraction_preserves_all_file_bytes() {
        compressed_parallel_extraction_preserves_all_file_bytes(true);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn plaintext_compressed_parallel_extraction_preserves_all_file_bytes() {
        compressed_parallel_extraction_preserves_all_file_bytes(false);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn compressed_parallel_extraction_preserves_all_file_bytes(protected: bool) {
        let root = unique_test_root("protected-parallel-workspace");
        fs::create_dir_all(&root).unwrap();
        let archive = root.join("source.lbox");
        let destination = root.join("extracted");
        let key = [71u8; 32];
        let signer = crate::OwnerSigningKeyPair::generate().unwrap();
        let mut writer = Lockbox::create_file_with_options(
            &archive,
            crate::LockboxCreateOptions::new(
                if protected {
                    crate::Encryption::Encrypted(crate::LockboxProtection::ContentKey(
                        crate::SecretVec::try_from_slice(&key).unwrap(),
                    ))
                } else {
                    crate::Encryption::None
                },
                if protected {
                    crate::Signing::Owner(&signer)
                } else {
                    crate::Signing::None
                },
            ),
        )
        .unwrap();
        for index in 0..256usize {
            let mut data = vec![(index % 251) as u8; 65536];
            data[..8].copy_from_slice(&(index as u64).to_le_bytes());
            writer
                .add_file(
                    &crate::LockboxPath::new(format!("/file-{index:03}.bin")).unwrap(),
                    &data,
                    false,
                )
                .unwrap();
        }
        writer.commit().unwrap();
        drop(writer);
        let archive_before = fs::read(&archive).unwrap();
        let mut reader = Lockbox::open(
            &archive,
            if protected {
                crate::LockboxOpen::ContentKey(crate::SecretVec::try_from_slice(&key).unwrap())
            } else {
                crate::LockboxOpen::Unencrypted
            },
        )
        .unwrap();
        reader.set_workload_profile(crate::WorkloadProfile::ExtractMany);
        reader.set_cache_limit(crate::CacheLimit::Auto);
        let policy = ExtractPolicy::default();
        let entries = reader.validate_extract_plan(&policy).unwrap();
        assert!(reader.should_extract_files_in_parallel(&entries));
        let files: Vec<_> = entries
            .iter()
            .copied()
            .filter(|e| e.node_kind == NodeKind::File)
            .enumerate()
            .collect();
        // This focused native test requires the same multicore host as the
        // existing parallel-extraction tests; do not silently skip the path.
        let workers = std::thread::available_parallelism()
            .unwrap()
            .get()
            .clamp(1, 4);
        assert!(workers > 1);
        let jobs = group_parallel_extraction_jobs(files, workers);
        assert!(jobs.iter().filter(|job| !job.is_empty()).count() > 1);
        assert!(entries
            .iter()
            .flat_map(|entry| &entry.chunks)
            .any(
                |chunk| chunk.compression != crate::compression::COMPRESSION_NONE
                    && chunk.compression_frame_len >= 65536
            ));
        reader.extract_to_directory(&destination, &policy).unwrap();
        assert_eq!(fs::read_dir(&destination).unwrap().count(), 256);
        for index in 0..256usize {
            let mut expected = vec![(index % 251) as u8; 65536];
            expected[..8].copy_from_slice(&(index as u64).to_le_bytes());
            assert_eq!(
                fs::read(destination.join(format!("file-{index:03}.bin"))).unwrap(),
                expected
            );
        }
        assert_eq!(fs::read(&archive).unwrap(), archive_before);
        drop(reader);
        fs::remove_dir_all(&root).unwrap();
    }
}
