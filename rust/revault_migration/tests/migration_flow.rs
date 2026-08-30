use revault_lockbox_api::{
    ContactKeyPair, FormTypeId, ListOptions, Lockbox, LockboxOpen, LockboxPath, LockboxProtection,
    OwnerSigningKeyPair, RecoveryScanner, SecretString, SecretVec,
};
use revault_lockbox_api_v1::{
    ContactKeyPair as V1ArchiveContactKeyPair, FormFieldDefinition as V1FormFieldDefinition,
    FormFieldKind as V1FormFieldKind, FormTypeId as V1FormTypeId, Lockbox as V1Lockbox,
    LockboxOpen as V1LockboxOpen, LockboxPath as V1LockboxPath,
    LockboxProtection as V1LockboxProtection, OwnerSigningKeyPair as V1OwnerSigningKeyPair,
    SecretString as V1SecretString, SecretVec as V1SecretVec, VariableName as V1VariableName,
};
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
fn checked_in_released_v1_archive_fixture_migrates_to_v2() {
    const FIXTURE_KEY: &[u8] = b"lockbox fixture content key";
    const FIXTURE_HEX: &str = include_str!(
        "../../revault_lockbox_api/tests/fixtures/golden/v1/content_key_basic.lbox.hex"
    );
    let temp = tempfile::tempdir().unwrap();
    let artifact = temp.path().join("released-v1.migration");
    let upgraded = temp.path().join("released-v1-v2.migration");
    let output = temp.path().join("released-v1-v2.lbox");
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
fn vault_v2_export_verify_import_round_trip() {
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

    export_vault_v2(&source, &artifact, &artifact_password, [1; 16]).unwrap();
    assert!(verify_vault_artifact(&artifact, &artifact_password).unwrap() > 2);
    drop(source);

    import_vault_v2(&artifact, &artifact_password, &output_root, &password).unwrap();
    let imported = VaultDirectory::open_or_create(&output_root, &password).unwrap();
    assert_eq!(
        imported.structure_version().unwrap(),
        CURRENT_VAULT_STRUCTURE_VERSION
    );
    assert_eq!(
        imported.profile_email("default").unwrap().as_deref(),
        Some("owner@example.test")
    );
    assert!(!imported.list_form_definitions().unwrap().is_empty());
}

#[test]
fn vault_v1_fixture_exports_upgrades_and_imports_as_v2() {
    let temp = tempfile::tempdir().unwrap();
    let source_root = temp.path().join("vault-v1");
    let source_path = source_root.join("local-vault.lbox");
    let exported = temp.path().join("vault-v1.migration");
    let upgraded = temp.path().join("vault-v2.migration");
    let imported_root = temp.path().join("vault-v2");
    let password = secret("v1 vault password");
    let v1_password = V1SecretString::try_from_slice(b"v1 vault password").unwrap();
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
fn archive_files_are_streamed_and_new_commit_opens_with_existing_access() {
    let temp = tempfile::tempdir().unwrap();
    let artifact = temp.path().join("archive.migration");
    let upgraded = temp.path().join("archive.latest.migration");
    let output = temp.path().join("imported.lbox");
    let password = secret("archive password");
    let v1_password = V1SecretString::try_from_slice(b"archive password").unwrap();
    let signing = V1OwnerSigningKeyPair::generate().unwrap();
    let mut source =
        V1Lockbox::create_in_memory(V1LockboxProtection::Password(&v1_password), &signing).unwrap();
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
    let read = reader.read(&mut bytes).unwrap();
    assert_eq!(read, bytes.len());
    assert!(bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| *byte == (index % 251) as u8));
    let imported_form_type = FormTypeId::new(form_type.as_str()).unwrap();
    let revisions = imported
        .list_form_definition_revisions(&imported_form_type)
        .unwrap();
    assert_eq!(revisions.len(), 2);
    let imported_form_path = LockboxPath::new(form_path.as_str()).unwrap();
    let form = imported
        .get_form_record(&imported_form_path)
        .unwrap()
        .unwrap();
    assert_eq!(form.definition_revision, 1);
    assert_eq!(form.values[0].captured_label, "Original label");
    assert!(form.values.iter().any(|value| value.field_id == "password"));
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
}

#[test]
fn migrated_v2_archive_preserves_metadata_and_supports_recovery() {
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
