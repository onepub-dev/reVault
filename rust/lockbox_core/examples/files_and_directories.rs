use lockbox_core::{
    ListOptions, Lockbox, LockboxOpen, LockboxPath, LockboxProtection, OwnerSigningKeyPair,
    SecretString,
};

fn main() -> lockbox_core::Result<()> {
    let root = example_root("files-and-directories")?;
    let host_file = root.join("host-note.txt");
    let lockbox_file = root.join("files.lbox");
    std::fs::write(&host_file, b"stored from the host filesystem")
        .map_err(|err| lockbox_core::Error::Io(err.to_string()))?;

    let pass_phrase = pass_phrase()?;
    let signing_key = OwnerSigningKeyPair::generate()?;
    let mut lockbox = Lockbox::create_file(
        &lockbox_file,
        LockboxProtection::Password(&pass_phrase),
        &signing_key,
    )?;

    lockbox.add_file(&path("/docs/readme.txt")?, b"stored from memory", false)?;
    lockbox.add_file_from_path(&host_file, &path("/docs/host-note.txt")?, false)?;
    lockbox.commit()?;

    let opened = Lockbox::open_file(&lockbox_file, LockboxOpen::Password(&pass_phrase))?;
    for entry in opened.list(ListOptions {
        recursive: true,
        ..ListOptions::new(&path("/docs")?)
    })? {
        let entry = entry?;
        println!("{} ({} bytes)", entry.path, entry.len);
    }

    let bytes = opened.get_file(&path("/docs/readme.txt")?)?;
    println!("{}", String::from_utf8_lossy(&bytes));
    Ok(())
}

fn path(value: &str) -> lockbox_core::Result<LockboxPath> {
    LockboxPath::new(value)
}

fn pass_phrase() -> lockbox_core::Result<SecretString> {
    Ok(SecretString::try_from_bytes(
        b"correct horse battery staple".to_vec(),
    )?)
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
