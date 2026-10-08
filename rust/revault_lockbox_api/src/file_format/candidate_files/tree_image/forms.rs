//! Test-only snapshot import/get. Historical capture admission differs from the
//! released migration import's stricter current-field policy.
use super::super::dense_catalogue::forms::{self, Capture, Definition, Field, Record};
use super::*;
use crate::file_format::{
    form_segments::{self, Layout, Source},
    page::secure_storage::PreparedSecurePages,
};
use crate::secret_vec::SecureVec;
use crate::{
    FormDefinition, FormFieldDefinition, FormRecord, FormTypeId, FormValue, LockboxPath,
    VariableSensitivity,
};
use std::collections::BTreeMap;
impl<S: Storage> TreeImage<S> {
    /// Read one selected captured field without assembling every value in the form.
    pub fn with_form_field_value<R>(
        &self,
        path: &LockboxPath,
        id: &str,
        f: impl FnOnce(&FormValue) -> R,
    ) -> Result<Option<R>> {
        let Some(record) = self
            .image
            .catalogue
            .forms
            .records
            .iter()
            .find(|r| &r.path == path)
        else {
            return Ok(None);
        };
        let Some(field) = record.fields.iter().find(|field| field.id == id) else {
            return Ok(None);
        };
        let value = field.read_value(
            &self.image.storage,
            self.image.anchor.archive,
            self.image.value_key.as_ref().ok_or(Error::CorruptRecord)?,
            self.image.anchor.sealed_len,
        )?;
        Ok(Some(f(&value)))
    }

    pub fn get_form_definition(
        &self,
        id: &FormTypeId,
        revision: u32,
    ) -> Result<Option<FormDefinition>> {
        self.image
            .catalogue
            .forms
            .definitions
            .iter()
            .find(|d| &d.type_id == id && d.revision == revision)
            .map(|d| {
                d.read(
                    &self.image.storage,
                    self.image.anchor.archive,
                    self.image.value_key.as_ref().ok_or(Error::CorruptRecord)?,
                    self.image.anchor.sealed_len,
                )
            })
            .transpose()
    }
    pub fn get_form_record(&self, path: &LockboxPath) -> Result<Option<FormRecord>> {
        self.image
            .catalogue
            .forms
            .records
            .iter()
            .find(|r| &r.path == path)
            .map(|r| {
                r.read(
                    &self.image.storage,
                    self.image.anchor.archive,
                    self.image.value_key.as_ref().ok_or(Error::CorruptRecord)?,
                    self.image.anchor.sealed_len,
                )
            })
            .transpose()
    }
}
type PayloadCallback<'a> = Box<dyn FnMut(usize) -> Result<SecureVec> + 'a>;
struct Batch<'a> {
    pages: PreparedSecurePages,
    callbacks: Vec<(usize, PayloadCallback<'a>)>,
    ids: Vec<([u8; 16], usize)>,
}
impl<'a> Batch<'a> {
    fn new() -> Result<Self> {
        Ok(Self {
            pages: PreparedSecurePages::new(4096)?,
            callbacks: Vec::new(),
            ids: Vec::new(),
        })
    }
    #[allow(clippy::too_many_arguments)]
    fn add(
        &mut self,
        source: Source<'a>,
        archive: LockboxId,
        mode: FormatMode,
        key: &[u8; 32],
        context: [u8; 32],
        sensitivity: VariableSensitivity,
    ) -> Result<Layout> {
        let mut id = [0; 16];
        getrandom::fill(&mut id).map_err(|e| Error::Io(e.to_string()))?;
        let prepared =
            form_segments::prepare_source(source, archive, mode, key, id, context, 1, sensitivity)?;
        let count = prepared.pages.len();
        self.pages.append(prepared.pages)?;
        self.callbacks.push((count, prepared.payload));
        self.ids.push((id, count));
        Ok(prepared.layout)
    }
}
#[allow(clippy::too_many_arguments)]
pub(in crate::file_format::candidate_files) fn import_definition(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    definition: &FormDefinition,
) -> Result<bool> {
    validate_definition(definition)?;
    let mut opened = TreeImage::open(
        allocation::compaction::View(&*storage),
        archive,
        mode,
        authority,
        key,
    )?;
    shared::prepare(&opened.image.anchor, authority, signer)?;
    opened.image.verify_all()?;
    if let Some(prior) = opened
        .image
        .catalogue
        .forms
        .definitions
        .iter()
        .find(|d| d.type_id == definition.type_id && d.revision == definition.revision)
    {
        if prior.matches(
            definition,
            &opened.image.storage,
            archive,
            opened
                .image
                .value_key
                .as_ref()
                .ok_or(Error::CorruptRecord)?,
            opened.image.anchor.sealed_len,
        )? {
            return Ok(false);
        }
        return Err(Error::AlreadyExists(
            "conflicting form definition revision".into(),
        ));
    }
    opened.image.catalogue.forms.admit(
        definition
            .fields
            .len()
            .checked_add(1)
            .ok_or(Error::CorruptRecord)?,
    )?;
    let anchor = opened.image.anchor.clone();
    let mut catalogue = opened.image.catalogue;
    let value_key = super::super::dense_image::value_key(mode, key)?;
    let mut batch = Batch::new()?;
    let parent = forms::definition_key(&definition.type_id, definition.revision);
    let normal = VariableSensitivity::Normal;
    let name = batch.add(
        Source::Text(&definition.name),
        archive,
        mode,
        &value_key,
        form_segments::context(&parent, 1, None),
        normal,
    )?;
    let description = batch.add(
        Source::Text(&definition.description),
        archive,
        mode,
        &value_key,
        form_segments::context(&parent, 2, None),
        normal,
    )?;
    let mut fields = Vec::new();
    for f in &definition.fields {
        let label = batch.add(
            Source::Text(&f.label),
            archive,
            mode,
            &value_key,
            form_segments::context(&parent, 3, Some((&f.id, f.kind))),
            normal,
        )?;
        fields.push(Field {
            id: f.id.clone(),
            kind: f.kind,
            required: f.required,
            label,
        });
    }
    catalogue.forms.definitions.push(Definition {
        type_id: definition.type_id.clone(),
        revision: definition.revision,
        alias: definition.alias.clone(),
        name,
        description,
        fields,
    });
    install(
        storage, archive, mode, authority, signer, key, anchor, catalogue, batch,
    )
}
#[allow(clippy::too_many_arguments)]
pub(in crate::file_format::candidate_files) fn import_captured_record(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    record: &FormRecord,
) -> Result<bool> {
    forms::Forms::default().admit(
        record
            .values
            .len()
            .checked_add(1)
            .ok_or(Error::CorruptRecord)?,
    )?;
    if record.path.file_path()?.as_str() != record.path.as_str() {
        return Err(Error::CorruptRecord);
    }
    FormRecord::validated_name(&record.name)?;
    let mut seen = std::collections::BTreeSet::new();
    for f in &record.values {
        FormFieldDefinition::validated_id(&f.field_id)?;
        FormFieldDefinition::validated_label(&f.captured_label)?;
        f.kind.validate_value(&f.value)?;
        if !seen.insert(&f.field_id) {
            return Err(Error::CorruptRecord);
        }
    }
    let mut opened = TreeImage::open(
        allocation::compaction::View(&*storage),
        archive,
        mode,
        authority,
        key,
    )?;
    shared::prepare(&opened.image.anchor, authority, signer)?;
    opened.image.verify_all()?;
    if opened
        .image
        .catalogue
        .forms
        .records
        .iter()
        .any(|r| r.path == record.path)
    {
        return Err(Error::AlreadyExists(record.path.to_string()));
    }
    let definition = opened
        .image
        .catalogue
        .forms
        .definitions
        .iter()
        .find(|d| d.type_id == record.type_id && d.revision == record.definition_revision)
        .ok_or(Error::CorruptRecord)?;
    if definition.alias != record.definition_alias {
        return Err(Error::CorruptRecord);
    }
    opened.image.catalogue.forms.admit(
        record
            .values
            .len()
            .checked_add(1)
            .ok_or(Error::CorruptRecord)?,
    )?;
    let anchor = opened.image.anchor.clone();
    let mut catalogue = opened.image.catalogue;
    create_parents(&mut catalogue, &record.path)?;
    let value_key = super::super::dense_image::value_key(mode, key)?;
    let mut batch = Batch::new()?;
    let mut id = [0; 16];
    getrandom::fill(&mut id).map_err(|e| Error::Io(e.to_string()))?;
    let normal = VariableSensitivity::Normal;
    let name = batch.add(
        Source::Text(&record.name),
        archive,
        mode,
        &value_key,
        form_segments::context(&id, 4, None),
        normal,
    )?;
    let mut fields = Vec::new();
    for f in &record.values {
        let label = batch.add(
            Source::Text(&f.captured_label),
            archive,
            mode,
            &value_key,
            form_segments::context(&id, 5, Some((&f.field_id, f.kind))),
            normal,
        )?;
        let (source, sensitivity) = match &f.value {
            FormValue::Normal(value) => (Source::Text(value), normal),
            FormValue::Secret(value) => (Source::Secret(value), VariableSensitivity::Secret),
        };
        let value = batch.add(
            source,
            archive,
            mode,
            &value_key,
            form_segments::context(&id, 6, Some((&f.field_id, f.kind))),
            sensitivity,
        )?;
        fields.push(Capture {
            id: f.field_id.clone(),
            kind: f.kind,
            label,
            value,
        });
    }
    catalogue.forms.records.push(Record {
        path: record.path.clone(),
        id,
        type_id: record.type_id.clone(),
        revision: record.definition_revision,
        alias: record.definition_alias.clone(),
        name,
        fields,
    });
    install(
        storage, archive, mode, authority, signer, key, anchor, catalogue, batch,
    )
}
#[allow(clippy::too_many_arguments)]
fn install(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    anchor: Anchor,
    mut catalogue: Catalogue,
    mut batch: Batch<'_>,
) -> Result<bool> {
    let records = catalogue.tree_records()?;
    let plan = tree::PreparedSecurePayloadPlan {
        base: shared::commitment(&anchor)?,
        retired: Vec::new(),
        pages: batch.pages,
        payload: Box::new(move |mut n| {
            for (count, callback) in &mut batch.callbacks {
                if n < *count {
                    return callback(n);
                }
                n -= *count;
            }
            Err(Error::CorruptRecord)
        }),
        rebind: Box::new(move |extents| {
            let mut offset = 0usize;
            let mut places = BTreeMap::new();
            for (id, count) in &batch.ids {
                let end = offset.checked_add(*count).ok_or(Error::CorruptRecord)?;
                places.insert(*id, extents.get(offset..end).ok_or(Error::CorruptRecord)?);
                offset = end;
            }
            if offset != extents.len() {
                return Err(Error::CorruptRecord);
            }
            for text in catalogue.forms.texts_mut() {
                if let Some(extents) = places.remove(&text.id) {
                    text.rebind(extents)?;
                }
            }
            if !places.is_empty() {
                return Err(Error::CorruptRecord);
            }
            catalogue.tree_records()
        }),
    };
    tree::rewrite_prepared_secure_payload_records(
        storage, archive, mode, authority, signer, key, records, plan,
    )
}
fn validate_definition(definition: &FormDefinition) -> Result<()> {
    if definition.revision == 0 {
        return Err(Error::CorruptRecord);
    }
    forms::Forms::default().admit(
        definition
            .fields
            .len()
            .checked_add(1)
            .ok_or(Error::CorruptRecord)?,
    )?;
    if definition.fields.is_empty() {
        return Err(Error::InvalidInput(
            "form definition requires at least one field".into(),
        ));
    }
    if FormDefinition::validated_alias(&definition.alias)? != definition.alias {
        return Err(Error::CorruptRecord);
    }
    if FormDefinition::validated_name(&definition.name)? != definition.name {
        return Err(Error::CorruptRecord);
    }
    if FormDefinition::validated_description(&definition.description)? != definition.description {
        return Err(Error::CorruptRecord);
    }
    let mut seen = std::collections::BTreeSet::new();
    for field in &definition.fields {
        if FormFieldDefinition::validated_id(&field.id)? != field.id
            || FormFieldDefinition::validated_label(&field.label)? != field.label
        {
            return Err(Error::CorruptRecord);
        }
        if !seen.insert(&field.id) {
            return Err(Error::InvalidInput(format!(
                "duplicate form field id: {}",
                field.id
            )));
        }
    }
    Ok(())
}
fn create_parents(catalogue: &mut Catalogue, path: &LockboxPath) -> Result<()> {
    let mut metadata = catalogue.filesystem_metadata()?;
    let Some(parent) = path.parent()? else {
        return Ok(());
    };
    // Public create_parent_dirs_for accepts any already-existing direct parent,
    // including a file. Preserve that observed form namespace state.
    if parent.as_str() == "/" || metadata.iter().any(|m| m.entry.path == parent) {
        return Ok(());
    }
    let mut chain = Vec::new();
    let mut next = Some(parent);
    while let Some(path) = next {
        if path.as_str() == "/" {
            break;
        }
        next = path.parent()?;
        chain.push(path);
    }
    chain.reverse();
    for path in chain {
        if let Some(existing) = metadata.iter().find(|m| m.entry.path == path) {
            if existing.entry.kind != crate::LockboxEntryKind::Directory {
                return Err(Error::InvalidOperation(
                    "form ancestor is not a directory".into(),
                ));
            }
        } else {
            metadata.push(Metadata {
                entry: crate::LockboxEntry {
                    path,
                    kind: crate::LockboxEntryKind::Directory,
                    len: 0,
                    permissions: crate::constants::DEFAULT_DIRECTORY_PERMISSIONS,
                },
                target: None,
            });
        }
    }
    catalogue.set_tree_metadata(&metadata)
}

mod lifecycle;
pub(in crate::file_format::candidate_files) use lifecycle::{
    delete_record, salvage_forms, SalvageEvent, SalvageReport,
};
