use super::context::{
    cli_error, default_vault, open_default_vault_with_password, open_existing,
    vault_password_without_open, Access, CliResult,
};
use crate::secret_prompt::prompt_secret;
use clap::ArgMatches;
use revault_lockbox_api::{
    probe_lockbox_format_version, Lockbox, OwnerSigningKeyPair, SecretString, SecretVec,
    LOCKBOX_FORMAT_VERSION,
};
use revault_migration::{
    export_archive, export_vault_v2, import_archive, import_vault_v2, upgrade_archive_artifact,
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
        Some(("archive", sub)) => archive_matches(sub, access),
        Some((other, _)) => Err(cli_error(format!("unknown migration command: {other}"))),
        None => Err(cli_error("migrate requires vault or archive")),
    }
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

fn archive_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    match matches.subcommand() {
        Some(("export", sub)) => archive_export(sub, access),
        Some(("upgrade", sub)) => archive_upgrade(sub),
        Some(("import", sub)) => archive_import(sub),
        Some(("verify", sub)) => archive_verify(sub),
        Some((other, _)) => Err(cli_error(format!(
            "unknown archive migration command: {other}"
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
    let count = export_vault_v2(&vault, &output, &artifact_password, operation_id)?;
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
    let count = import_vault_v2(&input, &artifact_password, &output, &vault_password)?;
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

fn archive_export(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let source = required_string(matches, "lockbox")?;
    let output = required_path(matches, "output")?;
    let artifact_password = if matches.get_flag("migration-password-stdin") {
        read_secret_lines_stdin(1)?.remove(0)
    } else {
        migration_password()?
    };
    let lockbox = open_existing(&source, access)?;
    let operation_id = random_id()?;
    let count = export_archive(&lockbox, &output, &artifact_password, operation_id)?;
    println!(
        "Exported {count} archive migration records to {}",
        output.display()
    );
    Ok(())
}

fn archive_import(matches: &ArgMatches) -> CliResult<()> {
    let input = required_path(matches, "artifact")?;
    let output = required_path(matches, "output")?;
    let password = migration_password()?;
    let signing = default_signing_key()?;
    let count = import_archive(&input, &password, &output, &signing)?;
    println!(
        "Imported {count} archive migration records into {}",
        output.display()
    );
    println!("A new signed commit chain was created for the migrated archive.");
    Ok(())
}

fn archive_upgrade(matches: &ArgMatches) -> CliResult<()> {
    let input = required_path(matches, "artifact")?;
    let output = required_path(matches, "output")?;
    let password = migration_password()?;
    let count = upgrade_archive_artifact(&input, &output, &password)?;
    println!(
        "Upgraded {count} archive migration frames to {}",
        output.display()
    );
    Ok(())
}

fn archive_verify(matches: &ArgMatches) -> CliResult<()> {
    let input = required_path(matches, "artifact")?;
    let password = migration_password()?;
    let count = verify_archive_artifact(&input, &password)?;
    println!("Archive migration artifact is valid ({count} frames).");
    Ok(())
}

fn migrate_vault_direct(matches: &ArgMatches) -> CliResult<()> {
    let replace = matches.get_flag("replace");
    let requested_output = matches.get_one::<String>("output").map(PathBuf::from);
    require_destination(replace, requested_output.as_deref(), "vault")?;
    let source = default_vault_dir()?;
    let source_password = vault_password_without_open()?;
    if !source.exists()
        && recover_interrupted_replacement(&source, ArtifactKind::Vault, &source_password)?
    {
        println!("Completed the interrupted vault replacement.");
        return Ok(());
    }
    let source_version = VaultDirectory::probe_structure_version(&source, &source_password)?;
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
        let complete =
            artifact.exists() && verify_vault_artifact(&artifact, &migration_key).is_ok();
        if !complete {
            remove_partial(&artifact)?;
            if source_version == CURRENT_VAULT_STRUCTURE_VERSION {
                let vault = VaultDirectory::open_or_create(&source, &source_password)?;
                export_vault_v2(&vault, &artifact, &migration_key, operation_id)?;
            } else {
                let exporter = resolve_exporter(
                    ArtifactKind::Vault,
                    source_version,
                    matches.get_one::<String>("exporter").map(PathBuf::from),
                )?;
                let artifact_password = SecretString::from_secure_vec(migration_key.try_clone()?);
                run_historical_vault_exporter(
                    &exporter,
                    &source,
                    &artifact,
                    &source_password,
                    &artifact_password,
                )?;
                journal.exporter_version = Some(exporter.display().to_string());
            }
        }
        journal.current_stage = MigrationStage::Upgrade;
        save_journal(&mut journal, &journal_path, &source_password)?;
    }
    if journal.current_stage == MigrationStage::Upgrade {
        let complete =
            upgraded.exists() && verify_vault_artifact(&upgraded, &migration_key).is_ok();
        if !complete {
            remove_partial(&upgraded)?;
            upgrade_vault_artifact(&artifact, &upgraded, &migration_key)?;
        }
        journal.current_stage = MigrationStage::Import;
        save_journal(&mut journal, &journal_path, &source_password)?;
    }
    if journal.current_stage == MigrationStage::Import {
        let complete = output.exists()
            && VaultDirectory::open_or_create(&output, &source_password)
                .and_then(|vault| vault.structure_version())
                .is_ok_and(|version| version == CURRENT_VAULT_STRUCTURE_VERSION);
        if !complete {
            remove_partial(&output)?;
            import_vault_v2(&upgraded, &migration_key, &output, &source_password)?;
        }
        journal.current_stage = MigrationStage::Validate;
        save_journal(&mut journal, &journal_path, &source_password)?;
    }
    if journal.current_stage == MigrationStage::Validate {
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
        let backup = versioned_backup_path(&source, source_version);
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
        println!("Vault migrated to format version {CURRENT_VAULT_STRUCTURE_VERSION}.");
        println!("Previous vault retained at {}", backup.display());
    } else {
        println!("Vault migrated to {}", output.display());
    }
    cleanup_work_dir(&work_dir);
    Ok(())
}

fn migrate_archive_direct(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let source = PathBuf::from(
        matches
            .get_one::<String>("lockbox")
            .ok_or_else(|| cli_error("archive migration requires a lockbox path"))?,
    );
    let replace = matches.get_flag("replace");
    let requested_output = matches.get_one::<String>("output").map(PathBuf::from);
    require_destination(replace, requested_output.as_deref(), "archive")?;
    let vault_root = default_vault_dir()?;
    if !vault_root.join("local-vault.lbox").exists() {
        return Err(cli_error(
            "archive migration requires a current vault; run `lockbox vault init` first",
        ));
    }
    let vault_password = vault_password_without_open()?;
    let vault_version = VaultDirectory::probe_structure_version(&vault_root, &vault_password)?;
    if vault_version < CURRENT_VAULT_STRUCTURE_VERSION {
        return Err(cli_error(format!(
            "archive migration requires the vault to be migrated first; vault format version {vault_version} is older than the current version {CURRENT_VAULT_STRUCTURE_VERSION}. Run `lockbox migrate vault --replace` or migrate it with `--output <directory>`"
        )));
    }
    if vault_version > CURRENT_VAULT_STRUCTURE_VERSION {
        return Err(cli_error(format!(
            "archive migration cannot run with a newer vault format version {vault_version}; this build supports version {CURRENT_VAULT_STRUCTURE_VERSION}. Install a newer reVault release"
        )));
    }
    let vault = open_default_vault_with_password(&vault_password)?;
    if !source.exists()
        && recover_interrupted_replacement(&source, ArtifactKind::Archive, &vault_password)?
    {
        println!("Completed the interrupted archive replacement.");
        return Ok(());
    }
    let source_version = probe_archive_path(&source)?;
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
            if source_version == u32::from(LOCKBOX_FORMAT_VERSION) {
                let lockbox = open_existing(&source.to_string_lossy(), access)?;
                export_archive(&lockbox, &artifact, &migration_key, operation_id)?;
            } else {
                let exporter = resolve_exporter(
                    ArtifactKind::Archive,
                    source_version,
                    matches.get_one::<String>("exporter").map(PathBuf::from),
                )?;
                let artifact_password = SecretString::from_secure_vec(migration_key.try_clone()?);
                run_historical_archive_exporter(
                    &exporter,
                    &source,
                    &artifact,
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
        let complete = output.exists() && Lockbox::inspect_file(&output).is_ok();
        if !complete {
            remove_partial(&output)?;
            let signing = vault.load_owner_signing_key(VaultDirectory::DEFAULT_KEY_NAME)?;
            import_archive(&upgraded, &migration_key, &output, &signing)?;
        }
        journal.current_stage = MigrationStage::Validate;
        save_journal(&mut journal, &journal_path, &vault_password)?;
    }
    if journal.current_stage == MigrationStage::Validate {
        Lockbox::inspect_file(&output)?;
        journal.current_stage = if replace {
            MigrationStage::Replace
        } else {
            MigrationStage::Complete
        };
        save_journal(&mut journal, &journal_path, &vault_password)?;
    }
    if replace {
        let backup = versioned_backup_path(&source, source_version);
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
        println!("Archive migrated to format version {LOCKBOX_FORMAT_VERSION}.");
        println!("Previous archive retained at {}", backup.display());
    } else {
        println!("Archive migrated to {}", output.display());
    }
    println!("A new signed commit chain was created for the migrated archive.");
    cleanup_work_dir(&work_dir);
    Ok(())
}

fn resolve_exporter(
    kind: ArtifactKind,
    source_version: u32,
    explicit: Option<PathBuf>,
) -> CliResult<PathBuf> {
    let release = exporter_release(kind, source_version).ok_or_else(|| {
        cli_error(format!(
            "no crates.io exporter is registered for {:?} format version {source_version}",
            kind
        ))
    })?;
    if let Some(path) = explicit {
        validate_exporter_capabilities(&path, release)?;
        return Ok(path);
    }
    let root = exporter_cache_root()?.join(format!("{}-{}", release.package, release.version));
    let binary_name = if cfg!(windows) {
        format!("{}.exe", release.binary)
    } else {
        release.binary.to_string()
    };
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

#[derive(Clone, Copy)]
struct ExporterRelease {
    package: &'static str,
    version: &'static str,
    binary: &'static str,
    protocol: u64,
    artifact: &'static str,
    native_version: u32,
    migration_schema: u32,
}

fn exporter_release(kind: ArtifactKind, source_version: u32) -> Option<ExporterRelease> {
    match (kind, source_version) {
        (ArtifactKind::Vault, 1) => Some(ExporterRelease {
            package: "revault_migrate_vault_v1",
            version: "0.0.1",
            binary: "revault-migrate-vault-v1",
            protocol: 1,
            artifact: "vault",
            native_version: 1,
            migration_schema: 1,
        }),
        (ArtifactKind::Archive, 1) => Some(ExporterRelease {
            package: "revault_migrate_archive_v1",
            version: "0.0.2",
            binary: "revault-migrate-archive-v1",
            protocol: 2,
            artifact: "archive",
            native_version: 1,
            migration_schema: 1,
        }),
        _ => None,
    }
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

fn capabilities_match(bytes: &[u8], release: ExporterRelease) -> bool {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return false;
    };
    let valid = value.get("protocol").and_then(|value| value.as_u64()) == Some(release.protocol)
        && value.get("artifact").and_then(|value| value.as_str()) == Some(release.artifact)
        && value.get("native_version").and_then(|value| value.as_u64())
            == Some(u64::from(release.native_version))
        && value
            .get("migration_schema")
            .and_then(|value| value.as_u64())
            == Some(u64::from(release.migration_schema));
    valid
}

fn run_historical_vault_exporter(
    exporter: &Path,
    source: &Path,
    output: &Path,
    vault_password: &SecretString,
    artifact_password: &SecretString,
) -> CliResult<()> {
    let mut child = Command::new(exporter)
        .args(["migrate", "vault", "export", "--output"])
        .arg(output)
        .arg("--source")
        .arg(source)
        .stdin(Stdio::piped())
        .spawn()?;
    write_subprocess_secrets(&mut child, &[vault_password, artifact_password])?;
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
    vault_password: &SecretString,
    artifact_password: &SecretString,
) -> CliResult<()> {
    let mut child = Command::new(exporter)
        .args(["migrate", "archive", "export"])
        .arg(source)
        .arg("--output")
        .arg(output)
        .stdin(Stdio::piped())
        .spawn()?;
    write_subprocess_secrets(&mut child, &[vault_password, artifact_password])?;
    let status = child.wait()?;
    if !status.success() {
        return Err(cli_error(format!(
            "historical archive exporter exited with {status}"
        )));
    }
    Ok(())
}

fn write_subprocess_secrets(
    child: &mut std::process::Child,
    secrets: &[&SecretString],
) -> CliResult<()> {
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| cli_error("exporter stdin unavailable"))?;
    stdin.write_all(b"LBXMIPC1")?;
    stdin.write_all(
        &u32::try_from(secrets.len())
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
    drop(stdin);
    Ok(())
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
    let mut file = File::open(path)?;
    let mut header = [0u8; 96];
    file.read_exact(&mut header)?;
    Ok(u32::from(probe_lockbox_format_version(&header)?))
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
) -> revault_migration::Result<MigrationJournal> {
    if path.exists() {
        MigrationJournal::load(path, passphrase)
    } else {
        initial.save(path, passphrase)?;
        Ok(initial)
    }
}

fn validate_resume(
    journal: &MigrationJournal,
    kind: ArtifactKind,
    source: &Path,
    source_version: u32,
    fingerprint: [u8; 32],
    output: &Path,
) -> CliResult<()> {
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
            || journal.source_path != source
            || journal.current_stage != MigrationStage::Replace
        {
            continue;
        }
        let Some(output) = journal.temporary_paths.last() else {
            continue;
        };
        let backup = versioned_backup_path(source, journal.source_format_version);
        if source.exists() || !backup.exists() || !output.exists() {
            continue;
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
    let mut buffer = [0u8; 1024 * 1024];
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

fn versioned_backup_path(source: &Path, version: u32) -> PathBuf {
    let name = source
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "artifact".to_string());
    source.with_file_name(format!("{name}.v{version}.pre-migration"))
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
            2,
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
        let source = temp.path().join("secrets.lbox");
        let backup = versioned_backup_path(&source, 1);
        let output = temp.path().join("migrated.lbox");
        fs::write(&backup, b"old").unwrap();
        fs::write(&output, b"new").unwrap();
        let work = temp.path().join(".revault-migration-test");
        fs::create_dir(&work).unwrap();
        let mut journal = new_journal(
            [4; 16],
            ArtifactKind::Archive,
            source.clone(),
            1,
            2,
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
        assert_eq!(fs::read(&source).unwrap(), b"new");
        assert_eq!(fs::read(&backup).unwrap(), b"old");
        assert!(!work.exists());
    }

    #[test]
    fn exporter_registry_and_capability_contract_are_exact() {
        let vault = exporter_release(ArtifactKind::Vault, 1).unwrap();
        assert_eq!(vault.package, "revault_migrate_vault_v1");
        assert_eq!(vault.binary, "revault-migrate-vault-v1");
        assert!(capabilities_match(
            br#"{"protocol":1,"artifact":"vault","native_version":1,"migration_schema":1}"#,
            vault
        ));
        assert!(!capabilities_match(
            br#"{"protocol":1,"artifact":"archive","native_version":1,"migration_schema":1}"#,
            vault
        ));

        let archive = exporter_release(ArtifactKind::Archive, 1).unwrap();
        assert_eq!(archive.package, "revault_migrate_archive_v1");
        assert_eq!(archive.binary, "revault-migrate-archive-v1");
        assert_eq!(archive.version, "0.0.2");
        assert!(capabilities_match(
            br#"{"protocol":2,"artifact":"archive","native_version":1,"migration_schema":1}"#,
            archive
        ));
        assert!(!capabilities_match(
            br#"{"protocol":1,"artifact":"archive","native_version":1,"migration_schema":1}"#,
            archive
        ));
    }
}
