use super::super::variables::Guarded;
use super::*;
use crate::file_format::candidate_files::dense_catalogue::forms::Forms;
fn definition() -> FormDefinition {
    FormDefinition {
        type_id: FormTypeId::new("------------------------------------").unwrap(),
        alias: "forms".into(),
        revision: 7,
        name: "name1111".into(),
        description: "desc2222".into(),
        fields: vec![
            FormFieldDefinition {
                id: "secret".into(),
                label: "Secret".into(),
                kind: FormFieldKind::Secret,
                required: false,
            },
            FormFieldDefinition {
                id: "late".into(),
                label: "Last label".into(),
                kind: FormFieldKind::Notes,
                required: false,
            },
        ],
    }
}

#[test]
fn typed_form_metadata_byte_budget_refuses_before_publication() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, false, false, false);
    let authority = authority(mode, &public);
    let mut storage = super::super::mutation::seed(mode, &authority, &owner);
    let before = storage.read_all().unwrap();
    let mut definition = definition();
    // Within the old 4096-row/page ceilings, but the aggregate metadata charge
    // exceeds the shared reader/writer byte budget. Publication must not change.
    definition.fields = (0..3100)
        .map(|index| FormFieldDefinition {
            id: format!("field_{index}"),
            label: "Field".into(),
            kind: FormFieldKind::Text,
            required: false,
        })
        .collect();
    let result = import_definition(
        &mut storage,
        archive(),
        mode,
        &authority,
        None,
        key(mode),
        &definition,
    );
    assert!(
        matches!(result, Err(Error::SecurityLimitExceeded(ref reason)) if reason.contains("metadata byte budget"))
    );
    assert_eq!(storage.read_all().unwrap(), before);
    AuditedTreeImage::open(storage, archive(), mode, &authority, key(mode))
        .unwrap()
        .image
        .verify_all()
        .unwrap();
}
#[test]
fn typed_form_guarded_admission_and_semantic_page_substitution_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let def = definition();
    let secret = Arc::new(SecretString::try_from_slice(b"synthetic").unwrap());
    let record = FormRecord {
        path: LockboxPath::new("/forms/record").unwrap(),
        name: "Record".into(),
        type_id: def.type_id.clone(),
        definition_alias: def.alias.clone(),
        definition_revision: def.revision,
        values: vec![FormFieldValue {
            field_id: "secret".into(),
            captured_label: "Captured".into(),
            kind: FormFieldKind::Secret,
            value: FormValue::Secret(secret),
        }],
    };
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = Guarded::new(
            super::super::mutation::seed(mode, &authority, &owner),
            Vec::new(),
        );
        import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &def,
        )
        .unwrap();
        import_captured_record(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &record,
        )
        .unwrap();
        let opened =
            AuditedTreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
                .unwrap();
        assert_eq!(
            opened
                .with_form_field_value(&record.path, "secret", |v| match v {
                    FormValue::Secret(v) => v.with_str(|s| assert_eq!(s, "synthetic")).unwrap(),
                    _ => panic!("secret downgraded"),
                })
                .unwrap(),
            Some(())
        );
        let rows = opened.image.catalogue.forms.encode().unwrap();
        let sealed = opened.image.anchor.sealed_len;
        assert!(Forms::decode(rows.clone(), mode, sealed).is_ok());
        for case in 0..9 {
            let mut changed = rows.clone();
            match case {
                0 => changed.push(changed[0].clone()),
                1 => changed.retain(|e| e.namespace != 8),
                2 => {
                    let field = changed.iter_mut().find(|e| e.namespace == 10).unwrap();
                    let last = field.key.len() - 1;
                    field.key[last] = 99;
                }
                3 => changed.retain(|e| e.namespace != 7),
                4 => {
                    let field = changed.iter_mut().find(|e| e.namespace == 10).unwrap();
                    let len = u16::from_le_bytes(field.value[..2].try_into().unwrap()) as usize;
                    field.value[2 + len] = FormFieldKind::Text.code();
                }
                _ => {
                    let (ns, cap) = [(7, 1850), (8, 990), (9, 1048), (10, 1847)][case - 5];
                    let row = changed.iter_mut().find(|e| e.namespace == ns).unwrap();
                    row.value.resize(cap + 1, 0);
                    assert!(Forms::admit_row(row).is_err());
                }
            }
            assert!(
                Forms::decode(changed, mode, sealed).is_err(),
                "mode{bits} case{case}"
            );
        }
        let mut catalogue = opened.image.catalogue;
        let id = catalogue.files[0].info.id;
        let original_id = catalogue.forms.definitions[0].name.id;
        catalogue.forms.definitions[0].name.id = id;
        assert!(catalogue.tree_records().is_err());
        catalogue.forms.definitions[0].name.id = original_id;
        let stable = storage.inner.read_all().unwrap();
        assert!(!import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &def
        )
        .unwrap());
        assert_eq!(storage.inner.read_all().unwrap(), stable);
        let mut late = def.clone();
        late.fields.last_mut().unwrap().label = "Different last label".into();
        assert!(import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &late
        )
        .is_err());
        assert_eq!(storage.inner.read_all().unwrap(), stable);
        let mut damaged = storage.clone();
        let extent = catalogue.forms.definitions[0]
            .fields
            .last()
            .unwrap()
            .label
            .extents[0];
        let byte = damaged.inner.read_at(extent.start + 100, 1).unwrap()[0];
        damaged
            .inner
            .write_at(extent.start + 100, &[byte ^ 1])
            .unwrap();
        let damaged_before = damaged.inner.read_all().unwrap();
        assert!(import_definition(
            &mut damaged,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &def
        )
        .is_err());
        assert_eq!(damaged.inner.read_all().unwrap(), damaged_before);
        let mut large = def.clone();
        large.type_id = FormTypeId::new("11111111-1111-1111-1111-111111111111").unwrap();
        large.fields = (0..4096)
            .map(|n| FormFieldDefinition {
                id: format!("field_{n}"),
                label: "Field".into(),
                kind: FormFieldKind::Text,
                required: false,
            })
            .collect();
        assert!(matches!(
            import_definition(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &large
            ),
            Err(Error::SecurityLimitExceeded(_))
        ));
        assert_eq!(storage.inner.read_all().unwrap(), stable);
        large.fields.truncate(1);
        large.name = "x".repeat(1024 * 1024 + 1);
        assert!(import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &large
        )
        .is_err());
        assert_eq!(storage.inner.read_all().unwrap(), stable);
        let mut too_long = record.clone();
        too_long.path = LockboxPath::new("/refused").unwrap();
        too_long.values = vec![FormFieldValue {
            field_id: "text".into(),
            captured_label: "Label".into(),
            kind: FormFieldKind::Text,
            value: FormValue::Normal("x".repeat(1024 * 1024 + 1)),
        }];
        assert!(import_captured_record(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &too_long
        )
        .is_err());
        assert_eq!(storage.inner.read_all().unwrap(), stable);
        let oversized =
            Arc::new(SecretString::try_from_slice(&vec![b'x'; 1024 * 1024 + 1]).unwrap());
        too_long.values[0].kind = FormFieldKind::Secret;
        too_long.values[0].value = FormValue::Secret(oversized);
        assert!(import_captured_record(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &too_long
        )
        .is_err());
        assert_eq!(storage.inner.read_all().unwrap(), stable);
        drop(too_long);
        for (kind, value) in [
            (FormFieldKind::Url, "invalid"),
            (FormFieldKind::Email, "invalid"),
            (FormFieldKind::Date, "2026-99-99"),
            (FormFieldKind::Month, "2026-99"),
            (FormFieldKind::Number, "invalid"),
        ] {
            let mut invalid = record.clone();
            invalid.path = LockboxPath::new("/badkind").unwrap();
            invalid.values[0].kind = kind;
            invalid.values[0].value = FormValue::Normal(value.into());
            assert!(import_captured_record(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &invalid
            )
            .is_err());
            assert_eq!(storage.inner.read_all().unwrap(), stable);
        }
        // Authenticated raw-record fixture deliberately swaps complete stored extents
        // while keeping each semantic descriptor/context: union remains identical.
        let d = &mut catalogue.forms.definitions[0];
        assert_eq!(d.name.length, d.description.length);
        std::mem::swap(&mut d.name.extents, &mut d.description.extents);
        let rows = catalogue.tree_records().unwrap();
        shared::tree::rewrite_records(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            rows,
        )
        .unwrap();
        match AuditedTreeImage::open(storage, archive(), mode, &authority, key(mode)) {
            Err(_) => assert!(mode.plaintext() && mode.signed()),
            Ok(opened) => {
                assert!(!(mode.plaintext() && mode.signed()));
                assert!(opened
                    .get_form_definition(&def.type_id, def.revision)
                    .is_err());
            }
        }
    }
}
