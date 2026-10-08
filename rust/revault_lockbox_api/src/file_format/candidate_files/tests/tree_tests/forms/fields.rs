//! Public CLI cannot construct this experimental adapter; compare reopened records.
use super::lifecycle::{collect, fixture};
use super::*;
use tree_image::forms::set_field;
fn bytes(storage: &impl Storage) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let len = storage.len().unwrap();
    let mut hash = Sha256::new();
    hash.update(len.to_le_bytes());
    let mut at = 0;
    while at < len {
        let n = (len - at).min(65536) as usize;
        storage
            .read_at_secure(at, n)
            .unwrap()
            .with_bytes(|b| hash.update(b))
            .unwrap();
        at += n as u64;
    }
    hash.finalize().into()
}
#[test]
fn typed_form_field_upgrade_mixed_history_and_context_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (mut storage, mut definitions, mut expected) = fixture(mode, &authority, &owner);
        let mut latest = definitions[0].clone();
        latest.revision = 2;
        latest.fields[0].kind = FormFieldKind::Text;
        latest.fields[0].label = "Current label".into();
        import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &latest,
        )
        .unwrap();
        definitions.push(latest.clone());
        // Import a latest-normal historical snapshot, not a schema-edit API exercise.
        let mut mixed = expected[0].clone();
        mixed.path = LockboxPath::new("/forms/mixed").unwrap();
        mixed.definition_revision = 2;
        mixed.values[0].kind = FormFieldKind::Text;
        mixed.values[0].value = FormValue::normal("normal neighbor");
        mixed.values.push(FormFieldValue {
            field_id: "removed".into(),
            captured_label: "Retained removed capture".into(),
            kind: FormFieldKind::Notes,
            value: FormValue::normal("untouched\nnotes"),
        });
        import_captured_record(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &mixed,
        )
        .unwrap();
        expected.push(mixed);
        // Historical secret can be replaced normally under the selected normal schema.
        let before = TreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let old = before
            .image
            .catalogue
            .forms
            .records
            .iter()
            .find(|r| r.path == expected[0].path)
            .unwrap()
            .fields[0]
            .value
            .clone();
        drop(before);
        assert!(set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &expected[0].path,
            "password",
            &FormValue::normal("secret 0")
        )
        .unwrap());
        expected[0].definition_revision = 2;
        expected[0].values[0].kind = FormFieldKind::Text;
        expected[0].values[0].captured_label = "Current label".into();
        expected[0].values[0].value = FormValue::normal("secret 0");
        let reopened = TreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let normal = reopened
            .image
            .catalogue
            .forms
            .records
            .iter()
            .find(|r| r.path == expected[0].path)
            .unwrap()
            .fields[0]
            .value
            .clone();
        assert_eq!(old.id, normal.id);
        assert_eq!(normal.revision, old.revision + 1);
        assert_ne!(normal.context, old.context);
        assert_ne!(normal.extents, old.extents);
        drop(reopened);
        let mut absent = expected[0].clone();
        absent.path = LockboxPath::new("/forms/absent").unwrap();
        absent.values.clear();
        import_captured_record(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &absent,
        )
        .unwrap();
        expected.push(absent);
        let value = FormValue::secret(SecretString::try_from_slice(b"secret 0").unwrap());
        assert!(set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &expected[0].path,
            "password",
            &value
        )
        .unwrap());
        latest.revision = 3;
        latest.fields[0].kind = FormFieldKind::Secret;
        definitions.push(latest.clone());
        for r in &mut expected {
            if r.type_id == latest.type_id {
                r.definition_revision = 3;
                for f in r.values.iter_mut().filter(|f| f.field_id == "password") {
                    f.kind = FormFieldKind::Secret;
                    f.captured_label = "Current label".into();
                    if let FormValue::Normal(s) = &f.value {
                        f.value =
                            FormValue::secret(SecretString::try_from_slice(s.as_bytes()).unwrap());
                    }
                }
            }
        }
        let reopened = TreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let secret = &reopened
            .image
            .catalogue
            .forms
            .records
            .iter()
            .find(|r| r.path == expected[0].path)
            .unwrap()
            .fields[0]
            .value;
        assert_eq!(secret.id, normal.id);
        assert_eq!(secret.revision, normal.revision + 1);
        assert_ne!(secret.context, normal.context);
        assert_ne!(secret.extents, normal.extents);
        for r in &expected {
            assert_eq!(reopened.get_form_record(&r.path).unwrap().as_ref(), Some(r));
        }
        for d in &definitions {
            assert_eq!(
                reopened
                    .get_form_definition(&d.type_id, d.revision)
                    .unwrap()
                    .as_ref(),
                Some(d)
            );
        }
        drop(reopened);
        let snapshot = bytes(&storage);
        assert!(!set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &expected[0].path,
            "password",
            &value
        )
        .unwrap());
        assert_eq!(snapshot, bytes(&storage));
        assert!(matches!(
            set_field(
                &mut storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                &expected[0].path,
                "password",
                &FormValue::normal("secret 0")
            ),
            Err(Error::InvalidOperation(_))
        ));
        assert_eq!(snapshot, bytes(&storage));
        let (_, _, records) = collect(&storage, mode, &authority).unwrap();
        for r in expected {
            assert!(records.contains(&r));
        }
    }
}

#[test]
fn typed_form_field_label_reference_only_and_kind_validation_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (mut storage, definitions, records) = fixture(mode, &authority, &owner);
        let target = &records[0];
        let value = &target.values[0].value;
        let layouts = |storage: &_| {
            let opened = TreeImage::open(
                crate::file_format::allocation_map::compaction::View(storage),
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
                .iter()
                .map(|l| l.encode_metadata())
                .collect::<Vec<_>>()
        };
        let before = TreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap()
        .image
        .catalogue
        .forms
        .records
        .into_iter()
        .find(|r| r.path == target.path)
        .unwrap();
        // First setter only catches label up; value extent stays exact.
        assert!(set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &target.path,
            "password",
            value
        )
        .unwrap());
        let after = TreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap()
        .image
        .catalogue
        .forms
        .records
        .into_iter()
        .find(|r| r.path == target.path)
        .unwrap();
        assert_eq!(before.fields[0].value, after.fields[0].value);
        assert_ne!(before.fields[0].label, after.fields[0].label);
        let mut latest = definitions[0].clone();
        latest.revision = 2;
        latest.alias = "newalias".into();
        import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &latest,
        )
        .unwrap();
        let before = layouts(&storage);
        assert!(set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &target.path,
            "password",
            value
        )
        .unwrap());
        assert_eq!(before, layouts(&storage));
        let reopened = TreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let r = reopened.get_form_record(&target.path).unwrap().unwrap();
        assert_eq!(r.definition_revision, 2);
        assert_eq!(r.definition_alias, "newalias");
        assert_eq!(
            reopened
                .get_form_record(&records[1].path)
                .unwrap()
                .unwrap()
                .definition_revision,
            1
        );
        drop(reopened);
        let snapshot = bytes(&storage);
        assert!(!set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &target.path,
            "password",
            value
        )
        .unwrap());
        assert_eq!(snapshot, bytes(&storage));
        latest.revision = 3;
        latest.fields[0].kind = FormFieldKind::Url;
        import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &latest,
        )
        .unwrap();
        let snapshot = bytes(&storage);
        assert!(matches!(
            set_field(
                &mut storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                &target.path,
                "password",
                &FormValue::normal("not a URL")
            ),
            Err(Error::InvalidInput(_))
        ));
        assert_eq!(snapshot, bytes(&storage));
        // Secret upgrade validates under Secret, not the superseded URL validator.
        assert!(set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &target.path,
            "password",
            value
        )
        .unwrap());
        let reopened = TreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let r = reopened.get_form_record(&target.path).unwrap().unwrap();
        assert_eq!(r.definition_revision, 4);
        assert_eq!(r.values[0].value, *value);
        assert_eq!(
            reopened
                .get_form_definition(&latest.type_id, 4)
                .unwrap()
                .unwrap()
                .fields[0]
                .kind,
            FormFieldKind::Secret
        );
    }
}
#[test]
fn typed_form_field_limits_refusals_and_verified_no_change_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (mut storage, definitions, records) = fixture(mode, &authority, &owner);
        let path = &records[0].path;
        let mut latest = definitions[0].clone();
        latest.revision = 2;
        latest.fields[0].kind = FormFieldKind::Notes;
        import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &latest,
        )
        .unwrap();
        let snapshot = bytes(&storage);
        for bad in [
            FormValue::normal("x".repeat(1048577)),
            FormValue::normal("embedded\0nul"),
            FormValue::secret(SecretString::try_from_slice(b"bad\0secret").unwrap()),
        ] {
            assert!(set_field(
                &mut storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                path,
                "password",
                &bad
            )
            .is_err());
            assert_eq!(snapshot, bytes(&storage));
        }
        assert!(matches!(
            set_field(
                &mut storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                path,
                "missing",
                &FormValue::normal("x")
            ),
            Err(Error::InvalidInput(_))
        ));
        assert_eq!(snapshot, bytes(&storage));
        assert!(matches!(
            set_field(
                &mut storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                &LockboxPath::new("/absent").unwrap(),
                "password",
                &FormValue::normal("x")
            ),
            Err(Error::NotFound(_))
        ));
        assert_eq!(snapshot, bytes(&storage));
        let text = "x".repeat(65535) + "🦀" + &"y".repeat(1048576 - 65539);
        let normal = FormValue::normal(&text);
        assert!(set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            path,
            "password",
            &normal
        )
        .unwrap());
        let reopened = TreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        assert_eq!(
            reopened.get_form_record(path).unwrap().unwrap().values[0].value,
            normal
        );
        drop(reopened);
        let secret = FormValue::secret(SecretString::try_from_slice(text.as_bytes()).unwrap());
        assert!(set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            path,
            "password",
            &secret
        )
        .unwrap());
        let reopened = TreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        assert_eq!(
            reopened.get_form_record(path).unwrap().unwrap().values[0].value,
            secret
        );
        let extent = reopened
            .image
            .catalogue
            .forms
            .records
            .iter()
            .find(|r| &r.path == path)
            .unwrap()
            .fields[0]
            .value
            .extents[0];
        drop(reopened);
        let snapshot = bytes(&storage);
        assert!(!set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            path,
            "password",
            &secret
        )
        .unwrap());
        assert_eq!(snapshot, bytes(&storage));
        // No-change still authenticates stored values before returning success.
        let at = extent.start + extent.len / 2;
        let old = storage.inner.read_at(at, 1).unwrap()[0];
        storage.inner.write_at(at, &[old ^ 1]).unwrap();
        let corrupt = bytes(&storage);
        assert!(set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            path,
            "password",
            &secret
        )
        .is_err());
        assert_eq!(corrupt, bytes(&storage));
        storage.inner.write_at(at, &[old]).unwrap();
        // Valid authenticated maximum definition revision refuses an upgrade atomically.
        latest.revision = u32::MAX;
        import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &latest,
        )
        .unwrap();
        let snapshot = bytes(&storage);
        assert!(matches!(
            set_field(
                &mut storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                path,
                "password",
                &secret
            ),
            Err(Error::SecurityLimitExceeded(_))
        ));
        assert_eq!(snapshot, bytes(&storage));
    }
}
// Authenticated fixture: render a valid maximum-revision payload through the
// existing writer, rather than changing a descriptor without its page identity.
fn install_max_text_revision(
    storage: &mut impl Storage,
    mode: FormatMode,
    authority: &Authority<'_>,
    owner: &OwnerSigningKeyPair,
    path: &LockboxPath,
) {
    use crate::file_format::{form_segments, publication_anchor::shared::tree};
    let opened = TreeImage::open(
        crate::file_format::allocation_map::compaction::View(&*storage),
        archive(),
        mode,
        authority,
        key(mode),
    )
    .unwrap();
    let old = opened
        .image
        .catalogue
        .forms
        .records
        .iter()
        .find(|r| &r.path == path)
        .unwrap()
        .fields[0]
        .value
        .clone();
    let base = shared::commitment(&opened.image.anchor).unwrap();
    let content_key = opened.image.value_key.unwrap();
    let mut catalogue = opened.image.catalogue;
    let source = tree::SelectedSource::open(
        &*storage,
        archive(),
        mode,
        authority,
        key(mode),
        base,
        old.extents.clone(),
    )
    .unwrap();
    let prepared = form_segments::prepare_stored(
        &old,
        source.view(&*storage),
        archive(),
        mode,
        &content_key,
        old.id,
        old.context,
        u64::MAX,
        old.sensitivity,
    )
    .unwrap();
    *catalogue
        .forms
        .texts_mut()
        .into_iter()
        .find(|l| l.id == old.id)
        .unwrap() = prepared.layout;
    let records = catalogue.tree_records().unwrap();
    let id = old.id;
    let plan = tree::PreparedStoragePayloadPlan {
        source,
        retired: old.extents,
        pages: prepared.pages,
        payload: prepared.payload,
        rebind: Box::new(move |extents| {
            catalogue
                .forms
                .texts_mut()
                .into_iter()
                .find(|l| l.id == id)
                .ok_or(Error::CorruptRecord)?
                .rebind(extents)?;
            catalogue.tree_records()
        }),
    };
    tree::rewrite_prepared_storage_payload_records(
        storage,
        archive(),
        mode,
        authority,
        mode.signed().then_some(owner),
        key(mode),
        records,
        plan,
    )
    .unwrap();
}
#[test]
fn typed_form_field_checked_text_revision_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (mut storage, _, records) = fixture(mode, &authority, &owner);
        let path = &records[0].path;
        let value = &records[0].values[0].value;
        set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            path,
            "password",
            value,
        )
        .unwrap();
        install_max_text_revision(&mut storage, mode, &authority, &owner, path);
        let snapshot = bytes(&storage);
        assert!(!set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            path,
            "password",
            value
        )
        .unwrap());
        assert_eq!(snapshot, bytes(&storage));
        let changed = FormValue::secret(SecretString::try_from_slice(b"replacement").unwrap());
        assert!(matches!(
            set_field(
                &mut storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                path,
                "password",
                &changed
            ),
            Err(Error::SecurityLimitExceeded(_))
        ));
        assert_eq!(snapshot, bytes(&storage));
        let reopened = TreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        assert_eq!(
            reopened.get_form_record(path).unwrap().unwrap().values[0].value,
            *value
        );
    }
}
#[test]
fn typed_form_field_upgrade_guarded_atomic_recovery_faults() {
    use super::super::variables::Guarded;
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    let mut recovery_cases = 0;
    for bits in [0, 1, 2, 3, 12, 13, 14, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (mut original, definitions, records) = fixture(mode, &authority, &owner);
        let path = &records[0].path;
        let mut latest = definitions[0].clone();
        latest.revision = 2;
        latest.fields[0].kind = FormFieldKind::Notes;
        latest.fields[0].label = "Upgrade label".into();
        import_definition(
            &mut original,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &latest,
        )
        .unwrap();
        set_field(
            &mut original,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            path,
            "password",
            &FormValue::normal("normal target"),
        )
        .unwrap();
        let mut neighbor = records[0].clone();
        neighbor.path = LockboxPath::new("/normal-neighbor").unwrap();
        neighbor.definition_revision = 2;
        neighbor.values[0].kind = FormFieldKind::Notes;
        neighbor.values[0].value = FormValue::normal("normal neighbor");
        neighbor.values.push(FormFieldValue {
            field_id: "removed".into(),
            captured_label: "Retained".into(),
            kind: FormFieldKind::Text,
            value: FormValue::normal("retain"),
        });
        import_captured_record(
            &mut original,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &neighbor,
        )
        .unwrap();
        neighbor.path = LockboxPath::new("/absent-field").unwrap();
        neighbor.values.clear();
        import_captured_record(
            &mut original,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &neighbor,
        )
        .unwrap();
        let (_, old_defs, old_records) = collect(&original, mode, &authority).unwrap();
        let opened = TreeImage::open(
            crate::file_format::allocation_map::compaction::View(&original),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let old_texts = opened
            .image
            .catalogue
            .forms
            .texts()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        let old_ids = opened
            .image
            .catalogue
            .forms
            .records
            .iter()
            .map(|r| (r.path.clone(), r.id))
            .collect::<Vec<_>>();
        drop(opened);
        let image = original.inner.read_all().unwrap();
        let spans = original.spans.borrow().clone();
        let override_value =
            FormValue::secret(SecretString::try_from_slice(b"override wins").unwrap());
        let mut successful = Guarded::new(StorageBackend::memory(image.clone()), spans.clone());
        set_field(
            &mut successful,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            path,
            "password",
            &override_value,
        )
        .unwrap();
        let (_, new_defs, new_records) = collect(&successful, mode, &authority).unwrap();
        assert_eq!(
            new_records.iter().find(|r| &r.path == path).unwrap().values[0].value,
            override_value
        );
        assert!(new_records
            .iter()
            .find(|r| r.path.as_str() == "/absent-field")
            .unwrap()
            .values
            .is_empty());
        let normal_neighbor = new_records
            .iter()
            .find(|r| r.path.as_str() == "/normal-neighbor")
            .unwrap();
        match &normal_neighbor.values[0].value {
            FormValue::Secret(v) => v.with_str(|s| assert_eq!(s, "normal neighbor")).unwrap(),
            _ => panic!("neighbor did not upgrade"),
        };
        assert_eq!(normal_neighbor.values[1].value, FormValue::normal("retain"));
        let verify = |s: &Guarded<StorageBackend>| {
            let (report, defs, got) = collect(s, mode, &authority).unwrap();
            assert_eq!(report, tree_image::forms::SalvageReport::default());
            let committed = defs.len() == new_defs.len();
            assert_eq!(
                defs,
                if committed {
                    new_defs.clone()
                } else {
                    old_defs.clone()
                }
            );
            assert_eq!(
                got,
                if committed {
                    new_records.clone()
                } else {
                    old_records.clone()
                }
            );
            let mut opened = TreeImage::open(
                crate::file_format::allocation_map::compaction::View(s),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap();
            for (path, id) in &old_ids {
                assert_eq!(
                    opened
                        .image
                        .catalogue
                        .forms
                        .records
                        .iter()
                        .find(|r| &r.path == path)
                        .unwrap()
                        .id,
                    *id
                );
            }
            let current = opened.image.catalogue.forms.texts();
            let bytes = s.inner.read_all().unwrap();
            for old in &old_texts {
                {
                    let new = current
                        .iter()
                        .find(|l| l.id == old.id)
                        .expect("existing text identity retained");
                    if !committed {
                        assert_eq!(*new, old);
                    } else {
                        assert!(new.revision == old.revision || new.revision == old.revision + 1);
                        if new.revision == old.revision {
                            assert_eq!(*new, old);
                        }
                    }
                }
                for extent in &old.extents {
                    if !current.iter().any(|l| l.extents.contains(extent)) {
                        let a = extent.start as usize;
                        let b = (extent.start + extent.len).min(bytes.len() as u64) as usize;
                        if a < bytes.len() {
                            assert!(bytes[a..b].iter().all(|b| *b == 0));
                        }
                    }
                }
            }
            if !committed {
                for (a, b) in s.spans.borrow().iter().skip(spans.len()) {
                    if (*a as usize) < bytes.len() {
                        assert!(bytes[*a as usize..(*b as usize).min(bytes.len())]
                            .iter()
                            .all(|b| *b == 0));
                    }
                }
                if bytes.len() > image.len() {
                    assert!(bytes[image.len()..].iter().all(|b| *b == 0));
                }
            }
            assert_eq!(
                opened
                    .get_variable(&VariableName::new("retained").unwrap())
                    .unwrap()
                    .as_deref(),
                Some("retained variable")
            );
            let mut file = Vec::new();
            opened
                .image
                .read_range(b"/docs/neighbor", 0, 8, |b| {
                    file.extend_from_slice(b);
                    Ok(())
                })
                .unwrap();
            assert_eq!(file, b"neighbor");
            committed
        };
        let mut observed = Guarded::new(
            CrashStore::new(image.clone(), None, 0, false),
            spans.clone(),
        );
        set_field(
            &mut observed,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            path,
            "password",
            &override_value,
        )
        .unwrap();
        let count = observed.inner.operations();
        for at in 0..count {
            for prefix in [0, 97, usize::MAX] {
                for persist in [false, true] {
                    let mut failed = Guarded::new(
                        CrashStore::new(image.clone(), Some(at), prefix, persist),
                        spans.clone(),
                    );
                    let _ = set_field(
                        &mut failed,
                        archive(),
                        mode,
                        &authority,
                        mode.signed().then_some(&owner),
                        key(mode),
                        path,
                        "password",
                        &override_value,
                    );
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
    println!("FORM_FIELD_UPGRADE_FAULTS cases={cases} interrupted_recovery={recovery_cases}");
}
#[test]
fn typed_form_field_move_delete_and_selected_salvage_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let (mut storage, _, records) = fixture(mode, &authority, &owner);
        let source = &records[0].path;
        let destination = LockboxPath::new("/moved/updated").unwrap();
        let value = FormValue::secret(SecretString::try_from_slice(b"updated then moved").unwrap());
        assert!(set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            source,
            "password",
            &value
        )
        .unwrap());
        let before = TreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap()
        .image
        .catalogue
        .forms
        .records
        .into_iter()
        .find(|r| &r.path == source)
        .unwrap();
        assert!(tree_image::forms::move_records(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &[(source.clone(), destination.clone())]
        )
        .unwrap());
        let snapshot = bytes(&storage);
        assert!(!set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &destination,
            "password",
            &value
        )
        .unwrap());
        assert_eq!(snapshot, bytes(&storage));
        let opened = TreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let moved = opened
            .image
            .catalogue
            .forms
            .records
            .iter()
            .find(|r| r.path == destination)
            .unwrap();
        assert_eq!(moved.id, before.id);
        assert_eq!(moved.fields[0].value, before.fields[0].value);
        assert!(opened.get_form_record(source).unwrap().is_none());
        assert_eq!(
            opened
                .get_form_record(&destination)
                .unwrap()
                .unwrap()
                .values[0]
                .value,
            value
        );
        drop(opened);
        assert!(tree_image::forms::delete_record(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &destination
        )
        .unwrap());
        let (report, definitions, selected) = collect(&storage, mode, &authority).unwrap();
        assert_eq!(report, tree_image::forms::SalvageReport::default());
        assert_eq!(definitions.len(), 2);
        assert_eq!(selected, records[1..]);
        let snapshot = bytes(&storage);
        assert!(matches!(
            set_field(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &destination,
                "password",
                &value
            ),
            Err(Error::NotFound(_))
        ));
        assert_eq!(snapshot, bytes(&storage));
    }
}
