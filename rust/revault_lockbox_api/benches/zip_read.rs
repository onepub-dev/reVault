//! Issue #310: file-backed ZIP comparison with fresh archive handles per sample.
//! Run: cargo bench -p revault_lockbox_api --bench zip_read
//! OS page caches stay warm; no decoded lockbox data survives between samples.
//! ZIP uses Rust deflate; lockbox uses its default Zstd or no compression.
//! Filter with REVAULT_ZIP_READ_CASE=large and REVAULT_ZIP_READ_MODE=plain.
//! REVAULT_ZIP_READ_SIZE sets large-file bytes; REVAULT_ZIP_READ_CORPUS=random
//! selects deterministic high-entropy data instead of the repeating pattern.
//! Optional COMPRESSION (true/false), PROFILE (Interactive/ReadMostly), and
//! ACCESS (stream/random/whole/range) filters use the REVAULT_ZIP_READ_ prefix.
//! CREATE_PROFILE=BulkImport packs small files; default is Interactive. Record
//! this fixture setting alongside CSV results when overriding it.
//! WRITE_PROFILE=1 emits CSV create/add/commit/close timings on stderr for each
//! fixture. Read samples still independently reopen and verify all stored bytes.
use revault_lockbox_api::{
    Compression, ContentStreamOptions, Encryption, Lockbox, LockboxCreateOptions, LockboxOpen,
    LockboxPath, LockboxProtection, OwnerSigningKeyPair, SecretVec, Signing, WorkloadProfile,
};
use std::fs::{self, File};
use std::hint::black_box;
use std::io::{Read, Seek, SeekFrom, Write};
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

fn sample(mut operation: impl FnMut() -> (Duration, Duration)) -> (Duration, Duration, Duration) {
    let iterations = std::env::var("REVAULT_ZIP_READ_SAMPLES")
        .map(|value| value.parse::<usize>().unwrap())
        .unwrap_or(5);
    assert!(iterations > 0);
    let mut elapsed = Vec::new();
    let mut opens = Vec::new();
    let mut reads = Vec::new();
    for _ in 0..iterations {
        let start = Instant::now();
        let (open, read) = operation();
        elapsed.push(start.elapsed());
        opens.push(open);
        reads.push(read);
    }
    elapsed.sort();
    opens.sort();
    reads.sort();
    (
        elapsed[elapsed.len() / 2],
        opens[opens.len() / 2],
        reads[reads.len() / 2],
    )
}

fn run(root: &Path, count: usize, size: usize, compressed: bool) {
    let corpus = std::env::var("REVAULT_ZIP_READ_CORPUS").unwrap_or_else(|_| "pattern".into());
    assert!(matches!(corpus.as_str(), "pattern" | "random"));
    let payloads: Vec<Vec<u8>> = (0..count)
        .map(|index| {
            (0..size)
                .map(|offset| {
                    if corpus == "pattern" {
                        ((offset * 13 + offset / 251 + index * 17) % 251) as u8
                    } else {
                        let mut value = (offset as u64)
                            .wrapping_add((index as u64) << 32)
                            .wrapping_add(0x9e37_79b9_7f4a_7c15);
                        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                        (value ^ (value >> 31)) as u8
                    }
                })
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
        if std::env::var("REVAULT_ZIP_READ_MODE").is_ok_and(|selected| selected != mode) {
            continue;
        }
        let archive = root.join(format!("{mode}.lbox"));
        let write_started = std::env::var("REVAULT_ZIP_READ_WRITE_PROFILE")
            .is_ok_and(|value| value == "1")
            .then(Instant::now);
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
        let created = write_started.map(|_| Instant::now());
        if std::env::var("REVAULT_ZIP_READ_CREATE_PROFILE").is_ok_and(|v| v == "BulkImport") {
            lockbox.set_workload_profile(WorkloadProfile::BulkImport);
        }
        for (path, payload) in paths.iter().zip(&payloads) {
            lockbox.add_file(path, payload, false).unwrap();
        }
        let added = write_started.map(|_| Instant::now());
        lockbox.commit().unwrap();
        let committed = write_started.map(|_| Instant::now());
        drop(lockbox);
        if let (Some(started), Some(created), Some(added), Some(committed)) =
            (write_started, created, added, committed)
        {
            let closed = Instant::now();
            let profile = std::env::var("REVAULT_ZIP_READ_CREATE_PROFILE")
                .unwrap_or_else(|_| "Interactive".into());
            eprintln!("{count},{size},{compressed},{mode},{profile},{corpus},{:.3},{:.3},{:.3},{:.3},{:.3},{}",
                created.duration_since(started).as_secs_f64() * 1000.0,
                added.duration_since(created).as_secs_f64() * 1000.0,
                committed.duration_since(added).as_secs_f64() * 1000.0,
                closed.duration_since(committed).as_secs_f64() * 1000.0,
                closed.duration_since(started).as_secs_f64() * 1000.0,
                fs::metadata(&archive).unwrap().len());
        }
        for profile in [WorkloadProfile::Interactive, WorkloadProfile::ReadMostly] {
            if std::env::var("REVAULT_ZIP_READ_PROFILE")
                .is_ok_and(|selected| selected != format!("{profile:?}"))
            {
                continue;
            }
            for access in ["stream", "random", "whole", "range"] {
                if std::env::var("REVAULT_ZIP_READ_ACCESS").is_ok_and(|selected| selected != access)
                {
                    continue;
                }
                if access == "range" && size < 1024 * 1024 {
                    continue;
                }
                let (zip_time, zip_open, zip_read) = sample(|| {
                    let started = Instant::now();
                    let mut zip = ZipArchive::new(File::open(&zip_path).unwrap()).unwrap();
                    let open = started.elapsed();
                    let started = Instant::now();
                    if access == "stream" {
                        for (index, payload) in payloads.iter().enumerate() {
                            check(&mut zip.by_index(index).unwrap(), payload);
                        }
                    } else if access == "whole" {
                        for &index in &order {
                            let mut file = zip.by_name(&paths[index].as_str()[1..]).unwrap();
                            let mut bytes = Vec::with_capacity(payloads[index].len());
                            file.read_to_end(&mut bytes).unwrap();
                            assert_eq!(bytes, payloads[index]);
                            black_box(bytes);
                        }
                    } else if access == "random" {
                        for &index in &order {
                            check(
                                &mut zip.by_name(&paths[index].as_str()[1..]).unwrap(),
                                &payloads[index],
                            );
                        }
                    } else {
                        for &index in &order {
                            for offset in [
                                (size / 2).saturating_sub(1024),
                                size.saturating_sub(8192),
                                0,
                            ] {
                                let len = 8192.min(size - offset);
                                if !compressed {
                                    let mut file =
                                        zip.by_name_seek(&paths[index].as_str()[1..]).unwrap();
                                    file.seek(SeekFrom::Start(offset as u64)).unwrap();
                                    check(
                                        &mut file.take(len as u64),
                                        &payloads[index][offset..offset + len],
                                    );
                                    continue;
                                }
                                let mut file = zip.by_name(&paths[index].as_str()[1..]).unwrap();
                                // Deflated ZIP entries must decode their prefix to reach a
                                // range; no full-entry CRC is checked for these partial reads.
                                std::io::copy(
                                    &mut file.by_ref().take(offset as u64),
                                    &mut std::io::sink(),
                                )
                                .unwrap();
                                check(
                                    &mut file.take(len as u64),
                                    &payloads[index][offset..offset + len],
                                );
                            }
                        }
                    }
                    (open, started.elapsed())
                });
                let (lockbox_time, lockbox_open, lockbox_read) = sample(|| {
                    let started = Instant::now();
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
                    let open = started.elapsed();
                    let started = Instant::now();
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
                    } else if access == "whole" {
                        for &index in &order {
                            let bytes = lockbox.get_file(&paths[index]).unwrap();
                            assert_eq!(bytes, payloads[index]);
                            black_box(bytes);
                        }
                    } else if access == "random" {
                        for &index in &order {
                            check(
                                &mut lockbox.open_file(&paths[index]).unwrap(),
                                &payloads[index],
                            );
                        }
                    } else {
                        for &index in &order {
                            let mut file = lockbox.open_file(&paths[index]).unwrap();
                            for offset in [
                                (size / 2).saturating_sub(1024),
                                size.saturating_sub(8192),
                                0,
                            ] {
                                let len = 8192.min(size - offset);
                                file.seek(SeekFrom::Start(offset as u64)).unwrap();
                                check(
                                    &mut file.by_ref().take(len as u64),
                                    &payloads[index][offset..offset + len],
                                );
                            }
                        }
                    }
                    (open, started.elapsed())
                });
                println!(
                    "{count},{size},{compressed},{mode},{profile:?},{access},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{},{corpus}",
                    zip_time.as_secs_f64() * 1000.0,
                    lockbox_time.as_secs_f64() * 1000.0,
                    lockbox_time.as_secs_f64() / zip_time.as_secs_f64(),
                    zip_open.as_secs_f64() * 1000.0,
                    zip_read.as_secs_f64() * 1000.0,
                    lockbox_open.as_secs_f64() * 1000.0,
                    lockbox_read.as_secs_f64() * 1000.0,
                    fs::metadata(&zip_path).unwrap().len(),
                    fs::metadata(&archive).unwrap().len(),
                );
            }
        }
        fs::remove_file(archive).unwrap();
    }
}

fn main() {
    if std::env::var("REVAULT_ZIP_READ_WRITE_PROFILE").is_ok_and(|v| v == "1") {
        eprintln!("files,bytes_per_file,compressed,mode,create_profile,corpus,create_ms,add_ms,commit_ms,close_ms,total_ms,archive_bytes");
    }
    for (name, allowed) in [
        ("REVAULT_ZIP_READ_COMPRESSION", &["true", "false"][..]),
        (
            "REVAULT_ZIP_READ_CREATE_PROFILE",
            &["Interactive", "BulkImport"][..],
        ),
        (
            "REVAULT_ZIP_READ_PROFILE",
            &["Interactive", "ReadMostly"][..],
        ),
        (
            "REVAULT_ZIP_READ_ACCESS",
            &["stream", "random", "whole", "range"][..],
        ),
    ] {
        if let Ok(value) = std::env::var(name) {
            assert!(allowed.contains(&value.as_str()), "invalid {name}: {value}");
        }
    }
    let root = std::env::temp_dir().join(format!("revault-zip-read-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    println!(
        "files,bytes_per_file,compressed,mode,profile,access,zip_ms,lockbox_ms,lockbox_over_zip,zip_open_ms,zip_read_ms,lockbox_open_ms,lockbox_read_ms,zip_bytes,lockbox_bytes,corpus"
    );
    for compressed in [false, true] {
        if std::env::var("REVAULT_ZIP_READ_COMPRESSION")
            .is_ok_and(|selected| selected != compressed.to_string())
        {
            continue;
        }
        let case = std::env::var("REVAULT_ZIP_READ_CASE").unwrap_or_default();
        if case.is_empty() || case == "small" {
            run(&root, 512, 4096, compressed);
        }
        if case.is_empty() || case == "large" {
            let size = std::env::var("REVAULT_ZIP_READ_SIZE")
                .map(|value| value.parse().unwrap())
                .unwrap_or(1024 * 1024);
            run(&root, 4, size, compressed);
        }
    }
    fs::remove_dir_all(root).unwrap();
}
