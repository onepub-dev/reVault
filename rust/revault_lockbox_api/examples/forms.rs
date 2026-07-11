use lockbox_core::{
    FormFieldDefinition, FormFieldKind, FormValue, Lockbox, LockboxOpen, LockboxPath,
    LockboxProtection, OwnerSigningKeyPair, SecretString,
};

fn main() -> lockbox_core::Result<()> {
    let root = example_root("forms")?;
    let lockbox_file = root.join("forms.lbox");
    let pass_phrase = pass_phrase()?;
    let signing_key = OwnerSigningKeyPair::generate()?;
    let password = SecretString::try_from_bytes(b"example-service-password".to_vec())?;

    let mut lockbox = Lockbox::create_file(
        &lockbox_file,
        LockboxProtection::Password(&pass_phrase),
        &signing_key,
    )?;
    let definition = lockbox.define_form(
        "login",
        "Login",
        vec![
            field("url", "URL", FormFieldKind::Url, true),
            field("username", "Username", FormFieldKind::Text, true),
            field("password", "Password", FormFieldKind::Secret, true),
        ],
    )?;
    lockbox.create_form_record(&path("/accounts/example")?, "login", "Example")?;
    lockbox.set_form_field_normal(&path("/accounts/example")?, "url", "https://example.com")?;
    lockbox.set_form_field_normal(&path("/accounts/example")?, "username", "alice")?;
    lockbox.set_form_field_secret(&path("/accounts/example")?, "password", &password)?;
    lockbox.commit()?;

    let opened = Lockbox::open(&lockbox_file, LockboxOpen::Password(&pass_phrase))?;
    let record = opened
        .get_form_record(&path("/accounts/example")?)?
        .expect("record exists");

    println!(
        "{} uses form {} revision {}",
        record.name, definition.alias, record.definition_revision
    );
    for field_value in record.values {
        match field_value.value {
            FormValue::Normal(value) => println!("{}={value}", field_value.field_id),
            FormValue::Secret(secret) => {
                let len = secret.with_str(|value| value.len())?;
                println!("{}=<secret, {len} chars>", field_value.field_id);
            }
        }
    }
    Ok(())
}

fn field(id: &str, label: &str, kind: FormFieldKind, required: bool) -> FormFieldDefinition {
    FormFieldDefinition {
        id: id.to_string(),
        label: label.to_string(),
        kind,
        required,
    }
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
