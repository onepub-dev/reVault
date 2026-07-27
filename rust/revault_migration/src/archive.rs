use crate::{
    ArchiveRecord, ArtifactKind, ArtifactReader, ArtifactWriter, FormDefinitionRecord,
    FormFieldRecord, FormRecordValue, FormValueRecord, MigrationError, MigrationHeader,
    MigrationPassphrase, MigrationRecord, Result, SecretBytes, JSON_FRAME_TYPE, RAW_FRAME_TYPE,
};
use revault_lockbox_api::{
    FormDefinition, FormFieldDefinition, FormFieldKind, FormFieldValue, FormRecord, FormTypeId,
    FormValue, ListOptions, Lockbox, LockboxEntryKind, LockboxId, LockboxPath, OpenFileOptions,
    OwnerSigningKeyPair, SecretString, SecretVec, VariableName, VariableSensitivity,
    LOCKBOX_FORMAT_VERSION,
};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use zeroize::{Zeroize, Zeroizing};

const FILE_CHUNK_BYTES: usize = 4 * 1024 * 1024;

/// Exports archive.
pub fn export_archive<State, P: MigrationPassphrase + ?Sized>(
    lockbox: &Lockbox<State>,
    output: &Path,
    artifact_passphrase: &P,
    operation_id: [u8; 16],
) -> Result<u64> {
    let header = MigrationHeader {
        artifact_kind: ArtifactKind::Archive,
        source_native_version: u32::from(LOCKBOX_FORMAT_VERSION),
        migration_schema_version: 1,
        target_native_version: Some(u32::from(LOCKBOX_FORMAT_VERSION)),
        operation_id,
    };
    let mut writer = ArtifactWriter::new_with_passphrase(
        BufWriter::new(create_new(output)?),
        header,
        artifact_passphrase,
    )?;
    let (content_key, key_directory) = lockbox
        .export_migration_key_material()
        .map_err(core_error)?;
    writer.write_json(&MigrationRecord::Archive(ArchiveRecord::Start {
        archive_id: *lockbox.lockbox_id().as_bytes(),
        format_version: u32::from(LOCKBOX_FORMAT_VERSION),
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

/// Imports archive.
pub fn import_archive<P: MigrationPassphrase + ?Sized>(
    artifact: &Path,
    artifact_passphrase: &P,
    output: &Path,
    signing_key: &OwnerSigningKeyPair,
) -> Result<u64> {
    if output.exists() {
        return Err(MigrationError::Io(format!(
            "destination already exists: {}",
            output.display()
        )));
    }
    let file = File::open(artifact).map_err(io_error)?;
    let mut reader =
        ArtifactReader::new_with_passphrase(BufReader::new(file), artifact_passphrase)?;
    if reader.header().artifact_kind != ArtifactKind::Archive {
        return Err(MigrationError::InvalidHeader(
            "artifact is not an archive migration".to_string(),
        ));
    }
    let mut lockbox: Option<Lockbox> = None;
    let mut count = 0u64;
    let mut saw_end = false;
    while let Some((frame_type, payload)) = reader.next_frame()? {
        let mut payload = Zeroizing::new(payload);
        if frame_type != JSON_FRAME_TYPE {
            return Err(MigrationError::CorruptFrame(
                "file chunk appears outside a file record".to_string(),
            ));
        }
        let record: MigrationRecord = serde_json::from_slice(&payload)
            .map_err(|err| MigrationError::Serialization(err.to_string()))?;
        payload.zeroize();
        let MigrationRecord::Archive(record) = record else {
            return Err(MigrationError::CorruptFrame(
                "vault record found in archive artifact".to_string(),
            ));
        };
        match record {
            ArchiveRecord::Start {
                archive_id,
                content_key,
                key_directory,
                ..
            } => {
                if lockbox.is_some() || count != 0 {
                    return Err(MigrationError::CorruptFrame(
                        "duplicate or misplaced archive start".to_string(),
                    ));
                }
                let mut created =
                    Lockbox::create_with_lockbox_id(content_key, LockboxId::from_bytes(archive_id));
                created.set_owner_signing_key(signing_key.try_clone().map_err(core_error)?);
                created
                    .import_migration_key_directory(key_directory.as_slice())
                    .map_err(core_error)?;
                lockbox = Some(created);
            }
            ArchiveRecord::FileStart {
                file_id,
                path,
                size,
                permissions,
            } => {
                let lockbox = lockbox.as_mut().ok_or_else(|| {
                    MigrationError::CorruptFrame("archive start is missing".to_string())
                })?;
                let path = LockboxPath::new(path).map_err(core_error)?;
                let mut handle = lockbox
                    .open_file_for_write(
                        &path,
                        OpenFileOptions {
                            create: true,
                            truncate: true,
                            permissions,
                        },
                    )
                    .map_err(core_error)?;
                let mut offset = 0u64;
                let mut digest = Sha256::new();
                loop {
                    let Some((next_type, next_payload)) = reader.next_frame()? else {
                        return Err(MigrationError::Incomplete);
                    };
                    let mut next_payload = Zeroizing::new(next_payload);
                    if next_type == RAW_FRAME_TYPE {
                        if next_payload.len() < 16 {
                            return Err(MigrationError::CorruptFrame(
                                "file chunk header is truncated".to_string(),
                            ));
                        }
                        let chunk_file = u64::from_le_bytes(next_payload[0..8].try_into().unwrap());
                        let chunk_offset =
                            u64::from_le_bytes(next_payload[8..16].try_into().unwrap());
                        if chunk_file != file_id || chunk_offset != offset {
                            return Err(MigrationError::CorruptFrame(
                                "file chunk sequence is invalid".to_string(),
                            ));
                        }
                        let bytes = &next_payload[16..];
                        handle.write_all(bytes).map_err(io_error)?;
                        digest.update(bytes);
                        offset = offset.saturating_add(bytes.len() as u64);
                        count = count.saturating_add(1);
                        next_payload.zeroize();
                        continue;
                    }
                    if next_type != JSON_FRAME_TYPE {
                        return Err(MigrationError::CorruptFrame(
                            "unexpected frame in file record".to_string(),
                        ));
                    }
                    let end: MigrationRecord = serde_json::from_slice(&next_payload)
                        .map_err(|err| MigrationError::Serialization(err.to_string()))?;
                    next_payload.zeroize();
                    let MigrationRecord::Archive(ArchiveRecord::FileEnd {
                        file_id: end_id,
                        size: end_size,
                        sha256,
                    }) = end
                    else {
                        return Err(MigrationError::CorruptFrame(
                            "file record does not end with file_end".to_string(),
                        ));
                    };
                    let actual_sha256: [u8; 32] = digest.clone().finalize().into();
                    if end_id != file_id
                        || end_size != size
                        || offset != size
                        || sha256 != actual_sha256
                    {
                        return Err(MigrationError::CorruptFrame(
                            "file size or checksum mismatch".to_string(),
                        ));
                    }
                    handle.flush().map_err(core_error)?;
                    count = count.saturating_add(1);
                    break;
                }
            }
            ArchiveRecord::Symlink { path, target } => {
                lockbox
                    .as_mut()
                    .ok_or_else(|| MigrationError::CorruptFrame("archive start is missing".into()))?
                    .add_symlink(
                        &LockboxPath::new(path).map_err(core_error)?,
                        &LockboxPath::new(target).map_err(core_error)?,
                        false,
                    )
                    .map_err(core_error)?;
            }
            ArchiveRecord::Variable {
                name,
                sensitivity,
                value,
            } => {
                let lockbox = lockbox.as_mut().ok_or_else(|| {
                    MigrationError::CorruptFrame("archive start is missing".into())
                })?;
                let name = VariableName::new(name).map_err(core_error)?;
                match sensitivity.as_str() {
                    "normal" => lockbox
                        .set_variable(
                            &name,
                            &String::from_utf8(value.into_vec()).map_err(|_| {
                                MigrationError::Serialization(
                                    "normal variable is not UTF-8".to_string(),
                                )
                            })?,
                        )
                        .map_err(core_error)?,
                    "secret" => lockbox
                        .set_secret_variable(
                            &name,
                            &SecretString::try_from_bytes(value.into_vec()).map_err(core_error)?,
                        )
                        .map_err(core_error)?,
                    _ => {
                        return Err(MigrationError::Serialization(format!(
                            "unknown variable sensitivity {sensitivity}"
                        )))
                    }
                }
            }
            ArchiveRecord::FormDefinition(value) => {
                lockbox
                    .as_mut()
                    .ok_or_else(|| MigrationError::CorruptFrame("archive start is missing".into()))?
                    .import_form_definition(form_from_record(value)?)
                    .map_err(core_error)?;
            }
            ArchiveRecord::FormRecord(value) => {
                import_form_record(
                    lockbox.as_mut().ok_or_else(|| {
                        MigrationError::CorruptFrame("archive start is missing".into())
                    })?,
                    value,
                )?;
            }
            ArchiveRecord::End { record_count } => {
                if record_count != count {
                    return Err(MigrationError::CorruptFrame(format!(
                        "archive record count mismatch: expected {record_count}, got {count}"
                    )));
                }
                saw_end = true;
            }
            ArchiveRecord::FileChunk { .. } | ArchiveRecord::FileEnd { .. } => {
                return Err(MigrationError::CorruptFrame(
                    "misplaced file record".to_string(),
                ));
            }
        }
        count = count.saturating_add(1);
    }
    if !reader.is_complete() || !saw_end {
        return Err(MigrationError::Incomplete);
    }
    let mut lockbox = lockbox.ok_or(MigrationError::Incomplete)?;
    lockbox.commit().map_err(core_error)?;
    lockbox.write_to_path(output).map_err(core_error)?;
    Ok(count)
}

/// Verifies archive artifact.
pub fn verify_archive_artifact<P: MigrationPassphrase + ?Sized>(
    path: &Path,
    passphrase: &P,
) -> Result<u64> {
    let file = File::open(path).map_err(io_error)?;
    let mut reader = ArtifactReader::new_with_passphrase(BufReader::new(file), passphrase)?;
    if reader.header().artifact_kind != ArtifactKind::Archive {
        return Err(MigrationError::InvalidHeader(
            "artifact is not an archive migration".to_string(),
        ));
    }
    let mut count = 0u64;
    let mut saw_start = false;
    let mut saw_end = false;
    let mut file_state: Option<(u64, u64, u64, Sha256)> = None;
    while let Some((frame_type, payload)) = reader.next_frame()? {
        let mut payload = Zeroizing::new(payload);
        if saw_end {
            return Err(MigrationError::CorruptFrame(
                "archive record appears after end".to_string(),
            ));
        }
        if frame_type == RAW_FRAME_TYPE {
            let Some((file_id, expected_size, offset, digest)) = file_state.as_mut() else {
                return Err(MigrationError::CorruptFrame(
                    "file chunk appears outside a file record".to_string(),
                ));
            };
            if payload.len() < 16 {
                return Err(MigrationError::CorruptFrame(
                    "file chunk header is truncated".to_string(),
                ));
            }
            let chunk_file = u64::from_le_bytes(payload[0..8].try_into().unwrap());
            let chunk_offset = u64::from_le_bytes(payload[8..16].try_into().unwrap());
            if chunk_file != *file_id || chunk_offset != *offset {
                return Err(MigrationError::CorruptFrame(
                    "file chunk sequence is invalid".to_string(),
                ));
            }
            let bytes = &payload[16..];
            *offset = offset.saturating_add(bytes.len() as u64);
            if *offset > *expected_size {
                return Err(MigrationError::CorruptFrame(
                    "file chunks exceed the declared size".to_string(),
                ));
            }
            digest.update(bytes);
            count = count.saturating_add(1);
            payload.zeroize();
            continue;
        }
        if frame_type != JSON_FRAME_TYPE {
            return Err(MigrationError::CorruptFrame(
                "unknown archive migration frame".to_string(),
            ));
        }
        let record: MigrationRecord = serde_json::from_slice(&payload)
            .map_err(|err| MigrationError::Serialization(err.to_string()))?;
        payload.zeroize();
        let MigrationRecord::Archive(record) = record else {
            return Err(MigrationError::CorruptFrame(
                "vault record found in archive artifact".to_string(),
            ));
        };
        match record {
            ArchiveRecord::Start { .. } if !saw_start && count == 0 => saw_start = true,
            ArchiveRecord::Start { .. } => {
                return Err(MigrationError::CorruptFrame(
                    "duplicate or misplaced archive start".to_string(),
                ));
            }
            ArchiveRecord::FileStart { file_id, size, .. } if saw_start && file_state.is_none() => {
                file_state = Some((file_id, size, 0, Sha256::new()));
            }
            ArchiveRecord::FileEnd {
                file_id,
                size,
                sha256,
            } => {
                let Some((open_id, expected_size, offset, digest)) = file_state.take() else {
                    return Err(MigrationError::CorruptFrame(
                        "file end appears outside a file record".to_string(),
                    ));
                };
                let actual: [u8; 32] = digest.finalize().into();
                if file_id != open_id
                    || size != expected_size
                    || offset != expected_size
                    || sha256 != actual
                {
                    return Err(MigrationError::CorruptFrame(
                        "file size or checksum mismatch".to_string(),
                    ));
                }
            }
            ArchiveRecord::FileChunk { .. } => {
                return Err(MigrationError::CorruptFrame(
                    "JSON file chunk is not supported".to_string(),
                ));
            }
            ArchiveRecord::End { record_count }
                if saw_start && file_state.is_none() && !saw_end =>
            {
                if record_count != count {
                    return Err(MigrationError::CorruptFrame(format!(
                        "archive record count mismatch: expected {record_count}, got {count}"
                    )));
                }
                saw_end = true;
            }
            ArchiveRecord::End { .. } => {
                return Err(MigrationError::CorruptFrame(
                    "misplaced archive end".to_string(),
                ));
            }
            _ if !saw_start || file_state.is_some() => {
                return Err(MigrationError::CorruptFrame(
                    "archive record appears in an invalid position".to_string(),
                ));
            }
            _ => {}
        }
        count = count.saturating_add(1);
    }
    if !reader.is_complete() || !saw_start || !saw_end || file_state.is_some() {
        return Err(MigrationError::Incomplete);
    }
    Ok(count)
}

/// Re-encrypts and advances an archive migration artifact through each
/// registered logical schema step. Schema 1 is currently the latest schema,
/// so this performs a validated canonical rewrite.
pub fn upgrade_archive_artifact<P: MigrationPassphrase + ?Sized>(
    input: &Path,
    output: &Path,
    passphrase: &P,
) -> Result<u64> {
    let file = File::open(input).map_err(io_error)?;
    let mut reader = ArtifactReader::new_with_passphrase(BufReader::new(file), passphrase)?;
    if reader.header().artifact_kind != ArtifactKind::Archive {
        return Err(MigrationError::InvalidHeader(
            "artifact is not an archive migration".to_string(),
        ));
    }
    if reader.header().migration_schema_version > 1 {
        return Err(MigrationError::InvalidHeader(format!(
            "archive migration schema {} is newer than this build supports",
            reader.header().migration_schema_version
        )));
    }
    let header = MigrationHeader {
        artifact_kind: ArtifactKind::Archive,
        source_native_version: reader.header().source_native_version,
        migration_schema_version: 1,
        target_native_version: Some(u32::from(LOCKBOX_FORMAT_VERSION)),
        operation_id: reader.header().operation_id,
    };
    let mut writer = ArtifactWriter::new_with_passphrase(
        BufWriter::new(create_new(output)?),
        header,
        passphrase,
    )?;
    let mut count = 0u64;
    while let Some((frame_type, payload)) = reader.next_frame()? {
        let mut payload = Zeroizing::new(payload);
        match frame_type {
            JSON_FRAME_TYPE => {
                let record: MigrationRecord = serde_json::from_slice(&payload)
                    .map_err(|err| MigrationError::Serialization(err.to_string()))?;
                if !matches!(record, MigrationRecord::Archive(_)) {
                    return Err(MigrationError::CorruptFrame(
                        "vault record found in archive artifact".to_string(),
                    ));
                }
                writer.write_json(&record)?;
                payload.zeroize();
            }
            RAW_FRAME_TYPE => {
                writer.write_raw(&payload)?;
                payload.zeroize();
            }
            _ => {
                return Err(MigrationError::CorruptFrame(
                    "unknown archive migration frame".to_string(),
                ));
            }
        }
        count = count.saturating_add(1);
    }
    if !reader.is_complete() {
        return Err(MigrationError::Incomplete);
    }
    writer.finish()?;
    Ok(count)
}

fn import_form_record(lockbox: &mut Lockbox, value: FormRecordValue) -> Result<()> {
    let path = LockboxPath::new(value.path).map_err(core_error)?;
    let mut values = Vec::with_capacity(value.values.len());
    for field in value.values {
        let text = String::from_utf8(field.value.into_vec()).map_err(|_| {
            MigrationError::Serialization("form field value is not UTF-8".to_string())
        })?;
        let form_value = if field.secret {
            FormValue::secret(SecretString::try_from_bytes(text.into_bytes()).map_err(core_error)?)
        } else {
            FormValue::normal(text)
        };
        values.push(FormFieldValue {
            field_id: field.field_id,
            captured_label: field.captured_label,
            kind: parse_form_kind(&field.kind)?,
            value: form_value,
        });
    }
    lockbox
        .import_migration_form_record(FormRecord {
            path,
            name: value.name,
            type_id: FormTypeId::new(value.type_id).map_err(core_error)?,
            definition_alias: value.definition_alias,
            definition_revision: value.definition_revision,
            values,
        })
        .map_err(core_error)
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

fn form_from_record(value: FormDefinitionRecord) -> Result<FormDefinition> {
    Ok(FormDefinition {
        type_id: FormTypeId::new(value.type_id).map_err(core_error)?,
        alias: value.alias,
        revision: value.revision,
        name: value.name,
        description: value.description,
        fields: value
            .fields
            .into_iter()
            .map(|field| {
                Ok(FormFieldDefinition {
                    id: field.id,
                    label: field.label,
                    kind: parse_form_kind(&field.kind)?,
                    required: field.required,
                })
            })
            .collect::<Result<Vec<_>>>()?,
    })
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

fn parse_form_kind(value: &str) -> Result<FormFieldKind> {
    match value {
        "text" => Ok(FormFieldKind::Text),
        "secret" => Ok(FormFieldKind::Secret),
        "url" => Ok(FormFieldKind::Url),
        "email" => Ok(FormFieldKind::Email),
        "date" => Ok(FormFieldKind::Date),
        "month" => Ok(FormFieldKind::Month),
        "notes" => Ok(FormFieldKind::Notes),
        "number" => Ok(FormFieldKind::Number),
        other => Err(MigrationError::Serialization(format!(
            "unknown form field kind {other}"
        ))),
    }
}

fn secret_bytes(value: &SecretVec) -> Result<Vec<u8>> {
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
