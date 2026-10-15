//! Cost-only experiment: payload I/O, signatures and publication excluded.
#[allow(dead_code)]
#[path = "../src/file_format/recovery_commitment.rs"]
mod commitment;
#[cfg(target_os = "linux")]
mod measurement {
    use super::commitment::{flat_digest, Context, Object, Proof, Tree};
    use serde_json::{json, Value};
    use sha2::{Digest, Sha256};
    use std::{hint::black_box, process::Command, time::Instant};
    fn resources() -> (f64, i64) {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
        // SAFETY: successful getrusage initializes the provided structure.
        assert_eq!(
            unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) },
            0
        );
        let usage = unsafe { usage.assume_init() };
        let seconds = |t: libc::timeval| t.tv_sec as f64 + t.tv_usec as f64 / 1e6;
        (
            seconds(usage.ru_utime) + seconds(usage.ru_stime),
            usage.ru_maxrss,
        )
    }
    fn sample(count: usize, variant: &str, operation: &str) -> Value {
        let context = Context {
            archive: [17; 16],
            sequence: 42,
            format: 3,
        };
        let mut objects: Vec<_> = (0..count)
            .map(|index| Object {
                namespace: 1,
                key: format!("/entry-{index:06}").into_bytes(),
                metadata: vec![index as u8; 128],
                logical_len: 4096,
                content: Sha256::digest(index.to_le_bytes()).into(),
            })
            .collect();
        let index = count / 2;
        let old = objects[index].clone();
        let mut replacement = old.clone();
        replacement.metadata[0] ^= 1;
        let baseline_flat = flat_digest(&context, &objects).unwrap();
        let mut tree = if variant == "tree" && operation != "build" {
            Some(Tree::build(context.clone(), &objects).unwrap())
        } else {
            None
        };
        let proof = tree
            .as_ref()
            .map(|tree| Proof::decode(&tree.proof(index).unwrap().encode().unwrap()).unwrap());
        let iterations = if variant == "tree" && operation != "build" {
            1000
        } else {
            1
        };
        let before = resources();
        let started = Instant::now();
        let mut flat = baseline_flat;
        for iteration in 0..iterations {
            match (variant, operation) {
                ("flat", "build" | "verify") => {
                    flat = black_box(flat_digest(&context, black_box(&objects)).unwrap())
                }
                ("flat", "replace") => {
                    objects[index] = replacement.clone();
                    flat = black_box(flat_digest(&context, black_box(&objects)).unwrap());
                }
                ("tree", "build") => {
                    tree = Some(black_box(
                        Tree::build(context.clone(), black_box(&objects)).unwrap(),
                    ))
                }
                ("tree", "verify") => tree
                    .as_ref()
                    .unwrap()
                    .root()
                    .verify(black_box(&old), black_box(proof.as_ref().unwrap()))
                    .unwrap(),
                ("tree", "replace") => {
                    let (previous, next) = if iteration % 2 == 0 {
                        (&old, &replacement)
                    } else {
                        (&replacement, &old)
                    };
                    tree.as_mut()
                        .unwrap()
                        .replace(index, black_box(previous), black_box(next))
                        .unwrap();
                }
                _ => unreachable!(),
            }
        }
        let elapsed = started.elapsed().as_secs_f64();
        let after = resources();
        let retained_hash_bytes = tree.as_ref().map_or(0, Tree::retained_hash_bytes);
        let proof_bytes = tree
            .as_ref()
            .map_or(0, |tree| tree.proof(index).unwrap().encode().unwrap().len());
        if let Some(tree) = &tree {
            assert_eq!(
                tree.root(),
                Tree::build(context.clone(), &objects).unwrap().root()
            );
            tree.root()
                .verify(&objects[index], &tree.proof(index).unwrap())
                .unwrap();
        } else {
            assert_eq!(flat, flat_digest(&context, &objects).unwrap());
        }
        json!({"kind":"sample","count":count,"variant":variant,"operation":operation,
            "iterations":iterations,"seconds_per_operation":elapsed/iterations as f64,
            "cpu_seconds_per_operation":(after.0-before.0)/iterations as f64,
            "peak_rss_kib":after.1,"baseline_peak_rss_kib":before.1,
            "retained_hash_bytes":retained_hash_bytes,"proof_bytes":proof_bytes,"verified":true})
    }
    fn interval(ratios: &[f64]) -> (f64, f64, f64) {
        assert!(!ratios.is_empty() && ratios.iter().all(|r| r.is_finite() && *r > 0.0));
        let logs: Vec<_> = ratios.iter().map(|r| r.ln()).collect();
        let average = (logs.iter().sum::<f64>() / logs.len() as f64).exp();
        let mut state = 0x7a12_8643_6bde_910fu64;
        let mut samples = Vec::with_capacity(10_000);
        for _ in 0..10_000 {
            let mut sum = 0.0;
            for _ in &logs {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                sum += logs[(state % logs.len() as u64) as usize];
            }
            samples.push((sum / logs.len() as f64).exp());
        }
        samples.sort_by(f64::total_cmp);
        (average, samples[249], samples[9749])
    }
    fn summarize(path: &str) {
        let input = std::fs::read_to_string(path).unwrap();
        let rows: Vec<Value> = input
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let expected = rows[0]["pairs"].as_u64().unwrap() as usize;
        for operation in ["build", "replace", "verify"] {
            let mut pairs = std::collections::BTreeMap::new();
            for row in &rows {
                if row["kind"] != "sample" || row["warmup"] == true || row["operation"] != operation
                {
                    continue;
                }
                assert_eq!(row["verified"], true);
                let pair = pairs
                    .entry(row["pair"].as_u64().unwrap())
                    .or_insert([None, None]);
                let index = match row["variant"].as_str().unwrap() {
                    "flat" => 0,
                    "tree" => 1,
                    _ => panic!("unknown variant"),
                };
                assert!(pair[index].replace(row).is_none(), "duplicate sample");
            }
            assert_eq!(pairs.len(), expected);
            for metric in [
                "seconds_per_operation",
                "cpu_seconds_per_operation",
                "peak_rss_kib",
            ] {
                let mut flat = Vec::new();
                let mut tree = Vec::new();
                let mut ratios = Vec::new();
                for pair in pairs.values() {
                    let a = pair[0].expect("missing flat")[metric].as_f64().unwrap();
                    let b = pair[1].expect("missing tree")[metric].as_f64().unwrap();
                    assert!(a > 0.0 && b > 0.0);
                    flat.push(a);
                    tree.push(b);
                    ratios.push(b / a);
                }
                let (ratio, low, high) = interval(&ratios);
                flat.sort_by(f64::total_cmp);
                tree.sort_by(f64::total_cmp);
                println!(
                    "{}",
                    json!({"operation":operation,"metric":metric,"pairs":expected,
                    "count":rows[0]["count"],"flat_median":flat[expected/2],"tree_median":tree[expected/2],
                    "tree_over_flat":ratio,"low95":low,"high95":high})
                );
            }
        }
    }
    pub fn main() {
        let args: Vec<_> = std::env::args().skip(1).collect();
        if args.first().is_some_and(|arg| arg == "summarize") {
            assert_eq!(args.len(), 2);
            summarize(&args[1]);
            return;
        }
        if args.first().is_some_and(|arg| arg == "sample") {
            assert_eq!(args.len(), 4);
            let count = args[1].parse().unwrap();
            assert!((1..=100_000).contains(&count));
            println!("{}", sample(count, &args[2], &args[3]));
            return;
        }
        assert_eq!(args.len(), 2, "usage: recovery_commitment COUNT PAIRS");
        let count: usize = args[0].parse().unwrap();
        let pairs: usize = args[1].parse().unwrap();
        assert!((1..=100_000).contains(&count) && (1..=1000).contains(&pairs));
        let executable = std::env::current_exe().unwrap();
        let status = std::fs::read_to_string("/proc/self/status").unwrap();
        let revision = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .unwrap();
        let hex = |bytes: &[u8]| {
            bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        };
        println!(
            "{}",
            json!({"kind":"environment","count":count,"pairs":pairs,"warmup_pairs":3,
            "scope":"hash/proof only; inputs resident; no payload I/O, owner signatures, publication or recovery traversal",
            "metadata_bytes_per_object":128,"runner_sha256":hex(&Sha256::digest(include_bytes!("recovery_commitment.rs"))),
            "prototype_sha256":hex(&Sha256::digest(include_bytes!("../src/file_format/recovery_commitment.rs"))),
            "executable_sha256":hex(&Sha256::digest(std::fs::read(&executable).unwrap())),
            "source_revision_at_run":String::from_utf8_lossy(&revision.stdout).trim(),
            "cargo_lock_sha256":hex(&Sha256::digest(include_bytes!("../../Cargo.lock"))),
            "cpuinfo":std::fs::read_to_string("/proc/cpuinfo").unwrap().lines().find(|line| line.starts_with("model name")).unwrap(),
            "cpu_affinity":status.lines().find(|line| line.starts_with("Cpus_allowed_list:")).unwrap(),
            "kernel":std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap(),
            "loadavg":std::fs::read_to_string("/proc/loadavg").unwrap()})
        );
        for pair in 0..pairs + 3 {
            for operation in ["build", "replace", "verify"] {
                let order = if pair % 2 == 0 {
                    ["flat", "tree"]
                } else {
                    ["tree", "flat"]
                };
                for variant in order {
                    let output = Command::new(&executable)
                        .args(["sample", &count.to_string(), variant, operation])
                        .output()
                        .unwrap();
                    assert!(
                        output.status.success(),
                        "{}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    let mut record: Value = serde_json::from_slice(&output.stdout).unwrap();
                    record["pair"] = json!(pair);
                    record["warmup"] = json!(pair < 3);
                    println!("{record}");
                }
            }
        }
    }
    #[cfg(test)]
    mod tests {
        #[test]
        fn known_pair_ratios_are_exact_and_reproducible() {
            let result = super::interval(&[2.0; 30]);
            assert!(
                (result.0 - 2.0).abs() < 1e-12
                    && (result.1 - 2.0).abs() < 1e-12
                    && (result.2 - 2.0).abs() < 1e-12
            );
            assert_eq!(
                super::interval(&[0.5, 1.0, 2.0]),
                super::interval(&[0.5, 1.0, 2.0])
            );
        }
    }
}
fn main() {
    #[cfg(target_os = "linux")]
    measurement::main();
    #[cfg(not(target_os = "linux"))]
    panic!("Linux resource accounting required");
}
