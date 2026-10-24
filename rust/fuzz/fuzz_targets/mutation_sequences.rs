#![no_main]

use libfuzzer_sys::fuzz_target;
use revault_lockbox_api::{
    Lockbox, LockboxOpen, LockboxPath, LockboxProtection, OwnerSigningKeyPair, SecretVec,
};
use std::collections::BTreeMap;

const CONTENT_KEY: &[u8] = b"mutation sequence fuzz key";

fn path(index: u8) -> LockboxPath {
    LockboxPath::new(format!("/files/{:02}.bin", index % 32)).unwrap()
}

fn contents(index: u8, size: u8, seed: u8) -> Vec<u8> {
    let len = match size % 8 {
        0 => 0,
        1 => 1,
        2 => 127,
        3 => 4 * 1024,
        4 => 10 * 1024,
        5 => 64 * 1024,
        6 => 256 * 1024,
        _ => 1024 * 1024 + 1,
    };
    let mut state = u64::from(seed) << 32 | u64::from(index) | 1;
    (0..len)
        .map(|_| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            (state >> 32) as u8
        })
        .collect()
}

fn verify<State>(lockbox: &Lockbox<State>, expected: &BTreeMap<LockboxPath, Vec<u8>>) {
    for index in 0..32 {
        let path = path(index);
        if let Some(contents) = expected.get(&path) {
            assert_eq!(lockbox.get_file(&path).unwrap(), *contents, "{path}");
        } else {
            assert!(
                matches!(
                    lockbox.get_file(&path),
                    Err(revault_lockbox_api::Error::NotFound(_))
                ),
                "unexpected live file {path}"
            );
        }
    }
}

fuzz_target!(|data: &[u8]| {
    let signing_key = OwnerSigningKeyPair::generate().unwrap();
    let mut lockbox = Lockbox::create_in_memory(
        LockboxProtection::ContentKey(SecretVec::try_from_slice(CONTENT_KEY).unwrap()),
        &signing_key,
    )
    .unwrap();
    let mut expected = BTreeMap::new();

    for (op_index, operation) in data.chunks_exact(4).take(128).enumerate() {
        let path = path(operation[1]);
        match operation[0] % 4 {
            0 | 1 => {
                let contents = contents(operation[1], operation[2], operation[3]);
                let replace = expected.contains_key(&path);
                lockbox.add_file(&path, &contents, replace).unwrap();
                expected.insert(path, contents);
            }
            2 => {
                if expected.remove(&path).is_some() {
                    lockbox.delete(&path).unwrap();
                }
            }
            _ => {
                lockbox
                    .commit()
                    .unwrap_or_else(|error| panic!("commit after operation {op_index}: {error:?}"));
                verify(&lockbox, &expected);
                lockbox = Lockbox::open_bytes_for_write(
                    lockbox.try_to_bytes().unwrap(),
                    LockboxOpen::ContentKey(SecretVec::try_from_slice(CONTENT_KEY).unwrap()),
                    &signing_key,
                )
                .unwrap();
                verify(&lockbox, &expected);
            }
        }
    }

    lockbox.commit().unwrap();
    let reopened = Lockbox::open_bytes(
        lockbox.try_to_bytes().unwrap(),
        LockboxOpen::ContentKey(SecretVec::try_from_slice(CONTENT_KEY).unwrap()),
    )
    .unwrap();
    verify(&reopened, &expected);
});
