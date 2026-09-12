mod e2e;
mod package_conformance;
mod publication;
mod release;
mod release_candidate;
mod release_status;
mod repository;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Parser)]
#[command(name = "revault-tool", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Prepare, publish and inspect releases.
    Release {
        #[command(subcommand)]
        command: release::ReleaseCommand,
    },
    /// Run development test suites.
    Test {
        #[command(subcommand)]
        command: TestCommand,
    },
    /// Run repository development and validation tasks.
    Dev {
        #[command(subcommand)]
        command: DevCommand,
    },
    /// Generate or inspect language bindings.
    Bindings {
        #[command(subcommand)]
        command: repository::BindingsCommand,
    },
    #[command(hide = true)]
    Internal {
        #[command(subcommand)]
        command: InternalCommand,
    },
}

#[derive(Subcommand)]
enum DevCommand {
    /// Run formatting, hard Clippy, and required tests.
    CheckRequired,
    /// Run advisory Clippy lint groups.
    ClippyAdvisory,
    /// Generate Rust API documentation.
    GenerateApiDocs,
    /// Install the CLI and historical migration executables.
    InstallCli,
    /// Run realistic CLI journeys and command coverage checks.
    TestCliE2e,
    /// Build portable CLI binaries in Docker.
    BuildCli {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run ignored network integration tests.
    RunNetworkTests,
    /// Run the key-server performance benchmark.
    MeasureKeyServerPerformance {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Compare lockbox compression with other tools.
    CompareArchiveCompression {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Upgrade dependencies and run validation.
    UpgradeDeps,
    /// Exercise agent key clearing across Unix sleep.
    AgentSleepUnix {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Drive the headless Windows libvirt sleep test.
    AgentSleepWindowsHost {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Start the visible Windows setup domain.
    AgentSleepWindowsSetup {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run the sleep test inside a Windows VM.
    AgentSleepWindowsVm {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
enum TestCommand {
    /// Check binding surfaces and run Linux package conformance and interoperability.
    Bindings(e2e::Matrix),
    /// Run realistic CLI end-to-end journeys.
    Cli,
}

#[derive(Subcommand)]
enum InternalCommand {
    #[command(flatten)]
    Release(release::InternalCommand),
    E2e {
        #[command(subcommand)]
        command: e2e::E2eCommand,
    },
    CheckBindings(repository::Check),
    PublishCrates {
        #[arg(value_enum)]
        target: release_candidate::Scope,
        #[arg(long)]
        publish: bool,
        #[arg(long, default_value = ".")]
        repository: PathBuf,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("revault-tool: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result {
    let cli = Cli::parse();
    match cli.command {
        Command::Release { command } => release::run(command),
        Command::Bindings { command } => repository::run(command),
        Command::Test { command } => match command {
            TestCommand::Bindings(args) => {
                repository::check(std::path::Path::new("."))?;
                repository::verify_generated(std::path::Path::new("."))?;
                e2e::run(e2e::E2eCommand::Matrix(args))
            }
            TestCommand::Cli => {
                let status = std::process::Command::new("cargo")
                    .args([
                        "run",
                        "--locked",
                        "--manifest-path",
                        "rust/Cargo.toml",
                        "-p",
                        "xtask",
                        "--",
                        "test-cli-e2e",
                    ])
                    .status()?;
                if !status.success() {
                    return Err("CLI tests failed".into());
                }
                Ok(())
            }
        },
        Command::Dev { command } => run_dev(command),
        Command::Internal { command } => match command {
            InternalCommand::Release(command) => release::run_internal(command),
            InternalCommand::E2e { command } => e2e::run(command),
            InternalCommand::CheckBindings(args) => repository::check(&args.repository),
            InternalCommand::PublishCrates {
                target,
                publish,
                repository,
            } => match target {
                release_candidate::Scope::Cli => release::publish_cli(&repository, publish),
                release_candidate::Scope::Bindings => {
                    release::publish_bindings(&repository, publish)
                }
                release_candidate::Scope::All => Err("Select cli or bindings".into()),
            },
        },
    }
}

fn run_dev(command: DevCommand) -> Result {
    let (task, args): (&str, Vec<String>) = match command {
        DevCommand::CheckRequired => ("check-required", Vec::new()),
        DevCommand::ClippyAdvisory => ("clippy-advisory", Vec::new()),
        DevCommand::GenerateApiDocs => ("generate-api-docs", Vec::new()),
        DevCommand::InstallCli => ("install-cli", Vec::new()),
        DevCommand::TestCliE2e => ("test-cli-e2e", Vec::new()),
        DevCommand::BuildCli { args } => ("build-cli", args),
        DevCommand::RunNetworkTests => ("run-network-tests", Vec::new()),
        DevCommand::MeasureKeyServerPerformance { args } => {
            ("measure-key-server-performance", args)
        }
        DevCommand::CompareArchiveCompression { args } => ("compare-archive-compression", args),
        DevCommand::UpgradeDeps => ("upgrade-deps", Vec::new()),
        DevCommand::AgentSleepUnix { args } => ("agent-sleep-unix", args),
        DevCommand::AgentSleepWindowsHost { args } => ("agent-sleep-windows-host", args),
        DevCommand::AgentSleepWindowsSetup { args } => ("agent-sleep-windows-setup", args),
        DevCommand::AgentSleepWindowsVm { args } => ("agent-sleep-windows-vm", args),
    };
    let repository = find_repository_root()?;
    let mut cargo = ProcessCommand::new("cargo");
    cargo.current_dir(&repository).args([
        "run",
        "--locked",
        "--manifest-path",
        "rust/Cargo.toml",
        "-p",
        "xtask",
        "--",
        "internal-dev",
        task,
    ]);
    cargo.args(args);
    let status = cargo.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("development task `{task}` exited with {status}").into())
    }
}

fn find_repository_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;
    loop {
        if current.join("rust/Cargo.toml").is_file() {
            return Ok(current);
        }
        if !current.pop() {
            return Err(
                "run revault-tool dev commands from the repository or a child directory".into(),
            );
        }
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;
    #[test]
    fn public_command_structure() {
        for args in [
            vec!["release", "prepare"],
            vec!["release", "prepare", "cli"],
            vec!["release", "prepare", "bindings"],
            vec!["release", "prepare", "all"],
            vec!["release", "publish"],
            vec!["release", "status", "--watch"],
            vec!["release", "logs"],
            vec!["bindings", "generate"],
            vec!["bindings", "status"],
            vec!["test", "cli"],
            vec!["test", "bindings"],
            vec!["dev", "check-required"],
            vec!["dev", "build-cli"],
        ] {
            assert!(Cli::try_parse_from(std::iter::once("revault-tool").chain(args)).is_ok());
        }
        for args in [
            vec!["publish-cli"],
            vec!["publish-bindings"],
            vec!["publish-migration"],
            vec!["release", "prepare", "--scope", "cli"],
            vec!["bindings", "check"],
            vec!["e2e", "matrix"],
            vec!["dev", "install-tool"],
        ] {
            assert!(Cli::try_parse_from(std::iter::once("revault-tool").chain(args)).is_err());
        }
    }
}
