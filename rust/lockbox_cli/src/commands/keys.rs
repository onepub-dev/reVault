use super::context::{
    cli_error, default_vault, ensure_default_vault_initialized, load_contact_file,
    load_contact_from_arg, load_contact_from_vault, load_private_key_from_arg,
    mirror_key_directory, mirror_key_directory_with_vault, open_existing, read_new_password,
    read_password, require_arg, Access, CliResult,
};
use super::output::{output_format_from_matches, print_records, OutputFormat};
use super::session::clear_default_if_matches;
use super::{
    default_lockbox_for_command, looks_like_lockbox_path, optional_lockbox_positionals,
    optional_lockbox_value, positional_values,
};
use clap::ArgMatches;
use lockbox_core::vault_integration::VaultOpen;
use lockbox_core::{
    ContactKeyPair, ContactPublicKey, Error, Lockbox, LockboxKeySlotProtection, LockboxOpen,
    LockboxProtection,
};
use lockbox_vault::{
    auto_open_scope, encode_hex, export_private_key, list as list_open_lockboxes, local_vault,
    AutoOpenScope, KeyFormat, NoopStore, SecretVec, Vault, VaultDirectory,
};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn create_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let mut args = Vec::new();
    if matches.get_flag("password") {
        args.push("--password".to_string());
    }
    if let Some(contact) = matches.get_one::<String>("for") {
        args.push("--contact".to_string());
        args.push(contact.clone());
    }
    args.push(required_value(matches, "lockbox"));
    create(&args, access)
}

pub(crate) fn create(args: &[String], access: &Access) -> CliResult<()> {
    if args.first().map(String::as_str) == Some("--password") {
        let lockbox_path = create_path(require_arg(args, 1, "lockbox")?)?;
        ensure_new_lockbox_path(&lockbox_path)?;
        ensure_default_vault_initialized()?;
        let vault = default_vault()?;
        let signing_key = vault.load_owner_signing_key(VaultDirectory::DEFAULT_KEY_NAME)?;
        println!("Creating lockbox: {}", lockbox_path.display());
        let password = read_new_password()?;
        let lb = local_vault().create_lockbox_with_signing_key(
            &lockbox_path,
            LockboxProtection::Password(&password),
            &signing_key,
        )?;
        remember_lockbox_password_if_enabled_with_vault(&lb, &password, &vault)?;
        mirror_key_directory_with_vault(&lb, &lockbox_path, &vault)?;
        return Ok(());
    }
    if args.first().map(String::as_str) == Some("--contact") {
        let contact_name = require_arg(args, 1, "contact")?;
        let lockbox_path = create_path(require_arg(args, 2, "lockbox")?)?;
        ensure_new_lockbox_path(&lockbox_path)?;
        ensure_default_vault_initialized()?;
        let vault = default_vault()?;
        let signing_key = vault.load_owner_signing_key(VaultDirectory::DEFAULT_KEY_NAME)?;
        let contact = load_contact_from_vault(contact_name, &vault)?;
        println!("Creating lockbox: {}", lockbox_path.display());
        let lb = Vault::new(NoopStore).create_lockbox_with_signing_key(
            &lockbox_path,
            LockboxProtection::ContactPublicKey {
                name: contact.name.map(|name| access_entry_name(&name)),
                contact: contact.public_key,
            },
            &signing_key,
        )?;
        mirror_key_directory_with_vault(&lb, &lockbox_path, &vault)?;
        return Ok(());
    }
    let lockbox_path = create_path(require_arg(args, 0, "lockbox")?)?;
    ensure_new_lockbox_path(&lockbox_path)?;
    println!("Creating lockbox: {}", lockbox_path.display());
    match access {
        Access::ContentKey(key) => {
            let vault = default_vault()?;
            let signing_key = vault.load_owner_signing_key(VaultDirectory::DEFAULT_KEY_NAME)?;
            let lb = Vault::new(NoopStore).create_lockbox_with_signing_key(
                &lockbox_path,
                LockboxProtection::ContentKey(key.try_clone()?),
                &signing_key,
            )?;
            mirror_key_directory_with_vault(&lb, &lockbox_path, &vault)?;
        }
        Access::PromptPassword => {
            ensure_default_vault_initialized()?;
            let vault = default_vault()?;
            let signing_key = vault.load_owner_signing_key(VaultDirectory::DEFAULT_KEY_NAME)?;
            let contact = load_contact_from_vault(VaultDirectory::DEFAULT_KEY_NAME, &vault)?;
            let lb = Vault::new(NoopStore).create_lockbox_with_signing_key(
                &lockbox_path,
                LockboxProtection::ContactPublicKey {
                    name: contact.name.map(|name| access_entry_name(&name)),
                    contact: contact.public_key,
                },
                &signing_key,
            )?;
            mirror_key_directory_with_vault(&lb, &lockbox_path, &vault)?;
        }
        Access::CacheOnly => {
            return Err(Error::InvalidInput("create requires an open method".to_string()).into());
        }
    }
    Ok(())
}

pub(crate) fn open_matches(matches: &ArgMatches) -> CliResult<()> {
    open_options(OpenOptions::from_matches(matches)?)
}

fn open_options(options: OpenOptions) -> CliResult<()> {
    let inspection = Lockbox::inspect_file(&options.lockbox_path)?;
    let has_password_slot = inspection
        .key_slots
        .iter()
        .any(|slot| slot.protection == LockboxKeySlotProtection::Password);
    let vault = default_vault()?;
    if matches!(options.password_source, PasswordSource::Prompt) {
        if let Some(lb) = open_with_vault_identity(&options, &vault)? {
            mirror_key_directory_with_vault(&lb, &options.lockbox_path, &vault)?;
            println!("Lockbox opened: {}", options.lockbox_path);
            return Ok(());
        }
        if !has_password_slot {
            return Err(cli_error(format!(
                "none of the local vault identities can open {}; the lockbox has no password access",
                options.lockbox_path
            )));
        }
    }
    let password = options.read_password()?;
    let signing_key = vault.load_owner_signing_key(VaultDirectory::DEFAULT_KEY_NAME)?;
    let lb = if let Some(ttl_seconds) = options.ttl_seconds {
        local_vault().open_lockbox_with_for_duration_and_signing_key(
            &options.lockbox_path,
            LockboxOpen::Password(&password),
            ttl_seconds,
            &signing_key,
        )?
    } else {
        local_vault().open_lockbox_with_signing_key(
            &options.lockbox_path,
            LockboxOpen::Password(&password),
            &signing_key,
        )?
    };
    mirror_key_directory_with_vault(&lb, &options.lockbox_path, &vault)?;
    remember_lockbox_password_if_enabled_with_vault(&lb, &password, &vault)?;
    if let Some(ttl_seconds) = options.ttl_seconds {
        local_vault().cache_lockbox_password_for_duration(
            &options.lockbox_path,
            &password,
            ttl_seconds,
        )?;
    }
    println!("Lockbox opened: {}", options.lockbox_path);
    Ok(())
}

fn open_with_vault_identity(
    options: &OpenOptions,
    vault: &VaultDirectory,
) -> CliResult<Option<Lockbox>> {
    let mut identities = vault.list_private_keys()?;
    if let Some(index) = identities
        .iter()
        .position(|name| name == VaultDirectory::DEFAULT_KEY_NAME)
    {
        let default = identities.remove(index);
        identities.insert(0, default);
    }
    for identity in identities {
        let keypair = vault.load_private_key(&identity)?;
        let signing_key = vault.load_owner_signing_key(&identity)?;
        let opened = if let Some(ttl_seconds) = options.ttl_seconds {
            local_vault().open_lockbox_with_for_duration_and_signing_key(
                &options.lockbox_path,
                LockboxOpen::ContactKeyPair(keypair),
                ttl_seconds,
                &signing_key,
            )
        } else {
            local_vault().open_lockbox_with_signing_key(
                &options.lockbox_path,
                LockboxOpen::ContactKeyPair(keypair),
                &signing_key,
            )
        };
        match opened {
            Ok(lockbox) => return Ok(Some(lockbox)),
            Err(Error::InvalidKey) => {}
            Err(err) => return Err(err.into()),
        }
    }
    Ok(None)
}

fn remember_lockbox_password_if_enabled_with_vault(
    lockbox: &Lockbox,
    password: &lockbox_vault::SecretString,
    vault: &VaultDirectory,
) -> CliResult<()> {
    if auto_open_scope()? == AutoOpenScope::Lockboxes {
        vault.remember_lockbox_password(lockbox.lockbox_id(), password)?;
    }
    Ok(())
}

pub(crate) fn close_matches(matches: &ArgMatches) -> CliResult<()> {
    close(&[optional_lockbox_value(matches, "lockbox")?])
}

pub(crate) fn close(args: &[String]) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    if lockbox_path == "--all" {
        return Err(Error::InvalidInput(
            "close --all has been removed; use `lockbox session close-all`".to_string(),
        )
        .into());
    }
    let was_open = lockbox_is_open(lockbox_path);
    local_vault().close_lockbox(lockbox_path)?;
    clear_default_if_matches(lockbox_path)?;
    if was_open {
        println!("Lockbox closed: {lockbox_path}");
    } else {
        println!("Lockbox was already closed: {lockbox_path}");
    }
    Ok(())
}

fn lockbox_is_open(lockbox_path: &str) -> bool {
    let Ok(lockbox_id) = VaultOpen::read_lockbox_id(Path::new(lockbox_path)) else {
        return false;
    };
    list_open_lockboxes()
        .map(|lockboxes| {
            lockboxes
                .iter()
                .any(|lockbox| lockbox.id == lockbox_id.to_string())
        })
        .unwrap_or(false)
}

pub(crate) fn keygen_matches(matches: &ArgMatches) -> CliResult<()> {
    keygen(&[
        required_value(matches, "private-key"),
        required_value(matches, "public-key"),
    ])
}

pub(crate) fn keygen(args: &[String]) -> CliResult<()> {
    let private_path = require_arg(args, 0, "private key path")?;
    let public_path = require_arg(args, 1, "public key path")?;
    let keypair = ContactKeyPair::generate()?;
    write_private_key(
        private_path,
        &export_private_key(&keypair, KeyFormat::RawHex)?,
    )?;
    fs::write(public_path, encode_hex(&keypair.public_key().to_bytes()))?;
    Ok(())
}

pub(crate) fn open_key_matches(matches: &ArgMatches) -> CliResult<()> {
    let values = positional_values(matches, "args");
    let args = match values.as_slice() {
        [] => vec![default_lockbox_for_command()?],
        [first] if looks_like_lockbox_path(first) => vec![first.clone()],
        [key] => vec![default_lockbox_for_command()?, key.clone()],
        [lockbox, key] if looks_like_lockbox_path(lockbox) => vec![lockbox.clone(), key.clone()],
        [lockbox, _] => {
            return Err(cli_error(format!(
                "lockbox path must end with .lbox: {lockbox}"
            )))
        }
        _ => unreachable!("clap limits open-key positional arguments"),
    };
    open_key(&args)
}

pub(crate) fn open_key(args: &[String]) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    let keypair = load_private_key_from_arg(args.get(1).map(String::as_str))?;
    let lb = local_vault().open_lockbox_with(lockbox_path, LockboxOpen::ContactKeyPair(keypair))?;
    mirror_key_directory(&lb, lockbox_path)?;
    Ok(())
}

pub(crate) fn access_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let (command, sub) = matches
        .subcommand()
        .ok_or_else(|| Error::InvalidInput("missing access command".to_string()))?;
    match command {
        "grant" => grant_access(
            &optional_lockbox_positionals(positional_values(sub, "args"), 1)?,
            access,
        ),
        "list" | "ls" => list_access_with_format(
            &optional_lockbox_positionals(positional_values(sub, "args"), 0)?,
            access,
            output_format_from_matches(sub)?,
        ),
        "refresh" => {
            let positionals = positional_values(sub, "args");
            let scope = if sub.get_flag("all") {
                if positionals.len() > 1 {
                    return Err(cli_error(
                        "access refresh --all accepts at most one identity argument",
                    ));
                }
                RefreshScope::All {
                    identity: positionals.first().cloned(),
                }
            } else {
                let args = optional_lockbox_positionals(positionals, 1)?;
                if args.len() > 2 {
                    return Err(cli_error(
                        "access refresh requires lockbox and identity arguments",
                    ));
                }
                RefreshScope::One {
                    lockbox_path: require_arg(&args, 0, "lockbox")?.to_string(),
                    identity: require_arg(&args, 1, "identity")?.to_string(),
                }
            };
            refresh_access_request(
                RefreshAccessRequest {
                    scope,
                    dry_run: sub.get_flag("dry-run"),
                    yes: sub.get_flag("yes"),
                },
                access,
            )
        }
        "revoke" => revoke_access(
            &optional_lockbox_positionals(positional_values(sub, "args"), 1)?,
            access,
        ),
        _ => Err(Error::InvalidInput(format!("unknown access command: {command}")).into()),
    }
}

pub(crate) fn grant_access(args: &[String], access: &Access) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    let contact_arg = require_arg(args, 1, "identity or contact")?;
    let contact = if let Some(public_key_path) = args.get(2) {
        load_contact_file(contact_arg, public_key_path)?
    } else {
        if Path::new(contact_arg).exists() {
            return Err(cli_error(
                "public key files require a contact name: lockbox access grant <lockbox> <name> <public-key>",
            ));
        }
        load_contact_from_arg(contact_arg)?
    };
    let name = contact.name.ok_or_else(|| {
        cli_error(
            "access entries require a name; use lockbox access grant <lockbox> <name> <public-key>",
        )
    })?;
    let mut lb = open_existing(lockbox_path, access)?;
    let slot_id = lb.add_contact_named(access_entry_name(&name), &contact.public_key)?;
    lb.commit()?;
    mirror_key_directory(&lb, lockbox_path)?;
    default_vault()?.remember_access_slot_label(lb.lockbox_id(), slot_id, name)?;
    Ok(())
}

fn required_value(matches: &ArgMatches, name: &str) -> String {
    matches
        .get_one::<String>(name)
        .unwrap_or_else(|| panic!("clap did not provide required argument {name}"))
        .clone()
}

fn list_access_with_format(
    args: &[String],
    access: &Access,
    format: OutputFormat,
) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    let lb = open_existing(lockbox_path, access)?;
    let owner = lb.owner_inspection()?;
    let owner_fingerprint = owner.fingerprint.unwrap_or_else(|| "-".to_string());
    let owner_signed = if owner.signed { "yes" } else { "no" }.to_string();
    let metadata = fs::metadata(lockbox_path).ok();
    let labels = access_slot_labels_by_slot(lb.lockbox_id());
    let created = metadata
        .as_ref()
        .and_then(|metadata| metadata.created().ok())
        .map(format_system_time_utc)
        .unwrap_or_else(|| "-".to_string());
    let updated = metadata
        .as_ref()
        .and_then(|metadata| metadata.modified().ok())
        .map(format_system_time_utc)
        .unwrap_or_else(|| "-".to_string());
    let mut rows = Vec::new();
    for slot in lb.list_key_slots() {
        let name = labels
            .get(&slot.id)
            .cloned()
            .unwrap_or_else(|| "-".to_string());
        rows.push(vec![
            slot.id.to_string(),
            name,
            format!("{:?}", slot.protection),
            slot.algorithm.to_string(),
            owner_fingerprint.clone(),
            owner_signed.clone(),
            created.clone(),
            updated.clone(),
        ]);
    }
    print_records(
        &[
            "slot",
            "name",
            "protection",
            "algorithm",
            "owner",
            "owner_signed",
            "created",
            "updated",
        ],
        rows,
        format,
    )?;
    Ok(())
}

pub(crate) fn revoke_access(args: &[String], access: &Access) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    let targets = args.get(1..).unwrap_or_default();
    if targets.is_empty() {
        return Err(cli_error("missing slot id or local access name"));
    }
    let mut lb = open_existing(lockbox_path, access)?;
    let mut revoked_slot_ids = BTreeSet::new();
    for target in targets {
        revoked_slot_ids.insert(resolve_access_revoke_target(&lb, target)?);
    }
    let retained_contacts = retained_contacts_after_revoke(&lb, &revoked_slot_ids)?;
    let old_labels = access_slot_labels_by_slot(lb.lockbox_id());
    let new_labels = lb.replace_content_key_with_contacts(&retained_contacts)?;
    mirror_key_directory(&lb, lockbox_path)?;
    let _ = local_vault().close_lockbox(lockbox_path);
    let vault = default_vault()?;
    for slot_id in old_labels.keys() {
        vault.forget_access_slot_label(lb.lockbox_id(), *slot_id)?;
    }
    for (name, slot_id) in new_labels {
        vault.remember_access_slot_label(lb.lockbox_id(), slot_id, name)?;
    }
    Ok(())
}

struct RefreshAccessRequest {
    scope: RefreshScope,
    dry_run: bool,
    yes: bool,
}

enum RefreshScope {
    All {
        identity: Option<String>,
    },
    One {
        lockbox_path: String,
        identity: String,
    },
}

fn refresh_access_request(request: RefreshAccessRequest, access: &Access) -> CliResult<()> {
    match request.scope {
        RefreshScope::All { identity } => {
            let vault = default_vault()?;
            let identities = match identity.as_deref() {
                Some(identity) => vec![identity.to_string()],
                None => vault.list_private_keys()?,
            };
            if identities.is_empty() {
                return Err(cli_error("no vault identities found to refresh"));
            }
            let known = vault.list_known_lockboxes()?;
            let mut missing = Vec::new();
            let mut inaccessible = Vec::new();
            let mut targets = Vec::new();
            for lockbox in &known {
                match fs::metadata(&lockbox.path) {
                    Ok(metadata) if metadata.is_dir() => {
                        inaccessible.push((
                            lockbox.path.clone(),
                            "lockbox path is a directory".to_string(),
                        ));
                        continue;
                    }
                    Ok(_) => {}
                    Err(err) if err.kind() == io::ErrorKind::NotFound => {
                        missing.push(lockbox.path.clone());
                        continue;
                    }
                    Err(err) => {
                        inaccessible.push((lockbox.path.clone(), err.to_string()));
                        continue;
                    }
                }
                match refresh_targets_for_lockbox(&lockbox.path, &identities, access) {
                    Ok(found) => targets.extend(found),
                    Err(err) => inaccessible.push((lockbox.path.clone(), err.to_string())),
                }
            }
            print_refresh_plan(
                identity.as_deref(),
                Some(known.len()),
                &targets,
                &missing,
                &inaccessible,
                request.dry_run,
                request.yes,
            );
            apply_refresh_plan(&targets, access, request.dry_run, request.yes)
        }
        RefreshScope::One {
            lockbox_path,
            identity,
        } => {
            let identities = vec![identity.clone()];
            let targets = refresh_targets_for_lockbox(&lockbox_path, &identities, access)?;
            print_refresh_plan(
                Some(&identity),
                None,
                &targets,
                &[],
                &[],
                request.dry_run,
                request.yes,
            );
            apply_refresh_plan(&targets, access, request.dry_run, request.yes)
        }
    }
}

#[derive(Debug, Clone)]
struct RefreshTarget {
    lockbox_path: String,
    identity: String,
    slot_count: usize,
}

fn refresh_targets_for_lockbox(
    lockbox_path: &str,
    identities: &[String],
    access: &Access,
) -> CliResult<Vec<RefreshTarget>> {
    let lb = open_existing(lockbox_path, access)?;
    let mut targets = Vec::new();
    for identity in identities {
        let slot_count = matching_contact_slot_ids(&lb, identity).len();
        if slot_count > 0 {
            targets.push(RefreshTarget {
                lockbox_path: lockbox_path.to_string(),
                identity: identity.clone(),
                slot_count,
            });
        }
    }
    Ok(targets)
}

fn matching_contact_slot_ids(lockbox: &Lockbox, identity: &str) -> Vec<u64> {
    let labels = access_slot_labels_by_slot(lockbox.lockbox_id());
    lockbox
        .list_key_slots()
        .into_iter()
        .filter(|slot| {
            slot.protection == LockboxKeySlotProtection::Contact
                && labels.get(&slot.id).is_some_and(|name| name == identity)
        })
        .map(|slot| slot.id)
        .collect()
}

fn access_slot_labels_by_slot(lockbox_id: lockbox_core::LockboxId) -> BTreeMap<u64, String> {
    default_vault()
        .and_then(|vault| Ok(vault.list_access_slot_labels(lockbox_id)?))
        .map(|labels| {
            labels
                .into_iter()
                .map(|label| (label.slot_id, label.name))
                .collect()
        })
        .unwrap_or_default()
}

fn resolve_access_revoke_target(lockbox: &Lockbox, target: &str) -> CliResult<u64> {
    if let Ok(slot_id) = target.parse::<u64>() {
        return Ok(slot_id);
    }
    let vault = default_vault()?;
    let mut labels = vault.find_access_slot_labels(lockbox.lockbox_id(), target)?;
    if labels.is_empty() && !target.contains(':') {
        labels.extend(
            vault.find_access_slot_labels(lockbox.lockbox_id(), &format!("identity:{target}"))?,
        );
        labels.extend(
            vault.find_access_slot_labels(lockbox.lockbox_id(), &format!("contact:{target}"))?,
        );
    }
    match labels.as_slice() {
        [label] => Ok(label.slot_id),
        [] => Err(cli_error(format!(
            "no local access label named {target}; use `lockbox access list` and revoke by slot id"
        ))),
        _ => Err(cli_error(format!(
            "multiple local access labels named {target}; revoke by slot id"
        ))),
    }
}

fn retained_contacts_after_revoke(
    lockbox: &Lockbox,
    revoked_slot_ids: &BTreeSet<u64>,
) -> CliResult<Vec<(String, ContactPublicKey)>> {
    let slots = lockbox.list_key_slots();
    if revoked_slot_ids.len() >= slots.len() {
        return Err(cli_error(
            "cannot revoke the last access entry; grant another identity or contact first",
        ));
    }
    let labels = access_slot_labels_by_slot(lockbox.lockbox_id());
    let mut retained = Vec::new();
    for slot in slots {
        if revoked_slot_ids.contains(&slot.id) {
            continue;
        }
        if slot.protection != LockboxKeySlotProtection::Contact {
            return Err(cli_error(format!(
                "cannot true-revoke while retaining non-contact access slot {}; rekey requires a vault-resolvable contact or identity",
                slot.id
            )));
        }
        let Some(name) = labels.get(&slot.id) else {
            return Err(cli_error(format!(
                "cannot true-revoke because retained access slot {} has no local vault label; use `lockbox access list` and restore the contact label before revoking",
                slot.id
            )));
        };
        let contact = load_contact_from_arg(name)?;
        retained.push((name.clone(), contact.public_key));
    }
    if retained.is_empty() {
        return Err(cli_error(
            "cannot true-revoke without at least one retained contact or identity",
        ));
    }
    Ok(retained)
}

fn access_entry_name(label: &str) -> String {
    label
        .strip_prefix("identity:")
        .or_else(|| label.strip_prefix("contact:"))
        .unwrap_or(label)
        .to_string()
}

fn print_refresh_plan(
    identity: Option<&str>,
    known_count: Option<usize>,
    targets: &[RefreshTarget],
    missing: &[String],
    inaccessible: &[(String, String)],
    dry_run: bool,
    yes: bool,
) {
    println!("Refresh plan:");
    match identity {
        Some(identity) => println!("  identity: {identity}"),
        None => println!("  identity: all"),
    }
    if let Some(known_count) = known_count {
        println!("  known lockboxes: {known_count}");
    }
    println!("  matching lockbox/identity pairs: {}", targets.len());
    println!(
        "  matching access entries: {}",
        targets
            .iter()
            .map(|target| target.slot_count)
            .sum::<usize>()
    );
    println!("  dry run: {}", if dry_run { "yes" } else { "no" });
    println!("  apply without prompt: {}", if yes { "yes" } else { "no" });
    println!("  missing: {}", missing.len());
    println!("  inaccessible: {}", inaccessible.len());
    if !targets.is_empty() {
        println!();
        println!("Refresh targets:");
        for target in targets {
            println!(
                "  {} {} ({} access entries)",
                target.lockbox_path, target.identity, target.slot_count
            );
        }
    }
    if !missing.is_empty() {
        println!();
        println!("Missing known lockboxes:");
        for path in missing {
            println!("  {path}");
        }
    }
    if !inaccessible.is_empty() {
        println!();
        println!("Inaccessible known lockboxes:");
        for (path, reason) in inaccessible {
            println!("  {path}: {reason}");
        }
    }
}

fn apply_refresh_plan(
    targets: &[RefreshTarget],
    access: &Access,
    dry_run: bool,
    yes: bool,
) -> CliResult<()> {
    if dry_run {
        println!();
        println!("No access entries were changed.");
        return Ok(());
    }
    if targets.is_empty() {
        println!();
        println!("No matching access entries found.");
        return Ok(());
    }
    if !yes && !confirm_access_refresh(targets.len())? {
        println!();
        println!("No access entries were changed.");
        return Ok(());
    }
    let public_keys = load_refresh_public_keys(targets)?;
    let mut updated = 0usize;
    for target in targets {
        let public_key = public_keys.get(&target.identity).ok_or_else(|| {
            cli_error(format!(
                "vault identity {} was not loaded for refresh",
                target.identity
            ))
        })?;
        if refresh_lockbox_identity(&target.lockbox_path, &target.identity, public_key, access)? {
            updated += 1;
        }
    }
    println!();
    println!("Refreshed access for {updated} lockbox/identity pairs.");
    Ok(())
}

fn load_refresh_public_keys(
    targets: &[RefreshTarget],
) -> CliResult<BTreeMap<String, ContactPublicKey>> {
    let vault = default_vault()?;
    let mut public_keys = BTreeMap::new();
    for target in targets {
        if public_keys.contains_key(&target.identity) {
            continue;
        }
        public_keys.insert(
            target.identity.clone(),
            vault.load_private_key(&target.identity)?.public_key(),
        );
    }
    Ok(public_keys)
}

fn refresh_lockbox_identity(
    lockbox_path: &str,
    identity: &str,
    public_key: &ContactPublicKey,
    access: &Access,
) -> CliResult<bool> {
    let mut lb = open_existing(lockbox_path, access)?;
    let old_slot_ids = matching_contact_slot_ids(&lb, identity);
    if old_slot_ids.is_empty() {
        return Ok(false);
    }
    let new_slot_id = lb.add_contact_named(identity.to_string(), public_key)?;
    default_vault()?.remember_access_slot_label(lb.lockbox_id(), new_slot_id, identity)?;
    for slot_id in old_slot_ids {
        if slot_id != new_slot_id {
            lb.delete_key(slot_id)?;
            let _ = default_vault().and_then(|vault| {
                vault.forget_access_slot_label(lb.lockbox_id(), slot_id)?;
                Ok(())
            });
        }
    }
    lb.commit()?;
    mirror_key_directory(&lb, lockbox_path)?;
    Ok(true)
}

fn confirm_access_refresh(target_count: usize) -> CliResult<bool> {
    eprintln!("Refresh access for {target_count} lockbox/identity pairs?");
    eprint!("Type 'yes' to apply: ");
    io::stderr().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(answer.trim() == "yes")
}

fn write_private_key(path: &str, bytes: &SecretVec) -> CliResult<()> {
    let mut file = create_private_key_file(path)?;
    bytes.with_bytes(|bytes| file.write_all(bytes))??;
    Ok(())
}

fn create_path(path: &str) -> CliResult<PathBuf> {
    let mut path = PathBuf::from(path);
    if path.extension().is_none() {
        path.set_extension("lbox");
    }
    Ok(path)
}

fn ensure_new_lockbox_path(path: &Path) -> CliResult<()> {
    if path.exists() {
        return Err(Error::AlreadyExists(path.display().to_string()).into());
    }
    Ok(())
}

struct OpenOptions {
    lockbox_path: String,
    ttl_seconds: Option<u64>,
    password_source: PasswordSource,
}

enum PasswordSource {
    Prompt,
    Variables(String),
    File(String),
    Stdin,
}

impl OpenOptions {
    fn from_matches(matches: &ArgMatches) -> CliResult<Self> {
        let mut ttl_seconds = matches
            .get_one::<String>("duration")
            .map(|value| parse_duration(value))
            .transpose()?;
        let mut password_source = PasswordSource::Prompt;
        if let Some(value) = matches.get_one::<String>("password-env") {
            ensure_prompt_password_source(&password_source)?;
            password_source = PasswordSource::Variables(value.clone());
        }
        if let Some(value) = matches.get_one::<String>("password-file") {
            ensure_prompt_password_source(&password_source)?;
            password_source = PasswordSource::File(value.clone());
        }
        if matches.get_flag("password-stdin") {
            ensure_prompt_password_source(&password_source)?;
            password_source = PasswordSource::Stdin;
        }
        let lockbox_path = create_path(&optional_lockbox_value(matches, "lockbox")?)?
            .to_string_lossy()
            .into_owned();
        if ttl_seconds.is_none() {
            ttl_seconds = default_session_duration()?;
        }
        Ok(Self {
            lockbox_path,
            ttl_seconds,
            password_source,
        })
    }

    fn read_password(&self) -> CliResult<lockbox_vault::SecretString> {
        match &self.password_source {
            PasswordSource::Prompt => read_password("Password: "),
            PasswordSource::Variables(name) => {
                let value = env::var(name)
                    .map_err(|_| Error::InvalidInput(format!("variable is not set: {name}")))?;
                lockbox_vault::SecretString::try_from_bytes(value.into_bytes()).map_err(Into::into)
            }
            PasswordSource::File(path) => secret_from_bytes(
                fs::read(path)
                    .map_err(|err| Error::Io(format!("read password file {path}: {err}")))?,
            ),
            PasswordSource::Stdin => {
                let mut bytes = Vec::new();
                io::stdin().read_to_end(&mut bytes)?;
                secret_from_bytes(bytes)
            }
        }
    }
}

fn ensure_prompt_password_source(source: &PasswordSource) -> CliResult<()> {
    if matches!(source, PasswordSource::Prompt) {
        Ok(())
    } else {
        Err(
            Error::InvalidInput("choose only one password source for lockbox open".to_string())
                .into(),
        )
    }
}

fn secret_from_bytes(mut bytes: Vec<u8>) -> CliResult<lockbox_vault::SecretString> {
    while matches!(bytes.last(), Some(b'\n' | b'\r')) {
        bytes.pop();
    }
    lockbox_vault::SecretString::try_from_bytes(bytes).map_err(Into::into)
}

fn default_session_duration() -> CliResult<Option<u64>> {
    if let Ok(value) = env::var("LOCKBOX_OPEN_DURATION") {
        return Ok(Some(parse_duration(&value)?));
    }
    let Some(path) = session_config_path() else {
        return Ok(None);
    };
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(None);
    };
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if matches!(key.trim(), "open_duration" | "session_duration") {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            return Ok(Some(parse_duration(value)?));
        }
    }
    Ok(None)
}

fn session_config_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("LOCKBOX_CONFIG") {
        return Some(PathBuf::from(path));
    }
    #[cfg(target_os = "macos")]
    {
        return env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library/Application Support/reVault/config.yaml"));
    }
    #[cfg(target_os = "windows")]
    {
        return env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|path| path.join("reVault").join("config.yaml"));
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(path).join("lockbox").join("config.yaml"));
        }
        return env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(".config/lockbox/config.yaml"));
    }
    #[allow(unreachable_code)]
    None
}

fn parse_duration(value: &str) -> CliResult<u64> {
    let value = value.trim();
    if value.is_empty() {
        return Err(Error::InvalidInput("duration cannot be empty".to_string()).into());
    }
    let split_at = value
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(value.len());
    let (number, unit) = value.split_at(split_at);
    let amount = number
        .parse::<u64>()
        .map_err(|_| Error::InvalidInput(format!("invalid duration: {value}")))?;
    if amount == 0 {
        return Err(Error::InvalidInput("duration must be greater than zero".to_string()).into());
    }
    let multiplier = match unit {
        "" | "s" | "sec" | "secs" => 1,
        "m" | "min" | "mins" => 60,
        "h" | "hr" | "hrs" => 60 * 60,
        "d" | "day" | "days" => 24 * 60 * 60,
        _ => return Err(Error::InvalidInput(format!("invalid duration unit: {unit}")).into()),
    };
    amount
        .checked_mul(multiplier)
        .ok_or_else(|| Error::InvalidInput(format!("duration is too large: {value}")).into())
}

fn format_system_time_utc(time: SystemTime) -> String {
    let Ok(duration) = time.duration_since(UNIX_EPOCH) else {
        return "-".to_string();
    };
    let seconds = duration.as_secs() as i64;
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    format!("{year:04}/{month:02}/{day:02} {hour:02}:{minute:02} UTC")
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096).div_euclid(365);
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2).div_euclid(153);
    let day = doy - (153 * mp + 2).div_euclid(5) + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

#[cfg(unix)]
fn create_private_key_file(path: &str) -> CliResult<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;

    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(path)?;
    set_private_key_permissions(path)?;
    Ok(file)
}

#[cfg(not(unix))]
fn create_private_key_file(path: &str) -> CliResult<fs::File> {
    fs::File::create(path).map_err(Into::into)
}

#[cfg(unix)]
fn set_private_key_permissions(path: &str) -> CliResult<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_private_key_permissions(_path: &str) -> CliResult<()> {
    Ok(())
}
