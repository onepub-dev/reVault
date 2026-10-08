//! Public API baseline for form snapshots; no experimental-format internals.
use revault_lockbox_api::{
    Compression, Encryption, FormFieldDefinition, FormFieldKind, FormRecord, FormTypeId, FormValue,
    Lockbox, LockboxCreateOptions, LockboxOpen, LockboxPath, LockboxProtection,
    OwnerSigningKeyPair, SecretString, SecretVec, Signing, SizePadding,
};

fn field(id: &str, label: &str, kind: FormFieldKind) -> FormFieldDefinition {
    FormFieldDefinition {
        id: id.into(),
        label: label.into(),
        kind,
        required: false,
    }
}
fn captured(record: &FormRecord, id: &str, label: &str, kind: FormFieldKind, text: &str) {
    let value = record.values.iter().find(|v| v.field_id == id).unwrap();
    assert_eq!(value.captured_label, label);
    assert_eq!(value.kind, kind);
    match &value.value {
        FormValue::Normal(value) => {
            assert!(!kind.is_secret());
            assert_eq!(value, text);
        }
        FormValue::Secret(value) => {
            assert!(kind.is_secret());
            value.with_str(|value| assert_eq!(value, text)).unwrap();
        }
    }
}

#[test]
fn public_form_history_preserves_mixed_captures_and_explicit_recreation_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let key = [91; 32];
    let path = LockboxPath::new("/captures/record").unwrap();
    // Public FormTypeId is a normalized36-character hex/dash identifier, not
    // a strict UUID parser. Preserve this accepted edge in later typed adapters.
    let type_id = FormTypeId::new("------------------------------------").unwrap();
    let secret = SecretString::try_from_slice(b"synthetic secret").unwrap();
    for bits in 0..16 {
        let signing = if bits & 2 != 0 {
            Signing::Owner(&owner)
        } else {
            Signing::None
        };
        let open = || {
            if bits & 1 != 0 {
                LockboxOpen::ContentKey(SecretVec::try_from_slice(&key).unwrap())
            } else {
                LockboxOpen::Unencrypted
            }
        };
        let mut archive = Lockbox::create_in_memory_with_options(LockboxCreateOptions {
            compression: if bits & 4 != 0 {
                Compression::default()
            } else {
                Compression::None
            },
            size_padding: if bits & 8 != 0 {
                SizePadding::Default
            } else {
                SizePadding::None
            },
            ..LockboxCreateOptions::new(
                if bits & 1 != 0 {
                    Encryption::Encrypted(LockboxProtection::ContentKey(
                        SecretVec::try_from_slice(&key).unwrap(),
                    ))
                } else {
                    Encryption::None
                },
                signing,
            )
        })
        .unwrap();
        archive
            .define_form_with_type_id(
                type_id.clone(),
                "capture",
                "Capture",
                vec![
                    field("retired", "Original retired", FormFieldKind::Text),
                    field("changed", "Original changed", FormFieldKind::Text),
                    field("trigger", "Trigger one", FormFieldKind::Text),
                    field("password", "Password one", FormFieldKind::Secret),
                ],
            )
            .unwrap();
        archive
            .create_form_record(&path, "capture", "Example")
            .unwrap();
        for (id, value) in [
            ("retired", "retained"),
            ("changed", "old text"),
            ("trigger", "one"),
        ] {
            archive.set_form_field_normal(&path, id, value).unwrap();
        }
        archive
            .set_form_field_secret(&path, "password", &secret)
            .unwrap();
        let second = archive
            .revise_form_definition(
                &type_id,
                "Capture two",
                "",
                vec![
                    field("changed", "New changed", FormFieldKind::Notes),
                    field("trigger", "Trigger two", FormFieldKind::Text),
                    field("password", "Password two", FormFieldKind::Secret),
                ],
            )
            .unwrap();
        assert_eq!(second.revision, 2);
        assert_eq!(
            archive
                .get_form_record(&path)
                .unwrap()
                .unwrap()
                .definition_revision,
            1
        );
        archive
            .set_form_field_normal(&path, "trigger", "two")
            .unwrap();
        archive.commit().unwrap();
        let bytes = archive.try_to_bytes().unwrap();
        drop(archive);
        let mut archive = Lockbox::open_bytes_for_write(bytes, open(), signing).unwrap();
        let record = archive.get_form_record(&path).unwrap().unwrap();
        assert_eq!(
            (
                &record.type_id,
                record.definition_revision,
                record.definition_alias.as_str()
            ),
            (&type_id, 2, "capture")
        );
        captured(
            &record,
            "retired",
            "Original retired",
            FormFieldKind::Text,
            "retained",
        );
        captured(
            &record,
            "changed",
            "Original changed",
            FormFieldKind::Text,
            "old text",
        );
        captured(
            &record,
            "trigger",
            "Trigger two",
            FormFieldKind::Text,
            "two",
        );
        captured(
            &record,
            "password",
            "Password one",
            FormFieldKind::Secret,
            "synthetic secret",
        );
        assert!(!archive
            .resolve_form_definition("capture")
            .unwrap()
            .fields
            .iter()
            .any(|f| f.id == "retired"));
        assert_eq!(
            archive
                .list_form_definition_revisions(&type_id)
                .unwrap()
                .len(),
            2
        );
        let before = archive.try_to_bytes().unwrap();
        assert!(archive
            .set_form_field_normal(&path, "password", "forbidden")
            .is_err());
        assert!(archive
            .revise_form_definition(
                &type_id,
                "Invalid",
                "",
                vec![field("password", "Downgrade", FormFieldKind::Text)]
            )
            .is_err());
        archive.commit().unwrap();
        assert_eq!(archive.try_to_bytes().unwrap(), before);
        let third = archive
            .revise_form_definition(
                &type_id,
                "Remove password",
                "",
                vec![
                    field("changed", "New changed", FormFieldKind::Notes),
                    field("trigger", "Trigger three", FormFieldKind::Text),
                ],
            )
            .unwrap();
        assert_eq!(third.revision, 3);
        let fourth = archive
            .revise_form_definition(
                &type_id,
                "Recreate password",
                "",
                vec![
                    field("changed", "New changed", FormFieldKind::Notes),
                    field("trigger", "Trigger four", FormFieldKind::Text),
                    field("password", "Recreated normal", FormFieldKind::Text),
                ],
            )
            .unwrap();
        assert_eq!(fourth.revision, 4);
        archive
            .set_form_field_normal(&path, "trigger", "four")
            .unwrap();
        let record = archive.get_form_record(&path).unwrap().unwrap();
        assert_eq!(record.definition_revision, 4);
        captured(
            &record,
            "password",
            "Password one",
            FormFieldKind::Secret,
            "synthetic secret",
        );
        archive.commit().unwrap();
        let bytes = archive.try_to_bytes().unwrap();
        drop(archive);
        let mut archive = Lockbox::open_bytes_for_write(bytes, open(), signing).unwrap();
        let record = archive.get_form_record(&path).unwrap().unwrap();
        captured(
            &record,
            "retired",
            "Original retired",
            FormFieldKind::Text,
            "retained",
        );
        captured(
            &record,
            "password",
            "Password one",
            FormFieldKind::Secret,
            "synthetic secret",
        );
        archive
            .set_form_field_normal(&path, "password", "recreated")
            .unwrap();
        archive
            .set_form_field_normal(&path, "changed", "new\nnotes")
            .unwrap();
        archive.commit().unwrap();
        let bytes = archive.try_to_bytes().unwrap();
        drop(archive);
        let archive = Lockbox::open_bytes(bytes, open()).unwrap();
        let record = archive.get_form_record(&path).unwrap().unwrap();
        assert_eq!(record.definition_revision, 4);
        assert_eq!(
            archive
                .list_form_definition_revisions(&type_id)
                .unwrap()
                .len(),
            4
        );
        captured(
            &record,
            "password",
            "Recreated normal",
            FormFieldKind::Text,
            "recreated",
        );
        captured(
            &record,
            "changed",
            "New changed",
            FormFieldKind::Notes,
            "new\nnotes",
        );
        captured(
            &record,
            "retired",
            "Original retired",
            FormFieldKind::Text,
            "retained",
        );
    }
}

#[test]
fn public_form_namespace_and_parent_creation_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let key = [92; 32];
    let shared = LockboxPath::new("/shared").unwrap();
    let child = LockboxPath::new("/shared/record").unwrap();
    let nested = LockboxPath::new("/new/deep/record").unwrap();
    for bits in 0..16 {
        let signing = if bits & 2 != 0 {
            Signing::Owner(&owner)
        } else {
            Signing::None
        };
        let open = || {
            if bits & 1 != 0 {
                LockboxOpen::ContentKey(SecretVec::try_from_slice(&key).unwrap())
            } else {
                LockboxOpen::Unencrypted
            }
        };
        let mut archive = Lockbox::create_in_memory_with_options(LockboxCreateOptions {
            compression: if bits & 4 != 0 {
                Compression::default()
            } else {
                Compression::None
            },
            size_padding: if bits & 8 != 0 {
                SizePadding::Default
            } else {
                SizePadding::None
            },
            ..LockboxCreateOptions::new(
                if bits & 1 != 0 {
                    Encryption::Encrypted(LockboxProtection::ContentKey(
                        SecretVec::try_from_slice(&key).unwrap(),
                    ))
                } else {
                    Encryption::None
                },
                signing,
            )
        })
        .unwrap();
        archive
            .define_form(
                "namespace",
                "Namespace",
                vec![field("text", "Text", FormFieldKind::Text)],
            )
            .unwrap();
        archive
            .add_file(&shared, b"independent file", false)
            .unwrap();
        archive
            .create_form_record(&shared, "namespace", "Same path")
            .unwrap();
        archive
            .create_form_record(&child, "namespace", "File parent")
            .unwrap();
        archive
            .create_form_record(&nested, "namespace", "New parents")
            .unwrap();
        archive
            .set_form_field_normal(&child, "text", "child value")
            .unwrap();
        archive.commit().unwrap();
        let bytes = archive.try_to_bytes().unwrap();
        drop(archive);
        let archive = Lockbox::open_bytes(bytes, open()).unwrap();
        assert_eq!(archive.get_file(&shared).unwrap(), b"independent file");
        assert!(!archive.is_dir(&shared));
        assert_eq!(
            archive.get_form_record(&shared).unwrap().unwrap().name,
            "Same path"
        );
        let record = archive.get_form_record(&child).unwrap().unwrap();
        captured(&record, "text", "Text", FormFieldKind::Text, "child value");
        assert_eq!(
            archive.get_form_record(&nested).unwrap().unwrap().name,
            "New parents"
        );
        assert!(archive.is_dir(&LockboxPath::new("/new").unwrap()));
        assert!(archive.is_dir(&LockboxPath::new("/new/deep").unwrap()));
    }
}
