use crate::command::{self, TaskResult};
use std::env;
use std::fs;
use std::process::Command;

const REQUIRED_PACKAGE_NAMES: &[&str] = &[
    "revault_page_api",
    "revault_lockbox_api",
    "revault_cli",
    "revault_vault_api",
    "revault_migration_format",
    "revault_migrate_vault_v1",
    "revault_migrate_archive_v1",
    "revault_migration",
    "revault_publish_protocol",
    "revault_key_server",
    "xtask",
];

const NON_PRODUCTION_PACKAGES: &[&str] = &[
    "revault_bindings",
    "revault_wasm_bindings",
    "revault_wire",
    "revault_tooling",
];

fn cargo_with_packages(subcommand: &str, packages: &[&str]) -> Command {
    let mut cargo = command::command("cargo");
    cargo.arg(subcommand);
    for package in packages {
        cargo.args(["-p", package]);
    }
    cargo
}

pub fn check_required() -> TaskResult {
    command::run(cargo_with_packages("fmt", REQUIRED_PACKAGE_NAMES).arg("--check"))?;
    let mut clippy = command::command("cargo");
    clippy.args(["clippy", "--workspace"]);
    for package in NON_PRODUCTION_PACKAGES {
        clippy.args(["--exclude", package]);
    }
    command::run(clippy.arg("--all-targets").args(["--", "-D", "warnings"]))?;

    test_package("revault_page_api")?;
    test_package("revault_lockbox_api")?;
    if env::var("RUNNER_OS").as_deref() == Ok("Windows") {
        command::run(command::command("cargo").args([
            "test",
            "--manifest-path",
            "revault_cli/Cargo.toml",
            "--test",
            "contact_receive_alias",
            "--test",
            "help_open_key",
            "--test",
            "publish_integration",
        ]))?;
    } else {
        test_package("revault_cli")?;
    }
    for package in &REQUIRED_PACKAGE_NAMES[3..] {
        test_package(package)?;
    }
    Ok(())
}

pub fn clippy_advisory() -> TaskResult {
    let mut clippy = command::command("cargo");
    clippy.args(["clippy", "--workspace"]);
    for package in [
        "revault_page_api",
        "revault_publish_protocol",
        "revault_key_server",
        "xtask",
    ]
    .iter()
    .chain(NON_PRODUCTION_PACKAGES.iter())
    {
        clippy.args(["--exclude", package]);
    }
    command::run(clippy.arg("--all-targets").args([
        "--",
        "-W",
        "clippy::pedantic",
        "-W",
        "clippy::nursery",
        "-W",
        "clippy::cargo",
        "-A",
        "clippy::redundant_pub_crate",
        "-A",
        "clippy::cargo_common_metadata",
        "-A",
        "clippy::multiple_crate_versions",
        "-A",
        "clippy::use_self",
    ]))
}

fn test_package(package: &str) -> TaskResult {
    let manifest = format!("{package}/Cargo.toml");
    command::run(command::command("cargo").args(["test", "--manifest-path", &manifest]))
}

pub fn generate_api_docs() -> TaskResult {
    command::run(command::command("cargo").args([
        "doc",
        "--manifest-path",
        "revault_lockbox_api/Cargo.toml",
        "--no-deps",
    ]))?;
    println!("Generated revault_lockbox_api API docs:");
    println!("  target/doc/revault_lockbox_api/index.html");
    Ok(())
}

pub fn run_network_tests() -> TaskResult {
    for args in [
        &[
            "test",
            "--manifest-path",
            "revault_key_server/Cargo.toml",
            "--test",
            "e2e_failover",
            "--",
            "--ignored",
            "--nocapture",
        ][..],
        &[
            "test",
            "--manifest-path",
            "revault_key_server/Cargo.toml",
            "--test",
            "protocol_store",
            "--",
            "--ignored",
            "--nocapture",
        ][..],
        &[
            "test",
            "--manifest-path",
            "revault_cli/Cargo.toml",
            "--test",
            "publish_integration",
            "--",
            "--ignored",
            "--nocapture",
        ][..],
    ] {
        command::run(command::command("cargo").args(args))?;
    }
    Ok(())
}

pub fn measure_key_server_performance(args: &[String]) -> TaskResult {
    let mut flows = env::var("LOCKBOX_SHARE_E2E_FLOWS").unwrap_or_else(|_| "50000".to_owned());
    let mut workers = env::var("LOCKBOX_SHARE_E2E_WORKERS").unwrap_or_else(|_| "128".to_owned());
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--flows" => flows = command::option_value(args, &mut index, "--flows")?,
            "--workers" => workers = command::option_value(args, &mut index, "--workers")?,
            "-h" | "--help" => {
                println!(
                    "Usage: cargo xtask measure-key-server-performance [--flows N] [--workers N]"
                );
                return Ok(());
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        index += 1;
    }

    let stamp = command::output_lossy(command::command("date").args(["-u", "+%Y%m%dT%H%M%SZ"]))?;
    let log_dir = command::workspace_root()?.join("target/perf");
    fs::create_dir_all(&log_dir)
        .map_err(|error| format!("cannot create {}: {error}", log_dir.display()))?;
    let log = log_dir.join(format!("key-server-heavy-failover-{stamp}.log"));

    let rustc = command::output_lossy(command::command("rustc").arg("--version"))?;
    let cargo_version = command::output_lossy(command::command("cargo").arg("--version"))?;
    let header = format!(
        "timestamp_utc={stamp}\nflows={flows}\nworkers={workers}\n{rustc}\n{cargo_version}\n"
    );
    fs::write(&log, &header)
        .map_err(|error| format!("cannot create {}: {error}", log.display()))?;
    print!("{header}");

    let mut cargo = command::command("cargo");
    cargo
        .env("LOCKBOX_SHARE_E2E_FLOWS", &flows)
        .env("LOCKBOX_SHARE_E2E_WORKERS", &workers)
        .args([
            "test",
            "--manifest-path",
            "revault_key_server/Cargo.toml",
            "--test",
            "e2e_failover",
            "heavy_failover_recovery_under_load",
            "--",
            "--ignored",
            "--nocapture",
        ]);
    command::run_and_tee(&mut cargo, &log)?;
    println!("performance_log={}", log.display());
    Ok(())
}
