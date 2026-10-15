use std::{fs, process::Command};
fn main() {
    let native = "target/issue310-stream-after";
    let default = "target/issue310-stream-before";
    for (label, corpus, pairs, samples, case, rows) in [
        ("pattern", "pattern", "5", "3", "all", 320),
        ("random", "random", "5", "3", "all", 320),
        (
            "signed-focused",
            "pattern",
            "15",
            "6",
            "encrypted-signed/true/write",
            300,
        ),
    ] {
        let root = format!("target/issue310-stream-{label}");
        fs::create_dir(&root).unwrap();
        let result = Command::new("taskset")
            .args([
                "-c",
                "2",
                native,
                "compare-write",
                default,
                native,
                &root,
                "8388608",
                corpus,
                pairs,
                case,
            ])
            .env("REVAULT_GATE_STREAM_INPUT", "1")
            .env("REVAULT_GATE_CHILD_SAMPLES", samples)
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
        assert_eq!(csv.lines().count(), rows + 1);
        let path = format!("revault_lockbox_api/benches/results/issue310-stream-{label}.csv");
        fs::write(&path, csv).unwrap();
        let summary = Command::new(native)
            .args(["summarize", &path])
            .output()
            .unwrap();
        assert!(summary.status.success());
        fs::write(
            format!("revault_lockbox_api/benches/results/issue310-stream-{label}-summary.csv"),
            &summary.stdout,
        )
        .unwrap();
        println!("{label}: {}", String::from_utf8_lossy(&summary.stdout));
    }
}
