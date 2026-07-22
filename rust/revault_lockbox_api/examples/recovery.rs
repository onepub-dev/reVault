use revault_lockbox_api::{
    Lockbox, LockboxPath, LockboxProtection, OwnerSigningKeyPair, RecoveryReportOptions,
    RecoveryScanner, SecretVec,
};

fn main() -> revault_lockbox_api::Result<()> {
    let root = example_root("recovery")?;
    let lockbox_file = root.join("recovery.lbox");
    let signing_key = OwnerSigningKeyPair::generate()?;
    let content_key = random_content_key()?;

    let mut lockbox = Lockbox::create_file(
        &lockbox_file,
        LockboxProtection::ContentKey(content_key.try_clone()?),
        &signing_key,
    )?;
    lockbox.create_dir(&path("/docs")?, false)?;
    lockbox.add_file(&path("/docs/a.txt")?, b"alpha", false)?;
    lockbox.add_file(&path("/docs/b.txt")?, b"bravo", false)?;
    lockbox.commit()?;

    let mut damaged = std::fs::read(&lockbox_file)
        .map_err(|err| revault_lockbox_api::Error::Io(err.to_string()))?;
    damaged[0] ^= 0xff;

    let report = content_key.with_bytes(|key| RecoveryScanner::scan_bytes(damaged.clone(), key))?;
    println!("{}", report.render(&RecoveryReportOptions::default()));

    let salvaged =
        RecoveryScanner::salvage_bytes_with_secret_key(damaged, &content_key, &signing_key)?;
    println!(
        "recovered /docs/a.txt: {}",
        String::from_utf8_lossy(&salvaged.get_file(&path("/docs/a.txt")?)?)
    );
    Ok(())
}

fn random_content_key() -> revault_lockbox_api::Result<SecretVec> {
    let mut key = SecretVec::new();
    key.resize_zeroed(32)?;
    key.with_mut_bytes(|bytes| {
        getrandom::fill(bytes).map_err(|err| revault_lockbox_api::Error::Io(err.to_string()))
    })??;
    Ok(key)
}

fn path(value: &str) -> revault_lockbox_api::Result<LockboxPath> {
    LockboxPath::new(value)
}

fn example_root(name: &str) -> revault_lockbox_api::Result<std::path::PathBuf> {
    let root = std::env::temp_dir()
        .join("lockbox-core-examples")
        .join(name);
    match std::fs::remove_dir_all(&root) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(revault_lockbox_api::Error::Io(err.to_string())),
    }
    std::fs::create_dir_all(&root)
        .map_err(|err| revault_lockbox_api::Error::Io(err.to_string()))?;
    Ok(root)
}
