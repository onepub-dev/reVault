use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::ArgMatches;
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
    match matches.subcommand() {
        Some(("default", sub)) => default_lockbox_matches(sub),
        Some(("close-all", _)) => {
            local_vault().close_all()?;
            clear_default_lockbox()?;
            println!("All lockbox sessions closed.");
            Ok(())
        }
        Some(("stop", _)) => {
            stop_agent()?;
            clear_default_lockbox()?;
            println!("Session agent stopped.");
            Ok(())
        }
        Some(("auto-open", sub)) => auto_open_matches(sub),
        Some((command, _)) => {
            Err(Error::InvalidInput(format!("unknown session command: {command}")).into())
        }
        None => list_sessions(output_format_from_matches(matches)?),
    }
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
            "vault pass phrase stored".to_string(),
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

    println!("Session agent:");
    println!("  enabled: {}", yes_no(agent_enabled));
    println!("  running: {}", yes_no(agent_running));
    println!();
    println!("Auto-open:");
    println!("  scope: {}", auto_open.scope.as_str());
    println!(
        "  vault pass phrase stored: {}",
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

fn auto_open_matches(matches: &ArgMatches) -> CliResult<()> {
    match matches.subcommand() {
        Some(("status", sub)) => auto_open_status(output_format_from_matches(sub)?),
        Some(("disable", sub)) => {
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
            let password = read_vault_password("Vault pass phrase: ")?;
            open_default_vault_with_password(&password)?;
            set_auto_open_scope(AutoOpenScope::Vault)?;
            put_platform_vault_password(&password)?;
            local_vault().close_all()?;
            auto_open_status(super::output::OutputFormat::Table)
        }
        Some(("lockboxes", _)) => {
            let password = read_vault_password("Vault pass phrase: ")?;
            open_default_vault_with_password(&password)?;
            set_auto_open_scope(AutoOpenScope::Lockboxes)?;
            put_platform_vault_password(&password)?;
            local_vault().close_all()?;
            auto_open_status(super::output::OutputFormat::Table)
        }
        Some((command, _)) => {
            Err(Error::InvalidInput(format!("unknown session auto-open command: {command}")).into())
        }
        None => auto_open_status(super::output::OutputFormat::Table),
    }
}

fn confirm_auto_open_disable(yes: bool) -> CliResult<bool> {
    if yes {
        return Ok(true);
    }

    eprintln!("Disable auto-open?");
    eprintln!("The stored vault pass phrase will be removed from the OS key store.");
    eprintln!("All open lockbox sessions will be closed.");
    eprint!("Type 'yes' to disable auto-open: ");
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
                "vault pass phrase stored".to_string(),
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
