use revault_lockbox_api::{
    Compression, ContentStreamOptions, ContentStreamOrder, Encryption, Error, Lockbox,
    LockboxCreateOptions, LockboxOpen, LockboxPath, LockboxProtection, OwnerSigningKeyPair,
    SecretVec, Signing,
};

// Public API lifecycle, run both with the default layout and native-block-layout.
// Nine MiB crosses the workspace threshold; the small leading file exercises
// scratch growth before the larger file's frames. All bytes are verified after
// an independent reopen, in both stream orders, including after visitor failure.
#[test]
fn large_streams_preserve_content_and_restart_after_visitor_error_in_all_protection_modes() {
    let signer = OwnerSigningKeyPair::generate().unwrap();
    let input = (0..8 * 1024 * 1024)
        .map(|i| ((i * 13 + i / 251) % 251) as u8)
        .collect::<Vec<_>>();
    let small = LockboxPath::new("/a-small").unwrap();
    let large = LockboxPath::new("/b-large").unwrap();
    for encrypted in [false, true] {
        for signed in [false, true] {
            let mut archive = Lockbox::create_in_memory_with_options(LockboxCreateOptions {
                compression: Compression::default(),
                ..LockboxCreateOptions::new(
                    if encrypted {
                        Encryption::Encrypted(LockboxProtection::ContentKey(
                            SecretVec::try_from_slice(&[61; 32]).unwrap(),
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
            archive
                .add_file(&small, &input[..1024 * 1024], false)
                .unwrap();
            archive.add_file(&large, &input, false).unwrap();
            archive.commit().unwrap();
            let bytes = archive.try_to_bytes().unwrap();
            drop(archive);
            let opened = Lockbox::open_bytes(
                bytes,
                if encrypted {
                    LockboxOpen::ContentKey(SecretVec::try_from_slice(&[61; 32]).unwrap())
                } else {
                    LockboxOpen::Unencrypted
                },
            )
            .unwrap();
            for order in [ContentStreamOrder::Logical, ContentStreamOrder::Physical] {
                let options = || ContentStreamOptions { order };
                assert!(matches!(
                    opened.stream_content(options(), |_, reader| {
                        let mut first = [0; 13];
                        reader.read_exact(&mut first).unwrap();
                        Err(Error::CorruptRecord)
                    }),
                    Err(Error::CorruptRecord)
                ));
                let mut total = 0u64;
                opened
                    .stream_content(options(), |chunk, reader| {
                        assert!(chunk.path == small || chunk.path == large);
                        let mut bytes = Vec::new();
                        reader.read_to_end(&mut bytes).unwrap();
                        let start = chunk.file_offset as usize;
                        let end = start + chunk.len as usize;
                        let expected = if chunk.path == small {
                            &input[..1024 * 1024]
                        } else {
                            &input
                        };
                        assert_eq!(bytes, expected[start..end]);
                        total += chunk.len;
                        Ok(())
                    })
                    .unwrap();
                assert_eq!(total, 9 * 1024 * 1024);
            }
        }
    }
}
