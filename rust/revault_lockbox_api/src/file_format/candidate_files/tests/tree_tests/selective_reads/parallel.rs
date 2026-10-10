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
        for workers in [1, 2, 4] {
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
