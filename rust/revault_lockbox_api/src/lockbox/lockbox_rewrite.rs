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
        let signing_key = source
            .owner_signing_key
            .as_ref()
            .map(crate::OwnerSigningKeyPair::try_clone)
            .transpose()?;
        let mut compacted = Lockbox::create_with_secret_key_and_options(
            key,
            source.lockbox_id,
            Self::options(source),
        );
        compacted.owner_signing_key = signing_key;
        compacted.set_creation_format(source.format_mode);
        Self::populate(source, &mut compacted, content, true)?;
        compacted.commit()?;
        Self::verify(source, &compacted, true)?;
        *source = compacted;
        Ok(())
    }

    fn rekey(
        self,
        retained_contacts: &[(String, ContactPublicKey)],
        retained_passwords: &[(String, crate::SecretString)],
    ) -> Result<Vec<(String, u64)>> {
        if let Some(path) = self.source.storage.path().map(ToOwned::to_owned) {
            return self.rekey_file_backed(path, retained_contacts, retained_passwords);
        }

        let Self { source, content } = self;
        let key = SecretVec::try_from_slice(&random_content_key()?)?;
        let signing_key = source
            .owner_signing_key
            .as_ref()
            .map(crate::OwnerSigningKeyPair::try_clone)
            .transpose()?;
        let mut rekeyed = Lockbox::create_with_secret_key_and_options(
            key,
            source.lockbox_id,
            Self::options(source),
        );
        rekeyed.owner_signing_key = signing_key;
        rekeyed.set_creation_format(source.format_mode);
        let slot_ids =
            Self::add_retained_contacts(&mut rekeyed, retained_contacts, retained_passwords)?;
        Self::populate(source, &mut rekeyed, content, false)?;
        rekeyed.commit()?;
        Self::verify(source, &rekeyed, false)?;
        *source = rekeyed;
        Ok(slot_ids)
    }

    fn compact_file_backed(self, path: PathBuf) -> Result<()> {
        let Self { source, content } = self;
        if std::fs::symlink_metadata(&path)
            .map_err(|e| Error::Io(e.to_string()))?
            .file_type()
            .is_symlink()
        {
            return Err(Error::InvalidOperation(
                "open the archive's real path before compacting or rekeying a symlink".into(),
            ));
        }
        let replacement = AtomicFileReplacement::for_compaction(&path)?;
        let mut owns_temporary = false;
        let options = Self::options(source);
        let signing_key = source
            .owner_signing_key
            .as_ref()
            .map(crate::OwnerSigningKeyPair::try_clone)
            .transpose()?;
        let result = (|| {
            let key = source.key.try_clone()?;
            let mut compacted = Lockbox::create_path_with_secret_key_and_options(
                replacement.temp_path(),
                key,
                source.lockbox_id,
                options,
            )?;
            owns_temporary = true;
            std::fs::set_permissions(
                replacement.temp_path(),
                std::fs::metadata(&path)
                    .map_err(|e| Error::Io(e.to_string()))?
                    .permissions(),
            )
            .map_err(|e| Error::Io(e.to_string()))?;
            compacted.owner_signing_key = signing_key
                .as_ref()
                .map(crate::OwnerSigningKeyPair::try_clone)
                .transpose()?;
            compacted.set_creation_format(source.format_mode);
            Self::populate(source, &mut compacted, content, true)?;
            #[cfg(test)]
            compaction_tests::checkpoint("prepared");
            compacted.commit()?;
            Self::verify(source, &compacted, true)?;
            #[cfg(test)]
            compaction_tests::checkpoint("verified");
            source.storage.ensure_current()?;
            replacement.publish()?;
            compacted.storage.relocate(&path);
            *source = compacted;
            #[cfg(test)]
            compaction_tests::checkpoint("published");
            source.poisoned = Some(
                "compaction publication durability is uncertain; reopen before continuing".into(),
            );
            replacement.sync_parent()?;
            source.poisoned = None;
            Ok(())
        })();
        if result.is_err() && owns_temporary {
            replacement.discard();
        }
        result
    }

    fn rekey_file_backed(
        self,
        path: PathBuf,
        retained_contacts: &[(String, ContactPublicKey)],
        retained_passwords: &[(String, crate::SecretString)],
    ) -> Result<Vec<(String, u64)>> {
        let Self { source, content } = self;
        if std::fs::symlink_metadata(&path)
            .map_err(|e| Error::Io(e.to_string()))?
            .file_type()
            .is_symlink()
        {
            return Err(Error::InvalidOperation(
                "open the archive's real path before compacting or rekeying a symlink".into(),
            ));
        }
        let replacement = AtomicFileReplacement::for_compaction(&path)?;
        let mut owns_temporary = false;
        let options = Self::options(source);
        let signing_key = source
            .owner_signing_key
            .as_ref()
            .map(crate::OwnerSigningKeyPair::try_clone)
            .transpose()?;
        let result = (|| {
            let key = SecretVec::try_from_slice(&random_content_key()?)?;
            let mut rekeyed = Lockbox::create_path_with_secret_key_and_options(
                replacement.temp_path(),
                key,
                source.lockbox_id,
                options,
            )?;
            owns_temporary = true;
            std::fs::set_permissions(
                replacement.temp_path(),
                std::fs::metadata(&path)
                    .map_err(|e| Error::Io(e.to_string()))?
                    .permissions(),
            )
            .map_err(|e| Error::Io(e.to_string()))?;
            rekeyed.owner_signing_key = signing_key
                .as_ref()
                .map(crate::OwnerSigningKeyPair::try_clone)
                .transpose()?;
            rekeyed.set_creation_format(source.format_mode);
            let slot_ids =
                Self::add_retained_contacts(&mut rekeyed, retained_contacts, retained_passwords)?;
            Self::populate(source, &mut rekeyed, content, false)?;
            rekeyed.commit()?;
            Self::verify(source, &rekeyed, false)?;
            source.storage.ensure_current()?;
            replacement.publish()?;
            rekeyed.storage.relocate(&path);
            *source = rekeyed;
            source.poisoned =
                Some("rekey publication durability is uncertain; reopen before continuing".into());
            replacement.sync_parent()?;
            source.poisoned = None;
            Ok(slot_ids)
        })();
        if result.is_err() && owns_temporary {
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
                    destination.set_permissions(&entry.path, entry.permissions)?;
                }
                LockboxEntryKind::Directory => {}
            }
        }
        // Mirror ownership is stored in variables. Restore it only after the
        // owned tree has been reconstructed, so ordinary path guards do not
        // reject the trusted rewrite of that tree.
        for (name, value) in content.variables {
            destination.set_variable_value(name, value)?;
        }
        Ok(())
    }

    fn verify(source: &Lockbox, destination: &Lockbox, preserve_access: bool) -> Result<()> {
        // Read persisted pages through a fresh cache, while retaining the same
        // locked file descriptor. Never trust only the writer's staged maps.
        let verified = Lockbox::open_storage_with_secret_key_mode(
            destination.storage.clone(),
            destination.key.try_clone()?,
            Self::options(destination),
            false,
        )?;
        verified.inspector().verify_storage()?;
        if source.lockbox_id != verified.lockbox_id
            || source.format_mode != verified.format_mode
            || source.clone_all_variable_values()? != verified.clone_all_variable_values()?
            || source.clone_all_form_state()? != verified.clone_all_form_state()?
        {
            return Err(Error::CorruptRecord);
        }
        if preserve_access && encode_access(source)? != encode_access(&verified)? {
            return Err(Error::CorruptRecord);
        }
        let expected = source.toc_entries.values().filter(|entry| !entry.deleted);
        if expected.clone().count()
            != verified
                .toc_entries
                .values()
                .filter(|entry| !entry.deleted)
                .count()
        {
            return Err(Error::CorruptRecord);
        }
        for entry in expected {
            let stored = verified
                .toc_entries
                .get(&entry.path)
                .ok_or(Error::CorruptRecord)?;
            if stored.deleted
                || entry.entry_kind() != stored.entry_kind()
                || entry.len != stored.len
                || entry.permissions != stored.permissions
            {
                return Err(Error::CorruptRecord);
            }
            match entry.entry_kind() {
                LockboxEntryKind::File => {
                    let mut original = FileEntryReader::new(source, entry)?;
                    let mut rewritten = FileEntryReader::new(&verified, stored)?;
                    let mut left = [0u8; 64 * 1024];
                    let mut right = [0u8; 64 * 1024];
                    let mut remaining = entry.len;
                    while remaining > 0 {
                        let size = remaining.min(left.len() as u64) as usize;
                        original
                            .read_exact(&mut left[..size])
                            .map_err(|e| Error::Io(e.to_string()))?;
                        rewritten
                            .read_exact(&mut right[..size])
                            .map_err(|e| Error::Io(e.to_string()))?;
                        if left[..size] != right[..size] {
                            return Err(Error::CorruptRecord);
                        }
                        remaining -= size as u64;
                    }
                    if original
                        .read(&mut left[..1])
                        .map_err(|e| Error::Io(e.to_string()))?
                        != 0
                        || rewritten
                            .read(&mut right[..1])
                            .map_err(|e| Error::Io(e.to_string()))?
                            != 0
                    {
                        return Err(Error::CorruptRecord);
                    }
                }
                LockboxEntryKind::Symlink => {
                    if source.get_symlink_target(&entry.path)?
                        != verified.get_symlink_target(&entry.path)?
                    {
                        return Err(Error::CorruptRecord);
                    }
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
        retained_passwords: &[(String, crate::SecretString)],
    ) -> Result<Vec<(String, u64)>> {
        let mut slot_ids = Vec::with_capacity(retained_contacts.len());
        for (name, contact) in retained_contacts {
            let slot_id = lockbox.add_contact_named(Self::access_entry_name(name), contact)?;
            slot_ids.push((name.clone(), slot_id));
        }
        for (name, password) in retained_passwords {
            let id = lockbox.add_password(password)?;
            slot_ids.push((name.clone(), id));
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

fn encode_access(lockbox: &Lockbox) -> Result<Vec<u8>> {
    crate::key_directory::encode_key_directory(&lockbox.key_slots, lockbox.lockbox_id, 0, 0)
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
    /// Rebuild the archive from its live state, committing pending changes.
    ///
    /// Preserves identity, format choices, access slots, owner, and all live
    /// records. Old commit history and reusable space are discarded. File-backed
    /// archives use a verified, synced replacement in the same directory and
    /// require temporary disk space for that replacement. An interrupted write
    /// leaves the original intact; publication is an atomic rename.
    pub fn compact(&mut self) -> Result<()> {
        self.require_clean_transaction()?;
        self.validate_owner_signing_key()?;
        if self.read_only || self.poisoned.is_some() {
            return Err(Error::InvalidOperation(
                "compaction requires a writable, unpoisoned lockbox".into(),
            ));
        }
        self.storage.ensure_current()?;
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
        self.replace_content_key_with_access(retained_contacts, &[])
    }

    /// Atomically replaces the content key and reconstructs retained contact and password slots.
    pub fn replace_content_key_with_access(
        &mut self,
        retained_contacts: &[(String, ContactPublicKey)],
        retained_passwords: &[(String, crate::SecretString)],
    ) -> Result<Vec<(String, u64)>> {
        if self.format_mode.plaintext() {
            return Err(Error::InvalidOperation(
                "unencrypted lockboxes have no content key to rotate".into(),
            ));
        }
        if retained_contacts.is_empty() && retained_passwords.is_empty() {
            return Err(Error::SecurityLimitExceeded(
                "refusing to rekey without retained access".to_string(),
            ));
        }
        LockboxRewrite::capture(self)?.rekey(retained_contacts, retained_passwords)
    }
}

#[cfg(test)]
mod compaction_tests {
    use super::*;
    use crate::storage::Storage;
    use std::io::{BufRead, Write};

    #[cfg(unix)]
    #[test]
    fn compaction_preserves_permissions_and_refuses_stale_or_symlink_paths() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!(
            "revault-compact-paths-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("box.lbox");
        let mut lb = Lockbox::create_path(&path, "permission fixture").unwrap();
        lb.add_file(
            &LockboxPath::new("/keep").unwrap(),
            b"private contents",
            false,
        )
        .unwrap();
        lb.commit().unwrap();
        let signer = lb.owner_signing_key.as_ref().unwrap().try_clone().unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        lb.compact().unwrap();
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        drop(lb);
        let before = std::fs::read(&path).unwrap();
        let link = dir.join("linked.lbox");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        let mut linked = Lockbox::open_path(&link, "permission fixture").unwrap();
        linked.set_owner_signing_key(signer.try_clone().unwrap());
        assert!(linked.compact().is_err());
        assert!(std::fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink());
        assert!(std::fs::read(&path).unwrap() == before);
        drop(linked);
        let mut stale = Lockbox::open_path(&path, "permission fixture").unwrap();
        stale.set_owner_signing_key(signer);
        let replacement = dir.join("external");
        std::fs::write(&replacement, b"replacement owned by someone else").unwrap();
        std::fs::rename(replacement, &path).unwrap();
        assert!(matches!(stale.compact(), Err(Error::LockUnavailable(_))));
        assert_eq!(
            std::fs::read(&path).unwrap(),
            b"replacement owned by someone else"
        );
        drop(stale);
        std::fs::remove_dir_all(dir).unwrap();
    }

    // Only present in the unit-test executable, never in the shipped CLI/API.
    pub(super) fn checkpoint(point: &str) {
        if std::env::var("REV313_COMPACT_STOP").as_deref() == Ok(point) {
            println!("REV313_COMPACT_READY");
            std::io::stdout().flush().unwrap();
            loop {
                std::thread::park();
            }
        }
    }

    #[test]
    fn compaction_process_child() {
        let Ok(path) = std::env::var("REV313_COMPACT_PATH") else {
            return;
        };
        let mut lb = Lockbox::create_path(&path, "compact process fixture").unwrap();
        lb.add_file(
            &LockboxPath::new("/keep").unwrap(),
            b"committed survivor",
            false,
        )
        .unwrap();
        let mut state = 123456789u64;
        let payload: Vec<u8> = (0..3 * 1024 * 1024)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state as u8
            })
            .collect();
        lb.add_file(&LockboxPath::new("/remove").unwrap(), &payload, false)
            .unwrap();
        lb.commit().unwrap();
        lb.delete(&LockboxPath::new("/remove").unwrap()).unwrap();
        lb.commit().unwrap();
        std::fs::copy(&path, format!("{path}.baseline")).unwrap();
        lb.compact().unwrap();
        panic!("compaction checkpoint was not reached");
    }

    #[test]
    fn compaction_survives_process_death_before_and_after_replacement() {
        for point in ["prepared", "verified", "published"] {
            let dir = std::env::temp_dir().join(format!(
                "revault-compact-death-{}-{point}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join("box.lbox");
            let mut child = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "lockbox::lockbox_rewrite::compaction_tests::compaction_process_child",
                    "--nocapture",
                ])
                .env("REV313_COMPACT_STOP", point)
                .env("REV313_COMPACT_PATH", &path)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::inherit())
                .spawn()
                .unwrap();
            let stdout = child.stdout.take().unwrap();
            let (send, recv) = std::sync::mpsc::channel();
            let reader = std::thread::spawn(move || {
                for line in std::io::BufReader::new(stdout).lines() {
                    if line.unwrap().contains("REV313_COMPACT_READY") {
                        send.send(()).unwrap();
                        return;
                    }
                }
            });
            let ready = recv.recv_timeout(std::time::Duration::from_secs(30));
            let _ = child.kill();
            child.wait().unwrap();
            reader.join().unwrap();
            ready.expect("child did not reach compaction checkpoint");
            let baseline = std::fs::read(path.with_file_name("box.lbox.baseline")).unwrap();
            let actual = std::fs::read(&path).unwrap();
            if point == "published" {
                assert!(actual.len() + 2 * 1024 * 1024 < baseline.len());
            } else {
                assert!(actual == baseline);
            }
            let opened = Lockbox::open_path(&path, "compact process fixture").unwrap();
            assert_eq!(
                opened
                    .get_file(&LockboxPath::new("/keep").unwrap())
                    .unwrap(),
                b"committed survivor"
            );
            assert!(opened
                .get_file(&LockboxPath::new("/remove").unwrap())
                .is_err());
            opened.inspector().verify_storage().unwrap();
            drop(opened);
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn compaction_refuses_a_poisoned_handle_without_changing_storage() {
        let mut lb = Lockbox::create("compaction failure fixture");
        lb.add_file(&LockboxPath::new("/keep").unwrap(), b"original", false)
            .unwrap();
        lb.commit().unwrap();
        let before = lb.storage.read_all().unwrap();
        lb.poisoned = Some("ambiguous publication".into());
        assert!(lb.compact().is_err());
        assert!(lb.storage.read_all().unwrap() == before);
    }
}
