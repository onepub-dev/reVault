use crate::{
    ArtifactKind, ArtifactReader, ArtifactWriter, FormDefinitionRecord, FormFieldRecord,
    MigrationError, MigrationHeader, MigrationPassphrase, MigrationRecord, ProfileGenerationRecord,
    ProfileRecord, Result, SecretBytes, VaultRecord,
};
use revault_lockbox_api::{
    ContactKeyPair, ContactPublicKey, FormDefinition, FormFieldDefinition, FormFieldKind,
    FormTypeId, LockboxId, OwnerSigningKeyPair, OwnerSigningPublicKey, SecretString, SecretVec,
};
use revault_vault_api::{
    AccessSlotLabel, KnownLockbox, ProfileGeneration, ProfileGenerationStatus, ProfileHistory,
    VaultDirectory, CURRENT_VAULT_STRUCTURE_VERSION,
};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub fn export_vault_v2<P: MigrationPassphrase + ?Sized>(
    vault: &VaultDirectory,
    output: &Path,
    artifact_passphrase: &P,
    operation_id: [u8; 16],
) -> Result<u64> {
    let file = create_new(output)?;
    let header = MigrationHeader {
        artifact_kind: ArtifactKind::Vault,
        source_native_version: vault.structure_version().map_err(core_error)?,
        migration_schema_version: 2,
        target_native_version: Some(CURRENT_VAULT_STRUCTURE_VERSION),
        operation_id,
    };
    let mut writer =
        ArtifactWriter::new_with_passphrase(BufWriter::new(file), header, artifact_passphrase)?;
    writer.write_json(&MigrationRecord::Vault(VaultRecord::Start {
        structure_version: vault.structure_version().map_err(core_error)?,
    }))?;

    for name in vault.list_private_keys().map_err(core_error)? {
        let history = vault.list_profile_generations(&name).map_err(core_error)?;
        let mut generations = Vec::with_capacity(history.generations.len());
        for item in &history.generations {
            let private = vault
                .load_private_key_generation(&name, item.index)
                .map_err(core_error)?
                .private_key_record()
                .map_err(core_error)?;
            let signing = vault
                .load_owner_signing_key_generation(&name, item.index)
                .map_err(core_error)?
                .private_key_record()
                .map_err(core_error)?;
            generations.push(ProfileGenerationRecord {
                index: item.index,
                status: generation_status_name(item.status).to_string(),
                created_at_unix_ms: item.created_at_unix_ms,
                retired_at_unix_ms: item.retired_at_unix_ms,
                contact_fingerprint: item.contact_fingerprint.clone(),
                private_open_key: SecretBytes::new(secret_bytes(&private)?),
                owner_signing_key: SecretBytes::new(secret_bytes(&signing)?),
            });
        }
        writer.write_json(&MigrationRecord::Vault(VaultRecord::Profile(
            ProfileRecord {
                name: name.clone(),
                active_generation: history.active_generation,
                email: vault.profile_email(&name).map_err(core_error)?,
                generations,
            },
        )))?;
    }

    for contact in vault.list_contacts().map_err(core_error)? {
        let signing_public_key = vault
            .load_contact_signing_key(&contact.name)
            .map(|key| key.to_bytes())
            .ok();
        writer.write_json(&MigrationRecord::Vault(VaultRecord::Contact {
            name: contact.name,
            public_key: contact.key.to_bytes(),
            signing_public_key,
        }))?;
    }

    for latest in vault.list_form_definitions().map_err(core_error)? {
        for definition in vault
            .list_form_definition_revisions(&latest.type_id)
            .map_err(core_error)?
        {
            writer.write_json(&MigrationRecord::Vault(VaultRecord::FormDefinition(
                form_to_record(definition),
            )))?;
        }
    }

    for known in vault.list_known_lockboxes().map_err(core_error)? {
        writer.write_json(&MigrationRecord::Vault(VaultRecord::KnownLockbox {
            lockbox_id: *known.lockbox_id.as_bytes(),
            path: known.path,
            last_seen_unix_ms: known.last_seen_unix_ms,
        }))?;
        for label in vault
            .list_access_slot_labels(known.lockbox_id)
            .map_err(core_error)?
        {
            writer.write_json(&MigrationRecord::Vault(VaultRecord::AccessLabel {
                lockbox_id: *label.lockbox_id.as_bytes(),
                slot_id: label.slot_id,
                name: label.name,
                updated_at_unix_ms: label.updated_at_unix_ms,
            }))?;
        }
        if let Some(password) = vault
            .remembered_lockbox_password(known.lockbox_id)
            .map_err(core_error)?
        {
            let value = password
                .with_bytes(|bytes| bytes.to_vec())
                .map_err(core_error)?;
            writer.write_json(&MigrationRecord::Vault(VaultRecord::LockboxPassword {
                lockbox_id: *known.lockbox_id.as_bytes(),
                value: SecretBytes::new(value),
            }))?;
        }
        if let Ok(bytes) = vault.load_key_directory_backup(known.lockbox_id) {
            writer.write_json(&MigrationRecord::Vault(VaultRecord::KeyDirectory {
                lockbox_id: *known.lockbox_id.as_bytes(),
                bytes: SecretBytes::new(bytes),
            }))?;
        }
    }
    let count = writer.records_written();
    writer.write_json(&MigrationRecord::Vault(VaultRecord::End {
        record_count: count,
    }))?;
    writer.finish()?;
    Ok(count + 1)
}

pub fn import_vault_v2<P: MigrationPassphrase + ?Sized>(
    artifact: &Path,
    artifact_passphrase: &P,
    output_root: &Path,
    vault_password: &SecretString,
) -> Result<u64> {
    if output_root.exists() {
        return Err(MigrationError::Io(format!(
            "destination already exists: {}",
            output_root.display()
        )));
    }
    let file = File::open(artifact).map_err(io_error)?;
    let mut reader =
        ArtifactReader::new_with_passphrase(BufReader::new(file), artifact_passphrase)?;
    require_vault_header(reader.header())?;
    let vault =
        VaultDirectory::replace_for_migration(output_root, vault_password).map_err(core_error)?;
    let mut records = 0u64;
    let mut saw_start = false;
    let mut saw_end = false;
    while let Some(record) = reader.next_json::<MigrationRecord>()? {
        let MigrationRecord::Vault(record) = record else {
            return Err(MigrationError::CorruptFrame(
                "archive record found in vault artifact".to_string(),
            ));
        };
        match record {
            VaultRecord::Start { .. } => {
                if saw_start || records != 0 {
                    return Err(MigrationError::CorruptFrame(
                        "duplicate or misplaced vault start".to_string(),
                    ));
                }
                saw_start = true;
            }
            VaultRecord::Profile(profile) => import_profile(&vault, profile)?,
            VaultRecord::Contact {
                name,
                public_key,
                signing_public_key,
            } => {
                let key = ContactPublicKey::from_bytes(&public_key).map_err(core_error)?;
                if vault.contact_exists(&name).map_err(core_error)? {
                    vault.delete_contact(&name).map_err(core_error)?;
                }
                vault.store_contact(&name, &key).map_err(core_error)?;
                if let Some(bytes) = signing_public_key {
                    let key = OwnerSigningPublicKey::from_bytes(&bytes).map_err(core_error)?;
                    vault
                        .store_contact_signing_key(&name, &key)
                        .map_err(core_error)?;
                }
            }
            VaultRecord::FormDefinition(value) => {
                vault
                    .import_form_definition(form_from_record(value)?)
                    .map_err(core_error)?;
            }
            VaultRecord::KnownLockbox {
                lockbox_id,
                path,
                last_seen_unix_ms,
            } => vault
                .restore_known_lockbox(KnownLockbox {
                    lockbox_id: LockboxId::from_bytes(lockbox_id),
                    path,
                    last_seen_unix_ms,
                })
                .map_err(core_error)?,
            VaultRecord::AccessLabel {
                lockbox_id,
                slot_id,
                name,
                updated_at_unix_ms,
            } => vault
                .restore_access_slot_label(AccessSlotLabel {
                    lockbox_id: LockboxId::from_bytes(lockbox_id),
                    slot_id,
                    name,
                    updated_at_unix_ms,
                })
                .map_err(core_error)?,
            VaultRecord::LockboxPassword { lockbox_id, value } => {
                let password =
                    SecretString::try_from_bytes(value.into_vec()).map_err(core_error)?;
                vault
                    .remember_lockbox_password(LockboxId::from_bytes(lockbox_id), &password)
                    .map_err(core_error)?;
            }
            VaultRecord::KeyDirectory { lockbox_id, bytes } => vault
                .store_key_directory_backup(LockboxId::from_bytes(lockbox_id), bytes.as_slice())
                .map_err(core_error)?,
            VaultRecord::End { record_count } => {
                if record_count != records {
                    return Err(MigrationError::CorruptFrame(format!(
                        "vault record count mismatch: expected {record_count}, got {records}"
                    )));
                }
                saw_end = true;
            }
        }
        records = records.saturating_add(1);
    }
    if !reader.is_complete() || !saw_start || !saw_end {
        return Err(MigrationError::Incomplete);
    }
    if vault.structure_version().map_err(core_error)? != CURRENT_VAULT_STRUCTURE_VERSION {
        return Err(MigrationError::CorruptFrame(
            "imported vault has the wrong structure version".to_string(),
        ));
    }
    Ok(records)
}

/// Re-encrypts and advances a vault migration artifact through each registered
/// logical schema step. Vault schema 1→2 is a terminology/schema normalization;
/// its logical profile records are already represented by stable migration IDs.
pub fn upgrade_vault_artifact<P: MigrationPassphrase + ?Sized>(
    input: &Path,
    output: &Path,
    passphrase: &P,
) -> Result<u64> {
    let file = File::open(input).map_err(io_error)?;
    let mut reader = ArtifactReader::new_with_passphrase(BufReader::new(file), passphrase)?;
    require_vault_header(reader.header())?;
    if reader.header().migration_schema_version > 2 {
        return Err(MigrationError::InvalidHeader(format!(
            "vault migration schema {} is newer than this build supports",
            reader.header().migration_schema_version
        )));
    }
    let header = MigrationHeader {
        artifact_kind: ArtifactKind::Vault,
        source_native_version: reader.header().source_native_version,
        migration_schema_version: 2,
        target_native_version: Some(CURRENT_VAULT_STRUCTURE_VERSION),
        operation_id: reader.header().operation_id,
    };
    let mut writer = ArtifactWriter::new_with_passphrase(
        BufWriter::new(create_new(output)?),
        header,
        passphrase,
    )?;
    let mut count = 0u64;
    while let Some(record) = reader.next_json::<MigrationRecord>()? {
        writer.write_json(&upgrade_vault_record(record)?)?;
        count = count.saturating_add(1);
    }
    if !reader.is_complete() {
        return Err(MigrationError::Incomplete);
    }
    writer.finish()?;
    Ok(count)
}

pub fn verify_vault_artifact<P: MigrationPassphrase + ?Sized>(
    path: &Path,
    passphrase: &P,
) -> Result<u64> {
    let file = File::open(path).map_err(io_error)?;
    let mut reader = ArtifactReader::new_with_passphrase(BufReader::new(file), passphrase)?;
    require_vault_header(reader.header())?;
    let mut count = 0u64;
    let mut saw_start = false;
    let mut saw_end = false;
    while let Some(record) = reader.next_json::<MigrationRecord>()? {
        let MigrationRecord::Vault(record) = record else {
            return Err(MigrationError::CorruptFrame(
                "archive record found in vault artifact".to_string(),
            ));
        };
        match record {
            VaultRecord::Start { .. } if !saw_start && count == 0 => saw_start = true,
            VaultRecord::Start { .. } => {
                return Err(MigrationError::CorruptFrame(
                    "duplicate or misplaced vault start".to_string(),
                ));
            }
            VaultRecord::End { record_count } if saw_start && !saw_end => {
                if record_count != count {
                    return Err(MigrationError::CorruptFrame(format!(
                        "vault record count mismatch: expected {record_count}, got {count}"
                    )));
                }
                saw_end = true;
            }
            VaultRecord::End { .. } => {
                return Err(MigrationError::CorruptFrame(
                    "duplicate or misplaced vault end".to_string(),
                ));
            }
            _ if !saw_start || saw_end => {
                return Err(MigrationError::CorruptFrame(
                    "vault record appears outside start/end".to_string(),
                ));
            }
            _ => {}
        }
        count = count.saturating_add(1);
    }
    if !reader.is_complete() || !saw_start || !saw_end {
        return Err(MigrationError::Incomplete);
    }
    Ok(count)
}

fn upgrade_vault_record(record: MigrationRecord) -> Result<MigrationRecord> {
    match record {
        MigrationRecord::Vault(value) => Ok(MigrationRecord::Vault(value)),
        MigrationRecord::Archive(_) => Err(MigrationError::CorruptFrame(
            "archive record found in vault artifact".to_string(),
        )),
    }
}

fn import_profile(vault: &VaultDirectory, profile: ProfileRecord) -> Result<()> {
    let mut history_items = Vec::with_capacity(profile.generations.len());
    let mut key_items = Vec::with_capacity(profile.generations.len());
    for item in profile.generations {
        let key = ContactKeyPair::from_private_key_record(
            SecretVec::try_from_vec(item.private_open_key.into_vec()).map_err(core_error)?,
        )
        .map_err(core_error)?;
        let signing = OwnerSigningKeyPair::from_private_key_record(
            SecretVec::try_from_vec(item.owner_signing_key.into_vec()).map_err(core_error)?,
        )
        .map_err(core_error)?;
        history_items.push(ProfileGeneration {
            index: item.index,
            status: parse_generation_status(&item.status)?,
            contact_fingerprint: item.contact_fingerprint,
            created_at_unix_ms: item.created_at_unix_ms,
            retired_at_unix_ms: item.retired_at_unix_ms,
        });
        key_items.push((item.index, key, signing));
    }
    vault
        .restore_profile_generations(
            ProfileHistory {
                name: profile.name,
                active_generation: profile.active_generation,
                generations: history_items,
            },
            key_items,
            profile.email.as_deref(),
            true,
        )
        .map_err(core_error)
}

fn require_vault_header(header: &MigrationHeader) -> Result<()> {
    if header.artifact_kind != ArtifactKind::Vault {
        return Err(MigrationError::InvalidHeader(
            "artifact is not a vault migration".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn form_to_record(value: FormDefinition) -> FormDefinitionRecord {
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
                kind: format!("{:?}", field.kind).to_ascii_lowercase(),
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

fn generation_status_name(value: ProfileGenerationStatus) -> &'static str {
    match value {
        ProfileGenerationStatus::Active => "active",
        ProfileGenerationStatus::Retired => "retired",
        ProfileGenerationStatus::Compromised => "compromised",
    }
}

fn parse_generation_status(value: &str) -> Result<ProfileGenerationStatus> {
    match value {
        "active" => Ok(ProfileGenerationStatus::Active),
        "retired" => Ok(ProfileGenerationStatus::Retired),
        "compromised" => Ok(ProfileGenerationStatus::Compromised),
        other => Err(MigrationError::Serialization(format!(
            "unknown profile generation status {other}"
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
