use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::path::PathBuf;

use super::Lockbox;
use crate::form::{FormDefinition, FormRecord};
use crate::key_slot::random_content_key;
use crate::key_wrap::ContactPublicKey;
use crate::secret_vec::SecretVec;
use crate::storage::atomic_file_replacement::AtomicFileReplacement;
use crate::toc_entry::TocEntry;
use crate::variable_btree::VariableValue;
use crate::{Error, LockboxEntryKind, LockboxOptions, LockboxPath, Result, VariableName};

type FormState = (
    BTreeMap<String, FormDefinition>,
    BTreeMap<LockboxPath, FormRecord>,
);

struct LiveLockboxContent {
    entries: Vec<TocEntry>,
    variables: BTreeMap<VariableName, VariableValue>,
    forms: FormState,
}

impl LiveLockboxContent {
    fn capture(lockbox: &Lockbox) -> Result<Self> {
        Ok(Self {
            entries: lockbox
                .toc_entries
                .values()
                .filter(|entry| !entry.deleted)
                .cloned()
                .collect(),
            variables: lockbox.clone_all_variable_values()?,
            forms: lockbox.clone_all_form_state()?,
        })
    }
}

/// Owns one complete compaction or content-key replacement transaction.
struct LockboxRewrite<'a> {
    source: &'a mut Lockbox,
    content: LiveLockboxContent,
}

impl<'a> LockboxRewrite<'a> {
    fn capture(source: &'a mut Lockbox) -> Result<Self> {
        let content = LiveLockboxContent::capture(source)?;
        Ok(Self { source, content })
    }

    fn compact(self) -> Result<()> {
        if let Some(path) = self.source.storage.path().map(ToOwned::to_owned) {
            return self.compact_file_backed(path);
        }

        let Self { source, content } = self;
        let key = source.key.try_clone()?;
        let signing_key = source.require_owner_signing_key()?.try_clone()?;
        let mut compacted = Lockbox::create_with_secret_key_and_options(
            key,
            source.lockbox_id,
            Self::options(source),
        );
        compacted.set_owner_signing_key(signing_key);
        Self::populate(source, &mut compacted, content, true)?;
        compacted.commit()?;
        *source = compacted;
        Ok(())
    }

    fn rekey(self, retained_contacts: &[(String, ContactPublicKey)]) -> Result<Vec<(String, u64)>> {
        if let Some(path) = self.source.storage.path().map(ToOwned::to_owned) {
            return self.rekey_file_backed(path, retained_contacts);
        }

        let Self { source, content } = self;
        let key = SecretVec::try_from_slice(&random_content_key()?)?;
        let signing_key = source.require_owner_signing_key()?.try_clone()?;
        let mut rekeyed = Lockbox::create_with_secret_key_and_options(
            key,
            source.lockbox_id,
            Self::options(source),
        );
        rekeyed.set_owner_signing_key(signing_key);
        let slot_ids = Self::add_retained_contacts(&mut rekeyed, retained_contacts)?;
        Self::populate(source, &mut rekeyed, content, false)?;
        rekeyed.commit()?;
        *source = rekeyed;
        Ok(slot_ids)
    }

    fn compact_file_backed(self, path: PathBuf) -> Result<()> {
        let Self { source, content } = self;
        let replacement = AtomicFileReplacement::for_compaction(&path);
        replacement.discard();
        let options = Self::options(source);
        let signing_key = source.require_owner_signing_key()?.try_clone()?;
        let result = (|| {
            let key = source.key.try_clone()?;
            let reopen_key = key.try_clone()?;
            let mut compacted = Lockbox::create_path_with_secret_key_and_options(
                replacement.temp_path(),
                key,
                source.lockbox_id,
                options,
            )?;
            compacted.set_owner_signing_key(signing_key.try_clone()?);
            Self::populate(source, &mut compacted, content, true)?;
            compacted.commit()?;
            drop(compacted);
            replacement.install()?;
            let mut reopened =
                Lockbox::open_path_with_secret_key_options(&path, reopen_key, options)?;
            reopened.set_owner_signing_key(signing_key);
            *source = reopened;
            Ok(())
        })();
        if result.is_err() {
            replacement.discard();
        }
        result
    }

    fn rekey_file_backed(
        self,
        path: PathBuf,
        retained_contacts: &[(String, ContactPublicKey)],
    ) -> Result<Vec<(String, u64)>> {
        let Self { source, content } = self;
        let replacement = AtomicFileReplacement::for_compaction(&path);
        replacement.discard();
        let options = Self::options(source);
        let signing_key = source.require_owner_signing_key()?.try_clone()?;
        let result = (|| {
            let key = SecretVec::try_from_slice(&random_content_key()?)?;
            let reopen_key = key.try_clone()?;
            let mut rekeyed = Lockbox::create_path_with_secret_key_and_options(
                replacement.temp_path(),
                key,
                source.lockbox_id,
                options,
            )?;
            rekeyed.set_owner_signing_key(signing_key.try_clone()?);
            let slot_ids = Self::add_retained_contacts(&mut rekeyed, retained_contacts)?;
            Self::populate(source, &mut rekeyed, content, false)?;
            rekeyed.commit()?;
            drop(rekeyed);
            replacement.install()?;
            let mut reopened =
                Lockbox::open_path_with_secret_key_options(&path, reopen_key, options)?;
            reopened.set_owner_signing_key(signing_key);
            *source = reopened;
            Ok(slot_ids)
        })();
        if result.is_err() {
            replacement.discard();
        }
        result
    }

    fn populate(
        source: &Lockbox,
        destination: &mut Lockbox,
        content: LiveLockboxContent,
        preserve_access: bool,
    ) -> Result<()> {
        if preserve_access {
            destination.key_slots = source.key_slots.clone();
            destination.key_directory.generation = source.key_directory.generation;
            destination.key_directory.dirty = !destination.key_slots.is_empty();
        }

        for (name, value) in content.variables {
            destination.set_variable_value(name, value)?;
        }
        for (key, definition) in content.forms.0 {
            destination.set_form_definition_value(key, definition)?;
        }
        for entry in content
            .entries
            .iter()
            .filter(|entry| entry.entry_kind() == LockboxEntryKind::Directory)
        {
            destination.create_dir(&entry.path, true)?;
            destination.set_permissions(&entry.path, entry.permissions)?;
        }
        for (path, record) in content.forms.1 {
            destination.create_parent_dirs_for(&path)?;
            destination.set_form_record_value(path, record)?;
        }

        for entry in content.entries {
            match entry.entry_kind() {
                LockboxEntryKind::File => {
                    let reader = FileEntryReader::new(source, &entry)?;
                    destination.create_parent_dirs_for(&entry.path)?;
                    destination.add_file_from_reader_with_permissions(
                        &entry.path,
                        reader,
                        entry.permissions,
                        false,
                    )?;
                }
                LockboxEntryKind::Symlink => {
                    let target = source.get_symlink_target(&entry.path)?;
                    destination.create_parent_dirs_for(&entry.path)?;
                    destination.add_symlink(&entry.path, &target, false)?;
                }
                LockboxEntryKind::Directory => {}
            }
        }
        Ok(())
    }

    fn options(lockbox: &Lockbox) -> LockboxOptions {
        LockboxOptions {
            workload_profile: lockbox.workload_profile,
            ..LockboxOptions::default()
        }
    }

    fn add_retained_contacts(
        lockbox: &mut Lockbox,
        retained_contacts: &[(String, ContactPublicKey)],
    ) -> Result<Vec<(String, u64)>> {
        let mut slot_ids = Vec::with_capacity(retained_contacts.len());
        for (name, contact) in retained_contacts {
            let slot_id = lockbox.add_contact_named(Self::access_entry_name(name), contact)?;
            slot_ids.push((name.clone(), slot_id));
        }
        Ok(slot_ids)
    }

    fn access_entry_name(label: &str) -> String {
        label
            .strip_prefix("profile:")
            .or_else(|| label.strip_prefix("contact:"))
            .unwrap_or(label)
            .to_string()
    }
}

struct FileEntryReader<'a> {
    lockbox: &'a Lockbox,
    entry: &'a crate::toc_entry::TocEntry,
    chunks: Vec<crate::file_chunk::FileChunk>,
    next_chunk: usize,
    current: Cursor<Vec<u8>>,
    written: u64,
}

impl<'a> FileEntryReader<'a> {
    fn new(lockbox: &'a Lockbox, entry: &'a crate::toc_entry::TocEntry) -> Result<Self> {
        if let Some(pending) = lockbox.pending_small_files.get(&entry.path) {
            if pending.data.len() as u64 != entry.len {
                return Err(Error::CorruptRecord);
            }
            return Ok(Self {
                lockbox,
                entry,
                chunks: Vec::new(),
                next_chunk: 0,
                current: Cursor::new(pending.data.to_vec()),
                written: 0,
            });
        }
        if entry.chunks.is_empty() {
            return Err(Error::CorruptRecord);
        }
        let mut chunks = entry.chunks.clone();
        chunks.sort_by_key(|chunk| chunk.file_offset);
        Ok(Self {
            lockbox,
            entry,
            chunks,
            next_chunk: 0,
            current: Cursor::new(Vec::new()),
            written: 0,
        })
    }
}

impl Read for FileEntryReader<'_> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let read = self.current.read(out)?;
            if read != 0 {
                self.written = self.written.saturating_add(read as u64);
                return Ok(read);
            }
            if self.next_chunk >= self.chunks.len() {
                if self.written == self.entry.len {
                    return Ok(0);
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "lockbox file length mismatch during compaction",
                ));
            }
            let chunk = &self.chunks[self.next_chunk];
            if chunk.file_offset != self.written {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "lockbox file chunk offset mismatch during compaction",
                ));
            }
            self.next_chunk += 1;
            let decoded = self
                .lockbox
                .read_file_chunk_compression_frame(self.entry.len, chunk)
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
            self.current = Cursor::new(decoded);
        }
    }
}

impl Lockbox {
    pub(crate) fn compact(&mut self) -> Result<()> {
        LockboxRewrite::capture(self)?.compact()
    }

    /// Replace the lockbox content key and grant access to the supplied contacts.
    ///
    /// This is the low-level primitive for true revocation. It rewrites the
    /// archive with a fresh content key and creates a new key directory
    /// containing only `retained_contacts`. Password slots and contacts not
    /// supplied by the caller are intentionally not preserved.
    pub fn replace_content_key_with_contacts(
        &mut self,
        retained_contacts: &[(String, ContactPublicKey)],
    ) -> Result<Vec<(String, u64)>> {
        if retained_contacts.is_empty() {
            return Err(Error::SecurityLimitExceeded(
                "refusing to rekey without retained access".to_string(),
            ));
        }
        LockboxRewrite::capture(self)?.rekey(retained_contacts)
    }
}
