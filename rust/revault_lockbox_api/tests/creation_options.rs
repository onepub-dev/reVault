use revault_lockbox_api::{
    Compression, Encryption, EncryptionMode, Lockbox, LockboxCreateOptions, LockboxOpen,
    LockboxPath, LockboxProtection, OwnerSigningKeyPair, SecretString, Signing, SigningMode,
    ZstdLevel,
};

#[test]
fn independent_modes_survive_reopen_and_mutation() {
    let password = SecretString::try_from_slice(b"creation options test password").unwrap();
    let signer = OwnerSigningKeyPair::generate().unwrap();
    let path = LockboxPath::new("/notes.txt").unwrap();
    let payload = b"A compressible line of text.\n".repeat(10_000);
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compression in [
                Compression::None,
                Compression::Zstd {
                    level: ZstdLevel::new(6).unwrap(),
                },
            ] {
                let encryption = if encrypted {
                    Encryption::Encrypted(LockboxProtection::Password(&password))
                } else {
                    Encryption::None
                };
                let signing = if signed {
                    Signing::Owner(&signer)
                } else {
                    Signing::None
                };
                let mut lb = Lockbox::create_in_memory_with_options(LockboxCreateOptions {
                    compression,
                    ..LockboxCreateOptions::new(encryption, signing)
                })
                .unwrap();
                lb.add_file(&path, &payload, false).unwrap();
                let variable = revault_lockbox_api::VariableName::new("test_variable").unwrap();
                lb.set_variable(&variable, "normal value").unwrap();
                let secret_name = revault_lockbox_api::VariableName::new("test_secret").unwrap();
                lb.set_secret_variable(&secret_name, &password).unwrap();
                lb.commit().unwrap();
                let bytes = lb.try_to_bytes().unwrap();
                let open = || {
                    if encrypted {
                        LockboxOpen::Password(&password)
                    } else {
                        LockboxOpen::Unencrypted
                    }
                };
                if signed {
                    assert!(
                        Lockbox::open_bytes_for_write(bytes.clone(), open(), Signing::None)
                            .is_err()
                    );
                }
                let reopened = Lockbox::open_bytes(bytes.clone(), open()).unwrap();
                assert_eq!(
                    reopened
                        .read_file_range(&path, 0, payload.len() as u64)
                        .unwrap(),
                    payload
                );
                assert_eq!(reopened.format_options().compression, compression);
                assert_eq!(
                    reopened.format_options().encryption,
                    if encrypted {
                        EncryptionMode::ChaCha20Poly1305
                    } else {
                        EncryptionMode::None
                    }
                );
                assert_eq!(
                    reopened.format_options().signing,
                    if signed {
                        SigningMode::Owner
                    } else {
                        SigningMode::None
                    }
                );
                assert_eq!(reopened.owner_inspection().unwrap().signed, signed);
                assert_eq!(
                    reopened.get_variable(&variable).unwrap().as_deref(),
                    Some("normal value")
                );
                if !encrypted && compression == Compression::None {
                    let marker = b"A compressible line of text.\n";
                    assert!(bytes.windows(marker.len()).any(|window| window == marker));
                }
                if encrypted {
                    let marker = b"A compressible line of text.\n";
                    assert!(!bytes.windows(marker.len()).any(|window| window == marker));
                }
                let mut writer = Lockbox::open_bytes_for_write(bytes, open(), signing).unwrap();
                writer.add_file(&path, b"replacement", true).unwrap();
                writer.commit().unwrap();
                let reopened = Lockbox::open_bytes(writer.try_to_bytes().unwrap(), open()).unwrap();
                assert_eq!(
                    reopened.read_file_range(&path, 0, 11).unwrap(),
                    b"replacement"
                );
            }
        }
    }
}

#[test]
fn levels_are_validated_and_encrypted_archives_refuse_credential_free_open() {
    assert!(ZstdLevel::new(0).is_err());
    assert!(ZstdLevel::new(23).is_err());
    assert!(ZstdLevel::new(22).is_ok());
    let password = SecretString::try_from_slice(b"password").unwrap();
    let lb = Lockbox::create_in_memory_with_options(LockboxCreateOptions::new(
        Encryption::Encrypted(LockboxProtection::Password(&password)),
        Signing::None,
    ))
    .unwrap();
    assert!(Lockbox::open_bytes(lb.try_to_bytes().unwrap(), LockboxOpen::Unencrypted).is_err());
}

#[test]
fn numeric_zstd_levels_round_trip_and_compression_reduces_storage() {
    let path = LockboxPath::new("/repeated.txt").unwrap();
    let payload = b"An easily compressed sentence repeated across the file.\n".repeat(1000);
    let mut raw = Lockbox::create_in_memory_with_options(LockboxCreateOptions {
        compression: Compression::None,
        ..LockboxCreateOptions::new(Encryption::None, Signing::None)
    })
    .unwrap();
    raw.add_file(&path, &payload, false).unwrap();
    raw.commit().unwrap();
    let uncompressed_size = raw.try_to_bytes().unwrap().len();
    for level in [1, 3, 9, 19, 22] {
        let mut lb = Lockbox::create_in_memory_with_options(LockboxCreateOptions {
            compression: Compression::Zstd {
                level: ZstdLevel::new(level).unwrap(),
            },
            ..LockboxCreateOptions::new(Encryption::None, Signing::None)
        })
        .unwrap();
        lb.add_file(&path, &payload, false).unwrap();
        lb.commit().unwrap();
        let bytes = lb.try_to_bytes().unwrap();
        assert!(bytes.len() < uncompressed_size, "level {level}");
        let reopened = Lockbox::open_bytes(bytes, LockboxOpen::Unencrypted).unwrap();
        assert_eq!(reopened.get_file(&path).unwrap(), payload, "level {level}");
    }
}
