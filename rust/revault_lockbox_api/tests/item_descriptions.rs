use revault_lockbox_api::{
    FormFieldDefinition, FormFieldKind, Lockbox, LockboxOpen, LockboxPath, LockboxProtection,
    OwnerSigningKeyPair, SecretString, VariableName,
};

#[test]
fn descriptions_persist_move_and_preserve_secret_values_and_revisions() {
    let password = SecretString::try_from_slice(b"synthetic password").unwrap();
    let signer = OwnerSigningKeyPair::generate().unwrap();
    let mut box_ =
        Lockbox::create_in_memory(LockboxProtection::Password(&password), &signer).unwrap();
    let variable = VariableName::new("/TOKEN").unwrap();
    box_.set_secret_variable(&variable, &password).unwrap();
    box_.set_variable_description(&variable, "Purpose\nSecond line\twith tab")
        .unwrap();
    let first = box_
        .define_form_with_description(
            "login",
            "Login",
            "Login guidance",
            vec![FormFieldDefinition {
                id: "password".into(),
                label: "Password".into(),
                kind: FormFieldKind::Secret,
                required: true,
            }],
        )
        .unwrap();
    let record = LockboxPath::new("/login").unwrap();
    box_.create_form_record(&record, "login", "Staging")
        .unwrap();
    box_.set_form_record_description(&record, "Staging\nRecreated nightly")
        .unwrap();
    let second = box_
        .set_form_field_description(
            "login",
            "password",
            "Use the app password\nNot the account password",
        )
        .unwrap();
    assert_eq!(second.revision, first.revision + 1);
    assert!(box_.form_field_descriptions(&first).unwrap().is_empty());
    assert_eq!(
        box_.set_form_field_description(
            "login",
            "password",
            "Use the app password\nNot the account password"
        )
        .unwrap()
        .revision,
        second.revision
    );
    let third = box_
        .set_form_definition_description("login", "New definition\nInstructions")
        .unwrap();
    assert_eq!(third.revision, second.revision + 1);
    assert!(box_
        .form_field_description("login", "password")
        .unwrap()
        .unwrap()
        .contains("app password"));
    assert_eq!(
        box_.get_form_record(&record)
            .unwrap()
            .unwrap()
            .definition_revision,
        first.revision
    );
    assert!(box_
        .set_form_field_description("login", "missing", "text")
        .is_err());
    assert_eq!(
        box_.resolve_form_definition("login").unwrap().revision,
        third.revision
    );

    let moved = VariableName::new("/NEW_TOKEN").unwrap();
    box_.move_variables(&[(variable.clone(), moved.clone())])
        .unwrap();
    let moved_record = LockboxPath::new("/moved").unwrap();
    box_.move_form_records(&[(record.clone(), moved_record.clone())])
        .unwrap();
    box_.commit().unwrap();
    let read = Lockbox::open_bytes(
        box_.try_to_bytes().unwrap(),
        LockboxOpen::Password(&password),
    )
    .unwrap();
    assert_eq!(
        read.variable_description(&moved).unwrap().as_deref(),
        Some("Purpose\nSecond line\twith tab")
    );
    assert!(read.variable_description(&variable).is_err());
    assert_eq!(
        read.form_record_description(&moved_record)
            .unwrap()
            .as_deref(),
        Some("Staging\nRecreated nightly")
    );
    read.with_secret_variable(&moved, |secret| {
        secret
            .with_str(|value| assert_eq!(value, "synthetic password"))
            .unwrap()
    })
    .unwrap()
    .unwrap();
    assert!(read
        .form_field_description("login", "password")
        .unwrap()
        .unwrap()
        .contains("app password"));

    let mut other =
        Lockbox::create_in_memory(LockboxProtection::Password(&password), &signer).unwrap();
    let definition = read.resolve_form_definition("login").unwrap();
    let descriptions = read.form_field_descriptions(&definition).unwrap();
    other
        .import_form_definition_with_descriptions(definition.clone(), descriptions.clone())
        .unwrap();
    other
        .import_form_definition_with_descriptions(definition.clone(), descriptions)
        .unwrap();
    assert!(other
        .import_form_definition_with_descriptions(definition, Default::default())
        .is_err());
    other.commit().unwrap();
    let copied = Lockbox::open_bytes(
        other.try_to_bytes().unwrap(),
        LockboxOpen::Password(&password),
    )
    .unwrap();
    assert_eq!(
        copied.form_field_description("login", "password").unwrap(),
        read.form_field_description("login", "password").unwrap()
    );

    box_.clear_variable_description(&moved).unwrap();
    box_.clear_variable_description(&moved).unwrap();
    box_.set_variable_description(&moved, "").unwrap();
    assert_eq!(
        box_.variable_description(&moved).unwrap(),
        Some(String::new())
    );
    box_.delete_variable(&moved).unwrap();
    box_.set_variable(&moved, "new value").unwrap();
    assert_eq!(box_.variable_description(&moved).unwrap(), None);
    box_.delete_form_record(&moved_record).unwrap();
    box_.create_form_record(&moved_record, "login", "Replacement")
        .unwrap();
    assert_eq!(box_.form_record_description(&moved_record).unwrap(), None);
    let cleared = box_
        .clear_form_field_description("login", "password")
        .unwrap();
    assert_eq!(
        box_.clear_form_field_description("login", "password")
            .unwrap()
            .revision,
        cleared.revision
    );
    assert!(box_
        .form_field_descriptions(&second)
        .unwrap()
        .contains_key("password"));
    box_.commit().unwrap();
    let read = Lockbox::open_bytes(
        box_.try_to_bytes().unwrap(),
        LockboxOpen::Password(&password),
    )
    .unwrap();
    assert_eq!(
        read.form_field_description("login", "password").unwrap(),
        None
    );
    assert_eq!(read.form_record_description(&moved_record).unwrap(), None);
}
