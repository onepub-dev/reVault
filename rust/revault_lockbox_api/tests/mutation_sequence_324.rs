//! Synthetic failure from GitHub issue #324. The original 246-byte input has
//! SHA-256 15884496fc422cb4c074064ec5c87653f8e6399d15a46cb9c5dfb2d13c2b166c.
//! Its 79-byte minimized form has SHA-256
//! 64aa58c6259d56ffd2ed71223430ae491b0dc8094a4cc2489682c829509a6557.
//! The October 5 development revision failed during pre-publication allocation
//! retirement. Both cases fail at ea966529 and pass at 808abf12, whose storage
//! changes replace guessed page sizes with persisted or format-aware lengths
//! and adopt the published cleanup free index in the writable handle. Both
//! cases also pass against the released 0.5.0 implementation.
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

#[test]
fn preserved_mutation_sequence_commits_and_reopens() {
    check_sequence(include_bytes!("fixtures/mutation-324.bin"));
}

#[test]
fn minimized_mutation_sequence_commits_and_reopens() {
    check_sequence(include_bytes!("fixtures/mutation-324-min.bin"));
}

fn check_sequence(data: &[u8]) {
    let signing_key = OwnerSigningKeyPair::generate().unwrap();
    let mut lockbox = Lockbox::create_in_memory(
        LockboxProtection::ContentKey(SecretVec::try_from_slice(CONTENT_KEY).unwrap()),
        &signing_key,
    )
    .unwrap();
    let mut expected = BTreeMap::new();

    for (index, operation) in data.chunks_exact(4).take(128).enumerate() {
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
                    .unwrap_or_else(|error| panic!("commit after operation {index}: {error:?}"));
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

    lockbox.commit().expect("final commit");
    let reopened = Lockbox::open_bytes(
        lockbox.try_to_bytes().unwrap(),
        LockboxOpen::ContentKey(SecretVec::try_from_slice(CONTENT_KEY).unwrap()),
    )
    .unwrap();
    verify(&reopened, &expected);
}
