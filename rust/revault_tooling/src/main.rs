mod e2e;
mod package_conformance;
mod publication;
mod release;
mod release_candidate;
mod release_status;
mod repository;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
        ] {
            assert!(Cli::try_parse_from(std::iter::once("revault-tool").chain(args)).is_err());
        }
    }
}
