use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Serialized byte field that clears its allocation when released.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

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
pub enum ArtifactKind {
    Vault,
    Archive,
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
pub struct MigrationHeader {
    pub artifact_kind: ArtifactKind,
    pub source_native_version: u32,
    pub migration_schema_version: u32,
    pub target_native_version: Option<u32>,
    pub operation_id: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileGenerationRecord {
    pub index: u16,
    pub status: String,
    pub created_at_unix_ms: u64,
    pub retired_at_unix_ms: Option<u64>,
    pub contact_fingerprint: Vec<u8>,
    pub private_open_key: SecretBytes,
    pub owner_signing_key: SecretBytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileRecord {
    pub name: String,
    pub active_generation: u16,
    pub email: Option<String>,
    pub generations: Vec<ProfileGenerationRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormFieldRecord {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormDefinitionRecord {
    pub type_id: String,
    pub alias: String,
    pub revision: u32,
    pub name: String,
    pub description: String,
    pub fields: Vec<FormFieldRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormValueRecord {
    pub field_id: String,
    pub captured_label: String,
    pub kind: String,
    pub secret: bool,
    pub value: SecretBytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormRecordValue {
    pub path: String,
    pub name: String,
    pub type_id: String,
    pub definition_alias: String,
    pub definition_revision: u32,
    pub values: Vec<FormValueRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum VaultRecord {
    Start {
        structure_version: u32,
    },
    Profile(ProfileRecord),
    Contact {
        name: String,
        public_key: Vec<u8>,
        signing_public_key: Option<Vec<u8>>,
    },
    FormDefinition(FormDefinitionRecord),
    KnownLockbox {
        lockbox_id: [u8; 16],
        path: String,
        last_seen_unix_ms: u64,
    },
    AccessLabel {
        lockbox_id: [u8; 16],
        slot_id: u64,
        name: String,
        updated_at_unix_ms: u64,
    },
    LockboxPassword {
        lockbox_id: [u8; 16],
        value: SecretBytes,
    },
    KeyDirectory {
        lockbox_id: [u8; 16],
        bytes: SecretBytes,
    },
    End {
        record_count: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ArchiveRecord {
    Start {
        archive_id: [u8; 16],
        format_version: u32,
        content_key: SecretBytes,
        key_directory: SecretBytes,
    },
    FileStart {
        file_id: u64,
        path: String,
        size: u64,
        permissions: Option<u32>,
    },
    FileChunk {
        file_id: u64,
        offset: u64,
        bytes: Vec<u8>,
    },
    FileEnd {
        file_id: u64,
        size: u64,
        sha256: [u8; 32],
    },
    Symlink {
        path: String,
        target: String,
    },
    Variable {
        name: String,
        sensitivity: String,
        value: SecretBytes,
    },
    FormDefinition(FormDefinitionRecord),
    FormRecord(FormRecordValue),
    End {
        record_count: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "artifact", content = "record", rename_all = "snake_case")]
pub enum MigrationRecord {
    Vault(VaultRecord),
    Archive(ArchiveRecord),
}
