use lockbox_core::{
    Lockbox, LockboxOpen, LockboxProtection, OwnerSigningKeyPair, SecretString, VariableName,
    VariableValueRef,
};

fn main() -> lockbox_core::Result<()> {
    let root = example_root("variables")?;
    let lockbox_file = root.join("variables.lbox");
    let pass_phrase = pass_phrase()?;
    let signing_key = OwnerSigningKeyPair::generate()?;
    let api_token = SecretString::try_from_bytes(b"token-from-password-manager".to_vec())?;

    let mut lockbox = Lockbox::create_file(
        &lockbox_file,
        LockboxProtection::Password(&pass_phrase),
        &signing_key,
    )?;
    lockbox.set_variable(&variable("APP_ENV")?, "production")?;
    lockbox.set_secret_variable(&variable("API_TOKEN")?, &api_token)?;
    lockbox.commit()?;

    let opened = Lockbox::open_file(&lockbox_file, LockboxOpen::Password(&pass_phrase))?;
    opened.visit_variables(|name, value| {
        match value {
            VariableValueRef::Normal(value) => println!("{name}={value}"),
            VariableValueRef::Secret(value) => {
                let len = value.with_str(|secret| secret.len())?;
                println!("{name}=<secret, {len} chars>");
            }
        }
        Ok(())
    })?;

    Ok(())
}

fn variable(value: &str) -> lockbox_core::Result<VariableName> {
    VariableName::new(value)
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
