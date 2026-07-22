use super::context::{
    cli_error, default_vault, open_default_vault_with_password, read_new_vault_password,
    read_replacement_vault_password, read_vault_password,
    remember_default_vault_password_with_warning, require_arg, CliResult,
};
use super::form::{default_form_alias, parse_field_spec, print_form_definition_saved};
use super::output::{output_format_from_matches, print_records, OutputFormat};
use super::session::{default_matches, replace_default_after_move};
use clap::ArgMatches;
use revault_lockbox_api::vault_integration::VaultOpen;
use revault_lockbox_api::{
    lock_path_for, ContactKeyPair, ContactPublicKey, Error, FileLockScope, Lockbox,
    OwnerSigningKeyPair, OwnerSigningPublicKey, ScopedFileLock, SecretVec,
};
use revault_publish_protocol::protocol::Status;
use revault_publish_protocol::{
    contact_fingerprint, normalize_contact_email, ClientError, ContactPublish, PublishClientPool,
    StickyPublishServer,
};
use revault_vault_api::{
    backup_default_vault, decode_fingerprint_crockford_96, decode_fingerprint_hex,
    default_vault_dir, default_vault_path, encode_hex, export_private_key, export_public_key,
    forget_platform_vault_password, format_fingerprint_crockford_96 as format_fingerprint_code,
    format_fingerprint_crockford_96_reading as format_fingerprint_reading,
    format_fingerprint_hex_pairs as format_hex_pairs, import_private_key, import_public_key,
    local_vault, public_key_fingerprint, restore_default_vault, set_auto_open_scope, AutoOpenScope,
    KeyFormat, ProfileGenerationStatus, VaultDirectory,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const PUBLISH_RECEIVE_VERIFICATION_ADVICE: &str = concat!(
    "public key received; verify it by asking the publisher for the fingerprint code. ",
    "You must initiate the communication over a channel you already trust. ",
    "If the publisher sends you the fingerprint before you ask, do not accept it."
);
const PUBLISH_FINGERPRINT_SECURITY_NOTE: &str = concat!(
    "use the 96-bit Crockford fingerprint code; short PINs are only ",
    "accidental-error checks and are too small to authenticate a public key ",
    "against substitution"
);
const FINGERPRINT_CHANNEL_PROMPT: &str = concat!(
    "How did you receive the fingerprint?\n",
    "  1) email\n",
    "  2) phone call from the key owner\n",
    "  3) phone call to the key owner\n",
    "  4) text/SMS message from the key owner\n",
    "  5) text/SMS message to the key owner\n",
    "  6) in person"
);

pub(crate) fn run_matches(matches: &ArgMatches) -> CliResult<()> {
    let (command, sub) = matches
        .subcommand()
        .ok_or_else(|| Error::InvalidInput("missing vault command".to_string()))?;
    match command {
        "init" => init_options(sub.get_flag("overwrite"), sub.get_flag("verify")),
        "backup" => backup_options(required_value(sub, "output"), sub.get_flag("overwrite")),
        "restore" => restore_options(required_value(sub, "backup"), sub.get_flag("overwrite")),
        "passphrase" => change_passphrase(&[]),
        "form" => vault_form_matches(sub),
        "profile" => vault_profile_matches(sub),
        "contact" => vault_contact_matches(sub),
        "lockbox" => vault_lockbox_matches(sub),
        _ => Err(Error::InvalidInput(format!("unknown vault command: {command}")).into()),
    }
}

fn required_value(matches: &ArgMatches, name: &str) -> String {
    matches
        .get_one::<String>(name)
        .unwrap_or_else(|| panic!("clap did not provide required argument {name}"))
        .clone()
}

fn optional_value<'a>(matches: &'a ArgMatches, name: &str) -> Option<&'a str> {
    matches.get_one::<String>(name).map(String::as_str)
}

fn string_values(matches: &ArgMatches, name: &str) -> Vec<String> {
    matches
        .get_many::<String>(name)
        .map(|values| values.cloned().collect())
        .unwrap_or_default()
}

fn vault_form_matches(matches: &ArgMatches) -> CliResult<()> {
    let (command, sub) = matches
        .subcommand()
        .ok_or_else(|| Error::InvalidInput("missing vault form command".to_string()))?;
    match command {
        "define" => form_define_matches(sub),
        "definitions" => form_definitions_with_format(output_format_from_matches(sub)?),
        _ => Err(Error::InvalidInput(format!("unknown vault form command: {command}")).into()),
    }
}

fn vault_profile_matches(matches: &ArgMatches) -> CliResult<()> {
    let (command, sub) = matches
        .subcommand()
        .ok_or_else(|| Error::InvalidInput("missing vault profile command".to_string()))?;
    match command {
        "list" | "ls" => list_profiles_with_format(output_format_from_matches(sub)?),
        "create" => keygen_options(optional_value(sub, "name"), sub.get_flag("overwrite")),
        "email" => profile_email_values(&string_values(sub, "args")),
        "fingerprint" => profile_fingerprint(&optional_string_arg(sub, "name")),
        "history" => profile_history_with_format(
            optional_value(sub, "name").unwrap_or(VaultDirectory::DEFAULT_KEY_NAME),
            output_format_from_matches(sub)?,
        ),
        "publish" => publish_profile_options(PublishCliOptions::from_publish_matches(sub)?),
        "backup" => profile_backup_options(ProfileBackupArgs::from_matches(sub)),
        "restore" => profile_restore_options(ProfileRestoreArgs::from_matches(sub)),
        "export" => export_public_options(
            ProfileExportArgs::from_matches(sub),
            KeyFormat::parse(optional_value(sub, "format").unwrap_or("lockbox"))?,
        ),
        "remove" => remove_key_options(optional_value(sub, "name"), sub.get_flag("force")),
        "rotate" => rotate_key(&optional_string_arg(sub, "name")),
        _ => Err(Error::InvalidInput(format!("unknown vault profile command: {command}")).into()),
    }
}

fn vault_contact_matches(matches: &ArgMatches) -> CliResult<()> {
    let (command, sub) = matches.subcommand().ok_or_else(|| {
        Error::InvalidInput(
            "missing vault contact command; use `lockbox vault contact list`, `lockbox vault contact import <name> <public-key>`, `lockbox vault contact receive <publish-code>`, or `lockbox vault contact remove <name>`"
                .to_string(),
        )
    })?;
    match command {
        "list" | "ls" => list_contacts_with_format(output_format_from_matches(sub)?),
        "import" => contact_import_options(ContactImportOptions::from_matches(sub)),
        "receive" => receive_publish_options(PublishCliOptions::from_receive_matches(sub)?),
        "remove" => remove_contact_name(&required_value(sub, "name")),
        _ => Err(Error::InvalidInput(format!("unknown vault contact command: {command}")).into()),
    }
}

fn vault_lockbox_matches(matches: &ArgMatches) -> CliResult<()> {
    let (command, sub) = matches.subcommand().ok_or_else(|| {
        Error::InvalidInput(
            "missing vault lockbox command; use `lockbox vault lockbox list`, `lockbox vault lockbox move <source> <destination>`, or `lockbox vault lockbox forget <lockbox>`"
                .to_string(),
        )
    })?;
    match command {
        "list" | "ls" => list_known_lockboxes_with_format(output_format_from_matches(sub)?),
        "move" => move_known_lockbox(
            &required_value(sub, "source"),
            &required_value(sub, "destination"),
        ),
        "forget" => forget_known_lockbox_path(&required_value(sub, "lockbox")),
        _ => Err(Error::InvalidInput(format!("unknown vault lockbox command: {command}")).into()),
    }
}

fn optional_string_arg(matches: &ArgMatches, name: &str) -> Vec<String> {
    optional_value(matches, name)
        .map(|value| vec![value.to_string()])
        .unwrap_or_default()
}

fn fingerprint_input_matches(input: &str, computed_fingerprint: &[u8]) -> CliResult<bool> {
    match decode_fingerprint_crockford_96(input) {
        Ok(code) => Ok(computed_fingerprint.starts_with(&code)),
        Err(code_err) => match decode_fingerprint_hex(input) {
            Ok(fingerprint) => Ok(fingerprint == computed_fingerprint),
            Err(_) => Err(Error::InvalidInput(format!(
                "{code_err}; expected a 20-character 96-bit Crockford fingerprint code or the full hex fingerprint"
            ))
            .into()),
        },
    }
}

fn print_fingerprint_lines(prefix: &str, fingerprint: &[u8]) {
    let code = format_fingerprint_code(fingerprint);
    println!("{prefix}={code}");
    println!("{prefix}_reading={}", format_fingerprint_reading(&code));
    println!("{prefix}_hex={}", format_hex_pairs(fingerprint));
}

fn change_passphrase(args: &[String]) -> CliResult<()> {
    if !args.is_empty() {
        return Err(
            Error::InvalidInput("vault passphrase does not accept arguments".to_string()).into(),
        );
    }
    let path = default_vault_path()?;
    if !path.exists() {
        return Err(Error::VaultUnavailable(
            "local vault is not initialized; run `lockbox vault init` first".to_string(),
        )
        .into());
    }

    let old_password = read_vault_password("Current vault pass phrase: ")?;
    open_default_vault_with_password(&old_password)?;

    let backup_path = passphrase_change_backup_path()?;
    backup_default_vault(&backup_path, false)?;
    let new_password = read_replacement_vault_password()?;
    VaultDirectory::change_default_password(&old_password, &new_password)?;
    remember_default_vault_password_with_warning(
        &new_password,
        "the vault passphrase changed successfully",
    );

    println!("Vault pass phrase changed successfully.");
    println!("Backup:");
    println!("  {}", backup_path.display());
    Ok(())
}

fn form_define_matches(matches: &ArgMatches) -> CliResult<()> {
    let alias = optional_value(matches, "alias").map(str::to_string);
    let name = optional_value(matches, "name")
        .map(str::to_string)
        .or_else(|| alias.clone())
        .ok_or_else(|| {
            Error::InvalidInput("vault form define requires an alias or --name".to_string())
        })?;
    let alias = alias.unwrap_or_else(|| default_form_alias(&name));
    let description = optional_value(matches, "description")
        .unwrap_or_default()
        .to_string();
    let type_id = optional_value(matches, "definition-id")
        .map(revault_lockbox_api::FormTypeId::new)
        .transpose()?;
    let fields = string_values(matches, "field")
        .iter()
        .map(|field| parse_field_spec(field))
        .collect::<CliResult<Vec<_>>>()?;
    form_define_options(alias, name, description, type_id, fields)
}

fn form_define_options(
    alias: String,
    name: String,
    description: String,
    type_id: Option<revault_lockbox_api::FormTypeId>,
    fields: Vec<revault_lockbox_api::FormFieldDefinition>,
) -> CliResult<()> {
    let vault = default_vault()?;
    let definition = if let Some(type_id) = type_id {
        vault.define_form_with_type_id_and_description(
            type_id,
            &alias,
            &name,
            &description,
            fields,
        )?
    } else {
        vault.define_form_with_description(&alias, &name, &description, fields)?
    };
    print_form_definition_saved(&definition);
    Ok(())
}

fn form_definitions_with_format(format: OutputFormat) -> CliResult<()> {
    let rows = default_vault()?
        .list_form_definitions()?
        .into_iter()
        .map(|definition| {
            vec![
                definition.alias,
                definition.type_id.to_string(),
                definition.revision.to_string(),
                definition.name,
                definition.description,
                definition.fields.len().to_string(),
            ]
        })
        .collect::<Vec<_>>();
    print_records(
        &[
            "alias",
            "definition_id",
            "revision",
            "name",
            "description",
            "fields",
        ],
        rows,
        format,
    )?;
    Ok(())
}

fn init_options(overwrite: bool, verify: bool) -> CliResult<()> {
    if overwrite && verify {
        return Err(Error::InvalidInput(
            "--overwrite and --verify cannot be used together".to_string(),
        )
        .into());
    }

    let path = default_vault_path()?;
    let existed = path.exists();
    if existed {
        let _ = forget_platform_vault_password();
        println!("Local vault already exists.");
        println!("Path: {}", path.display());
        if overwrite {
            println!(
                "WARNING: replacing it will remove profiles, contacts and \
key-directory backups stored in this vault."
            );
            let password = read_new_vault_password()?;
            let vault = VaultDirectory::replace_default(&password)?;
            let generated = ensure_default_private_key(&vault)?;
            let default_forms = vault.seed_default_form_definitions()?;
            set_auto_open_scope(AutoOpenScope::Lockboxes)?;
            remember_default_vault_password_with_warning(
                &password,
                "the vault was replaced successfully",
            );
            println!("Vault replaced successfully.");
            if generated {
                println!(
                    "Created default profile: {}",
                    VaultDirectory::DEFAULT_KEY_NAME
                );
            }
            if default_forms > 0 {
                println!("Default forms: {default_forms}");
            }
            if generated {
                print_default_profile_backup(&vault)?;
            }
            return Ok(());
        }
        if verify {
            let password = read_vault_password("Vault pass phrase: ")?;
            open_default_vault_with_password(&password)?;
            remember_default_vault_password_with_warning(
                &password,
                "the vault opened successfully",
            );
            println!("Vault opened successfully.");
            return Ok(());
        }
        println!("No changes made. Use `lockbox vault init --verify` to validate it.");
        println!("Use `lockbox vault init --overwrite` only when replacing the vault.");
        return Ok(());
    } else {
        println!("Create the local reVault vault.");
        println!();
        println!("Stores:");
        println!("  - profiles and contacts");
        println!("  - key-directory backups for published lockboxes");
        println!();
    }
    let password = read_new_vault_password()?;
    let vault = VaultDirectory::open_or_create_default(&password)?;
    let generated = ensure_default_private_key(&vault)?;
    let default_forms = vault.seed_default_form_definitions()?;
    set_auto_open_scope(AutoOpenScope::Lockboxes)?;
    remember_default_vault_password_with_warning(&password, "the vault was created successfully");
    println!("Vault created successfully.");
    println!();
    println!("Directory:");
    println!("  {}", vault.root().display());
    if generated {
        println!();
        println!("Profile: {}", VaultDirectory::DEFAULT_KEY_NAME);
    }
    if default_forms > 0 {
        println!("Default forms: {default_forms}");
    }
    if generated {
        print_default_profile_backup(&vault)?;
    }
    println!();
    println!("Pass phrase reminder:");
    println!("  Store the vault pass phrase somewhere safe.");
    println!("  If it is lost, reVault cannot recover this vault.");
    Ok(())
}

fn backup_options(output: String, overwrite: bool) -> CliResult<()> {
    let _manifest = backup_default_vault(&output, overwrite)?;
    println!("Backup completed successfully.");
    println!(
        "Vault path: {}",
        absolute_path(&default_vault_path()?)?.display()
    );
    println!(
        "Backup path: {}",
        absolute_path(&PathBuf::from(output))?.display()
    );
    Ok(())
}

fn absolute_path(path: &std::path::Path) -> CliResult<PathBuf> {
    if let Ok(path) = fs::canonicalize(path) {
        return Ok(path);
    }
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn passphrase_change_backup_path() -> CliResult<PathBuf> {
    Ok(default_vault_dir()?.join(format!(
        "local-vault-before-passphrase-change-{}.lockbox-backup",
        unix_ms_now()
    )))
}

fn restore_backup_path() -> CliResult<PathBuf> {
    Ok(default_vault_dir()?.join(format!(
        "local-vault-before-restore-{}.lockbox-backup",
        unix_ms_now()
    )))
}

fn profile_restore_backup_path(name: &str) -> CliResult<PathBuf> {
    let safe_name = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    Ok(default_vault_dir()?.join(format!(
        "local-vault-before-profile-restore-{safe_name}-{}.lockbox-backup",
        unix_ms_now()
    )))
}

fn restore_options(input: String, overwrite: bool) -> CliResult<()> {
    let existing_vault = default_vault_path()?;
    let backup_path = if overwrite && existing_vault.exists() {
        let backup_path = restore_backup_path()?;
        backup_default_vault(&backup_path, false)?;
        Some(backup_path)
    } else {
        None
    };
    let manifest = restore_default_vault(&input, overwrite)?;
    let _ = forget_platform_vault_password();
    if let Some(backup_path) = backup_path {
        println!("WARNING: replacing the local vault.");
        println!("Existing vault backed up before restore.");
        println!("Backup:");
        println!("  {}", backup_path.display());
    }
    println!("restored={input}");
    println!("vault_file={}", manifest.vault_file_name);
    println!("vault_size={}", manifest.vault_size);
    println!("vault_sha256={}", manifest.vault_sha256);
    println!("Vault restored successfully.");
    Ok(())
}

fn keygen_options(name: Option<&str>, overwrite: bool) -> CliResult<()> {
    let defaulted_name = name.is_none();
    let name = name.unwrap_or(VaultDirectory::DEFAULT_KEY_NAME);
    let vault = default_vault()?;
    if vault.private_key_exists(name)? && !overwrite {
        return Err(Error::AlreadyExists(format!("vault profile {name}")).into());
    }

    let keypair = ContactKeyPair::generate()?;
    vault.store_private_key(name, &keypair)?;
    if defaulted_name {
        println!("Using default profile name: {name}");
    }
    println!("Created vault profile: {name}");
    println!(
        "Export its public key with: lockbox vault profile export <public-key-output> --name {name}"
    );
    Ok(())
}

fn contact_import_options(options: ContactImportOptions) -> CliResult<()> {
    let name = require_arg(&options.positionals, 0, "contact name")?;
    let public_path = require_arg(&options.positionals, 1, "public key path")?;
    let vault = default_vault()?;
    if vault.contact_exists(name)? && !options.overwrite {
        return Err(Error::AlreadyExists(format!("contact {name}")).into());
    }
    let contact = import_public_key(&fs::read(public_path)?)?;
    let expected_fingerprint = options
        .fingerprint
        .clone()
        .map(Ok)
        .unwrap_or_else(|| prompt_line("Public key fingerprint code from key owner: "))?;
    let fingerprint_channel = verify_fingerprint_channel(options.fingerprint_channel.as_deref())?;
    let computed_fingerprint = public_key_fingerprint(&contact);
    if !fingerprint_input_matches(&expected_fingerprint, &computed_fingerprint)? {
        return Err(Error::InvalidInput(format!(
            "public key fingerprint mismatch for {name}; expected {}, computed {}",
            expected_fingerprint.trim(),
            format_fingerprint_code(&computed_fingerprint)
        ))
        .into());
    }
    vault.store_contact(name, &contact)?;
    println!("contact={name}");
    print_fingerprint_lines("public_key_fingerprint", &computed_fingerprint);
    println!("fingerprint_verified=yes");
    println!("fingerprint_channel={fingerprint_channel}");
    Ok(())
}

#[derive(Default)]
struct ContactImportOptions {
    overwrite: bool,
    fingerprint: Option<String>,
    fingerprint_channel: Option<String>,
    positionals: Vec<String>,
}

impl ContactImportOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            overwrite: matches.get_flag("overwrite"),
            fingerprint: optional_value(matches, "fingerprint").map(str::to_string),
            fingerprint_channel: optional_value(matches, "fingerprint-channel").map(str::to_string),
            positionals: vec![
                required_value(matches, "name"),
                required_value(matches, "public-key"),
            ],
        }
    }
}

fn publish_profile_options(options: PublishCliOptions) -> CliResult<()> {
    let profile = options
        .positionals
        .first()
        .map(String::as_str)
        .unwrap_or(VaultDirectory::DEFAULT_KEY_NAME);
    let vault = default_vault()?;
    let keypair = vault.load_private_key(profile)?;
    let public_key = keypair.public_key().to_bytes();
    let signing_public_key = vault
        .load_owner_signing_key(profile)?
        .public_key()
        .to_bytes();
    let now = unix_ms_now();
    let ttl_seconds = options.ttl_seconds.unwrap_or(900);
    let expires_at = now.saturating_add(ttl_seconds as u64 * 1000);
    let nonce = publish_nonce(profile, &public_key, now);
    if options.email.is_some() {
        return Err(Error::InvalidInput(
            "set the profile email with `lockbox vault profile email [profile] <email>` before publishing".to_string(),
        )
        .into());
    }
    let email = vault.profile_email(profile)?.ok_or_else(|| {
        cli_error(format!(
            "You may not publish a public key for a Profile that does not have an email address.\nThe profile `{profile}` has no email address.\nRun `lockbox vault profile email {profile} <email>`.\nThen run this command again."
        ))
    })?;
    let email = normalize_contact_email(&email)
        .map_err(|_| Error::InvalidInput("invalid profile email address".to_string()))?;
    let fingerprint = contact_fingerprint(&email, &public_key, &signing_public_key)
        .map_err(|_| Error::InvalidInput("invalid contact fingerprint fields".to_string()))?;
    let pool = publish_client_pool(&options)?;
    let result = pool
        .publish_contact(
            ttl_seconds,
            options.max_receives.unwrap_or(1),
            ContactPublish {
                profile,
                public_key: &public_key,
                signing_public_key: &signing_public_key,
                fingerprint: &fingerprint,
                publish_nonce: &nonce,
                created_at_unix_ms: now,
                expires_at_unix_ms: expires_at,
                verification_email: Some(&email),
            },
        )
        .map_err(clean_publish_error)?;
    println!("published=yes");
    println!("publish_code={}", result.publish_code);
    println!("email={email}");
    print_fingerprint_lines("contact_fingerprint", &fingerprint);
    println!(
        "fingerprint_purpose=do not send this fingerprint; ask the receiver to call you to obtain it"
    );
    println!("fingerprint_security={PUBLISH_FINGERPRINT_SECURITY_NOTE}");
    if let Some(url) = &result.verification_url {
        println!("verification_url={url}");
    }
    println!("verification_advice=check the inbox for {email} and click the verification link");
    println!(
        "verification_expires_at_utc={}",
        format_unix_ms_utc(result.expires_at_unix_ms)
    );
    println!(
        "verification_expires_at_unix_ms={}",
        result.expires_at_unix_ms
    );
    Ok(())
}

fn clean_publish_error(err: ClientError) -> Box<dyn std::error::Error> {
    match err {
        ClientError::Server {
            status: Status::RateLimited,
            ..
        } => cli_error("Too many verification emails. Try again later."),
        ClientError::Server {
            status: Status::StoreUnavailable,
            message,
        } if message == "could not send verification email" => {
            cli_error("Could not send verification email. Try again later.")
        }
        other => other.into(),
    }
}

fn receive_publish_options(options: PublishCliOptions) -> CliResult<()> {
    let publish_code = options
        .positionals
        .first()
        .cloned()
        .ok_or_else(|| Error::InvalidInput("missing publish code".to_string()))?;
    let expected_fingerprint = options
        .fingerprint
        .clone()
        .map(Ok)
        .unwrap_or_else(|| prompt_line("Fingerprint code from trusted second channel: "))?;
    let fingerprint_channel = verify_fingerprint_channel(options.fingerprint_channel.as_deref())?;
    let pool = publish_client_pool(&options)?;
    let received = pool.receive(&publish_code)?;
    let verification = received.email_verification.as_ref().ok_or_else(|| {
        Error::InvalidInput("publisher email has not been verified by the key server".to_string())
    })?;
    if !verification.verified {
        return Err(Error::InvalidInput(
            "publisher email has not been verified by the key server".to_string(),
        )
        .into());
    }
    let published_contact = revault_publish_protocol::decode_contact_publish(&received.payload)?;
    let computed_fingerprint = contact_fingerprint(
        &verification.email,
        &published_contact.public_key,
        &published_contact.signing_public_key,
    )
    .map_err(|_| Error::InvalidInput("invalid contact fingerprint fields".to_string()))?;
    if !fingerprint_input_matches(&expected_fingerprint, &computed_fingerprint)? {
        return Err(Error::InvalidInput(format!(
            "contact fingerprint mismatch for {}; expected {}, computed {}",
            verification.email,
            expected_fingerprint.trim(),
            format_fingerprint_code(&computed_fingerprint)
        ))
        .into());
    }
    let contact_name = options
        .positionals
        .get(1)
        .cloned()
        .unwrap_or_else(|| contact_name_from_email(&verification.email));
    let contact_public_key = ContactPublicKey::from_bytes(&published_contact.public_key)?;
    let signing_public = OwnerSigningPublicKey::from_bytes(&published_contact.signing_public_key)?;
    let vault = default_vault()?;
    if vault.contact_exists(&contact_name)? && !options.overwrite {
        return Err(Error::AlreadyExists(format!("contact {contact_name}")).into());
    }
    vault.store_contact(&contact_name, &contact_public_key)?;
    vault.store_contact_signing_key(&contact_name, &signing_public)?;
    println!("contact={contact_name}");
    println!("profile={}", published_contact.profile);
    println!("publish_code={publish_code}");
    println!("email={}", verification.email);
    print_fingerprint_lines("contact_fingerprint", &computed_fingerprint);
    println!("fingerprint_verified=yes");
    println!("fingerprint_channel={fingerprint_channel}");
    println!("fingerprint_security={PUBLISH_FINGERPRINT_SECURITY_NOTE}");
    println!("email_verification_email={}", verification.email);
    println!("email_verification_status=verified");
    println!(
        "email_verified_at_utc={}",
        format_unix_ms_utc(verification.verified_at_unix_ms)
    );
    println!(
        "email_verification_attestation={}",
        encode_hex(&verification.attestation)
    );
    println!("verification_advice={PUBLISH_RECEIVE_VERIFICATION_ADVICE}");
    Ok(())
}

#[derive(Default)]
struct PublishCliOptions {
    server: Option<String>,
    topology_url: Option<String>,
    ttl_seconds: Option<u32>,
    max_receives: Option<u16>,
    email: Option<String>,
    fingerprint: Option<String>,
    fingerprint_channel: Option<String>,
    overwrite: bool,
    positionals: Vec<String>,
}

impl PublishCliOptions {
    fn from_publish_matches(matches: &ArgMatches) -> CliResult<Self> {
        Ok(Self {
            server: optional_value(matches, "server").map(str::to_string),
            topology_url: optional_value(matches, "topology-url").map(str::to_string),
            ttl_seconds: optional_value(matches, "ttl").map(str::parse).transpose()?,
            max_receives: optional_value(matches, "max-receives")
                .map(str::parse)
                .transpose()?,
            email: None,
            fingerprint: None,
            fingerprint_channel: None,
            overwrite: false,
            positionals: optional_value(matches, "name")
                .map(|name| vec![name.to_string()])
                .unwrap_or_default(),
        })
    }

    fn from_receive_matches(matches: &ArgMatches) -> CliResult<Self> {
        let mut positionals = vec![required_value(matches, "publish-code")];
        if let Some(contact_name) = optional_value(matches, "contact-name") {
            positionals.push(contact_name.to_string());
        }
        Ok(Self {
            server: optional_value(matches, "server").map(str::to_string),
            topology_url: optional_value(matches, "topology-url").map(str::to_string),
            ttl_seconds: None,
            max_receives: None,
            email: None,
            fingerprint: optional_value(matches, "fingerprint").map(str::to_string),
            fingerprint_channel: optional_value(matches, "fingerprint-channel").map(str::to_string),
            overwrite: matches.get_flag("overwrite"),
            positionals,
        })
    }
}

fn publish_client_pool(options: &PublishCliOptions) -> CliResult<PublishClientPool> {
    if let Some(topology_url) = &options.topology_url {
        let pool = PublishClientPool::discover(&normalize_topology_url(topology_url))?;
        return with_persisted_publish_sticky(pool);
    }
    if let Some(server) = &options.server {
        return Ok(PublishClientPool::new([normalize_publish_url(server)])?);
    }
    let config = read_publish_config()?;
    if let Some(topology_url) = config.topology_url {
        let pool = PublishClientPool::discover(&normalize_topology_url(&topology_url))?;
        return with_persisted_publish_sticky(pool);
    }
    if let Some(server) = config.server {
        return Ok(PublishClientPool::new([normalize_publish_url(&server)])?);
    }
    let topology_urls = DEFAULT_PUBLISH_TOPOLOGY_URLS.map(normalize_topology_url);
    let pool = PublishClientPool::discover_from_urls(topology_urls)?;
    with_persisted_publish_sticky(pool)
}

const DEFAULT_PUBLISH_TOPOLOGY_URLS: [&str; 2] = [
    "https://keyshare0.revault.onepub.dev/v1/topology",
    "https://keyshare1.revault.onepub.dev/v1/topology",
];

#[derive(Default)]
struct PublishConfig {
    server: Option<String>,
    topology_url: Option<String>,
}

fn read_publish_config() -> CliResult<PublishConfig> {
    let path = std::env::var("LOCKBOX_PUBLISH_CONFIG")
        .map(PathBuf::from)
        .unwrap_or(default_vault_dir()?.join("config.yaml"));
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(PublishConfig::default()),
        Err(err) => return Err(err.into()),
    };
    let mut in_publish = false;
    let mut config = PublishConfig::default();
    for raw_line in text.lines() {
        let line = raw_line
            .split_once('#')
            .map(|(value, _)| value)
            .unwrap_or(raw_line);
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            in_publish = line.trim() == "publish:";
            continue;
        }
        if !in_publish {
            continue;
        }
        let Some((key, value)) = line.trim().split_once(':') else {
            continue;
        };
        let value = value.trim().trim_matches('"').to_string();
        match key.trim() {
            "server" => config.server = Some(value),
            "topology_url" => config.topology_url = Some(value),
            _ => {}
        }
    }
    Ok(config)
}

fn with_persisted_publish_sticky(pool: PublishClientPool) -> CliResult<PublishClientPool> {
    if let Some(sticky) = read_persisted_publish_sticky()? {
        let _ = pool.set_sticky_server(sticky.server_id, sticky.expires_at_unix_ms);
    }
    if let Some(sticky) = pool.ensure_sticky_server()? {
        write_persisted_publish_sticky(sticky)?;
    }
    Ok(pool)
}

fn read_persisted_publish_sticky() -> CliResult<Option<StickyPublishServer>> {
    let path = publish_sticky_path()?;
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.into()),
    };
    let mut server_id = None;
    let mut expires_at_unix_ms = None;
    for raw_line in text.lines() {
        let line = raw_line
            .split_once('#')
            .map(|(value, _)| value)
            .unwrap_or(raw_line)
            .trim();
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "server_id" => server_id = value.trim().parse::<u8>().ok(),
            "expires_at_unix_ms" => expires_at_unix_ms = value.trim().parse::<u64>().ok(),
            _ => {}
        }
    }
    let Some(server_id) = server_id else {
        return Ok(None);
    };
    let Some(expires_at_unix_ms) = expires_at_unix_ms else {
        return Ok(None);
    };
    if expires_at_unix_ms <= unix_ms_now() {
        return Ok(None);
    }
    Ok(Some(StickyPublishServer {
        server_id,
        expires_at_unix_ms,
    }))
}

fn write_persisted_publish_sticky(sticky: StickyPublishServer) -> CliResult<()> {
    let path = publish_sticky_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        format!(
            "server_id = {}\nexpires_at_unix_ms = {}\n",
            sticky.server_id, sticky.expires_at_unix_ms
        ),
    )?;
    Ok(())
}

fn publish_sticky_path() -> CliResult<PathBuf> {
    Ok(default_vault_dir()?.join(".publish-server-sticky"))
}

fn normalize_publish_url(value: &str) -> String {
    let value = value.trim();
    if value.starts_with("http://") || value.starts_with("https://") {
        value.to_string()
    } else {
        format!("http://{value}/v1/publish")
    }
}

fn normalize_topology_url(value: &str) -> String {
    let value = value.trim();
    if value.starts_with("http://") || value.starts_with("https://") {
        value.to_string()
    } else {
        format!("http://{value}/v1/topology")
    }
}

fn publish_nonce(profile: &str, public_key: &[u8], now: u64) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(profile.as_bytes());
    hasher.update(public_key);
    hasher.update(now.to_be_bytes());
    hasher.update(std::process::id().to_be_bytes());
    hasher.finalize()[..24].to_vec()
}

fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn format_unix_ms_utc(unix_ms: u64) -> String {
    let seconds = (unix_ms / 1000) as i64;
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    format!("{year:04}/{month:02}/{day:02} {hour:02}:{minute:02} UTC")
}

fn contact_name_from_email(email: &str) -> String {
    let mut name = String::with_capacity(email.len());
    let mut last_underscore = false;
    for ch in email.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            name.push(ch.to_ascii_lowercase());
            last_underscore = false;
        } else {
            if !last_underscore && !name.is_empty() {
                name.push('_');
            }
            last_underscore = true;
        }
    }
    while name.ends_with('_') {
        name.pop();
    }
    name
}

fn prompt_line(prompt: &str) -> CliResult<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    Ok(value.trim().to_string())
}

fn verify_fingerprint_channel(value: Option<&str>) -> CliResult<&'static str> {
    let selected = match value {
        Some(value) => value.to_string(),
        None => {
            println!("{FINGERPRINT_CHANNEL_PROMPT}");
            prompt_line("Fingerprint channel: ")?
        }
    };
    let normalized = selected
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '_', '/'], "-");
    match normalized.as_str() {
        "1" | "email" | "e-mail" => Err(Error::InvalidInput(
            "fingerprint channel rejected: email cannot be used because publisher email is already verified by the key server".to_string(),
        )
        .into()),
        "2" | "phone-call-from-owner" | "call-from-owner" | "owner-called"
        | "phone-call-from-key-owner" => Err(Error::InvalidInput(
            "fingerprint channel rejected: the receiver must initiate the fingerprint check"
                .to_string(),
        )
        .into()),
        "3" | "phone-call-to-owner" | "call-to-owner" | "called-owner"
        | "phone-call-to-key-owner" => Ok("phone-call-to-owner"),
        "4" | "text-from-owner" | "sms-from-owner" | "text-message-from-owner"
        | "sms-message-from-owner" | "text-from-key-owner" | "sms-from-key-owner" => {
            Err(Error::InvalidInput(
                "fingerprint channel rejected: the receiver must initiate the fingerprint check"
                    .to_string(),
            )
            .into())
        }
        "5" | "text-to-owner" | "sms-to-owner" | "text-message-to-owner"
        | "sms-message-to-owner" | "text-to-key-owner" | "sms-to-key-owner" => {
            Ok("sms-to-owner")
        }
        "6" | "in-person" | "inperson" | "face-to-face" => Ok("in-person"),
        _ => Err(Error::InvalidInput(format!(
            "unknown fingerprint channel: {selected}; use phone-call-to-owner, sms-to-owner, or in-person"
        ))
        .into()),
    }
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

fn decode_hex(value: &str) -> CliResult<Vec<u8>> {
    let value = value.trim();
    if !value.len().is_multiple_of(2) {
        return Err(Error::InvalidInput("hex value has odd length".to_string()).into());
    }
    let mut out = Vec::with_capacity(value.len() / 2);
    let bytes = value.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        let high = hex_digit(bytes[index])?;
        let low = hex_digit(bytes[index + 1])?;
        out.push((high << 4) | low);
        index += 2;
    }
    Ok(out)
}

fn hex_digit(byte: u8) -> CliResult<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(Error::InvalidInput("hex value contains non-hex digits".to_string()).into()),
    }
}

fn profile_restore_options(options: ProfileRestoreArgs) -> CliResult<()> {
    let vault = default_vault()?;
    let backup = read_profile_backup(&options.input_path)?;
    let name = options.name.unwrap_or(backup.name);
    let keypair = import_private_key(SecretVec::try_from_slice(
        backup.private_key_pem.as_bytes(),
    )?)?;
    let signing_key = owner_signing_key_from_hex_text(&backup.signing_private_hex)?;
    let existed = vault.private_key_exists(&name)?;
    let backup_path = if existed && options.overwrite {
        let backup_path = profile_restore_backup_path(&name)?;
        backup_default_vault(&backup_path, false)?;
        Some(backup_path)
    } else {
        None
    };
    vault.restore_private_key(&name, &keypair, Some(&signing_key), options.overwrite)?;
    let public_key = keypair.public_key();
    let fingerprint = public_key_fingerprint(&public_key);
    if existed && options.overwrite {
        println!("WARNING: replacing vault profile: {name}");
        println!("Existing vault backed up before profile restore.");
        if let Some(backup_path) = backup_path {
            println!("Backup:");
            println!("  {}", backup_path.display());
        }
    }
    println!("profile={name}");
    println!("public_key_fingerprint={}", format_hex_pairs(&fingerprint));
    println!("owner_signing_key=restored");
    Ok(())
}

fn profile_backup_options(options: ProfileBackupArgs) -> CliResult<()> {
    let vault = default_vault()?;
    let mut output = Vec::new();
    write_profile_backup(&mut output, &vault, &options.name)?;
    write_output_file(&options.output_path, &output, options.overwrite)?;
    println!("Profile backup completed successfully.");
    println!("profile={}", options.name);
    println!(
        "Backup path: {}",
        absolute_path(&PathBuf::from(&options.output_path))?.display()
    );
    Ok(())
}

fn write_output_file(path: &str, bytes: &[u8], overwrite: bool) -> CliResult<()> {
    let path = PathBuf::from(path);
    if path.exists() && !overwrite {
        return Err(Error::AlreadyExists(format!(
            "{}; pass --overwrite to replace it",
            path.display()
        ))
        .into());
    }
    fs::write(path, bytes)?;
    Ok(())
}

struct ParsedProfileBackup {
    name: String,
    private_key_pem: String,
    signing_private_hex: String,
}

fn read_profile_backup(path: &str) -> CliResult<ParsedProfileBackup> {
    parse_profile_backup(&fs::read_to_string(path)?)
}

fn parse_profile_backup(text: &str) -> CliResult<ParsedProfileBackup> {
    let name = text
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("Profile:").map(str::trim))
        .filter(|name| !name.is_empty())
        .ok_or_else(|| Error::InvalidInput("profile backup is missing Profile".to_string()))?
        .to_string();
    let private_key_pem = extract_labeled_block(
        text,
        "BEGIN LOCKBOX PRIVATE KEY",
        "END LOCKBOX PRIVATE KEY",
        "profile backup is missing profile private key",
    )?;
    let signing_private_hex = extract_owner_signing_hex(text)?;
    Ok(ParsedProfileBackup {
        name,
        private_key_pem,
        signing_private_hex,
    })
}

fn extract_labeled_block(
    text: &str,
    begin_label: &str,
    end_label: &str,
    missing_message: &str,
) -> CliResult<String> {
    let lines = text.lines().collect::<Vec<_>>();
    let start = lines
        .iter()
        .position(|line| line.contains(begin_label))
        .ok_or_else(|| Error::InvalidInput(missing_message.to_string()))?;
    let end = lines[start..]
        .iter()
        .position(|line| line.contains(end_label))
        .map(|offset| start + offset)
        .ok_or_else(|| Error::InvalidInput(missing_message.to_string()))?;
    let mut block = lines[start..=end].join("\n");
    block.push('\n');
    Ok(block)
}

fn extract_owner_signing_hex(text: &str) -> CliResult<String> {
    let Some((_, rest)) = text.split_once("Owner signing private key record (hex):") else {
        return Err(Error::InvalidInput(
            "profile backup is missing owner signing private key".to_string(),
        )
        .into());
    };
    let hex = rest
        .lines()
        .map(str::trim)
        .skip_while(|line| line.is_empty())
        .take_while(|line| !line.is_empty() && line.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .collect::<String>();
    if hex.is_empty() {
        return Err(Error::InvalidInput(
            "profile backup is missing owner signing private key".to_string(),
        )
        .into());
    }
    Ok(hex)
}

fn owner_signing_key_from_hex_text(text: &str) -> CliResult<OwnerSigningKeyPair> {
    let hex = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && line.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .collect::<String>();
    if hex.is_empty() {
        return Err(Error::InvalidInput(
            "owner signing private key file does not contain hex key material".to_string(),
        )
        .into());
    }
    let bytes = decode_hex(&hex)?;
    let record = SecretVec::try_from_slice(&bytes)?;
    Ok(OwnerSigningKeyPair::from_private_key_record(record)?)
}

fn remove_key_options(name: Option<&str>, force: bool) -> CliResult<()> {
    let name = name.unwrap_or(VaultDirectory::DEFAULT_KEY_NAME);
    if !force && !confirm_private_key_removal(name)? {
        println!("Vault profile not removed: {name}");
        return Ok(());
    }
    default_vault()?.delete_private_key(name)?;
    println!("Vault profile removed: {name}");
    Ok(())
}

fn rotate_key(args: &[String]) -> CliResult<()> {
    let name = args
        .first()
        .map(String::as_str)
        .unwrap_or(VaultDirectory::DEFAULT_KEY_NAME);
    let history = default_vault()?.rotate_private_key(name)?;
    println!("Rotated vault profile: {name}");
    println!("Active generation: {}", history.active_generation);
    println!(
        "Run `lockbox access refresh --all {name}` to update remembered lockboxes that use this profile."
    );
    Ok(())
}

fn remove_contact_name(name: &str) -> CliResult<()> {
    default_vault()?.delete_contact(name)?;
    Ok(())
}

fn list_profiles_with_format(format: OutputFormat) -> CliResult<()> {
    let vault = default_vault()?;
    let mut rows = Vec::new();
    for name in vault.list_private_keys()? {
        let email = vault
            .profile_email(&name)?
            .unwrap_or_else(|| "-".to_string());
        rows.push(vec![name, email]);
    }
    print_records(&["name", "email"], rows, format)?;
    Ok(())
}

fn profile_email_values(args: &[String]) -> CliResult<()> {
    let (name, email) = match args {
        [email] => (VaultDirectory::DEFAULT_KEY_NAME, email.as_str()),
        [name, email, ..] => (name.as_str(), email.as_str()),
        [] => {
            return Err(Error::InvalidInput("missing profile email address".to_string()).into());
        }
    };
    let email = normalize_contact_email(email)
        .map_err(|_| Error::InvalidInput("invalid profile email address".to_string()))?;
    default_vault()?.store_profile_email(name, &email)?;
    println!("profile={name}");
    println!("email={email}");
    Ok(())
}

fn profile_history_with_format(name: &str, format: OutputFormat) -> CliResult<()> {
    let history = default_vault()?.list_profile_generations(name)?;
    let rows = history
        .generations
        .into_iter()
        .map(|generation| {
            vec![
                history.name.clone(),
                generation.index.to_string(),
                profile_generation_status(generation.status).to_string(),
                encode_hex(&generation.contact_fingerprint),
                generation.created_at_unix_ms.to_string(),
                generation
                    .retired_at_unix_ms
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect::<Vec<_>>();
    print_records(
        &[
            "profile",
            "generation",
            "status",
            "fingerprint",
            "created_at_unix_ms",
            "retired_at_unix_ms",
        ],
        rows,
        format,
    )?;
    Ok(())
}

fn profile_fingerprint(args: &[String]) -> CliResult<()> {
    if args.len() > 1 {
        return Err(Error::InvalidInput(
            "vault profile fingerprint accepts at most one profile name".to_string(),
        )
        .into());
    }
    let profile = args
        .first()
        .map(String::as_str)
        .unwrap_or(VaultDirectory::DEFAULT_KEY_NAME);
    let vault = default_vault()?;
    let keypair = vault.load_private_key(profile)?;
    let public_key = keypair.public_key().to_bytes();
    let signing_public_key = vault
        .load_owner_signing_key(profile)?
        .public_key()
        .to_bytes();
    let email = vault.profile_email(profile)?.ok_or_else(|| {
        cli_error(format!(
            "Cannot calculate the publish fingerprint for `{profile}` because it has no email address.\nRun `lockbox vault profile email {profile} <email>`.\nThen run this command again."
        ))
    })?;
    let email = normalize_contact_email(&email)
        .map_err(|_| Error::InvalidInput("invalid profile email address".to_string()))?;
    let fingerprint = contact_fingerprint(&email, &public_key, &signing_public_key)
        .map_err(|_| Error::InvalidInput("invalid contact fingerprint fields".to_string()))?;

    println!("profile={profile}");
    println!("email={email}");
    print_fingerprint_lines("contact_fingerprint", &fingerprint);
    println!(
        "fingerprint_purpose=do not send this fingerprint; ask the receiver to call you to obtain it"
    );
    println!("fingerprint_security={PUBLISH_FINGERPRINT_SECURITY_NOTE}");
    Ok(())
}

fn list_contacts_with_format(format: OutputFormat) -> CliResult<()> {
    let vault = default_vault()?;
    let mut rows = Vec::new();
    for contact in vault.list_contacts()? {
        rows.push(vec![contact.name]);
    }
    print_records(&["name"], rows, format)?;
    Ok(())
}

fn list_known_lockboxes_with_format(format: OutputFormat) -> CliResult<()> {
    let vault = default_vault()?;
    let mut rows = Vec::<KnownLockboxListRow>::new();
    for lockbox in vault.list_known_lockboxes()? {
        rows.push(known_lockbox_list_row(&lockbox));
    }
    match format {
        OutputFormat::Table => print_known_lockbox_table(&rows),
        OutputFormat::Tsv | OutputFormat::Json => {
            let rows = rows
                .into_iter()
                .map(|row| {
                    vec![
                        row.name,
                        row.state,
                        row.owner,
                        row.size,
                        row.lockbox_id,
                        row.path,
                    ]
                })
                .collect::<Vec<_>>();
            print_records(
                &["name", "state", "owner", "size", "lockbox_id", "path"],
                rows,
                format,
            )?;
        }
    }
    Ok(())
}

struct KnownLockboxListRow {
    name: String,
    state: String,
    owner: String,
    size: String,
    lockbox_id: String,
    path: String,
}

fn known_lockbox_list_row(lockbox: &revault_vault_api::KnownLockbox) -> KnownLockboxListRow {
    let path = Path::new(&lockbox.path);
    let mut owner = "-".to_string();
    let mut size = "-".to_string();
    let state = match fs::metadata(path) {
        Ok(metadata) => {
            size = human_size(metadata.len());
            if let Ok(inspection) = Lockbox::inspect_file(path) {
                owner = if inspection.owner_signed {
                    "signed".to_string()
                } else {
                    "unsigned".to_string()
                };
            }
            "present"
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => "missing",
        Err(_) => "inaccessible",
    }
    .to_string();
    KnownLockboxListRow {
        name: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(lockbox.path.as_str())
            .to_string(),
        state,
        owner,
        size,
        lockbox_id: lockbox.lockbox_id.to_string(),
        path: lockbox.path.clone(),
    }
}

fn print_known_lockbox_table(rows: &[KnownLockboxListRow]) {
    if rows.is_empty() {
        println!("empty");
        return;
    }
    let name_width = column_width("name", rows.iter().map(|row| row.name.as_str()));
    let state_width = column_width("state", rows.iter().map(|row| row.state.as_str()));
    let owner_width = column_width("owner", rows.iter().map(|row| row.owner.as_str()));
    let size_width = column_width("size", rows.iter().map(|row| row.size.as_str()));
    let id_width = column_width("lockbox_id", rows.iter().map(|row| row.lockbox_id.as_str()));
    println!(
        "{:<name_width$}  {:<state_width$}  {:<owner_width$}  {:>size_width$}  {:<id_width$}  path",
        "name", "state", "owner", "size", "lockbox_id"
    );
    for row in rows {
        println!(
            "{:<name_width$}  {:<state_width$}  {:<owner_width$}  {:>size_width$}  {:<id_width$}  {}",
            row.name, row.state, row.owner, row.size, row.lockbox_id, row.path
        );
    }
}

fn column_width<'a>(header: &str, values: impl Iterator<Item = &'a str>) -> usize {
    values.fold(header.len(), |width, value| width.max(value.len()))
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 7] = ["B", "K", "M", "G", "T", "P", "E"];
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        return format!("{bytes}B");
    }
    if value >= 100.0 {
        format!("{value:.0}{}", UNITS[unit])
    } else if value >= 10.0 {
        format!("{value:.1}{}", UNITS[unit])
    } else {
        format!("{value:.2}{}", UNITS[unit])
    }
}

fn forget_known_lockbox_path(path: &str) -> CliResult<()> {
    default_vault()?.forget_known_lockbox(path)?;
    println!("Forgot known lockbox: {path}");
    Ok(())
}

fn move_known_lockbox(source: &str, destination: &str) -> CliResult<()> {
    let source_path = Path::new(source);
    let requested_destination = Path::new(destination);
    if !source_path.exists() {
        return Err(cli_error(format!("lockbox not found: {source}")));
    }
    if !source_path.is_file() {
        return Err(cli_error(format!("lockbox path is not a file: {source}")));
    }
    let destination_path = if requested_destination.is_dir() {
        let file_name = source_path
            .file_name()
            .ok_or_else(|| cli_error(format!("source lockbox has no file name: {source}")))?;
        requested_destination.join(file_name)
    } else {
        requested_destination.to_path_buf()
    };
    if destination_path.exists() {
        return Err(cli_error(format!(
            "destination already exists: {}; no files or vault records were changed",
            destination_path.display()
        )));
    }
    let parent = destination_path.parent().unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(cli_error(format!(
            "destination directory does not exist: {}; no files or vault records were changed",
            parent.display()
        )));
    }

    let lockbox_id = VaultOpen::read_lockbox_id(source_path)?;
    let was_default = default_matches(source)?;
    let vault = default_vault()?;
    local_vault().close_lockbox(source_path)?;
    let _lock = ScopedFileLock::acquire(source_path, FileLockScope::Lockbox)?;
    fs::rename(source_path, &destination_path).map_err(|err| {
        cli_error(format!(
            "could not move {source} to {}: {err}; no vault records were changed",
            destination_path.display()
        ))
    })?;

    let old_lock = lock_path_for(source_path);
    let new_lock = lock_path_for(&destination_path);
    if let Err(err) = fs::rename(&old_lock, &new_lock) {
        let _ = fs::rename(&destination_path, source_path);
        return Err(cli_error(format!(
            "could not move lock sidecar {} to {}: {err}; the lockbox move was rolled back",
            old_lock.display(),
            new_lock.display()
        )));
    }

    if let Err(err) = vault
        .forget_known_lockbox(source_path)
        .and_then(|()| vault.remember_known_lockbox(lockbox_id, &destination_path))
    {
        return Err(cli_error(format!(
            "lockbox moved to {}, but the vault path update failed: {err}. Run `lockbox vault lockbox forget {}` and reopen the destination.",
            destination_path.display(),
            source_path.display()
        )));
    }
    if was_default {
        replace_default_after_move(&destination_path)?;
    }
    println!("Lockbox moved: {source} -> {}", destination_path.display());
    if was_default {
        println!("Session default updated: {}", destination_path.display());
    }
    Ok(())
}

fn profile_generation_status(status: ProfileGenerationStatus) -> &'static str {
    match status {
        ProfileGenerationStatus::Active => "active",
        ProfileGenerationStatus::Retired => "retired",
        ProfileGenerationStatus::Compromised => "compromised",
    }
}

fn ensure_default_private_key(vault: &VaultDirectory) -> CliResult<bool> {
    if vault.private_key_exists(VaultDirectory::DEFAULT_KEY_NAME)? {
        return Ok(false);
    }
    vault.store_private_key(
        VaultDirectory::DEFAULT_KEY_NAME,
        &ContactKeyPair::generate()?,
    )?;
    Ok(true)
}

fn print_default_profile_backup(vault: &VaultDirectory) -> CliResult<()> {
    println!();
    write_profile_backup(&mut io::stdout(), vault, VaultDirectory::DEFAULT_KEY_NAME)
}

fn write_profile_backup(
    writer: &mut impl Write,
    vault: &VaultDirectory,
    profile: &str,
) -> CliResult<()> {
    let keypair = vault.load_private_key(profile)?;
    let public_key = keypair.public_key();
    let private_bytes = export_private_key(&keypair, KeyFormat::LockboxPem)?;
    let signing_key = vault.load_owner_signing_key(profile)?;
    let signing_private_record = signing_key.private_key_record()?;
    let fingerprint = public_key_fingerprint(&public_key);

    writeln!(
        writer,
        "Store the following private keys and your vault passphrase somewhere safe."
    )?;
    writeln!(
        writer,
        "Anyone with the profile private key can open lockboxes granted to this profile."
    )?;
    writeln!(
        writer,
        "Anyone with the signing private key can sign lockbox changes as this profile."
    )?;
    writeln!(writer)?;
    writeln!(writer, "Profile backup:")?;
    writeln!(writer, "  Profile: {profile}")?;
    writeln!(
        writer,
        "  Public key fingerprint: {}",
        format_hex_pairs(&fingerprint)
    )?;
    writeln!(writer)?;
    writeln!(writer, "Profile private key:")?;
    private_bytes.with_bytes(|bytes| writer.write_all(bytes))??;
    private_bytes.with_bytes(|bytes| {
        if !bytes.ends_with(b"\n") {
            writeln!(writer)?;
        }
        Ok::<_, io::Error>(())
    })??;
    writeln!(writer)?;
    writeln!(writer, "Owner signing private key record (hex):")?;
    signing_private_record.with_bytes(|bytes| write_wrapped_hex(writer, bytes))??;
    Ok(())
}

fn write_wrapped_hex(writer: &mut impl Write, bytes: &[u8]) -> io::Result<()> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for chunk in bytes.chunks(32) {
        for byte in chunk {
            writer.write_all(&[HEX[(byte >> 4) as usize], HEX[(byte & 0x0f) as usize]])?;
        }
        writeln!(writer)?;
    }
    Ok(())
}

fn export_public_options(options: ProfileExportArgs, format: KeyFormat) -> CliResult<()> {
    let vault = default_vault()?;
    let keypair = vault.load_private_key(&options.name)?;
    let public_key = keypair.public_key();
    let fingerprint = public_key_fingerprint(&public_key);
    fs::write(
        &options.output_path,
        export_public_key(&public_key, format)?,
    )?;
    println!("profile={}", options.name);
    print_fingerprint_lines("public_key_fingerprint", &fingerprint);
    Ok(())
}

fn confirm_private_key_removal(name: &str) -> CliResult<bool> {
    eprintln!("Remove vault profile '{name}'?");
    eprintln!(
        "Lockboxes that only this private key can open may become inaccessible from this vault."
    );
    eprint!("Type 'yes' to remove it: ");
    io::stderr().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(answer.trim() == "yes")
}

struct ProfileBackupArgs {
    name: String,
    output_path: String,
    overwrite: bool,
}

struct ProfileRestoreArgs {
    name: Option<String>,
    input_path: String,
    overwrite: bool,
}

struct ProfileExportArgs {
    name: String,
    output_path: String,
}

impl ProfileBackupArgs {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            name: matches
                .get_one::<String>("name")
                .cloned()
                .unwrap_or_else(|| VaultDirectory::DEFAULT_KEY_NAME.to_string()),
            output_path: required_value(matches, "output"),
            overwrite: matches.get_flag("overwrite"),
        }
    }
}

impl ProfileRestoreArgs {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            name: matches.get_one::<String>("name").cloned(),
            input_path: required_value(matches, "input"),
            overwrite: matches.get_flag("overwrite"),
        }
    }
}

impl ProfileExportArgs {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            name: matches
                .get_one::<String>("name")
                .cloned()
                .unwrap_or_else(|| VaultDirectory::DEFAULT_KEY_NAME.to_string()),
            output_path: required_value(matches, "output"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        clean_publish_error, contact_name_from_email, decode_fingerprint_crockford_96,
        decode_fingerprint_hex, fingerprint_input_matches, format_fingerprint_code,
        format_fingerprint_reading, format_hex_pairs, format_unix_ms_utc,
        verify_fingerprint_channel, DEFAULT_PUBLISH_TOPOLOGY_URLS,
        PUBLISH_RECEIVE_VERIFICATION_ADVICE,
    };
    use revault_publish_protocol::protocol::Status;
    use revault_publish_protocol::ClientError;

    #[test]
    fn publish_defaults_to_redundant_topology_bootstrap() {
        assert_eq!(
            DEFAULT_PUBLISH_TOPOLOGY_URLS,
            [
                "https://keyshare0.revault.onepub.dev/v1/topology",
                "https://keyshare1.revault.onepub.dev/v1/topology",
            ]
        );
    }

    #[test]
    fn publish_expiry_uses_human_readable_utc_time() {
        assert_eq!(format_unix_ms_utc(0), "1970/01/01 00:00 UTC");
        assert_eq!(
            format_unix_ms_utc(1_783_032_923_000),
            "2026/07/02 22:55 UTC"
        );
    }

    #[test]
    fn receive_publish_advice_requires_contact_initiated_trusted_channel() {
        assert!(PUBLISH_RECEIVE_VERIFICATION_ADVICE.contains("channel you already trust"));
        assert!(PUBLISH_RECEIVE_VERIFICATION_ADVICE.contains("You must initiate"));
        assert!(PUBLISH_RECEIVE_VERIFICATION_ADVICE.contains("do not accept it"));
    }

    #[test]
    fn contact_fingerprint_hex_uses_lowercase_pairs() {
        let bytes = [
            0x00, 0x01, 0x0a, 0x0b, 0x10, 0x11, 0x7f, 0x80, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0,
            0xfe, 0xff,
        ];
        let formatted = format_hex_pairs(&bytes);
        assert_eq!(formatted, "00 01 0a 0b 10 11 7f 80 ab bc cd de ef f0 fe ff");
        assert_eq!(decode_fingerprint_hex(&formatted).unwrap(), bytes);
        assert_eq!(
            decode_fingerprint_hex("00:01:0A:0B:10:11:7F:80:AB:BC:CD:DE:EF:F0:FE:FF").unwrap(),
            bytes
        );
        let short = decode_fingerprint_hex("123456").unwrap_err().to_string();
        assert!(short.contains("short PINs"));
        assert!(short.contains("authenticate a public key"));
    }

    #[test]
    fn fingerprint_code_uses_lowercase_crockford_and_accepts_uppercase() {
        let bytes = [
            0x00, 0x01, 0x0a, 0x0b, 0x10, 0x11, 0x7f, 0x80, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0,
            0xfe, 0xff,
        ];
        let code = format_fingerprint_code(&bytes);
        let groups = code.split('-').collect::<Vec<_>>();
        assert_eq!(groups.len(), 5);
        assert!(groups.iter().all(|group| group.len() == 4));
        assert_eq!(code, code.to_ascii_lowercase());
        for ambiguous in ['i', 'l', 'o', 'u'] {
            assert!(!code.contains(ambiguous));
        }

        let decoded = decode_fingerprint_crockford_96(&code).unwrap();
        assert_eq!(&decoded, &bytes[..12]);
        assert_eq!(
            decode_fingerprint_crockford_96(&code.to_ascii_uppercase()).unwrap(),
            decoded
        );
        assert!(fingerprint_input_matches(&code.to_ascii_uppercase(), &bytes).unwrap());
        assert!(fingerprint_input_matches(&format_hex_pairs(&bytes), &bytes).unwrap());
        let reading = format_fingerprint_reading(&code);
        assert_eq!(reading, reading.to_ascii_lowercase());
        assert!(reading.contains(" - "));
        assert!(reading.contains("zero"));
    }

    #[test]
    fn fingerprint_channel_requires_receiver_initiated_second_channel() {
        assert_eq!(
            verify_fingerprint_channel(Some("phone-call-to-owner")).unwrap(),
            "phone-call-to-owner"
        );
        assert_eq!(
            verify_fingerprint_channel(Some("5")).unwrap(),
            "sms-to-owner"
        );
        assert_eq!(
            verify_fingerprint_channel(Some("in person")).unwrap(),
            "in-person"
        );

        let email = verify_fingerprint_channel(Some("email"))
            .unwrap_err()
            .to_string();
        assert!(email.contains("email cannot be used"));

        let owner_initiated = verify_fingerprint_channel(Some("sms-from-owner"))
            .unwrap_err()
            .to_string();
        assert!(owner_initiated.contains("receiver must initiate"));
    }

    #[test]
    fn contact_name_defaults_from_email() {
        assert_eq!(
            contact_name_from_email("alice.publisher@example.test"),
            "alice_publisher_example_test"
        );
    }

    #[test]
    fn publish_rate_limit_error_is_short() {
        let err = clean_publish_error(ClientError::Server {
            status: Status::RateLimited,
            message: "rate limited".to_string(),
        });

        assert_eq!(
            err.to_string(),
            "Too many verification emails. Try again later."
        );
    }

    #[test]
    fn publish_smtp_error_is_short() {
        let err = clean_publish_error(ClientError::Server {
            status: Status::StoreUnavailable,
            message: "could not send verification email".to_string(),
        });

        assert_eq!(
            err.to_string(),
            "Could not send verification email. Try again later."
        );
    }
}
