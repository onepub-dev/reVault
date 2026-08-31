use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::{parser::ValueSource, ArgMatches};
use revault_lockbox_api::Error;
use revault_vault_api::{
    get_platform_vault_password, list as list_open_lockboxes, local_vault,
    platform_secret_store_status, put_platform_vault_password, set_auto_open_scope,
    stop as stop_agent, verify_agent_transport_security, AutoOpenScope,
};

use super::context::{
    ensure_lockbox_path_accessible, open_default_vault_with_password, read_vault_password,
    CliResult,
};
use super::output::{output_format_from_matches, print_records};

pub(crate) fn run_matches(matches: &ArgMatches) -> CliResult<()> {
    let format = output_format_from_matches(matches)?;
    let explicit_format = matches.value_source("format") == Some(ValueSource::CommandLine);
    match matches.subcommand() {
        Some(("default", sub)) => {
            reject_session_format(explicit_format, "default")?;
            default_lockbox_matches(sub)
        }
        Some(("close-all", _)) => {
            reject_session_format(explicit_format, "close-all")?;
            local_vault().close_all()?;
            clear_default_lockbox()?;
            println!("All open Lockboxes closed.");
            Ok(())
        }
        Some(("stop", _)) => {
            reject_session_format(explicit_format, "stop")?;
            stop_agent()?;
            clear_default_lockbox()?;
            println!("Session Agent stopped.");
            Ok(())
        }
        Some(("auto-open", sub)) => auto_open_matches(sub, explicit_format.then_some(format)),
        Some((command, _)) => {
            Err(Error::InvalidInput(format!("unknown session command: {command}")).into())
        }
        None => list_sessions(format),
    }
}

fn reject_session_format(explicit: bool, command: &str) -> CliResult<()> {
    if explicit {
        return Err(
            Error::InvalidInput(format!("--format is not supported by session {command}")).into(),
        );
    }
    Ok(())
}

fn default_lockbox_matches(matches: &ArgMatches) -> CliResult<()> {
    if matches.get_flag("clear") {
        clear_default_lockbox()?;
        println!("Default lockbox cleared.");
        return Ok(());
    }
    let lockbox_path = matches
        .get_one::<String>("lockbox")
        .ok_or_else(|| Error::InvalidInput("missing lockbox".to_string()))?;
    set_default_lockbox(lockbox_path)
}

fn set_default_lockbox(lockbox_path: &str) -> CliResult<()> {
    ensure_lockbox_path_accessible(lockbox_path)?;
    let lockbox_path = fs::canonicalize(lockbox_path)?;
    let lockbox_path = lockbox_path.to_string_lossy().into_owned();
    write_default_lockbox(&lockbox_path)?;
    println!("Default lockbox: {lockbox_path}");
    Ok(())
}

fn list_sessions(format: super::output::OutputFormat) -> CliResult<()> {
    let agent_enabled = agent_enabled();
    let agent_running = revault_vault_api::is_running();
    let auto_open = platform_secret_store_status()?;
    let vault_pass_phrase_stored = platform_vault_pass_phrase_stored();
    if !matches!(format, super::output::OutputFormat::Table) {
        let default = default_lockbox_path_value()?;
        let mut rows = Vec::new();
        rows.push(vec![
            "agent".to_string(),
            "enabled".to_string(),
            yes_no(agent_enabled).to_string(),
            String::new(),
            String::new(),
        ]);
        rows.push(vec![
            "agent".to_string(),
            "running".to_string(),
            yes_no(agent_running).to_string(),
            String::new(),
            String::new(),
        ]);
        rows.push(vec![
            "auto-open".to_string(),
            "scope".to_string(),
            auto_open.scope.as_str().to_string(),
            String::new(),
            String::new(),
        ]);
        rows.push(vec![
            "auto-open".to_string(),
            "Vault passphrase stored".to_string(),
            yes_no(vault_pass_phrase_stored).to_string(),
            String::new(),
            String::new(),
        ]);
        rows.push(vec![
            "lockbox".to_string(),
            "default".to_string(),
            if default.is_some() { "yes" } else { "no" }.to_string(),
            default.clone().unwrap_or_default(),
            String::new(),
        ]);
        for lockbox in list_open_lockboxes()? {
            let path = lockbox.path.unwrap_or_default();
            rows.push(vec![
                "lockbox".to_string(),
                "open".to_string(),
                if default.as_deref() == Some(path.as_str()) {
                    "yes".to_string()
                } else {
                    "no".to_string()
                },
                path,
                lockbox.id,
            ]);
        }
        print_records(&["kind", "state", "value", "path", "uuid"], rows, format)?;
        return Ok(());
    }

    println!("Session Agent:");
    println!("  enabled: {}", yes_no(agent_enabled));
    println!("  running: {}", yes_no(agent_running));
    println!();
    println!("Auto-open:");
    println!("  scope: {}", auto_open.scope.as_str());
    println!(
        "  Vault passphrase stored: {}",
        yes_no(vault_pass_phrase_stored)
    );
    println!();
    println!("Default lockbox:");
    match default_lockbox_path_value()? {
        Some(path) => println!("  {path}"),
        None => println!("  none"),
    }
    println!();
    println!("Open lockboxes:");
    let open = list_open_lockboxes()?;
    if open.is_empty() {
        println!("  none");
    } else {
        for lockbox in open {
            println!("  {}", lockbox.path.unwrap_or(lockbox.id));
        }
    }
    Ok(())
}

fn auto_open_matches(
    matches: &ArgMatches,
    inherited_format: Option<super::output::OutputFormat>,
) -> CliResult<()> {
    match matches.subcommand() {
        Some(("status", sub)) => auto_open_status(auto_open_status_format(sub, inherited_format)?),
        Some(("disable", sub)) => {
            reject_auto_open_format(inherited_format, "disable")?;
            if !confirm_auto_open_disable(sub.get_flag("yes"))? {
                println!("Auto-open not disabled.");
                return Ok(());
            }
            set_auto_open_scope(AutoOpenScope::Off)?;
            local_vault().close_all()?;
            clear_default_lockbox()?;
            auto_open_status(super::output::OutputFormat::Table)
        }
        Some(("vault", _)) => {
            reject_auto_open_format(inherited_format, "vault")?;
            let password = read_vault_password("Vault passphrase: ")?;
            open_default_vault_with_password(&password)?;
            set_auto_open_scope(AutoOpenScope::Vault)?;
            put_platform_vault_password(&password)?;
            local_vault().close_all()?;
            auto_open_status(super::output::OutputFormat::Table)
        }
        Some(("lockboxes", _)) => {
            reject_auto_open_format(inherited_format, "lockboxes")?;
            let password = read_vault_password("Vault passphrase: ")?;
            open_default_vault_with_password(&password)?;
            set_auto_open_scope(AutoOpenScope::Lockboxes)?;
            put_platform_vault_password(&password)?;
            local_vault().close_all()?;
            auto_open_status(super::output::OutputFormat::Table)
        }
        Some((command, _)) => {
            Err(Error::InvalidInput(format!("unknown session auto-open command: {command}")).into())
        }
        None => auto_open_status(inherited_format.unwrap_or(super::output::OutputFormat::Table)),
    }
}

fn auto_open_status_format(
    matches: &ArgMatches,
    inherited_format: Option<super::output::OutputFormat>,
) -> CliResult<super::output::OutputFormat> {
    if matches.value_source("format") == Some(ValueSource::CommandLine) {
        output_format_from_matches(matches)
    } else {
        Ok(inherited_format.unwrap_or(output_format_from_matches(matches)?))
    }
}

fn reject_auto_open_format(
    inherited_format: Option<super::output::OutputFormat>,
    command: &str,
) -> CliResult<()> {
    if inherited_format.is_some() {
        return Err(Error::InvalidInput(format!(
            "--format is only supported by session and session auto-open status, not auto-open {command}"
        ))
        .into());
    }
    Ok(())
}

fn confirm_auto_open_disable(yes: bool) -> CliResult<bool> {
    if yes {
        return Ok(true);
    }

    eprintln!("Disable Auto Open?");
    eprintln!("The stored Vault passphrase will be removed from the platform credential store.");
    eprintln!("All open Lockboxes will be closed.");
    eprint!("Type 'yes' to disable Auto Open: ");
    io::stderr().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(answer.trim() == "yes")
}

fn auto_open_status(format: super::output::OutputFormat) -> CliResult<()> {
    let status = platform_secret_store_status()?;
    let stored = platform_vault_pass_phrase_stored();
    print_records(
        &["property", "value"],
        vec![
            vec![
                "supported".to_string(),
                yes_no(status.supported).to_string(),
            ],
            vec!["scope".to_string(), status.scope.as_str().to_string()],
            vec![
                "Vault passphrase stored".to_string(),
                yes_no(stored).to_string(),
            ],
            vec!["backend".to_string(), status.backend.to_string()],
            vec!["vault".to_string(), status.item],
        ],
        format,
    )?;
    Ok(())
}

fn agent_enabled() -> bool {
    verify_agent_transport_security().is_ok()
}

fn platform_vault_pass_phrase_stored() -> bool {
    get_platform_vault_password()
        .map(|password| password.is_some())
        .unwrap_or(false)
}

fn default_lockbox_path_value() -> CliResult<Option<String>> {
    let path = default_lockbox_path()?;
    match fs::read_to_string(path) {
        Ok(value) => Ok(Some(value.trim_end_matches('\n').to_string())),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn clear_default_lockbox() -> CliResult<()> {
    match fs::remove_file(default_lockbox_path()?) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(err.into()),
    }
    Ok(())
}

fn write_default_lockbox(lockbox_path: &str) -> CliResult<()> {
    let path = default_lockbox_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{lockbox_path}\n"))?;
    Ok(())
}

fn default_lockbox_path() -> CliResult<PathBuf> {
    Ok(revault_vault_api::default_vault_dir()?.join(".default-lockbox"))
}

pub(crate) fn default_lockbox_or_none() -> CliResult<Option<String>> {
    default_lockbox_path_value()
}

pub(crate) fn clear_default_if_matches(path: &str) -> CliResult<()> {
    let Some(default) = default_lockbox_path_value()? else {
        return Ok(());
    };
    if default == path || canonical_path_matches(&default, path) {
        clear_default_lockbox()?;
    }
    Ok(())
}

pub(crate) fn default_matches(path: &str) -> CliResult<bool> {
    Ok(default_lockbox_path_value()?
        .is_some_and(|default| default == path || canonical_path_matches(&default, path)))
}

pub(crate) fn replace_default_after_move(path: &Path) -> CliResult<()> {
    let canonical = fs::canonicalize(path)?;
    write_default_lockbox(&canonical.to_string_lossy())
}

fn canonical_path_matches(active: &str, path: &str) -> bool {
    fs::canonicalize(path)
        .map(|path| path == Path::new(active))
        .unwrap_or(false)
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

#[cfg(test)]
mod argument_tests {
    use super::*;
    use crate::commands::help;

    #[test]
    fn session_format_is_inherited_by_auto_open_status() {
        let matches = help::command(false)
            .try_get_matches_from([
                "lockbox",
                "session",
                "--format",
                "json",
                "auto-open",
                "status",
            ])
            .unwrap();
        let session = matches.subcommand_matches("session").unwrap();
        let inherited = output_format_from_matches(session).unwrap();
        let status = session
            .subcommand_matches("auto-open")
            .unwrap()
            .subcommand_matches("status")
            .unwrap();
        assert_eq!(
            auto_open_status_format(status, Some(inherited)).unwrap(),
            super::super::output::OutputFormat::Json
        );
    }

    #[test]
    fn session_format_is_rejected_for_non_output_commands() {
        assert!(reject_session_format(true, "close-all").is_err());
        assert!(
            reject_auto_open_format(Some(super::super::output::OutputFormat::Json), "disable")
                .is_err()
        );
    }
}
