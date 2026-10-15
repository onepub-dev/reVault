//! The authenticated base free index is a conservative reservation journal.
//! The fixed header also reserves the append stream starting at sealed_len.
//! Neither journal component is overwritten before publication or rollback.
//!
//! Protocol ordering:
//! 1. Authenticate the base roots, free index, physical extents and sealed EOF.
//! 2. Publish preparation into both header slots, syncing each before payload
//!    writes. The entire base free list and everything appended after EOF are
//!    conservative reservations, so the journal cannot overflow mid-write.
//! 3. Write payloads and new metadata without overwriting the committed base.
//!    Flush them before publishing the new root. An unpublished interruption
//!    erases reserved reusable ranges and truncates the append stream back to
//!    the verified base, retaining the original commit and signing owner.
//! 4. Once the new root is published, recovery rolls forward: obsolete extents
//!    are zeroed in bounded buffers, with durable manifest checkpoints, then
//!    the post-cleanup free index becomes reusable. Never roll back a published
//!    commit or infer committed data by scanning abandoned pages.
//! 5. Tail reclamation, when possible, is a separate authenticated transition
//!    described in `tail_reclamation`; compaction verifies a fresh archive
//!    before atomically replacing the original and discarding retained history.
use super::Lockbox;
use crate::file_format::current_header::{publish_header, HEADER_LEN};
use crate::file_format::header_v2::{self, Publication};
use crate::storage::Storage;
use crate::{
    Error, Result, TransactionRecoveryControl, TransactionRecoveryPhase,
    TransactionRecoveryProgress,
};

impl<State> Lockbox<State> {
    pub(crate) fn begin_preparation(&mut self) -> Result<()> {
        self.require_clean_transaction()?;
        if self.preparing {
            return Ok(());
        }
        let header = crate::file_format::read_header(&self.storage.read_at(0, HEADER_LEN)?)?;
        let generation = header
            .generation
            .checked_add(1)
            .ok_or(Error::CorruptHeader)?;
        let mut publication = Publication {
            sealed_len: header.sealed_len,
            preparing: true,
            trim_origin_len: header.trim_origin_len,
            format_mode: header.format_mode,
            generation,
            commit_root_offset: header.commit_root_offset,
            sequence: header.sequence,
            key_directory_offset: header.key_directory_offset,
            key_directory_mirror_offset: header.key_directory_mirror_offset,
            lockbox_id: header.lockbox_id,
            commit_auth_offset: header.commit_auth_offset,
            cleanup_sequence: header.cleanup_sequence,
            cleanup_completed_ranges: 0,
            cleanup_completed_pages: 0,
            cleanup_completed_bytes: 0,
            metadata_auth_tag: [0; 24],
        };
        publication.metadata_auth_tag = self.key.with_bytes(|key| {
            crate::crypto::metadata_auth_tag(key, &header_v2::metadata_auth_message(publication))
        })?;
        self.poisoned =
            Some("preparation publication was interrupted; reopen before continuing".into());
        let slot = publish_header(&mut self.storage, header.slot_index, publication)?;
        self.header_slot = slot;
        self.header_generation = generation;
        self.transaction_start_len = header.sealed_len;
        self.preparing = true;
        // Both independently valid slots must advertise preparation before a
        // reused free page can become nonzero. A torn/corrupt intent slot must
        // never expose an older clean header over dirty reusable storage.
        publication.generation = generation.checked_add(1).ok_or(Error::CorruptHeader)?;
        publication.metadata_auth_tag = self.key.with_bytes(|key| {
            crate::crypto::metadata_auth_tag(key, &header_v2::metadata_auth_message(publication))
        })?;
        let slot = publish_header(&mut self.storage, slot, publication)?;
        self.header_slot = slot;
        self.header_generation = publication.generation;
        self.poisoned = None;
        Ok(())
    }
}

impl Lockbox<crate::Writable> {
    pub(crate) fn rollback_preparation_controlled(
        &mut self,
        mut progress: impl FnMut(TransactionRecoveryProgress) -> TransactionRecoveryControl,
    ) -> Result<bool> {
        let status = self
            .transaction_recovery_status()
            .ok_or(Error::CorruptHeader)?;
        self.validate_base_reservations()?;
        // A content-key holder can authenticate header metadata without being
        // the signing owner. Even with no reusable slots, a forged base length
        // must never allow recovery to truncate a committed live allocation.
        if self
            .storage_inventory(true)?
            .ranges
            .iter()
            .any(|(&offset, &len)| {
                offset
                    .checked_add(len)
                    .is_none_or(|end| end > self.transaction_start_len)
            })
        {
            return Err(Error::CorruptRecord);
        }
        let slots = self.free_space.slots_by_offset();
        let mut expected_bytes = 0u64;
        let mut previous_end = HEADER_LEN as u64;
        // Validate the complete reservation set before the first erase.
        for (index, slot) in slots.iter().enumerate() {
            let end = slot
                .offset
                .checked_add(slot.len)
                .ok_or(Error::CorruptRecord)?;
            if slot.len == 0 || slot.offset < previous_end || end > self.transaction_start_len {
                return Err(Error::CorruptRecord);
            }
            previous_end = end;
            if index < status.completed_ranges as usize {
                expected_bytes = expected_bytes
                    .checked_add(slot.len)
                    .ok_or(Error::CorruptRecord)?;
            }
        }
        if status.completed_ranges as usize > slots.len()
            || expected_bytes != status.completed_bytes
        {
            return Err(Error::CorruptHeader);
        }
        self.poisoned = Some("rollback was interrupted; reopen before continuing".into());
        let zeros = [0u8; 64 * 1024];
        for (index, slot) in slots
            .iter()
            .enumerate()
            .skip(status.completed_ranges as usize)
        {
            let mut cursor = slot.offset;
            let end = cursor + slot.len;
            while cursor < end {
                let len = (end - cursor).min(zeros.len() as u64) as usize;
                self.storage.write_at(cursor, &zeros[..len])?;
                cursor += len as u64;
            }
            self.storage.sync()?;
            self.cleanup_completed_ranges = index as u32 + 1;
            self.cleanup_completed_pages = self.cleanup_completed_ranges;
            self.cleanup_completed_bytes += slot.len;
            self.publish_transaction_header(self.cleanup_sequence)?;
            if progress(TransactionRecoveryProgress {
                phase: TransactionRecoveryPhase::Rollback,
                completed_ranges: self.cleanup_completed_ranges,
                total_ranges: status.range_count,
                completed_pages: self.cleanup_completed_ranges,
                total_pages: status.page_count,
                completed_bytes: self.cleanup_completed_bytes,
                total_bytes: status.total_bytes,
            }) == TransactionRecoveryControl::Cancel
            {
                return Ok(false);
            }
            self.poisoned = Some("rollback was interrupted; reopen before continuing".into());
        }
        // Truncation cannot remove this fixed-header journal or its base index.
        self.storage.truncate(self.transaction_start_len)?;
        self.storage.sync()?;
        if progress(TransactionRecoveryProgress {
            phase: TransactionRecoveryPhase::Rollback,
            completed_ranges: status.range_count,
            total_ranges: status.range_count,
            completed_pages: status.page_count,
            total_pages: status.page_count,
            completed_bytes: status.total_bytes,
            total_bytes: status.total_bytes,
        }) == TransactionRecoveryControl::Cancel
        {
            return Ok(false);
        }
        self.preparing = false;
        self.cleanup_completed_ranges = 0;
        self.cleanup_completed_pages = 0;
        self.cleanup_completed_bytes = 0;
        self.publish_transaction_header(self.sequence)?;
        self.rollback_required = false;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollback_rejects_a_shortened_base_before_mutating_storage() {
        let mut lb = Lockbox::create("secret");
        lb.add_file(&path("/keep"), b"committed bytes", false)
            .unwrap();
        lb.commit().unwrap();
        lb.begin_preparation().unwrap();
        lb.transaction_start_len = HEADER_LEN as u64;
        lb.publish_transaction_header(lb.cleanup_sequence).unwrap();
        lb.publish_transaction_header(lb.cleanup_sequence).unwrap();
        let malicious = lb.to_bytes();
        // Model a valid metadata MAC with an invalid truncation boundary.
        // Recovery must independently prove ownership, not trust the MAC alone.
        let reopened = Lockbox::open_storage_with_secret_key_mode(
            crate::storage::StorageBackend::memory(malicious.clone()),
            crate::SecretVec::try_from_slice(b"secret").unwrap(),
            crate::LockboxOptions::default(),
            true,
        );
        if let Ok(mut opened) = reopened {
            assert!(opened.complete_pending_transaction_cleanup().is_err());
            assert!(opened.to_bytes() == malicious);
        }
    }
    use crate::{LockboxPath, VariableName};

    fn path(value: &str) -> LockboxPath {
        LockboxPath::new(value).unwrap()
    }
    fn noise(len: usize) -> Vec<u8> {
        let mut state = 123456789u64;
        (0..len)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state as u8
            })
            .collect()
    }
    fn recover(bytes: Vec<u8>) -> Lockbox {
        let mut lb = Lockbox::open_storage_with_secret_key_mode(
            crate::storage::StorageBackend::memory(bytes),
            crate::SecretVec::try_from_slice(b"secret").unwrap(),
            crate::LockboxOptions::default(),
            true,
        )
        .unwrap();
        lb.complete_pending_transaction_cleanup().unwrap();
        lb
    }

    #[test]
    fn secure_page_reuse_preserves_neighboring_file_bytes() {
        let mut lb = Lockbox::create("secret");
        let data = noise(262144);
        lb.add_file(&path("/remove"), b"small", false).unwrap();
        lb.commit().unwrap();
        lb.add_file(&path("/keep"), &data, false).unwrap();
        lb.commit().unwrap();
        lb.delete(&path("/remove")).unwrap();
        lb.commit().unwrap();
        lb.define_form(
            "login",
            "Login",
            vec![crate::FormFieldDefinition {
                id: "name".into(),
                label: "Name".into(),
                kind: crate::FormFieldKind::Text,
                required: false,
            }],
        )
        .unwrap();
        lb.create_form_record(&path("/account"), "login", "Account")
            .unwrap();
        let name = VariableName::new("/review").unwrap();
        let baseline = lb.storage.len().unwrap();
        let initial_history = lb.storage_inventory(true).unwrap().history_bytes;
        for iteration in 0..100 {
            let value = format!(
                "{iteration}:{}",
                "x".repeat(if iteration % 2 == 0 { 16384 } else { 1 })
            );
            lb.set_variable(&name, &value).unwrap();
            lb.set_form_field_normal(&path("/account"), "name", &value)
                .unwrap();
            let changing = noise(if iteration % 2 == 0 { 128 * 1024 } else { 1 });
            lb.add_file(&path("/changing"), &changing, iteration != 0)
                .unwrap();
            lb.commit().unwrap();
            let opened = Lockbox::open_bytes_with_key(lb.to_bytes(), "secret").unwrap();
            assert_eq!(opened.get_file(&path("/keep")).unwrap(), data);
            assert_eq!(opened.get_file(&path("/changing")).unwrap(), changing);
            assert_eq!(
                opened.get_variable(&name).unwrap().as_deref(),
                Some(value.as_str())
            );
            assert!(
                matches!(opened.get_form_field(&path("/account"), "name").unwrap().unwrap().value, crate::FormValue::Normal(v) if v == value)
            );
            opened.inspector().verify_storage().unwrap();
            for slot in opened.free_space.slots_by_offset() {
                assert!(opened
                    .storage
                    .read_at(slot.offset, slot.len as usize)
                    .unwrap()
                    .iter()
                    .all(|byte| *byte == 0));
            }
        }
        let history_growth = lb.storage_inventory(true).unwrap().history_bytes - initial_history;
        assert!(
            lb.storage.len().unwrap() <= baseline + history_growth + 4 * 1024 * 1024,
            "reusable allocations grew without bound over 100 mixed-record updates"
        );
    }

    #[test]
    fn abandoned_stream_requires_rollback_and_truncates_on_recovery() {
        let mut lb = Lockbox::create("secret");
        lb.add_file(&path("/keep"), b"committed", false).unwrap();
        lb.commit().unwrap();
        let sealed_len = lb.storage.len().unwrap();
        lb.add_file(&path("/abandoned"), &noise(2 * 1024 * 1024), false)
            .unwrap();
        lb.flush_dirty_pages().unwrap();
        assert!(lb.storage.len().unwrap() > sealed_len);
        let bytes = lb.to_bytes();
        assert!(matches!(
            Lockbox::open_bytes_with_key(bytes.clone(), "secret"),
            Err(Error::RecoveryRequired { .. })
        ));
        let recovered = recover(bytes);
        assert_eq!(recovered.storage.len().unwrap(), sealed_len);
        assert_eq!(recovered.get_file(&path("/keep")).unwrap(), b"committed");
        assert!(recovered.get_file(&path("/abandoned")).is_err());
        assert!(recover(recovered.to_bytes())
            .transaction_recovery_status()
            .is_none());
    }

    #[test]
    fn rollback_failure_is_resumable_at_every_storage_operation() {
        let mut base = Lockbox::create("secret");
        base.add_file(&path("/keep"), b"committed", false).unwrap();
        base.add_file(&path("/remove"), &noise(256 * 1024), false)
            .unwrap();
        base.commit().unwrap();
        base.delete(&path("/remove")).unwrap();
        base.commit().unwrap();
        let sealed_len = base.storage.len().unwrap();
        base.add_file(&path("/abandoned"), &noise(512 * 1024), false)
            .unwrap();
        base.flush_dirty_pages().unwrap();
        let pending = base.to_bytes();
        let open = || {
            Lockbox::open_storage_with_secret_key_mode(
                crate::storage::StorageBackend::memory(pending.clone()),
                crate::SecretVec::try_from_slice(b"secret").unwrap(),
                crate::LockboxOptions::default(),
                true,
            )
            .unwrap()
        };
        let mut successful = open();
        successful.storage.reset_memory_operation_count();
        successful.complete_pending_transaction_cleanup().unwrap();
        let count = successful.storage.memory_operation_count();
        for failure in 0..count {
            let mut interrupted = open();
            interrupted
                .storage
                .fail_memory_operation_after_successes(failure);
            assert!(
                interrupted.complete_pending_transaction_cleanup().is_err(),
                "failure {failure}"
            );
            let recovered = recover(interrupted.to_bytes());
            assert_eq!(
                recovered.storage.len().unwrap(),
                sealed_len,
                "failure {failure}"
            );
            assert_eq!(recovered.get_file(&path("/keep")).unwrap(), b"committed");
            assert!(recovered.get_file(&path("/abandoned")).is_err());
            for slot in recovered.free_space.slots_by_offset() {
                assert!(recovered
                    .storage
                    .read_at(slot.offset, slot.len as usize)
                    .unwrap()
                    .iter()
                    .all(|byte| *byte == 0));
            }
        }
    }
}

#[cfg(test)]
mod process_tests {
    use super::*;
    use std::io::{BufRead, Write};
    use std::process::{Command, Stdio};
    fn password() -> crate::SecretString {
        crate::SecretString::try_from_slice(b"synthetic crash test").unwrap()
    }
    fn path(value: &str) -> crate::LockboxPath {
        crate::LockboxPath::new(value).unwrap()
    }
    fn checkpoint() -> ! {
        println!("REV313_READY");
        std::io::stdout().flush().unwrap();
        loop {
            std::thread::park();
        }
    }
    #[test]
    fn preparation_process_child() {
        let Some(file) = std::env::var_os("REV313_PROCESS_FILE") else {
            return;
        };
        let file = std::path::Path::new(&file);
        let password = password();
        if std::env::var("REV313_PROCESS_ROLE").unwrap() == "prepare" {
            // Unit-only access opens for staging without needing to transfer
            // an owner private key to a process that will never commit.
            let mut lb =
                Lockbox::open_file_opened_for_write(file, crate::LockboxOpen::Password(&password))
                    .unwrap();
            lb.add_file(&path("/reused"), b"unpublished", false)
                .unwrap();
            lb.flush_pending_small_files().unwrap();
            lb.flush_dirty_pages().unwrap();
            lb.add_file(&path("/appended"), &vec![42; 2 * 1024 * 1024], false)
                .unwrap();
            lb.flush_dirty_pages().unwrap();
            checkpoint();
        } else {
            Lockbox::recover_transaction_controlled(
                file,
                crate::LockboxOpen::Password(&password),
                |_| checkpoint(),
            )
            .unwrap();
        }
    }
    fn kill_at_checkpoint(file: &std::path::Path, role: &str) {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "lockbox::preparation::process_tests::preparation_process_child",
                "--nocapture",
            ])
            .env("REV313_PROCESS_FILE", file)
            .env("REV313_PROCESS_ROLE", role)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (send, receive) = std::sync::mpsc::channel();
        let reader = std::thread::spawn(move || {
            let found = std::io::BufReader::new(stdout)
                .lines()
                .any(|line| line.is_ok_and(|line| line == "REV313_READY"));
            let _ = send.send(found);
        });
        let ready = receive.recv_timeout(std::time::Duration::from_secs(30));
        let _ = child.kill();
        let status = child.wait().unwrap();
        reader.join().unwrap();
        assert!(
            matches!(ready, Ok(true)),
            "child did not reach {role} checkpoint: {status}"
        );
        assert!(!status.success());
    }
    #[test]
    fn preparation_and_rollback_survive_real_process_death() {
        let file = std::env::temp_dir().join(format!(
            "revault-313-kill-{}-{}.lbox",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let password = password();
        let signer = crate::OwnerSigningKeyPair::generate().unwrap();
        let mut base = Lockbox::create_file(
            &file,
            crate::LockboxProtection::Password(&password),
            &signer,
        )
        .unwrap();
        base.add_file(&path("/keep"), b"committed bytes", false)
            .unwrap();
        base.add_file(&path("/remove"), b"reusable bytes", false)
            .unwrap();
        base.commit().unwrap();
        base.delete(&path("/remove")).unwrap();
        base.commit().unwrap();
        let sealed_len = base.storage.len().unwrap();
        drop(base);
        kill_at_checkpoint(&file, "prepare");
        assert!(matches!(
            Lockbox::open(&file, crate::LockboxOpen::Password(&password)),
            Err(Error::RecoveryRequired { .. })
        ));
        kill_at_checkpoint(&file, "rollback");
        assert_eq!(
            Lockbox::inspect_transaction_recovery(&file, crate::LockboxOpen::Password(&password))
                .unwrap()
                .unwrap()
                .phase,
            TransactionRecoveryPhase::Rollback
        );
        assert!(Lockbox::recover_transaction(
            &file,
            crate::LockboxOpen::Password(&password),
            |_| {}
        )
        .unwrap());
        let opened = Lockbox::open(&file, crate::LockboxOpen::Password(&password)).unwrap();
        assert_eq!(opened.get_file(&path("/keep")).unwrap(), b"committed bytes");
        assert!(opened.get_file(&path("/reused")).is_err());
        assert!(opened.get_file(&path("/appended")).is_err());
        assert_eq!(opened.storage.len().unwrap(), sealed_len);
        opened.inspector().verify_storage().unwrap();
        drop(opened);
        assert!(!Lockbox::recover_transaction(
            &file,
            crate::LockboxOpen::Password(&password),
            |_| {}
        )
        .unwrap());
        std::fs::remove_file(file).unwrap();
    }
}
