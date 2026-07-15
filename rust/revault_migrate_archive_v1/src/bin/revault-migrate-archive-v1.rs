use revault_lockbox_api::{Lockbox, LockboxOpen, ReadOnly, SecretString};
use revault_migrate_archive_v1::export_archive_v1;
use revault_vault_api::{local_vault, VaultDirectory};
use sha2::{Digest, Sha256};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use zeroize::Zeroizing;

const IPC_MAGIC: &[u8; 8] = b"LBXMIPC1";
const MAX_SECRET_BYTES: usize = 1024 * 1024;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.as_slice() == ["capabilities"] {
        println!(
            "{{\"protocol\":2,\"artifact\":\"archive\",\"native_version\":1,\"migration_schema\":1}}"
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
    let mut secrets = read_secret_frame(2)?;
    let artifact_password = secrets.pop().expect("framed artifact password");
    let vault_password = secrets.pop().expect("framed vault password");
    let artifact_bytes = artifact_password.with_bytes(|bytes| Zeroizing::new(bytes.to_vec()))?;
    let lockbox = open_archive(source, &vault_password)?;
    export_archive_v1(&lockbox, &output, &artifact_bytes, random_id()?)?;
    if fingerprint_file(&PathBuf::from(source))? != source_fingerprint {
        return Err("v1 archive changed while it was being exported".into());
    }
    Ok(())
}

fn open_archive(
    source: &str,
    vault_password: &SecretString,
) -> Result<Lockbox<ReadOnly>, Box<dyn std::error::Error>> {
    if let Ok(lockbox) = local_vault().open_lockbox_read_only(source) {
        return Ok(lockbox);
    }

    let vault = VaultDirectory::open_or_create_default(vault_password)?;
    for profile in vault.list_private_keys()? {
        let history = vault.list_profile_generations(&profile)?;
        for generation in history.generations {
            let key = vault.load_private_key_generation(&profile, generation.index)?;
            if let Ok(lockbox) =
                Lockbox::open(&PathBuf::from(source), LockboxOpen::ContactKeyPair(key))
            {
                return Ok(lockbox);
            }
        }
    }

    if let Some(password) = SecretString::try_from_env("LOCKBOX_PASSWORD")? {
        return Ok(Lockbox::open(
            &PathBuf::from(source),
            LockboxOpen::Password(&password),
        )?);
    }

    Err(
        "no vault profile key can open the archive; if it is password-only, set LOCKBOX_PASSWORD"
            .into(),
    )
}

fn option<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|value| value == name)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

fn read_secret_frame(count: usize) -> Result<Vec<SecretString>, Box<dyn std::error::Error>> {
    let mut input = std::io::stdin().lock();
    let mut magic = [0u8; 8];
    input.read_exact(&mut magic)?;
    if &magic != IPC_MAGIC {
        return Err("invalid migration IPC protocol".into());
    }
    let mut count_bytes = [0u8; 4];
    input.read_exact(&mut count_bytes)?;
    if u32::from_le_bytes(count_bytes) as usize != count {
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
        values.push(SecretString::try_from_bytes(bytes)?);
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
