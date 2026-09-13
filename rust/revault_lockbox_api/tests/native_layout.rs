#![cfg(feature = "native-block-layout")]

use revault_lockbox_api::{
    Compression, Encryption, Lockbox, LockboxCreateOptions, LockboxOpen, LockboxPath,
    LockboxProtection, OwnerSigningKeyPair, SecretVec, Signing, SizePadding, VariableName,
};
use std::io::{Cursor, Read, Seek, SeekFrom};

// Wire assertion only: setup and persisted lifecycle operations use public APIs.
// Prevent a green experiment that silently falls back to the legacy writer.
fn assert_native_page(bytes: &[u8]) {
    assert!(bytes
        .windows(10)
        .any(|header| &header[..8] == b"LBX1PAG\0" && header[8..10] == 3u16.to_le_bytes()));
}

#[test]
fn public_streaming_writer_and_rewrites_keep_the_common_native_layout_in_all_modes() {
    let signer = OwnerSigningKeyPair::generate().unwrap();
    let path = LockboxPath::new("/stream").unwrap();
    let renamed = LockboxPath::new("/renamed").unwrap();
    let small = LockboxPath::new("/small").unwrap();
    let input = (0..131_079).map(|i| (i % 251) as u8).collect::<Vec<_>>();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compression in [Compression::None, Compression::default()] {
                for size_padding in [SizePadding::Default, SizePadding::None] {
                    let signing = if signed {
                        Signing::Owner(&signer)
                    } else {
                        Signing::None
                    };
                    let open = || {
                        if encrypted {
                            LockboxOpen::ContentKey(SecretVec::try_from_slice(&[63; 32]).unwrap())
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
                                        SecretVec::try_from_slice(&[63; 32]).unwrap(),
                                    ))
                                } else {
                                    Encryption::None
                                },
                                signing,
                            )
                        })
                        .unwrap();
                    archive
                        .add_file_from_reader(&path, Cursor::new(&input), false)
                        .unwrap();
                    archive
                        .add_file(&small, b"packed companion", false)
                        .unwrap();
                    archive.commit().unwrap();
                    let bytes = archive.try_to_bytes().unwrap();
                    assert_native_page(&bytes);
                    let mut archive =
                        Lockbox::open_bytes_for_write(bytes, open(), signing).unwrap();
                    assert_eq!(archive.get_file(&path).unwrap(), input);
                    archive.commit().unwrap();
                    archive.rename(&path, &renamed).unwrap();
                    archive.delete(&small).unwrap();
                    archive.commit().unwrap();
                    // Reuse retired control/file ranges while this write handle
                    // keeps its cache. Reopening only the verifier catches stale
                    // cache state that a fresh writer per iteration would hide.
                    let variable = VariableName::new("/changing").unwrap();
                    let mut state = 0x1234_5678u64;
                    for iteration in 0..8 {
                        let value = "v".repeat(if iteration % 2 == 0 { 16_384 } else { 1 });
                        let content = (0..if iteration % 2 == 0 { 128 * 1024 } else { 1 })
                            .map(|_| {
                                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                                (state >> 32) as u8
                            })
                            .collect::<Vec<_>>();
                        archive.set_variable(&variable, &value).unwrap();
                        archive.add_file(&small, &content, iteration != 0).unwrap();
                        archive.commit().unwrap();
                        let verified =
                            Lockbox::open_bytes(archive.try_to_bytes().unwrap(), open()).unwrap();
                        verified.inspector().verify_storage().unwrap();
                        assert_eq!(verified.get_file(&small).unwrap(), content);
                        assert_eq!(verified.get_file(&renamed).unwrap(), input);
                        assert_eq!(
                            verified.get_variable(&variable).unwrap().as_deref(),
                            Some(value.as_str())
                        );
                        assert_eq!(verified.format_options().size_padding, size_padding);
                    }
                    archive.delete(&small).unwrap();
                    archive.commit().unwrap();
                    archive.compact().unwrap();
                    let bytes = archive.try_to_bytes().unwrap();
                    assert_native_page(&bytes);
                    let reopened = Lockbox::open_bytes(bytes, open()).unwrap();
                    reopened.inspector().verify_storage().unwrap();
                    assert_eq!(reopened.format_options().size_padding, size_padding);
                    assert!(reopened.get_file(&path).is_err());
                    assert!(reopened.get_file(&small).is_err());
                    assert_eq!(reopened.get_file(&renamed).unwrap(), input);
                    let mut reader = reopened.open_file(&renamed).unwrap();
                    reader.seek(SeekFrom::Start(16_379)).unwrap();
                    let mut crossing = [0; 23];
                    reader.read_exact(&mut crossing).unwrap();
                    assert_eq!(crossing, input[16_379..16_402]);
                    let report = reopened.inspector().recovery_report();
                    assert_eq!(report.partial_files, 0);
                    assert_eq!(report.intact_file_count, 1);
                }
            }
        }
    }
}
