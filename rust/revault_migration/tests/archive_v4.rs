use archive_v3 as old;
use revault_lockbox_api as current;
use revault_migration::{import_archive, verify_imported_archive};
fn p(value: &str) -> old::LockboxPath {
    old::LockboxPath::new(value).unwrap()
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
#[test]
fn migration_removes_abandoned_imports_preserves_owner_and_verifies_resumed_output() {
    let temp = tempfile::tempdir().unwrap();
    let password = old::SecretString::try_from_slice(b"synthetic migration password").unwrap();
    let owner = old::OwnerSigningKeyPair::generate().unwrap();
    let owner_current =
        current::OwnerSigningKeyPair::from_private_key_record(owner.private_key_record().unwrap())
            .unwrap();
    let mut source =
        old::Lockbox::create_in_memory(old::LockboxProtection::Password(&password), &owner)
            .unwrap();
    let live = noise(256 * 1024);
    source.add_file(&p("/keep.bin"), &live, false).unwrap();
    source.set_permissions(&p("/keep.bin"), 0o600).unwrap();
    source.create_dir(&p("/empty"), true).unwrap();
    source
        .add_symlink(&p("/link"), &p("/keep.bin"), false)
        .unwrap();
    source.set_description("migration fixture").unwrap();
    let variable = old::VariableName::new("/token").unwrap();
    source
        .set_secret_variable(
            &variable,
            &old::SecretString::try_from_slice(&vec![b's'; 16384]).unwrap(),
        )
        .unwrap();
    source
        .define_form(
            "login",
            "Login",
            vec![old::FormFieldDefinition {
                id: "password".into(),
                label: "Password".into(),
                kind: old::FormFieldKind::Text,
                required: false,
            }],
        )
        .unwrap();
    source
        .create_form_record(&p("/account"), "login", "Account")
        .unwrap();
    source
        .set_form_field_secret(
            &p("/account"),
            "password",
            &old::SecretString::try_from_slice(b"form secret").unwrap(),
        )
        .unwrap();
    source
        .create_mirror_project(
            old::MirrorProject {
                name: "house".into(),
                source: temp.path().to_string_lossy().into_owned(),
                destination: p("/house"),
                includes: Vec::new(),
                excludes: vec!["*.tmp".into()],
                missing_file_policy: old::MirrorMissingFilePolicy::Remove,
                strict: true,
                host_identity: None,
            },
            false,
        )
        .unwrap();
    source.commit().unwrap();
    let payload = noise(10 * 1024 * 1024);
    for _ in 0..4 {
        source.set_workload_profile(old::WorkloadProfile::BulkImport);
        let before = source.try_to_bytes().unwrap().len();
        source
            .with_mirror_project_mutation("house", |box_, _| {
                box_.add_file(&p("/house/abandoned.bin"), &payload, false)
            })
            .unwrap();
        let interrupted = source.try_to_bytes().unwrap();
        assert!(interrupted.len() > before + 8 * 1024 * 1024);
        // Model process termination after streaming an import but before its
        // publication; the historical reader has no preparation rollback.
        source = old::Lockbox::open_bytes_for_write(
            interrupted,
            old::LockboxOpen::Password(&password),
            &owner,
        )
        .unwrap();
        assert!(source.get_file(&p("/house/abandoned.bin")).is_err());
    }
    let original = source.try_to_bytes().unwrap();
    let original_path = temp.path().join("original.lbox");
    std::fs::write(&original_path, &original).unwrap();
    let artifact = temp.path().join("source.migration");
    revault_migrate_archive_v3::export_archive(&source, &artifact, b"artifact password", [7; 16])
        .unwrap();
    let output = temp.path().join("clean.lbox");
    let wrong_owner = current::OwnerSigningKeyPair::generate().unwrap();
    assert!(import_archive(
        &artifact,
        b"artifact password",
        &temp.path().join("wrong.lbox"),
        &wrong_owner
    )
    .is_err());
    import_archive(&artifact, b"artifact password", &output, &owner_current).unwrap();
    verify_imported_archive(&artifact, b"artifact password", &output).unwrap();
    assert!(std::fs::metadata(&output).unwrap().len() < original.len() as u64 / 4);
    assert!(std::fs::read(&original_path).unwrap() == original);
    let opened =
        current::Lockbox::open(&output, current::LockboxOpen::Password(&password)).unwrap();
    assert_eq!(opened.format_version(), 4);
    assert!(opened.owner_signing_key_matches(&owner_current).unwrap());
    assert_eq!(
        opened
            .get_file(&current::LockboxPath::new("/keep.bin").unwrap())
            .unwrap(),
        live
    );
    assert!(opened.mirror_project("house").unwrap().unwrap().strict);
    opened
        .with_secret_variable(&current::VariableName::new("/token").unwrap(), |value| {
            value.with_bytes(|bytes| assert_eq!(bytes, vec![b's'; 16384]))
        })
        .unwrap()
        .unwrap()
        .unwrap();
    opened.inspector().verify_storage().unwrap();
    drop(opened);
    // A readable header is not enough to resume: changed logical content must
    // be rejected by the same complete verification used before installation.
    let mut changed = current::Lockbox::open_for_write(
        &output,
        current::LockboxOpen::Password(&password),
        &owner_current,
    )
    .unwrap();
    changed.set_description("tampered resumed output").unwrap();
    changed.commit().unwrap();
    drop(changed);
    assert!(verify_imported_archive(&artifact, b"artifact password", &output).is_err());
}
