//! Small, noninteractive consumers of variables and form fields.
use super::context::{cli_diagnostic, cli_error, open_existing_read_only, Access, CliResult};
use super::error_output::ExitCode;
use clap::{Arg, Command};
use clap_complete::engine::ArgValueCompleter;
use revault_lockbox_api::{
    FormValue, Lockbox, ReadOnly, SecretString, SecretVec, VariableName, VariableSensitivity,
};
use std::io::Write;
use zeroize::Zeroizing;

// The parser shares one positional argument for the optional lockbox and both
// selector forms. Describe those forms separately while keeping parsing intact.
const LBXX_HELP_TEMPLATE: &str = "{about}

Usage: {usage}

Arguments:
  [LOCKBOX]     Lockbox file path or alias. Accepts dev, dev.lbox,
                ./dev.lbox, /path/dev.lbox, or a@dev.
                Omit to use the session's default lockbox.
  VAR           Stored variable name, such as TOKEN
  FORM_FIELD    Form path and field, such as /work/github@token
  <COMMAND>...  Command and its arguments

Options:
  --lockbox <LOCKBOX>
      Explicitly identify the lockbox when its name could also be a
      variable name. All positional values then select variables or fields.
      Accepts the same forms as [LOCKBOX].

  -h, --help     Print help
  -V, --version  Print version

{after-help}";

const LBXV_HELP_TEMPLATE: &str = "{about}

Usage: {usage}

Arguments:
  [LOCKBOX]   Lockbox file path or alias. Accepts dev, dev.lbox,
              ./dev.lbox, /path/dev.lbox, or a@dev.
              Omit to use the session's default lockbox.
  VAR         Stored variable name, such as TOKEN
  FORM_FIELD  Form path and field, such as /work/github@token

Options:
  --lockbox <LOCKBOX>  Explicitly specify the lockbox by file path or alias.
                       Accepts the same forms as [LOCKBOX].
  -h, --help           Print help
  -V, --version        Print version

{after-help}";

pub(crate) fn form_selector(value: &str) -> CliResult<(&str, &str)> {
    let (path, field) = value
        .split_once('@')
        .ok_or_else(|| cli_error("expected /form/path@field"))?;
    if path.is_empty() || field.is_empty() || field.contains('@') {
        return Err(cli_error("expected /form/path@field with exactly one @"));
    }
    Ok((path, field))
}

pub(crate) fn command(exec: bool) -> Command {
    let command = Command::new(if exec { "lbxx" } else { "lbxv" })
        .version(env!("CARGO_PKG_VERSION"))
        .about(if exec { "Run a command with lockbox values as environment variables." } else { "Write a lockbox variable or form field value to standard output for use\nin shell commands and scripts, without adding a newline." })
        .override_usage(if exec { "lbxx [OPTIONS] [LOCKBOX] <VAR|FORM_FIELD>... -- <COMMAND>..." } else { "lbxv [OPTIONS] [LOCKBOX] <VAR|FORM_FIELD>" })
        .arg(Arg::new("lockbox").long("lockbox").value_name("LOCKBOX").help(if exec { "Use a lockbox file or alias instead of the session default" } else { "Lockbox filename, bare alias or a@alias (overrides the session default)" }).add(ArgValueCompleter::new(super::completion::lockbox_path_candidates)))
        .arg(Arg::new("selection").required(true).value_name("VAR|FORM_FIELD").help(if exec { "NAME, /form/path@field or ENV=selector" } else { "Optional lockbox, then NAME or /form/path@field" }).num_args(1..=if exec { usize::MAX } else { 2 }).add(ArgValueCompleter::new(super::completion::helper_candidates)))
        .after_help(if exec {
            "Environment variables:
  Variables use their stored names; form fields use their field names.
  Prefix either with ENV= to choose a different environment variable name.

Examples:
  lbxx TOKEN REGION -- dart pub get
  lbxx GITHUB_TOKEN=/work/github@token -- gh repo list
  lbxx dev TOKEN -- dart pub get
  lbxx --lockbox dev TOKEN -- dart pub get

If an alias and file share a name, use a@dev for the alias
or ./dev.lbox for the file.

For encrypted lockboxes, run `lbx [LOCKBOX] open` first.
For password-protected lockboxes, this asks for the password when needed.
Omit LOCKBOX to open the session’s default lockbox.

Alternatively, set the LOCKBOX_KEY environment variable to an existing
decryption key. Passwords must be provided through `lbx open`.

lbxx never asks for a password. Unencrypted lockboxes need no key.

Returns the command's exit status."
        } else {
            r#"Examples (Bash):
  Read a variable into a shell variable:
    TOKEN="$(lbxv ./dev.lbox TOKEN)"

  Read a form field using a lockbox alias:
    GITHUB_TOKEN="$(lbxv a@dev /work/github@token)"

  Pass a value as a command argument:
    mycommand --token "$(lbxv ./dev.lbox TOKEN)"

Writes the selected value exactly, whether normal or secret.

If an alias and file share a name, use a@dev for the alias
or ./dev.lbox for the file.

For encrypted lockboxes, run `lbx [LOCKBOX] open` first.
For password-protected lockboxes, this asks for the password when needed.
Omit LOCKBOX to open the session's default lockbox.

Alternatively, set the LOCKBOX_KEY environment variable to an existing
decryption key. Passwords must be provided through `lbx open`.

lbxv never asks for a password. Unencrypted lockboxes need no key."#
        });
    if exec {
        command.help_template(LBXX_HELP_TEMPLATE).arg(
            Arg::new("command")
                .value_name("COMMAND")
                .help("Command and arguments to run after --")
                .required(true)
                .last(true)
                .num_args(1..)
                .value_parser(clap::builder::OsStringValueParser::new()),
        )
    } else {
        command.help_template(LBXV_HELP_TEMPLATE)
    }
}

/// Recognize host paths without mistaking form selectors or assignments for them.
pub(crate) fn is_lockbox_argument(value: &str) -> bool {
    !value.contains('=')
        && (value.starts_with("a@")
            || ((!value.contains('@') || value.starts_with("./") || value.starts_with("../"))
                && (super::looks_like_lockbox_path(value)
                    || value.starts_with("./")
                    || value.starts_with("../")
                    || std::path::Path::new(value).is_file())))
        || (super::aliases::is_bare_name(value)
            && !value.contains('=')
            && (std::path::Path::new(&format!("{value}.lbox")).is_file()
                || super::completion::read_only_vault()
                    .and_then(|vault| vault.list_lockbox_aliases().ok())
                    .is_some_and(|aliases| aliases.iter().any(|alias| alias.name == value))))
}

fn value(lockbox: &Lockbox<ReadOnly>, selector: &str) -> CliResult<SecretString> {
    if selector.contains('@') {
        let (path, field) = form_selector(selector)?;
        let field = lockbox
            .get_form_field(&super::form::form_record_path(path)?, field)?
            .ok_or_else(|| cli_error("form field not found"))?;
        return match field.value {
            FormValue::Normal(value) => Ok(SecretString::try_from_bytes(value.into_bytes())?),
            FormValue::Secret(value) => Ok(value.try_clone()?),
        };
    }
    let name = VariableName::new(selector).map_err(|error| match error {
        revault_lockbox_api::Error::InvalidPath(detail) => cli_error(detail),
        other => other.into(),
    })?;
    match lockbox.variable_sensitivity(&name)? {
        Some(VariableSensitivity::Normal) => Ok(SecretString::try_from_bytes(
            lockbox
                .get_variable(&name)?
                .ok_or_else(|| cli_error("variable not found"))?
                .into_bytes(),
        )?),
        Some(VariableSensitivity::Secret) => Ok(lockbox
            .with_secret_variable(&name, |secret| secret.try_clone())?
            .ok_or_else(|| cli_error("variable not found"))??),
        None => Err(cli_error("variable not found")),
    }
}

fn destination(selection: &str) -> CliResult<(String, &str)> {
    let (name, source) = match selection.split_once('=') {
        Some((name, source)) => (name, source),
        None => {
            let name = if selection.contains('@') {
                form_selector(selection)?.1
            } else {
                selection.rsplit('/').next().unwrap_or("")
            };
            (name, selection)
        }
    };
    if source.is_empty()
        || name.is_empty()
        || !name.bytes().enumerate().all(|(index, byte)| {
            byte == b'_' || byte.is_ascii_alphabetic() || index > 0 && byte.is_ascii_digit()
        })
    {
        return Err(cli_error(
            "invalid environment name; use NAME=selector with letters, digits and underscore",
        ));
    }
    Ok((name.to_owned(), source))
}

pub(crate) fn run(binary: &str) -> CliResult<()> {
    let exec = binary == "lbxx";
    super::completion::capture_completion_shell();
    clap_complete::CompleteEnv::with_factory(|| command(exec))
        .bin(binary.to_owned())
        .complete();
    if std::env::args_os().len() == 1 {
        command(exec).print_long_help()?;
        println!();
        return Ok(());
    }
    let matches = command(exec).try_get_matches()?;
    let mut selections = matches
        .get_many::<String>("selection")
        .expect("required")
        .collect::<Vec<_>>();
    if !exec
        && selections.len() == 1
        && selections[0] == "/"
        && matches.get_one::<String>("lockbox").is_none()
    {
        return Err(cli_diagnostic(
            ExitCode::Usage,
            "Incomplete lbxv invocation",
            vec![(
                "Details".to_string(),
                "'/' names the host root directory or the lockbox root.\nIt identifies neither a lockbox file nor a variable or form field.".to_string(),
            )],
            "Specify a lockbox and value, e.g. `lbxv ./dev.lbox TOKEN`,\nor use `lbxv TOKEN` with a session default.",
        ));
    }
    let explicit = matches.get_one::<String>("lockbox").cloned().or_else(|| {
        if (is_lockbox_argument(selections[0])
            && (!super::aliases::is_bare_name(selections[0]) || selections.len() > 1))
            || (!exec && selections.len() == 2)
        {
            Some(selections.remove(0).clone())
        } else {
            None
        }
    });
    if selections.is_empty() || (!exec && selections.len() != 1) {
        return Err(cli_error(
            "expected a selector after the lockbox; use --help for examples",
        ));
    }
    let activity =
        revault_vault_api::begin_secret_activity(revault_vault_api::SecretActivityKind::Variables)?;
    let (path, identity) = super::aliases::select_noninteractive(explicit.as_deref())?;
    let access = match SecretString::try_from_env("LOCKBOX_KEY")? {
        Some(key) => {
            let mut bytes = key.with_str(|key| Zeroizing::new(key.as_bytes().to_vec()))?;
            Access::ContentKey(SecretVec::try_from_vec(std::mem::take(&mut *bytes))?)
        }
        None => Access::CacheOnly,
    };
    let lockbox = open_existing_read_only(&path, &access)?;
    drop(access);
    if identity.is_some_and(|id| lockbox.lockbox_id() != id) {
        return Err(cli_error("alias target identity changed"));
    }
    if !exec {
        if selections[0].contains('=') {
            return Err(cli_error("lbxv accepts a selector, not an assignment"));
        }
        let value = value(&lockbox, selections[0])?;
        value.with_str(|value| std::io::stdout().lock().write_all(value.as_bytes()))??;
        return Ok(());
    }
    let mut names = std::collections::HashSet::new();
    let mut environment = Vec::new();
    for selection in selections {
        let (name, source) = destination(selection)?;
        let key = if cfg!(windows) {
            name.to_ascii_uppercase()
        } else {
            name.clone()
        };
        if !names.insert(key) {
            return Err(cli_error("duplicate environment destination"));
        }
        let value = value(&lockbox, source)?;
        value.with_str(|text| {
            if text.contains('\0') {
                Err(cli_error("environment values cannot contain NUL"))
            } else {
                Ok(())
            }
        })??;
        environment.push((name, value));
    }
    // Release the archive lock before running a potentially long-lived child.
    drop(lockbox);
    drop(activity);
    let mut args = matches
        .get_many::<std::ffi::OsString>("command")
        .expect("required");
    let mut child = std::process::Command::new(args.next().expect("required"));
    child.args(args);
    for (name, value) in &environment {
        value.with_str(|value| {
            child.env(name, value);
        })?;
    }
    let spawned = child.spawn();
    // Command stores environment copies; release them promptly after spawn.
    drop(child);
    drop(environment);
    let status = spawned?.wait()?;
    let code = status.code().unwrap_or_else(|| {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            128 + status.signal().unwrap_or(1)
        }
        #[cfg(not(unix))]
        {
            1
        }
    });
    std::process::exit(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selectors_and_environment_names() {
        assert_eq!(
            destination("/work/github@token").unwrap(),
            ("token".into(), "/work/github@token")
        );
        assert_eq!(destination("TOKEN=/work/github@token").unwrap().0, "TOKEN");
        assert_eq!(destination("/work/TOKEN").unwrap().0, "TOKEN");
        for bad in ["", "1TOKEN", "A-B", "=TOKEN", "A=", "/form@", "/a@b@c"] {
            assert!(destination(bad).is_err(), "{bad}");
        }
        for bad in ["form", "@field", "form@", "form@field@other"] {
            assert!(form_selector(bad).is_err());
        }
    }
}
