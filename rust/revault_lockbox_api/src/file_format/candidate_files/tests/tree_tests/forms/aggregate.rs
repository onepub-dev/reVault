//! Linux fresh-process fixture. One caller secret is reused across distinct
//! captured fields; this does not claim all-values getter/concurrency capacity.
use super::super::variables::Guarded;
use super::*;
fn memory() -> serde_json::Value {
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    let field = |name: &str| {
        status
            .lines()
            .find_map(|l| l.strip_prefix(name))
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<u64>()
            .unwrap()
    };
    serde_json::json!({"vm_lck_kib":field("VmLck:"),"vm_rss_kib":field("VmRSS:"),"vm_hwm_kib":field("VmHWM:")})
}
#[test]
#[ignore = "fresh-process form aggregate endpoint probe"]
fn typed_form_aggregate_guarded_snapshot_probe() {
    run(false)
}
#[test]
#[ignore = "fresh-process selected-source aggregate endpoint probe"]
fn typed_form_selected_source_aggregate_probe() {
    run(true)
}
fn run(selected_source: bool) {
    let bits: usize = std::env::var("REVAULT_FORM_MODE").unwrap().parse().unwrap();
    assert!(bits < 16);
    let path = std::path::PathBuf::from(std::env::var_os("REVAULT_FORM_IMAGE").unwrap());
    assert!(path.is_absolute() && !path.exists());
    let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let signer = mode.signed().then_some(&owner);
    {
        let seed = super::super::mutation::seed(mode, &authority, &owner);
        std::fs::write(&path, seed.read_all().unwrap()).unwrap();
    }
    let before_source = memory();
    let size = 1024 * 1024;
    let source = Arc::new(SecretString::try_from_slice(&vec![b's'; size]).unwrap());
    let def = FormDefinition {
        type_id: FormTypeId::new("------------------------------------").unwrap(),
        alias: "aggregate".into(),
        revision: 1,
        name: "n".repeat(size),
        description: "d".repeat(size),
        fields: (0..8)
            .map(|n| FormFieldDefinition {
                id: format!("field_{n}"),
                label: if n == 0 {
                    "l".repeat(size)
                } else {
                    format!("Label {n}")
                },
                kind: FormFieldKind::Secret,
                required: false,
            })
            .collect(),
    };
    let record = FormRecord {
        path: LockboxPath::new("/aggregate/record").unwrap(),
        name: "r".repeat(size),
        type_id: def.type_id.clone(),
        definition_alias: def.alias.clone(),
        definition_revision: 1,
        values: def
            .fields
            .iter()
            .enumerate()
            .map(|(n, f)| FormFieldValue {
                field_id: f.id.clone(),
                captured_label: if n == 0 {
                    "c".repeat(size)
                } else {
                    f.label.clone()
                },
                kind: FormFieldKind::Secret,
                value: FormValue::Secret(source.clone()),
            })
            .collect(),
    };
    let after_source = memory();
    let mut storage = Guarded::new(StorageBackend::file_for_write(&path).unwrap(), Vec::new());
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
    let after_definition = memory();
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
    let after_record = memory();
    let ranges = storage.spans.borrow().clone();
    let record_path = record.path.clone();
    let type_id = def.type_id.clone();
    drop(storage);
    drop(record);
    drop(def);
    drop(source);
    let mut after_selected_source = None;
    let ranges = if selected_source {
        let writer = Guarded::new(StorageBackend::file_for_write(&path).unwrap(), ranges);
        let old = {
            let opened = TreeImage::open(
                crate::file_format::allocation_map::compaction::View(&writer),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap();
            opened
                .image
                .catalogue
                .forms
                .texts()
                .into_iter()
                .flat_map(|t| t.extents.iter().copied())
                .collect::<Vec<_>>()
        };
        let writer =
            super::selected_source::replace_without_clone(writer, mode, &authority, &owner)
                .unwrap();
        after_selected_source = Some(memory());
        for extent in old {
            if extent.start < writer.len().unwrap() {
                let len = extent.len.min(writer.len().unwrap() - extent.start) as usize;
                let bytes = writer.read_at_secure(extent.start, len).unwrap();
                bytes
                    .with_bytes(|b| assert!(b.iter().all(|b| *b == 0)))
                    .unwrap();
            }
        }
        let ranges = writer.spans.borrow().clone();
        drop(writer);
        ranges
    } else {
        ranges
    };
    let mut opened = TreeImage::open(
        Guarded::new(StorageBackend::file(&path).unwrap(), ranges),
        archive(),
        mode,
        &authority,
        key(mode),
    )
    .unwrap();
    opened.image.verify_all().unwrap();
    let definition = opened.get_form_definition(&type_id, 1).unwrap().unwrap();
    assert_eq!(definition.name.len(), size);
    assert!(definition.name.bytes().all(|b| b == b'n'));
    assert_eq!(definition.description.len(), size);
    assert!(definition.description.bytes().all(|b| b == b'd'));
    assert_eq!(definition.fields[0].label.len(), size);
    assert!(definition.fields[0].label.bytes().all(|b| b == b'l'));
    drop(definition);
    for n in 0..8 {
        assert_eq!(
            opened
                .with_form_field_value(&record_path, &format!("field_{n}"), |v| match v {
                    FormValue::Secret(v) => v
                        .with_str(|s| {
                            assert_eq!(s.len(), size);
                            assert!(s.bytes().all(|b| b == b's'));
                        })
                        .unwrap(),
                    _ => panic!("secret downgraded"),
                })
                .unwrap(),
            Some(())
        );
    }
    let texts = opened.image.catalogue.forms.texts();
    let logical_bytes: usize = texts.iter().map(|t| t.length).sum();
    let stored_payload_bytes: u64 = texts.iter().flat_map(|t| &t.extents).map(|e| e.len).sum();
    assert!(logical_bytes > 8 * size && stored_payload_bytes > 8 * size as u64);
    // Verify large record metadata individually; avoid materializing all8 secure
    // values via the full-object getter merely to inspect a normal name or label.
    let record = &opened.image.catalogue.forms.records[0];
    for (layout, byte) in [(&record.name, b'r'), (&record.fields[0].label, b'c')] {
        let bytes = layout
            .read(
                &opened.image.storage,
                archive(),
                opened.image.value_key.as_ref().unwrap(),
                opened.image.anchor.sealed_len,
            )
            .unwrap();
        bytes
            .with_bytes(|bytes| {
                assert_eq!(bytes.len(), size);
                assert!(bytes.iter().all(|b| *b == byte));
            })
            .unwrap();
    }
    let after_read = memory();

    let mut definitions = 0;
    let mut definition_fields = 0;
    let mut records = 0;
    let mut captures = 0;
    let mut ends = 0;
    let report = tree_image::forms::salvage_forms(
        &opened.image.storage,
        archive(),
        mode,
        &authority,
        key(mode),
        |event| {
            use tree_image::forms::SalvageEvent;
            let repeated = |text: &str, byte: u8| {
                assert_eq!(text.len(), size);
                assert!(text.bytes().all(|b| b == byte));
            };
            match event {
                SalvageEvent::DefinitionStart {
                    type_id: actual,
                    revision,
                    alias,
                    name,
                    description,
                    fields,
                } => {
                    assert_eq!(actual, type_id);
                    assert_eq!(revision, 1);
                    assert_eq!(alias, "aggregate");
                    assert_eq!(fields, 8);
                    repeated(&name, b'n');
                    repeated(&description, b'd');
                    definitions += 1;
                }
                SalvageEvent::DefinitionField(field) => {
                    assert_eq!(field.id, format!("field_{definition_fields}"));
                    assert_eq!(field.kind, FormFieldKind::Secret);
                    assert!(!field.required);
                    if definition_fields == 0 {
                        repeated(&field.label, b'l');
                    } else {
                        assert_eq!(field.label, format!("Label {definition_fields}"));
                    }
                    definition_fields += 1;
                }
                SalvageEvent::DefinitionEnd {
                    type_id: actual,
                    revision,
                } => {
                    assert_eq!(actual, type_id);
                    assert_eq!(revision, 1);
                    assert_eq!(definition_fields, 8);
                    ends += 1;
                }
                SalvageEvent::RecordStart {
                    path,
                    name,
                    type_id: actual,
                    definition_revision,
                    definition_alias,
                    fields,
                    ..
                } => {
                    assert_eq!(path, record_path);
                    assert_eq!(actual, type_id);
                    assert_eq!(definition_revision, 1);
                    assert_eq!(definition_alias, "aggregate");
                    assert_eq!(fields, 8);
                    repeated(&name, b'r');
                    records += 1;
                }
                SalvageEvent::CapturedField(field) => {
                    assert_eq!(field.field_id, format!("field_{captures}"));
                    assert_eq!(field.kind, FormFieldKind::Secret);
                    if captures == 0 {
                        repeated(&field.captured_label, b'c');
                    } else {
                        assert_eq!(field.captured_label, format!("Label {captures}"));
                    }
                    match field.value {
                        FormValue::Secret(secret) => secret.with_str(|s| repeated(s, b's'))?,
                        _ => panic!("secret downgraded"),
                    };
                    captures += 1;
                }
                SalvageEvent::RecordEnd { path, .. } => {
                    assert_eq!(path, record_path);
                    assert_eq!(captures, 8);
                    ends += 1;
                }
            }
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(report, tree_image::forms::SalvageReport::default());
    assert_eq!(
        (definitions, definition_fields, records, captures, ends),
        (1, 8, 1, 8, 2)
    );
    let after_salvage = memory();
    for metric in [
        &before_source,
        &after_source,
        &after_definition,
        &after_record,
        &after_read,
        &after_salvage,
    ] {
        assert!(metric["vm_lck_kib"].as_u64().unwrap() <= 8192);
    }
    if let Some(metric) = &after_selected_source {
        assert!(metric["vm_lck_kib"].as_u64().unwrap() <= 8192);
    }
    println!(
        "{} {}",
        if selected_source {
            "SELECTED_FORM_AGGREGATE"
        } else {
            "FORM_AGGREGATE"
        },
        serde_json::json!({"mode":bits,"source_secret_bytes":size,"secret_fields":8,"logical_bytes":logical_bytes,"stored_payload_bytes":stored_payload_bytes,"archive_bytes":opened.image.storage.len().unwrap(),"before_source":before_source,"after_source":after_source,"after_definition":after_definition,"after_record":after_record,"after_read":after_read,"after_salvage":after_salvage,"after_selected_source":after_selected_source})
    );
}
