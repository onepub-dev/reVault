#![cfg(feature = "external-source")]
//! API tests: external sources have no CLI equivalent. Fixtures use public APIs;
//! byte corruption and source failure injection exercise conditions no CLI can create.
use revault_lockbox_api::external_source::*;
use revault_lockbox_api::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

fn path(value: &str) -> LockboxPath {
    LockboxPath::new(value).unwrap()
}
fn fixture(compressed: bool, signed: bool, encrypted: bool) -> (Vec<u8>, Vec<u8>) {
    let signer = OwnerSigningKeyPair::generate().unwrap();
    let encryption = if encrypted {
        Encryption::Encrypted(LockboxProtection::ContentKey(
            SecretVec::try_from_slice(&[7; 32]).unwrap(),
        ))
    } else {
        Encryption::None
    };
    let mut options = LockboxCreateOptions::new(
        encryption,
        if signed {
            Signing::Owner(&signer)
        } else {
            Signing::None
        },
    );
    if !compressed {
        options.compression = Compression::None;
    }
    let mut archive = Lockbox::create_in_memory_with_options(options).unwrap();
    let payload: Vec<u8> = (0..300_137).map(|i| ((i * 73) % 251) as u8).collect();
    archive
        .add_file(&path("/data.bin"), &payload, false)
        .unwrap();
    archive.add_file(&path("/empty"), &[], false).unwrap();
    archive
        .add_symlink(&path("/link"), &path("/data.bin"), false)
        .unwrap();
    archive
        .set_variable(
            &VariableName::new("TEST_VALUE").unwrap(),
            "retained variable",
        )
        .unwrap();
    archive
        .define_form(
            "note",
            "Note",
            vec![
                FormFieldDefinition {
                    id: "body".into(),
                    label: "Body".into(),
                    kind: FormFieldKind::Text,
                    required: false,
                },
                FormFieldDefinition {
                    id: "secret".into(),
                    label: "Secret".into(),
                    kind: FormFieldKind::Secret,
                    required: false,
                },
            ],
        )
        .unwrap();
    archive
        .create_form_record(&path("/form"), "note", "record")
        .unwrap();
    archive
        .set_form_field_normal(&path("/form"), "body", "form contents")
        .unwrap();
    archive
        .set_form_field_secret(
            &path("/form"),
            "secret",
            &SecretString::try_from_slice(b"fixture form secret").unwrap(),
        )
        .unwrap();
    archive
        .set_secret_variable(
            &VariableName::new("SECRET_VALUE").unwrap(),
            &SecretString::try_from_slice(b"fixture variable secret").unwrap(),
        )
        .unwrap();
    archive.commit().unwrap();
    (archive.try_to_bytes().unwrap(), payload)
}
#[derive(Clone)]
struct Bytes(Arc<Vec<u8>>);
impl ReadAtSource for Bytes {
    fn len(&self) -> u64 {
        self.0.len() as u64
    }
    fn read_at(&self, offset: u64, output: &mut [u8]) -> std::result::Result<usize, SourceError> {
        output.copy_from_slice(&self.0[offset as usize..offset as usize + output.len()]);
        Ok(output.len())
    }
}
fn reader(source: Arc<dyn ReadAtSource>, encrypted: bool) -> ExternalReader {
    if encrypted {
        ExternalReader::with_content_key(
            source,
            SecretVec::try_from_slice(&[7; 32]).unwrap(),
            ExternalReaderOptions::default(),
        )
        .unwrap()
    } else {
        ExternalReader::new(source, ExternalReaderOptions::default()).unwrap()
    }
}
fn retry<T>(
    reader: &mut ExternalReader,
    sparse: &SparseSource,
    bytes: &[u8],
    mut operation: impl FnMut(&Lockbox<ReadOnly>) -> revault_lockbox_api::Result<T>,
) -> (T, usize) {
    for count in 0..4096 {
        match reader.try_read(&mut operation) {
            Ok(value) => return (value, count),
            Err(ExternalReadError::Source(SourceError::MissingRange { offset, length })) => sparse
                .supply(
                    "v1",
                    offset,
                    bytes[offset as usize..offset as usize + length].to_vec(),
                )
                .unwrap(),
            Err(error) => panic!("unexpected external failure: {error}"),
        }
    }
    panic!("retry loop did not converge")
}
#[test]
fn incremental_modes_preserve_files_and_other_record_types() {
    for compressed in [false, true] {
        for signed in [false, true] {
            for encrypted in [false, true] {
                let (bytes, payload) = fixture(compressed, signed, encrypted);
                let sparse = Arc::new(
                    SparseSource::new(bytes.len() as u64, "v1".into(), 4096, bytes.len() + 4096)
                        .unwrap(),
                );
                let mut reader = reader(sparse.clone(), encrypted);
                let ((actual, variable, form, link, empty), misses) =
                    retry(&mut reader, &sparse, &bytes, |archive| {
                        Ok((
                            archive.get_file(&path("/data.bin"))?,
                            archive.get_variable(&VariableName::new("TEST_VALUE")?)?,
                            archive.get_form_record(&path("/form"))?,
                            archive.get_symlink_target(&path("/link"))?,
                            archive.get_file(&path("/empty"))?,
                        ))
                    });
                assert_eq!(actual, payload);
                assert_eq!(variable.as_deref(), Some("retained variable"));
                let form = form.unwrap();
                assert_eq!(form.name, "record");
                assert_eq!(
                    form.values
                        .iter()
                        .find(|v| v.field_id == "body")
                        .unwrap()
                        .value,
                    FormValue::normal("form contents")
                );
                match &form
                    .values
                    .iter()
                    .find(|v| v.field_id == "secret")
                    .unwrap()
                    .value
                {
                    FormValue::Secret(value) => {
                        assert!(value.with_str(|s| s == "fixture form secret").unwrap())
                    }
                    _ => panic!("secret field lost sensitivity"),
                }
                assert_eq!(
                    retry(&mut reader, &sparse, &bytes, |archive| archive
                        .with_secret_variable(
                            &VariableName::new("SECRET_VALUE")?,
                            |value| value.with_str(|s| s == "fixture variable secret").unwrap()
                        ))
                    .0,
                    Some(true)
                );
                assert_eq!(link, path("/data.bin"));
                assert!(empty.is_empty());
                assert!(misses > 0 && misses <= bytes.len().div_ceil(4096));
                let (_, repeats) = retry(&mut reader, &sparse, &bytes, |archive| {
                    archive.get_file(&path("/data.bin"))
                });
                assert_eq!(repeats, 0);
                assert!(matches!(
                    reader.try_read(|archive| archive.get_file(&path("/absent"))),
                    Err(ExternalReadError::Archive(Error::NotFound(_)))
                ));
            }
        }
    }
}
#[test]
fn swallowed_errors_cannot_be_returned_as_success() {
    let (bytes, payload) = fixture(false, false, false);
    let sparse = Arc::new(
        SparseSource::new(bytes.len() as u64, "v1".into(), 4096, bytes.len() + 4096).unwrap(),
    );
    let mut reader = reader(sparse.clone(), false);
    retry(&mut reader, &sparse, &bytes, |_| Ok(()));
    assert!(sparse.cached_bytes().unwrap() < bytes.len());
    // Deliberately swallow every file-read error, as a fallback might do.
    assert!(matches!(
        reader.try_read(|archive| Ok(archive.get_file(&path("/data.bin")).is_ok())),
        Err(ExternalReadError::Source(SourceError::MissingRange { .. }))
    ));
    assert_eq!(
        retry(&mut reader, &sparse, &bytes, |archive| archive
            .get_file(&path("/data.bin")))
        .0,
        payload
    );
}
struct FaultSource {
    bytes: Bytes,
    call: AtomicUsize,
    fail_at: usize,
    trace: Mutex<Vec<(u64, usize)>>,
}
impl ReadAtSource for FaultSource {
    fn len(&self) -> u64 {
        self.bytes.len()
    }
    fn read_at(&self, offset: u64, output: &mut [u8]) -> std::result::Result<usize, SourceError> {
        self.trace.lock().unwrap().push((offset, output.len()));
        if self.call.fetch_add(1, Ordering::SeqCst) == self.fail_at {
            return Err(SourceError::MissingRange {
                offset,
                length: output.len(),
            });
        }
        self.bytes.read_at(offset, output)
    }
}
#[test]
fn every_open_and_lazy_record_read_failure_is_retryable_and_cannot_invoke_partial_open() {
    let (bytes, payload) = fixture(true, true, true);
    let bytes = Bytes(Arc::new(bytes));
    let baseline = Arc::new(FaultSource {
        bytes: bytes.clone(),
        call: AtomicUsize::new(0),
        fail_at: usize::MAX,
        trace: Mutex::new(vec![]),
    });
    let operation = |archive: &Lockbox<ReadOnly>| {
        archive.get_variable(&VariableName::new("TEST_VALUE")?)?;
        archive.get_form_record(&path("/form"))?;
        archive.get_file(&path("/data.bin"))
    };
    assert_eq!(
        reader(baseline.clone(), true).try_read(operation).unwrap(),
        payload
    );
    let calls = baseline.call.load(Ordering::SeqCst);
    assert!(calls > 4);
    for fail_at in 0..calls {
        let source = Arc::new(FaultSource {
            bytes: bytes.clone(),
            call: AtomicUsize::new(0),
            fail_at,
            trace: Mutex::new(vec![]),
        });
        let mut reader = reader(source, true);
        assert!(
            matches!(
                reader.try_read(operation),
                Err(ExternalReadError::Source(SourceError::MissingRange { .. }))
            ),
            "read {fail_at}"
        );
        assert_eq!(
            reader.try_read(operation).unwrap(),
            payload,
            "read {fail_at}"
        );
    }
    // Failure during open must not call a closure, even if open fallback succeeds.
    let open_calls = Arc::new(FaultSource {
        bytes: bytes.clone(),
        call: AtomicUsize::new(0),
        fail_at: usize::MAX,
        trace: Mutex::new(vec![]),
    });
    reader(open_calls.clone(), true)
        .try_read(|_| Ok(()))
        .unwrap();
    for fail_at in 0..open_calls.call.load(Ordering::SeqCst) {
        let source = Arc::new(FaultSource {
            bytes: bytes.clone(),
            call: AtomicUsize::new(0),
            fail_at,
            trace: Mutex::new(vec![]),
        });
        assert!(reader(source, true)
            .try_read::<()>(|_| panic!("closure called after incomplete open {fail_at}"))
            .is_err());
    }
}
#[test]
fn cache_bounds_eviction_overlap_identity_and_cancellation() {
    let cache = SparseSource::new(1600, "v1".into(), 512, 1024).unwrap();
    cache.supply("v1", 0, vec![1; 512]).unwrap();
    cache.supply("v1", 512, vec![2; 512]).unwrap();
    let mut out = [0; 8];
    cache.read_at(0, &mut out).unwrap(); // first block becomes most recently used
    cache.supply("v1", 1024, vec![3; 512]).unwrap();
    assert_eq!(cache.cached_bytes().unwrap(), 1024);
    assert!(matches!(
        cache.read_at(512, &mut out),
        Err(SourceError::MissingRange {
            offset: 512,
            length: 512
        })
    ));
    cache.supply("v1", 512, vec![2; 512]).unwrap();
    cache.read_at(1020, &mut out).unwrap();
    assert_eq!(out, [2, 2, 2, 2, 3, 3, 3, 3]);
    cache.supply("v1", 1536, vec![4; 64]).unwrap();
    assert!(cache.cached_bytes().unwrap() <= 1024);
    assert!(cache.supply("v1", 1536, vec![4; 63]).is_err());
    assert!(cache.supply("v1", u64::MAX, vec![4; 64]).is_err());
    assert!(cache.supply("v1", 1, vec![1; 512]).is_err());
    assert!(cache.read_at(u64::MAX, &mut out).is_err());
    assert_eq!(
        cache.supply("v2", 0, vec![1; 512]),
        Err(SourceError::VersionChanged)
    );
    assert_eq!(cache.cached_bytes().unwrap(), 0);
    assert_eq!(cache.validate(), Err(SourceError::VersionChanged));
    let cache = SparseSource::new(512, "v1".into(), 512, 512).unwrap();
    cache.supply("v1", 0, vec![1; 512]).unwrap();
    cache.supply("v1", 0, vec![1; 512]).unwrap();
    assert_eq!(
        cache.supply("v1", 0, vec![2; 512]),
        Err(SourceError::VersionChanged)
    );
}
#[test]
fn insufficient_cache_and_no_progress_exhaust_retry_budget() {
    let (bytes, _) = fixture(false, false, false);
    let sparse = Arc::new(SparseSource::new(bytes.len() as u64, "v1".into(), 512, 512).unwrap());
    let options = ExternalReaderOptions {
        max_missing_attempts: 8,
        ..Default::default()
    };
    let mut reader = ExternalReader::new(sparse.clone(), options).unwrap();
    for _ in 0..7 {
        match reader.try_read(|_| Ok(())) {
            Err(ExternalReadError::Source(SourceError::MissingRange { offset, length })) => sparse
                .supply(
                    "v1",
                    offset,
                    bytes[offset as usize..offset as usize + length].to_vec(),
                )
                .unwrap(),
            other => panic!("unexpected {other:?}"),
        }
    }
    assert_eq!(
        reader.try_read(|_| Ok(())),
        Err(ExternalReadError::RetryLimitExceeded)
    );
    assert_eq!(
        reader.try_read(|_| Ok(())),
        Err(ExternalReadError::RetryLimitExceeded)
    );
    assert!(sparse.cached_bytes().unwrap() <= 512);
}
#[test]
fn cancellation_and_version_change_are_checked_even_on_cache_hits() {
    let (bytes, _) = fixture(true, false, false);
    for cancel in [false, true] {
        let sparse = Arc::new(
            SparseSource::new(bytes.len() as u64, "v1".into(), 4096, bytes.len() + 4096).unwrap(),
        );
        let mut reader = reader(sparse.clone(), false);
        retry(&mut reader, &sparse, &bytes, |archive| {
            archive.get_file(&path("/data.bin"))
        });
        let expected = if cancel {
            sparse.cancel().unwrap();
            SourceError::Cancelled
        } else {
            assert_eq!(
                sparse.supply("v2", 0, vec![]),
                Err(SourceError::VersionChanged)
            );
            SourceError::VersionChanged
        };
        assert_eq!(
            reader.try_read(|archive| archive.get_file(&path("/data.bin"))),
            Err(ExternalReadError::Source(expected))
        );
    }
    let mut reader = reader(Arc::new(Bytes(Arc::new(bytes))), false);
    reader.cancel();
    assert_eq!(
        reader.try_read(|_| Ok(())),
        Err(ExternalReadError::Source(SourceError::Cancelled))
    );
}
struct InvalidSource {
    failure: Option<SourceError>,
}
impl ReadAtSource for InvalidSource {
    fn len(&self) -> u64 {
        4096
    }
    fn read_at(&self, _: u64, _: &mut [u8]) -> std::result::Result<usize, SourceError> {
        match &self.failure {
            Some(error) => Err(error.clone()),
            None => Ok(0),
        }
    }
}
#[test]
fn short_reads_invalid_missing_ranges_and_limits_fail_closed() {
    for failure in [
        None,
        Some(SourceError::MissingRange {
            offset: u64::MAX,
            length: 32,
        }),
        Some(SourceError::MissingRange {
            offset: 0,
            length: 0,
        }),
        Some(SourceError::Unavailable(
            "transport rejected response".into(),
        )),
    ] {
        let mut reader = reader(Arc::new(InvalidSource { failure }), false);
        assert!(matches!(
            reader.try_read(|_| Ok(())),
            Err(ExternalReadError::Source(SourceError::Unavailable(_)))
        ));
        assert!(reader
            .try_read::<()>(|_| panic!("terminal reader called closure"))
            .is_err());
    }
    let (bytes, _) = fixture(false, false, false);
    let mut reader = ExternalReader::new(
        Arc::new(Bytes(Arc::new(bytes))),
        ExternalReaderOptions {
            max_read_bytes: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(matches!(
        reader.try_read(|_| Ok(())),
        Err(ExternalReadError::Source(SourceError::Unavailable(_)))
    ));
}
#[test]
fn signed_content_corruption_truncation_and_wrong_key_are_rejected() {
    let (mut bytes, payload) = fixture(false, true, false);
    let offset = bytes
        .windows(64)
        .position(|value| value == &payload[..64])
        .unwrap();
    bytes[offset + 12] ^= 1;
    assert!(reader(Arc::new(Bytes(Arc::new(bytes))), false)
        .try_read(|archive| archive.get_file(&path("/data.bin")))
        .is_err());
    let (bytes, _) = fixture(true, true, true);
    let mut wrong = ExternalReader::with_content_key(
        Arc::new(Bytes(Arc::new(bytes.clone()))),
        SecretVec::try_from_slice(&[8; 32]).unwrap(),
        ExternalReaderOptions::default(),
    )
    .unwrap();
    assert!(wrong.try_read(|_| Ok(())).is_err());
    assert!(reader(Arc::new(Bytes(Arc::new(bytes.clone()))), false)
        .try_read(|_| Ok(()))
        .is_err());
    for end in [1, bytes.len() / 2, bytes.len() - 1] {
        assert!(
            reader(Arc::new(Bytes(Arc::new(bytes[..end].to_vec()))), true)
                .try_read(|archive| archive.get_file(&path("/data.bin")))
                .is_err()
        );
    }
}
#[test]
fn independent_readers_share_cache_without_sharing_failure_state() {
    let (bytes, payload) = fixture(true, false, false);
    let bytes = Arc::new(bytes);
    let sparse = Arc::new(
        SparseSource::new(bytes.len() as u64, "v1".into(), 4096, bytes.len() + 4096).unwrap(),
    );
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let sparse = sparse.clone();
            let bytes = bytes.clone();
            std::thread::spawn(move || {
                retry(
                    &mut reader(sparse.clone(), false),
                    &sparse,
                    &bytes,
                    |archive| archive.get_file(&path("/data.bin")),
                )
                .0
            })
        })
        .collect();
    for thread in threads {
        assert_eq!(thread.join().unwrap(), payload);
    }
    assert!(sparse.cached_bytes().unwrap() <= bytes.len());
}

#[test]
fn cumulative_read_budget_bounds_retry_work_without_network_progress() {
    let mut reader = ExternalReader::new(
        Arc::new(InvalidSource {
            failure: Some(SourceError::MissingRange {
                offset: 0,
                length: 512,
            }),
        }),
        ExternalReaderOptions {
            max_source_bytes_per_operation: 4096,
            max_missing_attempts: 1000,
            ..Default::default()
        },
    )
    .unwrap();
    let mut exhausted = false;
    for _ in 0..64 {
        match reader.try_read(|_| Ok(())) {
            Err(ExternalReadError::Source(SourceError::MissingRange { .. })) => {}
            Err(ExternalReadError::Source(SourceError::Unavailable(message))) => {
                assert!(message.contains("cumulative read budget"));
                exhausted = true;
                break;
            }
            other => panic!("unexpected result {other:?}"),
        }
    }
    assert!(
        exhausted,
        "read budget must terminate well before retry count"
    );
    assert!(matches!(
        reader.try_read(|_| Ok(())),
        Err(ExternalReadError::Source(SourceError::Unavailable(_)))
    ));
}

#[test]
fn source_invalidation_inside_an_operation_overrides_cached_success() {
    let (bytes, _) = fixture(true, false, false);
    let sparse = Arc::new(
        SparseSource::new(bytes.len() as u64, "v1".into(), 4096, bytes.len() + 4096).unwrap(),
    );
    let mut reader = reader(sparse.clone(), false);
    retry(&mut reader, &sparse, &bytes, |archive| {
        archive.get_file(&path("/data.bin"))
    });
    let result = reader.try_read(|archive| {
        let bytes = archive.get_file(&path("/data.bin"))?;
        sparse.cancel().unwrap();
        Ok(bytes)
    });
    assert_eq!(
        result,
        Err(ExternalReadError::Source(SourceError::Cancelled))
    );
}

struct MutableLength {
    bytes: Bytes,
    length: AtomicUsize,
}
impl ReadAtSource for MutableLength {
    fn len(&self) -> u64 {
        self.length.load(Ordering::SeqCst) as u64
    }
    fn read_at(&self, offset: u64, out: &mut [u8]) -> std::result::Result<usize, SourceError> {
        self.bytes.read_at(offset, out)
    }
}
#[test]
fn source_length_changes_are_rejected_before_and_after_cached_operations() {
    let (bytes, _) = fixture(true, false, false);
    for during in [false, true] {
        let source = Arc::new(MutableLength {
            length: AtomicUsize::new(bytes.len()),
            bytes: Bytes(Arc::new(bytes.clone())),
        });
        let mut reader = reader(source.clone(), false);
        reader
            .try_read(|archive| archive.get_file(&path("/data.bin")))
            .unwrap();
        if !during {
            source.length.fetch_add(1, Ordering::SeqCst);
        }
        let result = reader.try_read(|archive| {
            let value = archive.get_file(&path("/data.bin"))?;
            if during {
                source.length.fetch_add(1, Ordering::SeqCst);
            }
            Ok(value)
        });
        assert_eq!(
            result,
            Err(ExternalReadError::Source(SourceError::VersionChanged))
        );
    }
}

#[test]
fn sparse_construction_never_allocates_the_logical_archive_length() {
    let source = SparseSource::new(u64::MAX, "large immutable object".into(), 4096, 8192).unwrap();
    assert_eq!(source.len(), u64::MAX);
    assert_eq!(source.cached_bytes().unwrap(), 0);
    assert!(SparseSource::new(0, "v1".into(), 512, 512).is_err());
    assert!(SparseSource::new(512, "".into(), 512, 512).is_err());
    assert!(SparseSource::new(512, "v1".into(), 1, 512).is_err());
    assert!(SparseSource::new(512, "v1".into(), 512, 511).is_err());
}

#[test]
fn a_panicking_callback_cannot_leave_partially_observed_state_cached() {
    let (bytes, payload) = fixture(true, false, false);
    let source = Arc::new(FaultSource {
        bytes: Bytes(Arc::new(bytes)),
        call: AtomicUsize::new(0),
        fail_at: usize::MAX,
        trace: Mutex::new(vec![]),
    });
    let mut reader = reader(source.clone(), false);
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        reader.try_read::<()>(|archive| {
            archive.get_variable(&VariableName::new("TEST_VALUE")?)?;
            panic!("application callback failed")
        })
    }));
    assert!(panic.is_err());
    let reads = source.call.load(Ordering::SeqCst);
    reader.try_read(|_| Ok(())).unwrap();
    assert!(
        source.call.load(Ordering::SeqCst) > reads,
        "must reopen after callback unwinds"
    );
    assert_eq!(
        reader
            .try_read(|archive| archive.get_file(&path("/data.bin")))
            .unwrap(),
        payload
    );
}

struct InvalidValidation;
impl ReadAtSource for InvalidValidation {
    fn len(&self) -> u64 {
        4096
    }
    fn read_at(&self, _: u64, _: &mut [u8]) -> std::result::Result<usize, SourceError> {
        panic!("validation should fail before reads")
    }
    fn validate(&self) -> std::result::Result<(), SourceError> {
        Err(SourceError::MissingRange {
            offset: u64::MAX,
            length: 1,
        })
    }
}
#[test]
fn invalid_missing_ranges_from_validation_cannot_reach_the_transport() {
    assert!(matches!(
        reader(Arc::new(InvalidValidation), false).try_read(|_| Ok(())),
        Err(ExternalReadError::Source(SourceError::Unavailable(_)))
    ));
}
