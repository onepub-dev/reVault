//! Protected-mode regression gate. Same source must be built at both revisions.
//! write ROOT SIZE CORPUS SAMPLES creates fixtures and times create/add/commit.
//! read ROOT SIZE CORPUS SAMPLES reuses those exact bytes with fresh handles.
//! Synthetic key, warm OS cache, no password KDF; every returned byte is checked.
use revault_lockbox_api::{
    Compression, ContentStreamOptions, Encryption, Lockbox, LockboxCreateOptions, LockboxOpen,
    LockboxPath, LockboxProtection, OwnerSigningKeyPair, SecretVec, Signing,
};
use std::{fs, io::Read, path::PathBuf, time::Instant};
use zeroize::Zeroize;
const KEY: [u8; 32] = [71; 32];
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "compare") {
        compare(&args);
        return;
    }
    if args.first().is_some_and(|arg| arg == "summarize") {
        summarize(&args);
        return;
    }
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
    let mut shuffle = std::env::var("REVAULT_GATE_SEED")
        .map(|value| value.parse::<u64>().unwrap())
        .unwrap_or(0x1234_5678_9abc_def0u64);
    let selected = std::env::var("REVAULT_GATE_CASE")
        .ok()
        .filter(|case| case != "all");
    for sample in 0..samples {
        let mut order: Vec<_> = (0..if writing { 8 } else { 32 }).collect();
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
            let phase_name = if writing {
                "write"
            } else {
                ["stream", "whole", "range", "stream-content"][access]
            };
            if selected
                .as_ref()
                .is_some_and(|case| case != &format!("{mode}/{compressed}/{phase_name}"))
            {
                continue;
            }
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
                    2 => {
                        let start = size / 2;
                        let len = 8192.min(size - start);
                        let mut output =
                            lb.read_file_range(&path, start as u64, len as u64).unwrap();
                        assert_eq!(&output, &payload[start..start + len]);
                        output.zeroize();
                        phase = "range";
                    }
                    _ => {
                        let mut total = 0usize;
                        lb.stream_content(ContentStreamOptions::default(), |chunk, reader| {
                            assert_eq!(chunk.path.as_str(), path.as_str());
                            let mut buffer = [0; 32768];
                            let mut offset = chunk.file_offset as usize;
                            let start = offset;
                            loop {
                                let n = reader.read(&mut buffer).unwrap();
                                if n == 0 {
                                    break;
                                }
                                assert_eq!(&buffer[..n], &payload[offset..offset + n]);
                                offset += n;
                            }
                            assert_eq!((offset - start) as u64, chunk.len);
                            total += offset - start;
                            buffer.zeroize();
                            Ok(())
                        })
                        .unwrap();
                        assert_eq!(total, size);
                        phase = "stream-content";
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

// Drive immediate paired child runs locally rather than inserting tool-call
// delays between revisions. The parent inherits taskset affinity. All children
// read the same immutable synthetic fixtures; first sample is warmup only.
fn compare(args: &[String]) {
    use std::process::Command;
    assert_eq!(
        args.len(),
        8,
        "compare MAIN_EXE BRANCH_EXE ROOT SIZE CORPUS PAIRS CASE"
    );
    let pairs: usize = args[6].parse().unwrap();
    assert!(pairs > 0);
    let samples: usize = std::env::var("REVAULT_GATE_CHILD_SAMPLES")
        .map(|value| value.parse().unwrap())
        .unwrap_or(6);
    assert!(
        samples >= 2,
        "one warmup and at least one measured sample required"
    );
    let samples_arg = samples.to_string();
    let mut printed_header = false;
    for pair in 0..pairs {
        let order = if pair % 2 == 0 {
            [1, 2, 2, 1]
        } else {
            [2, 1, 1, 2]
        };
        for (position, variant) in order.into_iter().enumerate() {
            let output = Command::new(&args[variant])
                .args(["read", &args[3], &args[4], &args[5], &samples_arg])
                .env("REVAULT_GATE_CASE", &args[7])
                .env("REVAULT_GATE_SEED", (pair as u64 + 113).to_string())
                .output()
                .unwrap();
            assert!(output.status.success(), "child benchmark failed");
            let output = String::from_utf8(output.stdout).unwrap();
            let mut lines = output.lines();
            let header = lines.next().unwrap();
            assert!(header.starts_with("phase,mode,compressed,"));
            if !printed_header {
                println!("pair,position,reader,{header}");
                printed_header = true;
            }
            let mut count = 0;
            for line in lines {
                let fields: Vec<_> = line.split(',').collect();
                assert_eq!(fields.len(), 10);
                count += 1;
                if fields[5] != "0" {
                    let reader = if variant == 1 { "main" } else { "branch" };
                    println!("{pair},{position},{reader},{line}");
                }
            }
            assert_eq!(
                count,
                samples * if args[7] == "all" { 32 } else { 1 },
                "unexpected selected workload count"
            );
        }
        eprintln!("completed pair {}/{}", pair + 1, pairs);
    }
}

// Summarize paired raw measurements without choosing favorable individual runs.
// Confidence limits resample whole pairs, not correlated samples within a pair.
fn summarize(args: &[String]) {
    use std::collections::BTreeMap;
    type PairSamples = [Vec<[f64; 3]>; 2];
    type Cases = BTreeMap<String, BTreeMap<usize, PairSamples>>;
    assert_eq!(args.len(), 2, "summarize PAIRED_CSV");
    let csv = fs::read_to_string(&args[1]).unwrap();
    let mut cases: Cases = BTreeMap::new();
    for line in csv.lines().skip(1) {
        let fields: Vec<_> = line.split(',').collect();
        assert_eq!(fields.len(), 13);
        let pair = fields[0].parse::<usize>().unwrap();
        let variant = match fields[2] {
            "main" => 0,
            "branch" => 1,
            _ => panic!("reader"),
        };
        let case = format!("{}/{}/{}", fields[4], fields[5], fields[3]);
        let timing = [fields[9], fields[10], fields[11]].map(|v| v.parse::<f64>().unwrap());
        assert!(timing.iter().all(|v| v.is_finite() && *v > 0.0));
        cases.entry(case).or_default().entry(pair).or_default()[variant].push(timing);
    }
    println!("case,metric,main_us,branch_us,paired_change_pct,low95_pct,high95_pct,pairs");
    for (case, pairs) in cases {
        for (column, metric) in ["open", "read", "total"].iter().enumerate() {
            let mut ratios = Vec::new();
            let mut pooled = [Vec::new(), Vec::new()];
            for pair in pairs.values() {
                assert!(!pair[0].is_empty() && pair[0].len() == pair[1].len());
                let mut times = pair
                    .each_ref()
                    .map(|rows| rows.iter().map(|r| r[column]).collect::<Vec<_>>());
                for variant in 0..2 {
                    pooled[variant].extend_from_slice(&times[variant]);
                }
                ratios.push(median(&mut times[1]) / median(&mut times[0]));
            }
            assert!(!ratios.is_empty());
            let mut seed = 31917u32;
            let mut estimates = Vec::with_capacity(5000);
            for _ in 0..5000 {
                let mut sample = Vec::with_capacity(ratios.len());
                for _ in 0..ratios.len() {
                    seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    sample.push(ratios[seed as usize % ratios.len()]);
                }
                estimates.push(median(&mut sample));
            }
            estimates.sort_by(f64::total_cmp);
            let a = median(&mut pooled[0]);
            let b = median(&mut pooled[1]);
            let change = (median(&mut ratios) - 1.0) * 100.0;
            let low = (estimates[125] - 1.0) * 100.0;
            let high = (estimates[4875] - 1.0) * 100.0;
            println!(
                "{case},{metric},{a:.3},{b:.3},{change:.3},{low:.3},{high:.3},{}",
                pairs.len()
            );
        }
    }
}

fn median(values: &mut [f64]) -> f64 {
    assert!(!values.is_empty());
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}
