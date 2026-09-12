mod build;
mod command;
mod compression;
mod dependencies;
mod e2e;
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
        "dev-tools" => install::dev_tools(&args),
        "internal-dev" => internal_dev(&args),
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        _ => Err(format!("unknown task {task:?}; run `cargo xtask help`")),
    }
}

fn internal_dev(args: &[String]) -> Result<(), String> {
    let Some((task, rest)) = args.split_first() else {
        return Err("internal-dev requires a development task".to_owned());
    };
    match task.as_str() {
        "check-required" => no_args(task, rest, quality::check_required),
        "clippy-advisory" => no_args(task, rest, quality::clippy_advisory),
        "generate-api-docs" => no_args(task, rest, quality::generate_api_docs),
        "install-cli" => no_args(task, rest, install::cli),
        "test-cli-e2e" => no_args(task, rest, e2e::cli),
        "build-cli" => build::cli(rest),
        "run-network-tests" => no_args(task, rest, quality::run_network_tests),
        "measure-key-server-performance" => quality::measure_key_server_performance(rest),
        "compare-archive-compression" => compression::run(rest),
        "upgrade-deps" => no_args(task, rest, dependencies::upgrade),
        "agent-sleep-unix" => sleep::unix::run(rest),
        "agent-sleep-windows-host" => sleep::windows_host::run(rest),
        "agent-sleep-windows-setup" => sleep::windows_setup::run(rest),
        "agent-sleep-windows-vm" => sleep::windows_vm::run(rest),
        other => Err(format!("unknown development task {other:?}")),
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
  dev-tools                      Build and install revault-tool, the developer command hub
  Use `revault-tool dev --help` for repository validation, testing, build,
  dependency, performance, compression, and agent-sleep tasks."
    );
}
