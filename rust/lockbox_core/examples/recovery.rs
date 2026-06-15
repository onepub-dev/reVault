use lockbox_core::{
    Lockbox, LockboxPath, LockboxProtection, OwnerSigningKeyPair, RecoveryReportOptions,
    RecoveryScanner, SecretVec,
};

const CONTENT_KEY: &[u8] = b"example recovery content key";

fn main() -> lockbox_core::Result<()> {
    let root = example_root("recovery")?;
    let lockbox_file = root.join("recovery.lbox");
    let signing_key = OwnerSigningKeyPair::generate()?;

    let mut lockbox = Lockbox::create_file(
        &lockbox_file,
        LockboxProtection::ContentKey(SecretVec::try_from_slice(CONTENT_KEY)?),
        &signing_key,
    )?;
    lockbox.add_file(&path("/docs/a.txt")?, b"alpha", false)?;
    lockbox.add_file(&path("/docs/b.txt")?, b"bravo", false)?;
    lockbox.commit()?;

    let mut damaged =
        std::fs::read(&lockbox_file).map_err(|err| lockbox_core::Error::Io(err.to_string()))?;
    damaged[0] ^= 0xff;

    let report = RecoveryScanner::scan_bytes(damaged.clone(), CONTENT_KEY);
    println!("{}", report.render(&RecoveryReportOptions::default()));

    let salvaged = RecoveryScanner::salvage_bytes(damaged, CONTENT_KEY, &signing_key)?;
    println!(
        "recovered /docs/a.txt: {}",
        String::from_utf8_lossy(&salvaged.get_file(&path("/docs/a.txt")?)?)
    );
    Ok(())
}

fn path(value: &str) -> lockbox_core::Result<LockboxPath> {
    LockboxPath::new(value)
}

fn example_root(name: &str) -> lockbox_core::Result<std::path::PathBuf> {
    let root = std::env::temp_dir()
        .join("lockbox-core-examples")
        .join(name);
    match std::fs::remove_dir_all(&root) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(lockbox_core::Error::Io(err.to_string())),
    }
    std::fs::create_dir_all(&root).map_err(|err| lockbox_core::Error::Io(err.to_string()))?;
    Ok(root)
}
