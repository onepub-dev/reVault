//! Permanent native fixtures: release tests read these bytes, never regenerate them.
use revault_lockbox_api::{
    Lockbox, LockboxOpen, LockboxPath, OwnerSigningKeyPair, SecretString, LOCKBOX_FORMAT_VERSION,
};
use revault_migration::*;
use revault_vault_api::{VaultDirectory, CURRENT_VAULT_STRUCTURE_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

const PASSWORD: &[u8] = b"PUBLIC fixture password - never use outside tests";
const ARTIFACT_PASSWORD: &[u8] = b"PUBLIC migration artifact password";

#[derive(Serialize, Deserialize)]
struct Manifest {
    kind: String,
    native_version: u32,
    container_version: u16,
    writer: String,
    migrated_archive_mode: Option<u16>,
    sha256: String,
    password: String,
    coverage: Vec<String>,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/retained")
}
fn password() -> SecretString {
    SecretString::try_from_slice(PASSWORD).unwrap()
}
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// Compare logical records, independent of stream order and encryption randomness.
// Native version/mode and Start/End framing are verified separately. Raw file
// frames retain their file id and offset in the digest, checking every file byte.
// Synthetic private credentials are represented by hashes in reviewable snapshots.
fn snapshot(path: &Path) -> Vec<Value> {
    let mut reader = ArtifactReader::new(fs::File::open(path).unwrap(), ARTIFACT_PASSWORD).unwrap();
    let mut records = Vec::new();
    while let Some((kind, bytes)) = reader.next_frame().unwrap() {
        if kind == RAW_FRAME_TYPE {
            records.push(json!({"file_chunk_sha256": digest(&bytes), "length": bytes.len()}));
            continue;
        }
        assert_eq!(kind, JSON_FRAME_TYPE);
        let mut value: Value = serde_json::from_slice(&bytes).unwrap();
        let record = &mut value["record"];
        if record["type"] == "end" {
            continue;
        }
        if record["type"] == "start" {
            let body = record["value"].as_object_mut().unwrap();
            body.remove("format_version");
            body.remove("structure_version");
            body.remove("format_mode");
        }
        hash_bytes(&mut value);
        records.push(value);
    }
    assert!(reader.is_complete());
    records.sort_by_cached_key(|value| value.to_string());
    records
}

fn hash_bytes(value: &mut Value) {
    match value {
        Value::Array(items)
            if !items.is_empty() && items.iter().all(|v| v.as_u64().is_some_and(|n| n <= 255)) =>
        {
            let bytes: Vec<u8> = items.iter().map(|v| v.as_u64().unwrap() as u8).collect();
            *value = json!({"sha256": digest(&bytes), "length": bytes.len()});
        }
        Value::Array(items) => items.iter_mut().for_each(hash_bytes),
        Value::Object(items) => items.values_mut().for_each(hash_bytes),
        _ => {}
    }
}

fn export_fixture(dir: &Path, manifest: &Manifest, output: &Path) {
    let bytes = fs::read(dir.join("native.bin")).unwrap();
    assert_eq!(
        digest(&bytes),
        manifest.sha256,
        "retained native bytes changed: {}",
        dir.display()
    );
    assert_eq!(
        revault_lockbox_api::probe_lockbox_format_version(&bytes).unwrap(),
        manifest.container_version
    );
    assert_eq!(manifest.password.as_bytes(), PASSWORD);
    match (manifest.kind.as_str(), manifest.native_version) {
        ("archive", 1) => {
            use revault_lockbox_api_export_v1 as old;
            let password = old::SecretString::try_from_slice(PASSWORD).unwrap();
            let archive =
                old::Lockbox::open_bytes(bytes, old::LockboxOpen::Password(&password)).unwrap();
            revault_migrate_archive_v1::export_archive_v1(
                &archive,
                output,
                ARTIFACT_PASSWORD,
                [1; 16],
            )
            .unwrap();
        }
        ("archive", _) => {
            let archive = Lockbox::open_bytes(bytes, LockboxOpen::Password(&password())).unwrap();
            assert_eq!(u32::from(archive.format_version()), manifest.native_version);
            export_archive(&archive, output, ARTIFACT_PASSWORD, [1; 16]).unwrap();
        }
        ("vault", version) => {
            let temp = tempfile::tempdir().unwrap();
            let source = temp.path().join("vault");
            fs::create_dir(&source).unwrap();
            fs::write(source.join("local-vault.lbox"), &bytes).unwrap();
            match version {
                _ if manifest.container_version == 1 => {
                    revault_migrate_vault_v1::export_vault_v1(
                        &source,
                        PASSWORD,
                        output,
                        ARTIFACT_PASSWORD,
                        [1; 16],
                    )
                    .unwrap();
                }
                2 => {
                    let password =
                        revault_vault_api_v2::SecretString::try_from_slice(PASSWORD).unwrap();
                    let vault =
                        revault_vault_api_v2::VaultDirectory::open_or_create(&source, &password)
                            .unwrap();
                    assert_eq!(vault.structure_version().unwrap(), 2);
                    revault_migrate_vault_v2::export_vault_v2(
                        &vault,
                        output,
                        ARTIFACT_PASSWORD,
                        [1; 16],
                    )
                    .unwrap();
                }
                _ => {
                    let vault = VaultDirectory::open_or_create(&source, &password()).unwrap();
                    assert_eq!(vault.structure_version().unwrap(), version);
                    export_vault(&vault, output, ARTIFACT_PASSWORD, [1; 16]).unwrap();
                }
            }
            assert_eq!(
                fs::read(source.join("local-vault.lbox")).unwrap(),
                bytes,
                "export modified source"
            );
        }
        _ => panic!("unregistered fixture kind"),
    }
}

fn assert_snapshot(path: &Path, expected: &[Value], fixture: &Path) {
    let actual = snapshot(path);
    assert_eq!(
        actual.len(),
        expected.len(),
        "record count: {}",
        fixture.display()
    );
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(actual, expected, "record {index}: {}", fixture.display());
    }
}

fn archive_key_directory(path: &Path) -> Vec<u8> {
    let mut reader = ArtifactReader::new(fs::File::open(path).unwrap(), ARTIFACT_PASSWORD).unwrap();
    let Some(MigrationRecord::Archive(ArchiveRecord::Start { key_directory, .. })) =
        reader.next_json::<MigrationRecord>().unwrap()
    else {
        panic!("missing archive start");
    };
    key_directory.as_slice().to_vec()
}

fn assert_imported_archive_snapshot(
    source: &Path,
    imported: &Path,
    expected: &[Value],
    fixture: &Path,
) {
    let mut source_keys = archive_key_directory(source);
    let mut imported_keys = archive_key_directory(imported);
    // Import publishes a new key-directory generation. Only the generation word
    // may differ; compare all UUID, slot, wrapping and reserved bytes exactly.
    // This is a migration-library assertion of the exported backup representation,
    // not manipulation of the native fixture or CLI test state.
    for bytes in [&mut source_keys, &mut imported_keys] {
        assert!(bytes.len() >= 64);
        assert_eq!(&bytes[..8], b"LBX1KEY\0");
        assert_eq!(u16::from_le_bytes(bytes[8..10].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(bytes[12..16].try_into().unwrap()), 64);
        bytes[24..32].fill(0);
    }
    assert_eq!(
        source_keys,
        imported_keys,
        "access slots changed: {}",
        fixture.display()
    );
    let mut actual = snapshot(imported);
    let original_start = expected
        .iter()
        .find(|r| r["record"]["type"] == "start")
        .unwrap();
    for record in &mut actual {
        if record["record"]["type"] == "start" {
            record["record"]["value"]["key_directory"] =
                original_start["record"]["value"]["key_directory"].clone();
        }
    }
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(
            actual,
            expected,
            "imported record {index}: {}",
            fixture.display()
        );
    }
}

fn require_content_inventory(manifest: &Manifest, records: &[Value]) {
    let kinds: std::collections::BTreeSet<_> = records
        .iter()
        .filter_map(|r| r["record"]["type"].as_str())
        .collect();
    let required: &[&str] = if manifest.kind == "archive" {
        &[
            "start",
            "directory",
            "file_start",
            "file_end",
            "symlink",
            "variable",
            "form_definition",
            "form_record",
        ]
    } else {
        &[
            "start",
            "profile",
            "contact",
            "form_definition",
            "known_lockbox",
            "access_label",
            "lockbox_password",
            "key_directory",
        ]
    };
    for kind in required {
        assert!(kinds.contains(kind), "fixture lacks {kind}");
    }
    assert!(!manifest.coverage.is_empty());
    if manifest.kind == "archive" {
        let field_kinds: std::collections::BTreeSet<_> = records
            .iter()
            .filter(|r| r["record"]["type"] == "form_definition")
            .flat_map(|r| r["record"]["value"]["fields"].as_array().unwrap())
            .map(|f| f["kind"].as_str().unwrap())
            .collect();
        for kind in [
            "text", "secret", "url", "email", "date", "month", "notes", "number",
        ] {
            assert!(
                field_kinds.contains(kind),
                "fixture lacks form field kind {kind}"
            );
        }
        for name in [
            "/.revault/mirrors/remove",
            "/.revault/mirrors/retain",
            "/env/NORMAL",
            "/env/SECRET",
            "/env/EMPTY",
        ] {
            assert!(
                records
                    .iter()
                    .any(|r| r["record"]["type"] == "variable"
                        && r["record"]["value"]["name"] == name),
                "fixture lacks variable {name}"
            );
        }
        assert!(records.iter().any(|r| r.get("file_chunk_sha256").is_some()));
    } else {
        assert_eq!(
            kinds.contains("password_profile"),
            manifest.native_version >= 3
        );
        assert!(records.iter().any(|r| r["record"]["type"] == "profile"
            && r["record"]["value"]["generations"]
                .as_array()
                .unwrap()
                .len()
                >= 2));
    }
}

#[test]
fn every_retained_native_version_migrates_to_current() {
    let mut covered = std::collections::BTreeSet::new();
    for entry in fs::read_dir(root()).unwrap() {
        let dir = entry.unwrap().path();
        if !dir.is_dir() {
            continue;
        }
        let manifest: Manifest =
            serde_json::from_slice(&fs::read(dir.join("manifest.json")).unwrap()).unwrap();
        assert!(
            covered.insert((
                manifest.kind.clone(),
                manifest.native_version,
                manifest.container_version
            )),
            "duplicate version"
        );
        let expected: Vec<Value> =
            serde_json::from_slice(&fs::read(dir.join("expected.json")).unwrap()).unwrap();
        assert!(!expected.is_empty());
        require_content_inventory(&manifest, &expected);
        let temp = tempfile::tempdir().unwrap();
        let artifact = temp.path().join("export.migration");
        export_fixture(&dir, &manifest, &artifact);
        let exported_reader =
            ArtifactReader::new(fs::File::open(&artifact).unwrap(), ARTIFACT_PASSWORD).unwrap();
        assert_eq!(
            exported_reader.header().source_native_version,
            manifest.native_version
        );
        drop(exported_reader);
        let upgraded = temp.path().join("upgraded.migration");
        let reexport = temp.path().join("reexport.migration");
        if manifest.kind == "archive" {
            upgrade_archive_artifact(&artifact, &upgraded, ARTIFACT_PASSWORD).unwrap();
            verify_archive_artifact(&upgraded, ARTIFACT_PASSWORD).unwrap();
            assert_snapshot(&upgraded, &expected, &dir);
            let destination = temp.path().join("current.lbox");
            let signer = OwnerSigningKeyPair::generate().unwrap();
            import_archive(&upgraded, ARTIFACT_PASSWORD, &destination, &signer).unwrap();
            let archive = Lockbox::open(&destination, LockboxOpen::Password(&password())).unwrap();
            assert_eq!(archive.format_version(), LOCKBOX_FORMAT_VERSION);
            assert_eq!(
                Some(archive.export_migration_format_mode()),
                manifest.migrated_archive_mode,
                "encryption/signing/compression changed: {}",
                dir.display()
            );
            export_archive(&archive, &reexport, ARTIFACT_PASSWORD, [2; 16]).unwrap();
            assert_imported_archive_snapshot(&upgraded, &reexport, &expected, &dir);
            assert_eq!(archive.list_mirror_projects().unwrap().len(), 2);
            assert_eq!(
                archive
                    .get_file(&LockboxPath::new("/binary.bin").unwrap())
                    .unwrap(),
                (0..5 * 1024 * 1024 + 137)
                    .map(|i| (i % 251) as u8)
                    .collect::<Vec<_>>()
            );
            drop(archive);
            let mut writable =
                Lockbox::open_with_signer(&destination, LockboxOpen::Password(&password()), |_| {
                    signer.try_clone()
                })
                .unwrap();
            let path = LockboxPath::new("/mirrors/remove/file.txt").unwrap();
            assert!(writable.add_file(&path, b"outside mirror", true).is_err());
            writable
                .with_mirror_project_mutation("remove", |archive, _| {
                    archive.add_file(&path, b"after migration", true)
                })
                .unwrap();
            writable.commit().unwrap();
            drop(writable);
            let reopened = Lockbox::open(&destination, LockboxOpen::Password(&password())).unwrap();
            assert_eq!(reopened.get_file(&path).unwrap(), b"after migration");
        } else {
            upgrade_vault_artifact(&artifact, &upgraded, ARTIFACT_PASSWORD).unwrap();
            verify_vault_artifact(&upgraded, ARTIFACT_PASSWORD).unwrap();
            assert_snapshot(&upgraded, &expected, &dir);
            let destination = temp.path().join("vault");
            import_vault(&upgraded, ARTIFACT_PASSWORD, &destination, &password()).unwrap();
            let vault = VaultDirectory::open_or_create(&destination, &password()).unwrap();
            assert_eq!(
                vault.structure_version().unwrap(),
                CURRENT_VAULT_STRUCTURE_VERSION
            );
            export_vault(&vault, &reexport, ARTIFACT_PASSWORD, [2; 16]).unwrap();
            assert_snapshot(&reexport, &expected, &dir);
            vault.create_password_profile("post-migration").unwrap();
            let credential = vault.load_profile_password("post-migration").unwrap();
            drop(vault);
            let reopened = VaultDirectory::open_or_create(&destination, &password()).unwrap();
            assert_eq!(
                reopened.load_profile_password("post-migration").unwrap(),
                credential
            );
        }
        eprintln!(
            "verified retained {} v{}",
            manifest.kind, manifest.native_version
        );
    }
    // Pin known historical combinations as well as the current versions.
    // Vault structure and archive container versions advance independently.
    for required in [
        ("vault", 1, 1),
        ("vault", 2, 1),
        ("vault", 2, 2),
        ("vault", 3, 3),
    ] {
        assert!(
            covered.contains(&(required.0.to_string(), required.1, required.2)),
            "missing historical fixture: {required:?}"
        );
    }
    for version in 1..=u32::from(LOCKBOX_FORMAT_VERSION) {
        assert!(
            covered.contains(&("archive".to_string(), version, version as u16)),
            "retain archive version {version} before releasing"
        );
    }
    for version in 1..=CURRENT_VAULT_STRUCTURE_VERSION {
        assert!(
            covered
                .iter()
                .any(|(kind, native, _)| kind == "vault" && *native == version),
            "retain vault structure {version} before releasing"
        );
    }
    assert!(
        covered.contains(&(
            "vault".to_string(),
            CURRENT_VAULT_STRUCTURE_VERSION,
            LOCKBOX_FORMAT_VERSION
        )),
        "retain the current vault/container combination before releasing"
    );
}

#[path = "retained/generate.rs"]
mod generate;
