#![no_main]

use libfuzzer_sys::fuzz_target;
use revault_lockbox_api::{
    Lockbox, LockboxPath, LockboxProtection, OwnerSigningKeyPair, RecoveryScanner, SecretVec,
};

fuzz_target!(|data: &[u8]| {
    let signing_key = OwnerSigningKeyPair::generate().unwrap();
    let _ = RecoveryScanner::scan_bytes(data.to_vec(), b"fuzz key");
    let _ = RecoveryScanner::salvage_bytes(data.to_vec(), b"fuzz key", &signing_key);

    let key = b"fuzz key";
    let mut lockbox = Lockbox::create_in_memory(
        LockboxProtection::ContentKey(SecretVec::try_from_slice(key).unwrap()),
        &signing_key,
    )
    .unwrap();
    lockbox
        .add_file(
            &LockboxPath::new("/seed/file.bin").unwrap(),
            b"authenticated recovery seed",
            false,
        )
        .unwrap();
    lockbox.commit().unwrap();
    let mut mutated = lockbox.try_to_bytes().unwrap();
    if let Some((&mode, mutations)) = data.split_first() {
        if mode % 4 == 0 && !mutations.is_empty() {
            let new_len = usize::from(mutations[0]) * mutated.len() / 256;
            mutated.truncate(new_len);
        } else {
            for mutation in mutations.chunks_exact(3).take(32) {
                let index =
                    (usize::from(mutation[0]) << 8 | usize::from(mutation[1])) % mutated.len();
                mutated[index] ^= mutation[2];
            }
        }
    }
    let _ = RecoveryScanner::scan_bytes(mutated.clone(), key);
    let _ = RecoveryScanner::salvage_bytes(mutated, key, &signing_key);
});
