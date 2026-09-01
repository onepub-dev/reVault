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
use std::path::Path;

pub(crate) fn run_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    if let Some((command, command_matches)) = matches.subcommand() {
        match command {
            "recover" => {
                if command_lockbox().is_none() {
                    return Err(cli_error(
                        "doctor recover requires a lockbox path before `doctor`",
                    ));
                }
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
    match command_lockbox() {
        Some(lockbox) => run_lockbox(&lockbox, access, matches.get_flag("verbose")),
        None => run_global(),
    }
}

fn run_global() -> CliResult<()> {
    let vault_path = default_vault_path()?;
    println!("reVault");
    println!("  version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Local vault");
    println!("  path: {}", vault_path.display());
    println!("  exists: {}", yes_no(vault_path.exists()));
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
    match default_vault_noninteractive() {
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
                    println!("      run: lockbox vault lockbox forget {path}");
                }
            }
        }
        Ok(None) => {
            println!("  not checked: vault is closed");
        }
        Err(err) => {
            println!("  not checked: {err}");
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

    let inspection = Lockbox::inspect_file(path)?;
    println!("Lockbox");
    println!("  path: {lockbox_path}");
    println!("  size: {}", human_size(metadata.len()));
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
    print_revault_vault_api(&inspection);
    println!();
    print_encrypted_content(lockbox_path, access, verbose);
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
    println!("Configured access (public header)");
    println!("  pass phrase slots: {password_count}");
    println!("  contact-key slots: {contact_count}");
    if !inspection.key_slots.is_empty() {
        println!("  slots:");
        for slot in &inspection.key_slots {
            println!("    {}: {}", slot.id, slot_protection(slot.protection));
        }
    }
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

fn print_revault_vault_api(inspection: &LockboxFileInspection) {
    println!("Local vault");
    match default_vault_noninteractive() {
        Ok(Some(vault)) => {
            println!("  open: yes");
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
        }
        Ok(None) => {
            println!("  open: no");
            println!("  key-directory backup: not checked");
        }
        Err(err) => {
            println!("  open: no");
            println!("  key-directory backup: not checked: {err}");
        }
    }
}

fn print_encrypted_content(lockbox_path: &str, access: &Access, verbose: bool) {
    println!("Encrypted content");
    match open_existing_read_only(lockbox_path, access) {
        Ok(lockbox) => {
            println!("  state: healthy");
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
            if matches!(err.downcast_ref::<Error>(), Some(Error::VaultUnavailable(message)) if message.contains("no cached content key"))
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

fn slot_protection(protection: LockboxKeySlotProtection) -> &'static str {
    match protection {
        LockboxKeySlotProtection::Password => "pass phrase",
        LockboxKeySlotProtection::Contact => "contact key",
        _ => "unknown",
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
    if let Some(password) = SecretString::try_from_env("LOCKBOX_VAULT_PASSWORD")? {
        return Ok(Some(VaultDirectory::open_or_create_default(&password)?));
    }
    if !platform_secret_store_disabled()? {
        if let Ok(Some(password)) = get_platform_vault_password() {
            return Ok(Some(VaultDirectory::open_or_create_default(&password)?));
        }
    }
    Ok(None)
}
