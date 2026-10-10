//! Internal selected-tree fixtures: the public CLI cannot construct this adapter.
use super::*;
mod admission;
#[cfg(target_os = "linux")]
mod aggregate;
mod compaction;
mod installation;
mod lifecycle;
use crate::{
    FormDefinition, FormFieldDefinition, FormFieldKind, FormFieldValue, FormRecord, FormTypeId,
    FormValue, LockboxPath, SecretString, VariableName,
};
use std::sync::Arc;
use tree_image::forms::{import_captured_record, import_definition};
#[test]
fn typed_form_snapshots_large_metadata_history_and_retention_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let type_id = FormTypeId::new("------------------------------------").unwrap();
    let definition = FormDefinition {
        type_id: type_id.clone(),
        alias: "capture".into(),
        revision: 3,
        name: "n".repeat(70000),
        description: "d".repeat(80000),
        fields: vec![FormFieldDefinition {
            id: "current".into(),
            label: "l".repeat(65535) + "🦀",
            kind: FormFieldKind::Notes,
            required: false,
        }],
    };
    let secret =
        Arc::new(SecretString::try_from_slice(("s".repeat(65535) + "🦀").as_bytes()).unwrap());
    let record = FormRecord {
        path: LockboxPath::new("/docs/data").unwrap(),
        name: "Snapshot".into(),
        type_id: type_id.clone(),
        definition_alias: "capture".into(),
        definition_revision: 3,
        values: vec![
            FormFieldValue {
                field_id: "removed".into(),
                captured_label: "Historical".repeat(6000),
                kind: FormFieldKind::Text,
                value: FormValue::Normal("retained".into()),
            },
            FormFieldValue {
                field_id: "password".into(),
                captured_label: "Old password".into(),
                kind: FormFieldKind::Secret,
                value: FormValue::Secret(secret),
            },
        ],
    };
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = super::mutation::seed(mode, &authority, &owner);
        assert!(import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &definition
        )
        .unwrap());
        let stable = storage.read_all().unwrap();
        assert!(!import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &definition
        )
        .unwrap());
        assert_eq!(storage.read_all().unwrap(), stable);
        let mut conflict = definition.clone();
        conflict.name = "conflict".into();
        assert!(import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &conflict
        )
        .is_err());
        assert_eq!(storage.read_all().unwrap(), stable);
        assert!(import_captured_record(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &record
        )
        .unwrap());
        let mut child = record.clone();
        child.path = LockboxPath::new("/docs/data/child").unwrap();
        child.values.clear();
        assert!(import_captured_record(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &child
        )
        .unwrap());
        let mut nested = child.clone();
        nested.path = LockboxPath::new("/new/deep/record").unwrap();
        assert!(import_captured_record(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &nested
        )
        .unwrap());
        let stable = storage.read_all().unwrap();
        for bad in 0..5 {
            let mut invalid = record.clone();
            invalid.path = LockboxPath::new("/refused").unwrap();
            match bad {
                0 => invalid.definition_revision = 2,
                1 => invalid.definition_alias = "wrong".into(),
                2 => invalid.values.push(invalid.values[0].clone()),
                3 => invalid.values[1].kind = FormFieldKind::Text,
                _ => invalid.path = record.path.clone(),
            };
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
            assert_eq!(storage.read_all().unwrap(), stable);
        }
        assert!(tree_image::return_inline(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode)
        )
        .is_err());
        assert_eq!(storage.read_all().unwrap(), stable);
        let opened =
            AuditedTreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
                .unwrap();
        assert_eq!(
            opened.get_form_definition(&type_id, 3).unwrap(),
            Some(definition.clone())
        );
        assert_eq!(
            opened.get_form_record(&record.path).unwrap(),
            Some(record.clone())
        );
        assert_eq!(opened.get_form_record(&child.path).unwrap(), Some(child));
        assert!(opened
            .image
            .filesystem_metadata()
            .unwrap()
            .iter()
            .any(|m| m.entry.path.as_str() == "/new/deep"
                && m.entry.kind == crate::LockboxEntryKind::Directory));
        let selected =
            tree_image::TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
                .unwrap();
        assert_eq!(
            selected.get_form_definition(&type_id, 3).unwrap(),
            Some(definition.clone())
        );
        assert_eq!(
            selected.get_form_record(&record.path).unwrap(),
            Some(record.clone())
        );
        for field in &record.values {
            assert_eq!(
                selected
                    .with_form_field_value(&record.path, &field.field_id, |value| assert_eq!(
                        value,
                        &field.value
                    ))
                    .unwrap(),
                Some(())
            );
        }
        assert_eq!(
            selected
                .with_form_field_value(&record.path, "missing", |_| ())
                .unwrap(),
            None
        );
        drop(selected);
        let payloads: Vec<_> = opened
            .image
            .catalogue
            .forms
            .texts()
            .into_iter()
            .flat_map(|t| t.extents.clone())
            .collect();
        drop(opened);
        tree_image::variables::set_variable(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &VariableName::new("neighbor").unwrap(),
            "variable",
        )
        .unwrap();
        tree_image::update_files(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            vec![Input {
                path: b"/docs/data".to_vec(),
                reader: Cursor::new(b"replacement".to_vec()),
            }],
            &[],
        )
        .unwrap();
        let mut opened =
            AuditedTreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
                .unwrap();
        assert_eq!(
            opened.get_form_record(&record.path).unwrap(),
            Some(record.clone())
        );
        assert_eq!(
            opened.get_form_definition(&type_id, 3).unwrap(),
            Some(definition.clone())
        );
        assert_eq!(
            opened
                .image
                .catalogue
                .forms
                .texts()
                .into_iter()
                .flat_map(|t| t.extents.clone())
                .collect::<Vec<_>>(),
            payloads
        );
        let mut bytes = Vec::new();
        opened
            .image
            .read_range(b"/docs/neighbor", 0, 8, |p| {
                bytes.extend_from_slice(p);
                Ok(())
            })
            .unwrap();
        assert_eq!(bytes, b"neighbor");
    }
}

mod moves;

mod selected_source;

mod fields;

#[cfg(target_os = "linux")]
mod field_aggregate;

mod definitions;
