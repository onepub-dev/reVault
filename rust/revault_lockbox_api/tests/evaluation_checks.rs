// Share the benchmark's deterministic statistics/corpus checks without running
// timed workloads under a test harness or requiring a prebuilt benchmark binary.
#![cfg(target_os = "linux")]
#[allow(dead_code)]
#[path = "../benches/evaluation/runner.rs"]
mod runner;
