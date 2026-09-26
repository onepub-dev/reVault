use crate::{Lockbox, LockboxPath};
use std::fs::File;
use std::io::{Read, Result as IoResult};
use std::path::{Path, PathBuf};

const KEY: &[u8] = b"compression regression key";
const MIB: usize = 1024 * 1024;

fn p(path: impl AsRef<str>) -> LockboxPath {
    LockboxPath::new(path).unwrap()
}

#[test]
#[ignore = "compression corpus regression; GitHub Actions runs this explicitly"]
fn repeated_small_files_keep_meaningful_compression() {
    let file_count = 100_000usize;
    let file_size = 1024usize;
    let mut lockbox = Lockbox::create(KEY);
    let payload = repeated_small_payload(file_size);
    lockbox
        .create_parent_dirs_for(&p("/small/file.bin"))
        .unwrap();

    for index in 0..file_count {
        lockbox
            .add_file(&p(format!("/small/file-{index:05}.bin")), &payload, false)
            .unwrap();
    }
    lockbox.commit().unwrap();

    let logical = file_count * file_size;
    let lockbox_len = lockbox.to_bytes().len();

    assert!(
        lockbox_len < logical,
        "lockbox should compress repeated small-file corpus: lockbox={lockbox_len}, logical={logical}"
    );
}

#[test]
#[ignore = "compression corpus regression; GitHub Actions runs this explicitly"]
fn moderately_large_zero_file_uses_few_fixed_pages() {
    let logical = 256 * MIB;
    let mut lockbox = Lockbox::create(KEY);
    lockbox
        .create_parent_dirs_for(&p("/large/zero.bin"))
        .unwrap();

    lockbox
        .add_file_from_reader(
            &p("/large/zero.bin"),
            corpus_reader("zero-1g.bin", logical, Pattern::Zero),
            false,
        )
        .unwrap();
    lockbox.commit().unwrap();

    let lockbox_len = lockbox.to_bytes().len();
    assert!(
        lockbox_len <= 32 * MIB,
        "zero corpus should stay within four 8 MiB pages: lockbox={lockbox_len}"
    );
}

#[test]
#[ignore = "production-scale compression regression; GitHub Actions runs this explicitly"]
fn one_gib_zero_file_compression_does_not_regress() {
    let logical = 1024 * MIB;
    let mut lockbox = Lockbox::create(KEY);
    lockbox
        .create_parent_dirs_for(&p("/large/zero.bin"))
        .unwrap();

    lockbox
        .add_file_from_reader(
            &p("/large/zero.bin"),
            corpus_reader("zero-1g.bin", logical, Pattern::Zero),
            false,
        )
        .unwrap();
    lockbox.commit().unwrap();

    let lockbox_len = lockbox.to_bytes().len();
    assert!(
        lockbox_len <= 32 * MIB,
        "1 GiB zero corpus should stay within four 8 MiB pages: lockbox={lockbox_len}"
    );
}

#[test]
#[ignore = "production-scale compression regression; GitHub Actions runs this explicitly"]
fn one_gib_high_entropy_file_avoids_excessive_expansion() {
    let logical = 1024 * MIB;
    let mut lockbox = Lockbox::create(KEY);
    lockbox
        .create_parent_dirs_for(&p("/large/randomish.bin"))
        .unwrap();

    lockbox
        .add_file_from_reader(
            &p("/large/randomish.bin"),
            corpus_reader("randomish-1g.bin", logical, Pattern::Randomish),
            false,
        )
        .unwrap();
    lockbox.commit().unwrap();

    let lockbox_len = lockbox.to_bytes().len();
    assert!(
        lockbox_len <= logical + 32 * MIB,
        "1 GiB high-entropy corpus should not expand by more than four pages: lockbox={lockbox_len}"
    );
}

fn repeated_small_payload(len: usize) -> Vec<u8> {
    if let Some(path) = corpus_file("small-repeated-1k.bin") {
        if let Ok(bytes) = std::fs::read(path) {
            if bytes.len() == len {
                return bytes;
            }
        }
    }
    vec![b'x'; len]
}

fn corpus_reader(name: &str, len: usize, pattern: Pattern) -> Box<dyn Read> {
    if let Some(path) = corpus_file(name) {
        if let Ok(file) = File::open(path) {
            return Box::new(file.take(len as u64));
        }
    }
    Box::new(PatternReader::new(len, pattern))
}

fn corpus_file(name: &str) -> Option<PathBuf> {
    let root = std::env::var_os("LOCKBOX_COMPRESSION_CORPUS_DIR")?;
    Some(Path::new(&root).join(name))
}

struct PatternReader {
    remaining: usize,
    offset: usize,
    pattern: Pattern,
}

enum Pattern {
    Zero,
    Randomish,
}

impl PatternReader {
    fn new(bytes: usize, pattern: Pattern) -> Self {
        Self {
            remaining: bytes,
            offset: 0,
            pattern,
        }
    }
}

impl Read for PatternReader {
    fn read(&mut self, out: &mut [u8]) -> IoResult<usize> {
        let len = out.len().min(self.remaining);
        if len == 0 {
            return Ok(0);
        }
        match self.pattern {
            Pattern::Zero => out[..len].fill(0),
            Pattern::Randomish => {
                for (i, byte) in out[..len].iter_mut().enumerate() {
                    let n = (self.offset + i) as u64;
                    let mut x = n.wrapping_add(0x9e37_79b9_7f4a_7c15);
                    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                    *byte = (x ^ (x >> 31)) as u8;
                }
            }
        }
        self.offset += len;
        self.remaining -= len;
        Ok(len)
    }
}

#[test]
fn asymmetric_literal_weights_preserve_bytes_across_numeric_compression_levels() {
    use crate::{Compression, ZstdLevel};
    for level in [1, 3, 5, 9, 16, 22] {
        for ordinal in 0..3usize {
            let plain: Vec<u8> = (ordinal * 65536..(ordinal + 1) * 65536)
                .map(|n| ((n / 71 + n / 251) % 19) as u8)
                .collect();
            let (algorithm, encoded) = crate::compression::encode_with_compression(
                &plain,
                Compression::Zstd {
                    level: ZstdLevel::new(level).unwrap(),
                },
            );
            assert_eq!(algorithm, crate::compression::COMPRESSION_ZSTD);
            let decoded = crate::compression::decode_compression_frame(
                algorithm,
                &encoded,
                plain.len() as u64,
            )
            .unwrap();
            assert!(
                decoded == plain,
                "level={level},extent={ordinal}: encoder changed logical contents"
            );
        }
    }
}

#[test]
fn asymmetric_literal_files_survive_public_commit_and_independent_reopen() {
    use crate::{
        Compression, Encryption, LockboxCreateOptions, LockboxOpen, LockboxProtection,
        OwnerSigningKeyPair, SecretVec, Signing, SizePadding, WorkerPolicy,
    };
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let key = [71; 32];
    let inputs: Vec<Vec<u8>> = (0..3usize)
        .map(|ordinal| {
            (ordinal * 65536..(ordinal + 1) * 65536)
                .map(|n| ((n / 71 + n / 251) % 19) as u8)
                .collect()
        })
        .collect();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for padded in [false, true] {
                let mut archive = Lockbox::create_in_memory_with_options(LockboxCreateOptions {
                    compression: Compression::default(),
                    size_padding: if padded {
                        SizePadding::Default
                    } else {
                        SizePadding::None
                    },
                    ..LockboxCreateOptions::new(
                        if encrypted {
                            Encryption::Encrypted(LockboxProtection::ContentKey(
                                SecretVec::try_from_slice(&key).unwrap(),
                            ))
                        } else {
                            Encryption::None
                        },
                        if signed {
                            Signing::Owner(&owner)
                        } else {
                            Signing::None
                        },
                    )
                })
                .unwrap();
                archive.set_worker_policy(WorkerPolicy::Single);
                for (ordinal, plain) in inputs.iter().enumerate() {
                    archive
                        .add_file_from_reader(
                            &p(format!("/extent-{ordinal}")),
                            std::io::Cursor::new(plain),
                            false,
                        )
                        .unwrap();
                }
                archive.commit().unwrap();
                let reopened = Lockbox::open_bytes(
                    archive.to_bytes(),
                    if encrypted {
                        LockboxOpen::ContentKey(SecretVec::try_from_slice(&key).unwrap())
                    } else {
                        LockboxOpen::Unencrypted
                    },
                )
                .unwrap();
                for (ordinal, plain) in inputs.iter().enumerate() {
                    assert!(
                        reopened.get_file(&p(format!("/extent-{ordinal}"))).unwrap() == *plain,
                        "encrypted={encrypted},signed={signed},padded={padded},extent={ordinal}"
                    );
                }
            }
        }
    }
}
