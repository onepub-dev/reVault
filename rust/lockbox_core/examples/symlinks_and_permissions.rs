use lockbox_core::{
    Lockbox, LockboxOpen, LockboxPath, LockboxProtection, OwnerSigningKeyPair, SecretString,
};

fn main() -> lockbox_core::Result<()> {
    let root = example_root("symlinks-and-permissions")?;
    let lockbox_file = root.join("symlinks.lbox");
    let pass_phrase = pass_phrase()?;
    let signing_key = OwnerSigningKeyPair::generate()?;

    let mut lockbox = Lockbox::create_file(
        &lockbox_file,
        LockboxProtection::Password(&pass_phrase),
        &signing_key,
    )?;
    lockbox.add_file_with_permissions(
        &path("/bin/tool.sh")?,
        b"#!/bin/sh\necho from lockbox\n",
        0o755,
        false,
    )?;
    lockbox.add_symlink(&path("/bin/current")?, &path("/bin/tool.sh")?, false)?;
    lockbox.commit()?;

    let opened = Lockbox::open_file(&lockbox_file, LockboxOpen::Password(&pass_phrase))?;
    println!(
        "/bin/tool.sh permissions: {:o}",
        opened
            .permissions(&path("/bin/tool.sh")?)
            .unwrap_or_default()
    );
    println!(
        "/bin/current -> {}",
        opened.get_symlink_target(&path("/bin/current")?)?
    );
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
