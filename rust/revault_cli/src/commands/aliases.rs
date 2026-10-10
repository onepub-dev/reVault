use super::context::{cli_error, default_vault, CliResult};
use revault_lockbox_api::{Lockbox, LockboxId};

pub(crate) fn validate_name(name: &str) -> CliResult<()> {
    revault_vault_api::validate_vault_record_name(name)?;
    if name.len() > 128 {
        return Err(cli_error("lockbox alias exceeds 128 bytes"));
    }
    Ok(())
}

pub(crate) fn resolve_noninteractive(value: &str) -> CliResult<(String, Option<LockboxId>)> {
    let Some(name) = value.strip_prefix("a@") else {
        return Ok((value.to_owned(), None));
    };
    let vault = super::completion::read_only_vault()
        .ok_or_else(|| cli_error("Vault is locked; open it with lbx first"))?;
    let known = vault.resolve_lockbox_alias(name)?;
    check_identity(&known.path, known.lockbox_id)?;
    Ok((known.path, Some(known.lockbox_id)))
}

/// Only a leading a@ is special. ./a@name always remains a host filename.
pub(crate) fn resolve(value: &str) -> CliResult<(String, Option<LockboxId>)> {
    let Some(name) = value.strip_prefix("a@") else {
        return Ok((value.to_string(), None));
    };
    let known = default_vault()?.resolve_lockbox_alias(name)?;
    check_identity(&known.path, known.lockbox_id)?;
    Ok((known.path, Some(known.lockbox_id)))
}

pub(crate) fn check_identity(path: &str, expected: LockboxId) -> CliResult<()> {
    if Lockbox::inspect_file(path)?.lockbox_id != expected {
        return Err(cli_error(
            "alias target identity changed; explicitly update the alias",
        ));
    }
    Ok(())
}
