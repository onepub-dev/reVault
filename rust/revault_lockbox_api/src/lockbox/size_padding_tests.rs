//! Fault injection is private unit testing, not public CLI E2E setup.
use crate::{
    Compression, Encryption, LockboxCreateOptions, LockboxOptions, LockboxProtection,
    OwnerSigningKeyPair, Signing, SizePadding,
};
#[test]
fn commit_failures_recover_to_one_complete_generation_in_every_padding_and_protection_mode() {
    use crate::{Lockbox, LockboxOpen, LockboxPath, SecretVec, VariableName};
    let signer = OwnerSigningKeyPair::generate().unwrap();
    let key = [67; 32];
    let file = LockboxPath::new("/changed").unwrap();
    let keep = LockboxPath::new("/keep").unwrap();
    let variable = VariableName::new("revision").unwrap();
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
                    archive.add_file(&file, b"old", false).unwrap();
                    archive.add_file(&keep, b"must survive", false).unwrap();
                    archive.set_variable(&variable, "old").unwrap();
                    archive.commit().unwrap();
                    // Establish reusable extents before injecting commit failures.
                    archive.add_file(&file, b"old", true).unwrap();
                    archive.commit().unwrap();
                    let base = archive.to_bytes();
                    let stage = || {
                        let mut attempt =
                            Lockbox::open_bytes_for_write(base.clone(), open(), signing).unwrap();
                        attempt.add_file(&file, b"new", true).unwrap();
                        attempt.set_variable(&variable, "new").unwrap();
                        attempt
                    };
                    let mut successful = stage();
                    successful.storage.reset_memory_operation_count();
                    successful.commit().unwrap();
                    let count = successful.storage.memory_operation_count();
                    for failure in 0..count {
                        let mut attempt = stage();
                        attempt
                            .storage
                            .fail_memory_operation_after_successes(failure);
                        let committed = attempt.commit();
                        let mut recovered = Lockbox::open_storage_with_secret_key_mode(
                            crate::storage::StorageBackend::memory(attempt.to_bytes()),
                            SecretVec::try_from_slice(if encrypted { &key } else { &[0; 32] })
                                .unwrap(),
                            LockboxOptions::default(),
                            true,
                        )
                        .unwrap_or_else(|e| {
                            panic!(
                                "mode {encrypted}/{signed}/{compression:?}, failure {failure}: {e}"
                            )
                        });
                        recovered.complete_pending_transaction_cleanup().unwrap();
                        let content = recovered.get_file(&file).unwrap();
                        assert!(content == b"old" || content == b"new");
                        assert_eq!(
                            recovered
                                .get_variable(&variable)
                                .unwrap()
                                .unwrap()
                                .as_bytes(),
                            content
                        );
                        assert_eq!(recovered.get_file(&keep).unwrap(), b"must survive");
                        assert_eq!(recovered.format_options().size_padding, size_padding);
                        recovered.inspector().verify_storage().unwrap_or_else(|e| panic!("mode {encrypted}/{signed}/{compression:?}, failure {failure}/{count}, commit {committed:?}, sequence {} (base {}), len {} (base {}), content {:?}, free {:?}, live {:?}: {e}", recovered.sequence, archive.sequence, recovered.to_bytes().len(), base.len(), content, recovered.free_space.slots_by_offset(), recovered.storage_inventory(true).unwrap().ranges));
                    }
                }
            }
        }
    }
}
