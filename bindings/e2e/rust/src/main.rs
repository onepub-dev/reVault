use revault_api::lockbox::{
    Lockbox, LockboxOpen, LockboxPath, LockboxProtection, OwnerSigningKeyPair, SecretString,
    SecretVec,
};
use revault_api::VaultDirectory;
use std::error::Error;
use std::path::PathBuf;

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

fn main() {
    if let Err(error) = run() {
        eprintln!("Rust conformance failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result {
    std::env::set_var("LOCKBOX_PLATFORM_SECRET_STORE", "disabled");
    let arguments: Vec<_> = std::env::args().collect();
    if arguments.len() == 3 && arguments[1] == "--interop" {
        return open_foreign(&arguments[2]);
    }
    create_artifacts()
}

fn artifact_root(language: &str) -> Result<PathBuf> {
    let base = std::env::var_os("REVAULT_E2E_ARTIFACT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("revault-e2e-artifacts"));
    let root = base.join(language);
    std::fs::create_dir_all(&root)?;
    Ok(root)
}

fn content_key() -> Result<SecretVec> {
    Ok(SecretVec::try_from_slice(&[b'K'; 32])?)
}

fn password() -> Result<SecretString> {
    Ok(SecretString::try_from_bytes(
        b"new vault password".to_vec(),
    )?)
}

fn create_artifacts() -> Result {
    let root = artifact_root("rust")?;
    let archive = root.join("archive.lbox");
    let vault_root = root.join("vault");
    let _ = std::fs::remove_file(&archive);
    let _ = std::fs::remove_dir_all(&vault_root);
    let signing = OwnerSigningKeyPair::generate()?;
    let mut lockbox = Lockbox::create_file(
        &archive,
        LockboxProtection::ContentKey(content_key()?),
        &signing,
    )?;
    lockbox.add_file(
        &LockboxPath::new("/renamed.txt")?,
        b"replacement payload",
        false,
    )?;
    lockbox.commit()?;
    drop(lockbox);
    println!("ARTIFACT\trust\tarchive-created\t{}", archive.display());
    let opened = Lockbox::open(&archive, LockboxOpen::ContentKey(content_key()?))?;
    if opened.get_file(&LockboxPath::new("/renamed.txt")?)? != b"replacement payload" {
        return Err("Rust archive content mismatch".into());
    }
    println!("ARTIFACT\trust\tarchive-opened\t{}", archive.display());

    let vault = VaultDirectory::replace(&vault_root, &password()?)?;
    if vault.structure_version()? == 0 {
        return Err("Rust vault structure version is zero".into());
    }
    drop(vault);
    println!("ARTIFACT\trust\tvault-created\t{}", vault_root.display());
    let reopened = VaultDirectory::open_or_create(&vault_root, &password()?)?;
    if reopened.structure_version()? == 0 {
        return Err("Rust reopened vault structure version is zero".into());
    }
    println!("ARTIFACT\trust\tvault-opened\t{}", vault_root.display());
    Ok(())
}

fn open_foreign(producer: &str) -> Result {
    let root = artifact_root(producer)?;
    let archive = root.join("archive.lbox");
    let opened = Lockbox::open(&archive, LockboxOpen::ContentKey(content_key()?))?;
    if opened.get_file(&LockboxPath::new("/renamed.txt")?)? != b"replacement payload" {
        return Err("foreign archive content mismatch".into());
    }
    let vault_root = root.join("vault");
    if !vault_root.join("local-vault.lbox").is_file() {
        return Err("foreign vault artifact is missing".into());
    }
    let vault = VaultDirectory::open_or_create(&vault_root, &password()?)?;
    if vault.structure_version()? == 0 {
        return Err("foreign vault structure version is zero".into());
    }
    println!("INTEROP\trust\t{producer}\tarchive\t3");
    println!("INTEROP\trust\t{producer}\tvault\t2");
    Ok(())
}
