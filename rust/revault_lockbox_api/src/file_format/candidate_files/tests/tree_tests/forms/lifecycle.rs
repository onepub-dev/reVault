use super::super::variables::Guarded;
use super::*;
use tree_image::forms::{delete_record, salvage_forms, SalvageEvent, SalvageReport};

fn fixture(
    mode: FormatMode,
    authority: &Authority<'_>,
    owner: &OwnerSigningKeyPair,
) -> (
    Guarded<StorageBackend>,
    Vec<FormDefinition>,
    Vec<FormRecord>,
) {
    let mut storage = Guarded::new(
        super::super::mutation::seed(mode, authority, owner),
        Vec::new(),
    );
    tree_image::variables::set_variable(
        &mut storage,
        archive(),
        mode,
        authority,
        mode.signed().then_some(owner),
        key(mode),
        &VariableName::new("retained").unwrap(),
        "retained variable",
    )
    .unwrap();
    let definitions: Vec<_> = (0..2)
        .map(|n| FormDefinition {
            type_id: FormTypeId::new(format!("{n:036x}")).unwrap(),
            alias: format!("type{n}"),
            revision: 1,
            name: format!("Definition {n}"),
            description: "description".into(),
            fields: vec![FormFieldDefinition {
                id: "password".into(),
                label: "Password".into(),
                kind: FormFieldKind::Secret,
                required: false,
            }],
        })
        .collect();
    for d in &definitions {
        import_definition(
            &mut storage,
            archive(),
            mode,
            authority,
            mode.signed().then_some(owner),
            key(mode),
            d,
        )
        .unwrap();
    }
    let records: Vec<_> = (0..3)
        .map(|n| {
            let d = &definitions[usize::from(n == 2)];
            FormRecord {
                path: LockboxPath::new(format!("/forms/{n}")).unwrap(),
                name: format!("Record {n}"),
                type_id: d.type_id.clone(),
                definition_alias: d.alias.clone(),
                definition_revision: 1,
                values: vec![FormFieldValue {
                    field_id: "password".into(),
                    captured_label: "Historical password".into(),
                    kind: FormFieldKind::Secret,
                    value: FormValue::Secret(Arc::new(
                        SecretString::try_from_slice(format!("secret {n}").as_bytes()).unwrap(),
                    )),
                }],
            }
        })
        .collect();
    for r in &records {
        import_captured_record(
            &mut storage,
            archive(),
            mode,
            authority,
            mode.signed().then_some(owner),
            key(mode),
            r,
        )
        .unwrap();
    }
    (storage, definitions, records)
}
fn collect(
    storage: &impl Storage,
    mode: FormatMode,
    authority: &Authority<'_>,
) -> Result<(SalvageReport, Vec<FormDefinition>, Vec<FormRecord>)> {
    let mut definitions = Vec::new();
    let mut records = Vec::new();
    let mut definition: Option<(FormDefinition, usize)> = None;
    let mut record: Option<(FormRecord, [u8; 16], usize)> = None;
    let report = salvage_forms(storage, archive(), mode, authority, key(mode), |event| {
        match event {
            SalvageEvent::DefinitionStart {
                type_id,
                revision,
                alias,
                name,
                description,
                fields,
            } => {
                assert!(definition.is_none() && record.is_none());
                definition = Some((
                    FormDefinition {
                        type_id,
                        revision,
                        alias,
                        name,
                        description,
                        fields: Vec::new(),
                    },
                    fields,
                ));
            }
            SalvageEvent::DefinitionField(field) => {
                definition.as_mut().unwrap().0.fields.push(field)
            }
            SalvageEvent::DefinitionEnd { type_id, revision } => {
                let (d, count) = definition.take().unwrap();
                assert_eq!(d.type_id, type_id);
                assert_eq!(d.revision, revision);
                assert_eq!(d.fields.len(), count);
                definitions.push(d);
            }
            SalvageEvent::RecordStart {
                path,
                id,
                name,
                type_id,
                definition_revision,
                definition_alias,
                fields,
            } => {
                assert!(definition.is_none() && record.is_none());
                record = Some((
                    FormRecord {
                        path,
                        name,
                        type_id,
                        definition_revision,
                        definition_alias,
                        values: Vec::new(),
                    },
                    id,
                    fields,
                ));
            }
            SalvageEvent::CapturedField(field) => record.as_mut().unwrap().0.values.push(field),
            SalvageEvent::RecordEnd { path, id } => {
                let (r, expected, count) = record.take().unwrap();
                assert_eq!(r.path, path);
                assert_eq!(id, expected);
                assert_eq!(r.values.len(), count);
                records.push(r);
            }
        }
        Ok(())
    })?;
    assert!(definition.is_none() && record.is_none());
    Ok((report, definitions, records))
}
#[test]
fn typed_form_delete_and_stream_salvage_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (mut storage, definitions, records) = fixture(mode, &authority, &owner);
        let (report, actual_defs, actual_records) = collect(&storage, mode, &authority).unwrap();
        assert_eq!(report, SalvageReport::default());
        assert_eq!(actual_defs, definitions);
        assert_eq!(actual_records, records);
        let stable = storage.inner.read_all().unwrap();
        assert!(matches!(
            delete_record(
                &mut storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                &LockboxPath::new("/missing").unwrap()
            ),
            Err(Error::NotFound(_))
        ));
        assert_eq!(storage.inner.read_all().unwrap(), stable);
        let current =
            TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
        let form = &current.image.catalogue.forms.records[0];
        let mut retired = form.name.extents.clone();
        for f in &form.fields {
            retired.extend(f.label.extents.clone());
            retired.extend(f.value.extents.clone());
        }
        // One damaged captured value leaves its sibling and the independent definition usable.
        let mut damaged = storage.clone();
        let extent = form.fields[0].value.extents[0];
        let mut byte = damaged.inner.read_at(extent.start, 1).unwrap();
        byte[0] ^= 1;
        damaged.inner.write_at(extent.start, &byte).unwrap();
        let (report, defs, got) = collect(&damaged, mode, &authority).unwrap();
        assert_eq!(report.unavailable_records, vec![records[0].path.clone()]);
        assert!(report.unavailable_definitions.is_empty());
        assert_eq!(defs, definitions);
        assert_eq!(got, records[1..]);
        // An unavailable exact definition suppresses all and only its dependent records.
        let mut damaged = storage.clone();
        let extent = current.image.catalogue.forms.definitions[0].name.extents[0];
        let mut byte = damaged.inner.read_at(extent.start, 1).unwrap();
        byte[0] ^= 1;
        damaged.inner.write_at(extent.start, &byte).unwrap();
        let (report, defs, got) = collect(&damaged, mode, &authority).unwrap();
        assert_eq!(
            report.unavailable_definitions,
            vec![(definitions[0].type_id.clone(), 1)]
        );
        assert_eq!(
            report.unavailable_records,
            records[..2]
                .iter()
                .map(|r| r.path.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(defs, definitions[1..]);
        assert_eq!(got, records[2..]);
        let mut damaged = storage.clone();
        for start in [
            current.tree.anchor.index.primary,
            current.tree.anchor.index.mirror,
        ] {
            damaged
                .inner
                .write_at(start, &vec![0; current.tree.anchor.index.len as usize])
                .unwrap();
        }
        let mut calls = 0;
        assert!(
            salvage_forms(&damaged, archive(), mode, &authority, key(mode), |_| {
                calls += 1;
                Ok(())
            })
            .is_err()
        );
        assert_eq!(calls, 0);
        assert!(delete_record(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &records[0].path
        )
        .unwrap());
        let bytes = storage.inner.read_all().unwrap();
        for extent in retired {
            let start = extent.start as usize;
            if start < bytes.len() {
                let end = (extent.start + extent.len).min(bytes.len() as u64) as usize;
                assert!(bytes[start..end].iter().all(|b| *b == 0));
            }
        }
        let mut opened =
            TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode)).unwrap();
        assert!(opened.get_form_record(&records[0].path).unwrap().is_none());
        let mut neighbor = Vec::new();
        opened
            .image
            .read_range(b"/docs/neighbor", 0, 8, |part| {
                neighbor.extend_from_slice(part);
                Ok(())
            })
            .unwrap();
        assert_eq!(neighbor, b"neighbor");
        assert_eq!(
            opened
                .get_variable(&VariableName::new("retained").unwrap())
                .unwrap()
                .as_deref(),
            Some("retained variable")
        );
        for bank in [None, Some(0), Some(FAILURE_REGION)] {
            let mut damaged = storage.clone();
            if let Some(start) = bank {
                damaged
                    .inner
                    .write_at(start, &vec![0; FAILURE_REGION as usize])
                    .unwrap();
            }
            let (report, defs, got) = collect(&damaged, mode, &authority).unwrap();
            assert_eq!(report, SalvageReport::default());
            assert_eq!(defs, definitions);
            assert_eq!(got, records[1..]);
        }
        let mut starts = 0;
        let mut ends = 0;
        let result = salvage_forms(&storage, archive(), mode, &authority, key(mode), |event| {
            match event {
                SalvageEvent::RecordStart { .. } => starts += 1,
                SalvageEvent::CapturedField(_) => return Err(Error::Io("sink failed".into())),
                SalvageEvent::RecordEnd { .. } => ends += 1,
                _ => {}
            }
            Ok(())
        });
        assert!(matches!(result, Err(Error::Io(_))));
        assert_eq!(starts, 1);
        assert_eq!(ends, 0);
    }
}

#[test]
fn typed_form_deletion_guarded_atomic_recovery_faults() {
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    let mut recovery_cases = 0;
    for bits in [0, 1, 2, 3, 12, 13, 14, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (original, definitions, records) = fixture(mode, &authority, &owner);
        let bytes = original.inner.read_all().unwrap();
        let spans = original.spans.borrow().clone();
        let opened =
            TreeImage::open(original.clone(), archive(), mode, &authority, key(mode)).unwrap();
        let removed = &opened.image.catalogue.forms.records[0];
        let mut retired = removed.name.extents.clone();
        for field in &removed.fields {
            retired.extend(field.label.extents.clone());
            retired.extend(field.value.extents.clone());
        }
        let run = |s: &mut Guarded<CrashStore>| {
            delete_record(
                s,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                &records[0].path,
            )
        };
        let verify = |s: &Guarded<StorageBackend>| {
            let (report, defs, got) = collect(s, mode, &authority).unwrap();
            assert_eq!(report, SalvageReport::default());
            assert_eq!(defs, definitions);
            let committed = got == records[1..];
            assert!(committed || got == records);
            let mut opened =
                TreeImage::open(s.clone(), archive(), mode, &authority, key(mode)).unwrap();
            let mut neighbor = Vec::new();
            opened
                .image
                .read_range(b"/docs/neighbor", 0, 8, |part| {
                    neighbor.extend_from_slice(part);
                    Ok(())
                })
                .unwrap();
            assert_eq!(neighbor, b"neighbor");
            assert_eq!(
                opened
                    .get_variable(&VariableName::new("retained").unwrap())
                    .unwrap()
                    .as_deref(),
                Some("retained variable")
            );
            let recovered = s.inner.read_all().unwrap();
            if committed {
                for extent in &retired {
                    let start = extent.start as usize;
                    if start < recovered.len() {
                        let end = (extent.start + extent.len).min(recovered.len() as u64) as usize;
                        assert!(recovered[start..end].iter().all(|b| *b == 0));
                    }
                }
            } else if recovered.len() > bytes.len() {
                assert!(recovered[bytes.len()..].iter().all(|b| *b == 0));
            }
            committed
        };
        let mut observed = Guarded::new(
            CrashStore::new(bytes.clone(), None, 0, false),
            spans.clone(),
        );
        run(&mut observed).unwrap();
        let count = observed.inner.operations();
        for at in 0..count {
            for prefix in [0, 97, usize::MAX] {
                for persist in [false, true] {
                    let mut failed = Guarded::new(
                        CrashStore::new(bytes.clone(), Some(at), prefix, persist),
                        spans.clone(),
                    );
                    let _ = run(&mut failed);
                    let damaged = failed.inner.durable();
                    let tracked = failed.spans.borrow().clone();
                    let mut recovered =
                        Guarded::new(StorageBackend::memory(damaged.clone()), tracked.clone());
                    tree_image::recover(&mut recovered, archive(), mode, &authority, key(mode))
                        .unwrap_or_else(|e| {
                            panic!("mode={bits} at={at} prefix={prefix} persist={persist}: {e}")
                        });
                    let selected = verify(&recovered);
                    tree_image::recover(&mut recovered, archive(), mode, &authority, key(mode))
                        .unwrap();
                    assert_eq!(verify(&recovered), selected);
                    cases += 1;
                    if prefix == 97 && persist && [0, count / 2, count - 1].contains(&at) {
                        let mut observed = Guarded::new(
                            CrashStore::new(damaged.clone(), None, 0, false),
                            tracked.clone(),
                        );
                        tree_image::recover(&mut observed, archive(), mode, &authority, key(mode))
                            .unwrap();
                        for cut in 0..observed.inner.operations() {
                            for written in [0, 97, usize::MAX] {
                                let mut interrupted = Guarded::new(
                                    CrashStore::new(damaged.clone(), Some(cut), written, true),
                                    tracked.clone(),
                                );
                                let _ = tree_image::recover(
                                    &mut interrupted,
                                    archive(),
                                    mode,
                                    &authority,
                                    key(mode),
                                );
                                let mut resumed = Guarded::new(
                                    StorageBackend::memory(interrupted.inner.durable()),
                                    interrupted.spans.borrow().clone(),
                                );
                                tree_image::recover(
                                    &mut resumed,
                                    archive(),
                                    mode,
                                    &authority,
                                    key(mode),
                                )
                                .unwrap();
                                assert_eq!(verify(&resumed), selected);
                                recovery_cases += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    println!("TYPED_FORM_DELETE_FAULTS cases={cases} interrupted_recovery={recovery_cases}");
}

// Interior mutability is test-only adversarial storage. It injects observable
// delivery I/O errors or selected-control loss; it does not promise detection of
// arbitrary in-place mutation after the final read of an unrelated payload.
#[derive(Clone, Debug)]
struct DeliveryStore {
    inner: std::cell::RefCell<Guarded<StorageBackend>>,
    fail: std::cell::Cell<Option<(u64, u64)>>,
}
impl Storage for DeliveryStore {
    fn len(&self) -> Result<u64> {
        self.inner.borrow().len()
    }
    fn read_at(&self, at: u64, len: usize) -> Result<Vec<u8>> {
        self.inner.borrow().read_at(at, len)
    }
    fn read_at_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        if self
            .fail
            .get()
            .is_some_and(|(start, end)| at < end && start < at + out.len() as u64)
        {
            return Err(Error::Io("injected delivery read".into()));
        }
        self.inner.borrow().read_at_into(at, out)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        self.inner.get_mut().append(bytes)
    }
    fn write_at(&mut self, at: u64, bytes: &[u8]) -> Result<()> {
        self.inner.get_mut().write_at(at, bytes)
    }
    fn truncate(&mut self, len: u64) -> Result<()> {
        self.inner.get_mut().truncate(len)
    }
    fn sync(&self) -> Result<()> {
        self.inner.borrow().sync()
    }
}
#[test]
fn typed_form_stream_delivery_errors_are_fatal_after_partial_events() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in [0, 1, 2, 3, 12, 13, 14, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (original, _, _) = fixture(mode, &authority, &owner);
        let opened =
            TreeImage::open(original.clone(), archive(), mode, &authority, key(mode)).unwrap();
        let extent = opened.image.catalogue.forms.records[0].fields[0]
            .value
            .extents[0];
        let storage = DeliveryStore {
            inner: std::cell::RefCell::new(original.clone()),
            fail: std::cell::Cell::new(None),
        };
        let mut starts = 0;
        let mut ends = 0;
        let result = salvage_forms(&storage, archive(), mode, &authority, key(mode), |event| {
            match event {
                SalvageEvent::RecordStart { .. } => {
                    starts += 1;
                    storage
                        .fail
                        .set(Some((extent.start, extent.start + extent.len)));
                }
                SalvageEvent::RecordEnd { .. } => ends += 1,
                _ => {}
            }
            Ok(())
        });
        assert!(matches!(result, Err(Error::Io(_))));
        assert_eq!(starts, 1);
        assert_eq!(ends, 0);
        let storage = DeliveryStore {
            inner: std::cell::RefCell::new(original),
            fail: std::cell::Cell::new(None),
        };
        let mut starts = 0;
        let mut fields = 0;
        let mut ends = 0;
        let result = salvage_forms(&storage, archive(), mode, &authority, key(mode), |event| {
            match event {
                SalvageEvent::RecordStart { .. } => starts += 1,
                SalvageEvent::CapturedField(_) => {
                    fields += 1;
                    for bank in [0, FAILURE_REGION] {
                        storage
                            .inner
                            .borrow_mut()
                            .inner
                            .write_at(bank, &vec![0; FAILURE_REGION as usize])?;
                    }
                }
                SalvageEvent::RecordEnd { .. } => ends += 1,
                _ => {}
            }
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(starts, 1);
        assert_eq!(fields, 1);
        assert_eq!(ends, 0);
    }
}
