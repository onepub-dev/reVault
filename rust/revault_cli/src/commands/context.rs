use crate::secret_prompt::prompt_secret;
use revault_lockbox_api::vault_integration::VaultOpen;
use revault_lockbox_api::{
    ArtifactKind, ContactKeyPair, ContactPublicKey, Error, Lockbox, LockboxOpen, LockboxProtection,
    SecretVec,
};
use revault_vault_api::{
    auto_open_scope, default_vault_path, get_platform_vault_password, import_public_key,
    local_vault, platform_secret_store_disabled, put_platform_vault_password, AutoOpenScope,
    NoopStore, SecretString, Vault, VaultDirectory,
};
use std::fmt;
use std::fs;
use std::io::{self, IsTerminal, Write};
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
    let mut lockbox = match access {
        Access::ContentKey(key) => {
            let _vault = default_vault()?;
            Vault::new(NoopStore)
                .open_lockbox_with(path, LockboxOpen::ContentKey(key.try_clone()?))?
        }
        Access::PromptPassword => {
            return Err(cli_error(
                "password prompting is only used when creating a new lockbox; pass --key or open through the local vault",
            ));
        }
        Access::CacheOnly => match local_vault().open_lockbox(path) {
            Ok(lockbox) => lockbox,
            Err(Error::VaultUnavailable(message)) if message.contains("no cached content key") => {
                match auto_open_lockbox(path) {
                    Ok(lockbox) => lockbox,
                    Err(AutoOpenLockboxError::Disabled) => {
                        return Err(closed_lockbox_error(path, None));
                    }
                    Err(AutoOpenLockboxError::Unavailable(reason)) => {
                        return Err(closed_lockbox_error(path, Some(reason)));
                    }
                }
            }
            Err(err) => return Err(err.into()),
        },
    };
    attach_established_owner_signing_key(&mut lockbox);
    Ok(lockbox)
}

pub(crate) fn open_existing_read_only(
    path: &str,
    access: &Access,
) -> CliResult<Lockbox<revault_lockbox_api::ReadOnly>> {
    ensure_lockbox_path_accessible(path)?;
    match access {
        Access::ContentKey(key) => Ok(Lockbox::open(
            Path::new(path),
            LockboxOpen::ContentKey(key.try_clone()?),
        )?),
        Access::PromptPassword => Err(cli_error(
            "password prompting is only used when creating a new lockbox; pass --key or open through the local vault",
        )),
        Access::CacheOnly => Ok(local_vault().open_lockbox_read_only(path)?),
    }
}

fn attach_established_owner_signing_key(lockbox: &mut Lockbox) {
    let Ok(vault) = default_vault() else {
        return;
    };
    let Ok(profiles) = vault.list_private_keys() else {
        return;
    };
    for profile in profiles {
        let Ok(history) = vault.list_profile_generations(&profile) else {
            continue;
        };
        for generation in history.generations {
            let Ok(signing_key) =
                vault.load_owner_signing_key_generation(&profile, generation.index)
            else {
                continue;
            };
            if lockbox
                .owner_signing_key_matches(&signing_key)
                .unwrap_or(false)
            {
                lockbox.set_owner_signing_key(signing_key);
                return;
            }
        }
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
            artifact: revault_lockbox_api::ArtifactKind::Lockbox,
            found,
            supported,
        }) if found < supported => {
            details.push((
                "Auto-open".to_string(),
                format!(
                    "Your local vault uses Lockbox container format {found}; this reVault build uses container format {supported}."
                ),
            ));
            "Migrate the vault, then retry:\n  lbx migrate vault --replace".to_string()
        }
        Some(Error::UnsupportedFormatVersion {
            artifact: revault_lockbox_api::ArtifactKind::Lockbox,
            found,
            supported,
        }) => {
            details.push((
                "Auto-open".to_string(),
                format!(
                    "Your local vault uses Lockbox container format {found}; this reVault build supports container format {supported}."
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
    let password = match revault_lockbox_api::SecretString::try_from_env("LOCKBOX_VAULT_PASSWORD")
        .map_err(|err| AutoOpenLockboxError::Unavailable(err.into()))?
    {
        Some(password) => Some(password),
        None => get_platform_vault_password().map_err(AutoOpenLockboxError::Unavailable)?,
    }
    .ok_or_else(|| {
        AutoOpenLockboxError::Unavailable(Error::VaultUnavailable(
            "Vault passphrase is not stored for Auto Open".to_string(),
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
            Err(Error::InvalidInput(format!("unknown Vault passphrase choice: {value}")).into())
        }
    }
}

pub(crate) fn read_replacement_vault_password() -> CliResult<SecretString> {
    read_new_secondary_vault_password("passphrase change")
}

pub(crate) fn read_new_secondary_vault_password(cancel_action: &str) -> CliResult<SecretString> {
    if let Some(password) = SecretString::try_from_env("LOCKBOX_NEW_VAULT_PASSWORD")? {
        validate_new_vault_pass_phrase(&password)?;
        return Ok(password);
    }
    if !io::stdin().is_terminal() {
        return Err(cli_error(
            "new Vault passphrase is unavailable; set LOCKBOX_NEW_VAULT_PASSWORD",
        ));
    }
    read_new_vault_password_with_cancel(cancel_action)
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
    println!("Generated Vault passphrase:");
    println!();
    println!("  {phrase}");
    println!();
    println!("Store this in your password manager before continuing.");
    println!();
    let password = SecretString::try_from_bytes(phrase.as_bytes().to_vec())?;
    validate_new_vault_pass_phrase(&password)?;
    if !confirm_generated_vault_pass_phrase_stored()? {
        return Err(Error::InvalidInput(
            "Vault passphrase was not confirmed as stored".to_string(),
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
    let password = prompt_secret("New Vault passphrase (minimum 15 characters): ")?;
    validate_new_vault_pass_phrase(&password)?;
    let mut confirm = prompt_secret("Confirm Vault passphrase: ")?;
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
            "Vault passphrase must be at least {MIN_VAULT_PASS_PHRASE_CHARS} characters"
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
            "WARNING: {success}, but its passphrase could not be stored in the platform credential store. You will be prompted again."
        );
        eprintln!("Platform credential store error: {err}");
    }
}

pub(crate) fn default_vault() -> CliResult<VaultDirectory> {
    if auto_open_scope()? != revault_vault_api::AutoOpenScope::Off {
        // Start before opening the vault so the agent cannot inherit a vault
        // file lock. Failure is non-fatal: CI and agentless use remain valid.
        let _ = revault_vault_api::start();
    }
    let platform_enabled = !platform_secret_store_disabled()?;
    if let Some(password) = SecretString::try_from_env("LOCKBOX_VAULT_PASSWORD")? {
        return open_default_vault_with_password(&password);
    }
    if platform_enabled {
        if let Ok(Some(password)) = get_platform_vault_password() {
            // A stored credential is authoritative. Preserve and report any
            // open error: it may describe a required migration, damaged data,
            // or an unavailable file rather than an invalid passphrase.
            return open_default_vault_with_password(&password);
        }
    }

    let vault_id = default_vault_path()?.to_string_lossy().into_owned();
    if let Ok(Some(password)) = revault_vault_api::get_vault_unlock_key(&vault_id) {
        return open_default_vault_with_password(&password);
    }

    let password = prompt_secret("Vault passphrase: ").map_err(|err| Error::Io(err.to_string()))?;
    if platform_enabled {
        // Entering the Vault passphrase explicitly authorizes Auto Open to
        // remember it. Store it before opening so a required format migration
        // does not cause every subsequent command, including migration, to
        // prompt for the same passphrase again.
        remember_default_vault_password_with_warning(&password, "the passphrase was entered");
    }
    open_default_vault_with_password(&password)
}

/// Resolves the configured Vault passphrase without opening the Vault.
/// Migration uses this before selecting the historical reader for the source
/// format.
pub(crate) fn vault_password_without_open() -> CliResult<SecretString> {
    if let Some(password) = SecretString::try_from_env("LOCKBOX_VAULT_PASSWORD")? {
        return Ok(password);
    }
    if !platform_secret_store_disabled()? {
        if let Ok(Some(password)) = get_platform_vault_password() {
            return Ok(password);
        }
    }
    let vault_id = default_vault_path()?.to_string_lossy().into_owned();
    if let Ok(Some(password)) = revault_vault_api::get_vault_unlock_key(&vault_id) {
        return Ok(password);
    }
    prompt_secret("Vault passphrase: ").map_err(|err| Error::Io(err.to_string()).into())
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
        Err(Error::UnsupportedFormatVersion {
            artifact: ArtifactKind::Lockbox,
            found,
            supported,
        }) => Err(cli_diagnostic(
            ExitCode::UnsupportedFormat,
            "Unsupported Vault container format",
            vec![(
                "Details".to_string(),
                format!(
                    "Found Lockbox container version {found}; this reVault build supports container version {supported}. The encrypted Vault structure is detected separately during migration."
                ),
            )],
            "Run `lbx migrate vault --output <directory>` or use `--replace`.",
        )),
        Err(err) => match err {
            Error::InvalidKey | Error::CorruptHeader => Err(cli_error(
                "Vault open failed: check the Vault passphrase. If the passphrase is correct, the Vault file may be damaged",
            )),
            err => Err(err.into()),
        },
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
