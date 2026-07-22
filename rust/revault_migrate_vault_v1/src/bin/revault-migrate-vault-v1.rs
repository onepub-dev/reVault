use revault_lockbox_api_v1::SecretString;
use revault_migrate_vault_v1::export_vault_v1;
use revault_vault_api_v1::default_vault_dir;
use std::env;
use std::io::Read;
use std::path::PathBuf;
use zeroize::Zeroizing;

const IPC_MAGIC: &[u8; 8] = b"LBXMIPC1";
const MAX_SECRET_BYTES: usize = 1024 * 1024;

fn main() {
    // Windows executables start with a smaller main-thread stack than Rust
    // worker threads. Historical key decoding includes stack-heavy
    // cryptographic validation, so perform the export on an explicitly sized
    // stack on every platform for consistent behaviour.
    let worker = std::thread::Builder::new()
        .name("vault-v1-export".to_string())
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
            "{{\"protocol\":1,\"artifact\":\"vault\",\"native_version\":1,\"migration_schema\":1}}"
        );
        return Ok(());
    }
    if !args.starts_with(&["migrate".into(), "vault".into(), "export".into()]) {
        return Err("unsupported historical migration command".into());
    }
    let output = option(&args, "--output")
        .map(PathBuf::from)
        .ok_or("historical export requires --output")?;
    let source = option(&args, "--source")
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(default_vault_dir)?;
    let mut secrets = read_secret_frame(2)?.into_iter();
    let vault_password = secrets.next().expect("framed vault password");
    let artifact_password = secrets.next().expect("framed artifact password");
    let vault_bytes = vault_password.with_bytes(|bytes| Zeroizing::new(bytes.to_vec()))?;
    let artifact_bytes = artifact_password.with_bytes(|bytes| Zeroizing::new(bytes.to_vec()))?;
    export_vault_v1(
        &source,
        &vault_bytes,
        &output,
        &artifact_bytes,
        random_id()?,
    )?;
    Ok(())
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
