use crate::command::{self, TaskResult};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::thread;

const COVERAGE_ENV: &str = "LOCKBOX_E2E_COVERAGE_FILE";

#[derive(Default)]
struct Coverage {
    successful_options: BTreeMap<String, BTreeSet<String>>,
    rejected_commands: BTreeSet<String>,
}

pub fn cli() -> TaskResult {
    let workspace = command::workspace_root()?;
    let coverage_path = workspace.join("target/cli-e2e-coverage.tsv");
    let coverage_dir = workspace.join("target/cli-e2e-coverage");
    fs::create_dir_all(&coverage_dir)
        .map_err(|error| format!("cannot create {}: {error}", coverage_dir.display()))?;

    // Each shard runs its tests serially, isolating Session Agent and archive
    // locks. The shards themselves run concurrently, matching the CI matrix.
    let shards = cli_shards();
    let mut workers = Vec::new();
    for (name, args) in shards {
        let coverage = coverage_dir.join(format!("{name}.tsv"));
        fs::write(&coverage, [])
            .map_err(|error| format!("cannot reset {}: {error}", coverage.display()))?;
        workers.push(thread::spawn(move || run_cli_shard(name, args, coverage)));
    }
    let mut errors = Vec::new();
    for worker in workers {
        match worker
            .join()
            .map_err(|_| "CLI E2E shard panicked".to_owned())?
        {
            Ok(()) => {}
            Err(error) => errors.push(error),
        }
    }
    if !errors.is_empty() {
        return Err(format!("CLI E2E shard failures:\n  {}", errors.join("\n  ")).into());
    }
    merge_coverage(&coverage_dir, &coverage_path)?;

    let mut session_agent = command::command("cargo");
    session_agent.args([
        "test",
        "-p",
        "revault_cli",
        "--test",
        "agent_flow",
        "open_and_open_key_complete_real_session_flows",
        "--",
        "--ignored",
        "--exact",
    ]);
    session_agent.env(COVERAGE_ENV, &coverage_path);
    command::run(&mut session_agent)?;

    let mut network = command::command("cargo");
    network.args([
        "test",
        "-p",
        "revault_cli",
        "--test",
        "publish_integration",
        "--",
        "--ignored",
    ]);
    network.env(COVERAGE_ENV, &coverage_path);
    command::run(&mut network)?;

    let inventory = command::output_lossy(command::command("cargo").args([
        "test",
        "-p",
        "revault_cli",
        "print_complete_command_inventory",
        "--",
        "--ignored",
        "--nocapture",
    ]))?;
    let expected = parse_inventory(&inventory);
    let actual = parse_coverage(
        &fs::read_to_string(&coverage_path)
            .map_err(|error| format!("cannot read {}: {error}", coverage_path.display()))?,
    );
    enforce(expected, actual)
}

fn cli_shards() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        (
            "flow-vault",
            vec!["test", "-p", "revault_cli", "--test", "cli_flow", "vault_"],
        ),
        (
            "flow-open",
            vec![
                "test",
                "-p",
                "revault_cli",
                "--test",
                "cli_flow",
                "open",
                "--",
                "--skip",
                "vault_",
            ],
        ),
        (
            "flow-session",
            vec![
                "test",
                "-p",
                "revault_cli",
                "--test",
                "cli_flow",
                "session",
                "--",
                "--skip",
                "vault_",
                "--skip",
                "open",
            ],
        ),
        (
            "flow-form",
            vec![
                "test",
                "-p",
                "revault_cli",
                "--test",
                "cli_flow",
                "form",
                "--",
                "--skip",
                "vault_",
                "--skip",
                "open",
                "--skip",
                "session",
            ],
        ),
        (
            "flow-create",
            vec![
                "test",
                "-p",
                "revault_cli",
                "--test",
                "cli_flow",
                "create",
                "--",
                "--skip",
                "vault_",
                "--skip",
                "open",
                "--skip",
                "session",
                "--skip",
                "form",
            ],
        ),
        (
            "flow-remove-list",
            vec![
                "test",
                "-p",
                "revault_cli",
                "--test",
                "cli_flow",
                "remove",
                "--",
                "--skip",
                "vault_",
                "--skip",
                "open",
                "--skip",
                "session",
                "--skip",
                "form",
                "--skip",
                "create",
            ],
        ),
        (
            "flow-remaining",
            vec![
                "test",
                "-p",
                "revault_cli",
                "--test",
                "cli_flow",
                "--",
                "--skip",
                "vault_",
                "--skip",
                "open",
                "--skip",
                "session",
                "--skip",
                "form",
                "--skip",
                "create",
                "--skip",
                "remove",
                "--skip",
                "list",
            ],
        ),
        (
            "support",
            vec![
                "test",
                "-p",
                "revault_cli",
                "--bin",
                "lockbox",
                "--test",
                "agent_flow",
                "--test",
                "completion",
                "--test",
                "contact_receive_alias",
                "--test",
                "help_open_key",
                "--test",
                "publish_integration",
            ],
        ),
        (
            "migration-mirror-password",
            vec![
                "test",
                "-p",
                "revault_cli",
                "--test",
                "migration_cli",
                "--test",
                "mirror_cli",
                "--test",
                "password_profiles",
            ],
        ),
    ]
}

fn run_cli_shard(name: &str, args: Vec<&'static str>, coverage: PathBuf) -> TaskResult {
    let mut tests = command::command("cargo");
    tests.args(args).args(["--", "--test-threads=1"]);
    tests.env(COVERAGE_ENV, coverage);
    eprintln!("running CLI E2E shard {name}");
    command::run(&mut tests)
}

fn merge_coverage(dir: &std::path::Path, output: &std::path::Path) -> TaskResult {
    let mut merged = String::new();
    for entry in
        fs::read_dir(dir).map_err(|error| format!("cannot read {}: {error}", dir.display()))?
    {
        let path = entry
            .map_err(|error| format!("cannot read coverage entry: {error}"))?
            .path();
        if path.extension().and_then(|value| value.to_str()) == Some("tsv") {
            merged.push_str(
                &fs::read_to_string(&path)
                    .map_err(|error| format!("cannot read {}: {error}", path.display()))?,
            );
        }
    }
    fs::write(output, merged).map_err(|error| format!("cannot write {}: {error}", output.display()))
}

fn parse_inventory(output: &str) -> BTreeMap<String, BTreeSet<String>> {
    output
        .lines()
        .filter_map(|line| {
            let (path, options) = line.split_once('\t')?;
            if path.is_empty()
                || !path.bytes().all(|byte| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"-/".contains(&byte)
                })
            {
                return None;
            }
            Some((path.to_string(), split_options(options)))
        })
        .collect()
}

fn parse_coverage(output: &str) -> Coverage {
    let mut coverage = Coverage::default();
    for line in output.lines() {
        let mut fields = line.splitn(3, '\t');
        let (Some(outcome), Some(path), Some(options)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        if outcome == "ok" {
            coverage
                .successful_options
                .entry(path.to_string())
                .or_default()
                .extend(split_options(options));
        } else if outcome == "error" {
            coverage.rejected_commands.insert(path.to_string());
        }
    }
    coverage
}

fn split_options(options: &str) -> BTreeSet<String> {
    options
        .split(',')
        .filter(|option| !option.is_empty())
        .map(str::to_string)
        .collect()
}

fn enforce(expected: BTreeMap<String, BTreeSet<String>>, actual: Coverage) -> TaskResult {
    let mut missing = Vec::new();
    for (path, options) in expected {
        let Some(covered) = actual.successful_options.get(&path) else {
            missing.push(format!("no successful E2E invocation: {path}"));
            continue;
        };
        let missing_options = options.difference(covered).cloned().collect::<Vec<_>>();
        if !missing_options.is_empty() {
            missing.push(format!(
                "successful E2E options missing for {path}: {}",
                missing_options.join(", ")
            ));
        }
    }
    if missing.is_empty() {
        println!(
            "CLI E2E contract complete: {} commands have successful realistic coverage; {} commands also exercised handler rejection paths.",
            actual.successful_options.len(),
            actual.rejected_commands.len()
        );
        Ok(())
    } else {
        Err(format!(
            "CLI E2E contract gaps:\n  {}",
            missing.join("\n  ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_parser_merges_real_invocations() {
        let coverage = parse_coverage(
            "ok\tadd\trecursive,sources\nerror\tadd\tsources\nok\tadd\tinclude,sources\n",
        );
        assert_eq!(
            coverage.successful_options["add"],
            BTreeSet::from([
                "include".to_string(),
                "recursive".to_string(),
                "sources".to_string(),
            ])
        );
        assert!(coverage.rejected_commands.contains("add"));
    }
}
