use std::{hint::black_box, time::Instant};
use zstd_complete::{
    decoding::{DecoderWorkspace, FrameDecoder},
    encoding::{compress_to_vec, CompressionLevel},
};

fn main() {
    println!("bytes,corpus,compressed_bytes,mode,median_us");
    for size in [4096usize, 1048576, 4194304] {
        for corpus in ["pattern", "random"] {
            let input: Vec<u8> = (0..size)
                .map(|i| {
                    if corpus == "pattern" {
                        ((i * 13 + i / 251) % 251) as u8
                    } else {
                        let mut x = (i as u64).wrapping_add(0x9e3779b97f4a7c15);
                        x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                        x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
                        (x ^ (x >> 31)) as u8
                    }
                })
                .collect();
            let compressed = compress_to_vec(input.as_slice(), CompressionLevel::Fastest);
            let mut frame = FrameDecoder::new();
            let mut workspace = DecoderWorkspace::new(4 * 1024 * 1024, 0).unwrap();
            let mut times = [Vec::new(), Vec::new(), Vec::new()];
            // Rotate order; exclude warmup. Allocate output equally in all modes.
            for round in 0..110 {
                for j in 0..3 {
                    let mode = (round + j) % 3;
                    let start = Instant::now();
                    let mut output = Vec::with_capacity(size);
                    match mode {
                        0 => FrameDecoder::new()
                            .decode_all_to_vec(black_box(&compressed), &mut output)
                            .unwrap(),
                        1 => frame
                            .decode_all_to_vec(black_box(&compressed), &mut output)
                            .unwrap(),
                        _ => {
                            output.resize(size, 0);
                            let n = workspace
                                .decode_into(black_box(&compressed), &mut output)
                                .unwrap();
                            assert_eq!(n, size);
                        }
                    }
                    black_box(&output);
                    let elapsed = start.elapsed().as_secs_f64() * 1e6;
                    assert_eq!(output, input);
                    if round >= 10 {
                        times[mode].push(elapsed);
                    }
                }
            }
            for (mode, samples) in times.iter_mut().enumerate() {
                samples.sort_by(f64::total_cmp);
                println!(
                    "{size},{corpus},{},{},{:.3}",
                    compressed.len(),
                    ["fresh", "reused-frame", "workspace"][mode],
                    samples[samples.len() / 2]
                );
            }
        }
    }
}
