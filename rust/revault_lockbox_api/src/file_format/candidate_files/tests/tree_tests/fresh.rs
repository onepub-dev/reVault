//! Private adapter tests: no public CLI creates this experimental packed/tree format.
use super::*;

fn entries(count: usize, width: usize, len: u64) -> Vec<Metadata> {
    (0..count)
        .map(|index| Metadata {
            entry: crate::LockboxEntry {
                path: crate::LockboxPath::new(format!("/file-{index:04}-{}", "x".repeat(width)))
                    .unwrap(),
                kind: crate::LockboxEntryKind::File,
                len,
                permissions: 0o640,
            },
            target: None,
        })
        .collect()
}
fn seed(
    mode: FormatMode,
    authority: &Authority<'_>,
    owner: &OwnerSigningKeyPair,
    entries: &[Metadata],
) -> StorageBackend {
    Files::create(
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        authority,
        mode.signed().then_some(owner),
        key(mode),
        MAX_LOGICAL,
        entries.iter().enumerate().map(|(index, entry)| Input {
            path: entry.entry.path.as_str().as_bytes().to_vec(),
            reader: Cursor::new(vec![index as u8; entry.entry.len as usize]),
        }),
    )
    .unwrap()
}
fn verify<S: Storage>(
    storage: S,
    mode: FormatMode,
    authority: &Authority<'_>,
    entries: &[Metadata],
    source: &Files<impl Storage>,
) {
    let mut opened =
        AuditedTreeImage::open(storage, archive(), mode, authority, key(mode)).unwrap();
    assert_eq!(opened.image.filesystem_metadata().unwrap(), entries);
    assert!(opened
        .image
        .catalogue
        .dense_body_if_fits(&opened.image.codec, opened.image.anchor.sealed_len)
        .unwrap()
        .is_none());
    assert_eq!(opened.image.catalogue.files.len(), entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let path = entry.entry.path.as_str().as_bytes();
        let expected = source.info(path).unwrap().unwrap();
        let actual = opened.image.info(path).unwrap().unwrap();
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.len, expected.len);
        assert_eq!(actual.digest, expected.digest);
        let mut count = 0;
        opened
            .image
            .read_range(path, 0, entry.entry.len, |bytes| {
                assert!(bytes.iter().all(|byte| *byte == index as u8));
                count += bytes.len();
                Ok(())
            })
            .unwrap();
        assert_eq!(count as u64, entry.entry.len);
    }
}
#[test]
fn fresh_tree_export_beyond_dense_capacity_preserves_files_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let entries = entries(512, 170, 4096);
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let original = seed(mode, &authority, &owner, &entries).read_all().unwrap();
        let mut source = Files::open(
            StorageBackend::memory(original.clone()),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let output = tree_image::from_candidate(
            &mut source,
            StorageBackend::memory(Vec::new()),
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &entries,
        )
        .unwrap();
        verify(
            StorageBackend::memory(output.read_all().unwrap()),
            mode,
            &authority,
            &entries,
            &source,
        );
        assert_eq!(source.storage.read_all().unwrap(), original);
    }
}
#[test]
fn fresh_tree_export_failures_erase_destination_and_preserve_source() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let entries = entries(256, 220, 17);
    let mut cases = 0;
    for mode in [
        mode(false, false, true, true),
        mode(true, true, true, true),
        mode(false, true, false, false),
        mode(true, false, false, true),
    ] {
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let original = seed(mode, &authority, &owner, &entries).read_all().unwrap();
        let mut source = Files::open(
            StorageBackend::memory(original.clone()),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let observed = SharedMemory::new(Vec::new());
        tree_image::from_candidate(
            &mut source,
            observed.clone(),
            &authority,
            signer,
            key(mode),
            &entries,
        )
        .unwrap();
        verify(observed.clone(), mode, &authority, &entries, &source);
        for at in 0..observed.operations() {
            let output = SharedMemory::new(Vec::new());
            output.fail(at);
            assert!(
                tree_image::from_candidate(
                    &mut source,
                    output.clone(),
                    &authority,
                    signer,
                    key(mode),
                    &entries
                )
                .is_err(),
                "failure {at}"
            );
            assert_eq!(output.len().unwrap(), 0);
            assert_eq!(source.storage.read_all().unwrap(), original);
            cases += 1;
        }
        let output = SharedMemory::new(b"existing destination".to_vec());
        assert!(tree_image::from_candidate(
            &mut source,
            output.clone(),
            &authority,
            signer,
            key(mode),
            &entries
        )
        .is_err());
        assert_eq!(output.read_all().unwrap(), b"existing destination");
    }
    println!("FRESH_TREE_EXPORT_FAILURES {cases}");
}
#[test]
fn fresh_tree_export_refuses_metadata_mismatch_wrong_owner_and_corruption() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(true, true, true, true);
    let authority = authority(mode, &public);
    let entries = entries(256, 220, 17);
    let original = seed(mode, &authority, &owner, &entries).read_all().unwrap();
    let mut source = Files::open(
        StorageBackend::memory(original.clone()),
        archive(),
        mode,
        &authority,
        key(mode),
    )
    .unwrap();
    for attempt in 0..4 {
        let mut changed = entries.clone();
        match attempt {
            0 => {
                changed.pop();
            }
            1 => {
                let mut extra = changed[0].clone();
                extra.entry.path = crate::LockboxPath::new("/extra").unwrap();
                changed.push(extra);
            }
            2 => changed[0].entry.len += 1,
            _ => changed[0].entry.path = crate::LockboxPath::new("/renamed").unwrap(),
        }
        let output = SharedMemory::new(Vec::new());
        assert!(tree_image::from_candidate(
            &mut source,
            output.clone(),
            &authority,
            Some(&owner),
            key(mode),
            &changed
        )
        .is_err());
        assert_eq!(output.len().unwrap(), 0);
        assert_eq!(source.storage.read_all().unwrap(), original);
    }
    let wrong = OwnerSigningKeyPair::generate().unwrap();
    let wrong_public = wrong.public_key();
    let output = SharedMemory::new(Vec::new());
    assert!(tree_image::from_candidate(
        &mut source,
        output.clone(),
        &Authority::Owner(&wrong_public),
        Some(&wrong),
        key(mode),
        &entries
    )
    .is_err());
    assert_eq!(output.operations(), 0);
    // No public candidate access-tree writer exists; fabricate only the cached
    // marker to exercise the unconditional access-translation refusal.
    let original_keys = source.anchor.keys;
    source.anchor.keys = source.anchor.index;
    assert!(tree_image::from_candidate(
        &mut source,
        output.clone(),
        &authority,
        Some(&owner),
        key(mode),
        &entries
    )
    .is_err());
    assert_eq!(output.operations(), 0);
    source.anchor.keys = original_keys;
    let pack = first_record(&source, entries[0].entry.path.as_str().as_bytes()).extents[0];
    let byte = source.storage.read_at(pack.start, 1).unwrap()[0];
    source.storage.write_at(pack.start, &[byte ^ 1]).unwrap();
    let corrupt = source.storage.read_all().unwrap();
    assert!(tree_image::from_candidate(
        &mut source,
        output.clone(),
        &authority,
        Some(&owner),
        key(mode),
        &entries
    )
    .is_err());
    assert_eq!(output.len().unwrap(), 0);
    assert_eq!(source.storage.read_all().unwrap(), corrupt);
}

// Synthetic storage hooks are necessary to interleave a competing publication;
// the private adapter requires a stable snapshot/read lock in real callers.
#[derive(Clone, Debug)]
struct ChangingDestination {
    output: SharedMemory,
    source: SharedMemory,
    successor: Option<Vec<u8>>,
    during_publication: bool,
}
impl ChangingDestination {
    fn change_source(&mut self) {
        if let Some(bytes) = self.successor.take() {
            *self.source.0.lock().unwrap() = StorageBackend::memory(bytes);
        }
    }
}
impl Storage for ChangingDestination {
    fn len(&self) -> Result<u64> {
        self.output.len()
    }
    fn read_at(&self, at: u64, len: usize) -> Result<Vec<u8>> {
        self.output.read_at(at, len)
    }
    fn read_at_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        self.output.read_at_into(at, out)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        let at = self.output.append(bytes)?;
        if !self.during_publication
            && at == crate::file_format::publication_anchor::REGION_LEN as u64
        {
            self.change_source();
        }
        Ok(at)
    }
    fn write_at(&mut self, at: u64, bytes: &[u8]) -> Result<()> {
        self.output.write_at(at, bytes)?;
        if self.during_publication {
            self.change_source();
        }
        Ok(())
    }
    fn truncate(&mut self, len: u64) -> Result<()> {
        self.output.truncate(len)
    }
    fn sync(&self) -> Result<()> {
        self.output.sync()
    }
}
#[test]
fn fresh_tree_export_rechecks_source_before_and_after_publication() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(true, true, true, true);
    let authority = authority(mode, &public);
    let entries = entries(256, 220, 17);
    let original = seed(mode, &authority, &owner, &entries).read_all().unwrap();
    for during_publication in [false, true] {
        let source_storage = SharedMemory::new(original.clone());
        let mut source = Files::open(
            source_storage.clone(),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let base = source.anchor.commitment().unwrap();
        let mut next = source.anchor.clone();
        next.generation += 1;
        next.previous = base;
        let mut successor = StorageBackend::memory(original.clone());
        publication::publish(&mut successor, &next, &authority, Some(&owner), Some(base)).unwrap();
        let successor = successor.read_all().unwrap();
        let output = SharedMemory::new(Vec::new());
        let destination = ChangingDestination {
            output: output.clone(),
            source: source_storage.clone(),
            successor: Some(successor.clone()),
            during_publication,
        };
        assert!(tree_image::from_candidate(
            &mut source,
            destination,
            &authority,
            Some(&owner),
            key(mode),
            &entries
        )
        .is_err());
        assert_eq!(output.len().unwrap(), 0);
        assert_eq!(source_storage.read_all().unwrap(), successor);
        assert_eq!(
            publication::select(&source_storage, archive(), mode, &authority)
                .unwrap()
                .commitment,
            next.commitment().unwrap()
        );
    }
}

#[derive(Clone, Debug)]
struct CleanupFailure(SharedMemory);
impl Storage for CleanupFailure {
    fn len(&self) -> Result<u64> {
        self.0.len()
    }
    fn read_at(&self, at: u64, len: usize) -> Result<Vec<u8>> {
        self.0.read_at(at, len)
    }
    fn read_at_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        self.0.read_at_into(at, out)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        self.0.append(bytes)
    }
    fn write_at(&mut self, at: u64, bytes: &[u8]) -> Result<()> {
        self.0.write_at(at, bytes)
    }
    fn truncate(&mut self, _: u64) -> Result<()> {
        Err(Error::InvalidOperation("synthetic truncate failure".into()))
    }
    fn sync(&self) -> Result<()> {
        self.0.sync()
    }
}
#[test]
fn fresh_tree_export_reports_cleanup_failure_without_claiming_empty_output() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(true, true, true, true);
    let authority = authority(mode, &public);
    let entries = entries(256, 220, 17);
    let storage = seed(mode, &authority, &owner, &entries);
    let original = storage.read_all().unwrap();
    let mut source = Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
    let output = SharedMemory::new(Vec::new());
    let result = tree_image::from_candidate(
        &mut source,
        CleanupFailure(output.clone()),
        &authority,
        Some(&owner),
        key(mode),
        &entries[..entries.len() - 1],
    );
    assert!(
        matches!(result, Err(Error::InvalidOperation(ref reason)) if reason.contains("destination cleanup failed") && reason.contains("synthetic truncate failure"))
    );
    let bytes = output.read_all().unwrap();
    assert!(!bytes.is_empty());
    assert!(bytes.iter().all(|byte| *byte == 0));
    assert_eq!(source.storage.read_all().unwrap(), original);
}

#[test]
#[ignore = "manual file-backed 64 MiB streaming export probe"]
fn fresh_tree_export_streams_sixty_four_mib_without_dense_intermediate() {
    struct Directory(std::path::PathBuf);
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let mut nonce = [0; 8];
    getrandom::fill(&mut nonce).unwrap();
    let directory = Directory(std::env::temp_dir().join(format!(
        "revault-fresh-tree-{}-{}",
        std::process::id(),
        u64::from_le_bytes(nonce)
    )));
    std::fs::create_dir(&directory.0).unwrap();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut entries = entries(1, 8, 64 * 1024 * 1024);
    entries[0].entry.path = crate::LockboxPath::new("/large").unwrap();
    for mode in [
        mode(false, false, false, true),
        mode(false, true, true, true),
        mode(true, true, true, true),
    ] {
        let authority = authority(mode, &public);
        let source_path = directory.0.join(format!("source-{}", mode.0));
        let output_path = directory.0.join(format!("output-{}", mode.0));
        let storage = Files::create(
            StorageBackend::create_file(&source_path, &[]).unwrap(),
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            65536,
            [Input {
                path: b"/large".to_vec(),
                reader: Pattern {
                    position: 0,
                    len: entries[0].entry.len,
                    fail: None,
                },
            }],
        )
        .unwrap();
        let mut source = Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
        let before = source.anchor.commitment().unwrap();
        let output = tree_image::from_candidate(
            &mut source,
            StorageBackend::create_file(&output_path, &[]).unwrap(),
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &entries,
        )
        .unwrap();
        drop(output);
        let mut reopened = AuditedTreeImage::open(
            StorageBackend::file(&output_path).unwrap(),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let dense_fits = reopened
            .image
            .catalogue
            .dense_body_if_fits(&reopened.image.codec, reopened.image.anchor.sealed_len)
            .unwrap()
            .is_some();
        // The original raw 64 MiB failure has 1024 physical packs and exceeds
        // dense metadata capacity. Compression can share far fewer packs, so its
        // catalogue may fit even with the same 1024 logical fragments.
        if mode.options().compression == Compression::None {
            assert!(!dense_fits);
        }
        assert_eq!(reopened.image.catalogue.files[0].fragments.len(), 1024);
        let mut position = 0u64;
        reopened
            .image
            .read_range(b"/large", 0, entries[0].entry.len, |bytes| {
                for byte in bytes {
                    assert_eq!(*byte, pattern(position));
                    position += 1;
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(position, entries[0].entry.len);
        assert_eq!(source.anchor.commitment().unwrap(), before);
        assert_eq!(
            publication::select(&source.storage, archive(), mode, &authority)
                .unwrap()
                .commitment,
            before
        );
        source.audit_with(true).unwrap();
        println!("FRESH_TREE_STREAM mode={} source_bytes={} output_bytes={} logical_bytes={} fragments=1024 dense_fits={}", mode.0, source.storage.len().unwrap(), reopened.image.storage.len().unwrap(), position, dense_fits);
    }
}

#[test]
fn fresh_tree_export_refuses_authenticated_existing_access_tree() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(true, true, true, true);
    let authority = authority(mode, &public);
    let entries = entries(256, 220, 17);
    let storage = seed(mode, &authority, &owner, &entries);
    // No public packed-C access writer exists. Use its authenticated allocation
    // transaction to store a real synthetic password slot directory as a key
    // record; this is a persisted access-root fixture, not an invented anchor.
    let slot = crate::key_slot::KeySlot::password_bytes(
        1,
        b"synthetic fresh-export password",
        vec![27; 16],
        KEY,
    )
    .unwrap();
    let directory = publication::bootstrap::directory(archive(), 1, &[slot]).unwrap();
    let mut transaction =
        Transaction::begin(storage, archive(), mode, &authority, key(mode)).unwrap();
    transaction
        .put_key_record(1, b"synthetic-directory", &directory, &[])
        .unwrap();
    let (storage, _) = transaction.commit(&authority, Some(&owner)).unwrap();
    let before = storage.read_all().unwrap();
    let mut source = Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
    assert_ne!(source.anchor.keys, publication::RootRef::default());
    let entry = source
        .index
        .get(
            &source.storage,
            source.anchor.keys,
            source.anchor.sealed_len,
            1,
            b"synthetic-directory",
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        OwnedRecord::decode(&entry.value)
            .unwrap()
            .metadata
            .as_slice(),
        directory
    );
    let output = SharedMemory::new(Vec::new());
    let result = tree_image::from_candidate(
        &mut source,
        output.clone(),
        &authority,
        Some(&owner),
        key(mode),
        &entries,
    );
    assert!(
        matches!(result, Err(Error::InvalidOperation(ref reason)) if reason.contains("translate access slots"))
    );
    assert_eq!(output.operations(), 0);
    assert_eq!(output.len().unwrap(), 0);
    assert_eq!(source.storage.read_all().unwrap(), before);
}

#[test]
fn fresh_tree_export_exceeds_old_file_and_fragment_counts() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, false, false, false);
    let authority = authority(mode, &public);
    let entries = entries(10_000, 2, 17);
    let storage = seed(mode, &authority, &owner, &entries);
    let before = storage.read_all().unwrap();
    let mut source = Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
    let output = SharedMemory::new(Vec::new());
    tree_image::from_candidate(
        &mut source,
        output.clone(),
        &authority,
        None,
        key(mode),
        &entries,
    )
    .unwrap();
    verify(output.clone(), mode, &authority, &entries, &source);
    // Independently opened full payload validation must still catch a changed
    // committed pack even when metadata admission accepts the larger inventory.
    let mut reopened =
        AuditedTreeImage::open(output.clone(), archive(), mode, &authority, key(mode)).unwrap();
    reopened.image.verify_all().unwrap();
    let start = reopened.image.catalogue.packs[0].extent.start;
    drop(reopened);
    let mut damaged = output.clone();
    let byte = damaged.read_at(start, 1).unwrap()[0] ^ 1;
    damaged.write_at(start, &[byte]).unwrap();
    let result = AuditedTreeImage::open(damaged, archive(), mode, &authority, key(mode));
    assert!(result
        .and_then(|mut image| image.image.verify_all())
        .is_err());
    assert_eq!(source.storage.read_all().unwrap(), before);
}

#[test]
fn fresh_tree_export_metadata_budget_refusal_cleans_destination() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, false, false, false);
    let authority = authority(mode, &public);
    let mut entries = entries(1, 2, 17);
    let storage = seed(mode, &authority, &owner, &entries);
    let before = storage.read_all().unwrap();
    let mut source = Files::open(storage, archive(), mode, &authority, key(mode)).unwrap();
    // All directories are valid independent root children; their metadata alone
    // exceeds the admission budget, without constructing an oversized payload.
    for index in 0..32_768 {
        entries.push(Metadata {
            entry: crate::LockboxEntry {
                path: crate::LockboxPath::new(format!("/directory-{index:05}")).unwrap(),
                kind: crate::LockboxEntryKind::Directory,
                len: 0,
                permissions: 0o755,
            },
            target: None,
        });
    }
    let output = SharedMemory::new(Vec::new());
    let result = tree_image::from_candidate(
        &mut source,
        output.clone(),
        &authority,
        None,
        key(mode),
        &entries,
    );
    assert!(
        matches!(result, Err(Error::SecurityLimitExceeded(ref reason)) if reason.contains("metadata byte budget"))
    );
    assert_eq!(output.len().unwrap(), 0);
    assert_eq!(source.storage.read_all().unwrap(), before);
}
