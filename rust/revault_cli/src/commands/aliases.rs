use super::context::{cli_error, default_vault, CliMessage, CliResult};
use super::error_output::ExitCode;
use revault_lockbox_api::{Lockbox, LockboxId};

pub(crate) fn validate_name(name: &str) -> CliResult<()> {
    revault_vault_api::validate_vault_record_name(name)?;
    if name.len() > 128 {
        return Err(cli_error("lockbox alias exceeds 128 bytes"));
    }
    Ok(())
}

/// Selection and existence are separate: creation and stale-record maintenance
/// must be able to name an archive whose file is no longer present.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetPolicy {
    Existing,
    MustBeMissing,
    Remembered,
}

pub(crate) fn resolve_noninteractive(value: &str) -> CliResult<(String, Option<LockboxId>)> {
    resolve_selection(value, TargetPolicy::Existing, true)
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

/// Paths and extensions explicitly select files; bare names can select aliases.
pub(crate) fn resolve(value: &str) -> CliResult<(String, Option<LockboxId>)> {
    resolve_with_policy(value, TargetPolicy::Existing)
}

pub(crate) fn resolve_with_policy(
    value: &str,
    policy: TargetPolicy,
) -> CliResult<(String, Option<LockboxId>)> {
    resolve_selection(value, policy, false)
}

pub(crate) fn is_bare_name(value: &str) -> bool {
    !value.is_empty() && !value.contains(['/', '\\', '.', '@'])
}

fn alias_target(
    name: &str,
    explicit: bool,
    noninteractive: bool,
) -> CliResult<Option<revault_vault_api::KnownLockbox>> {
    // A missing Vault permits headless file use. An inaccessible Vault must not
    // silently hide an alias that could make a bare name ambiguous.
    if !explicit && !revault_vault_api::default_vault_path()?.try_exists()? {
        return Ok(None);
    }
    if noninteractive {
        let vault = super::completion::read_only_vault()
            .ok_or_else(|| cli_error("Vault is unavailable or locked; open it with lbx first, or use ./ or a .lbox extension to select a file"))?;
        if !explicit
            && !vault
                .list_lockbox_aliases()?
                .iter()
                .any(|alias| alias.name == name)
        {
            return Ok(None);
        }
        Ok(Some(vault.resolve_lockbox_alias(name)?))
    } else {
        let vault = default_vault()?;
        if !explicit
            && !vault
                .list_lockbox_aliases()?
                .iter()
                .any(|alias| alias.name == name)
        {
            return Ok(None);
        }
        Ok(Some(vault.resolve_lockbox_alias(name)?))
    }
}

fn resolve_selection(
    value: &str,
    policy: TargetPolicy,
    noninteractive: bool,
) -> CliResult<(String, Option<LockboxId>)> {
    let explicit_alias = value.strip_prefix("a@");
    if explicit_alias.is_none() && !is_bare_name(value) {
        if policy == TargetPolicy::MustBeMissing {
            validate_target(value, None, policy)?;
        }
        return Ok((value.to_string(), None));
    }
    let known = alias_target(
        explicit_alias.unwrap_or(value),
        explicit_alias.is_some(),
        noninteractive,
    )?;
    let local = if explicit_alias.is_some() {
        None
    } else {
        match std::fs::symlink_metadata(value) {
            Ok(_) => Some(value.to_owned()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let suffixed = format!("{value}.lbox");
                match std::fs::symlink_metadata(&suffixed) {
                    Ok(_) => Some(suffixed),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                    Err(error) => return Err(error.into()),
                }
            }
            Err(error) => return Err(error.into()),
        }
    };
    if let Some(known) = known {
        if let Some(local) = local {
            // Compare stable archive identities, including symlinks and copies,
            // rather than spellings of paths. Still validate the alias target.
            if std::path::Path::new(&known.path).try_exists()? {
                check_identity(&known.path, known.lockbox_id)?;
            }
            let local_identity = Lockbox::inspect_file(&local)
                .map_err(|error| {
                    ambiguous_name(
                        value,
                        &local,
                        vec![(format!("Local candidate {local}"), error.to_string())],
                    )
                })?
                .lockbox_id;
            if local_identity != known.lockbox_id {
                return Err(ambiguous_name(value, &local, Vec::new()));
            }
            validate_target(&local, Some(known.lockbox_id), policy)?;
            return Ok((local, Some(known.lockbox_id)));
        }
        validate_target(&known.path, Some(known.lockbox_id), policy)?;
        return Ok((known.path, Some(known.lockbox_id)));
    }
    if let Some(local) = local {
        validate_target(&local, None, policy)?;
        return Ok((local, None));
    }
    if policy == TargetPolicy::MustBeMissing {
        let path = format!("{value}.lbox");
        validate_target(&path, None, policy)?;
        return Ok((path, None));
    }
    Err(cli_error(format!("unknown lockbox alias: {value}; no local lockbox found; use ./ or a .lbox extension to select a filename")))
}

fn ambiguous_name(
    value: &str,
    local: &str,
    details: Vec<(String, String)>,
) -> Box<dyn std::error::Error> {
    let filename = if local.ends_with(".lbox") {
        format!("./{local} or {local}")
    } else {
        format!("./{local}")
    };
    Box::new(CliMessage {
        exit_code: ExitCode::General,
        summary: format!("ambiguous lockbox name: {value}"),
        details,
        next_step: Some(format!(
            "Use a@{value} for the alias, or {filename} for the local filename."
        )),
    })
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
