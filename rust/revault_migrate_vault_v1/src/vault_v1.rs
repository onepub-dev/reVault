use revault_lockbox_api_v1::{
    ContactKeyPair, ContactPublicKey, FormDefinition, FormFieldKind, ListOptions, Lockbox,
    LockboxEntryKind, LockboxId, LockboxOpen, LockboxPath, OwnerSigningKeyPair,
    OwnerSigningPublicKey, SecretString, SecretVec, VariableName, VariableSensitivity,
};
use revault_migration_format::{
    ArtifactKind, ArtifactWriter, FormDefinitionRecord, FormFieldRecord, MigrationError,
    MigrationHeader, MigrationRecord, ProfileGenerationRecord, ProfileRecord, Result, SecretBytes,
    VaultRecord,
};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read};
use std::path::Path;

const VAULT_FILE_NAME: &str = "local-vault.lbox";
const _: () = assert!(revault_vault_api_v1::CURRENT_VAULT_STRUCTURE_VERSION == 1);

/// Exports native Vault structure version 1 without exposing that reader to
/// the current `revault_vault_api`. This function is intended for the
/// crates.io-published historical exporter executable.
pub fn export_vault_v1(
    root: &Path,
    vault_password: &[u8],
    output: &Path,
    artifact_passphrase: &[u8],
    operation_id: [u8; 16],
) -> Result<u64> {
    let vault_password = SecretString::try_from_slice(vault_password).map_err(core_error)?;
    let vault_path = root.join(VAULT_FILE_NAME);
    let source_fingerprint = fingerprint_file(&vault_path)?;
    let lockbox =
        Lockbox::open(&vault_path, LockboxOpen::Password(&vault_password)).map_err(core_error)?;
    let structure = lockbox
        .get_file(&LockboxPath::new("/vault/structure-version").map_err(core_error)?)
        .map_err(core_error)?;
    let structure = std::str::from_utf8(&structure)
        .map_err(|_| MigrationError::Serialization("vault version is not UTF-8".into()))?
        .trim()
        .parse::<u32>()
        .map_err(|_| MigrationError::Serialization("vault version is invalid".into()))?;
    if structure != 1 {
        return Err(MigrationError::InvalidHeader(format!(
            "v1 exporter cannot export vault structure version {structure}"
        )));
    }
    let file = create_new(output)?;
    let header = MigrationHeader {
        artifact_kind: ArtifactKind::Vault,
        source_native_version: 1,
        migration_schema_version: 1,
        target_native_version: Some(2),
        operation_id,
    };
    let mut writer = ArtifactWriter::new(BufWriter::new(file), header, artifact_passphrase)?;
    writer.write_json(&MigrationRecord::Vault(VaultRecord::Start {
        structure_version: 1,
    }))?;

    for name in v1_profile_names(&lockbox)? {
        let history_path =
            LockboxPath::new(format!("/identity_histories/{name}.lbih")).map_err(core_error)?;
        let history = lockbox
            .get_file(&history_path)
            .ok()
            .map(|bytes| decode_history(&name, &bytes))
            .transpose()?;
        let email_path =
            LockboxPath::new(format!("/identity_emails/{name}.lbie")).map_err(core_error)?;
        let email = lockbox
            .get_file(&email_path)
            .ok()
            .map(|bytes| decode_email(&bytes))
            .transpose()?;
        let mut generations = Vec::new();
        if let Some(history) = history.as_ref() {
            generations.reserve(history.generations.len());
        }
        for generation in history
            .as_ref()
            .map(|history| history.generations.clone())
            .unwrap_or_default()
        {
            let encoded_name = encode_name_hex(&name);
            let private_name = VariableName::new(format!(
                "LOCKBOX_VAULT_PRIVATE_KEY_{encoded_name}_GEN_{:04}",
                generation.index
            ))
            .map_err(core_error)?;
            let signing_name = VariableName::new(format!(
                "LOCKBOX_VAULT_SIGNING_KEY_{encoded_name}_GEN_{:04}",
                generation.index
            ))
            .map_err(core_error)?;
            let private_encoded = secret_variable(&lockbox, &private_name)?;
            let signing_encoded = secret_variable(&lockbox, &signing_name)?;
            generations.push(ProfileGenerationRecord {
                index: generation.index,
                status: generation.status,
                created_at_unix_ms: generation.created_at_unix_ms,
                retired_at_unix_ms: generation.retired_at_unix_ms,
                contact_fingerprint: generation.contact_fingerprint,
                private_open_key: SecretBytes::new(decode_hex_secret(private_encoded.as_slice())?),
                owner_signing_key: SecretBytes::new(decode_hex_secret(signing_encoded.as_slice())?),
            });
        }
        if history.is_none() {
            let encoded_name = encode_name_hex(&name);
            let private_name =
                VariableName::new(format!("LOCKBOX_VAULT_PRIVATE_KEY_{encoded_name}"))
                    .map_err(core_error)?;
            let signing_name =
                VariableName::new(format!("LOCKBOX_VAULT_SIGNING_KEY_{encoded_name}"))
                    .map_err(core_error)?;
            let private_encoded = secret_variable(&lockbox, &private_name)?;
            let private_open_key = decode_hex_secret(private_encoded.as_slice())?;
            let keypair = ContactKeyPair::from_private_key_record(
                SecretVec::try_from_vec(private_open_key.clone()).map_err(core_error)?,
            )
            .map_err(core_error)?;
            let owner_signing_key = match optional_secret_variable(&lockbox, &signing_name)? {
                Some(value) => decode_hex_secret(value.as_slice())?,
                None => {
                    let key = OwnerSigningKeyPair::generate().map_err(core_error)?;
                    let record = key.private_key_record().map_err(core_error)?;
                    record
                        .with_bytes(|bytes| bytes.to_vec())
                        .map_err(core_error)?
                }
            };
            generations.push(ProfileGenerationRecord {
                index: 1,
                status: "active".to_string(),
                created_at_unix_ms: 0,
                retired_at_unix_ms: None,
                contact_fingerprint: contact_fingerprint(&keypair),
                private_open_key: SecretBytes::new(private_open_key),
                owner_signing_key: SecretBytes::new(owner_signing_key),
            });
        }
        writer.write_json(&MigrationRecord::Vault(VaultRecord::Profile(
            ProfileRecord {
                name,
                active_generation: history
                    .as_ref()
                    .map_or(1, |history| history.active_generation),
                email,
                generations,
            },
        )))?;
    }

    for name in record_names(&lockbox, "/contacts", ".pub")? {
        if name.ends_with(".signing") {
            continue;
        }
        let public_path = LockboxPath::new(format!("/contacts/{name}.pub")).map_err(core_error)?;
        let public_key = lockbox.get_file(&public_path).map_err(core_error)?;
        ContactPublicKey::from_bytes(&public_key).map_err(core_error)?;
        let signing_path =
            LockboxPath::new(format!("/contacts/{name}.signing.pub")).map_err(core_error)?;
        let signing_public_key = lockbox.get_file(&signing_path).ok();
        if let Some(bytes) = signing_public_key.as_ref() {
            OwnerSigningPublicKey::from_bytes(bytes).map_err(core_error)?;
        }
        writer.write_json(&MigrationRecord::Vault(VaultRecord::Contact {
            name,
            public_key,
            signing_public_key,
        }))?;
    }
    for latest in lockbox.list_form_definitions().map_err(core_error)? {
        for definition in lockbox
            .list_form_definition_revisions(&latest.type_id)
            .map_err(core_error)?
        {
            writer.write_json(&MigrationRecord::Vault(VaultRecord::FormDefinition(
                form_to_record(definition),
            )))?;
        }
    }

    for entry in list_files(&lockbox, "/known_lockboxes")? {
        let bytes = lockbox.get_file(&entry.path).map_err(core_error)?;
        let known = decode_known_lockbox(&bytes)?;
        writer.write_json(&MigrationRecord::Vault(VaultRecord::KnownLockbox {
            lockbox_id: known.0,
            path: known.1,
            last_seen_unix_ms: known.2,
        }))?;
    }
    for entry in list_files(&lockbox, "/access_slots")? {
        let bytes = lockbox.get_file(&entry.path).map_err(core_error)?;
        let label = decode_access_label(&bytes)?;
        writer.write_json(&MigrationRecord::Vault(VaultRecord::AccessLabel {
            lockbox_id: label.0,
            slot_id: label.1,
            name: label.2,
            updated_at_unix_ms: label.3,
        }))?;
    }
    for entry in list_files(&lockbox, "/key_directories")? {
        let Some(name) = entry
            .path
            .as_str()
            .rsplit('/')
            .next()
            .and_then(|name| name.strip_suffix(".keydir"))
        else {
            continue;
        };
        let id_bytes = decode_hex(&name.replace('-', ""))?;
        let id = LockboxId::from_bytes(
            id_bytes
                .try_into()
                .map_err(|_| MigrationError::Serialization("invalid key directory id".into()))?,
        );
        writer.write_json(&MigrationRecord::Vault(VaultRecord::KeyDirectory {
            lockbox_id: *id.as_bytes(),
            bytes: SecretBytes::new(lockbox.get_file(&entry.path).map_err(core_error)?),
        }))?;
    }
    for (name, sensitivity) in lockbox.list_variables().map_err(core_error)? {
        if sensitivity != VariableSensitivity::Secret {
            continue;
        }
        let raw = name.as_str().trim_start_matches('/');
        let Some(hex_id) = raw.strip_prefix("LOCKBOX_VAULT_LOCKBOX_PASSWORD_") else {
            continue;
        };
        let id_bytes = decode_hex(hex_id)?;
        let id: [u8; 16] = id_bytes
            .try_into()
            .map_err(|_| MigrationError::Serialization("invalid lockbox password id".into()))?;
        let value = secret_variable(&lockbox, &name)?;
        writer.write_json(&MigrationRecord::Vault(VaultRecord::LockboxPassword {
            lockbox_id: id,
            value,
        }))?;
    }
    let count = writer.records_written();
    writer.write_json(&MigrationRecord::Vault(VaultRecord::End {
        record_count: count,
    }))?;
    writer.finish()?;
    if fingerprint_file(&vault_path)? != source_fingerprint {
        return Err(MigrationError::Io(
            "v1 vault changed while it was being exported".to_string(),
        ));
    }
    Ok(count + 1)
}

fn v1_profile_names<State>(lockbox: &Lockbox<State>) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for (name, _) in lockbox.list_variables().map_err(core_error)? {
        let raw = name.as_str().trim_start_matches('/');
        let Some(hex) = raw.strip_prefix("LOCKBOX_VAULT_PRIVATE_KEY_") else {
            continue;
        };
        if hex.contains("_GEN_") {
            continue;
        }
        names.push(
            String::from_utf8(decode_hex(hex)?).map_err(|_| {
                MigrationError::Serialization("profile name is not UTF-8".to_string())
            })?,
        );
    }
    names.sort();
    names.dedup();
    Ok(names)
}

fn secret_variable<State>(lockbox: &Lockbox<State>, name: &VariableName) -> Result<SecretBytes> {
    lockbox
        .with_secret_variable(name, |value| value.with_bytes(|bytes| bytes.to_vec()))
        .map_err(core_error)?
        .ok_or_else(|| MigrationError::Serialization(format!("missing secret variable {name}")))?
        .map(SecretBytes::new)
        .map_err(core_error)
}

fn optional_secret_variable<State>(
    lockbox: &Lockbox<State>,
    name: &VariableName,
) -> Result<Option<SecretBytes>> {
    lockbox
        .with_secret_variable(name, |value| value.with_bytes(|bytes| bytes.to_vec()))
        .map_err(core_error)?
        .map(|value| value.map(SecretBytes::new).map_err(core_error))
        .transpose()
}

fn contact_fingerprint(keypair: &ContactKeyPair) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(keypair.public_key().to_bytes());
    hasher.finalize()[..16].to_vec()
}

fn decode_hex_secret(value: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(value)
        .map_err(|_| MigrationError::Serialization("private key hex is not UTF-8".into()))?;
    let bytes = decode_hex(text)?;
    // Validate key records while the historical reader still understands them.
    if text.starts_with("4c42583153505256") {
        OwnerSigningKeyPair::from_private_key_record(
            SecretVec::try_from_vec(bytes.clone()).map_err(core_error)?,
        )
        .map_err(core_error)?;
    } else {
        ContactKeyPair::from_private_key_record(
            SecretVec::try_from_vec(bytes.clone()).map_err(core_error)?,
        )
        .map_err(core_error)?;
    }
    Ok(bytes)
}

struct LegacyHistory {
    active_generation: u16,
    generations: Vec<LegacyGeneration>,
}

#[derive(Clone)]
struct LegacyGeneration {
    index: u16,
    status: String,
    created_at_unix_ms: u64,
    retired_at_unix_ms: Option<u64>,
    contact_fingerprint: Vec<u8>,
}

fn decode_history(name: &str, bytes: &[u8]) -> Result<LegacyHistory> {
    let mut reader = Reader::new(bytes);
    if reader.bytes(4)? != b"LBIH" || reader.u16()? != 1 {
        return Err(MigrationError::Serialization(format!(
            "invalid v1 profile history for {name}"
        )));
    }
    let active_generation = reader.u16()?;
    let count = reader.u16()? as usize;
    let mut generations = Vec::with_capacity(count);
    for _ in 0..count {
        let index = reader.u16()?;
        let status = match reader.u16()? {
            1 => "active",
            2 => "retired",
            3 => "compromised",
            other => {
                return Err(MigrationError::Serialization(format!(
                    "unknown v1 generation status {other}"
                )))
            }
        };
        let created_at_unix_ms = reader.u64()?;
        let retired = reader.u8()? != 0;
        let retired_at = reader.u64()?;
        let retired_at_unix_ms = retired.then_some(retired_at);
        let contact_fingerprint = reader.length_bytes()?.to_vec();
        generations.push(LegacyGeneration {
            index,
            status: status.to_string(),
            created_at_unix_ms,
            retired_at_unix_ms,
            contact_fingerprint,
        });
    }
    reader.finish()?;
    Ok(LegacyHistory {
        active_generation,
        generations,
    })
}

fn decode_email(bytes: &[u8]) -> Result<String> {
    let mut reader = Reader::new(bytes);
    if reader.bytes(4)? != b"LBIE" || reader.u16()? != 1 {
        return Err(MigrationError::Serialization(
            "invalid v1 profile email".into(),
        ));
    }
    let value = reader.string()?;
    reader.finish()?;
    Ok(value)
}

fn decode_known_lockbox(bytes: &[u8]) -> Result<([u8; 16], String, u64)> {
    let mut reader = Reader::new(bytes);
    if reader.bytes(4)? != b"LBKL" || reader.u16()? != 1 {
        return Err(MigrationError::Serialization(
            "invalid known lockbox record".into(),
        ));
    }
    let id = reader.bytes(16)?.try_into().unwrap();
    let path = reader.string()?;
    let seen = reader.u64()?;
    reader.finish()?;
    Ok((id, path, seen))
}

fn decode_access_label(bytes: &[u8]) -> Result<([u8; 16], u64, String, u64)> {
    let mut reader = Reader::new(bytes);
    if reader.bytes(4)? != b"LBAS" || reader.u16()? != 1 {
        return Err(MigrationError::Serialization(
            "invalid access label record".into(),
        ));
    }
    let id = reader.bytes(16)?.try_into().unwrap();
    let slot = reader.u64()?;
    let name = reader.string()?;
    let updated = reader.u64()?;
    reader.finish()?;
    Ok((id, slot, name, updated))
}

fn record_names<State>(
    lockbox: &Lockbox<State>,
    root: &str,
    extension: &str,
) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in list_files(lockbox, root)? {
        let Some(name) = entry
            .path
            .as_str()
            .rsplit('/')
            .next()
            .and_then(|name| name.strip_suffix(extension))
        else {
            continue;
        };
        names.push(name.to_string());
    }
    Ok(names)
}

fn list_files<State>(
    lockbox: &Lockbox<State>,
    root: &str,
) -> Result<Vec<revault_lockbox_api_v1::LockboxEntry>> {
    let root = LockboxPath::new(root).map_err(core_error)?;
    let mut options = ListOptions::new(&root);
    options.recursive = true;
    lockbox
        .list(options)
        .map_err(core_error)?
        .filter_map(|entry| match entry {
            Ok(entry) if entry.kind == LockboxEntryKind::File => Some(Ok(entry)),
            Ok(_) => None,
            Err(err) => Some(Err(core_error(err))),
        })
        .collect()
}

fn encode_name_hex(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect()
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

fn decode_hex(value: &str) -> Result<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return Err(MigrationError::Serialization(
            "hex value has odd length".into(),
        ));
    }
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|_| MigrationError::Serialization("invalid hex value".into()))
        })
        .collect()
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    fn bytes(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| MigrationError::Serialization("legacy record length overflow".into()))?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| MigrationError::Serialization("legacy record is truncated".into()))?;
        self.offset = end;
        Ok(value)
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.bytes(1)?[0])
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(self.bytes(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(self.bytes(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_be_bytes(self.bytes(8)?.try_into().unwrap()))
    }
    fn length_bytes(&mut self) -> Result<&'a [u8]> {
        let len = self.u32()? as usize;
        self.bytes(len)
    }
    fn string(&mut self) -> Result<String> {
        String::from_utf8(self.length_bytes()?.to_vec())
            .map_err(|_| MigrationError::Serialization("legacy string is not UTF-8".into()))
    }
    fn finish(&self) -> Result<()> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(MigrationError::Serialization(
                "legacy record has trailing bytes".into(),
            ))
        }
    }
}

fn create_new(path: &Path) -> Result<File> {
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|err| MigrationError::Io(err.to_string()))
}

fn fingerprint_file(path: &Path) -> Result<[u8; 32]> {
    let mut file = File::open(path).map_err(io_error)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(io_error)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(digest.finalize().into())
}

fn core_error(err: impl std::fmt::Display) -> MigrationError {
    MigrationError::Serialization(err.to_string())
}

fn io_error(err: std::io::Error) -> MigrationError {
    MigrationError::Io(err.to_string())
}
