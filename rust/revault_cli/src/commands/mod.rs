mod context;
mod doctor;
mod files;
mod form;
mod help;
mod keys;
mod output;
mod recovery;
mod session;
mod variables;
mod vault;
mod visualize;

use clap::ArgMatches;
use context::{cli_error, ensure_lockbox_path_accessible, Access, CliResult};
use lockbox_core::{Error, SecretVec, WorkerPolicy};
use lockbox_vault::SecretActivityKind;
use std::env as std_env;
use std::path::Path;

pub(crate) fn run() -> CliResult<()> {
    let args: Vec<String> = normalize_form_define_separator(std::env::args().skip(1).collect());
    if args.first().map(String::as_str) == Some("__agent") {
        return Ok(lockbox_vault::serve_agent()?);
    }
    if args.first().map(String::as_str) == Some("__agent_security_check") {
        return Ok(lockbox_vault::verify_agent_transport_security()?);
    }
    reject_variables_set_single_dash_secret(&args)?;

    let verbose_help = args.iter().any(|arg| arg == "--verbose");
    if args.is_empty() || is_top_level_help(&args) {
        help::usage(verbose_help);
        return Ok(());
    }
    let command = help::command(verbose_help);
    let matches =
        match command.try_get_matches_from(std::iter::once("lockbox".to_string()).chain(args)) {
            Ok(matches) => matches,
            Err(err) if err.kind() == clap::error::ErrorKind::DisplayHelp => {
                err.print()?;
                return Ok(());
            }
            Err(err) => return Err(err.into()),
        };

    let (command, command_matches) = matches
        .subcommand()
        .ok_or_else(|| Error::InvalidInput("missing command".to_string()))?;
    let _secret_activity = command_secret_activity(command)
        .map(lockbox_vault::begin_secret_activity)
        .transpose()?;
    let access = read_access(&matches, command)?;

    match command {
        "create" => keys::create_matches(command_matches, &access)?,
        "doctor" => doctor::run_matches(command_matches)?,
        "open" => keys::open_matches(command_matches)?,
        "close" => keys::close_matches(command_matches)?,
        "keygen" => keys::keygen_matches(command_matches)?,
        "open-key" => keys::open_key_matches(command_matches)?,
        "session" => session::run_matches(command_matches)?,
        "access" => keys::access_matches(command_matches, &access)?,
        "vault" => vault::run_matches(command_matches)?,
        "add" => files::add_matches(
            command_matches,
            &access,
            read_worker_policy(command_matches)?,
        )?,
        "extract" => files::extract_matches(command_matches, &access)?,
        "cat" => files::cat_matches(command_matches, &access)?,
        "list" => files::list_matches(command_matches, &access)?,
        "rm" => files::remove_matches(command_matches, &access)?,
        "rename" => files::rename_matches(command_matches, &access)?,
        "variable" => variables::run_matches(command_matches, &access)?,
        "form" => form::run_matches(command_matches, &access)?,
        "recover" => recovery::run_matches(command_matches, &access)?,
        "visualize" => visualize::run_matches(command_matches, &access)?,
        _ => return Err(Error::InvalidInput(format!("unknown command: {command}")).into()),
    }

    Ok(())
}

fn normalize_form_define_separator(mut args: Vec<String>) -> Vec<String> {
    if args.first().map(String::as_str) != Some("form")
        || args.get(1).map(String::as_str) != Some("define")
    {
        return args;
    }
    args.retain(|arg| arg != "--");
    args
}

fn reject_variables_set_single_dash_secret(args: &[String]) -> CliResult<()> {
    if matches!(args.first().map(String::as_str), Some("variable" | "var"))
        && args.get(1).map(String::as_str) == Some("set")
        && args.iter().skip(2).any(|arg| arg == "-secret")
    {
        return Err(cli_error("unknown option: -secret. Use --secret."));
    }
    Ok(())
}

fn command_secret_activity(command: &str) -> Option<SecretActivityKind> {
    match command {
        "open" => Some(SecretActivityKind::Open),
        "close" => Some(SecretActivityKind::Close),
        "add" | "extract" | "cat" | "list" | "rm" | "rename" | "visualize" => {
            Some(SecretActivityKind::Open)
        }
        "variable" => Some(SecretActivityKind::Variables),
        "form" => Some(SecretActivityKind::Form),
        "recover" => Some(SecretActivityKind::Recovery),
        "access" | "open-key" | "session" => Some(SecretActivityKind::Vault),
        _ => None,
    }
}

fn read_access(matches: &ArgMatches, command: &str) -> CliResult<Access> {
    if let Some(key) = matches.get_one::<String>("key") {
        return Ok(Access::ContentKey(SecretVec::try_from_vec(
            key.clone().into_bytes(),
        )?));
    }
    if let Ok(key) = std_env::var("LOCKBOX_KEY") {
        return Ok(Access::ContentKey(SecretVec::try_from_vec(
            key.into_bytes(),
        )?));
    }
    if command == "create" {
        Ok(Access::PromptPassword)
    } else {
        Ok(Access::CacheOnly)
    }
}

fn is_top_level_help(args: &[String]) -> bool {
    args.iter()
        .filter(|arg| arg.as_str() != "--verbose")
        .all(|arg| matches!(arg.as_str(), "--help" | "-h"))
}

fn read_worker_policy(matches: &ArgMatches) -> CliResult<WorkerPolicy> {
    let Some(value) = matches.get_one::<String>("jobs") else {
        return Ok(WorkerPolicy::Auto);
    };
    match value.as_str() {
        "auto" => Ok(WorkerPolicy::Auto),
        "1" => Ok(WorkerPolicy::Single),
        _ => {
            let jobs = value.parse::<usize>().map_err(|_| {
                Error::InvalidInput("--jobs must be auto, 1, or a positive integer".to_string())
            })?;
            if jobs == 0 {
                return Err(Error::InvalidInput(
                    "--jobs must be auto, 1, or a positive integer".to_string(),
                )
                .into());
            }
            Ok(WorkerPolicy::Threads(jobs))
        }
    }
}

pub(crate) fn default_lockbox_for_add() -> CliResult<String> {
    default_lockbox_for_add_if_set()?.ok_or_else(|| {
        cli_error("missing lockbox; pass a .lbox path or set a session default lockbox")
    })
}

fn default_lockbox_for_add_if_set() -> CliResult<Option<String>> {
    let Some(default) = session::default_lockbox_or_none()? else {
        return Ok(None);
    };
    ensure_lockbox_path_accessible(&default)
        .map_err(|_| cli_error(format!("session default lockbox not found: {default}")))?;
    Ok(Some(default))
}

pub(crate) fn optional_lockbox_value(matches: &ArgMatches, name: &str) -> CliResult<String> {
    match matches.get_one::<String>(name) {
        Some(value) => Ok(value.clone()),
        None => default_lockbox_for_command(),
    }
}

pub(crate) fn optional_lockbox_positionals(
    mut values: Vec<String>,
    required_after_lockbox: usize,
) -> CliResult<Vec<String>> {
    if values
        .first()
        .is_some_and(|value| looks_like_lockbox_path(value))
    {
        if values.len() < required_after_lockbox + 1 {
            return Err(cli_error("missing argument after lockbox"));
        }
        return Ok(values);
    }
    if values.len() < required_after_lockbox {
        return Err(cli_error("missing required argument"));
    }
    values.insert(0, default_lockbox_for_command()?);
    Ok(values)
}

pub(crate) fn default_lockbox_for_command() -> CliResult<String> {
    default_lockbox_for_add_if_set()?.ok_or_else(|| {
        cli_error("missing lockbox; pass a .lbox path or set a session default lockbox")
    })
}

pub(crate) fn looks_like_lockbox_path(value: &str) -> bool {
    value.ends_with(".lbox")
        || Path::new(value)
            .extension()
            .is_some_and(|ext| ext == "lbox")
}

pub(crate) fn positional_values(matches: &ArgMatches, name: &str) -> Vec<String> {
    matches
        .get_many::<String>(name)
        .map(|values| values.cloned().collect())
        .unwrap_or_default()
}
