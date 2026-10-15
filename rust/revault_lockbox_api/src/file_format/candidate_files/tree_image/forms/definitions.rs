//! Definition routing preserves the established public type-ID/alias precedence.
use super::*;
use crate::FormFieldKind;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::file_format::candidate_files) struct AssignedDefinition {
    pub type_id: FormTypeId,
    pub revision: u32,
    pub alias: String,
}
#[derive(Clone, Copy)]
pub(in crate::file_format::candidate_files) enum DefinitionTarget<'a> {
    Define(&'a str),
    Explicit(&'a FormTypeId, &'a str),
    Revise(&'a FormTypeId),
}
fn latest(forms: &forms::Forms, id: &FormTypeId) -> Option<usize> {
    forms
        .definitions
        .iter()
        .enumerate()
        .filter(|(_, d)| &d.type_id == id)
        .max_by_key(|(_, d)| d.revision)
        .map(|(i, _)| i)
}
fn latest_indices(forms: &forms::Forms) -> Vec<usize> {
    let mut selected = BTreeMap::new();
    for (i, d) in forms.definitions.iter().enumerate() {
        let replace = selected
            .get(&d.type_id)
            .is_none_or(|old: &usize| forms.definitions[*old].revision < d.revision);
        if replace {
            selected.insert(&d.type_id, i);
        }
    }
    selected.into_values().collect()
}
fn resolve(forms: &forms::Forms, reference: &str) -> Result<usize> {
    if let Ok(id) = FormTypeId::new(reference) {
        return latest(forms, &id).ok_or_else(|| Error::NotFound(format!("form type {id}")));
    }
    let alias = FormDefinition::validated_alias(reference)?;
    let matching = latest_indices(forms)
        .into_iter()
        .filter(|i| forms.definitions[*i].alias == alias)
        .collect::<Vec<_>>();
    match matching.as_slice() {
        [i] => Ok(*i),
        [] => Err(Error::NotFound(format!("form alias {alias}"))),
        _ => Err(Error::InvalidOperation(format!(
            "form alias {alias} is ambiguous; use a form type id"
        ))),
    }
}
impl<S: Storage> AuditedTreeImage<S> {
    pub fn resolve_form_definition(&self, reference: &str) -> Result<FormDefinition> {
        let index = resolve(&self.image.catalogue.forms, reference)?;
        self.image.catalogue.forms.definitions[index].read(
            &self.image.storage,
            self.image.anchor.archive,
            self.image.value_key.as_ref().ok_or(Error::CorruptRecord)?,
            self.image.anchor.sealed_len,
        )
    }
    pub fn list_form_definitions(&self) -> Result<Vec<FormDefinition>> {
        latest_indices(&self.image.catalogue.forms)
            .into_iter()
            .map(|i| {
                self.image.catalogue.forms.definitions[i].read(
                    &self.image.storage,
                    self.image.anchor.archive,
                    self.image.value_key.as_ref().ok_or(Error::CorruptRecord)?,
                    self.image.anchor.sealed_len,
                )
            })
            .collect()
    }
    pub fn list_form_definition_revisions(&self, id: &FormTypeId) -> Result<Vec<FormDefinition>> {
        let mut selected = self
            .image
            .catalogue
            .forms
            .definitions
            .iter()
            .filter(|d| &d.type_id == id)
            .collect::<Vec<_>>();
        selected.sort_by_key(|d| d.revision);
        selected
            .into_iter()
            .map(|d| {
                d.read(
                    &self.image.storage,
                    self.image.anchor.archive,
                    self.image.value_key.as_ref().ok_or(Error::CorruptRecord)?,
                    self.image.anchor.sealed_len,
                )
            })
            .collect()
    }
}
#[allow(clippy::too_many_arguments)]
pub(in crate::file_format::candidate_files) fn mutate_definition(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    target: DefinitionTarget<'_>,
    name: &str,
    description: &str,
    fields: &[FormFieldDefinition],
) -> Result<AssignedDefinition> {
    let supplied_alias = match target {
        DefinitionTarget::Define(alias) | DefinitionTarget::Explicit(_, alias) => {
            Some(FormDefinition::validated_alias(alias)?)
        }
        DefinitionTarget::Revise(_) => None,
    };
    let mut opened = AuditedTreeImage::open(
        allocation::compaction::View(&*storage),
        archive,
        mode,
        authority,
        key,
    )?;
    shared::prepare(&opened.image.anchor, authority, signer)?;
    opened.image.verify_all()?;
    let forms = &opened.image.catalogue.forms;
    let index = match target {
        DefinitionTarget::Define(reference) => match resolve(forms, reference) {
            Ok(i) => Some(i),
            Err(Error::NotFound(_)) => None,
            Err(e) => return Err(e),
        },
        DefinitionTarget::Explicit(id, _) => latest(forms, id),
        DefinitionTarget::Revise(id) => {
            Some(latest(forms, id).ok_or_else(|| Error::NotFound(format!("form type {id}")))?)
        }
    };
    let assigned = if let Some(i) = index {
        let previous = &forms.definitions[i];
        for old in &previous.fields {
            if old.kind == FormFieldKind::Secret
                && fields
                    .iter()
                    .any(|new| new.id == old.id && !new.kind.is_secret())
            {
                return Err(Error::InvalidOperation(format!("form field {} is secret; remove it in one definition revision before recreating it as non-secret",old.id)));
            }
        }
        AssignedDefinition {
            type_id: previous.type_id.clone(),
            revision: previous.revision.checked_add(1).ok_or_else(|| {
                Error::SecurityLimitExceeded("form definition revision exhausted".into())
            })?,
            alias: previous.alias.clone(),
        }
    } else {
        let type_id = match target {
            DefinitionTarget::Explicit(id, _) => id.clone(),
            DefinitionTarget::Define(_) => FormTypeId::new_random()?,
            DefinitionTarget::Revise(_) => return Err(Error::CorruptRecord),
        };
        AssignedDefinition {
            type_id,
            revision: 1,
            alias: supplied_alias.ok_or(Error::CorruptRecord)?,
        }
    };
    let parts = DefinitionParts {
        type_id: &assigned.type_id,
        revision: assigned.revision,
        alias: &assigned.alias,
        name,
        description,
        fields,
    };
    validate_definition_parts(parts)?;
    stage_definition(
        storage,
        archive,
        mode,
        authority,
        signer,
        key,
        opened.image.anchor.clone(),
        opened.image.catalogue,
        parts,
    )?;
    Ok(assigned)
}
#[allow(clippy::too_many_arguments)]
pub(in crate::file_format::candidate_files) fn create_record(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    path: &LockboxPath,
    reference: &str,
    name: &str,
) -> Result<()> {
    let path = path.file_path()?;
    FormRecord::validated_name(name)?;
    let mut opened = AuditedTreeImage::open(
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
        .any(|r| r.path == path)
    {
        return Err(Error::AlreadyExists(path.to_string()));
    }
    let index = resolve(&opened.image.catalogue.forms, reference)?;
    let selected = &opened.image.catalogue.forms.definitions[index];
    let assigned = AssignedDefinition {
        type_id: selected.type_id.clone(),
        revision: selected.revision,
        alias: selected.alias.clone(),
    };
    opened.image.catalogue.forms.admit(1)?;
    let anchor = opened.image.anchor.clone();
    let mut catalogue = opened.image.catalogue;
    create_parents(&mut catalogue, &path)?;
    let content_key = super::super::super::dense_image::value_key(mode, key)?;
    let mut id = [0; 16];
    getrandom::fill(&mut id).map_err(|e| Error::Io(e.to_string()))?;
    let mut batch = Batch::new()?;
    let name = batch.add(
        Source::Text(name),
        archive,
        mode,
        &content_key,
        form_segments::context(&id, 4, None),
        VariableSensitivity::Normal,
    )?;
    catalogue.forms.records.push(Record {
        path,
        id,
        type_id: assigned.type_id,
        revision: assigned.revision,
        alias: assigned.alias,
        name,
        fields: Vec::new(),
    });
    install(
        storage, archive, mode, authority, signer, key, anchor, catalogue, batch,
    )?;
    Ok(())
}
