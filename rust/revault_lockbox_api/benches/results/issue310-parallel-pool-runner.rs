use std::{fs, process::Command};

fn main() {
    let before = "target/issue310-parallel-pool-before";
    let after = "target/issue310-parallel-pool-after";
    for (label, size) in [("8m", "8388608"), ("32m", "33554432")] {
        let root = format!("target/issue310-parallel-pool-{label}");
        fs::create_dir(&root).unwrap();
        let result = Command::new("taskset")
            .args([
                "-c",
                "2-5",
                after,
                "compare-write",
                before,
                after,
                &root,
                size,
                "pattern",
                "15",
                "encrypted/true/write",
            ])
            .env("REVAULT_GATE_STREAM_INPUT", "1")
            .env("REVAULT_GATE_WORKERS", "4")
            .env("REVAULT_GATE_CHILD_SAMPLES", "6")
            .env_remove("REVAULT_GATE_WRITE_PROFILE")
            .env_remove("REVAULT_GATE_NO_SIZE_PADDING")
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let csv = String::from_utf8(result.stdout).unwrap();
        assert_eq!(csv.lines().count(), 301);
        let path =
            format!("revault_lockbox_api/benches/results/issue310-parallel-pool-{label}.csv");
        fs::write(&path, &csv).unwrap();
        let summary = Command::new(after)
            .args(["summarize", &path])
            .output()
            .unwrap();
        assert!(summary.status.success());
        fs::write(
            format!(
                "revault_lockbox_api/benches/results/issue310-parallel-pool-{label}-summary.csv"
            ),
            &summary.stdout,
        )
        .unwrap();
        println!("{label}: {}", String::from_utf8_lossy(&summary.stdout));
    }
}
