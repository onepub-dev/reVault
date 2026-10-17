use super::context::{
    cli_error, default_vault, open_default_vault_with_password, open_existing_read_only,
    remember_default_vault_password_with_warning, vault_password_without_open, Access, CliResult,
};
use super::migration_exporters::{capabilities_match, ExporterRelease};
use crate::secret_prompt::prompt_secret;
use clap::ArgMatches;
use revault_lockbox_api::{
    probe_lockbox_format_version, Lockbox, LockboxOpen, OwnerSigningKeyPair, SecretString,
    SecretVec, LOCKBOX_FORMAT_VERSION,
};
use revault_migration::{
    export_archive, export_vault, import_archive, import_vault, upgrade_archive_artifact,
    upgrade_vault_artifact, verify_archive_artifact, verify_vault_artifact, ArtifactKind,
    MigrationJournal, MigrationPassphrase, MigrationStage,
};
use revault_vault_api::{default_vault_dir, VaultDirectory, CURRENT_VAULT_STRUCTURE_VERSION};
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) fn run_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    match matches.subcommand() {
        Some(("vault", sub)) => vault_matches(sub),
        Some(("lockbox", sub)) => lockbox_matches(sub, access),
        Some(("all", _)) => migrate_all(access),
        Some((other, _)) => Err(cli_error(format!("unknown migration command: {other}"))),
        None => Err(cli_error("migrate requires vault, lockbox, or all")),
    }
}

fn migrate_all(access: &Access) -> CliResult<()> {
    if super::command_lockbox().is_some() {
        return Err(cli_error(
            "migrate all is not lockbox-scoped; run `lbx doctor migrate all --replace`",
        ));
    }
    let vault_root = default_vault_dir()?;
    if !vault_root.join("local-vault.lbox").try_exists()? {
        let password = vault_password_without_open()?;
        if !recover_interrupted_replacement(&vault_root, ArtifactKind::Vault, &password)? {
            return Err(cli_error(
                "migrate all requires an existing local Vault; no Vault was created",
            ));
        }
    }
    println!("Migrating Vault: {}", vault_root.display());
    migrate_vault(true, None, None)?;
    let known = default_vault()?.list_known_lockboxes()?;
    let mut paths = std::collections::BTreeSet::new();
    let mut succeeded = 0;
    let mut failed = 0;
    for lockbox in known {
        if !paths.insert(lockbox.path.clone()) {
            continue;
        }
        println!("Migrating Lockbox: {}", lockbox.path);
        super::set_command_lockbox(Some(lockbox.path.clone()));
        super::COMMAND_LOCKBOX_ID.with(|id| *id.borrow_mut() = Some(lockbox.lockbox_id));
        let result = (|| {
            if !Path::new(&lockbox.path).try_exists()? {
                let password = vault_password_without_open()?;
                if !recover_interrupted_replacement(
                    Path::new(&lockbox.path),
                    ArtifactKind::Archive,
                    &password,
                )? {
                    return Err(cli_error("known Lockbox is missing; restore it at its recorded path and retry, or forget its Vault record"));
                }
            }
            let version = probe_archive_path(Path::new(&lockbox.path))?;
            if version == u32::from(LOCKBOX_FORMAT_VERSION) {
                super::aliases::check_identity(&lockbox.path, lockbox.lockbox_id)?;
            }
            // Historical formats need their frozen exporters. The imported output's
            // preserved identity is checked before replacing the source below.
            migrate_archive(PathBuf::from(&lockbox.path), true, None, None, access)
        })();
        super::set_command_lockbox(None);
        super::COMMAND_LOCKBOX_ID.with(|id| *id.borrow_mut() = None);
        match result {
            Ok(()) => {
                succeeded += 1;
                println!("Completed Lockbox: {}", lockbox.path);
            }
            Err(error) => {
                failed += 1;
                eprintln!("Failed Lockbox {}: {error}", lockbox.path);
            }
        }
    }
    println!("Migration summary: Vault complete; {succeeded} Lockboxes complete; {failed} failed.");
    if failed > 0 {
        return Err(cli_error("some known Lockboxes could not be migrated; resolve the reported failures and rerun `lbx doctor migrate all --replace`"));
    }
    Ok(())
}

fn vault_matches(matches: &ArgMatches) -> CliResult<()> {
    match matches.subcommand() {
        Some(("export", sub)) => vault_export(sub),
        Some(("upgrade", sub)) => vault_upgrade(sub),
        Some(("import", sub)) => vault_import(sub),
        Some(("verify", sub)) => vault_verify(sub),
        Some((other, _)) => Err(cli_error(format!(
            "unknown vault migration command: {other}"
        ))),
        None => migrate_vault_direct(matches),
    }
}

fn lockbox_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    match matches.subcommand() {
        Some(("export", sub)) => archive_export(sub, access),
        Some(("upgrade", sub)) => archive_upgrade(sub),
        Some(("import", sub)) => archive_import(sub),
        Some(("verify", sub)) => archive_verify(sub),
        Some((other, _)) => Err(cli_error(format!(
            "unknown lockbox migration command: {other}"
        ))),
        None => migrate_archive_direct(matches, access),
    }
}

fn vault_export(matches: &ArgMatches) -> CliResult<()> {
    let output = required_path(matches, "output")?;
    let stdin_vault = matches.get_flag("vault-password-stdin");
    let stdin_artifact = matches.get_flag("migration-password-stdin");
    let mut stdin_secrets =
        read_secret_lines_stdin(usize::from(stdin_vault) + usize::from(stdin_artifact))?
            .into_iter();
    let vault_password = stdin_vault.then(|| stdin_secrets.next().expect("vault stdin secret"));
    let artifact_password = if stdin_artifact {
        stdin_secrets.next().expect("migration stdin secret")
    } else {
        migration_password()?
    };
    let vault = match vault_password {
        Some(password) => super::context::open_default_vault_with_password(&password)?,
        None => default_vault()?,
    };
    let operation_id = random_id()?;
    let count = export_vault(&vault, &output, &artifact_password, operation_id)?;
    println!(
        "Exported {count} vault migration records to {}",
        output.display()
    );
    Ok(())
}

fn vault_upgrade(matches: &ArgMatches) -> CliResult<()> {
    let input = required_path(matches, "artifact")?;
    let output = required_path(matches, "output")?;
    let password = migration_password()?;
    let count = upgrade_vault_artifact(&input, &output, &password)?;
    println!(
        "Upgraded {count} vault migration records to {}",
        output.display()
    );
    Ok(())
}

fn vault_import(matches: &ArgMatches) -> CliResult<()> {
    let input = required_path(matches, "artifact")?;
    let output = required_path(matches, "output")?;
    let artifact_password = migration_password()?;
    let vault_password = vault_password_without_open()?;
    let count = import_vault(&input, &artifact_password, &output, &vault_password)?;
    println!(
        "Imported {count} vault migration records into {}",
        output.display()
    );
    Ok(())
}

fn vault_verify(matches: &ArgMatches) -> CliResult<()> {
    let input = required_path(matches, "artifact")?;
    let password = migration_password()?;
    let count = verify_vault_artifact(&input, &password)?;
    println!("Vault migration artifact is valid ({count} records).");
    Ok(())
}

fn archive_source(matches: &ArgMatches) -> CliResult<String> {
    match (
        super::command_lockbox(),
        matches.get_one::<String>("lockbox"),
    ) {
        (Some(_), Some(_)) => Err(cli_error(
            "pass the lockbox either before doctor or as the migration argument, not both",
        )),
        (Some(path), None) => Ok(path),
        (None, Some(value)) => {
            let (path, identity) = super::aliases::resolve(value)?;
            super::COMMAND_LOCKBOX_ID.with(|id| *id.borrow_mut() = identity);
            super::set_command_lockbox(Some(path.clone()));
            Ok(path)
        }
        (None, None) => super::default_lockbox_for_command(),
    }
}

fn archive_export(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let source = archive_source(matches)?;
    let output = required_path(matches, "output")?;
    let artifact_password = if matches.get_flag("migration-password-stdin") {
        read_secret_lines_stdin(1)?.remove(0)
    } else {
        migration_password()?
    };
    let lockbox = open_existing_read_only(&source, access)?;
    let operation_id = random_id()?;
    let count = export_archive(&lockbox, &output, &artifact_password, operation_id)?;
    println!(
        "Exported {count} lockbox migration records to {}",
        output.display()
    );
    Ok(())
}

fn archive_import(matches: &ArgMatches) -> CliResult<()> {
    let input = required_path(matches, "artifact")?;
    let output = PathBuf::from(super::aliases::resolve(&required_string(matches, "output")?)?.0);
    let password = migration_password()?;
    let signing = default_signing_key()?;
    let count = import_archive(&input, &password, &output, &signing)?;
    println!(
        "Imported {count} lockbox migration records into {}",
        output.display()
    );
    Ok(())
}

fn archive_upgrade(matches: &ArgMatches) -> CliResult<()> {
    let input = required_path(matches, "artifact")?;
    let output = required_path(matches, "output")?;
    let password = migration_password()?;
    let count = upgrade_archive_artifact(&input, &output, &password)?;
    println!(
        "Upgraded {count} lockbox migration frames to {}",
        output.display()
    );
    Ok(())
}

fn archive_verify(matches: &ArgMatches) -> CliResult<()> {
    let input = required_path(matches, "artifact")?;
    let password = migration_password()?;
    let count = verify_archive_artifact(&input, &password)?;
    println!("Lockbox migration artifact is valid ({count} frames).");
    Ok(())
}

fn migrate_vault_direct(matches: &ArgMatches) -> CliResult<()> {
    let replace = matches.get_flag("replace");
    let requested_output = matches.get_one::<String>("output").map(PathBuf::from);
    let exporter = matches.get_one::<String>("exporter").map(PathBuf::from);
    migrate_vault(replace, requested_output, exporter)
}

fn migrate_vault(
    replace: bool,
    requested_output: Option<PathBuf>,
    exporter: Option<PathBuf>,
) -> CliResult<()> {
    require_destination(replace, requested_output.as_deref(), "vault")?;
    let source = default_vault_dir()?;
    let source_password = vault_password_without_open()?;
    if !source.exists()
        && recover_interrupted_replacement(&source, ArtifactKind::Vault, &source_password)?
    {
        remember_default_vault_password_with_warning(
            &source_password,
            "the interrupted vault replacement completed successfully",
        );
        println!("Completed the interrupted vault replacement.");
        return Ok(());
    }
    let source_container_version = probe_archive_path(&source.join("local-vault.lbox"))?;
    let experimental_export = source_container_version == 4 && exporter.is_some();
    let source_version = if experimental_export {
        CURRENT_VAULT_STRUCTURE_VERSION
    } else {
        probe_vault_source_version(&source, &source_password)?
    };
    if source_version > CURRENT_VAULT_STRUCTURE_VERSION {
        return Err(cli_error(
            "this build cannot migrate a newer Vault structure; use the release that created it",
        ));
    }
    if replace
        && !experimental_export
        && source_version == CURRENT_VAULT_STRUCTURE_VERSION
        && source_container_version == u32::from(LOCKBOX_FORMAT_VERSION)
    {
        remember_default_vault_password_with_warning(
            &source_password,
            "the vault passphrase was verified successfully",
        );
        println!(
            "Vault already uses container format {LOCKBOX_FORMAT_VERSION} and structure version {CURRENT_VAULT_STRUCTURE_VERSION}. No migration needed."
        );
        return Ok(());
    }
    // A current structure needs only a container rewrite. Preserve every raw
    // record through the archive workflow, including aliases and credentials
    // that are not represented by historical Vault migration schemas.
    let preserve_records =
        !experimental_export && source_version == CURRENT_VAULT_STRUCTURE_VERSION;
    let fingerprint = fingerprint_path(&source.join("local-vault.lbox"))?;
    let operation_id = deterministic_operation_id(
        ArtifactKind::Vault,
        &source,
        requested_output.as_deref(),
        fingerprint,
    );
    let output =
        requested_output.unwrap_or_else(|| temporary_vault_destination(&source, operation_id));
    let work_dir = migration_work_dir(&source, operation_id)?;
    let artifact = work_dir.join("vault.source.migration");
    let upgraded = work_dir.join("vault.latest.migration");
    let journal_path = work_dir.join("vault.migration-state");
    fs::create_dir_all(&work_dir)?;
    reject_unowned_destination(&journal_path, &output)?;
    let initial_journal = new_journal(
        operation_id,
        ArtifactKind::Vault,
        source.clone(),
        source_version,
        CURRENT_VAULT_STRUCTURE_VERSION,
        fingerprint,
        vec![artifact.clone(), upgraded.clone(), output.clone()],
    )?;
    let mut journal = load_or_create_journal(&journal_path, &source_password, initial_journal)?;
    validate_resume(
        &journal,
        ArtifactKind::Vault,
        &source,
        source_version,
        fingerprint,
        &output,
    )?;
    ensure_artifact_key(&mut journal)?;
    let migration_key = journal.artifact_key.try_clone()?;

    if journal.current_stage == MigrationStage::Export {
        let complete = artifact.exists()
            && if preserve_records {
                verify_archive_artifact(&artifact, &migration_key).is_ok()
            } else {
                verify_vault_artifact(&artifact, &migration_key).is_ok()
            };
        if !complete {
            remove_partial(&artifact)?;
            if preserve_records && source_container_version == u32::from(LOCKBOX_FORMAT_VERSION) {
                let lockbox = Lockbox::open(
                    &source.join("local-vault.lbox"),
                    LockboxOpen::Password(&source_password),
                )?;
                export_archive(&lockbox, &artifact, &migration_key, operation_id)?;
            } else {
                let exporter = resolve_exporter(
                    ArtifactKind::Vault,
                    if experimental_export {
                        4
                    } else {
                        source_container_version
                    },
                    exporter,
                )?;
                let artifact_password = SecretString::from_secure_vec(migration_key.try_clone()?);
                run_historical_vault_exporter(
                    &exporter,
                    &source,
                    &artifact,
                    &source_password,
                    &artifact_password,
                    preserve_records,
                )?;
                journal.exporter_version = Some(exporter.display().to_string());
            }
        }
        journal.current_stage = MigrationStage::Upgrade;
        save_journal(&mut journal, &journal_path, &source_password)?;
    }
    if journal.current_stage == MigrationStage::Upgrade {
        let complete = upgraded.exists()
            && if preserve_records {
                verify_archive_artifact(&upgraded, &migration_key).is_ok()
            } else {
                verify_vault_artifact(&upgraded, &migration_key).is_ok()
            };
        if !complete {
            remove_partial(&upgraded)?;
            if preserve_records {
                upgrade_archive_artifact(&artifact, &upgraded, &migration_key)?;
            } else {
                upgrade_vault_artifact(&artifact, &upgraded, &migration_key)?;
            }
        }
        journal.current_stage = MigrationStage::Import;
        save_journal(&mut journal, &journal_path, &source_password)?;
    }
    if journal.current_stage == MigrationStage::Import {
        let complete = output.exists()
            && probe_archive_path(&output.join("local-vault.lbox"))
                .is_ok_and(|version| version == u32::from(LOCKBOX_FORMAT_VERSION))
            && VaultDirectory::open_or_create(&output, &source_password)
                .and_then(|vault| vault.structure_version())
                .is_ok_and(|version| version == CURRENT_VAULT_STRUCTURE_VERSION);
        if !complete {
            remove_partial(&output)?;
            if preserve_records {
                let signer = revault_migration::vault_container_signing_key_from_artifact(
                    &upgraded,
                    &migration_key,
                )?;
                fs::create_dir_all(&output)?;
                fs::set_permissions(&output, fs::metadata(&source)?.permissions())?;
                import_archive(
                    &upgraded,
                    &migration_key,
                    &output.join("local-vault.lbox"),
                    &signer,
                )?;
                fs::set_permissions(
                    output.join("local-vault.lbox"),
                    fs::metadata(source.join("local-vault.lbox"))?.permissions(),
                )?;
            } else {
                import_vault(&upgraded, &migration_key, &output, &source_password)?;
            }
        }
        journal.current_stage = MigrationStage::Validate;
        save_journal(&mut journal, &journal_path, &source_password)?;
    }
    if journal.current_stage == MigrationStage::Validate {
        if probe_archive_path(&output.join("local-vault.lbox"))?
            != u32::from(LOCKBOX_FORMAT_VERSION)
        {
            return Err(cli_error("migrated vault container validation failed"));
        }
        if preserve_records {
            revault_migration::verify_imported_archive(
                &upgraded,
                &migration_key,
                &output.join("local-vault.lbox"),
            )?;
        }
        let imported = VaultDirectory::open_or_create(&output, &source_password)?;
        if imported.structure_version()? != CURRENT_VAULT_STRUCTURE_VERSION {
            return Err(cli_error("migrated vault validation failed"));
        }
        drop(imported);
        journal.current_stage = if replace {
            MigrationStage::Replace
        } else {
            MigrationStage::Complete
        };
        save_journal(&mut journal, &journal_path, &source_password)?;
    }
    if replace {
        if journal.current_stage != MigrationStage::Replace {
            return Err(cli_error(
                "vault migration journal has an invalid replacement stage",
            ));
        }
        let backup =
            versioned_backup_path(&source, source_version, CURRENT_VAULT_STRUCTURE_VERSION);
        if backup.exists() {
            return Err(cli_error(format!(
                "migration backup already exists: {}",
                backup.display()
            )));
        }
        fs::rename(&source, &backup)?;
        if let Err(err) = fs::rename(&output, &source) {
            let _ = fs::rename(&backup, &source);
            return Err(err.into());
        }
        println!("Vault migrated to container format {LOCKBOX_FORMAT_VERSION} and structure version {CURRENT_VAULT_STRUCTURE_VERSION}.");
        println!("Previous vault retained at {}", backup.display());
        remember_default_vault_password_with_warning(
            &source_password,
            "the vault migrated successfully",
        );
    } else {
        println!("Vault migrated to {}", output.display());
    }
    cleanup_work_dir(&work_dir);
    Ok(())
}

fn migrate_archive_direct(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let source = PathBuf::from(archive_source(matches)?);
    let replace = matches.get_flag("replace");
    let requested_output = matches
        .get_one::<String>("output")
        .map(|value| super::aliases::resolve(value).map(|(path, _)| PathBuf::from(path)))
        .transpose()?;
    let exporter = matches.get_one::<String>("exporter").map(PathBuf::from);
    migrate_archive(source, replace, requested_output, exporter, access)
}

fn migrate_archive(
    source: PathBuf,
    replace: bool,
    requested_output: Option<PathBuf>,
    exporter: Option<PathBuf>,
    access: &Access,
) -> CliResult<()> {
    require_destination(replace, requested_output.as_deref(), "lockbox")?;
    let vault_root = default_vault_dir()?;
    if !vault_root.join("local-vault.lbox").exists() {
        return Err(cli_error(
            "lockbox migration requires a current vault; run `lockbox vault init` first",
        ));
    }
    let vault_password = vault_password_without_open()?;
    probe_archive_path(&vault_root.join("local-vault.lbox"))?;
    let vault_version = VaultDirectory::probe_structure_version(&vault_root, &vault_password)?;
    if vault_version < CURRENT_VAULT_STRUCTURE_VERSION {
        return Err(cli_error(format!(
            "lockbox migration requires the vault to be migrated first; vault format version {vault_version} is older than the current version {CURRENT_VAULT_STRUCTURE_VERSION}. Run `lockbox doctor migrate vault --replace` or migrate it with `--output <directory>`"
        )));
    }
    if vault_version > CURRENT_VAULT_STRUCTURE_VERSION {
        return Err(cli_error(format!(
            "lockbox migration cannot run with a newer vault format version {vault_version}; this build supports version {CURRENT_VAULT_STRUCTURE_VERSION}. Install a newer reVault release"
        )));
    }
    if !source.exists()
        && recover_interrupted_replacement(&source, ArtifactKind::Archive, &vault_password)?
    {
        println!("Completed the interrupted lockbox replacement.");
        return Ok(());
    }
    let source_version = probe_archive_path(&source)?;
    if replace && source_version == u32::from(LOCKBOX_FORMAT_VERSION) && exporter.is_none() {
        // A format number alone does not identify the current authenticated graph.
        let _verified = open_existing_read_only(&source.to_string_lossy(), access)?;
        println!(
            "Lockbox already uses format version {LOCKBOX_FORMAT_VERSION}. No migration needed."
        );
        return Ok(());
    }
    let fingerprint = fingerprint_path(&source)?;
    let operation_id = deterministic_operation_id(
        ArtifactKind::Archive,
        &source,
        requested_output.as_deref(),
        fingerprint,
    );
    let output =
        requested_output.unwrap_or_else(|| temporary_archive_destination(&source, operation_id));
    let work_dir = migration_work_dir(&source, operation_id)?;
    fs::create_dir_all(&work_dir)?;
    let artifact = work_dir.join("archive.migration");
    let upgraded = work_dir.join("archive.latest.migration");
    let journal_path = work_dir.join("archive.migration-state");
    reject_unowned_destination(&journal_path, &output)?;
    let initial_journal = new_journal(
        operation_id,
        ArtifactKind::Archive,
        source.clone(),
        source_version,
        u32::from(LOCKBOX_FORMAT_VERSION),
        fingerprint,
        vec![artifact.clone(), upgraded.clone(), output.clone()],
    )?;
    let mut journal = load_or_create_journal(&journal_path, &vault_password, initial_journal)?;
    validate_resume(
        &journal,
        ArtifactKind::Archive,
        &source,
        source_version,
        fingerprint,
        &output,
    )?;
    ensure_artifact_key(&mut journal)?;
    let migration_key = journal.artifact_key.try_clone()?;
    if journal.current_stage == MigrationStage::Export {
        let complete =
            artifact.exists() && verify_archive_artifact(&artifact, &migration_key).is_ok();
        if !complete {
            remove_partial(&artifact)?;
            if source_version == u32::from(LOCKBOX_FORMAT_VERSION) && exporter.is_none() {
                let lockbox =
                    super::context::open_existing_for_migration(&source.to_string_lossy(), access)
                        .map_err(|error| archive_source_error(&source, error))?;
                export_archive(&lockbox, &artifact, &migration_key, operation_id)
                    .map_err(|error| archive_source_error(&source, Box::new(error)))?;
            } else {
                let exporter = resolve_exporter(ArtifactKind::Archive, source_version, exporter)?;
                let artifact_password = SecretString::from_secure_vec(migration_key.try_clone()?);
                let vault = open_default_vault_with_password(&vault_password)?;
                run_historical_archive_exporter(
                    &exporter,
                    &source,
                    &artifact,
                    &vault,
                    &vault_password,
                    &artifact_password,
                )?;
                journal.exporter_version = Some(exporter.display().to_string());
            }
        }
        journal.current_stage = MigrationStage::Upgrade;
        save_journal(&mut journal, &journal_path, &vault_password)?;
    }
    if journal.current_stage == MigrationStage::Upgrade {
        let complete =
            upgraded.exists() && verify_archive_artifact(&upgraded, &migration_key).is_ok();
        if !complete {
            remove_partial(&upgraded)?;
            upgrade_archive_artifact(&artifact, &upgraded, &migration_key)?;
        }
        journal.current_stage = MigrationStage::Import;
        save_journal(&mut journal, &journal_path, &vault_password)?;
    }
    if journal.current_stage == MigrationStage::Import {
        let complete = output.exists()
            && revault_migration::verify_imported_archive(&upgraded, &migration_key, &output)
                .is_ok();
        if !complete {
            remove_partial(&output)?;
            let vault = open_default_vault_with_password(&vault_password)?;
            let signing =
                match revault_migration::archive_owner_fingerprint(&upgraded, &migration_key)? {
                    Some(fingerprint) => {
                        vault.find_owner_signing_key(&fingerprint)?.ok_or_else(|| {
                            cli_error("the established archive owner's signing key is unavailable")
                        })?
                    }
                    None => vault.load_owner_signing_key(VaultDirectory::DEFAULT_KEY_NAME)?,
                };
            import_archive(&upgraded, &migration_key, &output, &signing)?;
        }
        journal.current_stage = MigrationStage::Validate;
        save_journal(&mut journal, &journal_path, &vault_password)?;
    }
    if journal.current_stage == MigrationStage::Validate {
        revault_migration::verify_imported_archive(&upgraded, &migration_key, &output)?;
        journal.current_stage = if replace {
            MigrationStage::Replace
        } else {
            MigrationStage::Complete
        };
        save_journal(&mut journal, &journal_path, &vault_password)?;
    }
    if replace {
        if let Some(expected) = super::COMMAND_LOCKBOX_ID.with(|id| *id.borrow()) {
            if Lockbox::inspect_file(&output)?.lockbox_id != expected {
                return Err(cli_error(
                    "known Lockbox identity changed; migration source was not replaced",
                ));
            }
        }
        let backup =
            versioned_backup_path(&source, source_version, u32::from(LOCKBOX_FORMAT_VERSION));
        if backup.exists() {
            return Err(cli_error(format!(
                "migration backup already exists: {}",
                backup.display()
            )));
        }
        fs::rename(&source, &backup)?;
        if let Err(err) = fs::rename(&output, &source) {
            let _ = fs::rename(&backup, &source);
            return Err(err.into());
        }
        println!("Lockbox migrated to format version {LOCKBOX_FORMAT_VERSION}.");
        println!("Previous lockbox retained at {}", backup.display());
    } else {
        println!("Lockbox migrated to {}", output.display());
    }
    cleanup_work_dir(&work_dir);
    Ok(())
}

fn resolve_exporter(
    kind: ArtifactKind,
    source_version: u32,
    explicit: Option<PathBuf>,
) -> CliResult<PathBuf> {
    if source_version == 4 {
        let path = explicit.ok_or_else(|| cli_error(
            "experimental format-4 migration requires an isolated checkpoint exporter; build tools/build_experimental_v4_exporter.dart and pass --exporter <path>"
        ))?;
        validate_exporter_capabilities(&path, experimental_v4_exporter(kind)?)?;
        return Ok(path);
    }
    let release = exporter_release(kind, source_version).ok_or_else(|| {
        cli_error(format!(
            "no crates.io exporter is registered for {kind:?} format version {source_version}"
        ))
    })?;
    if let Some(path) = explicit {
        validate_exporter_capabilities(&path, release)?;
        return Ok(path);
    }
    let binary_name = if cfg!(windows) {
        format!("{}.exe", release.binary)
    } else {
        release.binary.to_string()
    };
    if env::var_os("LOCKBOX_TEST_IGNORE_INSTALLED_EXPORTER").is_none() {
        if let Some(binary) = installed_exporter(&binary_name) {
            // A previous release may have installed a same-named executable
            // with an older protocol. Only an explicit --exporter is binding;
            // automatic discovery must fall through to the registered package.
            if validate_exporter_capabilities(&binary, release).is_ok() {
                return Ok(binary);
            }
        }
    }
    let root = exporter_cache_root()?.join(format!("{}-{}", release.package, release.version));
    let binary = root.join("bin").join(binary_name);
    if binary.exists() {
        validate_exporter_capabilities(&binary, release)?;
        return Ok(binary);
    }
    eprintln!(
        "Installing historical reVault exporter {} {} from crates.io...",
        release.package, release.version
    );
    let install = Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .args([
            "install",
            "--quiet",
            release.package,
            "--version",
            release.version,
            "--bin",
            release.binary,
            "--locked",
            "--root",
        ])
        .arg(&root)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()?;
    if !install.status.success() || !binary.exists() {
        let detail = String::from_utf8_lossy(&install.stderr);
        let detail = detail.trim();
        let detail = if detail.is_empty() {
            String::new()
        } else {
            format!("\nCargo reported:\n{detail}")
        };
        return Err(cli_error(format!(
            "failed to install {} {} from crates.io; install it manually and pass --exporter <path>{detail}",
            release.package, release.version,
        )));
    }
    eprintln!(
        "Historical reVault exporter {} {} is ready.",
        release.package, release.version
    );
    validate_exporter_capabilities(&binary, release)?;
    Ok(binary)
}

fn archive_source_error(
    source: &Path,
    error: Box<dyn std::error::Error>,
) -> Box<dyn std::error::Error> {
    let corrupt = matches!(
        error.downcast_ref::<revault_lockbox_api::Error>(),
        Some(
            revault_lockbox_api::Error::CorruptHeader
                | revault_lockbox_api::Error::CorruptRecord
                | revault_lockbox_api::Error::Truncated
        )
    ) || matches!(
        error.downcast_ref::<revault_migration::MigrationError>(),
        Some(revault_migration::MigrationError::SourceCorrupt(_))
    );
    if !corrupt {
        return error;
    }
    let source_arg = recovery_command_path(source);
    super::context::cli_diagnostic(
        super::error_output::ExitCode::CorruptData,
        "Migration could not read damaged source Lockbox data; the source was not replaced",
        vec![("Lockbox".into(), source.display().to_string()), ("Cause".into(), error.to_string())],
        format!("Inspect recoverable content with this reVault build:\n  lbx {source_arg} doctor recover --dry-run\nRecover to a new Lockbox, retaining incomplete fragments separately when needed:\n  lbx {source_arg} doctor recover\nRecovery reports the recovered Lockbox and any partial-file directory with a manifest of missing ranges. Verify recovered content and retain the original. Use --output to choose the recovered Lockbox path."),
    )
}

fn recovery_command_path(path: &Path) -> String {
    let value = path.to_string_lossy();
    #[cfg(windows)]
    {
        format!("'{}'", value.replace('\'', "''"))
    }
    #[cfg(not(windows))]
    {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn installed_exporter(binary_name: &str) -> Option<PathBuf> {
    let binary = env::current_exe().ok()?.parent()?.join(binary_name);
    binary.is_file().then_some(binary)
}

fn experimental_v4_exporter(kind: ArtifactKind) -> CliResult<ExporterRelease> {
    Ok(ExporterRelease {
        package: "revault_experimental_v4_exporter",
        version: "0.0.0",
        binary: match kind {
            ArtifactKind::Archive => "revault-migrate-archive-experimental-v4",
            ArtifactKind::Vault => "revault-migrate-vault-experimental-v4",
            ArtifactKind::Journal => {
                return Err(cli_error("journals are not historical source archives"))
            }
        },
        protocol: 3,
        artifact: if kind == ArtifactKind::Archive {
            "archive"
        } else {
            "vault"
        },
        native_version: 4,
        migration_schema: 3,
    })
}

fn exporter_release(kind: ArtifactKind, source_version: u32) -> Option<ExporterRelease> {
    let artifact = match kind {
        ArtifactKind::Archive => "archive",
        ArtifactKind::Vault => "vault",
        ArtifactKind::Journal => return None,
    };
    super::migration_exporters::EXPORTERS
        .iter()
        .copied()
        .find(|release| release.artifact == artifact && release.native_version == source_version)
}

fn validate_exporter_capabilities(path: &Path, release: ExporterRelease) -> CliResult<()> {
    let output = Command::new(path).arg("capabilities").output()?;
    if !output.status.success() {
        return Err(cli_error("historical exporter capability check failed"));
    }
    if !capabilities_match(&output.stdout, release) {
        return Err(cli_error(
            "historical exporter capabilities do not match the registered format",
        ));
    }
    Ok(())
}

fn run_historical_vault_exporter(
    exporter: &Path,
    source: &Path,
    output: &Path,
    vault_password: &SecretString,
    artifact_password: &SecretString,
    preserve_records: bool,
) -> CliResult<()> {
    let mut command = Command::new(exporter);
    command
        .args(["migrate", "vault", "export", "--output"])
        .arg(output)
        .arg("--source")
        .arg(source)
        .stdin(Stdio::piped());
    if preserve_records {
        command.arg("--preserve-records");
    }
    let mut child = command.spawn()?;
    write_subprocess_secrets(&mut child, &[vault_password, artifact_password], &[])?;
    let status = child.wait()?;
    if !status.success() {
        return Err(cli_error(format!(
            "historical vault exporter exited with {status}"
        )));
    }
    Ok(())
}

fn run_historical_archive_exporter(
    exporter: &Path,
    source: &Path,
    output: &Path,
    vault: &VaultDirectory,
    vault_password: &SecretString,
    artifact_password: &SecretString,
) -> CliResult<()> {
    let contact_keys = migration_contact_keys(vault)?;
    let mut child = Command::new(exporter)
        .args(["migrate", "archive", "export"])
        .arg(source)
        .arg("--output")
        .arg(output)
        .stdin(Stdio::piped())
        .spawn()?;
    write_subprocess_secrets(
        &mut child,
        &[vault_password, artifact_password],
        &contact_keys,
    )?;
    let status = child.wait()?;
    if !status.success() {
        if status.code() == Some(42) {
            return Err(archive_source_error(source, Box::new(revault_migration::MigrationError::SourceCorrupt("historical exporter could unlock the archive but its reachable contents failed authentication or integrity validation".into()))));
        }
        return Err(cli_error(format!(
            "historical lockbox exporter exited with {status}"
        )));
    }
    Ok(())
}

fn write_subprocess_secrets(
    child: &mut std::process::Child,
    secrets: &[&SecretString],
    binary_secrets: &[SecretVec],
) -> CliResult<()> {
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| cli_error("exporter stdin unavailable"))?;
    stdin.write_all(b"LBXMIPC1")?;
    stdin.write_all(
        &u32::try_from(secrets.len() + binary_secrets.len())
            .map_err(|_| cli_error("too many migration secrets"))?
            .to_le_bytes(),
    )?;
    for secret in secrets {
        secret.with_bytes(|bytes| -> std::io::Result<()> {
            let len = u32::try_from(bytes.len()).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "migration secret too long",
                )
            })?;
            stdin.write_all(&len.to_le_bytes())?;
            stdin.write_all(bytes)
        })??;
    }
    for secret in binary_secrets {
        secret.with_bytes(|bytes| -> std::io::Result<()> {
            let len = u32::try_from(bytes.len()).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "migration secret too long",
                )
            })?;
            stdin.write_all(&len.to_le_bytes())?;
            stdin.write_all(bytes)
        })??;
    }
    drop(stdin);
    Ok(())
}

fn migration_contact_keys(vault: &VaultDirectory) -> CliResult<Vec<SecretVec>> {
    let mut records = Vec::new();
    for profile in vault.list_private_keys()? {
        let active = vault.load_private_key(&profile)?;
        records.push(active.private_key_record()?);
        if let Ok(history) = vault.list_profile_generations(&profile) {
            for generation in history.generations {
                if generation.index == history.active_generation {
                    continue;
                }
                let key = vault.load_private_key_generation(&profile, generation.index)?;
                records.push(key.private_key_record()?);
            }
        }
    }
    Ok(records)
}

pub(crate) fn recover_historical_archive(
    source: &Path,
    output: Option<&Path>,
    overwrite: bool,
    dry_run: bool,
    format: &str,
    quiet: bool,
    access: &Access,
    directory: Option<&Path>,
    telemetry: &super::recovery_telemetry::RecoveryTelemetry,
) -> CliResult<()> {
    telemetry.event("selecting_exporter", None);
    let exporter = resolve_exporter(ArtifactKind::Archive, 2, None)?;
    if directory.is_some() {
        let capabilities = Command::new(&exporter).arg("capabilities").output()?;
        let capabilities: serde_json::Value = serde_json::from_slice(&capabilities.stdout)?;
        if capabilities["recovery_directory_protocol"] != 1 {
            return Err(cli_error("the installed format-2 exporter does not support recovery directories; use an exporter build with recovery-directory protocol 1"));
        }
    }
    let password = vault_password_without_open()?;
    let vault = open_default_vault_with_password(&password)?;
    let artifact_key = SecretString::from_secure_vec(generated_migration_key()?);
    let mut keys = match access {
        Access::ContentKey(key) => vec![key.try_clone()?],
        Access::CacheOnly => Vec::new(),
        Access::PromptPassword => {
            return Err(cli_error(
                "historical recovery requires an opening credential",
            ))
        }
    };
    let has_raw_key = !keys.is_empty();
    let mut signing_count = 0;
    if !dry_run {
        for profile in vault.list_private_keys()? {
            for generation in vault.list_profile_generations(&profile)?.generations {
                if let Ok(key) = vault.load_owner_signing_key_generation(&profile, generation.index)
                {
                    keys.push(key.private_key_record()?);
                    signing_count += 1;
                }
            }
        }
    }
    keys.extend(migration_contact_keys(&vault)?);
    let temporary = tempfile::tempdir()?;
    let artifact = temporary.path().join("recovery.migration");
    let partial_artifact = temporary.path().join("partial.migration");
    let mut command = Command::new(exporter);
    command
        .args(["migrate", "archive", "recover"])
        .arg(source)
        .arg("--output")
        .arg(&artifact)
        .arg("--partial-output")
        .arg(&partial_artifact)
        .arg("--signing-key-count")
        .arg(signing_count.to_string())
        .arg("--raw-key")
        .arg(if has_raw_key { "true" } else { "false" })
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if dry_run {
        command.arg("--dry-run");
    }
    let mut child = command.spawn()?;
    write_subprocess_secrets(&mut child, &[&password, &artifact_key], &keys)?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| cli_error("missing exporter error stream"))?;
    let result = std::thread::scope(|scope| {
        let reader = scope.spawn(move || -> std::io::Result<()> {
            use std::io::BufRead;
            for line in std::io::BufReader::new(stderr).lines() {
                let line = line?;
                if !telemetry.helper_event(&line) {
                    eprintln!("{line}");
                }
            }
            Ok(())
        });
        let result = child.wait_with_output();
        let _ = reader.join();
        result
    })?;
    if !result.status.success() {
        return Err(cli_error(format!(
            "historical recovery exporter exited with {}; the source was not changed",
            result.status
        )));
    }
    let mut report: serde_json::Value = serde_json::from_slice(&result.stdout)?;
    telemetry.counts(&report["file_recovery_plan"]);
    let mut partial_writer = None;
    if let Some(directory) = directory {
        if !dry_run && super::recovery_directory::has_partial_files(&report["file_recovery_plan"]) {
            let mut reader = revault_migration::ArtifactReader::new_with_passphrase(
                File::open(&partial_artifact)?,
                &artifact_key,
            )?;
            if reader.header().artifact_kind != ArtifactKind::Journal
                || reader.header().source_native_version != 2
            {
                return Err(cli_error("unexpected recovery-range artifact"));
            }
            let mut writer = super::recovery_directory::DirectoryWriter::new(directory)?;
            let mut manifest = None;
            while let Some((kind, bytes)) = reader.next_frame()? {
                let bytes = zeroize::Zeroizing::new(bytes);
                if manifest.is_some() {
                    return Err(cli_error("unexpected data after recovery manifest"));
                }
                match kind {
                    revault_migration::RAW_FRAME_TYPE if bytes.len() > 16 => {
                        let id = u64::from_le_bytes(bytes[..8].try_into().unwrap());
                        let offset = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
                        writer.range(id, offset, &bytes[16..])?;
                    }
                    revault_migration::JSON_FRAME_TYPE => {
                        manifest = Some(serde_json::from_slice::<serde_json::Value>(&bytes)?)
                    }
                    _ => return Err(cli_error("invalid recovery-range artifact frame")),
                }
            }
            if manifest.as_ref() != Some(&report["file_recovery_plan"]) {
                return Err(cli_error(
                    "recovery artifact manifest disagrees with the report",
                ));
            }
            partial_writer = Some(writer);
        }
        super::recovery_directory::set_archive_destination(
            &mut report["file_recovery_plan"],
            output.ok_or_else(|| cli_error("missing recovery archive destination"))?,
            directory,
            dry_run,
        )?;
    }
    if !dry_run {
        telemetry.event("importing_archive", None);
        verify_archive_artifact(&artifact, &artifact_key)?;
        let fingerprint = revault_migration::archive_owner_fingerprint(&artifact, &artifact_key)?
            .ok_or_else(|| {
            cli_error("historical recovery artifact has no established owner")
        })?;
        let signer = vault.find_owner_signing_key(&fingerprint)?.ok_or_else(|| {
            cli_error("the established archive owner's signing key is unavailable")
        })?;
        let destination =
            output.ok_or_else(|| cli_error("historical recovery requires a separate --output"))?;
        let parent = destination
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let staged = tempfile::tempdir_in(parent)?;
        let staged_path = staged.path().join("recovered.lbox");
        import_archive(&artifact, &artifact_key, &staged_path, &signer)?;
        telemetry.event("verifying_archive", None);
        revault_migration::verify_imported_archive(&artifact, &artifact_key, &staged_path)?;
        if let Some(expected) = super::COMMAND_LOCKBOX_ID.with(|id| *id.borrow()) {
            if Lockbox::inspect_file(&staged_path)?.lockbox_id != expected {
                return Err(cli_error(
                    "historical recovery source identity changed; output was not published",
                ));
            }
        }
        fs::File::open(&staged_path)?.sync_all()?;
        telemetry.event("publishing_outputs", None);
        if let Some(writer) = partial_writer {
            writer.finish(report["file_recovery_plan"].clone())?;
        }
        let publication = if overwrite {
            fs::rename(&staged_path, destination)
        } else {
            fs::hard_link(&staged_path, destination)
        };
        publication.map_err(|error| cli_error(format!("could not publish recovered archive: {error}; any completed partial-file recovery remains at {}", directory.map_or_else(|| "(none)".to_owned(), |path| path.display().to_string()))))?;
        #[cfg(unix)]
        fs::File::open(parent)?.sync_all()?;
    }
    telemetry.event("complete", None);
    let output_format = match format {
        "json" => super::output::OutputFormat::Json,
        "tsv" => super::output::OutputFormat::Tsv,
        _ => super::output::OutputFormat::Table,
    };
    if !quiet || dry_run {
        super::output::print_recovery_records(
            historical_recovery_report_rows(&report),
            output_format,
        )?;
    }
    if report.get("file_recovery_plan").is_some() {
        super::recovery_directory::print_plan(&report["file_recovery_plan"], output_format)?;
    }
    Ok(())
}

fn historical_recovery_report_rows(report: &serde_json::Value) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = ["intact_files", "partial_files", "corrupt_records"]
        .into_iter()
        .map(|name| {
            vec![
                if name == "partial_files" {
                    "source_partial_files".to_owned()
                } else {
                    name.to_owned()
                },
                report[name].to_string(),
            ]
        })
        .collect();
    let details = report
        .get("partial_file_details")
        .and_then(serde_json::Value::as_array);
    if let Some(toc_recovered) = report
        .get("toc_recovered")
        .and_then(serde_json::Value::as_bool)
    {
        rows.push(vec!["toc_recovered".to_owned(), toc_recovered.to_string()]);
    }
    rows.push(vec![
        "partial_file_details".to_owned(),
        details.map_or_else(
            || "[]".to_owned(),
            |details| serde_json::Value::Array(details.clone()).to_string(),
        ),
    ]);
    let unidentified = report["partial_files"]
        .as_u64()
        .unwrap_or(0)
        .saturating_sub(details.map_or(0, |details| details.len() as u64));
    rows.push(vec![
        "unidentified_partial_files".to_owned(),
        unidentified.to_string(),
    ]);
    rows
}

fn migration_password() -> CliResult<SecretString> {
    if let Some(value) = SecretString::try_from_env("LOCKBOX_MIGRATION_PASSWORD")? {
        return Ok(value);
    }
    prompt_secret("Migration artifact pass phrase: ").map_err(Into::into)
}

fn default_signing_key() -> CliResult<OwnerSigningKeyPair> {
    Ok(default_vault()?.load_owner_signing_key(VaultDirectory::DEFAULT_KEY_NAME)?)
}

fn random_id() -> CliResult<[u8; 16]> {
    let mut id = [0u8; 16];
    getrandom::fill(&mut id)?;
    Ok(id)
}

fn generated_migration_key() -> CliResult<SecretVec> {
    let mut key = SecretVec::new();
    key.resize_zeroed(32)?;
    key.with_mut_bytes(getrandom::fill)??;
    Ok(key)
}

fn probe_archive_path(path: &Path) -> CliResult<u32> {
    let mut header = Vec::new();
    File::open(path)?.take(384).read_to_end(&mut header)?;
    let version = u32::from(probe_lockbox_format_version(&header)?);
    if version > u32::from(LOCKBOX_FORMAT_VERSION) {
        return Err(cli_error(format!("archive format {version} is newer than this build supports; use a compatible newer build. Automatic downgrade is not supported")));
    }
    Ok(version)
}

fn probe_vault_source_version(root: &Path, password: &SecretString) -> CliResult<u32> {
    match VaultDirectory::probe_structure_version(root, password) {
        Ok(version) => Ok(version),
        Err(revault_lockbox_api::Error::UnsupportedFormatVersion {
            artifact: revault_lockbox_api::ArtifactKind::Lockbox,
            found: version @ 1..=3,
            ..
        }) => Ok(version),
        Err(error) => Err(error.into()),
    }
}

fn deterministic_operation_id(
    kind: ArtifactKind,
    source: &Path,
    output: Option<&Path>,
    fingerprint: [u8; 32],
) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(match kind {
        ArtifactKind::Vault => b"vault".as_slice(),
        ArtifactKind::Archive => b"archive".as_slice(),
        ArtifactKind::Journal => b"journal".as_slice(),
    });
    digest.update(source.as_os_str().as_encoded_bytes());
    if let Some(output) = output {
        digest.update([0]);
        digest.update(output.as_os_str().as_encoded_bytes());
    }
    digest.update(fingerprint);
    let value = digest.finalize();
    let mut id = [0u8; 16];
    id.copy_from_slice(&value[..16]);
    id
}

fn load_or_create_journal<P: MigrationPassphrase + ?Sized>(
    path: &Path,
    passphrase: &P,
    initial: MigrationJournal,
) -> CliResult<MigrationJournal> {
    if path.exists() {
        let saved = MigrationJournal::load(path, passphrase)?;
        if !restart_obsolete_migration(path, &saved, &initial)? {
            return Ok(saved);
        }
    }
    initial.save(path, passphrase)?;
    Ok(initial)
}

/// Retain an obsolete attempt intact before exporting the original source again.
/// Never reuse its artifacts or key: their schema and destination format belong
/// to the release that created the journal.
fn restart_obsolete_migration(
    path: &Path,
    saved: &MigrationJournal,
    initial: &MigrationJournal,
) -> CliResult<bool> {
    if saved.artifact_kind != initial.artifact_kind
        || saved.source_path != initial.source_path
        || saved.source_format_version != initial.source_format_version
        || saved.source_fingerprint != initial.source_fingerprint
        || saved.temporary_paths != initial.temporary_paths
        || !initial.source_path.exists()
    {
        return Ok(false);
    }
    // Future journals and containers must still fail without changing anything.
    if saved.target_format_version > initial.target_format_version
        || saved.source_format_version > initial.target_format_version
    {
        return Ok(false);
    }
    let output = initial
        .temporary_paths
        .last()
        .ok_or_else(|| cli_error("migration journal has no destination"))?;
    let work = path
        .parent()
        .ok_or_else(|| cli_error("migration journal has no directory"))?;
    let retained_output = work.join("previous-output");
    if initial.source_path.starts_with(output)
        || output.starts_with(&initial.source_path)
        || work.starts_with(output)
        || output.starts_with(work)
    {
        return Err(cli_error(
            "migration destination overlaps the source or migration state",
        ));
    }
    if output.exists() {
        let resolved_output = fs::canonicalize(output)?;
        for protected in [initial.source_path.as_path(), work] {
            let protected = fs::canonicalize(protected)?;
            if protected.starts_with(&resolved_output) || resolved_output.starts_with(&protected) {
                return Err(cli_error(
                    "migration destination overlaps the source or migration state",
                ));
            }
        }
    }
    let probe_output = if output.exists() {
        output
    } else {
        &retained_output
    };
    let container = if saved.artifact_kind == ArtifactKind::Vault {
        probe_output.join("local-vault.lbox")
    } else {
        probe_output.clone()
    };
    let output_version = if container.exists() {
        let mut header = Vec::new();
        File::open(&container)?.take(384).read_to_end(&mut header)?;
        probe_lockbox_format_version(&header).ok()
    } else {
        None
    };
    if output_version.is_some_and(|version| version > LOCKBOX_FORMAT_VERSION) {
        return Ok(false);
    }
    if saved.target_format_version == initial.target_format_version
        && !output_version.is_some_and(|version| version < LOCKBOX_FORMAT_VERSION)
    {
        return Ok(false);
    }
    // The output is normally beside the work directory, including --output.
    // Moving it into that directory first makes an interrupted preservation
    // retry safe: the saved journal remains available until the final rename.
    if output.exists() {
        if retained_output.exists() {
            return Err(cli_error(
                "cannot preserve obsolete migration: previous-output already exists",
            ));
        }
        fs::rename(output, &retained_output)?;
    }
    let preserved = work.with_file_name(format!(
        ".revault-preserved-migration-{}",
        hex_id(random_id()?)
    ));
    fs::rename(work, &preserved)?;
    fs::create_dir(work)?;
    println!(
        "Preserved the previous migration at {}; restarting from the unchanged source for the current format.",
        preserved.display()
    );
    Ok(true)
}

fn validate_resume(
    journal: &MigrationJournal,
    kind: ArtifactKind,
    source: &Path,
    source_version: u32,
    fingerprint: [u8; 32],
    output: &Path,
) -> CliResult<()> {
    validate_journal_versions(journal)?;
    if let Some(output) = journal.temporary_paths.last() {
        let container = if kind == ArtifactKind::Vault {
            output.join("local-vault.lbox")
        } else {
            output.clone()
        };
        if container.exists() {
            // Probe both known header layouts before a resumed Import can
            // remove_partial, including a structure-3 Vault in a newer container.
            let mut header = Vec::new();
            File::open(&container)?.take(384).read_to_end(&mut header)?;
            if let Ok(version) = probe_lockbox_format_version(&header) {
                if version != LOCKBOX_FORMAT_VERSION {
                    let guidance = if version > LOCKBOX_FORMAT_VERSION {
                        "use a compatible newer build. Automatic downgrade is not supported"
                    } else {
                        "use the explicit migration workflow to upgrade this older archive"
                    };
                    return Err(cli_error(format!("saved migration output uses unsupported format {version}; this build requires format {LOCKBOX_FORMAT_VERSION}; {guidance}")));
                }
            }
        }
    }
    if journal.artifact_kind != kind
        || journal.source_path != source
        || journal.source_format_version != source_version
        || journal.source_fingerprint != fingerprint
    {
        return Err(cli_error(
            "the saved migration does not match the current source; the source may have changed",
        ));
    }
    if journal.current_stage == MigrationStage::Export && output.exists() {
        return Err(cli_error(format!(
            "migration destination already exists without a completed import: {}",
            output.display()
        )));
    }
    Ok(())
}

fn ensure_artifact_key(journal: &mut MigrationJournal) -> CliResult<()> {
    if !journal.artifact_key.is_empty() {
        return Ok(());
    }
    if journal.current_stage != MigrationStage::Export {
        return Err(cli_error(
            "the saved migration state predates generated migration keys; remove the incomplete migration state and retry",
        ));
    }
    journal.artifact_key = generated_migration_key()?;
    Ok(())
}

fn save_journal<P: MigrationPassphrase + ?Sized>(
    journal: &mut MigrationJournal,
    path: &Path,
    journal_password: &P,
) -> CliResult<()> {
    journal.save(path, journal_password)?;
    Ok(())
}

fn reject_unowned_destination(journal_path: &Path, output: &Path) -> CliResult<()> {
    if !journal_path.exists() && output.exists() {
        return Err(cli_error(format!(
            "migration destination already exists: {}",
            output.display()
        )));
    }
    Ok(())
}

fn validate_journal_versions(journal: &MigrationJournal) -> revault_migration::Result<()> {
    let current = match journal.artifact_kind {
        ArtifactKind::Vault => CURRENT_VAULT_STRUCTURE_VERSION,
        ArtifactKind::Archive => u32::from(LOCKBOX_FORMAT_VERSION),
        ArtifactKind::Journal => {
            return Err(revault_migration::MigrationError::InvalidHeader(
                "unexpected journal artifact kind".into(),
            ))
        }
    };
    if journal.source_format_version > current || journal.target_format_version != current {
        let guidance = if journal.source_format_version > current
            || journal.target_format_version > current
        {
            "use a compatible newer build. Automatic downgrade is not supported"
        } else {
            "resume with the original compatible build, then explicitly migrate to the current format"
        };
        return Err(revault_migration::MigrationError::InvalidHeader(format!(
            "saved migration targets format {}; this build requires format {current}; {guidance}",
            journal.target_format_version,
        )));
    }
    Ok(())
}

fn recover_interrupted_replacement<P: MigrationPassphrase + ?Sized>(
    source: &Path,
    kind: ArtifactKind,
    passphrase: &P,
) -> revault_migration::Result<bool> {
    let Some(parent) = source.parent() else {
        return Ok(false);
    };
    let entries = match fs::read_dir(parent) {
        Ok(entries) => entries,
        Err(_) => return Ok(false),
    };
    for entry in entries.flatten() {
        let work_dir = entry.path();
        if !work_dir
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with(".revault-migration-"))
        {
            continue;
        }
        let journal_path = work_dir.join(match kind {
            ArtifactKind::Vault => "vault.migration-state",
            ArtifactKind::Archive => "archive.migration-state",
            ArtifactKind::Journal => continue,
        });
        let Ok(journal) = MigrationJournal::load(&journal_path, passphrase) else {
            continue;
        };
        if journal.artifact_kind != kind
            || !same_replacement_source(&journal.source_path, source)
            || journal.current_stage != MigrationStage::Replace
        {
            continue;
        }
        validate_journal_versions(&journal)?;
        let Some(output) = journal.temporary_paths.last() else {
            continue;
        };
        let backup = versioned_backup_path(
            source,
            journal.source_format_version,
            journal.target_format_version,
        );
        if source.exists() || !backup.exists() || !output.exists() {
            continue;
        }
        let container = if kind == ArtifactKind::Vault {
            output.join("local-vault.lbox")
        } else {
            output.clone()
        };
        let output_version = probe_archive_path(&container)
            .map_err(|error| revault_migration::MigrationError::InvalidHeader(error.to_string()))?;
        if output_version != u32::from(LOCKBOX_FORMAT_VERSION) {
            return Err(revault_migration::MigrationError::InvalidHeader(format!(
                "saved migration output uses older format {output_version}; explicitly migrate it before completing replacement"
            )));
        }
        fs::rename(output, source).map_err(|err| {
            revault_migration::MigrationError::Io(format!(
                "failed to finish interrupted replacement: {err}"
            ))
        })?;
        cleanup_work_dir(&work_dir);
        return Ok(true);
    }
    Ok(false)
}

fn same_replacement_source(saved: &Path, requested: &Path) -> bool {
    if saved == requested {
        return true;
    }
    // The source itself is absent between replacement renames. Canonicalize
    // its existing parent so a bulk run can resume a prior relative-path run.
    let normalize = |path: &Path| -> Option<PathBuf> {
        let parent = path.parent()?;
        let parent = if parent.as_os_str().is_empty() {
            Path::new(".")
        } else {
            parent
        };
        Some(fs::canonicalize(parent).ok()?.join(path.file_name()?))
    };
    normalize(saved)
        .is_some_and(|saved| normalize(requested).is_some_and(|requested| saved == requested))
}

fn remove_partial(path: &Path) -> CliResult<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn fingerprint_path(path: &Path) -> CliResult<[u8; 32]> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    // Windows CLI stacks cannot accommodate a 1 MiB local array.
    let mut buffer = vec![0u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(digest.finalize().into())
}

fn new_journal(
    operation_id: [u8; 16],
    artifact_kind: ArtifactKind,
    source_path: PathBuf,
    source_format_version: u32,
    target_format_version: u32,
    source_fingerprint: [u8; 32],
    temporary_paths: Vec<PathBuf>,
) -> CliResult<MigrationJournal> {
    let journal = MigrationJournal {
        operation_id,
        artifact_kind,
        source_path,
        source_format_version,
        source_fingerprint,
        target_format_version,
        current_stage: MigrationStage::Export,
        temporary_paths,
        exporter_version: None,
        artifact_key: generated_migration_key()?,
    };
    Ok(journal)
}

fn require_destination(replace: bool, output: Option<&Path>, kind: &str) -> CliResult<()> {
    if !replace && output.is_none() {
        return Err(cli_error(format!(
            "{kind} migration requires --output <path>, or pass --replace to retain a backup and replace the source"
        )));
    }
    Ok(())
}

fn migration_work_dir(source: &Path, operation_id: [u8; 16]) -> CliResult<PathBuf> {
    let parent = source
        .parent()
        .ok_or_else(|| cli_error("migration source has no parent"))?;
    Ok(parent.join(format!(".revault-migration-{}", hex_id(operation_id))))
}

fn temporary_vault_destination(source: &Path, id: [u8; 16]) -> PathBuf {
    source.with_file_name(format!(".vault-migrated-{}", hex_id(id)))
}

fn temporary_archive_destination(source: &Path, id: [u8; 16]) -> PathBuf {
    source.with_file_name(format!(".archive-migrated-{}.lbox", hex_id(id)))
}

fn versioned_backup_path(source: &Path, source_version: u32, target_version: u32) -> PathBuf {
    let name = source
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "artifact".to_string());
    source.with_file_name(format!(
        "{name}.v{source_version}-v{target_version}.pre-migration"
    ))
}

fn exporter_cache_root() -> CliResult<PathBuf> {
    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .ok_or_else(|| cli_error("home directory is unavailable"))?;
    Ok(PathBuf::from(home).join(".cache/revault/exporters"))
}

fn cleanup_work_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn hex_id(id: [u8; 16]) -> String {
    id.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn required_string(matches: &ArgMatches, name: &str) -> CliResult<String> {
    matches
        .get_one::<String>(name)
        .cloned()
        .ok_or_else(|| cli_error(format!("missing {name}")))
}

fn required_path(matches: &ArgMatches, name: &str) -> CliResult<PathBuf> {
    required_string(matches, name).map(PathBuf::from)
}

fn read_secret_lines_stdin(count: usize) -> CliResult<Vec<SecretString>> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let mut bytes = Vec::new();
        stdin.read_until(b'\n', &mut bytes)?;
        while bytes
            .last()
            .is_some_and(|byte| matches!(byte, b'\n' | b'\r'))
        {
            bytes.pop();
        }
        values.push(SecretString::try_from_bytes(bytes)?);
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn older_recovery_helper_reports_unidentified_partial_files() {
        let rows = historical_recovery_report_rows(&serde_json::json!({
            "intact_files": 66, "partial_files": 3, "corrupt_records": 0,
        }));
        assert!(rows.contains(&vec!["partial_file_details".to_owned(), "[]".to_owned()]));
        assert!(rows.contains(&vec![
            "unidentified_partial_files".to_owned(),
            "3".to_owned()
        ]));
    }

    #[test]
    fn source_corruption_guidance_is_typed_and_does_not_mislabel_io_failures() {
        let source = Path::new("/tmp/source.lbox");
        let corrupt = archive_source_error(
            source,
            Box::new(revault_migration::MigrationError::SourceCorrupt(
                "damaged record".into(),
            )),
        );
        assert!(corrupt.to_string().contains("doctor recover --dry-run"));
        assert!(corrupt.to_string().contains("source was not replaced"));
        for error in [
            revault_migration::MigrationError::Io("permission denied".into()),
            revault_migration::MigrationError::Io("corrupt lockbox page or record".into()),
        ] {
            assert!(!archive_source_error(source, Box::new(error))
                .to_string()
                .contains("recover"));
        }
    }

    #[test]
    fn direct_migration_identity_is_stable_and_destination_specific() {
        let fingerprint = [9; 32];
        let first = deterministic_operation_id(
            ArtifactKind::Archive,
            Path::new("/tmp/source.lbox"),
            Some(Path::new("/tmp/output.lbox")),
            fingerprint,
        );
        assert_eq!(
            first,
            deterministic_operation_id(
                ArtifactKind::Archive,
                Path::new("/tmp/source.lbox"),
                Some(Path::new("/tmp/output.lbox")),
                fingerprint,
            )
        );
        assert_ne!(
            first,
            deterministic_operation_id(
                ArtifactKind::Archive,
                Path::new("/tmp/source.lbox"),
                Some(Path::new("/tmp/other.lbox")),
                fingerprint,
            )
        );
    }

    #[test]
    fn obsolete_attempt_restarts_and_preserves_source_artifacts_and_output() {
        for stage in [
            MigrationStage::Export,
            MigrationStage::Import,
            MigrationStage::Replace,
        ] {
            let temp = tempfile::tempdir().unwrap();
            let source = temp.path().join("source.lbox");
            fs::write(&source, b"unchanged original").unwrap();
            let fingerprint = fingerprint_path(&source).unwrap();
            let work = temp.path().join(".revault-migration-test");
            fs::create_dir(&work).unwrap();
            let path = work.join("archive.migration-state");
            let artifact = work.join("archive.migration");
            fs::write(&artifact, b"encrypted old artifact").unwrap();
            // A user-selected output is outside the working directory.
            let output = temp.path().join("explicit-output.lbox");
            fs::write(&output, b"old output").unwrap();
            let make_initial = || {
                new_journal(
                    [1; 16],
                    ArtifactKind::Archive,
                    source.clone(),
                    2,
                    u32::from(LOCKBOX_FORMAT_VERSION),
                    fingerprint,
                    vec![artifact.clone(), output.clone()],
                )
                .unwrap()
            };
            let mut old = make_initial();
            old.target_format_version = 3;
            old.current_stage = stage;
            old.source_fingerprint = [0; 32];
            assert!(!restart_obsolete_migration(&path, &old, &make_initial()).unwrap());
            assert_eq!(fs::read(&output).unwrap(), b"old output");
            old.source_fingerprint = fingerprint;
            old.target_format_version = u32::from(LOCKBOX_FORMAT_VERSION) + 1;
            assert!(!restart_obsolete_migration(&path, &old, &make_initial()).unwrap());
            assert_eq!(fs::read(&output).unwrap(), b"old output");
            old.target_format_version = 3;
            old.save(&path, b"test password").unwrap();
            let old_bytes = fs::read(&path).unwrap();
            if stage == MigrationStage::Replace {
                // Simulate interruption between preserving the output and
                // renaming the old working directory.
                fs::rename(&output, work.join("previous-output")).unwrap();
            }
            let resumed = load_or_create_journal(&path, b"test password", make_initial()).unwrap();
            assert_eq!(resumed.current_stage, MigrationStage::Export);
            assert_eq!(
                resumed.target_format_version,
                u32::from(LOCKBOX_FORMAT_VERSION)
            );
            assert!(old
                .artifact_key
                .with_bytes(|old_key| resumed
                    .artifact_key
                    .with_bytes(|new_key| old_key != new_key)
                    .unwrap())
                .unwrap());
            assert_eq!(fs::read(&source).unwrap(), b"unchanged original");
            assert!(!output.exists());
            assert!(!artifact.exists());
            let preserved = fs::read_dir(temp.path())
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .find(|entry| {
                    entry
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with(".revault-preserved-migration-")
                })
                .unwrap();
            assert_eq!(
                fs::read(preserved.join("archive.migration-state")).unwrap(),
                old_bytes
            );
            assert_eq!(
                fs::read(preserved.join("archive.migration")).unwrap(),
                b"encrypted old artifact"
            );
            assert_eq!(
                fs::read(preserved.join("previous-output")).unwrap(),
                b"old output"
            );
            // A repeat resumes the new attempt with the same key.
            let repeated = load_or_create_journal(&path, b"test password", make_initial()).unwrap();
            assert!(resumed
                .artifact_key
                .with_bytes(|key| repeated
                    .artifact_key
                    .with_bytes(|other| key == other)
                    .unwrap())
                .unwrap());
        }
    }

    #[test]
    fn direct_migration_key_is_random_and_zeroizing() {
        let first = generated_migration_key().unwrap();
        let second = generated_migration_key().unwrap();
        assert_eq!(first.len(), 32);
        assert_eq!(second.len(), 32);
        assert!(first
            .with_bytes(|first| second.with_bytes(|second| first != second).unwrap())
            .unwrap());
    }

    #[test]
    fn resume_rejects_a_changed_source() {
        let journal = new_journal(
            [1; 16],
            ArtifactKind::Archive,
            PathBuf::from("source.lbox"),
            1,
            u32::from(LOCKBOX_FORMAT_VERSION),
            [2; 32],
            Vec::new(),
        )
        .unwrap();
        let error = validate_resume(
            &journal,
            ArtifactKind::Archive,
            Path::new("source.lbox"),
            1,
            [3; 32],
            Path::new("output.lbox"),
        )
        .unwrap_err();
        assert!(error.to_string().contains("source may have changed"));
    }

    #[test]
    fn interrupted_replace_is_finished_from_the_encrypted_journal() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("nested")).unwrap();
        let source = temp.path().join("secrets.lbox");
        let backup = versioned_backup_path(&source, 1, u32::from(LOCKBOX_FORMAT_VERSION));
        let output = temp.path().join("migrated.lbox");
        fs::write(&backup, b"old").unwrap();
        let signer = OwnerSigningKeyPair::generate().unwrap();
        let original = Lockbox::create_in_memory(
            revault_lockbox_api::LockboxProtection::ContentKey(
                SecretVec::try_from_slice(b"resume fixture").unwrap(),
            ),
            &signer,
        )
        .unwrap();
        let expected = original.try_to_bytes().unwrap();
        fs::write(&output, &expected).unwrap();
        let work = temp.path().join(".revault-migration-test");
        fs::create_dir(&work).unwrap();
        let mut journal = new_journal(
            [4; 16],
            ArtifactKind::Archive,
            temp.path().join("nested/../secrets.lbox"),
            1,
            u32::from(LOCKBOX_FORMAT_VERSION),
            [5; 32],
            vec![work.join("archive.migration"), output.clone()],
        )
        .unwrap();
        journal.current_stage = MigrationStage::Replace;
        journal
            .save(&work.join("archive.migration-state"), b"resume password")
            .unwrap();

        assert!(recover_interrupted_replacement(
            &source,
            ArtifactKind::Archive,
            b"resume password"
        )
        .unwrap());
        assert_eq!(fs::read(&source).unwrap(), expected);
        assert_eq!(fs::read(&backup).unwrap(), b"old");
        assert!(!work.exists());
    }

    #[test]
    fn newer_format_replacement_is_refused_without_modifying_files() {
        for kind in [ArtifactKind::Archive, ArtifactKind::Vault] {
            let current_target = match kind {
                ArtifactKind::Archive => u32::from(LOCKBOX_FORMAT_VERSION),
                ArtifactKind::Vault => CURRENT_VAULT_STRUCTURE_VERSION,
                ArtifactKind::Journal => unreachable!("fixture covers only archives and Vaults"),
            };
            for target in [3, 4] {
                for slot_offset in [0, 192] {
                    let temp = tempfile::tempdir().unwrap();
                    let source = temp.path().join("source");
                    let output = temp.path().join("output");
                    let container = if kind == ArtifactKind::Vault {
                        fs::create_dir(&output).unwrap();
                        output.join("local-vault.lbox")
                    } else {
                        output.clone()
                    };
                    // No current CLI can create a future header or damage its
                    // first slot. Keep opaque synthetic bytes for this refusal
                    // fixture, including a valid marker only in slot two.
                    let mut future = vec![0_u8; 384];
                    future[slot_offset..slot_offset + 8].copy_from_slice(b"LBX4HDR\0");
                    future[slot_offset + 8..slot_offset + 10].copy_from_slice(&5_u16.to_le_bytes());
                    // The discriminator accepts only checksum-valid future
                    // headers; random corruption must not imply a new format.
                    let mut checksum = Sha256::new();
                    checksum.update(b"lockbox-v1-public-checksum/sha256");
                    checksum.update(160_u64.to_le_bytes());
                    checksum.update(&future[slot_offset..slot_offset + 160]);
                    future[slot_offset + 160..slot_offset + 192]
                        .copy_from_slice(&checksum.finalize());
                    assert_eq!(probe_lockbox_format_version(&future).unwrap(), 5);
                    fs::write(&container, &future).unwrap();
                    let backup = versioned_backup_path(&source, 1, target);
                    fs::write(&backup, b"original backup").unwrap();
                    let work = temp.path().join(".revault-migration-future");
                    fs::create_dir(&work).unwrap();
                    let mut journal = new_journal(
                        [6; 16],
                        kind,
                        source.clone(),
                        1,
                        target,
                        [7; 32],
                        vec![output.clone()],
                    )
                    .unwrap();
                    journal.current_stage = MigrationStage::Replace;
                    let journal_path = work.join(if kind == ArtifactKind::Vault {
                        "vault.migration-state"
                    } else {
                        "archive.migration-state"
                    });
                    journal.save(&journal_path, b"resume password").unwrap();
                    let journal_bytes = fs::read(&journal_path).unwrap();
                    let error = recover_interrupted_replacement(&source, kind, b"resume password")
                        .unwrap_err();
                    let expected = if target == current_target {
                        "downgrade"
                    } else {
                        "saved migration targets format"
                    };
                    assert!(error.to_string().contains(expected), "{error}");
                    let error =
                        validate_resume(&journal, kind, &source, 1, [7; 32], &output).unwrap_err();
                    assert!(error.to_string().contains(expected), "{error}");
                    // The same resume guard is called before Import can discard
                    // an incomplete output. Assert that stage separately too.
                    journal.current_stage = MigrationStage::Import;
                    assert!(validate_resume(&journal, kind, &source, 1, [7; 32], &output).is_err());
                    // With a present original, the restart helper must also
                    // preserve a future-format output before resume rejects it.
                    fs::write(&source, b"original source").unwrap();
                    let initial = new_journal(
                        journal.operation_id,
                        kind,
                        source.clone(),
                        1,
                        current_target,
                        [7; 32],
                        vec![output.clone()],
                    )
                    .unwrap();
                    assert!(
                        !restart_obsolete_migration(&journal_path, &journal, &initial).unwrap()
                    );
                    assert_eq!(fs::read(&source).unwrap(), b"original source");
                    fs::remove_file(&source).unwrap();
                    assert!(!source.exists());
                    assert_eq!(fs::read(&container).unwrap(), future);
                    assert_eq!(fs::read(&backup).unwrap(), b"original backup");
                    assert_eq!(fs::read(&journal_path).unwrap(), journal_bytes);
                }
            }
        }
    }

    #[test]
    fn older_saved_output_is_not_deleted_or_installed_by_resume() {
        // Historical native setup has no current CLI writer. Use the pinned
        // released format-2 API solely to create the unsupported old output.
        use revault_lockbox_api_structure_v2 as old;
        let secret = old::SecretString::try_from_slice(b"old-output-password").unwrap();
        let signer = old::OwnerSigningKeyPair::generate().unwrap();
        let archive =
            old::Lockbox::create_in_memory(old::LockboxProtection::Password(&secret), &signer)
                .unwrap();
        let bytes = archive.try_to_bytes().unwrap();
        assert_eq!(probe_lockbox_format_version(&bytes).unwrap(), 2);
        for kind in [ArtifactKind::Archive, ArtifactKind::Vault] {
            let target = match kind {
                ArtifactKind::Archive => u32::from(LOCKBOX_FORMAT_VERSION),
                ArtifactKind::Vault => CURRENT_VAULT_STRUCTURE_VERSION,
                ArtifactKind::Journal => unreachable!("fixture covers only archives and Vaults"),
            };
            let temp = tempfile::tempdir().unwrap();
            let source = temp.path().join("source");
            let output = temp.path().join("output");
            let container = if kind == ArtifactKind::Vault {
                fs::create_dir(&output).unwrap();
                output.join("local-vault.lbox")
            } else {
                output.clone()
            };
            fs::write(&container, &bytes).unwrap();
            let backup = versioned_backup_path(&source, 1, target);
            fs::write(&backup, b"original backup").unwrap();
            let work = temp.path().join(".revault-migration-old-output");
            fs::create_dir(&work).unwrap();
            let mut journal = new_journal(
                [8; 16],
                kind,
                source.clone(),
                1,
                target,
                [9; 32],
                vec![output.clone()],
            )
            .unwrap();
            journal.current_stage = MigrationStage::Replace;
            let journal_path = work.join(if kind == ArtifactKind::Vault {
                "vault.migration-state"
            } else {
                "archive.migration-state"
            });
            journal.save(&journal_path, b"resume password").unwrap();
            let journal_bytes = fs::read(&journal_path).unwrap();
            let error =
                recover_interrupted_replacement(&source, kind, b"resume password").unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("saved migration output uses older format 2"),
                "{error}"
            );
            for stage in [MigrationStage::Import, MigrationStage::Replace] {
                journal.current_stage = stage;
                let error =
                    validate_resume(&journal, kind, &source, 1, [9; 32], &output).unwrap_err();
                assert!(
                    error
                        .to_string()
                        .contains("saved migration output uses unsupported format 2"),
                    "{error}"
                );
            }
            assert!(!source.exists());
            assert_eq!(fs::read(&container).unwrap(), bytes);
            assert_eq!(fs::read(&backup).unwrap(), b"original backup");
            assert_eq!(fs::read(&journal_path).unwrap(), journal_bytes);

            // Once an intact original is available, the high-level loader can
            // safely restart even when a Vault's structure target is unchanged
            // but its saved output uses an older container format.
            let source_container = if kind == ArtifactKind::Vault {
                fs::create_dir(&source).unwrap();
                source.join("local-vault.lbox")
            } else {
                source.clone()
            };
            fs::write(&source_container, b"original source").unwrap();
            journal.source_fingerprint = fingerprint_path(&source_container).unwrap();
            journal.save(&journal_path, b"resume password").unwrap();
            let initial = new_journal(
                journal.operation_id,
                kind,
                source.clone(),
                1,
                target,
                journal.source_fingerprint,
                vec![output.clone()],
            )
            .unwrap();
            let restarted =
                load_or_create_journal(&journal_path, b"resume password", initial).unwrap();
            assert_eq!(restarted.current_stage, MigrationStage::Export);
            assert!(!output.exists());
            assert_eq!(fs::read(&source_container).unwrap(), b"original source");
        }
    }

    #[test]
    fn exporter_registry_and_capability_contract_are_exact() {
        for version in 1..=3 {
            for kind in [ArtifactKind::Vault, ArtifactKind::Archive] {
                let release = exporter_release(kind, version).unwrap();
                assert_eq!(release.package, format!("revault_migrate_v{version}"));
                assert_eq!(release.version, "0.1.0");
            }
        }
        let raw = exporter_release(ArtifactKind::Vault, 3).unwrap();
        let recovery = exporter_release(ArtifactKind::Archive, 2).unwrap();
        assert!(!capabilities_match(
            br#"{"protocol":3,"artifact":"archive","native_version":2,"migration_schema":3}"#,
            recovery
        ));
        assert!(capabilities_match(br#"{"protocol":3,"artifact":"archive","native_version":2,"migration_schema":3,"recovery":true}"#, recovery));
        let mut capability = serde_json::json!({"protocol":4,"artifact":"vault","native_version":3,"migration_schema":3,"container_version":3,"structure_versions":[3],"migration_schemas":[3],"raw_container_export":true});
        assert!(capabilities_match(
            &serde_json::to_vec(&capability).unwrap(),
            raw
        ));
        capability["raw_container_export"] = false.into();
        assert!(!capabilities_match(
            &serde_json::to_vec(&capability).unwrap(),
            raw
        ));
        let vault_v2 = exporter_release(ArtifactKind::Vault, 2).unwrap();
        assert_eq!(vault_v2.package, "revault_migrate_v2");
        assert!(capabilities_match(
            br#"{"protocol":2,"artifact":"vault","native_version":2,"migration_schema":2,"container_version":2,"structure_versions":[2],"migration_schemas":[2]}"#,
            vault_v2
        ));
        assert!(!capabilities_match(
            br#"{"protocol":2,"artifact":"vault","native_version":2,"migration_schema":2,"container_version":1,"structure_versions":[2],"migration_schemas":[2]}"#,
            vault_v2
        ));
        let vault = exporter_release(ArtifactKind::Vault, 1).unwrap();
        assert_eq!(vault.package, "revault_migrate_v1");
        assert_eq!(vault.binary, "revault-migrate-vault-v1");

        assert!(capabilities_match(
            br#"{"protocol":2,"artifact":"vault","native_version":1,"migration_schema":2,"container_version":1,"structure_versions":[1,2],"migration_schemas":[1,2]}"#,
            vault
        ));
        assert!(!capabilities_match(
            br#"{"protocol":1,"artifact":"archive","native_version":1,"migration_schema":1}"#,
            vault
        ));

        let archive = exporter_release(ArtifactKind::Archive, 1).unwrap();
        assert_eq!(archive.package, "revault_migrate_v1");
        assert_eq!(archive.binary, "revault-migrate-archive-v1");

        assert!(capabilities_match(
            br#"{"protocol":3,"artifact":"archive","native_version":1,"migration_schema":2}"#,
            archive
        ));
        assert!(!capabilities_match(
            br#"{"protocol":1,"artifact":"archive","native_version":1,"migration_schema":1}"#,
            archive
        ));
    }
}
