//! Local, bounded redo journal for completed-exchange accounting.
use super::{Receipt, Stored};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Anonymous durable statistics for one exchange state directory.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExchangeStats {
    /// Version of this statistics document.
    pub schema_version: u32,
    /// Accounting scope; never a cluster-wide or unique-user count.
    pub scope: String,
    /// Time accounting was initialized, in Unix milliseconds.
    pub counting_since_unix_ms: u64,
    /// Locally committed reciprocal completions since initialization.
    pub completed_exchanges: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pending {
    before: ExchangeStats,
    after: ExchangeStats,
    receipt: Receipt,
}

pub(super) struct Accounting {
    directory: PathBuf,
    records: PathBuf,
}

fn invalid() -> io::Error {
    io::Error::other("invalid exchange accounting state")
}

#[cfg(test)]
thread_local! {
    static FAIL_POINT: std::cell::Cell<Option<u8>> = const { std::cell::Cell::new(None) };
}

fn checkpoint(point: u8) -> io::Result<()> {
    #[cfg(test)]
    if FAIL_POINT.with(|value| {
        if value.get() == Some(point) {
            value.set(None);
            true
        } else {
            false
        }
    }) {
        return Err(io::Error::other("injected accounting failure"));
    }
    let _ = point;
    Ok(())
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> io::Result<T> {
    if std::fs::metadata(path)?.len() > 16384 {
        return Err(invalid());
    }
    serde_json::from_slice(&std::fs::read(path)?).map_err(|_| invalid())
}

pub(super) fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().ok_or_else(invalid)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(bytes)?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|e| e.error)?;
    if path.file_name().is_some_and(|name| name == "pending") {
        checkpoint(8)?;
    }
    sync_directory(parent)
}

fn sync_directory(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

fn save<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    atomic_write(path, &serde_json::to_vec(value).map_err(|_| invalid())?)
}

/// The held handle protects against other processes and duplicate store instances.
pub(super) fn lock_directory(directory: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0);
    }
    let file = options.open(directory.join(".exchange-lock"))?;
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        // SAFETY: flock only operates on this live file descriptor; the handle
        // is retained by the store and closing it releases the advisory lock.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(io::Error::other("exchange directory is already open"));
        }
    }
    #[cfg(not(any(unix, windows)))]
    return Err(io::Error::other(
        "exclusive exchange storage is unsupported",
    ));
    Ok(file)
}

impl Accounting {
    pub(super) fn open(records: &Path, now: u64) -> io::Result<Self> {
        let directory = records.join("accounting");
        let marker = records.join(".exchange-accounting-initialized");
        if marker.exists() && !directory.is_dir() {
            return Err(invalid());
        }
        if !directory.try_exists()? {
            // Publish a complete initial state; a crash must not strand an
            // empty accounting directory that resembles damaged history.
            let initial = tempfile::tempdir_in(records)?;
            save(
                &initial.path().join("aggregate"),
                &ExchangeStats {
                    schema_version: 1,
                    scope: "local_state_directory".to_owned(),
                    counting_since_unix_ms: now,
                    completed_exchanges: 0,
                },
            )?;
            checkpoint(6)?;
            std::fs::rename(initial.path(), &directory)?;
            checkpoint(7)?;
            sync_directory(records)?;
        }
        let value = Self {
            directory,
            records: records.to_owned(),
        };
        value.recover()?;
        if !marker.exists() {
            atomic_write(&marker, b"1\n")?;
        }
        Ok(value)
    }

    fn aggregate(&self) -> io::Result<ExchangeStats> {
        let stats: ExchangeStats = read(&self.directory.join("aggregate"))?;
        if stats.schema_version != 1 || stats.scope != "local_state_directory" {
            return Err(invalid());
        }
        Ok(stats)
    }

    pub(super) fn stats(&self) -> io::Result<ExchangeStats> {
        self.recover()?;
        self.aggregate()
    }

    pub(super) fn recover(&self) -> io::Result<()> {
        let current = self.aggregate()?;
        let path = self.directory.join("pending");
        let pending: Pending = match read(&path) {
            Ok(value) => value,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        let mut expected = pending.before.clone();
        expected.completed_exchanges = expected
            .completed_exchanges
            .checked_add(1)
            .ok_or_else(invalid)?;
        let valid_token = revault_publish_protocol::exchange::valid_token;
        if pending.after != expected
            || (current != pending.before && current != pending.after)
            || pending.receipt.cancelled
            || !valid_token(&pending.receipt.id)
            || !valid_token(&pending.receipt.identity)
            || !valid_token(&pending.receipt.owner_hash)
            || !valid_token(&pending.receipt.offer_hash)
            || !pending
                .receipt
                .recipient_hash
                .as_deref()
                .is_some_and(valid_token)
            || !pending
                .receipt
                .acceptance_hash
                .as_deref()
                .is_some_and(valid_token)
        {
            return Err(invalid());
        }
        // A previous journal rename may be visible despite a failed directory
        // sync. Re-establish its durability before making either target durable;
        // otherwise a crash could retain the receipt but lose its accounting.
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?
            .sync_all()?;
        checkpoint(9)?;
        sync_directory(&self.directory)?;
        save(
            &self.records.join(format!("{}.json", pending.receipt.id)),
            &Stored::Receipt(pending.receipt),
        )?;
        checkpoint(2)?;
        save(&self.directory.join("aggregate"), &pending.after)?;
        checkpoint(3)?;
        std::fs::remove_file(path)?;
        checkpoint(5)?;
        sync_directory(&self.directory)?;
        checkpoint(4)
    }

    pub(super) fn complete(&self, receipt: Receipt) -> io::Result<()> {
        self.recover()?;
        let before = self.aggregate()?;
        let mut after = before.clone();
        after.completed_exchanges = after
            .completed_exchanges
            .checked_add(1)
            .ok_or_else(|| io::Error::other("exchange accounting counter exhausted"))?;
        checkpoint(0)?;
        save(
            &self.directory.join("pending"),
            &Pending {
                before,
                after: after.clone(),
                receipt,
            },
        )?;
        checkpoint(1)?;
        self.recover()?;
        crate::server_log::log_server_event(format!(
            "exchange accounting scope=local_state_directory counting_since_unix_ms={} completed_exchanges={}",
            after.counting_since_unix_ms, after.completed_exchanges
        ));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn receipt() -> Receipt {
        Receipt {
            id: "a".repeat(64),
            expiry: 2000,
            identity: "b".repeat(64),
            owner_hash: "c".repeat(64),
            recipient_hash: Some("d".repeat(64)),
            offer_hash: "e".repeat(64),
            acceptance_hash: Some("f".repeat(64)),
            cancelled: false,
        }
    }

    #[test]
    fn recovery_at_every_commit_boundary_is_idempotent() {
        for point in 0..=5 {
            let dir = tempfile::tempdir().unwrap();
            let accounting = Accounting::open(dir.path(), 1000).unwrap();
            FAIL_POINT.with(|value| value.set(Some(point)));
            assert!(accounting.complete(receipt()).is_err());
            drop(accounting);
            let accounting = Accounting::open(dir.path(), 3000).unwrap();
            let expected = u64::from(point > 0);
            for _ in 0..3 {
                assert_eq!(accounting.stats().unwrap().completed_exchanges, expected);
            }
            assert!(!dir.path().join("accounting/pending").exists());
            if point > 0 {
                let stored: Stored =
                    read(&dir.path().join(format!("{}.json", "a".repeat(64)))).unwrap();
                assert!(matches!(stored, Stored::Receipt(_)));
            }
        }
    }

    #[test]
    fn recovery_precedes_expiry_and_stats() {
        let dir = tempfile::tempdir().unwrap();
        let accounting = Accounting::open(dir.path(), 1000).unwrap();
        FAIL_POINT.with(|value| value.set(Some(1)));
        assert!(accounting.complete(receipt()).is_err());
        drop(accounting);
        let store = crate::exchange_store::ExchangeStore::open(
            dir.path(),
            crate::exchange_store::ExchangeLimits::default(),
            3000,
        )
        .unwrap();
        assert_eq!(store.stats().unwrap().completed_exchanges, 1);
        assert!(store.snapshot_ids().unwrap().is_empty());
        assert!(!dir.path().join(format!("{}.json", "a".repeat(64))).exists());
    }

    #[test]
    fn missing_or_corrupt_established_state_fails_closed() {
        for corrupt in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let accounting = Accounting::open(dir.path(), 1000).unwrap();
            let path = dir.path().join("accounting/aggregate");
            if corrupt {
                std::fs::write(path, b"not json").unwrap();
            } else {
                std::fs::remove_file(path).unwrap();
            }
            assert!(accounting.stats().is_err());
            assert!(Accounting::open(dir.path(), 2000).is_err());
        }
        let dir = tempfile::tempdir().unwrap();
        Accounting::open(dir.path(), 1000).unwrap();
        std::fs::remove_dir_all(dir.path().join("accounting")).unwrap();
        assert!(Accounting::open(dir.path(), 2000).is_err());
    }

    #[test]
    fn corrupt_pending_and_counter_overflow_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        let accounting = Accounting::open(dir.path(), 1000).unwrap();
        let mut stats = accounting.stats().unwrap();
        stats.completed_exchanges = u64::MAX;
        save(&dir.path().join("accounting/aggregate"), &stats).unwrap();
        assert!(accounting.complete(receipt()).is_err());
        assert_eq!(accounting.stats().unwrap(), stats);
        assert!(!dir.path().join("accounting/pending").exists());
        std::fs::write(dir.path().join("accounting/pending"), b"{}").unwrap();
        assert!(accounting.stats().is_err());
        assert!(Accounting::open(dir.path(), 2000).is_err());
    }

    #[test]
    fn preexisting_receipts_do_not_seed_the_baseline() {
        let dir = tempfile::tempdir().unwrap();
        let receipt = receipt();
        save(
            &dir.path().join(format!("{}.json", receipt.id)),
            &Stored::Receipt(receipt),
        )
        .unwrap();
        let store = crate::exchange_store::ExchangeStore::open(
            dir.path(),
            crate::exchange_store::ExchangeLimits::default(),
            1500,
        )
        .unwrap();
        assert_eq!(
            store.stats().unwrap(),
            ExchangeStats {
                schema_version: 1,
                scope: "local_state_directory".to_owned(),
                counting_since_unix_ms: 1500,
                completed_exchanges: 0,
            }
        );
    }

    #[test]
    fn interrupted_initialization_is_retryable_and_preserves_published_baseline() {
        for point in [6, 7] {
            let dir = tempfile::tempdir().unwrap();
            FAIL_POINT.with(|value| value.set(Some(point)));
            assert!(Accounting::open(dir.path(), 1000).is_err());
            assert_eq!(dir.path().join("accounting").exists(), point == 7);
            let accounting = Accounting::open(dir.path(), 2000).unwrap();
            let stats = accounting.stats().unwrap();
            assert_eq!(stats.completed_exchanges, 0);
            assert_eq!(
                stats.counting_since_unix_ms,
                if point == 7 { 1000 } else { 2000 }
            );
            assert!(dir.path().join(".exchange-accounting-initialized").exists());
        }
    }

    #[test]
    fn visible_pending_must_be_durable_before_recovery_applies_targets() {
        let dir = tempfile::tempdir().unwrap();
        let accounting = Accounting::open(dir.path(), 1000).unwrap();
        FAIL_POINT.with(|value| value.set(Some(8)));
        assert!(accounting.complete(receipt()).is_err());
        let pending = dir.path().join("accounting/pending");
        let record = dir.path().join(format!("{}.json", "a".repeat(64)));
        assert!(pending.exists());
        assert!(!record.exists());
        assert_eq!(accounting.aggregate().unwrap().completed_exchanges, 0);

        // Recovery must stop if establishing journal durability fails again.
        FAIL_POINT.with(|value| value.set(Some(9)));
        assert!(accounting.recover().is_err());
        assert!(pending.exists());
        assert!(!record.exists());
        assert_eq!(accounting.aggregate().unwrap().completed_exchanges, 0);

        accounting.recover().unwrap();
        assert!(record.exists());
        assert!(!pending.exists());
        for _ in 0..3 {
            assert_eq!(accounting.stats().unwrap().completed_exchanges, 1);
        }
        let reopened = Accounting::open(dir.path(), 2000).unwrap();
        assert_eq!(reopened.stats().unwrap().completed_exchanges, 1);
    }

    #[test]
    fn incompatible_or_malformed_pending_state_is_not_applied() {
        for corruption in 0..9 {
            let dir = tempfile::tempdir().unwrap();
            let accounting = Accounting::open(dir.path(), 1000).unwrap();
            let before = accounting.stats().unwrap();
            let mut after = before.clone();
            after.completed_exchanges = 1;
            let mut pending = Pending {
                before: before.clone(),
                after,
                receipt: receipt(),
            };
            match corruption {
                0 => {
                    pending.before.completed_exchanges = 10;
                    pending.after.completed_exchanges = 11;
                }
                1 => pending.receipt.id = "invalid".to_owned(),
                2 => pending.receipt.identity = "invalid".to_owned(),
                3 => pending.receipt.owner_hash = "invalid".to_owned(),
                4 => pending.receipt.offer_hash = "invalid".to_owned(),
                5 => pending.receipt.recipient_hash = Some("invalid".to_owned()),
                6 => pending.receipt.acceptance_hash = Some("invalid".to_owned()),
                7 => pending.receipt.recipient_hash = None,
                8 => pending.receipt.acceptance_hash = None,
                _ => unreachable!(),
            }
            save(&dir.path().join("accounting/pending"), &pending).unwrap();
            assert!(accounting.recover().is_err(), "corruption {corruption}");
            assert_eq!(accounting.aggregate().unwrap(), before);
            assert!(!dir.path().join(format!("{}.json", "a".repeat(64))).exists());
            assert!(dir.path().join("accounting/pending").exists());
            assert!(Accounting::open(dir.path(), 2000).is_err());
        }
    }
}
