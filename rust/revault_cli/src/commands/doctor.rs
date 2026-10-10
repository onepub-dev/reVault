use super::command_lockbox;
use super::context::{cli_error, open_existing_read_only, Access, CliResult};
use super::output::human_size;
use clap::ArgMatches;
use revault_lockbox_api::{Error, Lockbox, LockboxFileInspection, LockboxKeySlotProtection};
use revault_vault_api::{
    agent_log_destination, agent_sleep_support, default_vault_path, get_platform_vault_password,
    is_running, list, platform_secret_store_disabled, platform_secret_store_status,
    verify_agent_transport_security, SecretString, VaultDirectory,
};
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;

pub(crate) fn run_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    if let Some((command, command_matches)) = matches.subcommand() {
        match command {
            "recover" => {
                super::default_lockbox_for_command()?;
                return super::recovery::run_matches(command_matches, access);
            }
            "migrate" => return super::migrate::run_matches(command_matches, access),
            other => {
                return Err(cli_error(format!(
                    "unknown doctor maintenance command: {other}"
                )))
            }
        }
    }
    let selected = match command_lockbox() {
        Some(path) => Some(path),
        None if super::session::default_lockbox_or_none()?.is_some() => {
            Some(super::default_lockbox_for_command()?)
        }
        None => None,
    };
    match selected {
        Some(lockbox) => run_lockbox(&lockbox, access, matches.get_flag("verbose")),
        None => run_global(),
    }
}

fn run_global() -> CliResult<()> {
    let vault_path = default_vault_path()?;
    println!("reVault");
    println!("  version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    let vault = print_local_vault(&vault_path);
    println!(
        "  readable: {}",
        yes_no(std::fs::File::open(&vault_path).is_ok())
    );
    println!(
        "  writable: {}",
        yes_no(if vault_path.exists() {
            OpenOptions::new().append(true).open(&vault_path).is_ok()
        } else {
            vault_path
                .parent()
                .and_then(|parent| parent.metadata().ok())
                .map(|metadata| !metadata.permissions().readonly())
                .unwrap_or(false)
        })
    );
    println!();
    let auto_open = platform_secret_store_status()?;
    println!("Auto-open");
    println!("  supported: {}", yes_no(auto_open.supported));
    println!("  scope: {}", auto_open.scope.as_str());
    println!("  backend: {}", auto_open.backend);
    println!();
    println!("Session Agent");
    let sleep_support = agent_sleep_support();
    println!(
        "  transport security: {}",
        if verify_agent_transport_security().is_ok() {
            "ok"
        } else {
            "unsupported"
        }
    );
    println!(
        "  suspend management: {}",
        yes_no(sleep_support.supported())
    );
    println!(
        "  suspend notifications: {}",
        yes_no(sleep_support.suspend_notifications)
    );
    println!(
        "  sleep prevention: {}",
        yes_no(sleep_support.sleep_inhibition)
    );
    println!("  running: {}", yes_no(is_running()));
    match list() {
        Ok(lockboxes) => println!("  open lockboxes: {}", lockboxes.len()),
        Err(err) => println!("  open lockboxes: unknown: {err}"),
    }
    println!("  log: {}", agent_log_destination());
    println!();
    println!("Known lockboxes");
    match vault {
        Ok(Some(vault)) => {
            let known = vault.list_known_lockboxes()?;
            let mut present = 0usize;
            let mut missing = Vec::new();
            let mut inaccessible = Vec::new();
            for lockbox in known {
                match std::fs::metadata(&lockbox.path) {
                    Ok(_) => present += 1,
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                        missing.push(lockbox.path)
                    }
                    Err(_) => inaccessible.push(lockbox.path),
                }
            }
            println!("  present: {present}");
            println!("  missing: {}", missing.len());
            println!("  inaccessible: {}", inaccessible.len());
            if !missing.is_empty() {
                println!("  missing paths:");
                for path in missing {
                    println!("    {path}");
                    println!("      run: lockbox vault lockboxes forget {path}");
                }
            }
        }
        Ok(None) => {
            println!("  not checked: vault is closed");
        }
        Err(_) => {
            println!("  not checked: vault is unavailable (see Local vault status)");
        }
    }
    Ok(())
}

fn run_lockbox(lockbox_path: &str, access: &Access, verbose: bool) -> CliResult<()> {
    let path = Path::new(lockbox_path);
    let metadata = std::fs::metadata(path).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            cli_error(format!("lockbox not found: {lockbox_path}"))
        } else if err.kind() == std::io::ErrorKind::PermissionDenied {
            cli_error(format!("permission denied reading lockbox: {lockbox_path}"))
        } else {
            cli_error(format!("cannot access lockbox {lockbox_path}: {err}"))
        }
    })?;
    if metadata.is_dir() {
        return Err(cli_error(format!(
            "lockbox path is a directory: {lockbox_path}"
        )));
    }

    super::context::ensure_current_lockbox_format(lockbox_path)?;
    let inspection = Lockbox::inspect_file(path)?;
    println!("Lockbox");
    println!("  format version: {}", container_format_version(path)?);
    println!("  path: {lockbox_path}");
    println!("  size: {}", human_size(metadata.len()));
    if let Some(options) = inspection.format_options {
        println!(
            "  encryption: {}",
            match options.encryption {
                revault_lockbox_api::EncryptionMode::None => "none",
                revault_lockbox_api::EncryptionMode::ChaCha20Poly1305 => "chacha20-poly1305",
            }
        );
        println!(
            "  signing: {}",
            match options.signing {
                revault_lockbox_api::SigningMode::None => "none",
                revault_lockbox_api::SigningMode::Owner => "owner",
            }
        );
        match options.compression {
            revault_lockbox_api::Compression::None => println!("  compression: none"),
            revault_lockbox_api::Compression::Zstd { level } => {
                println!("  compression: zstd (level {})", level.get())
            }
        }
    }
    if !inspection.header_readable {
        println!("  warning: primary header is damaged; backup metadata was used");
    }
    if verbose {
        println!();
        println!("Technical details");
        println!("  lockbox id: {}", inspection.lockbox_id);
        println!("  physical bytes: {}", metadata.len());
        println!("  primary header: {}", header_status(&inspection));
        println!(
            "  key directory: generation {}, {} readable copies",
            inspection.key_directory_generation, inspection.key_directory_copy_count
        );
    }
    println!();
    print_access_methods(&inspection);
    println!();
    print_lockbox_session(&inspection);
    println!();
    let vault = print_revault_vault_api(&inspection);
    println!();
    print_encrypted_content(lockbox_path, access, verbose, vault.as_ref(), &inspection);
    Ok(())
}

fn print_access_methods(inspection: &LockboxFileInspection) {
    let password_count = inspection
        .key_slots
        .iter()
        .filter(|slot| slot.protection == LockboxKeySlotProtection::Password)
        .count();
    let contact_count = inspection
        .key_slots
        .iter()
        .filter(|slot| slot.protection == LockboxKeySlotProtection::Contact)
        .count();
    println!("Configured access");
    println!("  pass phrase slots: {password_count}");
    println!("  contact-key slots: {contact_count}");
}

fn print_lockbox_session(inspection: &LockboxFileInspection) {
    println!("Session");
    match list() {
        Ok(lockboxes) => {
            let cached = lockboxes
                .iter()
                .find(|lockbox| lockbox.id == inspection.lockbox_id.to_string());
            println!("  open: {}", yes_no(cached.is_some()));
            if let Some(cached) = cached.and_then(|lockbox| lockbox.path.as_deref()) {
                println!("  cached path: {cached}");
            }
        }
        Err(err) => {
            println!("  open: unknown");
            println!("  session check: {err}");
        }
    }
}

fn container_format_version(path: &Path) -> CliResult<u16> {
    let mut header = Vec::new();
    std::fs::File::open(path)?
        .take(384)
        .read_to_end(&mut header)?;
    Ok(revault_lockbox_api::probe_lockbox_format_version(&header)?)
}

fn print_local_vault(path: &Path) -> CliResult<Option<VaultDirectory>> {
    println!("Local vault");
    let version = container_format_version(path);
    match &version {
        Ok(version) => println!("  container format version: {version}"),
        Err(error) => println!("  container format version: not read ({error})"),
    }
    // Probe only the stable discriminator through the read-only core reader.
    // This does not open historical Vault records for normal CLI operations.
    match &version {
        Ok(2 | 3) => {
            let structure = (|| -> CliResult<Option<u32>> {
                let Some(password) = vault_password_noninteractive()? else {
                    return Ok(None);
                };
                let root = path
                    .parent()
                    .ok_or_else(|| cli_error("vault path has no parent"))?;
                Ok(Some(VaultDirectory::probe_structure_version(
                    root, &password,
                )?))
            })();
            match structure {
                Ok(Some(version)) => println!("  structure version: {version}"),
                Ok(None) => println!("  structure version: not read (vault is closed or absent)"),
                Err(error) => println!(
                    "  structure version: not read ({})",
                    error.to_string().lines().collect::<Vec<_>>().join(" ")
                ),
            }
        }
        Ok(_) => println!("  structure version: not read (unsupported container)"),
        Err(_) => println!("  structure version: not read (container unavailable)"),
    }
    println!("  path: {}", path.display());
    println!("  exists: {}", yes_no(path.exists()));
    if let Ok(version) = version {
        if version != revault_lockbox_api::LOCKBOX_FORMAT_VERSION {
            if version < revault_lockbox_api::LOCKBOX_FORMAT_VERSION {
                println!("  status: upgrade required; run: lbx doctor migrate vault --replace; or migrate the Vault and all known Lockboxes: lbx doctor migrate all --replace");
            } else {
                println!("  status: unsupported container; upgrade reVault. Automatic downgrade is not supported");
            }
            return Err(cli_error("vault container is unsupported"));
        }
    }
    let vault = default_vault_noninteractive();
    match &vault {
        Ok(Some(_)) => {
            println!("  open: yes");
        }
        Ok(None) => {
            println!("  open: no");
        }
        Err(error) => println!(
            "  status: {}",
            error.to_string().lines().collect::<Vec<_>>().join(" ")
        ),
    }
    vault
}

fn print_revault_vault_api(inspection: &LockboxFileInspection) -> Option<VaultDirectory> {
    let vault = match default_vault_path() {
        Ok(path) => print_local_vault(&path),
        Err(error) => {
            println!("Local vault");
            println!("  container format version: not read");
            println!("  status: {error}");
            return None;
        }
    };
    match vault {
        Ok(Some(vault)) => {
            println!(
                "  key-directory backup: {}",
                yes_no(
                    vault
                        .load_key_directory_backup(inspection.lockbox_id)
                        .is_ok()
                )
            );
            match vault.list_private_keys() {
                Ok(keys) => println!("  profiles: {}", keys.len()),
                Err(err) => println!("  profiles: not checked: {err}"),
            }
            Some(vault)
        }
        Ok(None) => {
            println!("  key-directory backup: not checked");
            None
        }
        Err(_) => None,
    }
}

fn print_encrypted_content(
    lockbox_path: &str,
    access: &Access,
    verbose: bool,
    vault: Option<&VaultDirectory>,
    inspection: &LockboxFileInspection,
) {
    println!("Encrypted content");
    match open_existing_read_only(lockbox_path, access) {
        Ok(lockbox) => {
            println!("  state: healthy");
            // Credential names are local Vault labels, not archive metadata.
            // Reuse the credential-list API without prompting to open a Vault.
            if let Some(labels) =
                vault.and_then(|vault| vault.list_access_slot_labels(lockbox.lockbox_id()).ok())
            {
                let names: Vec<_> = labels
                    .iter()
                    .filter(|label| {
                        !label.name.is_empty()
                            && inspection
                                .key_slots
                                .iter()
                                .any(|slot| slot.id == label.slot_id)
                    })
                    .collect();
                if !names.is_empty() {
                    println!("  credential names (local vault):");
                    for label in names {
                        println!("    {}", label.name);
                    }
                }
            }
            match lockbox.description() {
                Ok(Some(description)) => {
                    let mut lines = description.lines();
                    println!("  description: {}", lines.next().unwrap_or_default());
                    for line in lines {
                        println!("               {line}");
                    }
                }
                Ok(None) => println!("  description: not set"),
                Err(err) => println!("  description: not checked: {err}"),
            }
            if verbose {
                println!("  transaction recovery: not required");
            }
        }
        Err(err) => {
            if matches!(
                err.downcast_ref::<Error>(),
                Some(Error::RecoveryRequired { .. })
            ) {
                println!("  state: cleanup required");
                println!("  preview: lbx {lockbox_path} doctor recover --dry-run");
                println!("  recover: lbx {lockbox_path} doctor recover");
                return;
            }
            if super::error_output::exit_code(err.as_ref())
                == super::error_output::ExitCode::LockboxClosed.as_i32()
            {
                println!("  state: not checked (lockbox is closed)");
                println!("  next: open the lockbox, then run doctor again to check its health:");
                println!("    lbx {lockbox_path} open");
                println!("    lbx {lockbox_path} doctor");
                return;
            }
            println!("  checks failed: {err}");
        }
    }
}

fn header_status(inspection: &LockboxFileInspection) -> &'static str {
    if inspection.header_readable {
        "ok"
    } else {
        "corrupt; recovered key-directory metadata"
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn default_vault_noninteractive() -> Result<Option<VaultDirectory>, Box<dyn std::error::Error>> {
    super::context::ensure_current_default_vault_format()?;
    if !default_vault_path()?.exists() {
        return Ok(None);
    }
    if let Some(password) = vault_password_noninteractive()? {
        return Ok(Some(
            VaultDirectory::open_or_create_default(&password)
                .map_err(super::context::vault_open_error)?,
        ));
    }
    Ok(None)
}

fn vault_password_noninteractive() -> CliResult<Option<SecretString>> {
    if let Some(password) = SecretString::try_from_env("LOCKBOX_VAULT_PASSWORD")? {
        return Ok(Some(password));
    }
    if !platform_secret_store_disabled()? {
        if let Ok(Some(password)) = get_platform_vault_password() {
            return Ok(Some(password));
        }
    }
    Ok(None)
}
