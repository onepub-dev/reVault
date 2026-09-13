//! Tail reclamation never infers ownership from zero bytes.
//!
//! A sealed free index must cover the entire suffix and all live allocations
//! must end before it. Both authenticated header slots then record the target
//! `sealed_len` and the original index extent `trim_origin_len`, with a sync
//! after each publication. Only then may storage be truncated and synced.
//! Recovery repeats both publications before truncating, including when only
//! the first intent survived. A reader rejects an unfinished truncation.
//!
//! The original extent remains authenticated until the next ordinary commit
//! writes a new free index. Loading the old index validates its complete suffix
//! before clipping it to the new EOF. Thus a crash needs neither an in-place
//! index edit nor a reference to metadata in the region being removed.
use super::Lockbox;
use crate::free_slot::FreeSlot;
use crate::storage::Storage;
use crate::{
    Error, Result, TransactionRecoveryControl, TransactionRecoveryPhase,
    TransactionRecoveryProgress,
};

impl<State> Lockbox<State> {
    pub(crate) fn clip_trimmed_free_slots(&self, slots: Vec<FreeSlot>) -> Result<Vec<FreeSlot>> {
        if self.trim_origin_len == 0 {
            return Ok(slots);
        }
        if self.cleanup_sequence < self.published_sequence {
            return Err(Error::CorruptHeader);
        }
        let cut = self.transaction_start_len;
        let mut previous_end = crate::constants::HEADER_LEN as u64;
        let mut suffix_end = cut;
        let mut clipped = Vec::new();
        for slot in slots {
            let end = slot
                .offset
                .checked_add(slot.len)
                .ok_or(Error::CorruptRecord)?;
            if slot.len == 0 || slot.offset < previous_end || end > self.trim_origin_len {
                return Err(Error::CorruptRecord);
            }
            previous_end = end;
            if end > cut {
                if slot.offset.max(cut) != suffix_end {
                    return Err(Error::CorruptRecord);
                }
                suffix_end = end;
            }
            if slot.offset < cut {
                clipped.push(FreeSlot {
                    offset: slot.offset,
                    len: end.min(cut) - slot.offset,
                });
            }
        }
        if suffix_end != self.trim_origin_len {
            return Err(Error::CorruptRecord);
        }
        Ok(clipped)
    }
}

impl Lockbox<crate::Writable> {
    pub(crate) fn trim_free_tail(&mut self) -> Result<u64> {
        self.require_clean_transaction()?;
        if self.preparing {
            return Err(Error::InvalidOperation(
                "cannot trim an active transaction".into(),
            ));
        }
        let original_len = self.storage.len()?;
        let slots = self.free_space.slots_by_offset();
        let Some(tail) = slots
            .last()
            .filter(|s| s.offset.checked_add(s.len) == Some(original_len))
        else {
            return Ok(0);
        };
        // Includes the free index, manifests, key directories, and retained
        // commit history. No authoritative metadata can be removed.
        self.inspector().verify_storage()?;
        let cut = tail.offset;
        self.trim_origin_len = self.trim_origin_len.max(original_len);
        self.transaction_start_len = cut;
        self.free_space
            .replace_slots(slots[..slots.len() - 1].to_vec());
        self.finish_tail_truncation(|_| TransactionRecoveryControl::Continue)?;
        Ok(original_len - cut)
    }

    pub(crate) fn finish_tail_truncation(
        &mut self,
        mut progress: impl FnMut(TransactionRecoveryProgress) -> TransactionRecoveryControl,
    ) -> Result<bool> {
        let len = self.storage.len()?;
        if self.trim_origin_len <= self.transaction_start_len
            || len < self.transaction_start_len
            || len > self.trim_origin_len
        {
            return Err(Error::CorruptHeader);
        }
        self.validate_base_reservations()?;
        // Validate live ownership even when the clipped free list is empty.
        let inventory = self.storage_inventory(true)?;
        if inventory.ranges.iter().any(|(&offset, &size)| {
            offset
                .checked_add(size)
                .is_none_or(|end| end > self.transaction_start_len)
        }) {
            return Err(Error::CorruptRecord);
        }
        self.publish_transaction_header(self.sequence)?;
        self.publish_transaction_header(self.sequence)?;
        let update = TransactionRecoveryProgress {
            phase: TransactionRecoveryPhase::Truncate,
            completed_ranges: 0,
            total_ranges: 1,
            completed_pages: 0,
            total_pages: 1,
            completed_bytes: 0,
            total_bytes: self.trim_origin_len - self.transaction_start_len,
        };
        if progress(update) == TransactionRecoveryControl::Cancel {
            return Ok(false);
        }
        self.poisoned = Some("tail truncation was interrupted; reopen to finish recovery".into());
        self.storage.truncate(self.transaction_start_len)?;
        self.storage.sync()?;
        self.page_manager.borrow_mut().clear();
        *self.compression_frame_cache.borrow_mut() = super::CompressionFrameCache::default();
        self.poisoned = None;
        progress(TransactionRecoveryProgress {
            completed_ranges: 1,
            completed_pages: 1,
            completed_bytes: update.total_bytes,
            ..update
        });
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::{PageObject, PageObjectKind};
    use crate::storage::StorageBackend;
    use crate::{LockboxOptions, LockboxPath, SecretVec};
    fn path(value: &str) -> LockboxPath {
        LockboxPath::new(value).unwrap()
    }

    fn open(bytes: Vec<u8>) -> Lockbox {
        Lockbox::open_storage_with_secret_key_mode(
            StorageBackend::memory(bytes),
            SecretVec::try_from_slice(b"tail test").unwrap(),
            LockboxOptions::default(),
            true,
        )
        .unwrap()
    }

    fn with_free_suffix() -> (Lockbox, u64) {
        let mut lb = Lockbox::create("tail test");
        lb.add_file(&path("/keep"), b"retained contents", false)
            .unwrap();
        lb.add_file(&path("/remove"), b"discarded contents", false)
            .unwrap();
        lb.commit().unwrap();
        lb.delete(&path("/remove")).unwrap();
        lb.commit().unwrap();
        let cut = lb.storage.len().unwrap();
        // Unit-only fixture: the current writer appends live commit records at
        // EOF, so the public CLI cannot create a committed free suffix. Model a
        // valid placement from an allocator that reuses interior control pages.
        // Update the existing authenticated free-index leaf; no live root moves.
        let index = lb.free_index_offset;
        let mut slots = lb.free_space.slots_by_offset();
        slots.push(FreeSlot {
            offset: cut,
            len: 1024 * 1024,
        });
        let page = lb.read_page(index).unwrap();
        assert_eq!(page.objects[0].kind, PageObjectKind::FreeIndexLeaf);
        let objects = [PageObject::new(
            PageObjectKind::FreeIndexLeaf,
            page.objects[0].id,
            crate::free_index::encode_free_index_leaf(&slots),
        )];
        let encoded = lb
            .key
            .with_bytes(|key| {
                crate::page::encode_page_with_format(
                    lb.page_len_at(index).unwrap() as usize,
                    lb.lockbox_id,
                    index,
                    page.sequence,
                    key,
                    &objects,
                    lb.format_mode,
                )
            })
            .unwrap()
            .unwrap();
        lb.storage.write_at(index, &encoded).unwrap();
        lb.storage.append(&vec![0; 1024 * 1024]).unwrap();
        lb.free_space.replace_slots(slots);
        lb.page_manager.borrow_mut().clear();
        lb.transaction_start_len = lb.storage.len().unwrap();
        lb.publish_transaction_header(lb.sequence).unwrap();
        lb.publish_transaction_header(lb.sequence).unwrap();
        lb.inspector().verify_storage().unwrap();
        (lb, cut)
    }

    #[test]
    fn free_suffix_is_truncated_and_next_commit_replaces_the_old_index() {
        let (mut lb, cut) = with_free_suffix();
        assert_eq!(lb.trim_free_tail().unwrap(), 1024 * 1024);
        assert_eq!(lb.storage.len().unwrap(), cut);
        let mut reopened = open(lb.to_bytes());
        reopened.inspector().verify_storage().unwrap();
        assert_eq!(reopened.trim_free_tail().unwrap(), 0);
        // Retain the established signer for the next actual update.
        reopened.owner_signing_key = lb.owner_signing_key.take();
        reopened
            .add_file(&path("/next"), b"new contents", false)
            .unwrap();
        reopened.commit().unwrap();
        assert_eq!(reopened.trim_origin_len, 0);
        let reopened = open(reopened.to_bytes());
        assert_eq!(
            reopened.get_file(&path("/keep")).unwrap(),
            b"retained contents"
        );
        assert_eq!(reopened.get_file(&path("/next")).unwrap(), b"new contents");
        reopened.inspector().verify_storage().unwrap();
    }

    #[test]
    fn tail_truncation_recovers_every_storage_failure() {
        let (base, cut) = with_free_suffix();
        let bytes = base.to_bytes();
        let mut successful = open(bytes.clone());
        successful.storage.reset_memory_operation_count();
        successful.trim_free_tail().unwrap();
        let count = successful.storage.memory_operation_count();
        for failure in 0..count {
            let mut attempt = open(bytes.clone());
            attempt
                .storage
                .fail_memory_operation_after_successes(failure);
            // Diagnostic reads can consume the injected failure; regardless of
            // the return value, reopening must converge to the intact archive.
            let _ = attempt.trim_free_tail();
            let mut recovered = open(attempt.to_bytes());
            recovered.complete_pending_transaction_cleanup().unwrap();
            assert_eq!(recovered.storage.len().unwrap(), cut, "failure {failure}");
            assert_eq!(
                recovered.get_file(&path("/keep")).unwrap(),
                b"retained contents"
            );
            recovered.inspector().verify_storage().unwrap();
        }
    }

    #[test]
    fn cancelled_trim_and_a_damaged_header_slot_resume_without_data_loss() {
        let (mut lb, cut) = with_free_suffix();
        lb.trim_origin_len = lb.storage.len().unwrap();
        lb.transaction_start_len = cut;
        let slots = lb.free_space.slots_by_offset();
        lb.free_space
            .replace_slots(slots[..slots.len() - 1].to_vec());
        assert!(!lb
            .finish_tail_truncation(|_| TransactionRecoveryControl::Cancel)
            .unwrap());
        let bytes = lb.to_bytes();
        assert!(matches!(
            Lockbox::open_bytes_with_key(bytes.clone(), "tail test"),
            Err(Error::RecoveryRequired { .. })
        ));
        for slot in 0..2 {
            let mut torn = bytes.clone();
            torn[slot * crate::file_format::header_v2::SLOT_LEN + 24] ^= 1;
            let mut recovered = open(torn);
            assert_eq!(
                recovered.transaction_recovery_status().unwrap().phase,
                TransactionRecoveryPhase::Truncate
            );
            recovered.complete_pending_transaction_cleanup().unwrap();
            assert_eq!(recovered.storage.len().unwrap(), cut);
            recovered.inspector().verify_storage().unwrap();
        }
    }

    #[test]
    fn zero_live_content_and_unaccounted_zero_tail_are_not_free() {
        let mut lb = Lockbox::create("tail test");
        lb.add_file(&path("/zero"), &vec![0; 128 * 1024], false)
            .unwrap();
        lb.commit().unwrap();
        let len = lb.storage.len().unwrap();
        assert_eq!(lb.trim_free_tail().unwrap(), 0);
        assert_eq!(lb.storage.len().unwrap(), len);
        lb.storage.append(&vec![0; 4096]).unwrap();
        assert!(Lockbox::open_bytes_with_key(lb.to_bytes(), "tail test").is_err());
    }

    #[test]
    fn truncation_process_child() {
        let Ok(path) = std::env::var("REV313_TRUNCATE_PATH") else {
            return;
        };
        Lockbox::recover_transaction_controlled(
            std::path::Path::new(&path),
            crate::LockboxOpen::ContentKey(SecretVec::try_from_slice(b"tail test").unwrap()),
            |_| {
                use std::io::Write;
                println!("REV313_TRUNCATE_READY");
                std::io::stdout().flush().unwrap();
                loop {
                    std::thread::park();
                }
            },
        )
        .unwrap();
    }

    #[test]
    fn file_tail_truncation_resumes_after_process_death() {
        use std::io::BufRead;
        let (mut lb, cut) = with_free_suffix();
        lb.trim_origin_len = lb.storage.len().unwrap();
        lb.transaction_start_len = cut;
        let slots = lb.free_space.slots_by_offset();
        lb.free_space
            .replace_slots(slots[..slots.len() - 1].to_vec());
        lb.finish_tail_truncation(|_| TransactionRecoveryControl::Cancel)
            .unwrap();
        let path =
            std::env::temp_dir().join(format!("revault-tail-death-{}.lbox", std::process::id()));
        // A committed free suffix cannot currently be constructed with the CLI;
        // persist the authenticated unit fixture to exercise real file recovery.
        std::fs::write(&path, lb.to_bytes()).unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "lockbox::tail_reclamation::tests::truncation_process_child",
                "--nocapture",
            ])
            .env("REV313_TRUNCATE_PATH", &path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (send, receive) = std::sync::mpsc::channel();
        let reader = std::thread::spawn(move || {
            let found = std::io::BufReader::new(stdout)
                .lines()
                .any(|line| line.is_ok_and(|line| line.contains("REV313_TRUNCATE_READY")));
            let _ = send.send(found);
        });
        let ready = receive.recv_timeout(std::time::Duration::from_secs(30));
        let _ = child.kill();
        child.wait().unwrap();
        reader.join().unwrap();
        assert!(matches!(ready, Ok(true)));
        assert!(Lockbox::recover_transaction(
            &path,
            crate::LockboxOpen::ContentKey(SecretVec::try_from_slice(b"tail test").unwrap()),
            |_| {}
        )
        .unwrap());
        assert_eq!(std::fs::metadata(&path).unwrap().len(), cut);
        let opened = Lockbox::open_path(&path, "tail test").unwrap();
        assert_eq!(
            opened.get_file(&path_from("/keep")).unwrap(),
            b"retained contents"
        );
        opened.inspector().verify_storage().unwrap();
        drop(opened);
        assert!(!Lockbox::recover_transaction(
            &path,
            crate::LockboxOpen::ContentKey(SecretVec::try_from_slice(b"tail test").unwrap()),
            |_| {}
        )
        .unwrap());
        std::fs::remove_file(path).unwrap();
    }

    fn path_from(value: &str) -> LockboxPath {
        LockboxPath::new(value).unwrap()
    }
}
