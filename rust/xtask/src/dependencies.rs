use crate::command::{self, TaskResult};
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_RUST_VERSION: &str = "1.88";
const CARGO_EDIT_VERSION: &str = "0.13.7";

/// Upgrade every Cargo workspace and then validate the resulting dependency graph.
///
/// `cargo upgrade` is supplied by cargo-edit.  It is deliberately kept as an
/// external tool: Cargo itself can update a lockfile, but it cannot widen the
/// version requirements in Cargo.toml to include a new major release.
pub fn upgrade() -> TaskResult {
    command::require_commands(&["cargo", "rustc"])?;
    if !command::exists_on_path("cargo-upgrade") {
        println!("cargo-upgrade is not installed; installing cargo-edit {CARGO_EDIT_VERSION}");
        let mut install = command::command("cargo");
        install.args([
            "install",
            "cargo-edit",
            "--version",
            CARGO_EDIT_VERSION,
            "--locked",
        ]);
        command::run(&mut install)?;
        if !command::exists_on_path("cargo-upgrade") {
            return Err(
                "cargo-edit installation completed but cargo-upgrade is unavailable on PATH"
                    .to_owned(),
            );
        }
    }

    check_rust_version()?;

    let repo = command::repo_root()?;
    let workspaces = [
        repo.join("rust"),
        repo.join("rust/fuzz"),
        repo.join("bindings/rust"),
        repo.join("bindings/e2e/rust"),
    ];

    for workspace in &workspaces {
        upgrade_workspace(workspace)?;
    }

    check_rust_version()?;

    let rust_workspace = repo.join("rust");
    run_in(
        &rust_workspace,
        ["clippy", "--workspace", "--all-targets", "--all-features"],
    )?;
    run_in(&rust_workspace, ["test", "--workspace"])?;

    for workspace in &workspaces {
        run_in(workspace, ["tree"])?;
    }

    Ok(())
}

fn upgrade_workspace(workspace: &Path) -> TaskResult {
    println!("Upgrading dependencies in {}", workspace.display());
    let mut cargo = command::command("cargo");
    cargo
        .current_dir(workspace)
        .args(["upgrade", "--incompatible", "allow"]);
    command::run(&mut cargo)?;

    // Refresh transitive dependencies after Cargo.toml requirements have been
    // widened by cargo-upgrade.
    let mut update = command::command("cargo");
    update.current_dir(workspace).arg("update");
    command::run(&mut update)
}

fn run_in<const N: usize>(workspace: &Path, args: [&str; N]) -> TaskResult {
    let mut cargo = command::command("cargo");
    cargo.current_dir(workspace).args(args);
    command::run(&mut cargo)
}

fn check_rust_version() -> TaskResult {
    let repo = command::repo_root()?;
    let toolchain = fs::read_to_string(repo.join("rust-toolchain.toml"))
        .map_err(|error| format!("cannot read rust-toolchain.toml: {error}"))?;
    if !toolchain.contains("channel = \"1.88.0\"") {
        return Err("rust-toolchain.toml does not pin Rust 1.88.0".to_owned());
    }

    let manifests = cargo_manifests(&repo);
    for manifest in manifests {
        let contents = fs::read_to_string(&manifest)
            .map_err(|error| format!("cannot read {}: {error}", manifest.display()))?;
        if let Some(line) = contents
            .lines()
            .find(|line| line.starts_with("rust-version"))
        {
            let expected = format!("rust-version = \"{EXPECTED_RUST_VERSION}\"");
            if line.trim() != expected {
                return Err(format!(
                    "{} changes the Rust minimum version: {}",
                    manifest.display(),
                    line.trim()
                ));
            }
        }
    }

    let version = command::output_lossy(command::command("rustc").arg("--version"))?;
    if !version.contains("1.88") {
        return Err(format!(
            "dependency validation must run on Rust 1.88, found {version}"
        ));
    }
    Ok(())
}

fn cargo_manifests(repo: &Path) -> Vec<PathBuf> {
    [repo.join("rust"), repo.join("bindings")]
        .into_iter()
        .flat_map(|root| collect_manifests(&root))
        .collect()
}

fn collect_manifests(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .flat_map(|entry| {
            let path = entry.path();
            if path.file_name().is_some_and(|name| name == "target") {
                Vec::new()
            } else if path.file_name().is_some_and(|name| name == "Cargo.toml") {
                vec![path]
            } else if path.is_dir() {
                collect_manifests(&path)
            } else {
                Vec::new()
            }
        })
        .collect()
}
