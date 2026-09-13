use revault_lockbox_api::{
    Compression, Encryption, Lockbox, LockboxCreateOptions, LockboxOpen, LockboxPath,
    LockboxProtection, OwnerSigningKeyPair, SecretVec, Signing, SizePadding, VariableName,
};

#[test]
fn creation_padding_is_persistent_and_lifecycles_preserve_all_modes() {
    let signer = OwnerSigningKeyPair::generate().unwrap();
    let key = [67; 32];
    let path = LockboxPath::new("/small.bin").unwrap();
    let survivor = LockboxPath::new("/survivor.bin").unwrap();
    let variable = VariableName::new("small_variable").unwrap();
    let form = LockboxPath::new("/account").unwrap();
    let link = LockboxPath::new("/shortcut").unwrap();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compression in [Compression::None, Compression::default()] {
                let mut sizes = Vec::new();
                for size_padding in [SizePadding::Default, SizePadding::None] {
                    let signing = if signed {
                        Signing::Owner(&signer)
                    } else {
                        Signing::None
                    };
                    let open = || {
                        if encrypted {
                            LockboxOpen::ContentKey(SecretVec::try_from_slice(&key).unwrap())
                        } else {
                            LockboxOpen::Unencrypted
                        }
                    };
                    let mut archive =
                        Lockbox::create_in_memory_with_options(LockboxCreateOptions {
                            compression,
                            size_padding,
                            ..LockboxCreateOptions::new(
                                if encrypted {
                                    Encryption::Encrypted(LockboxProtection::ContentKey(
                                        SecretVec::try_from_slice(&key).unwrap(),
                                    ))
                                } else {
                                    Encryption::None
                                },
                                signing,
                            )
                        })
                        .unwrap();
                    archive.add_file(&path, &[3; 128], false).unwrap();
                    archive.add_file(&survivor, &[7; 113], false).unwrap();
                    archive.add_symlink(&link, &survivor, false).unwrap();
                    archive.set_variable(&variable, "first value").unwrap();
                    archive
                        .define_form(
                            "login",
                            "Login",
                            vec![revault_lockbox_api::FormFieldDefinition {
                                id: "name".into(),
                                label: "Name".into(),
                                kind: revault_lockbox_api::FormFieldKind::Text,
                                required: false,
                            }],
                        )
                        .unwrap();
                    archive
                        .create_form_record(&form, "login", "Account")
                        .unwrap();
                    archive
                        .set_form_field_normal(&form, "name", "first value")
                        .unwrap();
                    archive.commit().unwrap();
                    archive.inspector().verify_storage().unwrap();
                    let bytes = archive.try_to_bytes().unwrap();
                    sizes.push(bytes.len());
                    eprintln!("encrypted={encrypted} signed={signed} compression={compression:?} padding={size_padding:?}: {} bytes", bytes.len());
                    drop(archive);
                    let mut archive =
                        Lockbox::open_bytes_for_write(bytes, open(), signing).unwrap();
                    assert_eq!(archive.format_options().size_padding, size_padding);
                    assert_eq!(archive.format_options().compression, compression);
                    assert_eq!(archive.get_symlink_target(&link).unwrap(), survivor);
                    assert_eq!(archive.read_file_range(&path, 17, 29).unwrap(), [3; 29]);
                    let before = archive.try_to_bytes().unwrap();
                    archive.commit().unwrap();
                    assert_eq!(archive.try_to_bytes().unwrap(), before, "no-change commit");
                    // Replacements, growth/shrink and packed-page survivor relocation.
                    for length in [4097, 19, 12345, 0, 128] {
                        archive.add_file(&path, &vec![5; length], true).unwrap();
                        archive
                            .set_variable(&variable, &"v".repeat(length.min(2048)))
                            .unwrap();
                        archive
                            .set_form_field_normal(&form, "name", &"f".repeat(length.min(2048)))
                            .unwrap();
                        archive.commit().unwrap();
                        archive.inspector().verify_storage().unwrap();
                        let bytes = archive.try_to_bytes().unwrap();
                        drop(archive);
                        archive = Lockbox::open_bytes_for_write(bytes, open(), signing).unwrap();
                        assert_eq!(
                            archive.read_file_range(&path, 0, u64::MAX).unwrap(),
                            vec![5; length]
                        );
                        assert_eq!(
                            archive.read_file_range(&survivor, 0, 113).unwrap(),
                            [7; 113]
                        );
                        assert_eq!(
                            archive.get_variable(&variable).unwrap().unwrap(),
                            "v".repeat(length.min(2048))
                        );
                        assert!(
                            matches!(archive.get_form_field(&form, "name").unwrap().unwrap().value,
                            revault_lockbox_api::FormValue::Normal(value) if value == "f".repeat(length.min(2048)))
                        );
                        assert_eq!(archive.format_options().size_padding, size_padding);
                        assert_eq!(archive.get_symlink_target(&link).unwrap(), survivor);
                    }
                    archive.delete(&path).unwrap();
                    archive.delete_variable(&variable).unwrap();
                    archive.delete_form_record(&form).unwrap();
                    archive.commit().unwrap();
                    archive.inspector().verify_storage().unwrap();
                    let reopened =
                        Lockbox::open_bytes(archive.try_to_bytes().unwrap(), open()).unwrap();
                    assert_eq!(reopened.get_variable(&variable).unwrap(), None);
                    assert_eq!(
                        reopened.read_file_range(&survivor, 0, 113).unwrap(),
                        [7; 113]
                    );
                    assert!(reopened.read_file_range(&path, 0, 1).is_err());
                    archive.compact().unwrap();
                    archive.inspector().verify_storage().unwrap();
                    let compacted =
                        Lockbox::open_bytes(archive.try_to_bytes().unwrap(), open()).unwrap();
                    assert_eq!(compacted.format_options().size_padding, size_padding);
                    assert_eq!(
                        compacted.read_file_range(&survivor, 0, 113).unwrap(),
                        [7; 113]
                    );
                    assert_eq!(compacted.get_symlink_target(&link).unwrap(), survivor);
                }
                assert!(
                    sizes[1] < sizes[0],
                    "compact allocation must be smaller: {sizes:?}"
                );
            }
        }
    }
}

#[test]
fn compact_password_archives_bootstrap_keys_and_preserve_policy_after_file_compaction() {
    let temp = std::env::temp_dir().join(format!(
        "revault-compact-password-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&temp).unwrap();
    let signer = OwnerSigningKeyPair::generate().unwrap();
    let password =
        revault_lockbox_api::SecretString::try_from_slice(b"PUBLIC compact fixture password")
            .unwrap();
    let wrong =
        revault_lockbox_api::SecretString::try_from_slice(b"PUBLIC incorrect password").unwrap();
    let file = LockboxPath::new("/small").unwrap();
    for signed in [false, true] {
        let destination = temp.join(format!("{signed}.lbox"));
        let signing = if signed {
            Signing::Owner(&signer)
        } else {
            Signing::None
        };
        let mut archive = Lockbox::create_file_with_options(
            &destination,
            LockboxCreateOptions {
                size_padding: SizePadding::None,
                ..LockboxCreateOptions::new(
                    Encryption::Encrypted(LockboxProtection::Password(&password)),
                    signing,
                )
            },
        )
        .unwrap();
        archive
            .add_file(&file, b"password protected small file", false)
            .unwrap();
        archive.commit().unwrap();
        drop(archive);
        let inspected = Lockbox::inspect_file(&destination).unwrap();
        assert_eq!(
            inspected.format_options.unwrap().size_padding,
            SizePadding::None
        );
        assert_eq!(inspected.key_directory_copy_count, 2);
        assert!(Lockbox::open(&destination, LockboxOpen::Password(&wrong)).is_err());
        assert!(Lockbox::open(&destination, LockboxOpen::Unencrypted).is_err());
        let mut archive =
            Lockbox::open_for_write(&destination, LockboxOpen::Password(&password), signing)
                .unwrap();
        assert_eq!(
            archive.get_file(&file).unwrap(),
            b"password protected small file"
        );
        archive.compact().unwrap();
        drop(archive);
        let reopened = Lockbox::open(&destination, LockboxOpen::Password(&password)).unwrap();
        assert_eq!(reopened.format_options().size_padding, SizePadding::None);
        assert_eq!(
            reopened.get_file(&file).unwrap(),
            b"password protected small file"
        );
        reopened.inspector().verify_storage().unwrap();
    }
    std::fs::remove_dir_all(temp).unwrap();
}
