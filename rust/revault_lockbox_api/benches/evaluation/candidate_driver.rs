//! Compile with rustc --edition=2021. Adapts test-only C to the common runner.
use std::process::Command;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert!((args.len() == 3 && args[0] == "create") || (args.len() == 5 && args[0] == "sample"));
    assert_eq!(args[2], "lockbox");
    let binary = std::env::var_os("REVAULT_CANDIDATE_TEST_BINARY").expect("set test executable");
    let mut command = Command::new(binary);
    command
        .args([
            "--exact",
            "file_format::candidate_files::resource_probe::candidate_file_resource_probe",
            "--ignored",
            "--nocapture",
        ])
        .env("REVAULT_CANDIDATE_ROOT", &args[1])
        .env("REVAULT_CANDIDATE_PHASE", &args[0]);
    if args[0] == "sample" {
        command
            .env("REVAULT_CANDIDATE_ACCESS", &args[3])
            .env("REVAULT_CANDIDATE_PASSES", &args[4]);
    }
    let output = command.output().expect("start candidate child");
    assert!(
        output.status.success(),
        "candidate failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let records: Vec<_> = stdout
        .lines()
        .filter_map(|line| line.split_once("CANDIDATE_SAMPLE ").map(|(_, json)| json))
        .collect();
    assert_eq!(records.len(), 1, "candidate must emit one sample");
    println!("{}", records[0]);
}
