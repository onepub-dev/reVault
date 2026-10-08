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

#[test]
fn public_form_secret_upgrade_updates_all_captures_and_absent_references_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let key = [92; 32];
    let type_id = FormTypeId::new("111111111111111111111111111111111111").unwrap();
    let other_id = FormTypeId::new("222222222222222222222222222222222222").unwrap();
    let target = LockboxPath::new("/upgrade/target").unwrap();
    let sibling = LockboxPath::new("/upgrade/sibling").unwrap();
    let mixed = LockboxPath::new("/upgrade/mixed").unwrap();
    let absent = LockboxPath::new("/upgrade/absent").unwrap();
    let unrelated = LockboxPath::new("/other/record").unwrap();
    let old_secret = SecretString::try_from_slice(b"historical secret").unwrap();
    let replacement = SecretString::try_from_slice(b"target override").unwrap();
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
                "upgrade",
                "Upgrade",
                vec![
                    field("value", "Historical secret label", FormFieldKind::Secret),
                    field("keep", "Original keep", FormFieldKind::Text),
                ],
            )
            .unwrap();
        archive
            .create_form_record(&mixed, "upgrade", "Mixed")
            .unwrap();
        archive
            .set_form_field_secret(&mixed, "value", &old_secret)
            .unwrap();
        archive
            .set_form_field_normal(&mixed, "keep", "mixed keep")
            .unwrap();
        assert_eq!(
            archive
                .revise_form_definition(
                    &type_id,
                    "Upgrade two",
                    "",
                    vec![field("keep", "Current keep", FormFieldKind::Text)]
                )
                .unwrap()
                .revision,
            2
        );
        assert_eq!(
            archive
                .revise_form_definition(
                    &type_id,
                    "Upgrade three",
                    "",
                    vec![
                        field("value", "Current value label", FormFieldKind::Text),
                        field("keep", "Current keep", FormFieldKind::Text)
                    ]
                )
                .unwrap()
                .revision,
            3
        );
        for (path, name) in [
            (&target, "Target"),
            (&sibling, "Sibling"),
            (&absent, "Absent"),
        ] {
            archive.create_form_record(path, "upgrade", name).unwrap();
            archive.set_form_field_normal(path, "keep", name).unwrap();
        }
        archive
            .set_form_field_normal(&target, "value", "target normal")
            .unwrap();
        archive
            .set_form_field_normal(&sibling, "value", "sibling normal")
            .unwrap();
        archive
            .define_form_with_type_id(
                other_id.clone(),
                "other",
                "Other",
                vec![field("value", "Other label", FormFieldKind::Text)],
            )
            .unwrap();
        archive
            .create_form_record(&unrelated, "other", "Unrelated")
            .unwrap();
        archive
            .set_form_field_normal(&unrelated, "value", "untouched")
            .unwrap();
        archive.commit().unwrap();
        let bytes = archive.try_to_bytes().unwrap();
        drop(archive);
        let mut archive = Lockbox::open_bytes_for_write(bytes, open(), signing).unwrap();
        let old_mixed = archive.get_form_record(&mixed).unwrap().unwrap();
        assert_eq!(old_mixed.definition_revision, 1);
        captured(
            &old_mixed,
            "value",
            "Historical secret label",
            FormFieldKind::Secret,
            "historical secret",
        );
        captured(
            &old_mixed,
            "keep",
            "Original keep",
            FormFieldKind::Text,
            "mixed keep",
        );
        for path in [&target, &sibling, &absent] {
            assert_eq!(
                archive
                    .get_form_record(path)
                    .unwrap()
                    .unwrap()
                    .definition_revision,
                3
            );
        }
        captured(
            &archive.get_form_record(&target).unwrap().unwrap(),
            "value",
            "Current value label",
            FormFieldKind::Text,
            "target normal",
        );
        captured(
            &archive.get_form_record(&sibling).unwrap().unwrap(),
            "value",
            "Current value label",
            FormFieldKind::Text,
            "sibling normal",
        );
        captured(
            &archive.get_form_record(&absent).unwrap().unwrap(),
            "keep",
            "Current keep",
            FormFieldKind::Text,
            "Absent",
        );
        let other = archive.get_form_record(&unrelated).unwrap().unwrap();
        captured(
            &other,
            "value",
            "Other label",
            FormFieldKind::Text,
            "untouched",
        );
        assert!(archive.get_form_field(&absent, "value").unwrap().is_none());
        archive
            .set_form_field_secret(&target, "value", &replacement)
            .unwrap();
        archive.commit().unwrap();
        let bytes = archive.try_to_bytes().unwrap();
        drop(archive);
        let mut archive = Lockbox::open_bytes_for_write(bytes, open(), signing).unwrap();
        let definition = archive.resolve_form_definition("upgrade").unwrap();
        assert_eq!(definition.revision, 4);
        assert_eq!(
            definition
                .fields
                .iter()
                .find(|f| f.id == "value")
                .unwrap()
                .kind,
            FormFieldKind::Secret
        );
        for path in [&target, &sibling, &mixed, &absent] {
            let record = archive.get_form_record(path).unwrap().unwrap();
            assert_eq!(record.type_id, type_id);
            assert_eq!(record.definition_revision, 4);
            assert_eq!(record.definition_alias, "upgrade");
        }
        captured(
            &archive.get_form_record(&target).unwrap().unwrap(),
            "value",
            "Current value label",
            FormFieldKind::Secret,
            "target override",
        );
        captured(
            &archive.get_form_record(&sibling).unwrap().unwrap(),
            "value",
            "Current value label",
            FormFieldKind::Secret,
            "sibling normal",
        );
        let updated = archive.get_form_record(&mixed).unwrap().unwrap();
        captured(
            &updated,
            "value",
            "Current value label",
            FormFieldKind::Secret,
            "historical secret",
        );
        captured(
            &updated,
            "keep",
            "Original keep",
            FormFieldKind::Text,
            "mixed keep",
        );
        assert!(archive.get_form_field(&absent, "value").unwrap().is_none());
        assert_eq!(archive.get_form_record(&unrelated).unwrap().unwrap(), other);
        let expected = [&target, &sibling, &mixed, &absent, &unrelated]
            .into_iter()
            .map(|p| archive.get_form_record(p).unwrap().unwrap())
            .collect::<Vec<_>>();
        archive
            .set_form_field_secret(&target, "value", &replacement)
            .unwrap();
        archive.commit().unwrap();
        let bytes = archive.try_to_bytes().unwrap();
        drop(archive);
        let mut archive = Lockbox::open_bytes_for_write(bytes, open(), signing).unwrap();
        assert_eq!(
            archive
                .list_form_definition_revisions(&type_id)
                .unwrap()
                .len(),
            4
        );
        assert_eq!(
            archive.resolve_form_definition("upgrade").unwrap().revision,
            4
        );
        captured(
            &archive.get_form_record(&target).unwrap().unwrap(),
            "value",
            "Current value label",
            FormFieldKind::Secret,
            "target override",
        );
        assert_eq!(archive.get_form_record(&unrelated).unwrap().unwrap(), other);
        for record in &expected {
            assert_eq!(
                archive.get_form_record(&record.path).unwrap().as_ref(),
                Some(record)
            );
        }
        let before = archive.try_to_bytes().unwrap();
        assert!(archive
            .set_form_field_normal(&target, "value", "downgrade")
            .is_err());
        assert!(archive
            .set_form_field_secret(&target, "unknown", &replacement)
            .is_err());
        archive.commit().unwrap();
        let after = archive.try_to_bytes().unwrap();
        assert_eq!(after, before);
        drop(archive);
        let archive = Lockbox::open_bytes_for_write(after, open(), signing).unwrap();
        captured(
            &archive.get_form_record(&target).unwrap().unwrap(),
            "value",
            "Current value label",
            FormFieldKind::Secret,
            "target override",
        );
        assert_eq!(archive.get_form_record(&unrelated).unwrap().unwrap(), other);
        for record in &expected {
            assert_eq!(
                archive.get_form_record(&record.path).unwrap().as_ref(),
                Some(record)
            );
        }
    }
}

#[test]
fn public_form_definition_resolution_revision_and_empty_creation_all_modes() {
    use revault_lockbox_api::Error;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let key = [93; 32];
    let id = FormTypeId::new("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
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
        let mut required = field("value", "Required", FormFieldKind::Text);
        required.required = true;
        let first = archive
            .define_form_with_type_id_and_description(
                id.clone(),
                "base",
                "Name",
                "Description",
                vec![required.clone()],
            )
            .unwrap();
        assert_eq!(first.revision, 1);
        let path = LockboxPath::new("/new/deep/empty").unwrap();
        let captured = archive
            .create_form_record(&path, "base", "Empty required record")
            .unwrap();
        assert!(captured.values.is_empty());
        assert_eq!(captured.definition_revision, 1);
        assert!(matches!(
            archive.define_form_with_type_id(id.clone(), "bad alias", "No", vec![required.clone()]),
            Err(Error::InvalidInput(_))
        ));
        assert_eq!(
            archive
                .resolve_form_definition(id.as_str())
                .unwrap()
                .revision,
            1
        );
        let second = archive
            .define_form_with_type_id_and_description(
                id.clone(),
                "ignored_new_alias",
                "Name",
                "Description",
                vec![required.clone()],
            )
            .unwrap();
        assert_eq!(second.revision, 2);
        assert_eq!(second.alias, "base");
        let third = archive
            .define_form_with_description("base", "Name", "Description", vec![required.clone()])
            .unwrap();
        assert_eq!(third.revision, 3);
        assert_eq!(third.type_id, id);
        assert_eq!(
            archive.resolve_form_definition(&"B".repeat(36)).unwrap(),
            third
        );
        archive.commit().unwrap();
        let bytes = archive.try_to_bytes().unwrap();
        drop(archive);
        let mut archive = Lockbox::open_bytes_for_write(bytes, open(), signing).unwrap();
        assert_eq!(archive.get_form_record(&path).unwrap().unwrap(), captured);
        assert!(archive.is_dir(&LockboxPath::new("/new").unwrap()));
        assert!(archive.is_dir(&LockboxPath::new("/new/deep").unwrap()));
        assert_eq!(
            archive.list_form_definition_revisions(&id).unwrap(),
            vec![first.clone(), second.clone(), third.clone()]
        );
        // Exact import is idempotent, unlike repeated define.
        let before = archive.try_to_bytes().unwrap();
        assert_eq!(
            archive.import_form_definition(third.clone()).unwrap(),
            third
        );
        archive.commit().unwrap();
        assert_eq!(before, archive.try_to_bytes().unwrap());
        let mut conflicting = third.clone();
        conflicting.name = "Conflict".into();
        assert!(matches!(
            archive.import_form_definition(conflicting),
            Err(Error::InvalidOperation(_))
        ));
        let mut alias_collision = third.clone();
        alias_collision.type_id = FormTypeId::new("cccccccccccccccccccccccccccccccccccc").unwrap();
        alias_collision.revision = 1;
        archive
            .import_form_definition(alias_collision.clone())
            .unwrap();
        assert!(matches!(
            archive.resolve_form_definition("base"),
            Err(Error::InvalidOperation(_))
        ));
        assert!(matches!(
            archive.define_form("base", "No", vec![required.clone()]),
            Err(Error::InvalidOperation(_))
        ));
        assert!(matches!(
            archive.resolve_form_definition("bad alias"),
            Err(Error::InvalidInput(_))
        ));
        assert!(matches!(
            archive.resolve_form_definition("missing"),
            Err(Error::NotFound(_))
        ));
        assert!(matches!(
            archive.create_form_record(
                &LockboxPath::new("/refused/ambiguous").unwrap(),
                "base",
                "No"
            ),
            Err(Error::InvalidOperation(_))
        ));
        assert!(!archive.is_dir(&LockboxPath::new("/refused").unwrap()));
        assert!(matches!(
            archive.create_form_record(&path, id.as_str(), "Duplicate"),
            Err(Error::AlreadyExists(_))
        ));
        // Alias overlaps accepted type-ID syntax: resolver does not fall back to alias.
        let hex = "a".repeat(36);
        let h1 = archive
            .define_form(&hex, "Hex", vec![required.clone()])
            .unwrap();
        let h2 = archive
            .define_form(&hex, "Hex", vec![required.clone()])
            .unwrap();
        assert_ne!(h1.type_id, h2.type_id);
        assert_eq!((h1.revision, h2.revision), (1, 1));
        assert!(matches!(
            archive.resolve_form_definition(&hex),
            Err(Error::NotFound(_))
        ));
        let explicit = LockboxPath::new("/explicit/hex").unwrap();
        archive
            .create_form_record(&explicit, h1.type_id.as_str(), "Explicit ID")
            .unwrap();
        let routed_id = FormTypeId::new("dddddddddddddddddddddddddddddddddddd").unwrap();
        archive
            .define_form_with_type_id(
                routed_id.clone(),
                "typed_route",
                "Route one",
                vec![required.clone()],
            )
            .unwrap();
        let routed = archive
            .define_form(routed_id.as_str(), "Route two", vec![required.clone()])
            .unwrap();
        assert_eq!(routed.type_id, routed_id);
        assert_eq!(routed.revision, 2);
        assert_eq!(routed.alias, "typed_route");
        let mut imported_latest = third.clone();
        imported_latest.revision = 10;
        imported_latest.alias = "latest_alias".into();
        archive
            .import_form_definition(imported_latest.clone())
            .unwrap();
        assert_eq!(
            archive.resolve_form_definition("base").unwrap(),
            alias_collision
        );
        assert_eq!(
            archive.resolve_form_definition("latest_alias").unwrap(),
            imported_latest
        );
        archive.commit().unwrap();
        let bytes = archive.try_to_bytes().unwrap();
        drop(archive);
        let archive = Lockbox::open_bytes(bytes, open()).unwrap();
        assert_eq!(archive.get_form_record(&path).unwrap().unwrap(), captured);
        assert_eq!(
            archive.get_form_record(&explicit).unwrap().unwrap().type_id,
            h1.type_id
        );
        assert!(archive.is_dir(&LockboxPath::new("/explicit").unwrap()));
        assert!(!archive.is_dir(&LockboxPath::new("/refused").unwrap()));
        assert_eq!(
            archive.resolve_form_definition(id.as_str()).unwrap(),
            imported_latest
        );
        assert_eq!(
            archive
                .list_form_definition_revisions(&id)
                .unwrap()
                .iter()
                .map(|d| d.revision)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 10]
        );
        assert_eq!(
            archive
                .resolve_form_definition(alias_collision.type_id.as_str())
                .unwrap(),
            alias_collision
        );
        assert_eq!(
            archive.resolve_form_definition("base").unwrap(),
            alias_collision
        );
        assert_eq!(
            archive.resolve_form_definition("latest_alias").unwrap(),
            imported_latest
        );
        assert!(matches!(
            archive.resolve_form_definition(&hex),
            Err(Error::NotFound(_))
        ));
        let latest = archive.list_form_definitions().unwrap();
        let ids = latest
            .iter()
            .map(|d| d.type_id.as_str())
            .collect::<Vec<_>>();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);
        assert_eq!(latest.len(), 5);
        assert_eq!(
            archive.resolve_form_definition("typed_route").unwrap(),
            routed
        );
        assert_eq!(
            archive.resolve_form_definition(routed_id.as_str()).unwrap(),
            routed
        );
    }
}
