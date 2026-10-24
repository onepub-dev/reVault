use revault_lockbox_api::{
    FormFieldDefinition, FormFieldKind, Lockbox, LockboxOpen, LockboxPath, LockboxProtection,
    OwnerSigningKeyPair, SecretString, VariableName,
};
use revault_migration::{export_archive, import_archive, verify_imported_archive};

#[test]
fn native_descriptions_survive_export_import_without_hidden_variables() {
    let root = tempfile::tempdir().unwrap();
    let password =
        SecretString::try_from_slice(b"synthetic description migration password").unwrap();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let mut archive =
        Lockbox::create_in_memory(LockboxProtection::Password(&password), &owner).unwrap();
    let name = VariableName::new("TOKEN").unwrap();
    archive.set_secret_variable(&name, &password).unwrap();
    archive
        .set_variable_description(&name, "Deployment token\nRotate monthly")
        .unwrap();
    let definition = archive
        .define_form(
            "login",
            "Login",
            vec![FormFieldDefinition {
                id: "username".into(),
                label: "Username".into(),
                kind: FormFieldKind::Text,
                required: true,
                description: Some("Full email\nInclude the domain".into()),
            }],
        )
        .unwrap();
    let path = LockboxPath::new("/account").unwrap();
    archive
        .create_form_record(&path, "login", "Production")
        .unwrap();
    archive
        .set_form_field_normal(&path, "username", "alice@example.test")
        .unwrap();
    archive.set_form_record_description(&path, "").unwrap();
    archive.commit().unwrap();
    let artifact = root.path().join("export.migration");
    let output = root.path().join("imported.lbox");
    export_archive(&archive, &artifact, &password, [17; 16]).unwrap();
    import_archive(&artifact, &password, &output, &owner).unwrap();
    verify_imported_archive(&artifact, &password, &output).unwrap();
    let reopened = Lockbox::open(&output, LockboxOpen::Password(&password)).unwrap();
    assert_eq!(reopened.format_version(), 5);
    assert_eq!(reopened.list_variables().unwrap().len(), 1);
    assert_eq!(
        reopened.variable_description(&name).unwrap().as_deref(),
        Some("Deployment token\nRotate monthly")
    );
    assert_eq!(
        reopened.resolve_form_definition("login").unwrap(),
        definition
    );
    assert_eq!(
        reopened
            .get_form_record(&path)
            .unwrap()
            .unwrap()
            .description,
        Some(String::new())
    );
    reopened
        .with_secret_variable(&name, |secret| {
            secret
                .with_str(|text| assert_eq!(text, "synthetic description migration password"))
                .unwrap()
        })
        .unwrap()
        .unwrap();
}
