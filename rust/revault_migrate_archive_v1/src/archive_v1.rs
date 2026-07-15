use revault_lockbox_api::{
    FormDefinition, FormFieldKind, FormValue, ListOptions, Lockbox, LockboxEntryKind, LockboxPath,
    VariableSensitivity, LOCKBOX_FORMAT_VERSION,
};
use revault_migration_format::{
    ArchiveRecord, ArtifactKind, ArtifactWriter, FormDefinitionRecord, FormFieldRecord,
    FormRecordValue, FormValueRecord, MigrationError, MigrationHeader, MigrationRecord, Result,
    SecretBytes,
};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read};
use std::path::Path;

const FILE_CHUNK_BYTES: usize = 4 * 1024 * 1024;

/// Streams a native archive-format-v1 lockbox into migration schema 1.
pub fn export_archive_v1<State>(
    lockbox: &Lockbox<State>,
    output: &Path,
    artifact_passphrase: &[u8],
    operation_id: [u8; 16],
) -> Result<u64> {
    if u32::from(LOCKBOX_FORMAT_VERSION) != 1 {
        return Err(MigrationError::InvalidHeader(
            "the v1 exporter was built with a non-v1 archive API".to_string(),
        ));
    }
    let header = MigrationHeader {
        artifact_kind: ArtifactKind::Archive,
        source_native_version: 1,
        migration_schema_version: 1,
        target_native_version: None,
        operation_id,
    };
    let mut writer = ArtifactWriter::new(
        BufWriter::new(create_new(output)?),
        header,
        artifact_passphrase,
    )?;
    let (content_key, key_directory) = lockbox
        .export_migration_key_material()
        .map_err(core_error)?;
    writer.write_json(&MigrationRecord::Archive(ArchiveRecord::Start {
        archive_id: *lockbox.lockbox_id().as_bytes(),
        format_version: 1,
        content_key: SecretBytes::new(secret_bytes(&content_key)?),
        key_directory: SecretBytes::new(key_directory),
    }))?;

    let root = LockboxPath::new("/").map_err(core_error)?;
    let mut options = ListOptions::new(&root);
    options.recursive = true;
    let mut file_id = 0u64;
    for entry in lockbox.list(options).map_err(core_error)? {
        let entry = entry.map_err(core_error)?;
        match entry.kind {
            LockboxEntryKind::Directory => {}
            LockboxEntryKind::Symlink => {
                let target = lockbox
                    .get_symlink_target(&entry.path)
                    .map_err(core_error)?;
                writer.write_json(&MigrationRecord::Archive(ArchiveRecord::Symlink {
                    path: entry.path.to_string(),
                    target: target.to_string(),
                }))?;
            }
            LockboxEntryKind::File => {
                file_id = file_id.saturating_add(1);
                writer.write_json(&MigrationRecord::Archive(ArchiveRecord::FileStart {
                    file_id,
                    path: entry.path.to_string(),
                    size: entry.len,
                    permissions: Some(entry.permissions),
                }))?;
                let mut reader = lockbox.open_file(&entry.path).map_err(core_error)?;
                let mut offset = 0u64;
                let mut digest = Sha256::new();
                let mut chunk = vec![0u8; FILE_CHUNK_BYTES];
                loop {
                    let read = reader.read(&mut chunk).map_err(io_error)?;
                    if read == 0 {
                        break;
                    }
                    digest.update(&chunk[..read]);
                    let mut frame = Vec::with_capacity(16 + read);
                    frame.extend_from_slice(&file_id.to_le_bytes());
                    frame.extend_from_slice(&offset.to_le_bytes());
                    frame.extend_from_slice(&chunk[..read]);
                    writer.write_raw(&frame)?;
                    offset = offset.saturating_add(read as u64);
                }
                writer.write_json(&MigrationRecord::Archive(ArchiveRecord::FileEnd {
                    file_id,
                    size: offset,
                    sha256: digest.finalize().into(),
                }))?;
            }
        }
    }

    for (name, sensitivity) in lockbox.list_variables().map_err(core_error)? {
        let value = match sensitivity {
            VariableSensitivity::Normal => lockbox
                .get_variable(&name)
                .map_err(core_error)?
                .unwrap_or_default()
                .into_bytes(),
            VariableSensitivity::Secret => lockbox
                .with_secret_variable(&name, |value| value.with_bytes(|bytes| bytes.to_vec()))
                .map_err(core_error)?
                .ok_or_else(|| MigrationError::CorruptFrame("missing secret variable".into()))?
                .map_err(core_error)?,
        };
        writer.write_json(&MigrationRecord::Archive(ArchiveRecord::Variable {
            name: name.to_string(),
            sensitivity: match sensitivity {
                VariableSensitivity::Normal => "normal",
                VariableSensitivity::Secret => "secret",
            }
            .to_string(),
            value: SecretBytes::new(value),
        }))?;
    }

    for latest in lockbox.list_form_definitions().map_err(core_error)? {
        for definition in lockbox
            .list_form_definition_revisions(&latest.type_id)
            .map_err(core_error)?
        {
            writer.write_json(&MigrationRecord::Archive(ArchiveRecord::FormDefinition(
                form_to_record(definition),
            )))?;
        }
    }
    for record in lockbox.list_form_records().map_err(core_error)? {
        let mut values = Vec::with_capacity(record.values.len());
        for value in record.values {
            let (secret, bytes) = match value.value {
                FormValue::Normal(value) => (false, value.into_bytes()),
                FormValue::Secret(value) => (
                    true,
                    value
                        .with_bytes(|bytes| bytes.to_vec())
                        .map_err(core_error)?,
                ),
            };
            values.push(FormValueRecord {
                field_id: value.field_id,
                captured_label: value.captured_label,
                kind: form_kind_name(value.kind).to_string(),
                secret,
                value: SecretBytes::new(bytes),
            });
        }
        writer.write_json(&MigrationRecord::Archive(ArchiveRecord::FormRecord(
            FormRecordValue {
                path: record.path.to_string(),
                name: record.name,
                type_id: record.type_id.to_string(),
                definition_alias: record.definition_alias,
                definition_revision: record.definition_revision,
                values,
            },
        )))?;
    }
    let count = writer.records_written();
    writer.write_json(&MigrationRecord::Archive(ArchiveRecord::End {
        record_count: count,
    }))?;
    writer.finish()?;
    Ok(count + 1)
}

fn form_to_record(value: FormDefinition) -> FormDefinitionRecord {
    FormDefinitionRecord {
        type_id: value.type_id.to_string(),
        alias: value.alias,
        revision: value.revision,
        name: value.name,
        description: value.description,
        fields: value
            .fields
            .into_iter()
            .map(|field| FormFieldRecord {
                id: field.id,
                label: field.label,
                kind: form_kind_name(field.kind).to_string(),
                required: field.required,
            })
            .collect(),
    }
}

fn form_kind_name(value: FormFieldKind) -> &'static str {
    match value {
        FormFieldKind::Text => "text",
        FormFieldKind::Secret => "secret",
        FormFieldKind::Url => "url",
        FormFieldKind::Email => "email",
        FormFieldKind::Date => "date",
        FormFieldKind::Month => "month",
        FormFieldKind::Notes => "notes",
        FormFieldKind::Number => "number",
    }
}

fn secret_bytes(value: &revault_lockbox_api::SecretVec) -> Result<Vec<u8>> {
    value.with_bytes(|bytes| bytes.to_vec()).map_err(core_error)
}

fn create_new(path: &Path) -> Result<File> {
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(io_error)
}

fn core_error(err: impl std::fmt::Display) -> MigrationError {
    MigrationError::Serialization(err.to_string())
}

fn io_error(err: std::io::Error) -> MigrationError {
    MigrationError::Io(err.to_string())
}
