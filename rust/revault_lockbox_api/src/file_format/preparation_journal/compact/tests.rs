//! Internal representation tests; no public CLI writes this experimental journal.
use super::*;
use crate::storage::StorageBackend;
use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
fn context(encrypted: bool, signed: bool) -> Context {
    Context::new(
        LockboxId::from_bytes([81; 16]),
        FormatMode::new(LockboxFormatOptions {
            encryption: if encrypted {
                EncryptionMode::ChaCha20Poly1305
            } else {
                EncryptionMode::None
            },
            signing: if signed {
                SigningMode::Owner
            } else {
                SigningMode::None
            },
            compression: Compression::None,
            size_padding: SizePadding::Default,
        }),
        encrypted.then_some([82; 32].as_slice()),
    )
    .unwrap()
}
fn record(count: usize) -> Record {
    Record {
        sequence: 1,
        previous: [0; 32],
        base: [83; 32],
        active: true,
        cleanup_commit: [0; 32],
        cleanup_bytes: 0,
        reservations: (0..count)
            .map(|n| Reservation {
                namespace: FREE,
                base: 4 * FAILURE_REGION + n as u64 * 4096,
                start: 4 * FAILURE_REGION + n as u64 * 4096,
                len: 4096,
            })
            .collect(),
    }
}
fn install(storage: &mut StorageBackend, encoded: &Encoded, both: bool) {
    if let Some(bytes) = &encoded.overflow {
        for offset in [REGION_LEN as u64, REGION_LEN as u64 + FAILURE_REGION] {
            storage.write_at(offset, bytes).unwrap();
        }
    }
    storage.sync().unwrap();
    storage.write_at(STUB_OFFSET, &encoded.stub).unwrap();
    if both {
        storage
            .write_at(FAILURE_REGION + STUB_OFFSET, &encoded.stub)
            .unwrap();
    }
    storage.sync().unwrap();
}
#[test]
fn compact_journal_preserves_full_reservation_capacity_and_separated_overflow() {
    for encrypted in [false, true] {
        for signed in [false, true] {
            let context = context(encrypted, signed);
            for count in [0, 1, 155, 156, MAX_RESERVATIONS] {
                let expected = record(count);
                let locations = (count > 155)
                    .then_some((REGION_LEN as u64, REGION_LEN as u64 + FAILURE_REGION));
                let encoded = encode(&context, &expected, locations).unwrap();
                assert_eq!(encoded.stub.len(), STUB_BYTES);
                assert_eq!(
                    encoded.overflow.as_ref().map(Vec::len),
                    locations.map(|_| SLOT_BYTES)
                );
                let mut storage = StorageBackend::memory(vec![0; 4 * FAILURE_REGION as usize]);
                install(&mut storage, &encoded, true);
                assert!(select(&context, &storage).unwrap() == expected);
                for region in 0..4 {
                    let mut damaged = storage.clone();
                    damaged
                        .write_at(region * FAILURE_REGION, &vec![0; FAILURE_REGION as usize])
                        .unwrap();
                    assert!(select(&context, &damaged).unwrap() == expected);
                }
            }
            assert!(encode(&context, &record(MAX_RESERVATIONS + 1), None).is_err());
            assert!(encode(&context, &record(MAX_RESERVATIONS), None).is_err());
            assert!(encode(
                &context,
                &record(0),
                Some((REGION_LEN as u64, 3 * FAILURE_REGION))
            )
            .is_err());
        }
    }
}
#[test]
fn missing_new_overflow_cannot_restore_an_older_reservation_set() {
    let context = context(true, true);
    let old = record(1);
    let encoded_old = encode(&context, &old, None).unwrap();
    let mut storage = StorageBackend::memory(vec![0; 4 * FAILURE_REGION as usize]);
    install(&mut storage, &encoded_old, true);
    let mut new = record(MAX_RESERVATIONS);
    new.sequence = 2;
    new.previous = strong_checksum(&encoded_old.stub);
    let encoded = encode(
        &context,
        &new,
        Some((REGION_LEN as u64, 3 * FAILURE_REGION)),
    )
    .unwrap();
    install(&mut storage, &encoded, false);
    assert!(select(&context, &storage).unwrap() == new);
    storage
        .write_at(REGION_LEN as u64, &vec![0; 2 * FAILURE_REGION as usize])
        .unwrap();
    assert!(select(&context, &storage).is_err());
}
#[test]
fn compact_journal_binds_context_and_refuses_invalid_overflow_placement() {
    let context = context(true, false);
    let record = record(MAX_RESERVATIONS);
    for locations in [
        (8192, 3 * FAILURE_REGION),
        (REGION_LEN as u64 + 1, 3 * FAILURE_REGION),
        (REGION_LEN as u64, REGION_LEN as u64),
        (u64::MAX, 3 * FAILURE_REGION),
    ] {
        assert!(encode(&context, &record, Some(locations)).is_err());
    }
    let encoded = encode(
        &context,
        &record,
        Some((REGION_LEN as u64, 3 * FAILURE_REGION)),
    )
    .unwrap();
    let wrong = Context::new(context.archive, context.mode, Some(&[84; 32])).unwrap();
    assert!(decode(&wrong, &encoded.stub).is_err());
    assert!(context.decode(&encoded.stub).is_err());
    assert!(decode(&context, encoded.overflow.as_ref().unwrap()).is_err());
    for position in [0, 10, 12, 28, 32, 44, HEADER, STUB_CHECKSUM - 1] {
        let mut corrupt = encoded.stub.clone();
        corrupt[position] ^= 1;
        let checksum = strong_checksum(&corrupt[..STUB_CHECKSUM]);
        corrupt[STUB_CHECKSUM..].copy_from_slice(&checksum);
        assert!(decode(&context, &corrupt).is_err());
    }
}

#[test]
fn every_torn_stub_prefix_retains_the_old_or_new_complete_record() {
    for encrypted in [false, true] {
        let context = context(encrypted, !encrypted);
        let old = record(1);
        let old_encoded = encode(&context, &old, None).unwrap();
        let mut new = record(MAX_RESERVATIONS);
        new.sequence = 2;
        new.previous = strong_checksum(&old_encoded.stub);
        let new_encoded = encode(
            &context,
            &new,
            Some((REGION_LEN as u64, 3 * FAILURE_REGION)),
        )
        .unwrap();
        let mut prepared = StorageBackend::memory(vec![0; 4 * FAILURE_REGION as usize]);
        install(&mut prepared, &old_encoded, true);
        for offset in [REGION_LEN as u64, 3 * FAILURE_REGION] {
            prepared
                .write_at(offset, new_encoded.overflow.as_ref().unwrap())
                .unwrap();
        }
        prepared.sync().unwrap();
        let initial = prepared.read_all().unwrap();
        for prefix in 0..=STUB_BYTES {
            let mut interrupted = StorageBackend::memory(initial.clone());
            interrupted
                .write_at(STUB_OFFSET, &new_encoded.stub[..prefix])
                .unwrap();
            let selected = select(&context, &interrupted).unwrap();
            assert!(selected == old || selected == new, "prefix {prefix}");
        }
    }
}
#[test]
fn stub_record_identity_and_malformed_inline_counts_are_rejected() {
    let context = context(false, true);
    let mut expected = record(MAX_RESERVATIONS);
    let mut encoded = encode(
        &context,
        &expected,
        Some((REGION_LEN as u64, 3 * FAILURE_REGION)),
    )
    .unwrap();
    expected.base = [91; 32];
    let substituted = context.encode(&expected).unwrap();
    // Even a checksummed plaintext stub cannot disguise a different record base
    // behind a substituted, independently valid overflow digest.
    let digest = strong_checksum(&substituted);
    encoded.stub[HEADER + 97..HEADER + 129].copy_from_slice(&digest);
    let checksum = strong_checksum(&encoded.stub[..STUB_CHECKSUM]);
    encoded.stub[STUB_CHECKSUM..].copy_from_slice(&checksum);
    encoded.overflow = Some(substituted);
    let mut storage = StorageBackend::memory(vec![0; 4 * FAILURE_REGION as usize]);
    install(&mut storage, &encoded, true);
    assert!(select(&context, &storage).is_err());
    let inline = encode(&context, &record(0), None).unwrap();
    for count in [MAX_RESERVATIONS as u32, u32::MAX] {
        let mut bytes = inline.stub.clone();
        bytes[HEADER + 114..HEADER + 118].copy_from_slice(&count.to_le_bytes());
        let checksum = strong_checksum(&bytes[..STUB_CHECKSUM]);
        bytes[STUB_CHECKSUM..].copy_from_slice(&checksum);
        assert!(decode(&context, &bytes).is_err());
    }
    let mut bytes = inline.stub;
    bytes[STUB_CHECKSUM - 1] = 1;
    let checksum = strong_checksum(&bytes[..STUB_CHECKSUM]);
    bytes[STUB_CHECKSUM..].copy_from_slice(&checksum);
    assert!(decode(&context, &bytes).is_err());
}
#[derive(Clone, Debug)]
struct ReadFailure {
    storage: StorageBackend,
    at: u64,
}
impl Storage for ReadFailure {
    fn len(&self) -> Result<u64> {
        self.storage.len()
    }
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        assert!(len <= SLOT_BYTES);
        if offset == self.at {
            return Err(Error::InvalidInput(
                "injected compact-journal read failure".into(),
            ));
        }
        self.storage.read_at(offset, len)
    }
    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        out.copy_from_slice(&self.read_at(offset, out.len())?);
        Ok(())
    }
    fn append(&mut self, _: &[u8]) -> Result<u64> {
        panic!("selector wrote storage")
    }
    fn write_at(&mut self, _: u64, _: &[u8]) -> Result<()> {
        panic!("selector wrote storage")
    }
    fn truncate(&mut self, _: u64) -> Result<()> {
        panic!("selector truncated storage")
    }
    fn sync(&self) -> Result<()> {
        panic!("selector synced storage")
    }
}
#[test]
fn compact_journal_read_errors_are_not_treated_as_missing_copies() {
    let context = context(true, false);
    let encoded = encode(
        &context,
        &record(MAX_RESERVATIONS),
        Some((REGION_LEN as u64, 3 * FAILURE_REGION)),
    )
    .unwrap();
    let mut storage = StorageBackend::memory(vec![0; 4 * FAILURE_REGION as usize]);
    install(&mut storage, &encoded, true);
    for at in [STUB_OFFSET, STUB_OFFSET + FAILURE_REGION, REGION_LEN as u64] {
        let guarded = ReadFailure {
            storage: storage.clone(),
            at,
        };
        assert!(
            matches!(select(&context, &guarded), Err(Error::InvalidInput(ref reason)) if reason == "injected compact-journal read failure")
        );
    }
}
