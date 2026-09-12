//! Explicit append-only fixture creation. Never runs in normal CI.
use super::*;

fn write_new(path: &Path, bytes: &[u8]) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}

fn retain(kind: &str, version: u32, writer: &str, bytes: &[u8], coverage: &[&str]) {
    let container = revault_lockbox_api::probe_lockbox_format_version(bytes).unwrap();
    let name = if kind == "vault" && version == 2 && container == 1 {
        "vault-v2-container-v1".to_string()
    } else {
        format!("{kind}-v{version}")
    };
    let dir = root().join(name);
    fs::create_dir(&dir).expect("refusing to overwrite retained fixture directory");
    let manifest = Manifest {
        kind: kind.into(),
        native_version: version,
        container_version: revault_lockbox_api::probe_lockbox_format_version(bytes).unwrap(),
        writer: writer.into(),
        migrated_archive_mode: if kind == "archive" {
            // Legacy mode zero upgrades to signed/encrypted Zstd level 1 (mode 8).
            Some(if version <= 2 {
                8
            } else {
                Lockbox::open_bytes(bytes.to_vec(), LockboxOpen::Password(&password()))
                    .unwrap()
                    .export_migration_format_mode()
            })
        } else {
            None
        },
        sha256: digest(bytes),
        password: String::from_utf8(PASSWORD.to_vec()).unwrap(),
        coverage: coverage.iter().map(|s| s.to_string()).collect(),
    };
    if kind == "archive" {
        assert_eq!(u32::from(manifest.container_version), version);
    }
    write_new(&dir.join("native.bin"), bytes);
    write_new(
        &dir.join("manifest.json"),
        &serde_json::to_vec_pretty(&manifest).unwrap(),
    );
    let temp = tempfile::tempdir().unwrap();
    let exported = temp.path().join("export.migration");
    export_fixture(&dir, &manifest, &exported);
    let upgraded = temp.path().join("upgraded.migration");
    if kind == "archive" {
        upgrade_archive_artifact(&exported, &upgraded, ARTIFACT_PASSWORD).unwrap();
    } else {
        upgrade_vault_artifact(&exported, &upgraded, ARTIFACT_PASSWORD).unwrap();
    }
    write_new(
        &dir.join("expected.json"),
        &serde_json::to_vec_pretty(&snapshot(&upgraded)).unwrap(),
    );
}

macro_rules! archive_generator {
    ($function:ident, $api:ident, $version:expr, $writer:expr $(, $strict:expr)?) => {
        fn $function() {
            use $api as api;
            let password = api::SecretString::try_from_slice(PASSWORD).unwrap();
            let signer = api::OwnerSigningKeyPair::generate().unwrap();
            let mut archive = api::Lockbox::create_in_memory(api::LockboxProtection::Password(&password), &signer).unwrap();
            let contact = api::ContactKeyPair::generate().unwrap();
            archive.add_contact(&contact.public_key()).unwrap();
            archive.commit().unwrap();
            archive.set_description("Permanent complete migration fixture: Unicode café").unwrap();
            archive.create_dir(&api::LockboxPath::new("/empty/private").unwrap(), true).unwrap();
            archive.set_permissions(&api::LockboxPath::new("/empty/private").unwrap(), 0o700).unwrap();
            archive.add_file_with_permissions(&api::LockboxPath::new("/empty/zero.txt").unwrap(), b"", 0o640, false).unwrap();
            archive.add_file_with_permissions(&api::LockboxPath::new("/unicode-λ.txt").unwrap(), "Unicode café\n".as_bytes(), 0o600, false).unwrap();
            let binary: Vec<u8> = (0..5*1024*1024+137).map(|i| (i%251) as u8).collect();
            archive.add_file(&api::LockboxPath::new("/binary.bin").unwrap(), &binary, false).unwrap();
            archive.add_symlink(&api::LockboxPath::new("/link").unwrap(), &api::LockboxPath::new("/binary.bin").unwrap(), false).unwrap();
            archive.set_permissions(&api::LockboxPath::new("/link").unwrap(), 0o700).unwrap();
            archive.set_variable(&api::VariableName::new("/env/NORMAL").unwrap(), "value\nλ").unwrap();
            archive.set_variable(&api::VariableName::new("/env/EMPTY").unwrap(), "").unwrap();
            archive.set_secret_variable(&api::VariableName::new("/env/SECRET").unwrap(), &api::SecretString::try_from_slice(b"synthetic secret").unwrap()).unwrap();
            let type_id = api::FormTypeId::new("12345678-1234-1234-1234-123456789abc").unwrap();
            let fields = [
                ("text", api::FormFieldKind::Text, "alice"),
                ("secret", api::FormFieldKind::Secret, "synthetic secret"),
                ("url", api::FormFieldKind::Url, "https://example.test"),
                ("email", api::FormFieldKind::Email, "owner@example.test"),
                ("date", api::FormFieldKind::Date, "2026-09-13"),
                ("month", api::FormFieldKind::Month, "2026-09"),
                ("notes", api::FormFieldKind::Notes, "one\ntwo"),
                ("number", api::FormFieldKind::Number, "123.5"),
            ];
            let definitions: Vec<_> = fields.iter().map(|(id, kind, _)| api::FormFieldDefinition { id: id.to_string(), label: format!("Original {id}"), kind: *kind, required: true }).collect();
            archive.define_form_with_type_id(type_id.clone(), "complete", "All field types", definitions.clone()).unwrap();
            let form_path = api::LockboxPath::new("/original.form").unwrap();
            archive.create_form_record(&form_path, "complete", "Original form").unwrap();
            for (id, _, value) in fields {
                if id == "secret" {
                    archive.set_form_field_secret(&form_path, id, &api::SecretString::try_from_slice(value.as_bytes()).unwrap()).unwrap();
                } else { archive.set_form_field_normal(&form_path, id, value).unwrap(); }
            }
            let mut revised = definitions;
            revised[0].label = "Revised label".into();
            archive.revise_form_definition(&type_id, "Revision two", "Preserve revision history", revised).unwrap();
            archive.create_form_record(&api::LockboxPath::new("/latest.form").unwrap(), "complete", "Empty latest form").unwrap();
            archive.create_dir(&api::LockboxPath::new("/mirrors").unwrap(), true).unwrap();
            for (name, policy) in [("remove", api::MirrorMissingFilePolicy::Remove), ("retain", api::MirrorMissingFilePolicy::Retain)] {
                archive.create_mirror_project(api::MirrorProject {
                    name: name.into(), source: "/srv/fixture".into(), destination: api::LockboxPath::new(format!("/mirrors/{name}")).unwrap(),
                    includes: vec!["**/*.txt".into()], excludes: vec!["private/**".into()], missing_file_policy: policy,
                    $(strict: $strict,)? host_identity: Some("retained-fixture-identity".into()),
                }, false).unwrap();
                archive.with_mirror_project_mutation(name, |archive, project| {
                    archive.create_dir(&project.destination, true)?;
                    archive.add_file(&api::LockboxPath::new(format!("{}/file.txt", project.destination))?, b"original mirrored file", false)
                }).unwrap();
            }
            archive.commit().unwrap();
            // A committed replacement and deletion ensure the fixture has a real history.
            archive.add_file(&api::LockboxPath::new("/replace.txt").unwrap(), b"old", false).unwrap();
            archive.add_file(&api::LockboxPath::new("/deleted.txt").unwrap(), b"must not migrate", false).unwrap();
            archive.commit().unwrap();
            archive.add_file(&api::LockboxPath::new("/replace.txt").unwrap(), b"new", true).unwrap();
            archive.delete(&api::LockboxPath::new("/deleted.txt").unwrap()).unwrap();
            archive.commit().unwrap();
            retain("archive", $version, $writer, &archive.try_to_bytes().unwrap(), &[
                "description", "directories", "permissions", "empty-files", "unicode", "multi-frame-binary",
                "symlinks", "normal-variables", "secret-variables", "empty-variables", "form-definitions",
                "form-revisions", "all-eight-field-kinds", "empty-form", "mirror-remove", "mirror-retain",
                "mirror-rules-and-identity", "password-and-contact-access", "committed-replacement-and-deletion",
            ]);
        }
    }
}

archive_generator!(
    archive_v1,
    revault_lockbox_api_export_v1,
    1,
    "revault_lockbox_api =0.0.6"
);
archive_generator!(
    archive_v2,
    revault_lockbox_api_v2,
    2,
    "revault_lockbox_api =0.0.9",
    true
);
archive_generator!(
    archive_v3,
    revault_lockbox_api,
    3,
    "revault_lockbox_api 0.0.39 (workspace)",
    true
);

macro_rules! vault_generator {
    ($function:ident, $vault:ident, $api:ident, $version:expr, $writer:expr, $email:ident $(, $password_profile:ident)?) => {
        fn $function() {
            use $api as api;
            let temp = tempfile::tempdir().unwrap();
            let password = $vault::SecretString::try_from_slice(PASSWORD).unwrap();
            let vault = $vault::VaultDirectory::replace(temp.path(), &password).unwrap();
            vault.store_private_key("default", &api::ContactKeyPair::generate().unwrap()).unwrap();
            vault.$email("default", "owner@example.test").unwrap();
            vault.rotate_private_key("default").unwrap();
            let contact = api::ContactKeyPair::generate().unwrap();
            vault.store_contact("signed-contact", &contact.public_key()).unwrap();
            vault.store_contact_signing_key("signed-contact", &api::OwnerSigningKeyPair::generate().unwrap().public_key()).unwrap();
            vault.store_contact("unsigned-contact", &api::ContactKeyPair::generate().unwrap().public_key()).unwrap();
            vault.seed_default_form_definitions().unwrap();
            let id = api::LockboxId::new_random().unwrap();
            let known_path = temp.path().join("known.lbox");
            fs::write(&known_path, b"synthetic known path").unwrap();
            vault.remember_known_lockbox(id, &known_path).unwrap();
            vault.remember_access_slot_label(id, 42, "fixture slot").unwrap();
            vault.remember_lockbox_password(id, &api::SecretString::try_from_slice(b"remembered fixture password").unwrap()).unwrap();
            // Backup bytes are opaque to the vault; migration must preserve them exactly.
            vault.store_key_directory_backup(id, b"fixture opaque key directory backup").unwrap();
            $(vault.$password_profile("service").unwrap();)?
            drop(vault);
            let mut coverage = vec!["profile-key-and-signing-generations", "retired-generation", "profile-email", "signed-contact", "unsigned-contact", "form-definitions", "known-lockbox", "access-label", "remembered-password", "key-directory-backup"];
            if $version >= 3 { coverage.push("password-profile"); }
            retain("vault", $version, $writer, &fs::read(temp.path().join("local-vault.lbox")).unwrap(), &coverage);
        }
    }
}

vault_generator!(
    vault_v1,
    revault_vault_api_v1,
    revault_lockbox_api_vault_v1,
    1,
    "revault_vault_api =0.0.2; revault_lockbox_api =0.0.2",
    store_identity_email
);
vault_generator!(
    vault_v2,
    revault_vault_api_v2,
    revault_lockbox_api_v2,
    2,
    "revault_vault_api =0.0.10; revault_lockbox_api =0.0.9",
    store_profile_email
);
vault_generator!(
    vault_v3,
    revault_vault_api,
    revault_lockbox_api,
    3,
    "revault_vault_api 0.0.40; revault_lockbox_api 0.0.39 (workspace)",
    store_profile_email,
    create_password_profile
);

vault_generator!(
    vault_v2_container_v1,
    revault_vault_api_container_v1,
    revault_lockbox_api_export_v1,
    2,
    "revault_vault_api =0.0.6; revault_lockbox_api =0.0.6",
    store_profile_email
);

#[test]
#[ignore = "explicit fixture creation only; refuses to overwrite retained bytes"]
fn append_retained_fixture() {
    let name =
        std::env::var("REVAULT_APPEND_FIXTURE").expect("select archive-v1..v3 or vault-v1..v3");
    assert!(
        !root().join(&name).exists(),
        "refusing to overwrite retained fixture directory"
    );
    match name.as_str() {
        "archive-v1" => archive_v1(),
        "archive-v2" => archive_v2(),
        "archive-v3" => {
            assert_eq!(LOCKBOX_FORMAT_VERSION, 3);
            archive_v3();
        }
        "vault-v1" => vault_v1(),
        "vault-v2" => vault_v2(),
        "vault-v2-container-v1" => vault_v2_container_v1(),
        "vault-v3" => {
            assert_eq!(CURRENT_VAULT_STRUCTURE_VERSION, 3);
            vault_v3();
        }
        _ => panic!("add an explicit writer and coverage inventory for the new native version"),
    }
}
