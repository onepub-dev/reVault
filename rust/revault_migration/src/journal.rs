use crate::{
    ArtifactKind, ArtifactReader, ArtifactWriter, MigrationError, MigrationHeader,
    MigrationPassphrase, Result,
};
use revault_lockbox_api::SecretVec;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStage {
    Export,
    Upgrade,
    Import,
    Validate,
    Replace,
    Complete,
}

#[derive(Debug)]
pub struct MigrationJournal {
    pub operation_id: [u8; 16],
    pub artifact_kind: ArtifactKind,
    pub source_path: PathBuf,
    pub source_format_version: u32,
    pub source_fingerprint: [u8; 32],
    pub target_format_version: u32,
    pub current_stage: MigrationStage,
    pub temporary_paths: Vec<PathBuf>,
    pub exporter_version: Option<String>,
    /// Random key used for the migration artifacts. It is held in the
    /// lockbox API secure heap. The encrypted journal stores it in a separate
    /// encrypted raw frame rather than in the JSON representation.
    pub artifact_key: SecretVec,
}

#[derive(Serialize, Deserialize)]
struct MigrationJournalWire {
    operation_id: [u8; 16],
    artifact_kind: ArtifactKind,
    source_path: PathBuf,
    source_format_version: u32,
    source_fingerprint: [u8; 32],
    target_format_version: u32,
    current_stage: MigrationStage,
    temporary_paths: Vec<PathBuf>,
    exporter_version: Option<String>,
}

impl PartialEq for MigrationJournal {
    fn eq(&self, other: &Self) -> bool {
        self.operation_id == other.operation_id
            && self.artifact_kind == other.artifact_kind
            && self.source_path == other.source_path
            && self.source_format_version == other.source_format_version
            && self.source_fingerprint == other.source_fingerprint
            && self.target_format_version == other.target_format_version
            && self.current_stage == other.current_stage
            && self.temporary_paths == other.temporary_paths
            && self.exporter_version == other.exporter_version
            && self.artifact_key == other.artifact_key
    }
}

impl Eq for MigrationJournal {}

impl MigrationJournal {
    /// Atomically saves this journal as a one-record encrypted migration
    /// artifact. The temporary file is synchronized before replacement.
    pub fn save<P: MigrationPassphrase + ?Sized>(&self, path: &Path, passphrase: &P) -> Result<()> {
        let parent = path.parent().ok_or_else(|| {
            MigrationError::Io("journal path has no parent directory".to_string())
        })?;
        fs::create_dir_all(parent).map_err(|err| MigrationError::Io(err.to_string()))?;
        let temporary = path.with_extension("migration-state.tmp");
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)
            .map_err(|err| MigrationError::Io(err.to_string()))?;
        let header = MigrationHeader {
            artifact_kind: ArtifactKind::Journal,
            source_native_version: self.source_format_version,
            migration_schema_version: 1,
            target_native_version: Some(self.target_format_version),
            operation_id: self.operation_id,
        };
        let mut writer = ArtifactWriter::new_with_passphrase(file, header, passphrase)?;
        writer.write_json(&MigrationJournalWire {
            operation_id: self.operation_id,
            artifact_kind: self.artifact_kind,
            source_path: self.source_path.clone(),
            source_format_version: self.source_format_version,
            source_fingerprint: self.source_fingerprint,
            target_format_version: self.target_format_version,
            current_stage: self.current_stage,
            temporary_paths: self.temporary_paths.clone(),
            exporter_version: self.exporter_version.clone(),
        })?;
        writer.write_secure_raw(&self.artifact_key)?;
        let file = writer.finish()?;
        file.sync_all()
            .map_err(|err| MigrationError::Io(err.to_string()))?;
        fs::rename(&temporary, path).map_err(|err| MigrationError::Io(err.to_string()))
    }

    pub fn load<P: MigrationPassphrase + ?Sized>(path: &Path, passphrase: &P) -> Result<Self> {
        let file = File::open(path).map_err(|err| MigrationError::Io(err.to_string()))?;
        let mut reader = ArtifactReader::new_with_passphrase(BufReader::new(file), passphrase)?;
        if reader.header().artifact_kind != ArtifactKind::Journal {
            return Err(MigrationError::InvalidHeader(
                "artifact is not a migration journal".to_string(),
            ));
        }
        let wire: MigrationJournalWire = reader.next_json()?.ok_or(MigrationError::Incomplete)?;
        let (frame_type, artifact_key) = reader
            .next_secure_frame()?
            .ok_or(MigrationError::Incomplete)?;
        if frame_type != crate::RAW_FRAME_TYPE {
            return Err(MigrationError::CorruptFrame(
                "journal key is not a raw record".to_string(),
            ));
        }
        if reader.next_frame()?.is_some() || !reader.is_complete() {
            return Err(MigrationError::CorruptFrame(
                "journal contains unexpected records".to_string(),
            ));
        }
        Ok(Self {
            operation_id: wire.operation_id,
            artifact_kind: wire.artifact_kind,
            source_path: wire.source_path,
            source_format_version: wire.source_format_version,
            source_fingerprint: wire.source_fingerprint,
            target_format_version: wire.target_format_version,
            current_stage: wire.current_stage,
            temporary_paths: wire.temporary_paths,
            exporter_version: wire.exporter_version,
            artifact_key,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypted_journal_round_trips_atomically() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("archive.migration-state");
        let value = MigrationJournal {
            operation_id: [3; 16],
            artifact_kind: ArtifactKind::Archive,
            source_path: PathBuf::from("secrets.lbox"),
            source_format_version: 1,
            source_fingerprint: [4; 32],
            target_format_version: 2,
            current_stage: MigrationStage::Export,
            temporary_paths: vec![PathBuf::from("temporary")],
            exporter_version: Some("0.0.1".to_string()),
            artifact_key: SecretVec::try_from_slice(b"artifact key").unwrap(),
        };
        value.save(&path, b"journal password").unwrap();
        assert_eq!(
            MigrationJournal::load(&path, b"journal password").unwrap(),
            value
        );
    }

    #[test]
    fn encrypted_journal_round_trips_with_secure_passphrase() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("vault.migration-state");
        let passphrase =
            revault_lockbox_api::SecretString::try_from_slice(b"journal password").unwrap();
        let value = MigrationJournal {
            operation_id: [8; 16],
            artifact_kind: ArtifactKind::Vault,
            source_path: PathBuf::from("vault"),
            source_format_version: 1,
            source_fingerprint: [9; 32],
            target_format_version: 2,
            current_stage: MigrationStage::Export,
            temporary_paths: vec![PathBuf::from("temporary")],
            exporter_version: None,
            artifact_key: SecretVec::try_from_slice(b"artifact key").unwrap(),
        };

        value.save(&path, &passphrase).unwrap();
        assert_eq!(MigrationJournal::load(&path, &passphrase).unwrap(), value);
    }
}
