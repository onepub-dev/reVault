//! Whole-archive evaluation with isolated samples and CPU/peak-RSS evidence.
//! See docs/archive_v4_evaluation.md. Build before timing; run the executable.
#[cfg(target_os = "linux")]
#[path = "evaluation/runner.rs"]
mod runner;

fn main() {
    #[cfg(target_os = "linux")]
    runner::main();
    #[cfg(not(target_os = "linux"))]
    panic!("archive_evaluation currently requires Linux resource accounting");
}
