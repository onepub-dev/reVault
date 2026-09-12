use revault_lockbox_api::{
    ContactKeyPair, FormTypeId, ListOptions, Lockbox, LockboxOpen, LockboxPath, LockboxProtection,
    OwnerSigningKeyPair, RecoveryScanner, SecretString, SecretVec, LOCKBOX_FORMAT_VERSION,
};
use revault_lockbox_api_export_v1::{
    ContactKeyPair as V1ArchiveContactKeyPair, FormFieldDefinition as V1FormFieldDefinition,
    FormFieldKind as V1FormFieldKind, FormTypeId as V1FormTypeId, Lockbox as V1Lockbox,
    LockboxOpen as V1LockboxOpen, LockboxPath as V1LockboxPath,
    LockboxProtection as V1LockboxProtection, OwnerSigningKeyPair as V1OwnerSigningKeyPair,
    SecretString as V1SecretString, SecretVec as V1SecretVec, VariableName as V1VariableName,
};
use revault_lockbox_api_v1::SecretString as V1VaultSecretString;
use revault_lockbox_api_vault_v1::ContactKeyPair as V1ContactKeyPair;
use revault_migrate_archive_v1::export_archive_v1;
use revault_migrate_vault_v1::export_vault_v1;
use revault_migration::{
    export_archive, export_vault_v2, import_archive, import_vault_v2, upgrade_archive_artifact,
    upgrade_vault_artifact, verify_archive_artifact, verify_vault_artifact,
};
use revault_migration_format::{
    ArchiveRecord, ArtifactKind, ArtifactWriter, MigrationHeader, MigrationRecord, SecretBytes,
};
use revault_vault_api::{VaultDirectory, CURRENT_VAULT_STRUCTURE_VERSION};
use revault_vault_api_v1::VaultDirectory as V1VaultDirectory;
use std::io::Read;

fn secret(value: &str) -> SecretString {
    SecretString::try_from_slice(value.as_bytes()).unwrap()
}

fn archive_artifact_writer() -> ArtifactWriter<Vec<u8>> {
    ArtifactWriter::new(
        Vec::new(),
        MigrationHeader {
            artifact_kind: ArtifactKind::Archive,
            source_native_version: 1,
            migration_schema_version: 2,
            target_native_version: Some(2),
            operation_id: [9; 16],
        },
        b"artifact password",
    )
    .unwrap()
}

fn archive_start() -> MigrationRecord {
    MigrationRecord::Archive(ArchiveRecord::Start {
        archive_id: [4; 16],
        format_version: 1,
        format_mode: None,
        content_key: SecretBytes::new(vec![5; 32]),
        key_directory: SecretBytes::new(vec![6; 32]),
        description: None,
    })
}

#[test]
fn archive_verifier_rejects_semantically_invalid_record_sequences() {
    let temp = tempfile::tempdir().unwrap();
    let cases = [
        "duplicate-start",
        "record-before-start",
        "missing-end",
        "wrong-chunk-offset",
    ];

    for case in cases {
        let mut writer = archive_artifact_writer();
        match case {
            "duplicate-start" => {
                writer.write_json(&archive_start()).unwrap();
                writer.write_json(&archive_start()).unwrap();
            }
            "record-before-start" => writer
                .write_json(&MigrationRecord::Archive(ArchiveRecord::Directory {
                    path: "/early".to_string(),
                    permissions: None,
                }))
                .unwrap(),
            "missing-end" => writer.write_json(&archive_start()).unwrap(),
            "wrong-chunk-offset" => {
                writer.write_json(&archive_start()).unwrap();
                writer
                    .write_json(&MigrationRecord::Archive(ArchiveRecord::FileStart {
                        file_id: 7,
                        path: "/file".to_string(),
                        size: 1,
                        permissions: None,
                    }))
                    .unwrap();
                let mut chunk = Vec::from(7u64.to_le_bytes());
                chunk.extend_from_slice(&1u64.to_le_bytes());
                chunk.push(42);
                writer.write_raw(&chunk).unwrap();
            }
            _ => unreachable!(),
        }
        let path = temp.path().join(format!("{case}.migration"));
        std::fs::write(&path, writer.finish().unwrap()).unwrap();
        assert!(
            verify_archive_artifact(&path, b"artifact password").is_err(),
            "{case} unexpectedly verified"
        );
    }
}

#[test]
fn checked_in_released_v1_archive_fixture_migrates_to_current_format() {
    const FIXTURE_KEY: &[u8] = b"lockbox fixture content key";
    const FIXTURE_HEX: &str = include_str!(
        "../../revault_lockbox_api/tests/fixtures/golden/v1/content_key_basic.lbox.hex"
    );
    let temp = tempfile::tempdir().unwrap();
    let artifact = temp.path().join("released-v1.migration");
    let upgraded = temp.path().join("released-v1-current.migration");
    let output = temp.path().join("released-v1-current.lbox");
    let fixture = V1Lockbox::open_bytes(
        decode_hex(FIXTURE_HEX),
        V1LockboxOpen::ContentKey(V1SecretVec::try_from_slice(FIXTURE_KEY).unwrap()),
    )
    .unwrap();

    export_archive_v1(&fixture, &artifact, b"artifact password", [8; 16]).unwrap();
    upgrade_archive_artifact(&artifact, &upgraded, b"artifact password").unwrap();
    import_archive(
        &upgraded,
        b"artifact password",
        &output,
        &OwnerSigningKeyPair::generate().unwrap(),
    )
    .unwrap();

    let migrated = Lockbox::open(
        &output,
        LockboxOpen::ContentKey(SecretVec::try_from_slice(FIXTURE_KEY).unwrap()),
    )
    .unwrap();
    assert_eq!(migrated.format_version(), LOCKBOX_FORMAT_VERSION);
    assert_eq!(
        migrated
            .get_file(&LockboxPath::new("/docs/readme.txt").unwrap())
            .unwrap(),
        b"golden fixture readme\n"
    );
    assert_eq!(
        migrated
            .get_variable(&revault_lockbox_api::VariableName::new("FEATURE_FLAG").unwrap())
            .unwrap()
            .as_deref(),
        Some("enabled")
    );
}

fn decode_hex(value: &str) -> Vec<u8> {
    let digits = value
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    assert_eq!(digits.len() % 2, 0);
    digits
        .chunks_exact(2)
        .map(|pair| (hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]))
        .collect()
}

fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        b'A'..=b'F' => value - b'A' + 10,
        _ => panic!("invalid fixture hex"),
    }
}

#[test]
fn current_vault_all_record_families_round_trip() {
    let temp = tempfile::tempdir().unwrap();
    let source_root = temp.path().join("source-vault");
    let output_root = temp.path().join("imported-vault");
    let artifact = temp.path().join("vault.migration");
    let password = secret("migration vault password");
    let artifact_password = secret("artifact password");
    let source = VaultDirectory::replace(&source_root, &password).unwrap();
    source
        .store_private_key("default", &ContactKeyPair::generate().unwrap())
        .unwrap();
    source
        .store_profile_email("default", "owner@example.test")
        .unwrap();
    source.seed_default_form_definitions().unwrap();
    source.create_password_profile("production-server").unwrap();
    let profile_password = source.load_profile_password("production-server").unwrap();
    let contact = ContactKeyPair::generate().unwrap();
    let contact_signer = OwnerSigningKeyPair::generate().unwrap();
    source
        .store_contact("colleague", &contact.public_key())
        .unwrap();
    source
        .store_contact_signing_key("colleague", &contact_signer.public_key())
        .unwrap();
    let id = revault_lockbox_api::LockboxId::new_random().unwrap();
    source
        .remember_known_lockbox(id, temp.path().join("known.lbox"))
        .unwrap();
    source
        .remember_access_slot_label(id, 7, "colleague slot")
        .unwrap();
    source
        .remember_lockbox_password(id, &secret("remembered password"))
        .unwrap();
    source
        .store_key_directory_backup(id, b"opaque key directory backup")
        .unwrap();
    let known = source.list_known_lockboxes().unwrap();
    let labels = source.list_access_slot_labels(id).unwrap();

    export_vault_v2(&source, &artifact, &artifact_password, [1; 16]).unwrap();
    assert!(verify_vault_artifact(&artifact, &artifact_password).unwrap() > 2);
    drop(source);

    import_vault_v2(&artifact, &artifact_password, &output_root, &password).unwrap();
    let imported = VaultDirectory::open_or_create(&output_root, &password).unwrap();
    assert_eq!(
        imported.load_contact("colleague").unwrap().to_bytes(),
        contact.public_key().to_bytes()
    );
    assert_eq!(
        imported
            .load_contact_signing_key("colleague")
            .unwrap()
            .to_bytes(),
        contact_signer.public_key().to_bytes()
    );
    assert_eq!(imported.list_known_lockboxes().unwrap(), known);
    assert_eq!(imported.list_access_slot_labels(id).unwrap(), labels);
    imported
        .remembered_lockbox_password(id)
        .unwrap()
        .unwrap()
        .with_str(|value| assert_eq!(value, "remembered password"))
        .unwrap();
    assert_eq!(
        imported.load_key_directory_backup(id).unwrap(),
        b"opaque key directory backup"
    );

    assert_eq!(
        imported.structure_version().unwrap(),
        CURRENT_VAULT_STRUCTURE_VERSION
    );
    assert_eq!(
        imported.profile_email("default").unwrap().as_deref(),
        Some("owner@example.test")
    );
    assert!(!imported.list_form_definitions().unwrap().is_empty());
    assert_eq!(
        profile_password,
        imported.load_profile_password("production-server").unwrap()
    );
}

#[test]
fn vault_v1_fixture_exports_upgrades_and_imports_as_current_format() {
    let temp = tempfile::tempdir().unwrap();
    let source_root = temp.path().join("vault-v1");
    let source_path = source_root.join("local-vault.lbox");
    let exported = temp.path().join("vault-v1.migration");
    let upgraded = temp.path().join("vault-current.migration");
    let imported_root = temp.path().join("vault-current");
    let password = secret("v1 vault password");
    let v1_password = V1VaultSecretString::try_from_slice(b"v1 vault password").unwrap();
    let fixture = V1VaultDirectory::replace(&source_root, &v1_password).unwrap();
    fixture
        .store_private_key("default", &V1ContactKeyPair::generate().unwrap())
        .unwrap();
    fixture
        .store_identity_email("default", "v1@example.test")
        .unwrap();
    fixture.rotate_private_key("default").unwrap();
    fixture.seed_default_form_definitions().unwrap();
    drop(fixture);
    let source_before = std::fs::read(&source_path).unwrap();

    password
        .with_bytes(|password| {
            export_vault_v1(
                &source_root,
                password,
                &exported,
                b"artifact password",
                [7; 16],
            )
        })
        .unwrap()
        .unwrap();
    assert_eq!(std::fs::read(&source_path).unwrap(), source_before);
    upgrade_vault_artifact(&exported, &upgraded, b"artifact password").unwrap();
    import_vault_v2(&upgraded, b"artifact password", &imported_root, &password).unwrap();
    let imported = VaultDirectory::open_or_create(&imported_root, &password).unwrap();
    assert_eq!(
        imported.structure_version().unwrap(),
        CURRENT_VAULT_STRUCTURE_VERSION
    );
    assert_eq!(imported.list_private_keys().unwrap(), vec!["default"]);
    assert_eq!(
        imported.profile_email("default").unwrap().as_deref(),
        Some("v1@example.test")
    );
    assert_eq!(
        imported
            .list_profile_generations("default")
            .unwrap()
            .generations
            .len(),
        2
    );
}

#[test]
fn v1_archive_all_records_migrate_to_current_with_existing_access() {
    let temp = tempfile::tempdir().unwrap();
    let artifact = temp.path().join("archive.migration");
    let upgraded = temp.path().join("archive.latest.migration");
    let output = temp.path().join("imported.lbox");
    let password = secret("archive password");
    let v1_password = V1SecretString::try_from_slice(b"archive password").unwrap();
    let signing = V1OwnerSigningKeyPair::generate().unwrap();
    let mut source =
        V1Lockbox::create_in_memory(V1LockboxProtection::Password(&v1_password), &signing).unwrap();
    source.set_description("v1 migration fixture").unwrap();
    source
        .create_dir(&V1LockboxPath::new("/mirrors").unwrap(), true)
        .unwrap();
    // Pinned historical public API creates a real v1 mirror record.
    for (name, policy) in [
        (
            "remove",
            revault_lockbox_api_export_v1::MirrorMissingFilePolicy::Remove,
        ),
        (
            "retain",
            revault_lockbox_api_export_v1::MirrorMissingFilePolicy::Retain,
        ),
    ] {
        source
            .create_mirror_project(
                revault_lockbox_api_export_v1::MirrorProject {
                    name: name.into(),
                    source: "/srv/project".into(),
                    destination: V1LockboxPath::new(format!("/mirrors/{name}")).unwrap(),
                    includes: vec!["**/*.txt".into()],
                    excludes: vec!["private/**".into()],
                    missing_file_policy: policy,
                    host_identity: Some("fixture-host-identity".into()),
                },
                false,
            )
            .unwrap();
        source
            .with_mirror_project_mutation(name, |archive, project| {
                archive.create_dir(&project.destination, true)?;
                archive.add_file(
                    &V1LockboxPath::new(format!("{}/kept.txt", project.destination))?,
                    b"mirror bytes",
                    false,
                )
            })
            .unwrap();
    }

    let additional_contact = V1ArchiveContactKeyPair::generate().unwrap();
    source
        .add_contact(&additional_contact.public_key())
        .unwrap();
    let path = V1LockboxPath::new("/large.bin").unwrap();
    let mut input = PatternReader {
        remaining: 12 * 1024 * 1024 + 137,
        position: 0,
    };
    source
        .add_file_from_reader(&path, &mut input, false)
        .unwrap();
    let empty_dir = V1LockboxPath::new("/empty/private").unwrap();
    source.create_dir(&empty_dir, true).unwrap();
    source.set_permissions(&empty_dir, 0o700).unwrap();
    let empty_file = V1LockboxPath::new("/empty/zero.txt").unwrap();
    source
        .add_file_with_permissions(&empty_file, b"", 0o640, false)
        .unwrap();
    let symlink = V1LockboxPath::new("/latest.bin").unwrap();
    source.add_symlink(&symlink, &path, false).unwrap();
    source.set_permissions(&symlink, 0o700).unwrap();
    let normal_variable = V1VariableName::new("/deploy/REGION").unwrap();
    let secret_variable = V1VariableName::new("/deploy/TOKEN").unwrap();
    source
        .set_variable(&normal_variable, "ap-southeast-2")
        .unwrap();
    let v1_secret = V1SecretString::try_from_slice(b"migration-secret").unwrap();
    source
        .set_secret_variable(&secret_variable, &v1_secret)
        .unwrap();
    let typed_fields = [
        ("url", V1FormFieldKind::Url, "https://example.test/path"),
        ("email", V1FormFieldKind::Email, "owner@example.test"),
        ("date", V1FormFieldKind::Date, "2026-09-13"),
        ("month", V1FormFieldKind::Month, "2026-09"),
        ("notes", V1FormFieldKind::Notes, "line one\nline two"),
        ("number", V1FormFieldKind::Number, "123.5"),
    ];
    for (name, kind, value) in &typed_fields {
        source
            .define_form(
                name,
                name,
                vec![V1FormFieldDefinition {
                    id: "value".into(),
                    label: name.to_string(),
                    kind: *kind,
                    required: true,
                }],
            )
            .unwrap();
        let path = V1LockboxPath::new(format!("/{name}.form")).unwrap();
        source.create_form_record(&path, name, name).unwrap();
        source.set_form_field_normal(&path, "value", value).unwrap();
    }
    let form_type = V1FormTypeId::new("12345678-1234-1234-1234-123456789abc").unwrap();
    source
        .define_form_with_type_id(
            form_type.clone(),
            "migration",
            "Migration form v1",
            vec![
                V1FormFieldDefinition {
                    id: "username".into(),
                    label: "Original label".into(),
                    kind: V1FormFieldKind::Text,
                    required: true,
                },
                V1FormFieldDefinition {
                    id: "password".into(),
                    label: "Password".into(),
                    kind: V1FormFieldKind::Secret,
                    required: true,
                },
            ],
        )
        .unwrap();
    let form_path = V1LockboxPath::new("/login.form").unwrap();
    source
        .create_form_record(&form_path, "migration", "Login")
        .unwrap();
    source
        .set_form_field_normal(&form_path, "username", "alice")
        .unwrap();
    source
        .set_form_field_secret(&form_path, "password", &v1_secret)
        .unwrap();
    source
        .revise_form_definition(
            &form_type,
            "Migration form v2",
            "new revision",
            vec![
                V1FormFieldDefinition {
                    id: "username".into(),
                    label: "Changed label".into(),
                    kind: V1FormFieldKind::Text,
                    required: true,
                },
                V1FormFieldDefinition {
                    id: "password".into(),
                    label: "Password".into(),
                    kind: V1FormFieldKind::Secret,
                    required: true,
                },
            ],
        )
        .unwrap();
    source.commit().unwrap();
    let source_owner = source.owner_inspection().unwrap().fingerprint.unwrap();

    export_archive_v1(&source, &artifact, b"artifact password", [2; 16]).unwrap();
    assert!(verify_archive_artifact(&artifact, b"artifact password").unwrap() > 4);
    upgrade_archive_artifact(&artifact, &upgraded, b"artifact password").unwrap();
    assert!(verify_archive_artifact(&upgraded, b"artifact password").unwrap() > 4);
    let migrated_signing = OwnerSigningKeyPair::generate().unwrap();
    import_archive(&upgraded, b"artifact password", &output, &migrated_signing).unwrap();

    let imported = Lockbox::open(&output, LockboxOpen::Password(&password)).unwrap();
    assert_eq!(imported.format_version(), LOCKBOX_FORMAT_VERSION);
    assert_eq!(
        imported.description().unwrap().as_deref(),
        Some("v1 migration fixture")
    );
    assert_eq!(imported.list_mirror_projects().unwrap().len(), 2);
    for (name, policy) in [
        (
            "remove",
            revault_lockbox_api::MirrorMissingFilePolicy::Remove,
        ),
        (
            "retain",
            revault_lockbox_api::MirrorMissingFilePolicy::Retain,
        ),
    ] {
        let project = imported.mirror_project(name).unwrap().unwrap();
        assert_eq!(
            project,
            revault_lockbox_api::MirrorProject {
                name: name.into(),
                source: "/srv/project".into(),
                destination: LockboxPath::new(format!("/mirrors/{name}")).unwrap(),
                includes: vec!["**/*.txt".into()],
                excludes: vec!["private/**".into()],
                missing_file_policy: policy,
                strict: false,
                host_identity: Some("fixture-host-identity".into()),
            }
        );
        assert_eq!(
            imported
                .get_file(&LockboxPath::new(format!("/mirrors/{name}/kept.txt")).unwrap())
                .unwrap(),
            b"mirror bytes"
        );
    }
    let additional_contact_record = additional_contact.private_key_record().unwrap();
    let additional_contact_record = additional_contact_record
        .with_bytes(|bytes| bytes.to_vec())
        .unwrap();
    assert!(Lockbox::open(
        &output,
        LockboxOpen::ContactKeyPair(
            revault_lockbox_api::ContactKeyPair::from_private_key_record(
                revault_lockbox_api::SecretVec::try_from_vec(additional_contact_record).unwrap()
            )
            .unwrap()
        )
    )
    .is_ok());
    assert_eq!(
        imported.lockbox_id().as_bytes(),
        source.lockbox_id().as_bytes()
    );
    let imported_owner = imported.owner_inspection().unwrap().fingerprint.unwrap();
    assert_ne!(source_owner, imported_owner);
    let root = LockboxPath::new("/").unwrap();
    let mut options = ListOptions::new(&root);
    options.recursive = true;
    let entries = imported
        .list(options)
        .unwrap()
        .collect::<revault_lockbox_api::Result<Vec<_>>>()
        .unwrap();
    let imported_path = LockboxPath::new(path.as_str()).unwrap();
    assert_eq!(
        entries
            .iter()
            .find(|entry| entry.path == imported_path)
            .unwrap()
            .len,
        12 * 1024 * 1024 + 137
    );
    let mut reader = imported.open_file(&imported_path).unwrap();
    let mut bytes = [0u8; 4096];
    let mut position = 0;
    loop {
        let read = reader.read(&mut bytes).unwrap();
        if read == 0 {
            break;
        }
        assert!(bytes[..read]
            .iter()
            .enumerate()
            .all(|(index, byte)| *byte == ((position + index) % 251) as u8));
        position += read;
    }
    assert_eq!(position, 12 * 1024 * 1024 + 137);
    for (name, kind, expected) in &typed_fields {
        let record = imported
            .get_form_record(&LockboxPath::new(format!("/{name}.form")).unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(record.values.len(), 1);
        assert_eq!(format!("{:?}", record.values[0].kind), format!("{kind:?}"));
        assert!(
            matches!(&record.values[0].value, revault_lockbox_api::FormValue::Normal(value) if value == expected)
        );
    }
    let imported_form_type = FormTypeId::new(form_type.as_str()).unwrap();
    let revisions = imported
        .list_form_definition_revisions(&imported_form_type)
        .unwrap();
    assert_eq!(revisions.len(), 2);
    let original_revisions = source.list_form_definition_revisions(&form_type).unwrap();
    for (expected, actual) in original_revisions.iter().zip(&revisions) {
        assert_eq!(actual.type_id.as_str(), expected.type_id.as_str());
        assert_eq!(actual.alias, expected.alias);
        assert_eq!(actual.revision, expected.revision);
        assert_eq!(actual.name, expected.name);
        assert_eq!(actual.description, expected.description);
        assert_eq!(actual.fields.len(), expected.fields.len());
        for (expected, actual) in expected.fields.iter().zip(&actual.fields) {
            assert_eq!(actual.id, expected.id);
            assert_eq!(actual.label, expected.label);
            assert_eq!(actual.required, expected.required);
            assert_eq!(format!("{:?}", actual.kind), format!("{:?}", expected.kind));
        }
    }

    let imported_form_path = LockboxPath::new(form_path.as_str()).unwrap();
    let form = imported
        .get_form_record(&imported_form_path)
        .unwrap()
        .unwrap();
    assert_eq!(form.definition_revision, 1);
    assert_eq!(form.values[0].captured_label, "Original label");
    let username_value = form
        .values
        .iter()
        .find(|value| value.field_id == "username")
        .expect("migrated normal form field");
    assert!(!username_value.value.is_secret());
    assert!(
        matches!(&username_value.value, revault_lockbox_api::FormValue::Normal(value) if value == "alice")
    );
    let password_value = form
        .values
        .iter()
        .find(|value| value.field_id == "password")
        .expect("migrated secret form field");
    assert!(password_value.value.is_secret());
    match &password_value.value {
        revault_lockbox_api::FormValue::Secret(value) => value
            .with_str(|value| assert_eq!(value, "migration-secret"))
            .unwrap(),
        revault_lockbox_api::FormValue::Normal(_) => panic!("secret form field was downgraded"),
    }
    let imported_empty_dir = LockboxPath::new(empty_dir.as_str()).unwrap();
    let imported_empty_file = LockboxPath::new(empty_file.as_str()).unwrap();
    let imported_symlink = LockboxPath::new(symlink.as_str()).unwrap();
    assert_eq!(imported.permissions(&imported_empty_dir), Some(0o700));
    assert_eq!(imported.permissions(&imported_empty_file), Some(0o640));
    assert_eq!(imported.get_file(&imported_empty_file).unwrap(), b"");
    assert_eq!(
        imported.get_symlink_target(&imported_symlink).unwrap(),
        imported_path
    );
    assert_eq!(imported.permissions(&imported_symlink), Some(0o700));
    assert_eq!(
        imported
            .get_variable(
                &revault_lockbox_api::VariableName::new(normal_variable.as_str()).unwrap()
            )
            .unwrap()
            .as_deref(),
        Some("ap-southeast-2")
    );
    imported
        .with_secret_variable(
            &revault_lockbox_api::VariableName::new(secret_variable.as_str()).unwrap(),
            |value| {
                value
                    .with_str(|value| assert_eq!(value, "migration-secret"))
                    .unwrap();
            },
        )
        .unwrap()
        .unwrap();
    drop(reader);
    drop(imported);
    let mut writable = Lockbox::open_with_signer(&output, LockboxOpen::Password(&password), |_| {
        migrated_signing.try_clone()
    })
    .unwrap();
    let managed = LockboxPath::new("/mirrors/remove/kept.txt").unwrap();
    assert!(writable.add_file(&managed, b"unauthorized", true).is_err());
    writable
        .with_mirror_project_mutation("remove", |archive, _| {
            archive.add_file(&managed, b"updated after migration", true)
        })
        .unwrap();
    writable.commit().unwrap();
    drop(writable);
    let reopened = Lockbox::open(&output, LockboxOpen::Password(&password)).unwrap();
    assert_eq!(
        reopened.get_file(&managed).unwrap(),
        b"updated after migration"
    );
    assert_eq!(reopened.list_mirror_projects().unwrap().len(), 2);
}

#[test]
fn current_archive_preserves_metadata_and_supports_recovery() {
    let temp = tempfile::tempdir().unwrap();
    let artifact = temp.path().join("archive.migration");
    let output = temp.path().join("migrated.lbox");
    let key = b"migration-content-key";
    let signing = OwnerSigningKeyPair::generate().unwrap();
    let mut source = Lockbox::create_in_memory(
        LockboxProtection::ContentKey(SecretVec::try_from_slice(key).unwrap()),
        &signing,
    )
    .unwrap();
    source.set_description("migration description").unwrap();
    let empty = LockboxPath::new("/empty/private").unwrap();
    source.create_dir(&empty, true).unwrap();
    source.set_permissions(&empty, 0o700).unwrap();
    let target = LockboxPath::new("/data.txt").unwrap();
    source.add_file(&target, b"recover me", false).unwrap();
    let link = LockboxPath::new("/current").unwrap();
    source.add_symlink(&link, &target, false).unwrap();
    source.set_permissions(&link, 0o700).unwrap();
    source.commit().unwrap();

    export_archive(&source, &artifact, b"artifact password", [9; 16]).unwrap();
    import_archive(&artifact, b"artifact password", &output, &signing).unwrap();
    let migrated = Lockbox::open(
        &output,
        LockboxOpen::ContentKey(SecretVec::try_from_slice(key).unwrap()),
    )
    .unwrap();
    assert_eq!(
        migrated.description().unwrap().as_deref(),
        Some("migration description")
    );
    assert_eq!(migrated.permissions(&empty), Some(0o700));
    assert_eq!(migrated.permissions(&link), Some(0o700));

    let mut damaged = std::fs::read(&output).unwrap();
    damaged[0] ^= 0xff;
    damaged[160] ^= 0xff;
    let report = RecoveryScanner::scan_bytes(damaged.clone(), key);
    assert!(report
        .intact_files
        .iter()
        .any(|entry| entry.path == "/data.txt"));
    let recovered = RecoveryScanner::salvage_bytes(damaged, key, &signing).unwrap();
    assert_eq!(recovered.get_file(&target).unwrap(), b"recover me");
}

struct PatternReader {
    remaining: usize,
    position: usize,
}

#[test]
fn archive_migration_preserves_independent_format_choices() {
    use revault_lockbox_api::{
        Compression, Encryption, LockboxCreateOptions, LockboxOpen, Signing, ZstdLevel,
    };
    let temp = tempfile::tempdir().unwrap();
    let signer = OwnerSigningKeyPair::generate().unwrap();
    for signed in [false, true] {
        for compression in [
            Compression::None,
            Compression::Zstd {
                level: ZstdLevel::new(9).unwrap(),
            },
        ] {
            let mut source = Lockbox::create_in_memory_with_options(LockboxCreateOptions {
                compression,
                ..LockboxCreateOptions::new(
                    Encryption::None,
                    if signed {
                        Signing::Owner(&signer)
                    } else {
                        Signing::None
                    },
                )
            })
            .unwrap();
            let path = LockboxPath::new("/notes.txt").unwrap();
            source
                .add_file(&path, b"migrated plaintext content", false)
                .unwrap();
            source.commit().unwrap();
            let artifact = temp
                .path()
                .join(format!("{signed}-{compression:?}.migration"));
            let destination = temp.path().join(format!("{signed}-{compression:?}.lbox"));
            export_archive(&source, &artifact, b"artifact password".as_slice(), [7; 16]).unwrap();
            import_archive(
                &artifact,
                b"artifact password".as_slice(),
                &destination,
                &signer,
            )
            .unwrap();
            let imported = Lockbox::open(&destination, LockboxOpen::Unencrypted).unwrap();
            assert_eq!(imported.format_version(), 3);
            assert_eq!(imported.format_options(), source.format_options());
            assert_eq!(
                imported.get_file(&path).unwrap(),
                b"migrated plaintext content"
            );
        }
    }
}

#[test]
fn historical_v2_archive_migrates_to_current_format() {
    let temp = tempfile::tempdir().unwrap();
    let signer = OwnerSigningKeyPair::generate().unwrap();
    let key = b"legacy v2 migration test key";
    // Only a historical writer can create this fixture; never relabel a current header.
    let old_signer = revault_lockbox_api_v2::OwnerSigningKeyPair::generate().unwrap();
    let mut old = revault_lockbox_api_v2::Lockbox::create_in_memory(
        revault_lockbox_api_v2::LockboxProtection::ContentKey(
            revault_lockbox_api_v2::SecretVec::try_from_slice(key).unwrap(),
        ),
        &old_signer,
    )
    .unwrap();
    old.add_file(
        &revault_lockbox_api_v2::LockboxPath::new("/legacy.txt").unwrap(),
        b"legacy data",
        false,
    )
    .unwrap();
    old.commit().unwrap();
    let source = Lockbox::open_bytes(
        old.try_to_bytes().unwrap(),
        LockboxOpen::ContentKey(SecretVec::try_from_slice(key).unwrap()),
    )
    .unwrap();
    assert_eq!(source.format_version(), 2);
    let path = LockboxPath::new("/legacy.txt").unwrap();
    let artifact = temp.path().join("legacy.migration");
    let destination = temp.path().join("upgraded.lbox");
    export_archive(&source, &artifact, b"artifact password".as_slice(), [8; 16]).unwrap();
    import_archive(
        &artifact,
        b"artifact password".as_slice(),
        &destination,
        &signer,
    )
    .unwrap();
    let upgraded = Lockbox::open(
        &destination,
        LockboxOpen::ContentKey(SecretVec::try_from_slice(key).unwrap()),
    )
    .unwrap();
    assert_eq!(upgraded.format_version(), 3);
    assert_eq!(upgraded.format_options(), source.format_options());
    assert_eq!(upgraded.get_file(&path).unwrap(), b"legacy data");
}

impl Read for PatternReader {
    fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        let count = output.len().min(self.remaining);
        for (offset, byte) in output[..count].iter_mut().enumerate() {
            *byte = ((self.position + offset) % 251) as u8;
        }
        self.position += count;
        self.remaining -= count;
        Ok(count)
    }
}
