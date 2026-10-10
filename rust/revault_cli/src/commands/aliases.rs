use super::context::{cli_error, default_vault, CliMessage, CliResult};
use super::error_output::ExitCode;
use revault_lockbox_api::{Lockbox, LockboxId};

/// Selection and existence are separate: creation and stale-record maintenance
/// must be able to name an archive whose file is no longer present.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetPolicy {
    Existing,
    MustBeMissing,
    Remembered,
}

pub(crate) fn resolve_noninteractive(value: &str) -> CliResult<(String, Option<LockboxId>)> {
    let Some(name) = value.strip_prefix("a@") else {
        return Ok((value.to_owned(), None));
    };
    let vault = super::completion::read_only_vault()
        .ok_or_else(|| cli_error("Vault is locked; open it with lbx first"))?;
    let known = vault.resolve_lockbox_alias(name)?;
    validate_target(&known.path, Some(known.lockbox_id), TargetPolicy::Existing)?;
    Ok((known.path, Some(known.lockbox_id)))
}

pub(crate) fn select_noninteractive(value: Option<&str>) -> CliResult<(String, Option<LockboxId>)> {
    let default;
    let value = match value {
        Some(value) => value,
        None => {
            default = super::session::default_lockbox_or_none()?.ok_or_else(|| {
                cli_error(
                    "missing lockbox; pass a path or a@alias, or run `lbx session default LOCKBOX`",
                )
            })?;
            &default
        }
    };
    let resolved = resolve_noninteractive(value)?;
    validate_target(&resolved.0, resolved.1, TargetPolicy::Existing)?;
    Ok(resolved)
}

/// Only a leading a@ is special. ./a@name always remains a host filename.
pub(crate) fn resolve(value: &str) -> CliResult<(String, Option<LockboxId>)> {
    resolve_with_policy(value, TargetPolicy::Existing)
}

pub(crate) fn resolve_with_policy(
    value: &str,
    policy: TargetPolicy,
) -> CliResult<(String, Option<LockboxId>)> {
    let Some(name) = value.strip_prefix("a@") else {
        if policy == TargetPolicy::MustBeMissing {
            validate_target(value, None, policy)?;
        }
        return Ok((value.to_string(), None));
    };
    let known = default_vault()?.resolve_lockbox_alias(name)?;
    validate_target(&known.path, Some(known.lockbox_id), policy)?;
    Ok((known.path, Some(known.lockbox_id)))
}

pub(crate) fn validate_target(
    path: &str,
    identity: Option<LockboxId>,
    policy: TargetPolicy,
) -> CliResult<()> {
    if policy == TargetPolicy::Remembered {
        return Ok(());
    }
    match std::fs::symlink_metadata(path) {
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                && policy == TargetPolicy::MustBeMissing =>
        {
            return Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(missing_target(path));
        }
        Err(error) => return Err(error.into()),
        Ok(_) if policy == TargetPolicy::MustBeMissing => {
            return Err(existing_creation_target(path));
        }
        Ok(metadata) if metadata.is_dir() => {
            return Err(cli_error(format!("lockbox path is a directory: {path}")));
        }
        Ok(_) => {}
    }
    if let Some(identity) = identity {
        check_identity(path, identity)?;
    }
    Ok(())
}

pub(crate) fn missing_target(path: &str) -> Box<dyn std::error::Error> {
    Box::new(CliMessage {
        exit_code: ExitCode::NotFound,
        summary: format!("lockbox not found: {path}"),
        details: Vec::new(),
        next_step: Some(format!(
            "Select an existing lockbox, or explicitly create it with `lbx {path} create`"
        )),
    })
}

pub(crate) fn missing_default(path: &str) -> Box<dyn std::error::Error> {
    Box::new(CliMessage {
        exit_code: ExitCode::NotFound,
        summary: format!("session default lockbox not found: {path}"),
        details: Vec::new(),
        next_step: Some(format!("Select an existing lockbox with `lbx session default LOCKBOX`, or explicitly create it with `lbx {path} create`")),
    })
}

pub(crate) fn existing_creation_target(path: &str) -> Box<dyn std::error::Error> {
    Box::new(CliMessage {
        exit_code: ExitCode::General,
        summary: format!("lockbox already exists: {path}"),
        details: Vec::new(),
        next_step: Some("Choose a different path to create a new lockbox. Create never opens or replaces an existing archive.".to_owned()),
    })
}

pub(crate) fn check_identity(path: &str, expected: LockboxId) -> CliResult<()> {
    if Lockbox::inspect_file(path)?.lockbox_id != expected {
        return Err(cli_error(
            "alias target identity changed; explicitly update the alias",
        ));
    }
    Ok(())
}
