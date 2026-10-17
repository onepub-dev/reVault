use super::context::{cli_error, default_vault, lockbox_open_error, Access, CliResult};
use super::optional_lockbox_value;
use super::output::human_size;
use super::output::{output_format_from_matches, print_records, OutputFormat};
use clap::ArgMatches;
use revault_lockbox_api::vault_integration::VaultOpen;
use revault_lockbox_api::{
    Error, FileLockScope, RecoveryReport, RecoveryScanner, ScopedFileLock, SecretVec,
};
use revault_lockbox_api::{
    Lockbox, LockboxOpen, TransactionRecoveryProgress, TransactionRecoveryStatus,
};
use revault_vault_api::{get as get_cached_content_key, VaultDirectory};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub(crate) fn run_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let options = RecoverOptions::from_matches(matches)?;
    if let Some(path) = &options.diagnostics {
        let identity = |path: &Path| -> std::io::Result<PathBuf> {
            let absolute = std::path::absolute(path)?;
            Ok(fs::canonicalize(&absolute).unwrap_or_else(|_| {
                let parent = absolute.parent().unwrap_or(Path::new("/"));
                fs::canonicalize(parent)
                    .unwrap_or_else(|_| parent.to_owned())
                    .join(absolute.file_name().unwrap_or_default())
            }))
        };
        let diagnostic = identity(Path::new(path))?;
        let output = options
            .output
            .clone()
            .unwrap_or_else(|| default_recovered_path(&options.lockbox_path));
        let directory =
            options
                .directory
                .clone()
                .map(PathBuf::from)
                .unwrap_or(default_recovery_directory(Path::new(
                    &options.lockbox_path,
                ))?);
        if [
            Path::new(&options.lockbox_path),
            Path::new(&output),
            directory.as_path(),
        ]
        .iter()
        .any(|path| identity(path).is_ok_and(|path| path == diagnostic))
        {
            return Err(cli_error("diagnostics must use a separate new file, not the source or a recovery destination"));
        }
    }
    let mut telemetry = super::recovery_telemetry::RecoveryTelemetry::new(
        options.diagnostics.as_deref().map(Path::new),
        options.quiet,
    )?;
    let result = std::thread::scope(|scope| {
        let (stop, stopped) = std::sync::mpsc::channel();
        let monitor_telemetry = &telemetry;
        let monitor = scope.spawn(move || {
            while matches!(
                stopped.recv_timeout(std::time::Duration::from_secs(5)),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout)
            ) {
                monitor_telemetry.heartbeat();
            }
        });
        telemetry.event("starting", None);
        let result = run_options(options, access, &telemetry);
        let _ = stop.send(());
        let _ = monitor.join();
        result
    });
    if let Err(error) = telemetry.finish(result.as_ref().err().map(|error| &**error)) {
        if result.is_ok() {
            return Err(error);
        }
        eprintln!("Recovery diagnostic report could not be completed: {error}");
    }
    result
}

fn run_options(
    options: RecoverOptions,
    access: &Access,
    telemetry: &super::recovery_telemetry::RecoveryTelemetry,
) -> CliResult<()> {
    telemetry.event("reading_source", None);
    let output = options
        .output
        .clone()
        .unwrap_or_else(|| default_recovered_path(&options.lockbox_path));
    let directory = match options.directory.as_deref() {
        Some(path) => std::path::absolute(path)?,
        None => default_recovery_directory(Path::new(&options.lockbox_path))?,
    };
    let mut header = Vec::new();
    fs::File::open(&options.lockbox_path)?
        .take(384)
        .read_to_end(&mut header)?;
    match revault_lockbox_api::probe_lockbox_format_version(&header) {
        Ok(2) => {
            let output = Path::new(&output);
            if !options.dry_run {
                if same_existing_path(Path::new(&options.lockbox_path), output) {
                    return Err(cli_error("historical recovery requires a separate output; the original must be retained"));
                }
                if output.exists() && !options.overwrite {
                    return Err(Error::AlreadyExists(output.display().to_string()).into());
                }
            }
            return super::migrate::recover_historical_archive(
                Path::new(&options.lockbox_path), Some(output), options.overwrite,
                options.dry_run, match options.format {
                    OutputFormat::Table => "table",
                    OutputFormat::Tsv => "tsv",
                    OutputFormat::Json => "json",
                }, options.quiet, access, Some(&directory), telemetry,
            );
        }
        Ok(1) => return Err(cli_error("format-1 recovery is not supported by this build's recovery scanner; retain the original and restore a known-good backup. Migration can read intact format-1 archives, but cannot repair damaged ones")),
        Ok(found) if found != revault_lockbox_api::LOCKBOX_FORMAT_VERSION => return Err(lockbox_open_error(&options.lockbox_path, Error::UnsupportedFormatVersion {
            artifact: revault_lockbox_api::ArtifactKind::Lockbox,
            found: u32::from(found), supported: u32::from(revault_lockbox_api::LOCKBOX_FORMAT_VERSION),
        })),
        Ok(_) | Err(Error::CorruptHeader | Error::CorruptRecord | Error::Truncated) => {}
        Err(error) => return Err(error.into()),
    }
    if options.overwrite && same_existing_path(Path::new(&options.lockbox_path), Path::new(&output))
    {
        match inspect_pending_cleanup(&options.lockbox_path, access) {
            Ok(Some(status)) => {
                if options.dry_run {
                    print_pending_cleanup(&status, options.format)?;
                    return Ok(());
                }
                return recover_pending_cleanup(
                    &options.lockbox_path,
                    access,
                    &status,
                    options.quiet,
                );
            }
            Ok(None) => {}
            Err(err)
                if matches!(
                    err.downcast_ref::<Error>(),
                    Some(Error::CorruptHeader | Error::CorruptRecord | Error::Truncated)
                ) => {}
            Err(err) => return Err(err),
        }
    }
    if options.dry_run {
        telemetry.event("scanning_records", None);
        let report = scan_report(&options.lockbox_path, access, options.quiet)?;
        print_report(&report, options.format)?;
        let key = match access {
            Access::ContentKey(key) => key.try_clone()?,
            Access::CacheOnly => cached_key(&options.lockbox_path)?,
            Access::PromptPassword => {
                return Err(cli_error("recover requires --key or an open lockbox"))
            }
        };
        let bytes = read_recovery_bytes(&options.lockbox_path, options.quiet)?;
        let raw = key.with_bytes(|key| zeroize::Zeroizing::new(key.to_vec()))?;
        let mut plan = RecoveryScanner::plan_file_fragments_and_progress(
            &bytes,
            &raw,
            None,
            |_, _, _| Ok(()),
            |done, total| telemetry.event("verifying_files", Some((done, total))),
        )?;
        telemetry.counts(&plan);
        super::recovery_directory::set_archive_destination(
            &mut plan,
            Path::new(&output),
            &directory,
            true,
        )?;
        super::recovery_directory::print_plan(&plan, options.format)?;
        return Ok(());
    }
    let output_path = Path::new(&output);
    let input_path = Path::new(&options.lockbox_path);
    let in_place = same_existing_path(input_path, output_path);
    if output_path.exists() && !options.overwrite {
        return Err(Error::AlreadyExists(output).into());
    }
    let recovery_key = match access {
        Access::ContentKey(key) => key.try_clone()?,
        Access::CacheOnly => cached_key(&options.lockbox_path)?,
        Access::PromptPassword => {
            return Err(
                Error::InvalidInput("recover requires --key or an open lockbox".into()).into(),
            )
        }
    };
    let source_guard = if in_place {
        ScopedFileLock::acquire(input_path, FileLockScope::Recovery)?
    } else {
        ScopedFileLock::read_archive(input_path)?
    };
    let bytes = source_guard.read_archive_bytes()?;
    telemetry.event("scanning_records", None);
    recovery_stage(options.quiet, "scanning readable encrypted records.");
    let raw = recovery_key.with_bytes(|key| zeroize::Zeroizing::new(key.to_vec()))?;
    let source_report = RecoveryScanner::scan_slice(&bytes, &raw);
    let signing_key = default_vault()?.load_owner_signing_key(VaultDirectory::DEFAULT_KEY_NAME)?;
    let mut partial_writer = None;
    telemetry.event("restoring_archive", None);
    let (recovered, mut plan) = RecoveryScanner::salvage_with_file_fragments_and_progress(
        &bytes,
        &raw,
        &signing_key,
        |id, offset, data| {
            if partial_writer.is_none() {
                partial_writer = Some(
                    super::recovery_directory::DirectoryWriter::new(&directory)
                        .map_err(|error| Error::Io(error.to_string()))?,
                );
            }
            partial_writer
                .as_mut()
                .unwrap()
                .range(id, offset, data)
                .map_err(|error| Error::Io(error.to_string()))
        },
        |done, total| telemetry.event("verifying_files", Some((done, total))),
    )?;
    telemetry.counts(&plan);
    super::recovery_directory::set_archive_destination(&mut plan, output_path, &directory, false)?;
    if super::recovery_directory::has_partial_files(&plan) && partial_writer.is_none() {
        partial_writer = Some(super::recovery_directory::DirectoryWriter::new(&directory)?);
    }
    let parent = output_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut staged = tempfile::NamedTempFile::new_in(parent)?;
    telemetry.event("writing_archive", None);
    use std::io::Write;
    staged.write_all(&recovered.try_to_bytes()?)?;
    staged.as_file().sync_all()?;
    let replacement_guard = ScopedFileLock::lock_archive(staged.as_file().try_clone()?)?;
    let destination_guard = if !in_place && output_path.exists() {
        Some(ScopedFileLock::acquire(
            output_path,
            FileLockScope::Recovery,
        )?)
    } else {
        None
    };
    let damaged_original = if in_place {
        let backup = next_damaged_backup_path(input_path);
        fs::hard_link(input_path, &backup).map_err(|err| {
            Error::Io(format!(
                "retain damaged lockbox {} at {}: {err}",
                options.lockbox_path,
                backup.display()
            ))
        })?;
        Some(backup)
    } else {
        None
    };
    if let Some(writer) = partial_writer {
        writer.finish(plan.clone())?;
    }
    telemetry.event("publishing_outputs", None);
    let publication = (|| -> CliResult<()> {
        if options.overwrite {
            #[cfg(windows)]
            {
                // tempfile uses MoveFileExW alone. std::fs::rename also supports
                // replacing an open destination via FileRenameInfoEx on Windows.
                // Keep the archive guards held throughout publication.
                let (_file, path) = staged.keep().map_err(|err| err.error)?;
                let path = tempfile::TempPath::try_from_path(path)?;
                fs::rename(&path, output_path).map_err(|err| {
                    Error::Io(format!(
                        "publish recovered lockbox {}: {err}",
                        output_path.display()
                    ))
                })?;
            }
            #[cfg(not(windows))]
            staged.persist(output_path).map_err(|err| err.error)?;
        } else {
            staged
                .persist_noclobber(output_path)
                .map_err(|err| err.error)?;
        }
        Ok(())
    })();
    publication.map_err(|error| {
        cli_error(format!(
            "could not publish recovered Lockbox: {error}; partial-file recovery retained at {}",
            plan["recovery_directory"]
                .as_str()
                .unwrap_or("(none required)")
        ))
    })?;
    #[cfg(unix)]
    fs::File::open(parent)?.sync_all()?;
    drop(destination_guard);
    drop(source_guard);
    drop(replacement_guard);
    let report = scan_report(&output, access, options.quiet)?;
    telemetry.event("complete", None);
    let mut rows = report_rows(&source_report, Some(&output), damaged_original.as_deref());
    rows.push(vec![
        "output_intact_file_count".to_owned(),
        report.intact_file_count.to_string(),
    ]);
    super::output::print_recovery_records(rows, options.format)?;
    super::recovery_directory::print_plan(&plan, options.format)?;
    Ok(())
}

struct RecoverOptions {
    lockbox_path: String,
    directory: Option<String>,
    output: Option<String>,
    overwrite: bool,
    dry_run: bool,
    quiet: bool,
    format: OutputFormat,
    diagnostics: Option<String>,
}

impl RecoverOptions {
    fn from_matches(matches: &ArgMatches) -> CliResult<Self> {
        let output = matches
            .get_one::<String>("output")
            .map(|value| super::aliases::resolve(value).map(|(path, _)| path))
            .transpose()?;
        let overwrite = matches.get_flag("overwrite");
        if overwrite && output.is_none() {
            return Err(cli_error(
                "--overwrite requires --output; file recovery always creates a new directory",
            ));
        }
        let dry_run = matches.get_flag("dry-run");
        if dry_run && output.is_some() {
            return Err(
                Error::InvalidInput("--dry-run cannot be used with --output".to_string()).into(),
            );
        }
        if dry_run && overwrite {
            return Err(Error::InvalidInput(
                "--dry-run cannot be used with --overwrite".to_string(),
            )
            .into());
        }
        Ok(Self {
            lockbox_path: optional_lockbox_value(matches, "lockbox")?,
            directory: matches.get_one::<String>("directory").cloned(),
            output,
            overwrite,
            dry_run,
            quiet: matches.get_flag("quiet"),
            format: output_format_from_matches(matches)?,
            diagnostics: matches.get_one::<String>("diagnostics").cloned(),
        })
    }
}

fn default_recovery_directory(source: &Path) -> CliResult<PathBuf> {
    let source = std::path::absolute(source)?;
    let name = source
        .file_name()
        .ok_or_else(|| cli_error("recovery source has no file name"))?
        .to_string_lossy();
    let parent = source
        .parent()
        .ok_or_else(|| cli_error("recovery source has no parent directory"))?;
    for index in 0u64.. {
        let suffix = if index == 0 {
            String::new()
        } else {
            format!(".{index}")
        };
        let candidate = parent.join(format!("{name}.recovered-files{suffix}"));
        match fs::symlink_metadata(&candidate) {
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(candidate),
            Err(error) => return Err(error.into()),
        }
    }
    unreachable!()
}

fn inspect_pending_cleanup(
    lockbox_path: &str,
    access: &Access,
) -> CliResult<Option<TransactionRecoveryStatus>> {
    let path = Path::new(lockbox_path);
    Ok(Lockbox::inspect_transaction_recovery(
        path,
        recovery_open(lockbox_path, access)?,
    )?)
}

fn recover_pending_cleanup(
    lockbox_path: &str,
    access: &Access,
    status: &TransactionRecoveryStatus,
    quiet: bool,
) -> CliResult<()> {
    if !quiet {
        if status.phase == revault_lockbox_api::TransactionRecoveryPhase::Rollback {
            eprintln!("Lockbox rollback is required. Unpublished changes will be erased; the previous committed state will be restored.");
        } else if status.phase == revault_lockbox_api::TransactionRecoveryPhase::Truncate {
            eprintln!(
                "Lockbox tail reclamation is pending; the verified free suffix will be truncated."
            );
        } else {
            eprintln!(
            "Lockbox cleanup is required. The changes are already committed; cleanup is safe to interrupt and resume ({} of {} complete).",
            human_size(status.completed_bytes), human_size(status.total_bytes),
        );
        }
    }
    let mut last_percent = None;
    let recovered = Lockbox::recover_transaction(
        Path::new(lockbox_path),
        recovery_open(lockbox_path, access)?,
        |progress| {
            if !quiet {
                print_transaction_progress(progress, &mut last_percent);
            }
        },
    )?;
    if !quiet {
        if recovered {
            eprintln!("Lockbox cleanup complete: {lockbox_path}");
        } else {
            eprintln!("No interrupted cleanup is required: {lockbox_path}");
        }
    }
    Ok(())
}

/// Completes authenticated cleanup before an ordinary writable command opens
/// the lockbox, while making the automatic state change visible to the user.
pub(crate) fn complete_pending_cleanup_if_available(
    lockbox_path: &str,
    access: &Access,
) -> CliResult<()> {
    let open = if super::context::is_unencrypted(lockbox_path)? {
        Some(LockboxOpen::Unencrypted)
    } else {
        match access {
            Access::ContentKey(key) => Some(LockboxOpen::ContentKey(key.try_clone()?)),
            Access::CacheOnly => {
                cached_key_if_available(lockbox_path)?.map(LockboxOpen::ContentKey)
            }
            Access::PromptPassword => None,
        }
    };
    let Some(open) = open else {
        return Ok(());
    };
    let Some(status) = Lockbox::inspect_transaction_recovery(Path::new(lockbox_path), open)
        .map_err(|error| lockbox_open_error(lockbox_path, error))?
    else {
        return Ok(());
    };
    recover_pending_cleanup(lockbox_path, access, &status, false)
}

fn recovery_open<'a>(lockbox_path: &str, access: &'a Access) -> CliResult<LockboxOpen<'a>> {
    if super::context::is_unencrypted(lockbox_path)? {
        return Ok(LockboxOpen::Unencrypted);
    }
    match access {
        Access::ContentKey(key) => Ok(LockboxOpen::ContentKey(key.try_clone()?)),
        Access::CacheOnly => Ok(LockboxOpen::ContentKey(cached_key(lockbox_path)?)),
        Access::PromptPassword => Err(Error::InvalidInput(
            "doctor recover requires --key or an open lockbox".to_string(),
        )
        .into()),
    }
}

fn print_pending_cleanup(
    status: &TransactionRecoveryStatus,
    format: OutputFormat,
) -> CliResult<()> {
    print_records(
        &["field", "value"],
        vec![
            vec![
                "operation".to_string(),
                match status.phase {
                    revault_lockbox_api::TransactionRecoveryPhase::Rollback => {
                        "rollback_preparation"
                    }
                    revault_lockbox_api::TransactionRecoveryPhase::Truncate => "truncate_free_tail",
                    revault_lockbox_api::TransactionRecoveryPhase::Cleanup => {
                        "complete_pending_cleanup"
                    }
                }
                .to_string(),
            ],
            vec![
                "transaction_sequence".to_string(),
                status.transaction_sequence.to_string(),
            ],
            vec![
                "completed_pages".to_string(),
                status.completed_pages.to_string(),
            ],
            vec!["page_count".to_string(), status.page_count.to_string()],
            vec![
                "completed_ranges".to_string(),
                status.completed_ranges.to_string(),
            ],
            vec!["range_count".to_string(), status.range_count.to_string()],
            vec![
                "completed_bytes".to_string(),
                status.completed_bytes.to_string(),
            ],
            vec!["total_bytes".to_string(), status.total_bytes.to_string()],
        ],
        format,
    )
}

fn print_transaction_progress(
    progress: TransactionRecoveryProgress,
    last_percent: &mut Option<u32>,
) {
    let percent = if progress.total_bytes == 0 {
        100
    } else {
        ((progress.completed_bytes.saturating_mul(100) / progress.total_bytes).min(100)) as u32
    };
    if *last_percent != Some(percent) && (percent == 100 || percent % 5 == 0) {
        eprintln!(
            "Lockbox cleanup: {percent}% ({} of {}; safe to interrupt)",
            human_size(progress.completed_bytes),
            human_size(progress.total_bytes),
        );
        *last_percent = Some(percent);
    }
}

fn scan_report(lockbox_path: &str, access: &Access, quiet: bool) -> CliResult<RecoveryReport> {
    let bytes = read_recovery_bytes(lockbox_path, quiet)?;
    recovery_stage(quiet, "scanning readable encrypted records.");
    match access {
        Access::ContentKey(key) => scan_bytes_with_secret_key(bytes, key),
        Access::CacheOnly => {
            let key = cached_key(lockbox_path)?;
            scan_bytes_with_secret_key(bytes, &key)
        }
        Access::PromptPassword => {
            Err(Error::InvalidInput("recover requires --key or an open lockbox".to_string()).into())
        }
    }
}

fn read_recovery_bytes(lockbox_path: &str, quiet: bool) -> CliResult<Vec<u8>> {
    let size = fs::metadata(lockbox_path)?.len();
    recovery_stage(
        quiet,
        format!("reading {} from {lockbox_path}.", human_size(size)),
    );
    Ok(ScopedFileLock::read_archive(Path::new(lockbox_path))?.read_archive_bytes()?)
}

fn recovery_stage(quiet: bool, message: impl AsRef<str>) {
    if !quiet {
        eprintln!("Recovery: {}", message.as_ref());
    }
}

fn scan_bytes_with_secret_key(bytes: Vec<u8>, key: &SecretVec) -> CliResult<RecoveryReport> {
    let mut key_bytes = key.with_bytes(|key| key.to_vec())?;
    let report = RecoveryScanner::scan_bytes(bytes, &key_bytes);
    key_bytes.fill(0);
    Ok(report)
}

fn cached_key(lockbox_path: &str) -> CliResult<SecretVec> {
    let lockbox_id = VaultOpen::read_lockbox_id(Path::new(lockbox_path)).map_err(|_| {
        cli_error(format!(
            "cannot read lockbox id from {lockbox_path}; run `lockbox {lockbox_path} doctor recover --key <key>` for badly damaged headers"
        ))
    })?;
    get_cached_content_key(lockbox_id)?.ok_or_else(|| {
        cli_error(format!(
            "lockbox is closed: {lockbox_path}. Run `lockbox open {lockbox_path}` first or pass --key."
        ))
    })
}

fn cached_key_if_available(lockbox_path: &str) -> CliResult<Option<SecretVec>> {
    let Ok(lockbox_id) = VaultOpen::read_lockbox_id(Path::new(lockbox_path)) else {
        return Ok(None);
    };
    Ok(get_cached_content_key(lockbox_id).ok().flatten())
}

fn print_report(report: &RecoveryReport, format: OutputFormat) -> CliResult<()> {
    super::output::print_recovery_records(report_rows(report, None, None), format)
}

fn report_rows(
    report: &RecoveryReport,
    output: Option<&str>,
    damaged_original: Option<&Path>,
) -> Vec<Vec<String>> {
    let mut rows = vec![
        vec![
            "intact_file_count".to_string(),
            report.intact_file_count.to_string(),
        ],
        vec![
            "source_partial_files".to_string(),
            report.partial_files.to_string(),
        ],
        vec![
            "corrupt_records".to_string(),
            report.corrupt_records.to_string(),
        ],
        vec![
            "toc_recovered".to_string(),
            report.toc_recovered.to_string(),
        ],
        vec![
            "variables_recovered".to_string(),
            report.variables_recovered.to_string(),
        ],
        vec![
            "variable_count".to_string(),
            report.variable_count.to_string(),
        ],
        vec![
            "forms_recovered".to_string(),
            report.forms_recovered.to_string(),
        ],
        vec![
            "form_definition_count".to_string(),
            report.form_definition_count.to_string(),
        ],
        vec![
            "form_record_count".to_string(),
            report.form_record_count.to_string(),
        ],
    ];
    rows.push(vec![
        "partial_file_details".to_owned(),
        serde_json::Value::Array(
            report
                .partial_file_details
                .iter()
                .map(|(path, reason)| serde_json::json!({"path": path, "reason": reason}))
                .collect(),
        )
        .to_string(),
    ]);
    rows.push(vec![
        "unidentified_partial_files".to_owned(),
        report
            .partial_files
            .saturating_sub(report.partial_file_details.len())
            .to_string(),
    ]);
    if let Some(output) = output {
        rows.push(vec!["output".to_string(), output.to_string()]);
    }
    if let Some(damaged_original) = damaged_original {
        rows.push(vec![
            "damaged_original".to_string(),
            damaged_original.display().to_string(),
        ]);
    }
    rows
}

fn default_recovered_path(lockbox_path: &str) -> String {
    let path = Path::new(lockbox_path);
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("lockbox");
    parent
        .join(format!("{stem}.recovered.lbox"))
        .display()
        .to_string()
}

fn same_existing_path(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn next_damaged_backup_path(input_path: &Path) -> PathBuf {
    let parent = input_path.parent().unwrap_or_else(|| Path::new(""));
    let file_name = input_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("lockbox.lbox");
    let mut candidate = parent.join(format!("{file_name}.damaged"));
    let mut index = 1usize;
    while candidate.exists() {
        candidate = parent.join(format!("{file_name}.damaged.{index}"));
        index += 1;
    }
    candidate
}
