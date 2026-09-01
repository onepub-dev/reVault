mod completion;
mod context;
mod doctor;
mod error_output;
mod files;
mod filters;
mod form;
mod help;
mod keys;
mod migrate;
mod mirror;
mod mirror_index;
mod output;
mod recovery;
mod session;
mod variables;
mod vault;
mod visualize;

use clap::ArgMatches;
use context::{cli_error, ensure_lockbox_path_accessible, Access, CliResult};
use revault_lockbox_api::{Error, SecretVec, WorkerPolicy};
use revault_vault_api::SecretActivityKind;
use std::cell::RefCell;
use std::env as std_env;
#[cfg(debug_assertions)]
use std::fs::OpenOptions;
#[cfg(debug_assertions)]
use std::io::Write as _;
use std::path::Path;

pub(crate) use error_output::{exit_code, print_error};

thread_local! {
    static COMMAND_LOCKBOX: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub(crate) fn run() -> CliResult<()> {
    let binary_name = std::env::args_os()
        .next()
        .and_then(|value| {
            Path::new(&value)
                .file_stem()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "lockbox".to_string());
    clap_complete::CompleteEnv::with_factory(|| help::command(false))
        .bin(binary_name)
        .complete();
    let args: Vec<String> = normalize_form_define_separator(std::env::args().skip(1).collect());
    if args.first().map(String::as_str) == Some("__agent") {
        return Ok(revault_vault_api::serve_agent()?);
    }
    if args.first().map(String::as_str) == Some("__agent_security_check") {
        return Ok(revault_vault_api::verify_agent_transport_security()?);
    }
    reject_variables_set_single_dash_secret(&args)?;

    let verbose_help = args.iter().any(|arg| arg == "--verbose");
    if args.is_empty() || is_top_level_help(&args) {
        help::usage(verbose_help);
        return Ok(());
    }
    let command = help::command(verbose_help);
    reject_unknown_top_level_token(&args, &command)?;
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
    let command_lockbox = matches.get_one::<String>("command-lockbox").cloned();
    if command_lockbox.is_some() && !command_accepts_lockbox(command) {
        return Err(cli_error(format!(
            "{command} is not a lockbox-scoped command; place the command immediately after `lockbox`"
        )));
    }
    set_command_lockbox(command_lockbox);
    let secret_activity = if command == "doctor"
        && matches!(
            command_matches.subcommand_name(),
            Some("recover" | "migrate")
        ) {
        Some(SecretActivityKind::Recovery)
    } else {
        command_secret_activity(command)
    };
    let _secret_activity = secret_activity
        .map(revault_vault_api::begin_secret_activity)
        .transpose()?;
    let access = read_access(&matches, command)?;

    let result = match command {
        "create" => keys::create_matches(command_matches, &access),
        "doctor" => doctor::run_matches(command_matches, &access),
        "open" => keys::open_matches(command_matches),
        "close" => keys::close_matches(command_matches),
        "keygen" => keys::keygen_matches(command_matches),
        "open-key" => keys::open_key_matches(command_matches),
        "session" => session::run_matches(command_matches),
        "completion" => completion::run_matches(command_matches),
        "access" => keys::access_matches(command_matches, &access),
        "vault" => vault::run_matches(command_matches),
        "add" => files::add_matches(
            command_matches,
            &access,
            read_worker_policy(command_matches)?,
        ),
        "mirror" => mirror::run_matches(command_matches, &access),
        "extract" => files::extract_matches(command_matches, &access),
        "cat" => files::cat_matches(command_matches, &access),
        "list" => files::list_matches(command_matches, &access),
        "remove" => files::remove_matches(command_matches, &access),
        "move" => files::rename_matches(command_matches, &access),
        "variable" => variables::run_matches(command_matches, &access),
        "description" => variables::description_matches(command_matches, &access),
        "form" => form::run_matches(command_matches, &access),
        "visualize" => visualize::run_matches(command_matches, &access),
        _ => Err(Error::InvalidInput(format!("unknown command: {command}")).into()),
    };
    record_e2e_invocation(command, command_matches, result.is_ok());
    result
}

#[cfg(debug_assertions)]
fn record_e2e_invocation(command: &str, matches: &ArgMatches, succeeded: bool) {
    let Some(path) = std_env::var_os("LOCKBOX_E2E_COVERAGE_FILE") else {
        return;
    };
    let mut command_path = command.to_string();
    let mut selected = matches;
    while let Some((name, child)) = selected.subcommand() {
        command_path.push('/');
        command_path.push_str(name);
        selected = child;
    }
    let mut options = selected
        .ids()
        .filter(|id| {
            selected.value_source(id.as_str()) == Some(clap::parser::ValueSource::CommandLine)
        })
        .map(|id| id.as_str().to_string())
        .collect::<Vec<_>>();
    options.sort_unstable();
    options.dedup();
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(
            file,
            "{}\t{}\t{}",
            if succeeded { "ok" } else { "error" },
            command_path,
            options.join(",")
        );
    }
}

#[cfg(not(debug_assertions))]
fn record_e2e_invocation(_command: &str, _matches: &ArgMatches, _succeeded: bool) {}

fn command_accepts_lockbox(command: &str) -> bool {
    matches!(
        command,
        "create"
            | "open"
            | "close"
            | "add"
            | "mirror"
            | "extract"
            | "cat"
            | "list"
            | "ls"
            | "remove"
            | "rm"
            | "delete"
            | "move"
            | "rename"
            | "mv"
            | "variable"
            | "var"
            | "variables"
            | "description"
            | "form"
            | "access"
            | "doctor"
            | "visualize"
            | "visualise"
            | "open-key"
    )
}

fn set_command_lockbox(lockbox: Option<String>) {
    COMMAND_LOCKBOX.with(|selected| {
        *selected.borrow_mut() = lockbox;
    });
}

pub(crate) fn command_lockbox() -> Option<String> {
    COMMAND_LOCKBOX.with(|selected| selected.borrow().clone())
}

fn normalize_form_define_separator(mut args: Vec<String>) -> Vec<String> {
    let command_index = usize::from(
        args.first()
            .is_some_and(|arg| command_accepts_lockbox_at_position(arg, &args)),
    );
    if args.get(command_index).map(String::as_str) != Some("form")
        || args.get(command_index + 1).map(String::as_str) != Some("define")
    {
        return args;
    }
    args.retain(|arg| arg != "--");
    args
}

fn reject_variables_set_single_dash_secret(args: &[String]) -> CliResult<()> {
    let command_index = usize::from(
        args.first()
            .is_some_and(|arg| command_accepts_lockbox_at_position(arg, args)),
    );
    if matches!(
        args.get(command_index).map(String::as_str),
        Some("variable" | "var")
    ) && args.get(command_index + 1).map(String::as_str) == Some("set")
        && args
            .iter()
            .skip(command_index + 2)
            .any(|arg| arg == "-secret")
    {
        return Err(cli_error("unknown option: -secret. Use --secret."));
    }
    Ok(())
}

fn command_accepts_lockbox_at_position(first: &str, args: &[String]) -> bool {
    looks_like_lockbox_path(first)
        && args
            .get(1)
            .is_some_and(|command| command_accepts_lockbox(command))
}

fn reject_unknown_top_level_token(
    args: &[String],
    command: &clap::Command,
) -> Result<(), clap::Error> {
    let Some((index, first)) = top_level_candidate(args) else {
        return Ok(());
    };
    if looks_like_lockbox_path(first) || is_top_level_command(command, first) {
        return Ok(());
    }
    let Some(next) = args.get(index + 1) else {
        return Ok(());
    };
    let unambiguously_command = looks_like_lockbox_path(next)
        || (is_top_level_command(command, next) && !command_accepts_lockbox(next));
    if !unambiguously_command {
        return Ok(());
    }
    Err(command.clone().error(
        clap::error::ErrorKind::InvalidSubcommand,
        format!("unrecognized subcommand '{first}'"),
    ))
}

fn top_level_candidate(args: &[String]) -> Option<(usize, &str)> {
    let mut index = 0;
    while let Some(arg) = args.get(index) {
        match arg.as_str() {
            "--verbose" => index += 1,
            "--key" => index += 2,
            value if value.starts_with("--key=") => index += 1,
            value if value.starts_with('-') => return None,
            value => return Some((index, value)),
        }
    }
    None
}

fn is_top_level_command(command: &clap::Command, value: &str) -> bool {
    command.get_subcommands().any(|subcommand| {
        subcommand.get_name() == value || subcommand.get_all_aliases().any(|alias| alias == value)
    })
}

fn command_secret_activity(command: &str) -> Option<SecretActivityKind> {
    match command {
        "open" => Some(SecretActivityKind::Open),
        "close" => Some(SecretActivityKind::Close),
        "add" | "mirror" | "extract" | "cat" | "list" | "remove" | "delete" | "move"
        | "visualize" => Some(SecretActivityKind::Open),
        "variable" | "description" => Some(SecretActivityKind::Variables),
        "form" => Some(SecretActivityKind::Form),
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
    if let Some(lockbox) = command_lockbox() {
        return Ok(lockbox);
    }
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

pub(crate) fn optional_lockbox_value(_matches: &ArgMatches, _name: &str) -> CliResult<String> {
    default_lockbox_for_command()
}

pub(crate) fn optional_lockbox_positionals(
    mut values: Vec<String>,
    required_after_lockbox: usize,
) -> CliResult<Vec<String>> {
    if let Some(lockbox) = command_lockbox() {
        if values.len() < required_after_lockbox {
            return Err(cli_error("missing required argument"));
        }
        values.insert(0, lockbox);
        return Ok(values);
    }
    if values
        .first()
        .is_some_and(|value| looks_like_lockbox_path(value))
    {
        return Err(cli_error(
            "lockbox paths must precede lockbox-scoped commands; use `lockbox LOCKBOX COMMAND ...`",
        ));
    }
    if values.len() < required_after_lockbox {
        return Err(cli_error("missing required argument"));
    }
    values.insert(0, default_lockbox_for_command()?);
    Ok(values)
}

pub(crate) fn default_lockbox_for_command() -> CliResult<String> {
    if let Some(lockbox) = command_lockbox() {
        return Ok(lockbox);
    }
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
