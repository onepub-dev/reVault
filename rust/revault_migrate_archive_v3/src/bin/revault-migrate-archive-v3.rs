use revault_lockbox_api::{
    ContactKeyPair, Lockbox, LockboxOpen, ReadOnly, SecretString, SecretVec,
};
use revault_migrate_archive_v3::export_archive;
use sha2::{Digest, Sha256};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use zeroize::Zeroizing;

const IPC_MAGIC: &[u8; 8] = b"LBXMIPC1";
const MAX_SECRET_BYTES: usize = 1024 * 1024;
const MAX_SECRET_COUNT: usize = 4096;

fn main() {
    // Windows executables start with a smaller main-thread stack than Rust
    // worker threads. Historical key decoding includes stack-heavy
    // cryptographic validation, so perform the export on an explicitly sized
    // stack on every platform for consistent behaviour.
    let worker = std::thread::Builder::new()
        .name("archive-v3-export".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| run().map_err(|err| err.to_string()));
    let result = match worker {
        Ok(worker) => worker
            .join()
            .unwrap_or_else(|_| Err("historical migration worker panicked".to_string())),
        Err(err) => Err(format!(
            "failed to start historical migration worker: {err}"
        )),
    };
    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.as_slice() == ["capabilities"] {
        println!(
            "{{\"protocol\":3,\"artifact\":\"archive\",\"native_version\":3,\"migration_schema\":3}}"
        );
        return Ok(());
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
    export_archive(&lockbox, &output, artifact_bytes.as_slice(), random_id()?)?;
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
    for record in contact_key_records {
        let recovery_key = ContactKeyPair::from_private_key_record(record.try_clone()?)?;
        if Lockbox::recover_transaction(&copy, LockboxOpen::ContactKeyPair(recovery_key), |_| {})
            .is_err()
        {
            continue;
        }
        let key = ContactKeyPair::from_private_key_record(record)?;
        if let Ok(lockbox) = Lockbox::open(&copy, LockboxOpen::ContactKeyPair(key)) {
            return Ok(lockbox);
        }
    }
    if Lockbox::recover_transaction(&copy, LockboxOpen::Unencrypted, |_| {}).is_ok() {
        return Ok(Lockbox::open(&copy, LockboxOpen::Unencrypted)?);
    }
    if let Some(password) = SecretString::try_from_env("LOCKBOX_PASSWORD")? {
        Lockbox::recover_transaction(&copy, LockboxOpen::Password(&password), |_| {})?;
        return Ok(Lockbox::open(&copy, LockboxOpen::Password(&password))?);
    }

    Err(
        "no vault Profile key can open the archive; if it is password-only, set LOCKBOX_PASSWORD"
            .into(),
    )
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

fn random_id() -> Result<[u8; 16], revault_migration_format::MigrationError> {
    let mut id = [0u8; 16];
    getrandom::fill(&mut id)
        .map_err(|err| revault_migration_format::MigrationError::Io(err.to_string()))?;
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
