//! Versioned, encrypted, authenticated, streaming migration artifacts shared
//! by historical reVault exporters and current importers.

mod artifact;
mod model;

pub use artifact::{
    ArtifactReader, ArtifactWriter, MigrationError, MigrationPassphrase, Result, JSON_FRAME_TYPE,
    MAX_FRAME_BYTES, RAW_FRAME_TYPE,
};
pub use model::{
    ArchiveRecord, ArtifactKind, FormDefinitionRecord, FormFieldRecord, FormRecordValue,
    FormValueRecord, MigrationHeader, MigrationRecord, ProfileGenerationRecord, ProfileRecord,
    SecretBytes, VaultRecord,
};
