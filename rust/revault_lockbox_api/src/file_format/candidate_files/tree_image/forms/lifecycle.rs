//! Selected record deletion and bounded streamed salvage. Callers stage events
//! until object End and whole-call success and keep storage stable throughout.
use super::*;
use crate::file_format::candidate_files::dense_catalogue::forms::read_normal;
use crate::{FormFieldValue, LockboxPath};
use std::collections::BTreeSet;

#[allow(clippy::too_many_arguments)]
pub(in crate::file_format::candidate_files) fn delete_record(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    path: &LockboxPath,
) -> Result<bool> {
    let path = path.file_path()?;
    let mut opened = AuditedTreeImage::open(
        allocation::compaction::View(&*storage),
        archive,
        mode,
        authority,
        key,
    )?;
    shared::prepare(&opened.image.anchor, authority, signer)?;
    opened.image.verify_all()?;
    let index = opened
        .image
        .catalogue
        .forms
        .records
        .iter()
        .position(|r| r.path == path)
        .ok_or_else(|| Error::NotFound(format!("form record {path}")))?;
    let base = shared::commitment(&opened.image.anchor)?;
    let mut catalogue = opened.image.catalogue;
    let record = catalogue.forms.records.remove(index);
    let mut retired = record.name.extents;
    for field in record.fields {
        retired.extend(field.label.extents);
        retired.extend(field.value.extents);
    }
    let records = catalogue.tree_records()?;
    let plan = tree::PreparedSecurePayloadPlan {
        base,
        retired,
        pages: PreparedSecurePages::new(0)?,
        payload: Box::new(|_| Err(Error::CorruptRecord)),
        rebind: Box::new(move |extents| {
            if !extents.is_empty() {
                return Err(Error::CorruptRecord);
            }
            catalogue.tree_records()
        }),
    };
    tree::rewrite_prepared_secure_payload_records(
        storage, archive, mode, authority, signer, key, records, plan,
    )
}

pub(in crate::file_format::candidate_files) enum SalvageEvent {
    DefinitionStart {
        type_id: FormTypeId,
        revision: u32,
        alias: String,
        name: String,
        description: String,
        fields: usize,
    },
    DefinitionField(FormFieldDefinition),
    DefinitionEnd {
        type_id: FormTypeId,
        revision: u32,
    },
    RecordStart {
        path: LockboxPath,
        id: [u8; 16],
        name: String,
        type_id: FormTypeId,
        definition_revision: u32,
        definition_alias: String,
        fields: usize,
    },
    CapturedField(FormFieldValue),
    RecordEnd {
        path: LockboxPath,
        id: [u8; 16],
    },
}
#[derive(Default, Debug, PartialEq, Eq)]
pub(in crate::file_format::candidate_files) struct SalvageReport {
    pub unavailable_definitions: Vec<(FormTypeId, u32)>,
    pub unavailable_records: Vec<LockboxPath>,
}
fn unavailable(error: Error) -> Result<()> {
    match error {
        Error::CorruptRecord | Error::CorruptHeader | Error::Truncated => Ok(()),
        other => Err(other),
    }
}
/// Validate complete selected membership before events, then validate each whole
/// object before its Start. Delivery re-reads guarded pages. Any delivery/sink
/// failure is fatal and may follow partial events: callers must discard staging.
/// Commitment checks do not detect arbitrary in-place mutation after a last read;
/// stable storage is a caller precondition, not a lock implemented by this API.
#[allow(clippy::too_many_arguments)]
pub(in crate::file_format::candidate_files) fn salvage_forms(
    storage: &impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
    mut sink: impl FnMut(SalvageEvent) -> Result<()>,
) -> Result<SalvageReport> {
    let codec = Codec::shared_packed(archive, mode, key)?;
    let (catalogue, tree) = Catalogue::with_tree(&codec, |visitor| {
        Tree::salvage_visit(storage, archive, mode, authority, key, visitor)
    })?;
    let base = shared::commitment(&tree.anchor)?;
    let stable = || -> Result<()> {
        if shared::commitment(&shared::open_private(storage, archive, mode, authority, key)?.0)?
            != base
        {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    };
    stable()?;
    let mut report = SalvageReport::default();
    if catalogue.forms.is_empty() {
        return Ok(report);
    }
    let content_key = super::super::super::dense_image::value_key(mode, key)?;
    let sealed = tree.anchor.sealed_len;
    let mut available = BTreeSet::new();
    for definition in &catalogue.forms.definitions {
        if let Err(error) = definition.verify(storage, archive, &content_key, sealed) {
            unavailable(error)?;
            report
                .unavailable_definitions
                .push((definition.type_id.clone(), definition.revision));
            continue;
        }
        stable()?;
        sink(SalvageEvent::DefinitionStart {
            type_id: definition.type_id.clone(),
            revision: definition.revision,
            alias: definition.alias.clone(),
            name: read_normal(&definition.name, storage, archive, &content_key, sealed)?,
            description: read_normal(
                &definition.description,
                storage,
                archive,
                &content_key,
                sealed,
            )?,
            fields: definition.fields.len(),
        })?;
        for field in &definition.fields {
            sink(SalvageEvent::DefinitionField(FormFieldDefinition {
                id: field.id.clone(),
                label: read_normal(&field.label, storage, archive, &content_key, sealed)?,
                kind: field.kind,
                required: field.required,
            }))?;
        }
        stable()?;
        sink(SalvageEvent::DefinitionEnd {
            type_id: definition.type_id.clone(),
            revision: definition.revision,
        })?;
        available.insert((definition.type_id.clone(), definition.revision));
    }
    for record in &catalogue.forms.records {
        if !available.contains(&(record.type_id.clone(), record.revision)) {
            report.unavailable_records.push(record.path.clone());
            continue;
        }
        if let Err(error) = record.verify(storage, archive, &content_key, sealed) {
            unavailable(error)?;
            report.unavailable_records.push(record.path.clone());
            continue;
        }
        stable()?;
        sink(SalvageEvent::RecordStart {
            path: record.path.clone(),
            id: record.id,
            name: read_normal(&record.name, storage, archive, &content_key, sealed)?,
            type_id: record.type_id.clone(),
            definition_revision: record.revision,
            definition_alias: record.alias.clone(),
            fields: record.fields.len(),
        })?;
        for field in &record.fields {
            sink(SalvageEvent::CapturedField(FormFieldValue {
                field_id: field.id.clone(),
                captured_label: read_normal(&field.label, storage, archive, &content_key, sealed)?,
                kind: field.kind,
                value: field.read_value(storage, archive, &content_key, sealed)?,
            }))?;
        }
        stable()?;
        sink(SalvageEvent::RecordEnd {
            path: record.path.clone(),
            id: record.id,
        })?;
    }
    stable()?;
    Ok(report)
}

#[test]
fn form_salvage_only_marks_stored_corruption_unavailable() {
    for error in [
        Error::SecurityLimitExceeded("access".into()),
        Error::Io("read".into()),
    ] {
        assert_eq!(unavailable(error.clone()), Err(error));
    }
    for error in [Error::CorruptRecord, Error::CorruptHeader, Error::Truncated] {
        assert!(unavailable(error).is_ok());
    }
}
