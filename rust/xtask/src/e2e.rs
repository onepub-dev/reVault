use crate::command::{self, TaskResult};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

const COVERAGE_ENV: &str = "LOCKBOX_E2E_COVERAGE_FILE";

#[derive(Default)]
struct Coverage {
    successful_options: BTreeMap<String, BTreeSet<String>>,
    rejected_commands: BTreeSet<String>,
}

pub fn cli() -> TaskResult {
    let workspace = command::workspace_root()?;
    let coverage_path = workspace.join("target/cli-e2e-coverage.tsv");
    if let Some(parent) = coverage_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::write(&coverage_path, [])
        .map_err(|error| format!("cannot reset {}: {error}", coverage_path.display()))?;

    let mut tests = command::command("cargo");
    tests.args(["test", "-p", "revault_cli", "--tests"]);
    tests.env(COVERAGE_ENV, &coverage_path);
    command::run(&mut tests)?;

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
