//! Issue #310: file-backed ZIP comparison with fresh archive handles per sample.
//! Run: cargo bench -p revault_lockbox_api --bench zip_read
//! OS page caches stay warm; no decoded lockbox data survives between samples.
//! ZIP uses Rust deflate; lockbox uses its default Zstd or no compression.
use revault_lockbox_api::{
    Compression, ContentStreamOptions, Encryption, Lockbox, LockboxCreateOptions, LockboxOpen,
    LockboxPath, LockboxProtection, OwnerSigningKeyPair, SecretVec, Signing, WorkloadProfile,
};
use std::fs::{self, File};
use std::hint::black_box;
use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const KEY: [u8; 32] = [71; 32];

// Check every returned byte, including streaming chunks. Identical work is done
// for ZIP and lockbox, without disk output or a sink that can discard reads.
fn check(reader: &mut dyn Read, expected: &[u8]) {
    let mut buffer = [0u8; 32 * 1024];
    let mut offset = 0;
    loop {
        let len = reader.read(&mut buffer).unwrap();
        if len == 0 {
            break;
        }
        assert_eq!(buffer[..len], expected[offset..offset + len]);
        offset += len;
    }
    assert_eq!(offset, expected.len());
    black_box(offset);
}

fn sample(mut operation: impl FnMut()) -> Duration {
    let iterations = std::env::var("REVAULT_ZIP_READ_SAMPLES")
        .map(|value| value.parse::<usize>().unwrap())
        .unwrap_or(5);
    assert!(iterations > 0);
    let mut elapsed = Vec::new();
    for _ in 0..iterations {
        let start = Instant::now();
        operation();
        elapsed.push(start.elapsed());
    }
    elapsed.sort();
    elapsed[elapsed.len() / 2]
}

fn run(root: &Path, count: usize, size: usize, compressed: bool) {
    let payloads: Vec<Vec<u8>> = (0..count)
        .map(|index| {
            (0..size)
                .map(|offset| ((offset * 13 + offset / 251 + index * 17) % 251) as u8)
                .collect()
        })
        .collect();
    let paths: Vec<_> = (0..count)
        .map(|index| LockboxPath::new(format!("/file-{index:06}.bin")).unwrap())
        .collect();
    let mut order: Vec<usize> = (0..count).collect();
    let mut state = 123456789u64;
    for index in (1..count).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        order.swap(index, state as usize % (index + 1));
    }
    let zip_path = root.join("comparison.zip");
    let mut zip = ZipWriter::new(File::create(&zip_path).unwrap());
    for (path, payload) in paths.iter().zip(&payloads) {
        zip.start_file(
            &path.as_str()[1..],
            SimpleFileOptions::default().compression_method(if compressed {
                CompressionMethod::Deflated
            } else {
                CompressionMethod::Stored
            }),
        )
        .unwrap();
        zip.write_all(payload).unwrap();
    }
    zip.finish().unwrap();
    let signer = OwnerSigningKeyPair::generate().unwrap();
    for (mode, encrypted, signed) in [
        ("plain", false, false),
        ("encrypted", true, false),
        ("signed", false, true),
        ("encrypted-signed", true, true),
    ] {
        let archive = root.join(format!("{mode}.lbox"));
        let mut lockbox = Lockbox::create_file_with_options(
            &archive,
            LockboxCreateOptions {
                compression: if compressed {
                    Compression::default()
                } else {
                    Compression::None
                },
                ..LockboxCreateOptions::new(
                    if encrypted {
                        Encryption::Encrypted(LockboxProtection::ContentKey(
                            SecretVec::try_from_slice(&KEY).unwrap(),
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
            },
        )
        .unwrap();
        for (path, payload) in paths.iter().zip(&payloads) {
            lockbox.add_file(path, payload, false).unwrap();
        }
        lockbox.commit().unwrap();
        drop(lockbox);
        for profile in [WorkloadProfile::Interactive, WorkloadProfile::ReadMostly] {
            for access in ["stream", "random"] {
                let zip_time = sample(|| {
                    let mut zip = ZipArchive::new(File::open(&zip_path).unwrap()).unwrap();
                    if access == "stream" {
                        for (index, payload) in payloads.iter().enumerate() {
                            check(&mut zip.by_index(index).unwrap(), payload);
                        }
                    } else {
                        for &index in &order {
                            check(
                                &mut zip.by_name(&paths[index].as_str()[1..]).unwrap(),
                                &payloads[index],
                            );
                        }
                    }
                });
                let lockbox_time = sample(|| {
                    let mut lockbox = Lockbox::open(
                        &archive,
                        if encrypted {
                            LockboxOpen::ContentKey(SecretVec::try_from_slice(&KEY).unwrap())
                        } else {
                            LockboxOpen::Unencrypted
                        },
                    )
                    .unwrap();
                    lockbox.set_workload_profile(profile);
                    if access == "stream" {
                        let mut total = 0u64;
                        lockbox
                            .stream_content(ContentStreamOptions::default(), |chunk, reader| {
                                let index = chunk.path.as_str()[6..12].parse::<usize>().unwrap();
                                let start = chunk.file_offset as usize;
                                let end = start + chunk.len as usize;
                                check(reader, &payloads[index][start..end]);
                                total += chunk.len;
                                Ok(())
                            })
                            .unwrap();
                        assert_eq!(total, (count * size) as u64);
                    } else {
                        for &index in &order {
                            check(
                                &mut lockbox.open_file(&paths[index]).unwrap(),
                                &payloads[index],
                            );
                        }
                    }
                });
                println!(
                    "{count},{size},{compressed},{mode},{profile:?},{access},{:.3},{:.3},{:.3}",
                    zip_time.as_secs_f64() * 1000.0,
                    lockbox_time.as_secs_f64() * 1000.0,
                    lockbox_time.as_secs_f64() / zip_time.as_secs_f64()
                );
            }
        }
        fs::remove_file(archive).unwrap();
    }
}

fn main() {
    let root = std::env::temp_dir().join(format!("revault-zip-read-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    println!(
        "files,bytes_per_file,compressed,mode,profile,access,zip_ms,lockbox_ms,lockbox_over_zip"
    );
    for compressed in [false, true] {
        run(&root, 512, 4096, compressed);
        run(&root, 4, 1024 * 1024, compressed);
    }
    fs::remove_dir_all(root).unwrap();
}
