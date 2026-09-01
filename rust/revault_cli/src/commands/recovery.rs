use super::context::{cli_error, default_vault, lockbox_open_error, Access, CliResult};
use super::optional_lockbox_value;
use super::output::human_size;
use super::output::{output_format_from_matches, print_records, OutputFormat};
use clap::ArgMatches;
use revault_lockbox_api::vault_integration::VaultOpen;
use revault_lockbox_api::{Error, RecoveryReport, RecoveryScanner, SecretVec};
use revault_lockbox_api::{
    Lockbox, LockboxOpen, TransactionRecoveryProgress, TransactionRecoveryStatus,
};
use revault_vault_api::{get as get_cached_content_key, VaultDirectory};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn run_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let options = RecoverOptions::from_matches(matches)?;
    run_options(options, access)
}

fn run_options(options: RecoverOptions, access: &Access) -> CliResult<()> {
    match inspect_pending_cleanup(&options.lockbox_path, access) {
        Ok(Some(status)) => {
            if options.dry_run {
                print_pending_cleanup(&status, options.format)?;
                return Ok(());
            }
            return recover_pending_cleanup(&options.lockbox_path, access, &status, options.quiet);
        }
        Ok(None) => {}
        Err(err)
            if matches!(
                err.downcast_ref::<Error>(),
                Some(Error::CorruptHeader | Error::CorruptRecord | Error::Truncated)
            ) => {}
        Err(err) => return Err(err),
    }
    if options.dry_run {
        let report = scan_report(&options.lockbox_path, access, options.quiet)?;
        print_report(&report, options.format)?;
        return Ok(());
    }

    let output = options
        .output
        .clone()
        .unwrap_or_else(|| default_recovered_path(&options.lockbox_path));
    let output_path = Path::new(&output);
    let input_path = Path::new(&options.lockbox_path);
    let in_place = same_existing_path(input_path, output_path);
    if output_path.exists() && !options.overwrite {
        return Err(Error::AlreadyExists(output).into());
    }
    let bytes = read_recovery_bytes(&options.lockbox_path, options.quiet)?;
    recovery_stage(options.quiet, "scanning readable encrypted records.");
    let recovered = salvage_bytes(&options.lockbox_path, bytes, access)?;
    let damaged_original = if in_place {
        let backup = next_damaged_backup_path(input_path);
        fs::rename(input_path, &backup).map_err(|err| {
            Error::Io(format!(
                "move damaged lockbox {} to {}: {err}",
                options.lockbox_path,
                backup.display()
            ))
        })?;
        Some(backup)
    } else {
        None
    };
    fs::write(&output, recovered.try_to_bytes()?)
        .map_err(|err| Error::Io(format!("write recovered lockbox {output}: {err}")))?;
    let report = scan_report(&output, access, options.quiet)?;
    let rows = report_rows(&report, Some(&output), damaged_original.as_deref());
    print_records(&["field", "value"], rows, options.format)?;
    Ok(())
}

struct RecoverOptions {
    lockbox_path: String,
    output: Option<String>,
    overwrite: bool,
    dry_run: bool,
    quiet: bool,
    format: OutputFormat,
}

impl RecoverOptions {
    fn from_matches(matches: &ArgMatches) -> CliResult<Self> {
        let output = matches.get_one::<String>("output").cloned();
        let overwrite = matches.get_flag("overwrite");
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
            output,
            overwrite,
            dry_run,
            quiet: matches.get_flag("quiet"),
            format: output_format_from_matches(matches)?,
        })
    }
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
        eprintln!(
            "Lockbox cleanup is required. The changes are already committed; cleanup is safe to interrupt and resume ({} of {} complete).",
            human_size(status.completed_bytes), human_size(status.total_bytes),
        );
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
    let open = match access {
        Access::ContentKey(key) => Some(LockboxOpen::ContentKey(key.try_clone()?)),
        Access::CacheOnly => cached_key_if_available(lockbox_path)?.map(LockboxOpen::ContentKey),
        Access::PromptPassword => None,
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
                "complete_pending_cleanup".to_string(),
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
    fs::read(lockbox_path)
        .map_err(|err| Error::Io(format!("read lockbox {lockbox_path}: {err}")).into())
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

fn salvage_bytes(
    lockbox_path: &str,
    bytes: Vec<u8>,
    access: &Access,
) -> CliResult<revault_lockbox_api::Lockbox> {
    let signing_key = default_vault()?.load_owner_signing_key(VaultDirectory::DEFAULT_KEY_NAME)?;
    match access {
        Access::ContentKey(key) => Ok(RecoveryScanner::salvage_bytes_with_secret_key(
            bytes,
            key,
            &signing_key,
        )?),
        Access::CacheOnly => {
            let key = cached_key(lockbox_path)?;
            Ok(RecoveryScanner::salvage_bytes_with_secret_key(
                bytes,
                &key,
                &signing_key,
            )?)
        }
        Access::PromptPassword => {
            Err(Error::InvalidInput("recover requires --key or an open lockbox".to_string()).into())
        }
    }
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
    print_records(&["field", "value"], report_rows(report, None, None), format)
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
            "partial_files".to_string(),
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
