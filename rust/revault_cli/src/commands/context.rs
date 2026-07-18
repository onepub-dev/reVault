use crate::secret_prompt::prompt_secret;
use revault_lockbox_api::vault_integration::VaultOpen;
use revault_lockbox_api::{
    ContactKeyPair, ContactPublicKey, Error, Lockbox, LockboxOpen, LockboxProtection, SecretVec,
};
use revault_vault_api::{
    auto_open_scope, default_vault_path, forget_platform_vault_password,
    get_platform_vault_password, import_public_key, local_vault, platform_secret_store_disabled,
    put_platform_vault_password, AutoOpenScope, NoopStore, SecretString, Vault, VaultDirectory,
};
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::error_output::ExitCode;

pub(crate) type CliResult<T> = Result<T, Box<dyn std::error::Error>>;
const MIN_VAULT_PASS_PHRASE_CHARS: usize = 15;

#[derive(Debug)]
pub(crate) struct CliMessage {
    pub(super) exit_code: ExitCode,
    pub(super) summary: String,
    pub(super) details: Vec<(String, String)>,
    pub(super) next_step: Option<String>,
}

impl CliMessage {
    pub(crate) fn exit_code(&self) -> ExitCode {
        self.exit_code
    }

    pub(crate) fn summary(&self) -> &str {
        &self.summary
    }

    pub(crate) fn details(&self) -> &[(String, String)] {
        &self.details
    }

    pub(crate) fn next_step(&self) -> Option<&str> {
        self.next_step.as_deref()
    }
}

impl fmt::Display for CliMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary)?;
        for (label, value) in &self.details {
            write!(f, ". {label}: {value}")?;
        }
        if let Some(next_step) = &self.next_step {
            write!(f, ". {next_step}")?;
        }
        Ok(())
    }
}

impl std::error::Error for CliMessage {}

pub(crate) fn cli_error(message: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(CliMessage {
        exit_code: ExitCode::General,
        summary: message.into(),
        details: Vec::new(),
        next_step: None,
    })
}

fn cli_diagnostic(
    exit_code: ExitCode,
    summary: impl Into<String>,
    details: Vec<(String, String)>,
    next_step: impl Into<String>,
) -> Box<dyn std::error::Error> {
    Box::new(CliMessage {
        exit_code,
        summary: summary.into(),
        details,
        next_step: Some(next_step.into()),
    })
}

pub(crate) enum Access {
    ContentKey(SecretVec),
    PromptPassword,
    CacheOnly,
}

pub(crate) fn open_existing(path: &str, access: &Access) -> CliResult<Lockbox> {
    ensure_lockbox_path_accessible(path)?;
    match access {
        Access::ContentKey(key) => {
            let _vault = default_vault()?;
            Ok(Vault::new(NoopStore)
                .open_lockbox_with(path, LockboxOpen::ContentKey(key.try_clone()?))?)
        }
        Access::PromptPassword => Err(cli_error(
            "password prompting is only used when creating a new lockbox; pass --key or open through the local vault",
        )),
        Access::CacheOnly => match local_vault().open_lockbox(path) {
            Ok(lockbox) => Ok(lockbox),
            Err(Error::VaultUnavailable(message)) if message.contains("no cached content key") => {
                match auto_open_lockbox(path) {
                    Ok(lockbox) => Ok(lockbox),
                    Err(AutoOpenLockboxError::Disabled) => Err(closed_lockbox_error(path, None)),
                    Err(AutoOpenLockboxError::Unavailable(reason)) => {
                        Err(closed_lockbox_error(path, Some(reason)))
                    }
                }
            }
            Err(err) => Err(err.into()),
        },
    }
}

enum AutoOpenLockboxError {
    Disabled,
    Unavailable(Error),
}

fn closed_lockbox_error(path: &str, reason: Option<Error>) -> Box<dyn std::error::Error> {
    let mut details = vec![("Lockbox".to_string(), path.to_string())];
    let next_step = match reason {
        Some(Error::UnsupportedFormatVersion {
            artifact: revault_lockbox_api::ArtifactKind::Vault,
            found,
            supported,
        }) if found < supported => {
            details.push((
                "Auto-open".to_string(),
                format!(
                    "Your local vault uses format version {found}; this reVault build uses version {supported}."
                ),
            ));
            "Migrate the vault, then retry:\n  lbx migrate vault --replace".to_string()
        }
        Some(Error::UnsupportedFormatVersion {
            artifact: revault_lockbox_api::ArtifactKind::Vault,
            found,
            supported,
        }) => {
            details.push((
                "Auto-open".to_string(),
                format!(
                    "Your local vault uses format version {found}; this reVault build supports version {supported}."
                ),
            ));
            "Install a newer reVault release, then retry.".to_string()
        }
        Some(reason) => {
            details.push(("Auto-open".to_string(), reason.to_string()));
            format!("Open the lockbox explicitly:\n  lbx open {path}")
        }
        None => format!("Open the lockbox first:\n  lbx open {path}"),
    };
    cli_diagnostic(
        ExitCode::LockboxClosed,
        "Lockbox is closed",
        details,
        next_step,
    )
}

fn auto_open_lockbox(path: &str) -> Result<Lockbox, AutoOpenLockboxError> {
    let scope = auto_open_scope().map_err(AutoOpenLockboxError::Unavailable)?;
    if scope != AutoOpenScope::Lockboxes {
        return Err(AutoOpenLockboxError::Disabled);
    }
    let password = revault_lockbox_api::SecretString::try_from_env("LOCKBOX_VAULT_PASSWORD")
        .map_err(|err| AutoOpenLockboxError::Unavailable(err.into()))?
        .or(get_platform_vault_password().unwrap_or_default())
        .ok_or_else(|| {
            AutoOpenLockboxError::Unavailable(Error::VaultUnavailable(
                "vault pass phrase is not stored for auto-open".to_string(),
            ))
        })?;
    let vault = VaultDirectory::open_or_create_default(&password)
        .map_err(AutoOpenLockboxError::Unavailable)?;
    let lockbox_id =
        VaultOpen::read_lockbox_id(Path::new(path)).map_err(AutoOpenLockboxError::Unavailable)?;
    if let Some(lockbox_password) = vault
        .remembered_lockbox_password(lockbox_id)
        .map_err(AutoOpenLockboxError::Unavailable)?
    {
        if let Ok(lockbox) =
            Vault::new(NoopStore).open_lockbox_with_password(path, &lockbox_password)
        {
            let _ = local_vault().open_lockbox_with_password(path, &lockbox_password);
            return Ok(lockbox);
        }
    }
    let profiles = vault
        .list_private_keys()
        .map_err(AutoOpenLockboxError::Unavailable)?;
    for profile in profiles {
        let Ok(keypair) = vault.load_private_key(&profile) else {
            continue;
        };
        let Ok(signing_key) = vault.load_owner_signing_key(&profile) else {
            continue;
        };
        let Ok(lockbox) = Lockbox::open_for_write(
            Path::new(path),
            LockboxOpen::ContactKeyPair(keypair),
            &signing_key,
        ) else {
            continue;
        };
        let Ok(cache_keypair) = vault.load_private_key(&profile) else {
            return Ok(lockbox);
        };
        if local_vault()
            .open_lockbox_with(path, LockboxOpen::ContactKeyPair(cache_keypair))
            .is_ok()
        {
            return match local_vault().open_lockbox(path) {
                Ok(cached) => Ok(cached),
                Err(_) => Ok(lockbox),
            };
        }
        return Ok(lockbox);
    }
    Err(AutoOpenLockboxError::Unavailable(Error::VaultUnavailable(
        "no remembered pass phrase or vault profile could open it".to_string(),
    )))
}

pub(crate) fn open_or_create(path: &str, access: &Access) -> CliResult<Lockbox> {
    if Path::new(path).exists() {
        open_existing(path, access)
    } else {
        match access {
            Access::ContentKey(key) => {
                let _vault = default_vault()?;
                let lockbox = Vault::new(NoopStore)
                    .create_lockbox(path, LockboxProtection::ContentKey(key.try_clone()?))?;
                mirror_key_directory(&lockbox, path)?;
                Ok(lockbox)
            }
            Access::PromptPassword => {
                let password = read_new_password().map_err(|err| Error::Io(err.to_string()))?;
                let lockbox = local_vault().create_lockbox_with_password(path, &password)?;
                mirror_key_directory(&lockbox, path)?;
                Ok(lockbox)
            }
            Access::CacheOnly => Err(cli_error(format!("lockbox not found: {path}"))),
        }
    }
}

pub(crate) fn ensure_lockbox_path_accessible(path: &str) -> CliResult<()> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => {
            Err(cli_error(format!("lockbox path is a directory: {path}")))
        }
        Ok(_) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            Err(cli_error(format!("lockbox not found: {path}")))
        }
        Err(err) if err.kind() == io::ErrorKind::PermissionDenied => Err(cli_error(format!(
            "permission denied reading lockbox: {path}"
        ))),
        Err(err) => Err(cli_error(format!("cannot access lockbox {path}: {err}"))),
    }
}

pub(crate) fn require_arg<'a>(args: &'a [String], index: usize, name: &str) -> CliResult<&'a str> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| Error::InvalidInput(format!("missing {name}")).into())
}

pub(crate) fn read_password(prompt: &str) -> CliResult<SecretString> {
    if let Some(password) = SecretString::try_from_env("LOCKBOX_PASSWORD")? {
        return Ok(password);
    }
    Ok(prompt_secret(prompt)?)
}

pub(crate) fn read_new_password() -> CliResult<SecretString> {
    if let Some(password) = SecretString::try_from_env("LOCKBOX_PASSWORD")? {
        return Ok(password);
    }
    let password = prompt_secret("New password: ")?;
    let mut confirm = prompt_secret("Confirm password: ")?;
    if password != confirm {
        confirm.zeroize()?;
        return Err(Error::InvalidInput("passwords do not match".to_string()).into());
    }
    confirm.zeroize()?;
    Ok(password)
}

pub(crate) fn read_vault_password(prompt: &str) -> CliResult<SecretString> {
    if let Some(password) = SecretString::try_from_env("LOCKBOX_VAULT_PASSWORD")? {
        return Ok(password);
    }
    Ok(prompt_secret(prompt)?)
}

pub(crate) fn read_new_vault_password() -> CliResult<SecretString> {
    read_new_vault_password_with_cancel("vault init")
}

fn read_new_vault_password_with_cancel(cancel_action: &str) -> CliResult<SecretString> {
    if let Some(password) = SecretString::try_from_env("LOCKBOX_VAULT_PASSWORD")? {
        validate_new_vault_pass_phrase(&password)?;
        return Ok(password);
    }
    match read_vault_passphrase_mode(cancel_action)?.as_str() {
        "" | "1" => read_generated_vault_pass_phrase(),
        "2" => read_manual_vault_pass_phrase(),
        "3" => Err(Error::InvalidInput(format!("{cancel_action} cancelled")).into()),
        value => {
            Err(Error::InvalidInput(format!("unknown vault passphrase choice: {value}")).into())
        }
    }
}

pub(crate) fn read_replacement_vault_password() -> CliResult<SecretString> {
    if let Some(password) = SecretString::try_from_env("LOCKBOX_NEW_VAULT_PASSWORD")? {
        validate_new_vault_pass_phrase(&password)?;
        return Ok(password);
    }
    read_new_vault_password_with_cancel("passphrase change")
}

fn read_vault_passphrase_mode(cancel_action: &str) -> CliResult<String> {
    println!("Vault passphrase:");
    println!("  1. Generate a strong passphrase");
    println!("  2. Enter my own passphrase");
    println!("  3. Cancel {cancel_action}");
    print!("Choose [1]: ");
    io::stdout().flush()?;
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    Ok(choice.trim().to_string())
}

fn read_generated_vault_pass_phrase() -> CliResult<SecretString> {
    let phrase = generated_vault_pass_phrase()?;
    println!();
    println!("Generated vault passphrase:");
    println!();
    println!("  {phrase}");
    println!();
    println!("Store this in your password manager before continuing.");
    println!();
    let password = SecretString::try_from_bytes(phrase.as_bytes().to_vec())?;
    validate_new_vault_pass_phrase(&password)?;
    if !confirm_generated_vault_pass_phrase_stored()? {
        return Err(Error::InvalidInput(
            "vault passphrase was not confirmed as stored".to_string(),
        )
        .into());
    }
    Ok(password)
}

fn confirm_generated_vault_pass_phrase_stored() -> CliResult<bool> {
    print!("Continue after storing it? [y/N]: ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes" | "YES" | "Yes"))
}

fn read_manual_vault_pass_phrase() -> CliResult<SecretString> {
    let password = prompt_secret("New vault passphrase (minimum 15 characters): ")?;
    validate_new_vault_pass_phrase(&password)?;
    let mut confirm = prompt_secret("Confirm vault passphrase: ")?;
    if password != confirm {
        confirm.zeroize()?;
        return Err(Error::InvalidInput("pass phrases do not match".to_string()).into());
    }
    confirm.zeroize()?;
    Ok(password)
}

fn generated_vault_pass_phrase() -> CliResult<String> {
    const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";
    let mut out = String::with_capacity(24);
    let mut bytes = [0u8; 20];
    getrandom::fill(&mut bytes).map_err(|err| Error::Io(err.to_string()))?;
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 && index % 4 == 0 {
            out.push('-');
        }
        out.push(ALPHABET[(byte & 0b0001_1111) as usize] as char);
    }
    bytes.fill(0);
    Ok(out)
}

fn validate_new_vault_pass_phrase(password: &SecretString) -> CliResult<()> {
    let chars = password.with_str(|text| text.chars().count())?;
    if chars < MIN_VAULT_PASS_PHRASE_CHARS {
        return Err(Error::InvalidInput(format!(
            "vault passphrase must be at least {MIN_VAULT_PASS_PHRASE_CHARS} characters"
        ))
        .into());
    }
    Ok(())
}

pub(crate) fn remember_default_vault_password(password: &SecretString) -> Result<(), Error> {
    if !platform_secret_store_disabled()? {
        put_platform_vault_password(password)?;
    }
    Ok(())
}

pub(crate) fn remember_default_vault_password_with_warning(password: &SecretString, success: &str) {
    if let Err(err) = remember_default_vault_password(password) {
        eprintln!(
            "WARNING: {success}, but its passphrase could not be stored in the platform secret store. You will be prompted again."
        );
        eprintln!("Platform secret-store error: {err}");
    }
}

pub(crate) fn default_vault() -> CliResult<VaultDirectory> {
    if auto_open_scope()? != revault_vault_api::AutoOpenScope::Off {
        // Start before opening the vault so the agent cannot inherit a vault
        // file lock. Failure is non-fatal: CI and agentless use remain valid.
        let _ = revault_vault_api::start();
    }
    let platform_enabled = !platform_secret_store_disabled()?;
    if platform_enabled {
        if let Ok(Some(password)) = get_platform_vault_password() {
            match open_default_vault_with_password(&password) {
                Ok(vault) => return Ok(vault),
                Err(_) => {
                    let _ = forget_platform_vault_password();
                }
            }
        }
    }

    let vault_id = default_vault_path()?.to_string_lossy().into_owned();
    if let Ok(Some(password)) = revault_vault_api::get_vault_unlock_key(&vault_id) {
        if let Ok(vault) = open_default_vault_with_password(&password) {
            return Ok(vault);
        }
        let _ = revault_vault_api::forget_vault_unlock_key(&vault_id);
    }

    if let Some(password) = SecretString::try_from_env("LOCKBOX_VAULT_PASSWORD")? {
        return open_default_vault_with_password(&password);
    }

    let password =
        prompt_secret("Vault pass phrase: ").map_err(|err| Error::Io(err.to_string()))?;
    let vault = open_default_vault_with_password(&password)?;
    if platform_enabled {
        remember_default_vault_password_with_warning(&password, "the vault opened successfully");
    }
    Ok(vault)
}

/// Resolves the configured vault password without opening the native vault.
/// Migration uses this before selecting the historical reader for the source
/// format.
pub(crate) fn vault_password_without_open() -> CliResult<SecretString> {
    if !platform_secret_store_disabled()? {
        if let Ok(Some(password)) = get_platform_vault_password() {
            return Ok(password);
        }
    }
    let vault_id = default_vault_path()?.to_string_lossy().into_owned();
    if let Ok(Some(password)) = revault_vault_api::get_vault_unlock_key(&vault_id) {
        return Ok(password);
    }
    if let Some(password) = SecretString::try_from_env("LOCKBOX_VAULT_PASSWORD")? {
        return Ok(password);
    }
    prompt_secret("Vault pass phrase: ").map_err(|err| Error::Io(err.to_string()).into())
}

pub(crate) fn open_default_vault_with_password(
    password: &SecretString,
) -> CliResult<VaultDirectory> {
    match VaultDirectory::open_or_create_default(password) {
        Ok(vault) => {
            let vault_id = default_vault_path()?.to_string_lossy().into_owned();
            let _ = revault_vault_api::put_vault_unlock_key(
                &vault_id,
                password.try_clone()?,
                None,
            );
            Ok(vault)
        }
        Err(Error::InvalidKey | Error::CorruptHeader) => Err(cli_error(
            "vault open failed: check the vault pass phrase. If the pass phrase is correct, the local vault file may be damaged",
        )),
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn ensure_default_vault_initialized() -> Result<(), Error> {
    if default_vault_path()?.exists() {
        return Ok(());
    }
    Err(Error::VaultUnavailable(
        "local vault is not initialized; run `lockbox vault init` first".to_string(),
    ))
}

pub(crate) fn mirror_key_directory(lockbox: &Lockbox, path: impl AsRef<Path>) -> CliResult<()> {
    if lockbox.list_key_slots().is_empty() {
        return Ok(());
    }
    ensure_default_vault_initialized()?;
    let vault = default_vault()?;
    mirror_key_directory_with_vault(lockbox, path, &vault)
}

pub(crate) fn mirror_key_directory_with_vault(
    lockbox: &Lockbox,
    path: impl AsRef<Path>,
    vault: &VaultDirectory,
) -> CliResult<()> {
    if lockbox.list_key_slots().is_empty() {
        return Ok(());
    }
    let backup = VaultOpen::export_key_directory_backup(lockbox)?;
    vault.store_key_directory_backup(lockbox.lockbox_id(), &backup)?;
    vault.remember_known_lockbox(lockbox.lockbox_id(), path)?;
    Ok(())
}

pub(crate) fn load_private_key_from_arg(arg: Option<&str>) -> CliResult<ContactKeyPair> {
    let vault = default_vault()?;
    let name_or_path = arg.unwrap_or(VaultDirectory::DEFAULT_KEY_NAME);
    Ok(vault.load_private_key(name_or_path)?)
}

pub(crate) struct ResolvedContact {
    pub(crate) name: Option<String>,
    pub(crate) public_key: ContactPublicKey,
}

pub(crate) fn load_contact_file(name: &str, path: &str) -> CliResult<ResolvedContact> {
    Ok(ResolvedContact {
        name: Some(name.to_string()),
        public_key: import_public_key(&std::fs::read(path)?)?,
    })
}

pub(crate) fn load_contact_from_arg(arg: &str) -> CliResult<ResolvedContact> {
    if std::path::Path::new(arg).exists() {
        return Ok(ResolvedContact {
            name: None,
            public_key: import_public_key(&std::fs::read(arg)?)?,
        });
    }
    let vault = default_vault()?;
    load_contact_from_vault(arg, &vault)
}

pub(crate) fn load_contact_from_vault(
    arg: &str,
    vault: &VaultDirectory,
) -> CliResult<ResolvedContact> {
    if std::path::Path::new(arg).exists() {
        return Ok(ResolvedContact {
            name: None,
            public_key: import_public_key(&std::fs::read(arg)?)?,
        });
    }
    if let Some(name) = arg.strip_prefix("profile:") {
        if name.is_empty() {
            return Err(cli_error("missing profile name after profile:"));
        }
        return Ok(ResolvedContact {
            name: Some(format!("profile:{name}")),
            public_key: vault.load_private_key(name)?.public_key(),
        });
    }
    if let Some(name) = arg.strip_prefix("contact:") {
        if name.is_empty() {
            return Err(cli_error("missing contact name after contact:"));
        }
        return Ok(ResolvedContact {
            name: Some(format!("contact:{name}")),
            public_key: vault.load_contact(name)?,
        });
    }
    let is_profile = vault.private_key_exists(arg)?;
    let is_contact = vault.contact_exists(arg)?;
    match (is_profile, is_contact) {
        (true, true) => Err(cli_error(format!(
            "ambiguous access target: {arg} matches both a profile and a contact. Use profile:{arg} or contact:{arg}."
        ))),
        (true, false) => Ok(ResolvedContact {
            name: Some(arg.to_string()),
            public_key: vault.load_private_key(arg)?.public_key(),
        }),
        (false, true) => Ok(ResolvedContact {
            name: Some(arg.to_string()),
            public_key: vault.load_contact(arg)?,
        }),
        (false, false) => Err(cli_error(format!(
            "profile or contact not found: {arg}. Use a saved profile, saved contact, or pass a name with a public key file."
        ))),
    }
}
