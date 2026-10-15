//! Encoder-only experiment, not a CLI E2E or a production selection policy.
//! Every output is independently decoded and byte-checked outside encoding time.
use std::time::Instant;
use zeroize::Zeroize;
use zstd_complete::{
    decoding::FrameDecoder,
    encoding::{
        compress_slice_c_level, compress_to_vec, encode_all, CompressionLevel, CompressionStrategy,
        CompressionTuning, EncoderOptions, LongDistanceMatching,
    },
};

fn fast(input: &[u8]) -> Vec<u8> {
    compress_to_vec(input, CompressionLevel::Fastest)
}

fn encode(input: &[u8], level: i32, strategy: usize) -> (Vec<u8>, bool) {
    match strategy {
        0 => (compress_slice_c_level(input, level), false),
        1 => (fast(input), true),
        2 => {
            let mut numeric = compress_slice_c_level(input, level);
            let mut alternative = fast(input);
            let selected_fast = alternative.len() < numeric.len();
            if selected_fast {
                std::mem::swap(&mut numeric, &mut alternative);
            }
            alternative.zeroize();
            (numeric, selected_fast)
        }
        3 => {
            // Three separated samples reduce prefix-only bias. Deliberately
            // require a substantial win before changing the numeric encoder.
            let len = input.len().min(16 * 1024);
            let mut numeric_bytes = 0;
            let mut fast_bytes = 0;
            for offset in [0, (input.len() - len) / 2, input.len() - len] {
                let sample = &input[offset..offset + len];
                let mut numeric = compress_slice_c_level(sample, level);
                let mut alternative = fast(sample);
                numeric_bytes += numeric.len();
                fast_bytes += alternative.len();
                numeric.zeroize();
                alternative.zeroize();
            }
            let selected_fast = fast_bytes * 4 < numeric_bytes * 3;
            (
                if selected_fast {
                    fast(input)
                } else {
                    compress_slice_c_level(input, level)
                },
                selected_fast,
            )
        }
        4 => (compress_to_vec(input, CompressionLevel::Default), true),
        5 => (compress_to_vec(input, CompressionLevel::Better), true),
        6..=10 => {
            let tuning = match strategy {
                6 => CompressionTuning::new(),
                7 => CompressionTuning::new().with_min_match(7),
                8 => CompressionTuning::new()
                    .with_strategy(CompressionStrategy::Lazy2)
                    .with_search_log(5)
                    .with_min_match(5),
                9 => CompressionTuning::new()
                    .with_long_distance_matching(LongDistanceMatching::new().with_min_match(64)),
                10 => CompressionTuning::new()
                    .with_strategy(CompressionStrategy::DoubleFast)
                    .with_hash_log(20)
                    .with_min_match(7),
                _ => unreachable!(),
            };
            (
                encode_all(
                    input,
                    EncoderOptions::new(CompressionLevel::try_from(level).unwrap())
                        .with_tuning(tuning),
                )
                .unwrap(),
                false,
            )
        }
        _ => unreachable!(),
    }
}

fn payload(kind: &str, size: usize) -> Vec<u8> {
    let text = b"{\"event\":\"read\",\"path\":\"/documents/report.txt\",\"status\":\"ok\"}\n";
    (0..size)
        .map(|n| {
            let mut x = (n as u64).wrapping_add(0x9e3779b97f4a7c15);
            x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
            let random = (x ^ (x >> 31)) as u8;
            let pattern = ((n * 13 + n / 251) % 251) as u8;
            match kind {
                "pattern" => pattern,
                "zero" => 0,
                "text" => text[n % text.len()],
                "random" => random,
                "mixed" => {
                    if (n / 4096) % 2 == 0 {
                        pattern
                    } else {
                        random
                    }
                }
                "pattern-prefix" => {
                    if n < size / 4 {
                        pattern
                    } else {
                        random
                    }
                }
                "random-prefix" => {
                    if n < size / 4 {
                        random
                    } else {
                        pattern
                    }
                }
                _ => unreachable!(),
            }
        })
        .collect()
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(args.len(), 3, "encoder_selection SIZE LEVEL REPEATS");
    let size: usize = args[0].parse().unwrap();
    let level: i32 = args[1].parse().unwrap();
    let repeats: usize = args[2].parse().unwrap();
    assert!(size >= 16384 && repeats > 0);
    println!("corpus,size,level,repeat,strategy,fast_selected,stored_bytes,encode_us,decode_us");
    for kind in [
        "pattern",
        "zero",
        "text",
        "random",
        "mixed",
        "pattern-prefix",
        "random-prefix",
    ] {
        let input = payload(kind, size);
        for repeat in 0..=repeats {
            for step in 0..11 {
                // Rotate execution order; repeat zero is a discarded warmup.
                let strategy = (step + repeat) % 11;
                let start = Instant::now();
                let (mut encoded, selected_fast) = encode(&input, level, strategy);
                let encode_us = start.elapsed().as_secs_f64() * 1e6;
                let mut decoded = Vec::with_capacity(size);
                let start = Instant::now();
                FrameDecoder::new()
                    .decode_all_to_vec(&encoded, &mut decoded)
                    .unwrap();
                let decode_us = start.elapsed().as_secs_f64() * 1e6;
                assert_eq!(decoded, input);
                if repeat > 0 {
                    let name = [
                        "numeric",
                        "fast",
                        "dual",
                        "sampled",
                        "stream-default",
                        "stream-better",
                        "numeric-buffered",
                        "numeric-min7",
                        "numeric-lazy2",
                        "numeric-ldm",
                        "numeric-hash20",
                    ][strategy];
                    println!("{kind},{size},{level},{repeat},{name},{selected_fast},{},{encode_us:.3},{decode_us:.3}", encoded.len());
                }
                encoded.zeroize();
                decoded.zeroize();
            }
        }
    }
}
