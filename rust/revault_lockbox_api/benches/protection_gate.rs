//! Protected-mode regression gate. Same source must be built at both revisions.
//! write ROOT SIZE CORPUS SAMPLES creates fixtures and times create/add/commit.
//! read ROOT SIZE CORPUS SAMPLES reuses those exact bytes with fresh handles.
//! Synthetic key, warm OS cache, no password KDF; every returned byte is checked.
use revault_lockbox_api::{
    Compression, Encryption, Lockbox, LockboxCreateOptions, LockboxOpen, LockboxPath,
    LockboxProtection, OwnerSigningKeyPair, SecretVec, Signing,
};
use std::{fs, io::Read, path::PathBuf, time::Instant};
use zeroize::Zeroize;
const KEY: [u8; 32] = [71; 32];
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(args.len(), 5, "write|read ROOT SIZE pattern|random SAMPLES");
    let writing = match args[0].as_str() {
        "write" => true,
        "read" => false,
        _ => panic!("phase"),
    };
    let root = PathBuf::from(&args[1]);
    let size: usize = args[2].parse().unwrap();
    let corpus = &args[3];
    assert!(size > 0 && size <= 256 * 1024 * 1024);
    assert!(matches!(corpus.as_str(), "pattern" | "random"));
    let samples: usize = args[4].parse().unwrap();
    assert!(samples > 0);
    let payload: Vec<u8> = (0..size)
        .map(|n| {
            if corpus == "pattern" {
                ((n * 13 + n / 251) % 251) as u8
            } else {
                let mut v = (n as u64).wrapping_add(0x9e37_79b9_7f4a_7c15);
                v = (v ^ (v >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                v = (v ^ (v >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                (v ^ (v >> 31)) as u8
            }
        })
        .collect();
    let path = LockboxPath::new("/payload.bin").unwrap();
    // Signing key generation is not part of archive create/read measurements.
    let signer = writing.then(|| OwnerSigningKeyPair::generate().unwrap());
    println!("phase,mode,compressed,size,corpus,sample,open_us,read_us,total_us,archive_bytes");
    let modes = [
        ("plain", false, false),
        ("encrypted", true, false),
        ("signed", false, true),
        ("encrypted-signed", true, true),
    ];
    let mut shuffle = 0x1234_5678_9abc_def0u64;
    for sample in 0..samples {
        let mut order: Vec<_> = (0..if writing { 8 } else { 24 }).collect();
        for i in (1..order.len()).rev() {
            shuffle ^= shuffle << 13;
            shuffle ^= shuffle >> 7;
            shuffle ^= shuffle << 17;
            order.swap(i, shuffle as usize % (i + 1));
        }
        for case in order {
            let access = if writing { 0 } else { case / 8 };
            let mode_index = case % 4;
            let compressed = case % 8 >= 4;
            let (mode, encrypted, signed) = modes[mode_index];
            let file = root.join(format!("{mode}-{compressed}.lbox"));
            if writing && sample == 0 {
                assert!(!file.exists(), "refuse existing fixture");
            }
            if writing && sample > 0 {
                fs::remove_file(&file).unwrap();
            }
            let started = Instant::now();
            let (open_us, read_us, phase);
            if writing {
                let mut lb = Lockbox::create_file_with_options(
                    &file,
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
                                Signing::Owner(signer.as_ref().unwrap())
                            } else {
                                Signing::None
                            },
                        )
                    },
                )
                .unwrap();
                lb.add_file(&path, &payload, false).unwrap();
                lb.commit().unwrap();
                drop(lb);
                open_us = 0.0;
                read_us = 0.0;
                phase = "write";
            } else {
                let lb = Lockbox::open(
                    &file,
                    if encrypted {
                        LockboxOpen::ContentKey(SecretVec::try_from_slice(&KEY).unwrap())
                    } else {
                        LockboxOpen::Unencrypted
                    },
                )
                .unwrap();
                open_us = started.elapsed().as_secs_f64() * 1e6;
                let reading = Instant::now();
                match access {
                    0 => {
                        let mut reader = lb.open_file(&path).unwrap();
                        let mut buffer = [0u8; 32768];
                        let mut offset = 0;
                        loop {
                            let n = reader.read(&mut buffer).unwrap();
                            if n == 0 {
                                break;
                            }
                            assert_eq!(&buffer[..n], &payload[offset..offset + n]);
                            offset += n;
                        }
                        assert_eq!(offset, size);
                        buffer.zeroize();
                        phase = "stream";
                    }
                    1 => {
                        let mut output = lb.read_file_range(&path, 0, size as u64).unwrap();
                        assert_eq!(output, payload);
                        output.zeroize();
                        phase = "whole";
                    }
                    _ => {
                        let start = size / 2;
                        let len = 8192.min(size - start);
                        let mut output =
                            lb.read_file_range(&path, start as u64, len as u64).unwrap();
                        assert_eq!(&output, &payload[start..start + len]);
                        output.zeroize();
                        phase = "range";
                    }
                }
                read_us = reading.elapsed().as_secs_f64() * 1e6;
                drop(lb);
            }
            let total_us = started.elapsed().as_secs_f64() * 1e6;
            let disk_len = fs::metadata(&file).unwrap().len();
            if writing {
                // Independent persisted readback, outside the write timer.
                let lb = Lockbox::open(
                    &file,
                    if encrypted {
                        LockboxOpen::ContentKey(SecretVec::try_from_slice(&KEY).unwrap())
                    } else {
                        LockboxOpen::Unencrypted
                    },
                )
                .unwrap();
                let mut out = lb.read_file_range(&path, 0, size as u64).unwrap();
                assert_eq!(out, payload);
                out.zeroize();
            }
            println!("{phase},{mode},{compressed},{size},{corpus},{sample},{open_us:.3},{read_us:.3},{total_us:.3},{disk_len}");
        }
    }
}
