use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Serialized byte field that clears its allocation when released.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    /// Creates a value from the supplied data.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Borrows the contained bytes.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Reports whether empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Transfers the contained bytes to the caller.
    pub fn into_vec(mut self) -> Vec<u8> {
        std::mem::take(&mut self.0)
    }
}

impl AsRef<[u8]> for SecretBytes {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Drop for SecretBytes {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Represents artifact kind.
pub enum ArtifactKind {
    /// Represents the vault case.
    Vault,
    /// Represents the archive case.
    Archive,
    /// Represents the journal case.
    Journal,
}

impl ArtifactKind {
    pub(crate) fn code(self) -> u8 {
        match self {
            Self::Vault => 1,
            Self::Archive => 2,
            Self::Journal => 3,
        }
    }

    pub(crate) fn from_code(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Vault),
            2 => Some(Self::Archive),
            3 => Some(Self::Journal),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Represents migration header.
pub struct MigrationHeader {
    /// Represents the artifact kind carried by this record case.
    pub artifact_kind: ArtifactKind,
    /// Represents the source native version carried by this record case.
    pub source_native_version: u32,
    /// Represents the migration schema version carried by this record case.
    pub migration_schema_version: u32,
    /// Represents the target native version carried by this record case.
    pub target_native_version: Option<u32>,
    /// Represents the operation id carried by this record case.
    pub operation_id: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Represents profile generation record.
pub struct ProfileGenerationRecord {
    /// Represents the index carried by this record case.
    pub index: u16,
    /// Represents the status carried by this record case.
    pub status: String,
    /// Represents the created at unix ms carried by this record case.
    pub created_at_unix_ms: u64,
    /// Represents the retired at unix ms carried by this record case.
    pub retired_at_unix_ms: Option<u64>,
    /// Represents the contact fingerprint carried by this record case.
    pub contact_fingerprint: Vec<u8>,
    /// Represents the private open key carried by this record case.
    pub private_open_key: SecretBytes,
    /// Represents the owner signing key carried by this record case.
    pub owner_signing_key: SecretBytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Represents profile record.
pub struct ProfileRecord {
    /// Represents the name carried by this record case.
    pub name: String,
    /// Represents the active generation carried by this record case.
    pub active_generation: u16,
    /// Represents the email carried by this record case.
    pub email: Option<String>,
    /// Represents the generations carried by this record case.
    pub generations: Vec<ProfileGenerationRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Represents form field record.
pub struct FormFieldRecord {
    /// Represents the id carried by this record case.
    pub id: String,
    /// Represents the label carried by this record case.
    pub label: String,
    /// Represents the kind carried by this record case.
    pub kind: String,
    /// Represents the required carried by this record case.
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Represents form definition record.
pub struct FormDefinitionRecord {
    /// Represents the type id carried by this record case.
    pub type_id: String,
    /// Represents the alias carried by this record case.
    pub alias: String,
    /// Represents the revision carried by this record case.
    pub revision: u32,
    /// Represents the name carried by this record case.
    pub name: String,
    /// Represents the description carried by this record case.
    pub description: String,
    /// Represents the fields carried by this record case.
    pub fields: Vec<FormFieldRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Represents form value record.
pub struct FormValueRecord {
    /// Represents the field id carried by this record case.
    pub field_id: String,
    /// Represents the captured label carried by this record case.
    pub captured_label: String,
    /// Represents the kind carried by this record case.
    pub kind: String,
    /// Represents the secret carried by this record case.
    pub secret: bool,
    /// Represents the value carried by this record case.
    pub value: SecretBytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Represents form record value.
pub struct FormRecordValue {
    /// Represents the path carried by this record case.
    pub path: String,
    /// Represents the name carried by this record case.
    pub name: String,
    /// Represents the type id carried by this record case.
    pub type_id: String,
    /// Represents the definition alias carried by this record case.
    pub definition_alias: String,
    /// Represents the definition revision carried by this record case.
    pub definition_revision: u32,
    /// Represents the values carried by this record case.
    pub values: Vec<FormValueRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
/// Represents vault record.
pub enum VaultRecord {
    /// Represents the start case.
    Start {
        /// Represents the structure version carried by this record case.
        structure_version: u32,
    },
    /// Represents the profile case.
    Profile(ProfileRecord),
    /// Represents the contact case.
    Contact {
        /// Represents the name carried by this record case.
        name: String,
        /// Represents the public key carried by this record case.
        public_key: Vec<u8>,
        /// Represents the signing public key carried by this record case.
        signing_public_key: Option<Vec<u8>>,
    },
    /// Represents the form definition case.
    FormDefinition(FormDefinitionRecord),
    /// Represents the known lockbox case.
    KnownLockbox {
        /// Represents the id carried by this record case.
        lockbox_id: [u8; 16],
        /// Represents the path carried by this record case.
        path: String,
        /// Represents the last seen unix ms carried by this record case.
        last_seen_unix_ms: u64,
    },
    /// Represents the access label case.
    AccessLabel {
        /// Represents the id carried by this record case.
        lockbox_id: [u8; 16],
        /// Represents the slot id carried by this record case.
        slot_id: u64,
        /// Represents the name carried by this record case.
        name: String,
        /// Represents the updated at unix ms carried by this record case.
        updated_at_unix_ms: u64,
    },
    /// Represents the lockbox password case.
    LockboxPassword {
        /// Represents the id carried by this record case.
        lockbox_id: [u8; 16],
        /// Represents the value carried by this record case.
        value: SecretBytes,
    },
    /// Represents the key directory case.
    KeyDirectory {
        /// Represents the id carried by this record case.
        lockbox_id: [u8; 16],
        /// Represents the bytes carried by this record case.
        bytes: SecretBytes,
    },
    /// Represents the end case.
    End {
        /// Represents the record count carried by this record case.
        record_count: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
/// Represents archive record.
pub enum ArchiveRecord {
    /// Represents the start case.
    Start {
        /// Represents the archive id carried by this record case.
        archive_id: [u8; 16],
        /// Represents the format version carried by this record case.
        format_version: u32,
        /// Represents the content key carried by this record case.
        content_key: SecretBytes,
        /// Represents the key directory carried by this record case.
        key_directory: SecretBytes,
    },
    /// Represents the file start case.
    FileStart {
        /// Represents the file id carried by this record case.
        file_id: u64,
        /// Represents the path carried by this record case.
        path: String,
        /// Represents the size carried by this record case.
        size: u64,
        /// Represents the permissions carried by this record case.
        permissions: Option<u32>,
    },
    /// Represents the file chunk case.
    FileChunk {
        /// Represents the file id carried by this record case.
        file_id: u64,
        /// Represents the offset carried by this record case.
        offset: u64,
        /// Represents the bytes carried by this record case.
        bytes: Vec<u8>,
    },
    /// Represents the file end case.
    FileEnd {
        /// Represents the file id carried by this record case.
        file_id: u64,
        /// Represents the size carried by this record case.
        size: u64,
        /// Value for sha256.
        sha256: [u8; 32],
    },
    /// Represents the symlink case.
    Symlink {
        /// Represents the path carried by this record case.
        path: String,
        /// Represents the target carried by this record case.
        target: String,
    },
    /// Represents the variable case.
    Variable {
        /// Represents the name carried by this record case.
        name: String,
        /// Represents the sensitivity carried by this record case.
        sensitivity: String,
        /// Represents the value carried by this record case.
        value: SecretBytes,
    },
    /// Represents the form definition case.
    FormDefinition(FormDefinitionRecord),
    /// Represents the form record case.
    FormRecord(FormRecordValue),
    /// Represents the end case.
    End {
        /// Represents the record count carried by this record case.
        record_count: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "artifact", content = "record", rename_all = "snake_case")]
/// Represents migration record.
pub enum MigrationRecord {
    /// Represents the vault case.
    Vault(VaultRecord),
    /// Represents the archive case.
    Archive(ArchiveRecord),
}
