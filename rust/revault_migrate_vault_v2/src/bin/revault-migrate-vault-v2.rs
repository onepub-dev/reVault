use revault_lockbox_api_v2::SecretString;
use revault_migrate_vault_v2::export_vault_v2;
use revault_vault_api_v2::{default_vault_dir, VaultDirectory};
use std::env;
use std::io::Read;
use std::path::PathBuf;

const IPC_MAGIC: &[u8; 8] = b"LBXMIPC1";
const MAX_SECRET_BYTES: usize = 1024 * 1024;

fn main() {
    let result = std::thread::Builder::new()
        .name("vault-v2-export".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| run().map_err(|error| error.to_string()))
        .and_then(|worker| {
            worker
                .join()
                .map_err(|_| std::io::Error::other("historical migration worker panicked"))
        });
    match result {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.as_slice() == ["capabilities"] {
        println!(
            "{{\"protocol\":1,\"artifact\":\"vault\",\"native_version\":2,\"migration_schema\":2}}"
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
    if !source.join("local-vault.lbox").is_file() {
        return Err(format!("vault does not exist at {}", source.display()).into());
    }
    let mut secrets = read_secret_frame(2)?.into_iter();
    let vault_password = secrets.next().expect("framed vault password");
    let artifact_password = secrets.next().expect("framed artifact password");
    let vault = VaultDirectory::open_or_create(&source, &vault_password)?;
    let result = artifact_password.with_bytes(|passphrase| {
        export_vault_v2(&vault, &output, passphrase, random_id()?)
    })??;
    let _ = result;
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
    let mut magic = [0_u8; 8];
    input.read_exact(&mut magic)?;
    if &magic != IPC_MAGIC {
        return Err("invalid migration IPC protocol".into());
    }
    let mut count_bytes = [0_u8; 4];
    input.read_exact(&mut count_bytes)?;
    if u32::from_le_bytes(count_bytes) as usize != count {
        return Err("unexpected migration secret count".into());
    }
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let mut len_bytes = [0_u8; 4];
        input.read_exact(&mut len_bytes)?;
        let len = u32::from_le_bytes(len_bytes) as usize;
        if len == 0 || len > MAX_SECRET_BYTES {
            return Err("invalid migration secret length".into());
        }
        let mut bytes = vec![0_u8; len];
        input.read_exact(&mut bytes)?;
        values.push(SecretString::try_from_bytes(bytes)?);
    }
    Ok(values)
}

fn random_id() -> revault_migration_format::Result<[u8; 16]> {
    let mut id = [0_u8; 16];
    getrandom::fill(&mut id)
        .map_err(|error| revault_migration_format::MigrationError::Io(error.to_string()))?;
    Ok(id)
}
