use std::collections::BTreeMap;
use std::path::Path;

use super::Lockbox;
use crate::commit_auth::{commit_auth_digest, commit_auth_message, decode_commit_auth, CommitAuth};
use crate::compression::validate_compression_frame_lengths;
use crate::constants::DEFAULT_MAX_FILE_BYTES;
use crate::crypto::strong_checksum;
use crate::file_format::{
    decode_compression_frame_segment_payload_view, decode_symlink_payload, decode_toc_node,
};
use crate::form_btree::{decode_form_node_secure, FormEntryValue, FormNode};
use crate::key_directory::{best_key_directory, scan_key_directories};
use crate::lockbox_id::LockboxId;
use crate::node_kind::NodeKind;
use crate::page::PageObjectKind;
use crate::page_scanner::PageScanner;
use crate::signing::verify_commit_signatures;
use crate::toc_entry::TocEntry;
use crate::variable_btree::{decode_variable_node_secure, VariableNode, VariableValue};
use crate::OwnerSigningKeyPair;
use crate::{
    Error, FormDefinition, FormRecord, FormTypeId, LockboxEntry, LockboxPath, RecoveryReport,
    Result, VariableName,
};
use zeroize::Zeroizing;

#[path = "recovery_ranges.rs"]
mod ranges;

/// Scans damaged lockbox storage and reports or salvages recoverable content.
///
/// Recovery is intentionally separated from `Lockbox` because it operates on
/// storage bytes and can succeed when a normal open fails.
pub struct RecoveryScanner;

impl RecoveryScanner {
    /// Scan a lockbox file from disk and report recoverable entries.
    pub fn scan_path(path: &Path, key: impl AsRef<[u8]>) -> RecoveryReport {
        use crate::storage::{Storage, StorageBackend};
        match StorageBackend::file(path).and_then(|storage| storage.read_all()) {
            Ok(bytes) => Self::scan_bytes(bytes, key),
            Err(_) => RecoveryReport {
                intact_files: Vec::new(),
                intact_file_count: 0,
                partial_files: 0,
                partial_file_details: Vec::new(),
                corrupt_records: 1,
                toc_recovered: false,
                variables_recovered: false,
                variable_count: 0,
                forms_recovered: false,
                form_definition_count: 0,
                form_record_count: 0,
            },
        }
    }

    /// Scan lockbox storage bytes and report recoverable entries.
    pub fn scan_bytes(bytes: Vec<u8>, key: impl AsRef<[u8]>) -> RecoveryReport {
        recover_bytes(bytes, key)
    }

    /// Scan borrowed storage bytes without making another archive-sized copy.
    pub fn scan_slice(bytes: &[u8], key: impl AsRef<[u8]>) -> RecoveryReport {
        RecoverySession::new(bytes, key.as_ref()).report()
    }

    /// Salvage a damaged lockbox from storage bytes into a new opened lockbox.
    ///
    /// Returns `Error::CorruptRecord` if the selected publication cannot be
    /// authenticated, including when the supplied key is wrong. Authentication
    /// cannot distinguish an incorrect key from damaged encrypted metadata.
    /// Invalid selected metadata and storage/write failures are also returned;
    /// unauthenticated historical records never establish current membership.
    pub fn salvage_bytes(
        bytes: Vec<u8>,
        key: impl AsRef<[u8]>,
        signing_key: &OwnerSigningKeyPair,
    ) -> Result<Lockbox> {
        salvage_bytes(bytes, key, signing_key)
    }

    /// Salvage a damaged lockbox using a content key held in secure memory.
    pub fn salvage_bytes_with_secret_key(
        bytes: Vec<u8>,
        key: &crate::SecretVec,
        signing_key: &OwnerSigningKeyPair,
    ) -> Result<Lockbox> {
        let key_bytes = Zeroizing::new(key.with_bytes(|key| key.to_vec())?);
        salvage_bytes(bytes, &*key_bytes, signing_key)
    }
}

fn recover_bytes(bytes: Vec<u8>, key: impl AsRef<[u8]>) -> RecoveryReport {
    RecoverySession::new(&bytes, key.as_ref()).report()
}

fn salvage_bytes(
    bytes: Vec<u8>,
    key: impl AsRef<[u8]>,
    signing_key: &OwnerSigningKeyPair,
) -> Result<Lockbox> {
    RecoverySession::new(&bytes, key.as_ref()).salvage(signing_key)
}

struct RecoverySession<'a> {
    bytes: &'a [u8],
    key: &'a [u8],
    lockbox_id: LockboxId,
    scanner: PageScanner<'a>,
}

impl<'a> RecoverySession<'a> {
    fn new(bytes: &'a [u8], key: &'a [u8]) -> Self {
        let lockbox_id = lockbox_id_from_bytes_unchecked(bytes);
        Self {
            bytes,
            key,
            lockbox_id,
            scanner: PageScanner::new(bytes, lockbox_id, key),
        }
    }

    fn report(&self) -> RecoveryReport {
        let mut toc_entries = BTreeMap::new();
        let mut toc_recovered = false;
        let mut metadata = RecoveredMetadata::default();
        let mut corrupt_records = 0;
        if let Some((root, _)) = header_commit_root_for_recovery(&self.scanner, self.bytes) {
            metadata = recover_metadata_from_commit_root(&self.scanner, &root);
            match decode_toc_btree_from_offset(
                &self.scanner,
                root.toc_root_offset,
                root.toc_root_commitment,
                0,
            ) {
                Ok(entries) => {
                    toc_entries = entries;
                    toc_recovered = true;
                }
                Err(_) => corrupt_records += 1,
            }
        } else {
            corrupt_records += 1;
        }
        // An authenticated empty TOC is authoritative. Scanned historical
        // records cannot establish membership in the selected publication.
        let mut intact_files = Vec::new();
        let mut intact_file_count = 0;
        let mut partial_files = 0;
        let mut partial_file_details = Vec::new();
        for entry in toc_entries.values().filter(|entry| !entry.deleted) {
            let result = match entry.node_kind {
                NodeKind::File => read_page_file_bytes(&self.scanner, entry).map(|bytes| {
                    let _bytes = crate::page_buffer::ZeroizingBytes::new(bytes);
                }),
                NodeKind::Symlink => recover_symlink_target(&self.scanner, entry).map(|_| ()),
                NodeKind::Directory => Ok(()),
            };
            if result.is_ok() {
                intact_file_count += 1;
            } else {
                partial_files += 1;
                corrupt_records += 1;
                let reason = match result.unwrap_err() {
                    Error::InvalidKey => "payload authentication failed",
                    Error::Truncated => "file data is truncated",
                    Error::SecurityLimitExceeded(_) => "file exceeds recovery safety limits",
                    _ => "file metadata or content failed integrity validation",
                };
                partial_file_details.push((entry.path.to_string(), reason.to_owned()));
                continue;
            }
            intact_files.push(LockboxEntry {
                path: entry.path.clone(),
                kind: entry.entry_kind(),
                len: entry.len,
                permissions: entry.permissions,
            });
        }

        RecoveryReport {
            intact_files,
            intact_file_count,
            partial_files,
            partial_file_details,
            corrupt_records,
            toc_recovered,
            variables_recovered: metadata.variables.is_some(),
            variable_count: metadata.variables.as_ref().map_or(0, BTreeMap::len),
            forms_recovered: metadata.forms.is_some(),
            form_definition_count: metadata
                .forms
                .as_ref()
                .map_or(0, |forms| latest_form_definition_count(&forms.definitions)),
            form_record_count: metadata
                .forms
                .as_ref()
                .map_or(0, |forms| forms.records.len()),
        }
    }

    fn salvage(&self, signing_key: &OwnerSigningKeyPair) -> Result<Lockbox> {
        let (root, _) = header_commit_root_for_recovery(&self.scanner, self.bytes)
            .ok_or(Error::CorruptRecord)?;
        let metadata = recover_metadata_from_commit_root(&self.scanner, &root);
        let latest_paths = decode_toc_btree_from_offset(
            &self.scanner,
            root.toc_root_offset,
            root.toc_root_commitment,
            0,
        )?;
        let mut recovered = Lockbox::create_with_secret_key_and_options(
            crate::SecretVec::try_from_slice(self.key)?,
            self.lockbox_id,
            crate::LockboxOptions::default(),
        );
        recovered.set_owner_signing_key(signing_key.try_clone()?);
        recovered.set_creation_format(self.scanner.format_mode);
        for entry in latest_paths
            .values()
            .filter(|entry| !entry.deleted && entry.node_kind == NodeKind::Directory)
        {
            recovered.create_dir(&entry.path, true)?;
            recovered.set_permissions(&entry.path, entry.permissions)?;
        }
        for entry in latest_paths
            .values()
            .filter(|entry| !entry.deleted && entry.node_kind != NodeKind::Directory)
        {
            match entry.node_kind {
                NodeKind::File => {
                    if let Ok(bytes) = read_page_file_bytes(&self.scanner, entry) {
                        let bytes = crate::page_buffer::ZeroizingBytes::new(bytes);
                        recovered.create_parent_dirs_for(&entry.path)?;
                        recovered.add_file_with_permissions(
                            &entry.path,
                            &bytes,
                            entry.permissions,
                            false,
                        )?;
                    }
                }
                NodeKind::Symlink => {
                    if let Ok(target) = recover_symlink_target(&self.scanner, entry) {
                        recovered.create_parent_dirs_for(&entry.path)?;
                        recovered.add_symlink(&entry.path, &target, false)?;
                        recovered.set_permissions(&entry.path, entry.permissions)?;
                    }
                }
                NodeKind::Directory => {}
            }
        }
        if let Some(variables) = metadata.variables {
            for (name, value) in variables {
                recovered.set_variable_value(name, value)?;
            }
        }
        if let Some(forms) = metadata.forms {
            for (key, definition) in forms.definitions {
                recovered.set_form_definition_value(key, definition)?;
            }
            for (path, record) in forms.records {
                recovered.create_parent_dirs_for(&path)?;
                recovered.set_form_record_value(path, record)?;
            }
        }
        recovered.commit()?;
        Ok(recovered)
    }
}

fn header_commit_root_for_recovery(
    scanner: &PageScanner<'_>,
    _bytes: &[u8],
) -> Option<(crate::commit_root::CommitRoot, bool)> {
    let header = scanner.authenticated_header().ok()?;
    if header.commit_auth_offset == 0 || header.commit_root_offset == 0 || header.sequence == 0 {
        return None;
    }
    let auth = decode_commit_auth(
        &scanner
            .commit_auth_payload_at(header.commit_auth_offset)
            .ok()?,
    )
    .ok()?;
    if auth.commit_root_offset != header.commit_root_offset {
        return None;
    }
    let root =
        valid_commit_root_from_auth_chain(scanner, header.commit_auth_offset, header.lockbox_id)?;
    if root.sequence != header.sequence {
        return None;
    }
    let public_header_root = (header.commit_root_offset > 0)
        .then(|| scanner.commit_root_at(header.commit_root_offset).ok())
        .flatten()
        .is_some_and(|public_root| public_root == root);
    Some((root, public_header_root))
}

fn valid_commit_root_from_auth_chain(
    scanner: &PageScanner<'_>,
    mut auth_offset: u64,
    lockbox_id: LockboxId,
) -> Option<crate::commit_root::CommitRoot> {
    let mut expected_digest = None;
    let mut selected = None;
    let mut newer_signatures: Option<Vec<crate::commit_auth::CommitSignature>> = None;
    while auth_offset != 0 {
        let auth_payload = scanner.commit_auth_payload_at(auth_offset).ok()?;
        let digest = commit_auth_digest(&auth_payload);
        if expected_digest.is_some_and(|expected| digest != expected) {
            return None;
        }
        let auth = decode_commit_auth(&auth_payload).ok()?;
        if auth.lockbox_id != lockbox_id {
            return None;
        }
        let message = commit_auth_message(&auth).ok()?;
        if auth.flags != u64::from(scanner.format_mode.0) {
            return None;
        }
        if scanner.format_mode.signed() {
            verify_commit_signatures(&message, &auth.signatures).ok()?;
            if newer_signatures.as_ref().is_some_and(|newer| {
                !crate::signing::commit_signature_keys_match(&auth.signatures, newer)
            }) {
                return None;
            }
        } else if !auth.signatures.is_empty() {
            return None;
        }
        if selected.is_none() {
            // A previous publication is historical state, never a substitute
            // for a damaged selected root. Validate the selected root first.
            selected = Some(verified_commit_root_from_auth(scanner, &auth)?);
        }
        if auth.previous_auth_offset >= auth_offset {
            return None;
        }
        expected_digest = Some(auth.previous_auth_digest);
        newer_signatures = Some(auth.signatures);
        auth_offset = auth.previous_auth_offset;
    }
    selected
}

fn verified_commit_root_from_auth(
    scanner: &PageScanner<'_>,
    auth: &CommitAuth,
) -> Option<crate::commit_root::CommitRoot> {
    let root_payload = scanner
        .commit_root_payload_at(auth.commit_root_offset)
        .ok()?;
    if strong_checksum(&root_payload) != auth.commit_root_digest {
        return None;
    }
    let root = crate::commit_root::decode_commit_root(&root_payload).ok()?;
    (root.sequence == auth.sequence && root.flags == u64::from(scanner.format_mode.0))
        .then_some(root)
}

#[derive(Default)]
struct RecoveredMetadata {
    variables: Option<BTreeMap<VariableName, VariableValue>>,
    forms: Option<RecoveredForms>,
}

struct RecoveredForms {
    definitions: BTreeMap<String, FormDefinition>,
    records: BTreeMap<LockboxPath, FormRecord>,
}

fn recover_metadata_from_commit_root(
    scanner: &PageScanner<'_>,
    commit_root: &crate::commit_root::CommitRoot,
) -> RecoveredMetadata {
    RecoveredMetadata {
        variables: recover_variables_from_root(
            scanner,
            commit_root.variable_root_offset,
            commit_root.variable_root_commitment,
        )
        .ok(),
        forms: recover_forms_from_root(
            scanner,
            commit_root.form_root_offset,
            commit_root.form_root_commitment,
        )
        .ok(),
    }
}

fn recover_variables_from_root(
    scanner: &PageScanner<'_>,
    root_offset: u64,
    commitment: crate::file_format::page_commitment::PageCommitment,
) -> Result<BTreeMap<VariableName, VariableValue>> {
    if root_offset == 0 {
        return Ok(BTreeMap::new());
    }
    let mut variables = BTreeMap::new();
    decode_variable_node_into(scanner, root_offset, commitment, &mut variables, 0)?;
    Ok(variables)
}

fn decode_variable_node_into(
    scanner: &PageScanner<'_>,
    offset: u64,
    commitment: crate::file_format::page_commitment::PageCommitment,
    variables: &mut BTreeMap<VariableName, VariableValue>,
    depth: usize,
) -> Result<()> {
    if depth > 8 {
        return Err(Error::CorruptRecord);
    }
    let payload = scanner.authenticated_secure_object_payload_at(
        offset,
        &[
            PageObjectKind::VariableLeaf,
            PageObjectKind::VariableInternal,
        ],
        commitment,
    )?;
    match decode_variable_node_secure(&payload)? {
        VariableNode::Leaf(entries) => {
            for entry in entries {
                variables.insert(VariableName::new(entry.name)?, entry.value);
            }
        }
        VariableNode::Internal(children) => {
            drop(payload);
            for child in children {
                decode_variable_node_into(
                    scanner,
                    child.offset,
                    child.commitment,
                    variables,
                    depth + 1,
                )?;
            }
        }
    }
    Ok(())
}

fn recover_forms_from_root(
    scanner: &PageScanner<'_>,
    root_offset: u64,
    commitment: crate::file_format::page_commitment::PageCommitment,
) -> Result<RecoveredForms> {
    if root_offset == 0 {
        return Ok(RecoveredForms {
            definitions: BTreeMap::new(),
            records: BTreeMap::new(),
        });
    }
    let mut definitions = BTreeMap::new();
    let mut records = BTreeMap::new();
    decode_form_node_into(
        scanner,
        root_offset,
        commitment,
        &mut definitions,
        &mut records,
        0,
    )?;
    Ok(RecoveredForms {
        definitions,
        records,
    })
}

fn decode_form_node_into(
    scanner: &PageScanner<'_>,
    offset: u64,
    commitment: crate::file_format::page_commitment::PageCommitment,
    definitions: &mut BTreeMap<String, FormDefinition>,
    records: &mut BTreeMap<LockboxPath, FormRecord>,
    depth: usize,
) -> Result<()> {
    if depth > 8 {
        return Err(Error::CorruptRecord);
    }
    let payload = scanner.authenticated_secure_object_payload_at(
        offset,
        &[PageObjectKind::FormLeaf, PageObjectKind::FormInternal],
        commitment,
    )?;
    match decode_form_node_secure(&payload)? {
        FormNode::Leaf(entries) => {
            for entry in entries {
                match entry.value {
                    FormEntryValue::Definition(definition) => {
                        definitions.insert(entry.key, definition);
                    }
                    FormEntryValue::Record(record) => {
                        records.insert(record.path.clone(), record);
                    }
                }
            }
        }
        FormNode::Internal(children) => {
            drop(payload);
            for child in children {
                decode_form_node_into(
                    scanner,
                    child.offset,
                    child.commitment,
                    definitions,
                    records,
                    depth + 1,
                )?;
            }
        }
    }
    Ok(())
}

fn latest_form_definition_count(definitions: &BTreeMap<String, FormDefinition>) -> usize {
    let mut latest = BTreeMap::<FormTypeId, u32>::new();
    for definition in definitions.values() {
        let revision = latest.entry(definition.type_id.clone()).or_default();
        *revision = (*revision).max(definition.revision);
    }
    latest.len()
}

fn decode_toc_btree_from_offset(
    scanner: &PageScanner<'_>,
    offset: u64,
    commitment: crate::file_format::page_commitment::PageCommitment,
    depth: usize,
) -> Result<BTreeMap<LockboxPath, TocEntry>> {
    if depth > 8 {
        return Err(crate::Error::CorruptRecord);
    }
    let payload = scanner.toc_node_payload_at(offset, commitment)?;

    let mut toc_entries = BTreeMap::new();
    match decode_toc_node(&payload)? {
        crate::file_format::TocNode::Leaf(entries) => {
            for entry in entries {
                toc_entries.insert(entry.path.clone(), entry);
            }
        }
        crate::file_format::TocNode::Internal(children) => {
            for child in children {
                let child_toc_entries = decode_toc_btree_from_offset(
                    scanner,
                    child.offset,
                    child.commitment,
                    depth + 1,
                )?;
                toc_entries.extend(child_toc_entries);
            }
        }
    }
    Ok(toc_entries)
}

fn read_page_file_bytes(
    scanner: &PageScanner<'_>,
    entry: &crate::toc_entry::TocEntry,
) -> Result<Vec<u8>> {
    let mut chunks = entry.chunks.clone();
    chunks.sort_by_key(|chunk| chunk.file_offset);
    let expected_len = entry.len;
    if expected_len > DEFAULT_MAX_FILE_BYTES {
        return Err(Error::SecurityLimitExceeded(
            "recovered file exceeds default safety limit".to_string(),
        ));
    }
    let capacity = usize::try_from(expected_len).map_err(|_| Error::CorruptRecord)?;
    let mut out = crate::page_buffer::ZeroizingBytes::new(Vec::with_capacity(capacity));
    for chunk in chunks {
        validate_compression_frame_lengths(chunk.compression_frame_len, chunk.compressed_len)?;
        if chunk.compressed_len > crate::constants::DEFAULT_MAX_PAGE_LOGICAL_BYTES as u64 {
            return Err(Error::SecurityLimitExceeded(
                "compressed compression-frame exceeds safety limit".to_string(),
            ));
        }
        if chunk.file_offset < out.len() as u64 || chunk.file_offset > expected_len {
            return Err(Error::CorruptRecord);
        }
        out.resize(
            usize::try_from(chunk.file_offset).map_err(|_| Error::CorruptRecord)?,
            0,
        );
        #[cfg(any(test, feature = "native-block-layout"))]
        if chunk.block_frame.is_some() {
            let decoded = crate::page_buffer::ZeroizingBytes::new(
                scanner.read_native_chunk(expected_len, &chunk)?,
            );
            out.extend_from_slice(&decoded);
            if out.len() as u64 > expected_len {
                return Err(Error::CorruptRecord);
            }
            continue;
        }
        let compressed_len =
            usize::try_from(chunk.compressed_len).map_err(|_| Error::CorruptRecord)?;
        let mut stored = Zeroizing::new(vec![0u8; compressed_len]);
        for segment in &chunk.segments {
            let payload = scanner.file_segment_payload_at(
                segment.page_offset,
                segment.object_id,
                segment.page_len,
            )?;
            let decoded = decode_compression_frame_segment_payload_view(&payload)?;
            let manifest_slice_missing = decoded.manifest.as_ref().is_some_and(|manifest| {
                manifest
                    .slice_for(
                        &chunk.stored_path,
                        chunk.file_offset,
                        chunk.compression_frame_offset,
                        chunk.len,
                    )
                    .filter(|slice| slice.total_len == 0 || slice.total_len == expected_len)
                    .is_none()
            });
            if decoded.compression_frame_id != chunk.compression_frame_id
                || decoded.compression_frame_len != chunk.compression_frame_len
                || decoded.compressed_len != chunk.compressed_len
                || decoded.compression != chunk.compression
                || decoded.compression_frame_digest != chunk.compression_frame_digest
                || manifest_slice_missing
                || decoded.segment_offset != segment.segment_offset
                || decoded.data.len() as u64 != segment.segment_len
            {
                return Err(Error::CorruptRecord);
            }
            let start =
                usize::try_from(segment.segment_offset).map_err(|_| Error::CorruptRecord)?;
            let end = start
                .checked_add(decoded.data.len())
                .ok_or(Error::CorruptRecord)?;
            if end > stored.len() {
                return Err(Error::CorruptRecord);
            }
            stored[start..end].copy_from_slice(decoded.data);
        }
        if strong_checksum(stored.as_slice()) != chunk.compression_frame_digest {
            return Err(Error::CorruptRecord);
        }
        let decoded = Zeroizing::new(crate::compression::decode_compression_frame(
            chunk.compression,
            stored.as_slice(),
            chunk.compression_frame_len,
        )?);
        let start =
            usize::try_from(chunk.compression_frame_offset).map_err(|_| Error::CorruptRecord)?;
        let len = usize::try_from(chunk.len).map_err(|_| Error::CorruptRecord)?;
        let end = start.checked_add(len).ok_or(Error::CorruptRecord)?;
        if end > decoded.len() {
            return Err(Error::CorruptRecord);
        }
        out.extend_from_slice(&decoded[start..end]);
        if out.len() as u64 > expected_len {
            return Err(Error::CorruptRecord);
        }
    }
    if out.len() as u64 > expected_len {
        return Err(Error::CorruptRecord);
    }
    out.resize(capacity, 0);
    Ok(std::mem::take(&mut *out))
}

fn recover_symlink_target(scanner: &PageScanner<'_>, entry: &TocEntry) -> Result<LockboxPath> {
    if entry.record_offset == 0 || entry.record_object_id == 0 {
        return Err(Error::CorruptRecord);
    }
    let payload = scanner.authenticated_entry_object_payload(entry)?;
    let (path, target) = decode_symlink_payload(&payload)?;
    if path != entry.path {
        return Err(Error::CorruptRecord);
    }
    Ok(target)
}

fn lockbox_id_from_bytes_unchecked(bytes: &[u8]) -> LockboxId {
    if let Ok(header) = crate::file_format::current_header::read_header_for_scan(bytes) {
        return header.lockbox_id;
    }
    if let Some(directory) = best_key_directory(scan_key_directories(bytes, None)) {
        return directory.lockbox_id;
    }
    if let Some(id) = bytes
        .get(56..72)
        .and_then(|bytes| bytes.try_into().ok())
        .map(LockboxId::from_bytes)
    {
        return id;
    }
    LockboxId::from_bytes([0; 16])
}

#[cfg(test)]
mod committed_membership_tests {
    use super::*;
    use crate::{CacheLimit, LockboxOptions};

    #[test]
    fn recovery_and_symlink_lifecycle_follow_current_membership_with_and_without_cache() {
        let key = b"committed recovery lifecycle";
        let signer = OwnerSigningKeyPair::generate().unwrap();
        let path = |name: &str| LockboxPath::new(name).unwrap();
        for cache_limit in [CacheLimit::Bytes(0), CacheLimit::Bytes(1024 * 1024)] {
            let options = LockboxOptions {
                cache_limit,
                ..LockboxOptions::default()
            };
            let mut archive = Lockbox::create(key);
            archive.set_owner_signing_key(signer.try_clone().unwrap());
            archive.add_file(&path("/a"), b"first", false).unwrap();
            archive.add_file(&path("/b"), b"second", false).unwrap();
            archive
                .add_symlink(&path("/x"), &path("/a"), false)
                .unwrap();
            archive
                .add_symlink(&path("/y"), &path("/a"), false)
                .unwrap();
            archive.commit().unwrap();
            let mut archive =
                Lockbox::open_bytes_with_key_options(archive.to_bytes(), key, options).unwrap();
            archive.set_owner_signing_key(signer.try_clone().unwrap());
            archive.add_file(&path("/b"), b"replacement", true).unwrap();
            archive.delete(&path("/a")).unwrap();
            archive.add_symlink(&path("/x"), &path("/b"), true).unwrap();
            archive.delete(&path("/y")).unwrap();
            archive.commit().unwrap();
            let bytes = archive.to_bytes();
            let reopened =
                Lockbox::open_bytes_with_key_options(bytes.clone(), key, options).unwrap();
            assert_eq!(reopened.get_file(&path("/b")).unwrap(), b"replacement");
            assert_eq!(
                reopened.get_symlink_target(&path("/x")).unwrap(),
                path("/b")
            );
            assert!(reopened.get_file(&path("/a")).is_err());
            assert!(reopened.get_symlink_target(&path("/y")).is_err());
            let report = RecoveryScanner::scan_bytes(bytes.clone(), key);
            assert!(report.toc_recovered);
            assert_eq!(report.partial_files, 0);
            assert_eq!(report.intact_file_count, 2);
            let recovered = RecoveryScanner::salvage_bytes(bytes, key, &signer).unwrap();
            assert_eq!(recovered.get_file(&path("/b")).unwrap(), b"replacement");
            assert_eq!(
                recovered.get_symlink_target(&path("/x")).unwrap(),
                path("/b")
            );
            archive.delete(&path("/x")).unwrap();
            archive.delete(&path("/b")).unwrap();
            archive.commit().unwrap();
            let empty = archive.to_bytes();
            let report = RecoveryScanner::scan_bytes(empty.clone(), key);
            assert!(report.toc_recovered);
            assert_eq!(report.intact_file_count, 0);
            assert!(report.intact_files.is_empty());
            let recovered = RecoveryScanner::salvage_bytes(empty, key, &signer).unwrap();
            assert!(recovered.get_file(&path("/b")).is_err());
            assert!(recovered.get_symlink_target(&path("/x")).is_err());
        }
    }
}

#[cfg(test)]
#[path = "recovery_commitment_tests.rs"]
mod commitment_tests;
