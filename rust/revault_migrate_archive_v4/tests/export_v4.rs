//! Exporter API tests: setup uses the frozen public Rust API, not archive
//! internals. These are not CLI lifecycle E2E tests.
use revault_lockbox_api::{
    Compression, Encryption, Lockbox, LockboxCreateOptions, LockboxOpen, LockboxPath,
    LockboxProtection, OwnerSigningKeyPair, SecretVec, Signing,
};
use revault_migration_format::{
    ArchiveRecord, ArtifactReader, MigrationRecord, JSON_FRAME_TYPE, RAW_FRAME_TYPE,
};
use std::{collections::BTreeMap, fs::File, process::Command};

#[test]
fn binaries_advertise_the_frozen_archive_and_vault_container_versions() {
    for (binary, artifact) in [
        (env!("CARGO_BIN_EXE_revault-migrate-archive-v4"), "archive"),
        (env!("CARGO_BIN_EXE_revault-migrate-vault-v4"), "vault"),
    ] {
        let output = Command::new(binary).arg("capabilities").output().unwrap();
        assert!(output.status.success());
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["artifact"], artifact);
        assert_eq!(value["migration_schema"], 3);
        if artifact == "archive" {
            assert_eq!(value["native_version"], 4);
        } else {
            assert_eq!(value["container_version"], 4);
            assert_eq!(value["structure_versions"], serde_json::json!([3]));
        }
    }
}

#[test]
fn frozen_reader_exports_committed_bytes_across_protection_and_compression_modes() {
    for (encrypted, signed, compression) in [
        (false, false, Compression::None),
        (false, true, Compression::None),
        (true, false, Compression::default()),
        (true, true, Compression::default()),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.lbox");
        let artifact = directory.path().join("export.migration");
        let signer = OwnerSigningKeyPair::generate().unwrap();
        let mut archive = Lockbox::create_file_with_options(
            &source,
            LockboxCreateOptions {
                compression,
                ..LockboxCreateOptions::new(
                    if encrypted {
                        Encryption::Encrypted(LockboxProtection::ContentKey(
                            SecretVec::try_from_slice(&[42; 32]).unwrap(),
                        ))
                    } else {
                        Encryption::None
                    },
                    if signed {
                        Signing::Owner(&signer)
                    } else {
                        Signing::None
                    },
                )
            },
        )
        .unwrap();
        let payload: Vec<u8> = (0..5 * 1024 * 1024 + 13).map(|n| (n % 251) as u8).collect();
        let path = LockboxPath::new("/payload.bin").unwrap();
        archive.add_file(&path, b"initial", false).unwrap();
        archive.commit().unwrap();
        archive.commit().unwrap();
        archive.add_file(&path, &payload, true).unwrap();
        archive.commit().unwrap();
        drop(archive);
        let archive = Lockbox::open(
            &source,
            if encrypted {
                LockboxOpen::ContentKey(SecretVec::try_from_slice(&[42; 32]).unwrap())
            } else {
                LockboxOpen::Unencrypted
            },
        )
        .unwrap();
        assert_eq!(archive.format_version(), 4);
        let before = std::fs::read(&source).unwrap();
        revault_migrate_archive_v4::export_archive(
            &archive,
            &artifact,
            b"test artifact passphrase".as_slice(),
            [7; 16],
        )
        .unwrap();
        let mut reader =
            ArtifactReader::new(File::open(&artifact).unwrap(), b"test artifact passphrase")
                .unwrap();
        assert_eq!(reader.header().source_native_version, 4);
        assert_eq!(reader.header().target_native_version, Some(5));
        let mut paths = BTreeMap::new();
        let mut contents: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
        let mut saw_end = false;
        while let Some((kind, frame)) = reader.next_frame().unwrap() {
            match kind {
                JSON_FRAME_TYPE => {
                    match serde_json::from_slice::<MigrationRecord>(&frame).unwrap() {
                        MigrationRecord::Archive(ArchiveRecord::FileStart {
                            file_id,
                            path,
                            ..
                        }) => {
                            paths.insert(file_id, path);
                        }
                        MigrationRecord::Archive(ArchiveRecord::End { .. }) => saw_end = true,
                        _ => {}
                    }
                }
                RAW_FRAME_TYPE => {
                    let id = u64::from_le_bytes(frame[..8].try_into().unwrap());
                    let offset = u64::from_le_bytes(frame[8..16].try_into().unwrap());
                    let bytes = contents.entry(id).or_default();
                    assert_eq!(offset, bytes.len() as u64);
                    bytes.extend_from_slice(&frame[16..]);
                }
                _ => panic!("unexpected frame type"),
            }
        }
        assert!(saw_end);
        assert_eq!(paths.len(), 1);
        let id = *paths.keys().next().unwrap();
        assert_eq!(paths[&id], "/payload.bin");
        assert_eq!(contents[&id], payload);
        assert_eq!(std::fs::read(&source).unwrap(), before);
        let saved = std::fs::read(&artifact).unwrap();
        assert!(revault_migrate_archive_v4::export_archive(
            &archive,
            &artifact,
            b"test artifact passphrase".as_slice(),
            [8; 16]
        )
        .is_err());
        assert_eq!(std::fs::read(&artifact).unwrap(), saved);
    }
}
