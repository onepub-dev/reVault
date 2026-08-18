use revault_lockbox_api_v2::{FormDefinition, FormFieldKind};
use revault_migration_format::{
    ArtifactKind, ArtifactWriter, FormDefinitionRecord, FormFieldRecord, MigrationError,
    MigrationHeader, MigrationRecord, ProfileGenerationRecord, ProfileRecord, Result, SecretBytes,
    VaultRecord,
};
use revault_vault_api_v2::{ProfileGenerationStatus, VaultDirectory};
use std::fs::{File, OpenOptions};
use std::io::BufWriter;
use std::path::Path;

/// Streams a native vault-format-v2 vault into logical migration schema 2.
pub fn export_vault_v2(
    vault: &VaultDirectory,
    output: &Path,
    artifact_passphrase: &[u8],
    operation_id: [u8; 16],
) -> Result<u64> {
    if revault_vault_api_v2::CURRENT_VAULT_STRUCTURE_VERSION != 2 {
        return Err(MigrationError::InvalidHeader(
            "the v2 exporter was built with a non-v2 vault API".to_string(),
        ));
    }
    let mut writer = ArtifactWriter::new(
        BufWriter::new(create_new(output)?),
        MigrationHeader {
            artifact_kind: ArtifactKind::Vault,
            source_native_version: 2,
            migration_schema_version: 2,
            target_native_version: None,
            operation_id,
        },
        artifact_passphrase,
    )?;
    writer.write_json(&MigrationRecord::Vault(VaultRecord::Start {
        structure_version: 2,
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
        writer.write_json(&MigrationRecord::Vault(VaultRecord::Profile(ProfileRecord {
            name: name.clone(),
            active_generation: history.active_generation,
            email: vault.profile_email(&name).map_err(core_error)?,
            generations,
        })))?;
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

fn generation_status_name(value: ProfileGenerationStatus) -> &'static str {
    match value {
        ProfileGenerationStatus::Active => "active",
        ProfileGenerationStatus::Retired => "retired",
        ProfileGenerationStatus::Compromised => "compromised",
    }
}

fn secret_bytes(value: &revault_lockbox_api_v2::SecretVec) -> Result<Vec<u8>> {
    value.with_bytes(|bytes| bytes.to_vec()).map_err(core_error)
}

fn create_new(path: &Path) -> Result<File> {
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| MigrationError::Io(error.to_string()))
}

fn core_error(error: impl std::fmt::Display) -> MigrationError {
    MigrationError::Serialization(error.to_string())
}
