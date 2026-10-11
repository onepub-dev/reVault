//! Internal experimental API: public archives do not expose this worker policy.
use super::*;

#[test]
fn parallel_selected_reads_preserve_order_errors_and_pool_reuse_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (storage, bytes) = files(mode, &authority, &owner);
        let audit = AuditedTreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
            .unwrap();
        let bad = &audit.image.catalogue.files[0].fragments[1];
        let bad_at = audit.image.catalogue.packs[bad.pack].extent.start + bad.relative as u64;
        for workers in [0, 1, 2, 4] {
            // Rc/RefCell storage is intentionally !Send/!Sync: no I/O crosses to
            // the pool. The sink also stays on this thread.
            let trace = Trace {
                storage: storage.clone(),
                reads: Rc::new(RefCell::new(Vec::new())),
            };
            let mut reader =
                TreeImage::open(trace, archive(), mode, &authority, key(mode)).unwrap();
            reader.image.set_read_workers(workers).unwrap();
            assert!(reader.image.set_read_workers(3).is_err());
            assert_eq!(
                read(&mut reader, b"/file-a", 0, bytes.len() as u64).unwrap(),
                bytes
            );
            assert_eq!(
                read(&mut reader, b"/file-z", 123, 135_000).unwrap(),
                bytes[123..135_123]
            );
            let mut calls = 0;
            assert!(reader
                .image
                .read_range(b"/file-a", 0, bytes.len() as u64, |_| {
                    calls += 1;
                    Err(Error::Io("synthetic sink failure".into()))
                })
                .is_err());
            assert_eq!(calls, 1);
            let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ =
                    reader
                        .image
                        .read_range(b"/file-a", 0, bytes.len() as u64, |_| -> Result<()> {
                            panic!("synthetic sink unwind")
                        });
            }));
            assert!(panic.is_err());
            assert_eq!(
                read(&mut reader, b"/file-a", 0, bytes.len() as u64).unwrap(),
                bytes
            );
            assert_eq!(
                read(&mut reader, b"/file-a", 31, 17).unwrap(),
                bytes[31..48]
            );

            let mut damaged = StorageBackend::memory(storage.read_all().unwrap());
            let old = damaged.read_at(bad_at, 1).unwrap()[0];
            damaged.write_at(bad_at, &[old ^ 1]).unwrap();
            let mut reader =
                TreeImage::open(damaged, archive(), mode, &authority, key(mode)).unwrap();
            reader.image.set_read_workers(workers).unwrap();
            let mut prefix = Vec::new();
            assert!(reader
                .image
                .read_range(b"/file-a", 0, bytes.len() as u64, |part| {
                    prefix.extend_from_slice(part);
                    Ok(())
                })
                .is_err());
            assert_eq!(prefix, bytes[..bad.descriptor.offset as usize]);
            assert!(reader.image.verify_all().is_err());
        }
    }
}

#[test]
fn automatic_workers_qualify_each_request_and_reuse_pool_after_errors() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        let mode = mode(encrypted, true, true, true);
        let authority = authority(mode, &public);
        let (storage, bytes) = files_with_len(mode, &authority, &owner, 33 * 1024 * 1024);
        let audit = AuditedTreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
            .unwrap();
        let fragments = &audit.image.catalogue.files[0].fragments;
        let payloads: Vec<_> = fragments
            .iter()
            .map(|fragment| {
                let start = audit.image.catalogue.packs[fragment.pack].extent.start
                    + fragment.relative as u64;
                (start, start + fragment.descriptor.stored_len() as u64)
            })
            .collect();
        let bad = &fragments[16 * 1024 * 1024 / 65_536 + 5];
        let bad_offset = bad.descriptor.offset as usize;
        let bad_at = audit.image.catalogue.packs[bad.pack].extent.start + bad.relative as u64;
        let mut damaged = StorageBackend::memory(storage.read_all().unwrap());
        let old = damaged.read_at(bad_at, 1).unwrap()[0];
        damaged.write_at(bad_at, &[old ^ 1]).unwrap();
        let trace = Trace {
            storage,
            reads: Rc::new(RefCell::new(Vec::new())),
        };
        let observed = trace.reads.clone();
        let mut reader = TreeImage::open(trace, archive(), mode, &authority, key(mode)).unwrap();
        reader.image.set_read_workers(0).unwrap();
        for (offset, len) in [(65_530usize, 31usize), (0, 16 * 1024 * 1024)] {
            assert_eq!(
                read(&mut reader, b"/file-a", offset as u64, len as u64).unwrap(),
                bytes[offset..offset + len]
            );
            assert_eq!(reader.image.read_pool_workers(), 0);
        }
        let mut seen = 0usize;
        observed.borrow_mut().clear();
        assert!(reader
            .image
            .read_range(b"/file-a", 0, bytes.len() as u64, |part| {
                assert_eq!(part, &bytes[seen..seen + part.len()]);
                seen += part.len();
                let fetched = observed
                    .borrow()
                    .iter()
                    .filter(|read| payloads.contains(read))
                    .count();
                if seen > 16 * 1024 * 1024 {
                    assert_eq!(fetched, 256 + 32);
                    Err(Error::Io("synthetic auto sink failure".into()))
                } else {
                    assert_eq!(fetched, seen.div_ceil(65_536));
                    Ok(())
                }
            })
            .is_err());
        assert_eq!(reader.image.read_pool_workers(), 4);
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut seen = 0usize;
            let _ =
                reader
                    .image
                    .read_range(b"/file-a", 0, bytes.len() as u64, |part| -> Result<()> {
                        seen += part.len();
                        if seen > 16 * 1024 * 1024 {
                            panic!("synthetic auto sink unwind");
                        }
                        Ok(())
                    });
        }));
        assert!(panic.is_err());
        assert_eq!(
            read(&mut reader, b"/file-a", 0, bytes.len() as u64).unwrap(),
            bytes
        );
        assert_eq!(
            read(&mut reader, b"/file-z", 65_530, 31).unwrap(),
            bytes[65_530..65_561]
        );
        assert_eq!(reader.image.read_pool_workers(), 4);
        reader.image.set_read_workers(1).unwrap();
        assert_eq!(reader.image.read_pool_workers(), 0);
        let mut reader = TreeImage::open(damaged, archive(), mode, &authority, key(mode)).unwrap();
        reader.image.set_read_workers(0).unwrap();
        let mut prefix = Vec::new();
        assert!(reader
            .image
            .read_range(b"/file-a", 0, bytes.len() as u64, |part| {
                prefix.extend_from_slice(part);
                Ok(())
            })
            .is_err());
        assert_eq!(reader.image.read_pool_workers(), 4);
        assert_eq!(prefix, bytes[..bad_offset]);
    }
}

#[test]
fn automatic_workers_reset_on_raw_chunks_and_keep_serial_callbacks_bounded() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, false, true, true);
    let authority = authority(mode, &public);
    let unit = 65_536usize;
    let run = 17 * 1024 * 1024;
    let second_run = 16 * 1024 * 1024;
    let mut bytes = content(run + second_run + unit);
    let mut random = 0x8164_1a37_a9c2_e51bu64;
    for byte in &mut bytes[run..run + unit] {
        random ^= random << 13;
        random ^= random >> 7;
        random ^= random << 17;
        *byte = random as u8;
    }
    let (storage, bytes) = files_with_bytes(mode, &authority, &owner, bytes);
    let audit =
        AuditedTreeImage::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
    let payloads: Vec<_> = audit.image.catalogue.files[0]
        .fragments
        .iter()
        .map(|fragment| {
            let start =
                audit.image.catalogue.packs[fragment.pack].extent.start + fragment.relative as u64;
            (start, start + fragment.descriptor.stored_len() as u64)
        })
        .collect();
    let trace = Trace {
        storage,
        reads: Rc::new(RefCell::new(Vec::new())),
    };
    let observed = trace.reads.clone();
    let mut reader = TreeImage::open(trace, archive(), mode, &authority, key(mode)).unwrap();
    let census = reader.image.codec_census(b"/file-a").unwrap();
    assert_eq!(census.0, 1);
    assert_eq!(census.1, ((run + second_run) / unit) as u64);
    reader.image.set_read_workers(0).unwrap();
    for _ in 0..2 {
        observed.borrow_mut().clear();
        let mut position = 0usize;
        reader
            .image
            .read_range(b"/file-a", 0, bytes.len() as u64, |part| {
                assert_eq!(part, &bytes[position..position + part.len()]);
                position += part.len();
                let fetched = observed
                    .borrow()
                    .iter()
                    .filter(|read| payloads.contains(read))
                    .count();
                if position <= 16 * 1024 * 1024 || position > run {
                    assert_eq!(fetched, position.div_ceil(unit));
                } else {
                    assert_eq!(fetched, run / unit);
                    assert!(fetched <= position.div_ceil(unit) + 31);
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(position, bytes.len());
        assert_eq!(reader.image.read_pool_workers(), 4);
    }
}

#[test]
fn parallel_selected_reads_bound_input_batches_before_callbacks() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, false, false, true);
    let authority = authority(mode, &public);
    let (storage, bytes) = files_with_len(mode, &authority, &owner, 40 * 65_536 + 17);
    let audit =
        AuditedTreeImage::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
    let payloads: Vec<_> = audit.image.catalogue.files[0]
        .fragments
        .iter()
        .map(|fragment| {
            let start =
                audit.image.catalogue.packs[fragment.pack].extent.start + fragment.relative as u64;
            (start, start + fragment.descriptor.stored_len() as u64)
        })
        .collect();
    for workers in [2, 4] {
        let trace = Trace {
            storage: storage.clone(),
            reads: Rc::new(RefCell::new(Vec::new())),
        };
        let observed = trace.reads.clone();
        let mut reader = TreeImage::open(trace, archive(), mode, &authority, key(mode)).unwrap();
        reader.image.set_read_workers(workers).unwrap();
        let mut offset = 0;
        reader
            .image
            .read_range(b"/file-a", 0, bytes.len() as u64, |part| {
                if offset == 0 {
                    let fetched = observed
                        .borrow()
                        .iter()
                        .filter(|read| payloads.contains(read))
                        .count();
                    assert_eq!(fetched, workers * 8);
                }
                assert_eq!(part, &bytes[offset..offset + part.len()]);
                offset += part.len();
                Ok(())
            })
            .unwrap();
        assert_eq!(offset, bytes.len());
        observed.borrow_mut().clear();
        let mut prefix = Vec::new();
        // A tiny request straddling two chunks must retain serial first-byte
        // behavior, even after this session has already used its worker pool.
        reader
            .image
            .read_range(b"/file-a", 65_533, 7, |part| {
                if prefix.is_empty() {
                    let fetched = observed
                        .borrow()
                        .iter()
                        .filter(|read| payloads.contains(read))
                        .count();
                    assert_eq!(fetched, 1);
                }
                prefix.extend_from_slice(part);
                Ok(())
            })
            .unwrap();
        assert_eq!(prefix, bytes[65_533..65_540]);
    }
}
