#![deny(missing_docs)]

//! Encrypted, authenticated, streaming artifacts used to migrate reVault
//! native vault and archive formats without retaining historical readers in
//! the current storage crates.

mod archive;
mod journal;
mod vault;

pub use archive::{
    archive_owner_fingerprint, export_archive, import_archive, upgrade_archive_artifact,
    verify_archive_artifact, verify_imported_archive,
};
pub use journal::{MigrationJournal, MigrationStage};
pub use revault_migration_format::{
    ArchiveRecord, ArtifactKind, ArtifactReader, ArtifactWriter, FormDefinitionRecord,
    FormFieldRecord, FormRecordValue, FormValueRecord, MigrationError, MigrationHeader,
    MigrationPassphrase, MigrationRecord, ProfileGenerationRecord, ProfileRecord, Result,
    SecretBytes, VaultRecord, JSON_FRAME_TYPE, MAX_FRAME_BYTES, RAW_FRAME_TYPE,
};
pub use vault::{export_vault, import_vault, upgrade_vault_artifact, verify_vault_artifact};
// Retain source compatibility for callers of the original schema-2 API names.
pub use vault::{export_vault as export_vault_v2, import_vault as import_vault_v2};
