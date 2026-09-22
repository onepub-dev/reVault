use std::{fs, process::Command};
fn main() {
    let after = "target/issue310-pooled-encoder-after";
    let before = "target/issue310-wipe256-before";
    let root = "target/issue310-pooled-encoder-focused";
    fs::create_dir(root).unwrap();
    let r = Command::new("taskset")
        .args([
            "-c",
            "2",
            after,
            "compare-write",
            before,
            after,
            root,
            "8388608",
            "pattern",
            "15",
            "encrypted/true/write",
        ])
        .env("REVAULT_GATE_CHILD_SAMPLES", "6")
        .env_remove("REVAULT_GATE_WRITE_PROFILE")
        .env_remove("REVAULT_GATE_NO_SIZE_PADDING")
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let csv = String::from_utf8(r.stdout).unwrap();
    assert_eq!(csv.lines().count(), 301);
    let path = "revault_lockbox_api/benches/results/issue310-pooled-encoder-focused.csv";
    fs::write(path, csv).unwrap();
    let s = Command::new(after)
        .args(["summarize", path])
        .output()
        .unwrap();
    assert!(s.status.success());
    fs::write(
        "revault_lockbox_api/benches/results/issue310-pooled-encoder-focused-summary.csv",
        &s.stdout,
    )
    .unwrap();
    print!("{}", String::from_utf8_lossy(&s.stdout));
}
