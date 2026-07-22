use revault_lockbox_api::{
    ContactKeyPair, FormFieldDefinition, FormFieldKind, FormTypeId, ListOptions, Lockbox,
    LockboxOpen, LockboxPath, LockboxProtection, OwnerSigningKeyPair, SecretString,
};
use revault_lockbox_api_v1::{ContactKeyPair as V1ContactKeyPair, SecretString as V1SecretString};
use revault_migrate_archive_v1::export_archive_v1;
use revault_migrate_vault_v1::export_vault_v1;
use revault_migration::{
    export_vault_v2, import_archive, import_vault_v2, upgrade_archive_artifact,
    upgrade_vault_artifact, verify_archive_artifact, verify_vault_artifact,
};
use revault_vault_api::{VaultDirectory, CURRENT_VAULT_STRUCTURE_VERSION};
use revault_vault_api_v1::VaultDirectory as V1VaultDirectory;
use std::io::Read;

fn secret(value: &str) -> SecretString {
    SecretString::try_from_slice(value.as_bytes()).unwrap()
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
    let signing = OwnerSigningKeyPair::generate().unwrap();
    let mut source =
        Lockbox::create_in_memory(LockboxProtection::Password(&password), &signing).unwrap();
    let path = LockboxPath::new("/large.bin").unwrap();
    let mut input = PatternReader {
        remaining: 12 * 1024 * 1024 + 137,
        position: 0,
    };
    source
        .add_file_from_reader(&path, &mut input, false)
        .unwrap();
    let form_type = FormTypeId::new("12345678-1234-1234-1234-123456789abc").unwrap();
    source
        .define_form_with_type_id(
            form_type.clone(),
            "migration",
            "Migration form v1",
            vec![FormFieldDefinition {
                id: "username".into(),
                label: "Original label".into(),
                kind: FormFieldKind::Text,
                required: true,
            }],
        )
        .unwrap();
    let form_path = LockboxPath::new("/login.form").unwrap();
    source
        .create_form_record(&form_path, "migration", "Login")
        .unwrap();
    source
        .set_form_field_normal(&form_path, "username", "alice")
        .unwrap();
    source
        .revise_form_definition(
            &form_type,
            "Migration form v2",
            "new revision",
            vec![FormFieldDefinition {
                id: "username".into(),
                label: "Changed label".into(),
                kind: FormFieldKind::Text,
                required: true,
            }],
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
    assert_eq!(
        entries.iter().find(|entry| entry.path == path).unwrap().len,
        12 * 1024 * 1024 + 137
    );
    let mut reader = imported.open_file(&path).unwrap();
    let mut bytes = [0u8; 4096];
    let read = reader.read(&mut bytes).unwrap();
    assert_eq!(read, bytes.len());
    assert!(bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| *byte == (index % 251) as u8));
    let revisions = imported.list_form_definition_revisions(&form_type).unwrap();
    assert_eq!(revisions.len(), 2);
    let form = imported.get_form_record(&form_path).unwrap().unwrap();
    assert_eq!(form.definition_revision, 1);
    assert_eq!(form.values[0].captured_label, "Original label");
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
