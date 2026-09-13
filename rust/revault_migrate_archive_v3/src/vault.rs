//! Logical vault-v3 export through the frozen reader from commit e5f534bfde6c7165f8168ada95ce98af2526805c.
use revault_lockbox_api::{FormDefinition, SecretVec};
use revault_migration_format::*;
use revault_vault_api::{ProfileGenerationStatus, VaultDirectory, CURRENT_VAULT_STRUCTURE_VERSION};
use std::{
    fs::{File, OpenOptions},
    io::BufWriter,
    path::Path,
};

/// Export every persisted vault profile, credential, contact and schema.
pub fn export_vault<P: MigrationPassphrase + ?Sized>(
    vault: &VaultDirectory,
    output: &Path,
    artifact_passphrase: &P,
    operation_id: [u8; 16],
) -> Result<u64> {
    let file = create_new(output)?;
    let header = MigrationHeader {
        artifact_kind: ArtifactKind::Vault,
        source_native_version: vault.structure_version().map_err(core_error)?,
        migration_schema_version: 3,
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

    for name in vault.list_password_profiles().map_err(core_error)? {
        let password = vault.load_profile_password(&name).map_err(core_error)?;
        let bytes = password
            .with_bytes(|bytes| bytes.to_vec())
            .map_err(core_error)?;
        writer.write_json(&MigrationRecord::Vault(VaultRecord::PasswordProfile {
            name,
            password: SecretBytes::new(bytes),
        }))?;
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
                kind: format!("{:?}", field.kind).to_ascii_lowercase(),
                required: field.required,
            })
            .collect(),
    }
}

fn generation_status_name(value: ProfileGenerationStatus) -> &'static str {
    match value {
        ProfileGenerationStatus::Active => "active",
        ProfileGenerationStatus::Retired => "retired",
        ProfileGenerationStatus::Compromised => "compromised",
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
