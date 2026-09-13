use revault_lockbox_api::{
    Compression, Encryption, Lockbox, LockboxCreateOptions, LockboxOpen, LockboxPath,
    LockboxProtection, OwnerSigningKeyPair, SecretVec, Signing, SizePadding, VariableName,
};

#[test]
fn migration_preserves_compact_native_v4_in_all_protection_modes() {
    let temp = tempfile::tempdir().unwrap();
    let signer = OwnerSigningKeyPair::generate().unwrap();
    let key = [29; 32];
    let file = LockboxPath::new("/small").unwrap();
    let variable = VariableName::new("small_variable").unwrap();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compression in [Compression::None, Compression::default()] {
                let case = temp
                    .path()
                    .join(format!("{encrypted}-{signed}-{compression:?}"));
                std::fs::create_dir(&case).unwrap();
                let mut source = Lockbox::create_in_memory_with_options(LockboxCreateOptions {
                    size_padding: SizePadding::None,
                    compression,
                    ..LockboxCreateOptions::new(
                        if encrypted {
                            Encryption::Encrypted(LockboxProtection::ContentKey(
                                SecretVec::try_from_slice(&key).unwrap(),
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
                })
                .unwrap();
                source
                    .add_file(&file, b"small migrated content", false)
                    .unwrap();
                source.set_variable(&variable, "metadata").unwrap();
                source.commit().unwrap();
                let artifact = case.join("export.migration");
                let destination = case.join("imported.lbox");
                revault_migration::export_archive(
                    &source,
                    &artifact,
                    b"PUBLIC test artifact password",
                    [7; 16],
                )
                .unwrap();
                revault_migration::import_archive(
                    &artifact,
                    b"PUBLIC test artifact password",
                    &destination,
                    &signer,
                )
                .unwrap();
                let imported = Lockbox::open(
                    &destination,
                    if encrypted {
                        LockboxOpen::ContentKey(SecretVec::try_from_slice(&key).unwrap())
                    } else {
                        LockboxOpen::Unencrypted
                    },
                )
                .unwrap();
                assert_eq!(imported.format_version(), 4);
                assert_eq!(imported.format_options(), source.format_options());
                assert_eq!(imported.get_file(&file).unwrap(), b"small migrated content");
                assert_eq!(
                    imported.get_variable(&variable).unwrap().as_deref(),
                    Some("metadata")
                );
                imported.inspector().verify_storage().unwrap();
            }
        }
    }
}
