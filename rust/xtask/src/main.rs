mod command;
mod compression;
mod install;
mod quality;
mod sleep;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(task) = args.next() else {
        print_help();
        return Ok(());
    };
    let args: Vec<String> = args.collect();

    match task.as_str() {
        "check-required" => no_args(&task, &args, quality::check_required),
        "clippy-advisory" => no_args(&task, &args, quality::clippy_advisory),
        "generate-api-docs" => no_args(&task, &args, quality::generate_api_docs),
        "install-cli" => no_args(&task, &args, install::cli),
        "run-network-tests" => no_args(&task, &args, quality::run_network_tests),
        "measure-key-server-performance" => quality::measure_key_server_performance(&args),
        "compare-archive-compression" => compression::run(&args),
        "agent-sleep-unix" => sleep::unix::run(&args),
        "agent-sleep-windows-host" => sleep::windows_host::run(&args),
        "agent-sleep-windows-setup" => sleep::windows_setup::run(&args),
        "agent-sleep-windows-vm" => sleep::windows_vm::run(&args),
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        _ => Err(format!("unknown task {task:?}; run `cargo xtask help`")),
    }
}

fn no_args(task: &str, args: &[String], action: fn() -> Result<(), String>) -> Result<(), String> {
    if args.is_empty() {
        action()
    } else if args.len() == 1 && matches!(args[0].as_str(), "-h" | "--help") {
        println!("Usage: cargo xtask {task}");
        Ok(())
    } else {
        Err(format!("{task} does not accept arguments"))
    }
}

fn print_help() {
    println!(
        "\
reVault workspace tasks

Usage: cargo xtask <task> [options]

Tasks:
  check-required                 Run formatting, hard Clippy, and required tests
  clippy-advisory                Run the advisory Clippy lint groups
  generate-api-docs              Generate revault_lockbox_api documentation
  install-cli                    Install local lockbox and lbx executables
  run-network-tests              Run ignored network integration tests
  measure-key-server-performance Run and capture the heavy failover benchmark
  compare-archive-compression    Compare lockbox compression with other tools
  agent-sleep-unix               Exercise agent key clearing across Unix sleep
  agent-sleep-windows-host       Drive the headless Windows libvirt sleep test
  agent-sleep-windows-setup      Start the visible Windows setup domain
  agent-sleep-windows-vm         Run the sleep test inside a Windows VM"
    );
}
