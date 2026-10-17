use crate::archive_reader::{
    ContactKeyPair, Lockbox, LockboxOpen, ReadOnly, SecretString, SecretVec,
};
use crate::export_archive::export_archive;
use sha2::{Digest, Sha256};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use zeroize::Zeroizing;

const IPC_MAGIC: &[u8; 8] = b"LBXMIPC1";
const MAX_SECRET_BYTES: usize = 1024 * 1024;
const MAX_SECRET_COUNT: usize = 4096;

pub(super) fn main() {
    // Windows executables start with a smaller main-thread stack than Rust
    // worker threads. Historical key decoding includes stack-heavy
    // cryptographic validation, so perform the export on an explicitly sized
    // stack on every platform for consistent behaviour.
    let worker = std::thread::Builder::new()
        .name("archive-v3-export".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            run().map_err(|err| {
                let status = if matches!(
                    err.downcast_ref::<current_migration_format::MigrationError>(),
                    Some(current_migration_format::MigrationError::SourceCorrupt(_))
                ) {
                    42
                } else {
                    1
                };
                (status, err.to_string())
            })
        });
    let result = match worker {
        Ok(worker) => worker
            .join()
            .unwrap_or_else(|_| Err((1, "historical migration worker panicked".to_string()))),
        Err(err) => Err((
            1,
            format!("failed to start historical migration worker: {err}"),
        )),
    };
    if let Err((status, err)) = result {
        eprintln!("error: {err}");
        std::process::exit(status);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.as_slice() == ["capabilities"] {
        println!(
            "{{\"protocol\":3,\"artifact\":\"archive\",\"native_version\":2,\"migration_schema\":3,\"recovery\":true,\"recovery_directory_protocol\":1}}"
        );
        return Ok(());
    }
    if args.starts_with(&["migrate".into(), "archive".into(), "recover".into()]) {
        return recover(&args).map_err(|error| {
            use crate::archive_reader::Error;
            let category = match error.downcast_ref::<Error>() {
                Some(Error::InvalidKey) => "authentication",
                Some(Error::CorruptHeader | Error::CorruptRecord | Error::Truncated) => "integrity",
                Some(Error::InvalidOperation(_)) => "mutation",
                Some(Error::Io(_)) => "io",
                _ => "other",
            };
            eprintln!("REVAULT_RECOVERY_EVENT {}", serde_json::json!({"stage":"helper_failed","error_category":category,"version":env!("CARGO_PKG_VERSION")}));
            error
        });
    }
    if !args.starts_with(&["migrate".into(), "archive".into(), "export".into()]) {
        return Err("unsupported historical migration command".into());
    }
    let source = args.get(3).ok_or("archive export requires a source")?;
    let output = option(&args, "--output")
        .map(PathBuf::from)
        .ok_or("historical export requires --output")?;
    let source_fingerprint = fingerprint_file(&PathBuf::from(source))?;
    let mut secrets = read_secret_frame()?;
    if secrets.len() < 2 {
        return Err("historical archive export requires two framed passwords".into());
    }
    let contact_keys = secrets.split_off(2);
    let artifact_password = SecretString::from_secure_vec(secrets.pop().unwrap());
    let vault_password = SecretString::from_secure_vec(secrets.pop().unwrap());
    let artifact_bytes = artifact_password.with_bytes(|bytes| Zeroizing::new(bytes.to_vec()))?;
    let working = tempfile::tempdir()?;
    std::fs::copy(source, working.path().join("source.lbox"))?;
    let lockbox = open_working_copy(&working, &vault_password, contact_keys)?;
    if lockbox.format_version() != 2 {
        return Err("exporter requires archive format 2".into());
    }
    export_archive(&lockbox, &output, artifact_bytes.as_slice(), random_id()?).map_err(
        |error| {
            // Opening succeeded: authentication failures while reading reachable
            // payloads indicate source damage, not an incorrect opening password.
            let message = error.to_string();
            if message.contains("payload authentication failed")
                || message.contains("corrupt")
                || message.contains("truncated")
            {
                current_migration_format::MigrationError::SourceCorrupt(message)
            } else {
                error
            }
        },
    )?;
    if fingerprint_file(&PathBuf::from(source))? != source_fingerprint {
        return Err("v2/v3 archive changed while it was being exported".into());
    }
    Ok(())
}

fn open_working_copy(
    working: &tempfile::TempDir,
    _vault_password: &SecretString,
    contact_key_records: Vec<SecretVec>,
) -> Result<Lockbox<ReadOnly>, Box<dyn std::error::Error>> {
    // This helper accepts only our private temporary directory. The original
    // archive path is never passed to any recovery or write-capable API.
    let copy = working.path().join("source.lbox");
    if let Some(key) = SecretString::try_from_env("LOCKBOX_KEY")? {
        let mut bytes = SecretVec::new();
        key.append_to_secure_vec(&mut bytes)?;
        let key = bytes;
        if Lockbox::recover_transaction(&copy, LockboxOpen::ContentKey(key.try_clone()?), |_| {})
            .is_ok()
        {
            return Ok(Lockbox::open(&copy, LockboxOpen::ContentKey(key))?);
        }
    }
    for record in contact_key_records {
        let recovery_key = ContactKeyPair::from_private_key_record(record.try_clone()?)?;
        if crate::archive_reader::vault_integration::VaultOpen::path_with_contact(
            &copy,
            &recovery_key,
        )
        .is_err()
        {
            continue;
        }
        Lockbox::recover_transaction(&copy, LockboxOpen::ContactKeyPair(recovery_key), |_| {})
            .map_err(authenticated_source_error)?;
        let key = ContactKeyPair::from_private_key_record(record)?;
        return Lockbox::open(&copy, LockboxOpen::ContactKeyPair(key))
            .map_err(authenticated_source_error);
    }
    if Lockbox::recover_transaction(&copy, LockboxOpen::Unencrypted, |_| {}).is_ok() {
        return Ok(Lockbox::open(&copy, LockboxOpen::Unencrypted)?);
    }
    if let Some(password) = SecretString::try_from_env("LOCKBOX_PASSWORD")? {
        crate::archive_reader::vault_integration::VaultOpen::path_with_password(&copy, &password)?;
        Lockbox::recover_transaction(&copy, LockboxOpen::Password(&password), |_| {})
            .map_err(authenticated_source_error)?;
        return Lockbox::open(&copy, LockboxOpen::Password(&password))
            .map_err(authenticated_source_error);
    }

    Err(
        "no vault Profile key can open the archive; if it is password-only, set LOCKBOX_PASSWORD"
            .into(),
    )
}

fn authenticated_source_error(error: crate::archive_reader::Error) -> Box<dyn std::error::Error> {
    use crate::archive_reader::Error;
    if matches!(
        error,
        Error::CorruptHeader | Error::CorruptRecord | Error::Truncated | Error::InvalidKey
    ) {
        Box::new(current_migration_format::MigrationError::SourceCorrupt(
            error.to_string(),
        ))
    } else {
        Box::new(error)
    }
}

fn recovery_event(stage: &'static str, progress: Option<(u64, u64)>) {
    let mut event = serde_json::json!({"stage":stage,"version":env!("CARGO_PKG_VERSION")});
    if let Some((done, total)) = progress {
        event["completed_files"] = done.into();
        event["total_files"] = total.into();
    }
    eprintln!("REVAULT_RECOVERY_EVENT {event}");
}

fn recover(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    use crate::archive_reader::vault_integration::VaultOpen;
    use crate::archive_reader::{OwnerSigningKeyPair, RecoveryScanner, ScopedFileLock};
    let source = PathBuf::from(args.get(3).ok_or("recovery requires a source")?);
    let output =
        PathBuf::from(option(args, "--output").ok_or("recovery requires an artifact output")?);
    let signing_count: usize = option(args, "--signing-key-count")
        .ok_or("recovery requires owner keys")?
        .parse()?;
    let mut secrets = read_secret_frame()?.into_iter();
    let _vault_password = secrets.next().ok_or("missing framed vault password")?;
    let artifact_password = secrets.next().ok_or("missing framed artifact password")?;
    let raw_key = match option(args, "--raw-key") {
        Some("true") => Some(secrets.next().ok_or("missing framed raw key")?),
        Some("false") => None,
        _ => return Err("recovery requires explicit raw-key presence".into()),
    };
    let mut signing_keys = Vec::new();
    for _ in 0..signing_count {
        signing_keys.push(OwnerSigningKeyPair::from_private_key_record(
            secrets.next().ok_or("missing owner key")?,
        )?);
    }
    recovery_event("reading_source", None);
    let guard = ScopedFileLock::read_archive(&source)?;
    let bytes = guard.read_archive_bytes()?;
    if crate::archive_reader::probe_lockbox_format_version(&bytes)? != 2 {
        return Err("recovery exporter requires archive format 2".into());
    }
    let working = tempfile::tempdir()?;
    let copy = working.path().join("source.lbox");
    std::fs::write(&copy, &bytes)?;
    recovery_event("unlocking", None);
    let mut key = raw_key;
    for record in secrets {
        if key.is_some() {
            break;
        }
        let contact = ContactKeyPair::from_private_key_record(record)?;
        if let Ok(opened) = VaultOpen::path_with_contact(&copy, &contact) {
            key = Some(opened.try_clone_key()?);
        }
    }
    if key.is_none() {
        if let Some(password) = SecretString::try_from_env("LOCKBOX_PASSWORD")? {
            key = Some(VaultOpen::path_with_password(&copy, &password)?.try_clone_key()?);
        }
    }
    let key = key.ok_or("no supplied credential can unlock the historical access directory")?;
    let raw = key.with_bytes(|bytes| Zeroizing::new(bytes.to_vec()))?;
    let expected = RecoveryScanner::owner_fingerprint_for_recovery(&bytes, &raw)?;
    if args.iter().any(|arg| arg == "--recovery-directory") {
        use current_migration_format::{ArtifactKind, ArtifactWriter, MigrationHeader};
        let dry_run = args.iter().any(|arg| arg == "--dry-run");
        let password = artifact_password.with_bytes(|bytes| Zeroizing::new(bytes.to_vec()))?;
        let mut writer = if dry_run {
            None
        } else {
            Some(ArtifactWriter::new(
                std::io::BufWriter::new(
                    std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&output)?,
                ),
                MigrationHeader {
                    artifact_kind: ArtifactKind::Journal,
                    source_native_version: 2,
                    migration_schema_version: 3,
                    target_native_version: None,
                    operation_id: random_id()?,
                },
                &password,
            )?)
        };
        let plan = RecoveryScanner::recover_ranges(&bytes, &raw, |id, offset, data| {
            if let Some(writer) = writer.as_mut() {
                let mut frame = Zeroizing::new(Vec::with_capacity(16 + data.len()));
                frame.extend_from_slice(&id.to_le_bytes());
                frame.extend_from_slice(&offset.to_le_bytes());
                frame.extend_from_slice(data);
                writer
                    .write_raw(&frame)
                    .map_err(|error| crate::archive_reader::Error::Io(error.to_string()))?;
            }
            Ok(())
        })?;
        if let Some(mut writer) = writer {
            writer.write_json(&plan)?;
            writer.finish()?;
        }
        println!("{plan}");
        return Ok(());
    }
    recovery_event("scanning_records", None);
    let (report, partial_file_details) = RecoveryScanner::report_with_file_details(&bytes, &raw);
    let mut report_json = serde_json::json!({
        "intact_files": report.intact_file_count,
        "partial_files": report.partial_files,
        "corrupt_records": report.corrupt_records,
        "toc_recovered": report.toc_recovered,
        "partial_file_details": partial_file_details.iter().map(|(path, reason)| serde_json::json!({"path": path, "reason": reason})).collect::<Vec<_>>(),
        "unidentified_partial_files": report.partial_files.saturating_sub(partial_file_details.len()),
    });
    if args.iter().any(|arg| arg == "--dry-run") {
        report_json["file_recovery_plan"] = RecoveryScanner::plan_file_fragments_and_progress(
            &bytes,
            &raw,
            None,
            |_, _, _| Ok(()),
            |done, total| recovery_event("verifying_files", Some((done, total))),
        )?;
        println!("{report_json}");
        return Ok(());
    }
    let signer = signing_keys
        .into_iter()
        .find(|candidate| {
            RecoveryScanner::signing_key_fingerprint(candidate)
                .is_ok_and(|fingerprint| fingerprint == expected)
        })
        .ok_or("the established archive owner's signing key is unavailable")?;
    let artifact_password = artifact_password.with_bytes(|bytes| Zeroizing::new(bytes.to_vec()))?;
    let partial_output =
        option(args, "--partial-output").ok_or("recovery requires an encrypted partial output")?;
    let mut partial_writer = current_migration_format::ArtifactWriter::new(
        std::io::BufWriter::new(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(partial_output)?,
        ),
        current_migration_format::MigrationHeader {
            artifact_kind: current_migration_format::ArtifactKind::Journal,
            source_native_version: 2,
            migration_schema_version: 3,
            target_native_version: None,
            operation_id: random_id()?,
        },
        &artifact_password,
    )?;
    recovery_event("restoring_archive", None);
    let (recovered, plan) = RecoveryScanner::salvage_with_file_fragments_and_progress(
        &bytes,
        &raw,
        &signer,
        |id, offset, data| {
            let mut frame = Zeroizing::new(Vec::with_capacity(16 + data.len()));
            frame.extend_from_slice(&id.to_le_bytes());
            frame.extend_from_slice(&offset.to_le_bytes());
            frame.extend_from_slice(data);
            partial_writer
                .write_raw(&frame)
                .map_err(|error| crate::archive_reader::Error::Io(error.to_string()))
        },
        |done, total| recovery_event("verifying_files", Some((done, total))),
    )?;
    partial_writer.write_json(&plan)?;
    partial_writer.finish()?;
    report_json["file_recovery_plan"] = plan;
    recovery_event("exporting_archive", None);
    export_archive(
        &recovered,
        &output,
        artifact_password.as_slice(),
        random_id()?,
    )?;
    println!("{report_json}");
    Ok(())
}

fn option<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|value| value == name)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

fn read_secret_frame() -> Result<Vec<SecretVec>, Box<dyn std::error::Error>> {
    let mut input = std::io::stdin().lock();
    let mut magic = [0u8; 8];
    input.read_exact(&mut magic)?;
    if &magic != IPC_MAGIC {
        return Err("invalid migration IPC protocol".into());
    }
    let mut count_bytes = [0u8; 4];
    input.read_exact(&mut count_bytes)?;
    let count = u32::from_le_bytes(count_bytes) as usize;
    if !(2..=MAX_SECRET_COUNT).contains(&count) {
        return Err("unexpected migration secret count".into());
    }
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let mut len_bytes = [0u8; 4];
        input.read_exact(&mut len_bytes)?;
        let len = u32::from_le_bytes(len_bytes) as usize;
        if len == 0 || len > MAX_SECRET_BYTES {
            return Err("invalid migration secret length".into());
        }
        let mut bytes = vec![0u8; len];
        input.read_exact(&mut bytes)?;
        values.push(SecretVec::try_from_vec(bytes)?);
    }
    Ok(values)
}

fn random_id() -> Result<[u8; 16], current_migration_format::MigrationError> {
    let mut id = [0u8; 16];
    getrandom::fill(&mut id)
        .map_err(|err| current_migration_format::MigrationError::Io(err.to_string()))?;
    Ok(id)
}

fn fingerprint_file(path: &std::path::Path) -> Result<[u8; 32], Box<dyn std::error::Error>> {
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
