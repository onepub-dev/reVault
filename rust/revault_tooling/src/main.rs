mod e2e;
mod package_conformance;
mod publication;
mod release;
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
    /// Build, verify, stage, and assemble release artifacts.
    Release {
        #[command(subcommand)]
        command: release::ReleaseCommand,
    },
    /// Run and verify binding conformance suites.
    E2e {
        #[command(subcommand)]
        command: e2e::E2eCommand,
    },
    /// Generate or check all language binding surfaces.
    Bindings {
        #[command(subcommand)]
        command: repository::BindingsCommand,
    },
    /// Publish the CLI and its workspace dependencies in dependency order.
    #[command(name = "publish-cli", visible_alias = "publish-migration")]
    PublishCli {
        #[arg(long, conflicts_with = "publish")]
        dry_run: bool,
        #[arg(long, conflicts_with = "dry_run")]
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
        Command::E2e { command } => e2e::run(command),
        Command::Bindings { command } => repository::run(command),
        Command::PublishCli {
            dry_run,
            publish,
            repository,
        } => release::publish_cli(&repository, publish && !dry_run),
    }
}
