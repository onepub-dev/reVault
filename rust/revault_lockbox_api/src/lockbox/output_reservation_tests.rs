use super::*;

#[test]
fn whole_file_reservation_rejects_unaddressable_sparse_length() {
    let path = LockboxPath::new("/oversized").unwrap();
    let mut archive = Lockbox::create(b"test-key");
    archive.add_file(&path, &[], false).unwrap();
    archive.commit().unwrap();
    // Unit-level malformed metadata: no real allocation or giant public write
    // is needed to exercise the fallible output-reservation boundary.
    let entry = archive.toc_entries.get_mut(&path).unwrap();
    entry.len = u64::MAX;
    entry.chunks.clear();
    assert!(matches!(
        archive.get_file(&path),
        Err(Error::SecurityLimitExceeded(_))
    ));
}
