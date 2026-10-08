use super::*;
pub(super) fn dense_seed(
    mode: FormatMode,
    authority: &Authority<'_>,
    owner: &OwnerSigningKeyPair,
) -> StorageBackend {
    let mut storage = seed(mode, authority, owner);
    tree_image::return_inline(
        &mut storage,
        archive(),
        mode,
        authority,
        mode.signed().then_some(owner),
        key(mode),
    )
    .unwrap();
    assert!(
        shared::open_private(&storage, archive(), mode, authority, key(mode))
            .unwrap()
            .1
            .starts_with(b"RV4DENS3")
    );
    storage
}
pub(super) fn selected(
    storage: &StorageBackend,
    mode: FormatMode,
    authority: &Authority<'_>,
) -> (bool, Vec<u8>, usize) {
    let copy = StorageBackend::memory(storage.read_all().unwrap());
    let dense = shared::open_private(&copy, archive(), mode, authority, key(mode))
        .unwrap()
        .1
        .starts_with(b"RV4DENS3");
    let mut image = if dense {
        super::super::super::super::dense_image::Image::open(
            copy,
            archive(),
            mode,
            authority,
            key(mode),
        )
        .unwrap()
    } else {
        TreeImage::open(copy, archive(), mode, authority, key(mode))
            .unwrap()
            .image
    };
    let mut read = |path: &[u8]| {
        let len = image.info(path).unwrap().unwrap().len;
        let mut bytes = Vec::new();
        image
            .read_range(path, 0, len, |part| {
                bytes.extend_from_slice(part);
                Ok(())
            })
            .unwrap();
        bytes
    };
    assert_eq!(read(b"/docs/neighbor"), b"neighbor");
    let data = read(b"/docs/data");
    let count = image.catalogue.files.len();
    (dense, data, count)
}
#[test]
fn typed_dense_payload_return_capacity_refusals_do_not_relocate() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, false, false, false);
    let authority = authority(mode, &public);
    for envelope in [false, true] {
        let mut storage = seed(mode, &authority, &owner);
        let mut metadata = TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
            .unwrap()
            .image
            .filesystem_metadata()
            .unwrap();
        for i in 0..if envelope { 900 } else { 1100 } {
            let path = if envelope {
                format!("/d-{i:04}-{}", "x".repeat(52))
            } else {
                format!("/d{i:04}")
            };
            metadata.push(Metadata {
                entry: crate::LockboxEntry {
                    path: crate::LockboxPath::new(path).unwrap(),
                    kind: crate::LockboxEntryKind::Directory,
                    len: 0,
                    permissions: 0o700,
                },
                target: None,
            });
        }
        metadata.sort_by(|a, b| a.entry.path.cmp(&b.entry.path));
        tree_image::replace_metadata(
            &mut storage,
            archive(),
            mode,
            &authority,
            None,
            key(mode),
            &metadata,
        )
        .unwrap();
        let before = storage.read_all().unwrap();
        assert!(
            !tree_image::can_return_inline(&storage, archive(), mode, &authority, key(mode))
                .unwrap()
        );
        assert!(tree_image::return_inline(
            &mut storage,
            archive(),
            mode,
            &authority,
            None,
            key(mode)
        )
        .is_err());
        assert_eq!(storage.read_all().unwrap(), before);
    }
}
#[test]
fn typed_dense_payload_dispatch_retains_inline_and_bounded_aging_all_modes() {
    typed_dense_payload_aging(0..16, 32);
}

#[test]
#[ignore = "bounded 1000-cycle qualification; set REVAULT_TYPED_AGING_GROUP=0..3"]
fn typed_dense_payload_thousand_cycle_qualification() {
    let group: u8 = std::env::var("REVAULT_TYPED_AGING_GROUP")
        .unwrap()
        .parse()
        .unwrap();
    assert!(group < 4);
    typed_dense_payload_aging(group * 4..group * 4 + 4, 1000);
}

fn typed_dense_payload_aging(modes: std::ops::Range<u8>, cycles: usize) {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in modes {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = dense_seed(mode, &authority, &owner);
        let mut warmup_peak = 0;
        for cycle in 0..cycles {
            let data = vec![cycle as u8; if cycle % 3 == 0 { 70000 } else { 2000 }];
            tree_image::update_files(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                inputs(&data),
                &[],
            )
            .unwrap();
            let actual = selected(&storage, mode, &authority);
            assert!(actual.0);
            assert_eq!(actual.1, data);
            assert_eq!(actual.2, 4);
            let before = storage.read_all().unwrap();
            assert!(!tree_image::update_files(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                inputs(&data),
                &[]
            )
            .unwrap());
            assert_eq!(storage.read_all().unwrap(), before);
            if cycle % 2 == 0 {
                tree_image::remove_files(
                    &mut storage,
                    archive(),
                    mode,
                    &authority,
                    signer,
                    key(mode),
                    &[b"/docs/new".to_vec(), b"/docs/zero".to_vec()],
                )
                .unwrap();
                let actual = selected(&storage, mode, &authority);
                assert!(actual.0);
                assert_eq!(actual.1, data);
                assert_eq!(actual.2, 2);
            }
            if cycle < 16 {
                warmup_peak = warmup_peak.max(storage.len().unwrap());
            } else {
                assert!(
                    storage.len().unwrap() <= warmup_peak,
                    "mode={bits} cycle={cycle}"
                );
            }
        }
        for bank in [0, FAILURE_REGION] {
            let mut lost = storage.clone();
            lost.write_at(bank, &vec![0; FAILURE_REGION as usize])
                .unwrap();
            tree_image::recover(&mut lost, archive(), mode, &authority, key(mode)).unwrap();
            assert_eq!(
                selected(&lost, mode, &authority),
                selected(&storage, mode, &authority)
            );
        }
        println!("TYPED_DENSE_PAYLOAD_AGING mode={bits} cycles={cycles} warmup_completed_peak={warmup_peak} final_bytes={}", storage.len().unwrap());
    }
}
#[test]
fn typed_dense_payload_interruptions_preserve_complete_selected_file_set() {
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for bits in [0, 3, 12, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let seed = dense_seed(mode, &authority, &owner).read_all().unwrap();
        let run = |storage: &mut CrashStore| {
            tree_image::update_files(
                storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                inputs(b"replacement"),
                &[],
            )
        };
        let mut observed = CrashStore::new(seed.clone(), None, 0, false);
        run(&mut observed).unwrap();
        for at in 0..observed.operations() {
            for prefix in [0, 97, usize::MAX] {
                for persist in [false, true] {
                    let mut failed = CrashStore::new(seed.clone(), Some(at), prefix, persist);
                    let _ = run(&mut failed);
                    let mut reopened = StorageBackend::memory(failed.durable());
                    tree_image::recover(&mut reopened, archive(), mode, &authority, key(mode))
                        .unwrap_or_else(|e| {
                            panic!("mode={bits} at={at} prefix={prefix} persist={persist}: {e}")
                        });
                    let actual = selected(&reopened, mode, &authority);
                    assert!(actual.1 == b"payload" || actual.1 == b"replacement");
                    assert_eq!(actual.2, if actual.1 == b"payload" { 2 } else { 4 });
                    tree_image::recover(&mut reopened, archive(), mode, &authority, key(mode))
                        .unwrap();
                    assert_eq!(selected(&reopened, mode, &authority), actual);
                    cases += 1;
                }
            }
        }
    }
    println!("TYPED_DENSE_PAYLOAD_INTERRUPTION_CASES {cases}");
}
