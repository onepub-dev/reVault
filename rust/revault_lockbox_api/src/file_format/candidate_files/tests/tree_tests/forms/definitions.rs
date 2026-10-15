use super::super::variables::Guarded;
use super::*;
use tree_image::forms::{
    create_record, mutate_definition, set_field, AssignedDefinition, DefinitionTarget,
};
fn fields(kind: FormFieldKind) -> Vec<FormFieldDefinition> {
    vec![
        FormFieldDefinition {
            id: "value".into(),
            label: "Value".into(),
            kind,
            required: true,
        },
        FormFieldDefinition {
            id: "keep".into(),
            label: "Keep".into(),
            kind: FormFieldKind::Text,
            required: false,
        },
    ]
}
fn identity(s: &impl Storage) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let len = s.len().unwrap();
    let mut h = Sha256::new();
    h.update(len.to_le_bytes());
    let mut at = 0;
    while at < len {
        let n = (len - at).min(65536) as usize;
        s.read_at_secure(at, n)
            .unwrap()
            .with_bytes(|b| h.update(b))
            .unwrap();
        at += n as u64;
    }
    h.finalize().into()
}
#[test]
fn typed_form_definition_revision_resolver_and_empty_create_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let id = FormTypeId::new("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = Guarded::new(
            super::super::mutation::seed(mode, &authority, &owner),
            Vec::new(),
        );
        let assigned: AssignedDefinition = mutate_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            DefinitionTarget::Explicit(&id, "base"),
            "Name",
            "Description",
            &fields(FormFieldKind::Text),
        )
        .unwrap();
        assert_eq!(assigned.revision, 1);
        assert_eq!(assigned.type_id, id);
        let first = LockboxPath::new("/new/deep/first").unwrap();
        let second = LockboxPath::new("/second").unwrap();
        for p in [&first, &second] {
            create_record(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                p,
                "base",
                "Empty",
            )
            .unwrap();
        }
        let opened = AuditedTreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        for p in [&first, &second] {
            let r = opened.get_form_record(p).unwrap().unwrap();
            assert!(r.values.is_empty());
            assert_eq!(r.definition_revision, 1);
        }
        assert!(opened
            .image
            .catalogue
            .filesystem_metadata()
            .unwrap()
            .iter()
            .any(|m| m.entry.path.as_str() == "/new/deep"));
        drop(opened);
        let snapshot = identity(&storage);
        assert!(matches!(
            mutate_definition(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                DefinitionTarget::Explicit(&id, "bad alias"),
                "No",
                "",
                &fields(FormFieldKind::Text)
            ),
            Err(Error::InvalidInput(_))
        ));
        assert_eq!(snapshot, identity(&storage));
        let assigned = mutate_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            DefinitionTarget::Explicit(&id, "ignored_alias"),
            "Name",
            "Description",
            &fields(FormFieldKind::Text),
        )
        .unwrap();
        assert_eq!(assigned.revision, 2);
        assert_eq!(assigned.alias, "base");
        for (p, value) in [(&first, "first"), (&second, "second")] {
            set_field(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                p,
                "value",
                &FormValue::normal(value),
            )
            .unwrap();
        }
        // Explicit schema upgrade preserves all captures/references. Only an automatic
        // setter upgrade converts every matching captured value in the type.
        let assigned = mutate_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            DefinitionTarget::Revise(&id),
            "Secret schema",
            "Description",
            &fields(FormFieldKind::Secret),
        )
        .unwrap();
        assert_eq!(assigned.revision, 3);
        let opened = AuditedTreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        for (p, text) in [(&first, "first"), (&second, "second")] {
            let r = opened.get_form_record(p).unwrap().unwrap();
            assert_eq!(r.definition_revision, 2);
            assert_eq!(r.values[0].kind, FormFieldKind::Text);
            assert_eq!(r.values[0].value, FormValue::normal(text));
        }
        drop(opened);
        let secret = FormValue::secret(SecretString::try_from_slice(b"target only").unwrap());
        set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &first,
            "value",
            &secret,
        )
        .unwrap();
        let opened = AuditedTreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        assert_eq!(
            opened
                .resolve_form_definition(&"B".repeat(36))
                .unwrap()
                .revision,
            3
        );
        assert_eq!(
            opened.get_form_record(&first).unwrap().unwrap().values[0].value,
            secret
        );
        let neighbor = opened.get_form_record(&second).unwrap().unwrap();
        assert_eq!(neighbor.definition_revision, 2);
        assert_eq!(neighbor.values[0].value, FormValue::normal("second"));
        assert_eq!(
            opened
                .list_form_definition_revisions(&id)
                .unwrap()
                .iter()
                .map(|d| d.revision)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(opened.list_form_definitions().unwrap().len(), 1);
        drop(opened);
        let snapshot = identity(&storage);
        assert!(matches!(
            mutate_definition(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                DefinitionTarget::Revise(&id),
                "No",
                "",
                &fields(FormFieldKind::Text)
            ),
            Err(Error::InvalidOperation(_))
        ));
        assert_eq!(snapshot, identity(&storage));
        mutate_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            DefinitionTarget::Revise(&id),
            "Removed",
            "",
            &fields(FormFieldKind::Text)[1..],
        )
        .unwrap();
        assert_eq!(
            mutate_definition(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                DefinitionTarget::Define(id.as_str()),
                "Recreated",
                "",
                &fields(FormFieldKind::Text)
            )
            .unwrap()
            .revision,
            5
        );
        set_field(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &first,
            "value",
            &FormValue::normal("normal again"),
        )
        .unwrap();
        let opened = AuditedTreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        assert_eq!(
            opened.get_form_record(&first).unwrap().unwrap().values[0].value,
            FormValue::normal("normal again")
        );
        assert_eq!(opened.get_form_record(&second).unwrap().unwrap(), neighbor);
        drop(opened);
    }
}
#[test]
fn typed_form_definition_alias_import_and_refusal_controls_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let id = FormTypeId::new("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
    let other = FormTypeId::new("cccccccccccccccccccccccccccccccccccc").unwrap();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = Guarded::new(
            super::super::mutation::seed(mode, &authority, &owner),
            Vec::new(),
        );
        mutate_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            DefinitionTarget::Explicit(&id, "base"),
            "Name",
            "Description",
            &fields(FormFieldKind::Text),
        )
        .unwrap();
        assert_eq!(
            mutate_definition(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                DefinitionTarget::Define("base"),
                "Name",
                "Description",
                &fields(FormFieldKind::Text)
            )
            .unwrap()
            .revision,
            2
        );
        let mut definition = AuditedTreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap()
        .get_form_definition(&id, 2)
        .unwrap()
        .unwrap();
        let snapshot = identity(&storage);
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
        assert_eq!(snapshot, identity(&storage));
        let mut conflict = definition.clone();
        conflict.name = "Conflict".into();
        assert!(matches!(
            import_definition(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &conflict
            ),
            Err(Error::AlreadyExists(_))
        ));
        assert_eq!(snapshot, identity(&storage));
        let mut collision = definition.clone();
        collision.type_id = other.clone();
        collision.revision = 1;
        import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &collision,
        )
        .unwrap();
        let snapshot = identity(&storage);
        for reference in ["base", "missing", "bad alias"] {
            assert!(create_record(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &LockboxPath::new("/refused/record").unwrap(),
                reference,
                "No"
            )
            .is_err());
            assert_eq!(snapshot, identity(&storage));
        }
        assert!(matches!(
            mutate_definition(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                DefinitionTarget::Define("base"),
                "No",
                "",
                &fields(FormFieldKind::Text)
            ),
            Err(Error::InvalidOperation(_))
        ));
        assert_eq!(snapshot, identity(&storage));
        definition.revision = 10;
        definition.alias = "latest_alias".into();
        import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &definition,
        )
        .unwrap();
        let hex = "a".repeat(36);
        let one = mutate_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            DefinitionTarget::Define(&hex),
            "Hex",
            "",
            &fields(FormFieldKind::Text),
        )
        .unwrap();
        let two = mutate_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            DefinitionTarget::Define(&hex),
            "Hex",
            "",
            &fields(FormFieldKind::Text),
        )
        .unwrap();
        assert_ne!(one.type_id, two.type_id);
        let opened = AuditedTreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        assert_eq!(opened.resolve_form_definition("base").unwrap(), collision);
        assert_eq!(
            opened.resolve_form_definition("latest_alias").unwrap(),
            definition
        );
        assert_eq!(
            opened.resolve_form_definition(&"B".repeat(36)).unwrap(),
            definition
        );
        assert!(matches!(
            opened.resolve_form_definition(&hex),
            Err(Error::NotFound(_))
        ));
        assert!(!opened
            .image
            .catalogue
            .filesystem_metadata()
            .unwrap()
            .iter()
            .any(|m| m.entry.path.as_str() == "/refused"));
        assert_eq!(
            opened
                .list_form_definition_revisions(&id)
                .unwrap()
                .iter()
                .map(|d| d.revision)
                .collect::<Vec<_>>(),
            vec![1, 2, 10]
        );
        let latest = opened.list_form_definitions().unwrap();
        let ids = latest.iter().map(|d| &d.type_id).collect::<Vec<_>>();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
        drop(opened);
        let path = LockboxPath::new("/created/by_id").unwrap();
        create_record(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &path,
            one.type_id.as_str(),
            "Created",
        )
        .unwrap();
        let snapshot = identity(&storage);
        assert!(matches!(
            create_record(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &path,
                one.type_id.as_str(),
                "Duplicate"
            ),
            Err(Error::AlreadyExists(_))
        ));
        assert_eq!(snapshot, identity(&storage));
        for invalid in [Vec::new(), vec![fields(FormFieldKind::Text)[0].clone(); 2]] {
            assert!(mutate_definition(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                DefinitionTarget::Revise(&id),
                "No",
                "",
                &invalid
            )
            .is_err());
            assert_eq!(snapshot, identity(&storage));
        }
        definition.revision = u32::MAX;
        import_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &definition,
        )
        .unwrap();
        let snapshot = identity(&storage);
        assert!(matches!(
            mutate_definition(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                DefinitionTarget::Revise(&id),
                "No",
                "",
                &fields(FormFieldKind::Text)
            ),
            Err(Error::SecurityLimitExceeded(_))
        ));
        assert_eq!(snapshot, identity(&storage));
    }
}
#[test]
fn typed_form_definition_aggregate_normal_metadata_and_per_text_limits_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let id = FormTypeId::new("------------------------------------").unwrap();
    let size = 1048576;
    // Explicit ordinary caller metadata exceeds8MiB. Guarded staging/validation is
    // per-text; this is not a whole-process or ordinary caller/getter memory bound.
    let name = "n".repeat(size);
    let description = "d".repeat(size);
    let fields = (0..9)
        .map(|n| FormFieldDefinition {
            id: format!("field_{n}"),
            label: if n == 0 {
                "x".repeat(65535) + "🦀" + &"x".repeat(size - 65539)
            } else {
                std::iter::repeat_n((b'a' + n) as char, size).collect()
            },
            kind: FormFieldKind::Text,
            required: false,
        })
        .collect::<Vec<_>>();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = Guarded::new(
            super::super::mutation::seed(mode, &authority, &owner),
            Vec::new(),
        );
        let snapshot = identity(&storage);
        assert!(matches!(
            mutate_definition(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                DefinitionTarget::Explicit(&id, "aggregate"),
                &"n".repeat(size + 1),
                &description,
                &fields
            ),
            Err(Error::SecurityLimitExceeded(_))
        ));
        assert_eq!(snapshot, identity(&storage));
        mutate_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            DefinitionTarget::Explicit(&id, "aggregate"),
            &name,
            &description,
            &fields,
        )
        .unwrap();
        let opened = AuditedTreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let logical: usize = opened
            .image
            .catalogue
            .forms
            .texts()
            .iter()
            .map(|l| l.length)
            .sum();
        assert_eq!(logical, 11 * size);
        let definition = opened.get_form_definition(&id, 1).unwrap().unwrap();
        assert_eq!(definition.name, name);
        assert_eq!(definition.description, description);
        assert_eq!(definition.fields, fields);
        drop(opened);
        // Existing exact import retains bounded sequential equality and no-change.
        let snapshot = identity(&storage);
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
        assert_eq!(snapshot, identity(&storage));
        drop(definition);
    }
}
#[derive(Debug)]
struct SwitchImage {
    old: Guarded<StorageBackend>,
    new: Guarded<StorageBackend>,
    control_reads: std::cell::Cell<usize>,
    switch_at: Option<usize>,
    switched: std::cell::Cell<bool>,
    mutations: std::cell::Cell<usize>,
    trace: std::cell::RefCell<Vec<(u64, usize, bool)>>,
}
impl Clone for SwitchImage {
    fn clone(&self) -> Self {
        panic!("definition route cloned storage")
    }
}
impl SwitchImage {
    fn observe(&self, at: u64, len: usize) {
        if at == 0 && len == crate::file_format::publication_anchor::SLOT_LEN {
            let n = self.control_reads.get() + 1;
            self.control_reads.set(n);
            if self.switch_at == Some(n) {
                assert!(!self.switched.replace(true));
            }
        }
        self.trace.borrow_mut().push((at, len, self.switched.get()));
    }
    fn active(&self) -> &Guarded<StorageBackend> {
        if self.switched.get() {
            &self.new
        } else {
            &self.old
        }
    }
    fn mutation(&self) -> Result<()> {
        self.mutations.set(self.mutations.get() + 1);
        Err(Error::Io("unexpected write in stale-base fixture".into()))
    }
}
impl Storage for SwitchImage {
    fn len(&self) -> Result<u64> {
        self.active().len()
    }
    fn read_at(&self, at: u64, len: usize) -> Result<Vec<u8>> {
        self.observe(at, len);
        self.active().read_at(at, len)
    }
    fn read_at_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        self.observe(at, out.len());
        self.active().read_at_into(at, out)
    }
    fn append(&mut self, _: &[u8]) -> Result<u64> {
        self.mutation()?;
        unreachable!()
    }
    fn write_at(&mut self, _: u64, _: &[u8]) -> Result<()> {
        self.mutation()
    }
    fn truncate(&mut self, _: u64) -> Result<()> {
        self.mutation()
    }
    fn sync(&self) -> Result<()> {
        self.mutation()
    }
}
#[test]
fn typed_form_definition_shared_staging_refuses_concurrent_selected_change_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let id = FormTypeId::new("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut original = Guarded::new(
            super::super::mutation::seed(mode, &authority, &owner),
            Vec::new(),
        );
        mutate_definition(
            &mut original,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            DefinitionTarget::Explicit(&id, "base"),
            "Old",
            "",
            &fields(FormFieldKind::Text),
        )
        .unwrap();
        let old_bytes = original.inner.read_all().unwrap();
        let old_spans = original.spans.borrow().clone();
        let old_commit = shared::commitment(
            &AuditedTreeImage::open(
                crate::file_format::allocation_map::compaction::View(&original),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap()
            .image
            .anchor,
        )
        .unwrap();
        mutate_definition(
            &mut original,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            DefinitionTarget::Revise(&id),
            "External commit",
            "",
            &fields(FormFieldKind::Text),
        )
        .unwrap();
        let new_bytes = original.inner.read_all().unwrap();
        let new_spans = original.spans.borrow().clone();
        let expected_hash = identity(&original);
        let (new_anchor, manifest) =
            shared::open_private(&original, archive(), mode, &authority, key(mode)).unwrap();
        let new_commit = shared::commitment(&new_anchor).unwrap();
        assert_ne!(old_commit, new_commit);
        assert_eq!(&manifest[..8], b"RV4TRE01");
        let new_root = u64::from_le_bytes(manifest[8..16].try_into().unwrap());
        let new_root_len = u64::from_le_bytes(manifest[24..32].try_into().unwrap()) as usize;
        let make = |switch_at| SwitchImage {
            old: Guarded::new(StorageBackend::memory(old_bytes.clone()), old_spans.clone()),
            new: Guarded::new(StorageBackend::memory(new_bytes.clone()), new_spans.clone()),
            control_reads: std::cell::Cell::new(0),
            switch_at,
            switched: std::cell::Cell::new(false),
            mutations: std::cell::Cell::new(0),
            trace: std::cell::RefCell::new(Vec::new()),
        };
        // Calibrate the complete reader phase; preparation from borrowed normal text
        // performs no storage reads. The next slot0 selection is writer admission.
        let calibration = make(None);
        let mut opened = AuditedTreeImage::open(
            crate::file_format::allocation_map::compaction::View(&calibration),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        opened.image.verify_all().unwrap();
        assert_eq!(
            shared::commitment(&opened.image.anchor).unwrap(),
            old_commit
        );
        drop(opened);
        let preflight = calibration.control_reads.get();
        assert!(preflight > 0);
        for route in 0..3 {
            let mut storage = make(Some(preflight + 1));
            let result = match route {
                0 => {
                    let d = FormDefinition {
                        type_id: id.clone(),
                        alias: "base".into(),
                        revision: 3,
                        name: "Import".into(),
                        description: String::new(),
                        fields: fields(FormFieldKind::Text),
                    };
                    import_definition(
                        &mut storage,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        &d,
                    )
                    .map(|_| ())
                }
                1 => mutate_definition(
                    &mut storage,
                    archive(),
                    mode,
                    &authority,
                    signer,
                    key(mode),
                    DefinitionTarget::Revise(&id),
                    "Revise",
                    "",
                    &fields(FormFieldKind::Text),
                )
                .map(|_| ()),
                _ => create_record(
                    &mut storage,
                    archive(),
                    mode,
                    &authority,
                    signer,
                    key(mode),
                    &LockboxPath::new("/stale/record").unwrap(),
                    "base",
                    "Create",
                ),
            };
            assert!(
                matches!(result, Err(Error::CorruptRecord)),
                "mode={bits} route={route}: {result:?}"
            );
            assert!(storage.switched.get());
            assert_eq!(storage.control_reads.get(), preflight + 1);
            assert_eq!(storage.mutations.get(), 0);
            assert!(
                storage
                    .trace
                    .borrow()
                    .contains(&(new_root, new_root_len, true)),
                "new authenticated index not traversed"
            );
            assert_eq!(identity(&storage), expected_hash);
            let opened = AuditedTreeImage::open(
                crate::file_format::allocation_map::compaction::View(&storage),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap();
            assert_eq!(
                shared::commitment(&opened.image.anchor).unwrap(),
                new_commit
            );
            assert_eq!(
                opened.resolve_form_definition("base").unwrap().name,
                "External commit"
            );
            assert!(opened
                .get_form_record(&LockboxPath::new("/stale/record").unwrap())
                .unwrap()
                .is_none());
        }
        println!("DEFINITION_STALE_ADMISSION mode={bits} reader_control_selections={preflight} writer_switch_selection={}",preflight+1);
    }
}
#[test]
fn typed_form_empty_creation_preserves_file_namespace_and_name_limits_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let id = FormTypeId::new("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut storage = Guarded::new(
            super::super::mutation::seed(mode, &authority, &owner),
            Vec::new(),
        );
        mutate_definition(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            DefinitionTarget::Explicit(&id, "create"),
            "Create",
            "",
            &fields(FormFieldKind::Text),
        )
        .unwrap();
        let same = LockboxPath::new("/docs/neighbor").unwrap();
        let child = LockboxPath::new("/docs/neighbor/child").unwrap();
        let before_empty = identity(&storage);
        assert!(matches!(
            create_record(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &LockboxPath::new("/refused/parents/record").unwrap(),
                "create",
                ""
            ),
            Err(Error::InvalidInput(_))
        ));
        assert_eq!(identity(&storage), before_empty);
        assert!(!AuditedTreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode)
        )
        .unwrap()
        .image
        .filesystem_metadata()
        .unwrap()
        .iter()
        .any(|m| m.entry.path.as_str().starts_with("/refused")));
        for path in [&same, &child] {
            create_record(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                path,
                "create",
                "Named record",
            )
            .unwrap();
        }
        let maximum = LockboxPath::new("/large/name").unwrap();
        let name = "x".repeat(65535) + "🦀" + &"n".repeat(1048576 - 65539);
        let snapshot = identity(&storage);
        assert!(matches!(
            create_record(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                &maximum,
                "create",
                &"n".repeat(1048577)
            ),
            Err(Error::SecurityLimitExceeded(_))
        ));
        assert_eq!(identity(&storage), snapshot);
        create_record(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &maximum,
            "create",
            &name,
        )
        .unwrap();
        let mut opened = AuditedTreeImage::open(
            crate::file_format::allocation_map::compaction::View(&storage),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        for path in [&same, &child] {
            let r = opened.get_form_record(path).unwrap().unwrap();
            assert_eq!(r.name, "Named record");
            assert!(r.values.is_empty());
        }
        let r = opened.get_form_record(&maximum).unwrap().unwrap();
        assert_eq!(r.name, name);
        assert!(r.values.is_empty());
        let mut bytes = Vec::new();
        opened
            .image
            .read_range(b"/docs/neighbor", 0, 8, |b| {
                bytes.extend_from_slice(b);
                Ok(())
            })
            .unwrap();
        assert_eq!(bytes, b"neighbor");
        assert!(opened
            .image
            .filesystem_metadata()
            .unwrap()
            .iter()
            .any(|m| m.entry.path == same && m.entry.kind == crate::LockboxEntryKind::File));
    }
}
#[test]
fn typed_form_definition_and_empty_record_atomic_recovery_faults() {
    use super::lifecycle::{collect, fixture};
    use crate::file_format::preparation_journal::tests::CrashStore;
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    let mut recoveries = 0;
    for bits in [0, 1, 2, 3, 12, 13, 14, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let (original, old_defs, old_records) = fixture(mode, &authority, &owner);
        let image = original.inner.read_all().unwrap();
        let spans = original.spans.borrow().clone();
        let type_id = old_defs[0].type_id.clone();
        let opened = AuditedTreeImage::open(
            crate::file_format::allocation_map::compaction::View(&original),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        let payloads = opened.tree.graph.payloads();
        let old_ids = opened
            .image
            .catalogue
            .forms
            .records
            .iter()
            .map(|r| (r.path.clone(), r.id))
            .collect::<Vec<_>>();
        let old_dirs = opened
            .image
            .filesystem_metadata()
            .unwrap()
            .into_iter()
            .filter(|m| m.entry.kind == crate::LockboxEntryKind::Directory)
            .map(|m| m.entry.path)
            .collect::<Vec<_>>();
        drop(opened);
        for operation in 0..2 {
            let path = LockboxPath::new("/atomic/nested/record").unwrap();
            let new_fields = vec![FormFieldDefinition {
                id: "password".into(),
                label: "Revised label".into(),
                kind: FormFieldKind::Secret,
                required: true,
            }];
            let mut new_defs = old_defs.clone();
            let mut new_records = old_records.clone();
            let mut new_dirs = old_dirs.clone();
            if operation == 0 {
                let mut d = old_defs[0].clone();
                d.revision = 2;
                d.name = "Revised definition".into();
                d.description = "Revised description".into();
                d.fields = new_fields.clone();
                new_defs.push(d);
                new_defs.sort_by(|a, b| (&a.type_id, a.revision).cmp(&(&b.type_id, b.revision)));
            } else {
                new_records.push(FormRecord {
                    path: path.clone(),
                    name: "Empty record".into(),
                    type_id: type_id.clone(),
                    definition_alias: old_defs[0].alias.clone(),
                    definition_revision: 1,
                    values: Vec::new(),
                });
                new_records.sort_by(|a, b| a.path.cmp(&b.path));
                new_dirs.extend([
                    LockboxPath::new("/atomic").unwrap(),
                    LockboxPath::new("/atomic/nested").unwrap(),
                ]);
                new_dirs.sort();
            }
            let run = |s: &mut Guarded<CrashStore>| {
                if operation == 0 {
                    mutate_definition(
                        s,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        DefinitionTarget::Revise(&type_id),
                        "Revised definition",
                        "Revised description",
                        &new_fields,
                    )
                    .map(|_| ())
                } else {
                    create_record(
                        s,
                        archive(),
                        mode,
                        &authority,
                        signer,
                        key(mode),
                        &path,
                        type_id.as_str(),
                        "Empty record",
                    )
                }
            };
            let verify = |s: &Guarded<StorageBackend>| {
                let (report, defs, records) = collect(s, mode, &authority).unwrap();
                assert_eq!(report, tree_image::forms::SalvageReport::default());
                let committed = if operation == 0 {
                    defs.len() == new_defs.len()
                } else {
                    records.len() == new_records.len()
                };
                assert_eq!(
                    defs,
                    if committed {
                        new_defs.clone()
                    } else {
                        old_defs.clone()
                    }
                );
                assert_eq!(
                    records,
                    if committed {
                        new_records.clone()
                    } else {
                        old_records.clone()
                    }
                );
                let mut opened = AuditedTreeImage::open(
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
                let dirs = opened
                    .image
                    .filesystem_metadata()
                    .unwrap()
                    .into_iter()
                    .filter(|m| m.entry.kind == crate::LockboxEntryKind::Directory)
                    .map(|m| m.entry.path)
                    .collect::<Vec<_>>();
                assert_eq!(
                    dirs,
                    if committed {
                        new_dirs.clone()
                    } else {
                        old_dirs.clone()
                    }
                );
                let bytes = s.inner.read_all().unwrap();
                for e in &payloads {
                    let a = e.start as usize;
                    let b = (e.start + e.len) as usize;
                    assert_eq!(&bytes[a..b], &image[a..b]);
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
                let mut neighbor = Vec::new();
                opened
                    .image
                    .read_range(b"/docs/neighbor", 0, 8, |b| {
                        neighbor.extend_from_slice(b);
                        Ok(())
                    })
                    .unwrap();
                assert_eq!(neighbor, b"neighbor");
                committed
            };
            let mut observed = Guarded::new(
                CrashStore::new(image.clone(), None, 0, false),
                spans.clone(),
            );
            run(&mut observed).unwrap();
            let count = observed.inner.operations();
            let successful = Guarded::new(
                StorageBackend::memory(observed.inner.durable()),
                observed.spans.borrow().clone(),
            );
            assert!(verify(&successful));
            for at in 0..count {
                for prefix in [0, 97, usize::MAX] {
                    for persist in [false, true] {
                        let mut failed = Guarded::new(
                            CrashStore::new(image.clone(), Some(at), prefix, persist),
                            spans.clone(),
                        );
                        let _ = run(&mut failed);
                        let damaged = failed.inner.durable();
                        let tracked = failed.spans.borrow().clone();
                        let mut recovered =
                            Guarded::new(StorageBackend::memory(damaged.clone()), tracked.clone());
                        tree_image::recover(&mut recovered,archive(),mode,&authority,key(mode)).unwrap_or_else(|e|panic!("mode={bits} operation={operation} at={at} prefix={prefix} persist={persist}: {e}"));
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
                            tree_image::recover(
                                &mut observed,
                                archive(),
                                mode,
                                &authority,
                                key(mode),
                            )
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
                                    recoveries += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("FORM_DEFINITION_CREATE_FAULTS cases={cases} interrupted_recovery={recoveries}");
}
