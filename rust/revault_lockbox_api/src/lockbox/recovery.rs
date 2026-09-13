use std::collections::BTreeMap;
use std::path::Path;

use super::Lockbox;
use crate::commit_auth::{commit_auth_digest, commit_auth_message, decode_commit_auth, CommitAuth};
use crate::compression::validate_compression_frame_lengths;
use crate::constants::DEFAULT_MAX_FILE_BYTES;
use crate::crypto::strong_checksum;
use crate::file_format::{
    decode_compression_frame_segment_payload_view, decode_index_records, decode_symlink_payload,
    decode_toc_node, read_header,
};
use crate::form_btree::{decode_form_node_secure, FormEntryValue, FormNode};
use crate::key_directory::{best_key_directory, scan_key_directories};
use crate::lockbox_id::LockboxId;
use crate::node_kind::NodeKind;
use crate::page::PageObjectKind;
use crate::page_scanner::PageScanner;
use crate::record::{DecodedRecord, RecordKind};
use crate::signing::verify_commit_signatures;
use crate::toc_entry::TocEntry;
use crate::variable_btree::{decode_variable_node_secure, VariableNode, VariableValue};
use crate::OwnerSigningKeyPair;
use crate::{
    Error, FormDefinition, FormRecord, FormTypeId, LockboxEntry, LockboxPath, RecoveryReport,
    Result, VariableName,
};
use zeroize::Zeroizing;

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

    /// Salvage a damaged lockbox from storage bytes into a new opened lockbox.
    ///
    /// Returns `Error::InvalidKey` if the supplied key cannot authenticate any
    /// recoverable records, or `Error::CorruptRecord`/storage errors if the
    /// recovered content cannot be written into the new lockbox.
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
    let key = key.as_ref().to_vec();
    RecoverySession::new(&bytes, &key).report()
}

fn salvage_bytes(
    bytes: Vec<u8>,
    key: impl AsRef<[u8]>,
    signing_key: &OwnerSigningKeyPair,
) -> Result<Lockbox> {
    let key_bytes = key.as_ref().to_vec();
    RecoverySession::new(&bytes, &key_bytes).salvage(signing_key)
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
        #[cfg(any(test, feature = "native-block-layout"))]
        let mut native_toc_available = false;
        let mut metadata = RecoveredMetadata::default();

        if let Some((commit_root, public_header_root)) =
            header_commit_root_for_recovery(&self.scanner, self.bytes)
        {
            metadata = recover_metadata_from_commit_root(&self.scanner, &commit_root);
            if let Ok(decoded) =
                decode_toc_btree_from_offset(&self.scanner, commit_root.toc_root_offset, 0)
            {
                toc_entries = decoded;
                toc_recovered = public_header_root;
                #[cfg(any(test, feature = "native-block-layout"))]
                {
                    native_toc_available = true;
                }
            }
        }

        let scan = self.scanner.scan_records();
        let scanned_segments = collect_scanned_file_segments(&scan.records);
        let mut corrupt_records = scan.corrupt_records;
        if toc_entries.is_empty() {
            for record in &scan.records {
                match decode_index_records(record) {
                    Ok(entries) => {
                        for entry in entries {
                            apply_scanned_entry(&mut toc_entries, entry);
                        }
                    }
                    Err(_) => corrupt_records += 1,
                }
            }
        }
        #[cfg(any(test, feature = "native-block-layout"))]
        if !native_toc_available {
            attach_native_manifest_entries(&mut toc_entries, &scan.native_pages);
        }
        attach_scanned_file_segments(&mut toc_entries, &scanned_segments);

        #[cfg(any(test, feature = "native-block-layout"))]
        let native_signed_snapshot = self.native_signed_snapshot(&toc_entries);

        let mut intact_files = Vec::new();
        let mut intact_file_count = 0;
        let mut partial_files = 0;
        for entry in toc_entries.values().filter(|entry| !entry.deleted) {
            let native_authorized = {
                #[cfg(any(test, feature = "native-block-layout"))]
                {
                    self.native_entry_authorized(entry, native_signed_snapshot.as_ref())
                }
                #[cfg(not(any(test, feature = "native-block-layout")))]
                {
                    true
                }
            };
            let complete = native_authorized
                && match entry.node_kind {
                    NodeKind::File => read_page_file_bytes(&self.scanner, entry)
                        .map(crate::page_buffer::ZeroizingBytes::new)
                        .is_ok(),
                    NodeKind::Symlink => recover_symlink_target(&self.scanner, entry).is_ok(),
                    NodeKind::Directory => true,
                };
            if complete {
                intact_file_count += 1;
            } else {
                partial_files += 1;
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
        let metadata = header_commit_root_for_recovery(&self.scanner, self.bytes)
            .map(|(commit_root, _)| recover_metadata_from_commit_root(&self.scanner, &commit_root))
            .unwrap_or_default();
        let scan = self.scanner.scan_records();
        let scanned_segments = collect_scanned_file_segments(&scan.records);
        let mut recovered = Lockbox::create_with_secret_key_and_options(
            crate::SecretVec::try_from_slice(self.key)?,
            self.lockbox_id,
            crate::LockboxOptions::default(),
        );
        recovered.set_owner_signing_key(signing_key.try_clone()?);
        recovered.set_creation_format(self.scanner.format_mode);
        let mut latest_paths = BTreeMap::new();
        for record in &scan.records {
            if let Ok(entries) = decode_index_records(record) {
                for entry in entries {
                    apply_scanned_entry(&mut latest_paths, entry);
                }
            }
        }
        #[cfg(any(test, feature = "native-block-layout"))]
        if let Some(entries) =
            header_commit_root_for_recovery(&self.scanner, self.bytes).and_then(|(root, _)| {
                decode_toc_btree_from_offset(&self.scanner, root.toc_root_offset, 0).ok()
            })
        {
            latest_paths.extend(
                entries.into_iter().filter(|(_, entry)| {
                    entry.chunks.iter().any(|chunk| chunk.block_frame.is_some())
                }),
            );
        } else {
            attach_native_manifest_entries(&mut latest_paths, &scan.native_pages);
        }
        attach_scanned_file_segments(&mut latest_paths, &scanned_segments);
        #[cfg(any(test, feature = "native-block-layout"))]
        let native_signed_snapshot = self.native_signed_snapshot(&latest_paths);

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
            #[cfg(any(test, feature = "native-block-layout"))]
            if entry.node_kind == NodeKind::File
                && entry.chunks.iter().any(|chunk| chunk.block_frame.is_some())
            {
                if self.native_entry_authorized(entry, native_signed_snapshot.as_ref()) {
                    if let Ok(file_bytes) = read_page_file_bytes(&self.scanner, entry) {
                        let file_bytes = crate::page_buffer::ZeroizingBytes::new(file_bytes);
                        recovered.create_parent_dirs_for(&entry.path)?;
                        recovered.add_file_with_permissions(
                            &entry.path,
                            &file_bytes,
                            entry.permissions,
                            false,
                        )?;
                    }
                }
                continue;
            }
            let record = if entry.record_object_id == 0 {
                self.scanner.record_at(entry.record_offset)
            } else {
                self.scanner
                    .record_object_at(entry.record_offset, entry.record_object_id)
            };
            if let Ok(record) = record {
                match record.header.kind {
                    RecordKind::FilePage => {
                        if let Ok(file_bytes) = read_page_file_bytes(&self.scanner, entry) {
                            let file_bytes = crate::page_buffer::ZeroizingBytes::new(file_bytes);
                            recovered.create_parent_dirs_for(&entry.path)?;
                            recovered.add_file_with_permissions(
                                &entry.path,
                                &file_bytes,
                                entry.permissions,
                                false,
                            )?;
                        }
                    }
                    RecordKind::Symlink => {
                        if let Ok((path, target)) = decode_symlink_payload(&record.payload) {
                            recovered.create_parent_dirs_for(&path)?;
                            recovered.add_symlink(&path, &target, false)?;
                        }
                    }
                    _ => {}
                }
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

    #[cfg(any(test, feature = "native-block-layout"))]
    fn native_signed_snapshot(&self, entries: &BTreeMap<LockboxPath, TocEntry>) -> Option<Lockbox> {
        if !self.scanner.format_mode.signed()
            || !entries
                .values()
                .any(|entry| entry.chunks.iter().any(|chunk| chunk.block_frame.is_some()))
        {
            return None;
        }
        let key = crate::SecretVec::try_from_slice(self.key).ok()?;
        let mut snapshot = self.bytes.to_vec();
        if read_header(&snapshot).is_err() {
            let repaired = crate::file_format::current_header::recover_header_magic(&snapshot)?;
            snapshot[..repaired.len()].copy_from_slice(&repaired);
        }
        Lockbox::open_storage_with_secret_key_mode(
            crate::storage::StorageBackend::memory(snapshot),
            key,
            crate::LockboxOptions::default(),
            false,
        )
        .ok()
    }

    #[cfg(any(test, feature = "native-block-layout"))]
    fn native_entry_authorized(&self, entry: &TocEntry, snapshot: Option<&Lockbox>) -> bool {
        !self.scanner.format_mode.signed()
            || !entry.chunks.iter().any(|chunk| chunk.block_frame.is_some())
            || snapshot
                .and_then(|snapshot| snapshot.toc_entries.get(&entry.path))
                .is_some_and(|trusted| {
                    crate::toc_codec::TocEncoder::new([trusted]).encode()
                        == crate::toc_codec::TocEncoder::new([entry]).encode()
                })
    }
}

fn header_commit_root_for_recovery(
    scanner: &PageScanner<'_>,
    bytes: &[u8],
) -> Option<(crate::commit_root::CommitRoot, bool)> {
    let header = read_header(bytes).ok()?;
    if header.commit_auth_offset == 0 {
        return (header.commit_root_offset > 0)
            .then(|| scanner.commit_root_at(header.commit_root_offset).ok())
            .flatten()
            .map(|root| (root, true));
    }
    let root =
        valid_commit_root_from_auth_chain(scanner, header.commit_auth_offset, header.lockbox_id)?;
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
        } else if !auth.signatures.is_empty() {
            return None;
        }
        if let Some(root) = verified_commit_root_from_auth(scanner, &auth) {
            return Some(root);
        }
        expected_digest = Some(auth.previous_auth_digest);
        auth_offset = auth.previous_auth_offset;
    }
    None
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
        variables: recover_variables_from_root(scanner, commit_root.variable_root_offset).ok(),
        forms: recover_forms_from_root(scanner, commit_root.form_root_offset).ok(),
    }
}

fn recover_variables_from_root(
    scanner: &PageScanner<'_>,
    root_offset: u64,
) -> Result<BTreeMap<VariableName, VariableValue>> {
    if root_offset == 0 {
        return Ok(BTreeMap::new());
    }
    let mut variables = BTreeMap::new();
    decode_variable_node_into(scanner, root_offset, &mut variables, 0)?;
    Ok(variables)
}

fn decode_variable_node_into(
    scanner: &PageScanner<'_>,
    offset: u64,
    variables: &mut BTreeMap<VariableName, VariableValue>,
    depth: usize,
) -> Result<()> {
    if depth > 8 {
        return Err(Error::CorruptRecord);
    }
    let payload = scanner.secure_object_payload_at(
        offset,
        &[
            PageObjectKind::VariableLeaf,
            PageObjectKind::VariableInternal,
        ],
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
                decode_variable_node_into(scanner, child.offset, variables, depth + 1)?;
            }
        }
    }
    Ok(())
}

fn recover_forms_from_root(scanner: &PageScanner<'_>, root_offset: u64) -> Result<RecoveredForms> {
    if root_offset == 0 {
        return Ok(RecoveredForms {
            definitions: BTreeMap::new(),
            records: BTreeMap::new(),
        });
    }
    let mut definitions = BTreeMap::new();
    let mut records = BTreeMap::new();
    decode_form_node_into(scanner, root_offset, &mut definitions, &mut records, 0)?;
    Ok(RecoveredForms {
        definitions,
        records,
    })
}

fn decode_form_node_into(
    scanner: &PageScanner<'_>,
    offset: u64,
    definitions: &mut BTreeMap<String, FormDefinition>,
    records: &mut BTreeMap<LockboxPath, FormRecord>,
    depth: usize,
) -> Result<()> {
    if depth > 8 {
        return Err(Error::CorruptRecord);
    }
    let payload = scanner.secure_object_payload_at(
        offset,
        &[PageObjectKind::FormLeaf, PageObjectKind::FormInternal],
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
                decode_form_node_into(scanner, child.offset, definitions, records, depth + 1)?;
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
    depth: usize,
) -> Result<BTreeMap<LockboxPath, TocEntry>> {
    if depth > 8 {
        return Err(crate::Error::CorruptRecord);
    }
    let payload = scanner.toc_node_payload_at(offset)?;

    let mut toc_entries = BTreeMap::new();
    match decode_toc_node(&payload)? {
        crate::file_format::TocNode::Leaf(entries) => {
            for entry in entries {
                toc_entries.insert(entry.path.clone(), entry);
            }
        }
        crate::file_format::TocNode::Internal(children) => {
            for child in children {
                let child_toc_entries =
                    decode_toc_btree_from_offset(scanner, child.offset, depth + 1)?;
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
    let expected_len = if entry.len == 0 && chunks.iter().any(|chunk| chunk.len > 0) {
        chunks.iter().try_fold(0u64, |max_end, chunk| {
            let end = chunk
                .file_offset
                .checked_add(chunk.len)
                .ok_or(Error::CorruptRecord)?;
            Ok::<u64, Error>(max_end.max(end))
        })?
    } else {
        entry.len
    };
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
        if chunk.file_offset != out.len() as u64 {
            return Err(Error::CorruptRecord);
        }
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
            let record = scanner.record_object_at(segment.page_offset, segment.object_id)?;
            let decoded = decode_compression_frame_segment_payload_view(&record.payload)?;
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
    if out.len() as u64 != expected_len {
        return Err(Error::CorruptRecord);
    }
    Ok(std::mem::take(&mut *out))
}

fn recover_symlink_target(scanner: &PageScanner<'_>, entry: &TocEntry) -> Result<LockboxPath> {
    if entry.record_offset == 0 || entry.record_object_id == 0 {
        return Err(Error::CorruptRecord);
    }
    let record = scanner.record_object_at(entry.record_offset, entry.record_object_id)?;
    if record.header.kind != RecordKind::Symlink {
        return Err(Error::CorruptRecord);
    }
    let (path, target) = decode_symlink_payload(&record.payload)?;
    if path != entry.path {
        return Err(Error::CorruptRecord);
    }
    Ok(target)
}

#[derive(Debug, Clone)]
struct ScannedCompressionFrameSegment {
    compression_frame_id: u64,
    compression_frame_len: u64,
    compressed_len: u64,
    compression: u8,
    compression_frame_digest: [u8; 32],
    segment: crate::file_chunk::CompressionFrameSegment,
}

fn collect_scanned_file_segments(
    records: &[DecodedRecord],
) -> BTreeMap<u64, Vec<ScannedCompressionFrameSegment>> {
    let mut segments: BTreeMap<u64, Vec<ScannedCompressionFrameSegment>> = BTreeMap::new();
    for record in records {
        if record.header.kind != RecordKind::FilePage {
            continue;
        }
        let Ok(decoded) = decode_compression_frame_segment_payload_view(&record.payload) else {
            continue;
        };
        segments
            .entry(decoded.compression_frame_id)
            .or_default()
            .push(ScannedCompressionFrameSegment {
                compression_frame_id: decoded.compression_frame_id,
                compression_frame_len: decoded.compression_frame_len,
                compressed_len: decoded.compressed_len,
                compression: decoded.compression,
                compression_frame_digest: decoded.compression_frame_digest,
                segment: crate::file_chunk::CompressionFrameSegment {
                    page_offset: record.offset,
                    page_len: record.header.total_len,
                    object_id: record.object_id,
                    segment_offset: decoded.segment_offset,
                    segment_len: decoded.data.len() as u64,
                },
            });
    }
    segments
}

fn attach_scanned_file_segments(
    toc_entries: &mut BTreeMap<LockboxPath, TocEntry>,
    scanned_segments: &BTreeMap<u64, Vec<ScannedCompressionFrameSegment>>,
) {
    for entry in toc_entries.values_mut() {
        if entry.deleted || entry.node_kind != NodeKind::File {
            continue;
        }
        for chunk in &mut entry.chunks {
            // Native references bind one physical page through their descriptor.
            // Legacy scan candidates must never extend that typed reference.
            #[cfg(any(test, feature = "native-block-layout"))]
            if chunk.block_frame.is_some() {
                continue;
            }
            let Some(frame_segments) = scanned_segments.get(&chunk.compression_frame_id) else {
                continue;
            };
            for scanned in frame_segments {
                if scanned.compression_frame_id != chunk.compression_frame_id
                    || scanned.compression_frame_len != chunk.compression_frame_len
                    || scanned.compressed_len != chunk.compressed_len
                    || scanned.compression != chunk.compression
                    || scanned.compression_frame_digest != chunk.compression_frame_digest
                {
                    continue;
                }
                if !chunk.segments.iter().any(|existing| {
                    existing.page_offset == scanned.segment.page_offset
                        && existing.object_id == scanned.segment.object_id
                        && existing.segment_offset == scanned.segment.segment_offset
                }) {
                    chunk.segments.push(scanned.segment.clone());
                }
            }
            chunk.segments.sort_by_key(|segment| segment.segment_offset);
        }
    }
}

/// Manifest-only reconstruction is a recovery candidate, not evidence of a
/// committed file or owner authorization. Conflicting versions are omitted
/// instead of splicing unrelated generations into an apparently intact file.
#[cfg(any(test, feature = "native-block-layout"))]
fn attach_native_manifest_entries(
    entries: &mut BTreeMap<LockboxPath, TocEntry>,
    pages: &[crate::file_format::indexed_frame::block_page::ScannedBlockPage],
) {
    use crate::file_chunk::{BlockFrameReference, CompressionFrameSegment, FileChunk};
    let mut candidates: BTreeMap<LockboxPath, TocEntry> = BTreeMap::new();
    let mut conflicts = std::collections::BTreeSet::new();
    for page in pages {
        let reference = std::sync::Arc::new(BlockFrameReference {
            descriptor: page.descriptor.clone(),
            sequence: page.sequence,
        });
        for slice in &page.manifest.slices {
            if entries.contains_key(&slice.path) || conflicts.contains(&slice.path) {
                continue;
            }
            let chunk = FileChunk {
                block_frame: Some(reference.clone()),
                stored_path: slice.path.clone(),
                file_offset: slice.file_offset,
                len: slice.len,
                compression_frame_offset: slice.compression_frame_offset,
                compression_frame_len: page.descriptor.logical_len,
                compressed_len: page.descriptor.stored_len,
                compression: page.descriptor.compression,
                compression_frame_id: page.descriptor.frame_id,
                compression_frame_digest: page.descriptor.index_commitment,
                segments: vec![CompressionFrameSegment {
                    page_offset: page.offset,
                    page_len: page.physical_len as u64,
                    object_id: page.page_id,
                    segment_offset: 0,
                    segment_len: page.descriptor.stored_len,
                }],
            };
            let entry = candidates
                .entry(slice.path.clone())
                .or_insert_with(|| TocEntry {
                    path: slice.path.clone(),
                    len: slice.total_len,
                    record_offset: page.offset,
                    record_len: page.physical_len as u64,
                    record_object_id: page.page_id,
                    deleted: false,
                    node_kind: NodeKind::File,
                    permissions: slice.permissions,
                    chunks: Vec::new(),
                });
            if entry.len != slice.total_len
                || entry.permissions != slice.permissions
                || entry.chunks.iter().any(|old| {
                    old.file_offset == chunk.file_offset
                        || (old.file_offset < chunk.file_offset.saturating_add(chunk.len)
                            && chunk.file_offset < old.file_offset.saturating_add(old.len))
                })
            {
                conflicts.insert(slice.path.clone());
                continue;
            }
            entry.chunks.push(chunk);
        }
    }
    entries.extend(
        candidates
            .into_iter()
            .filter(|(path, _)| !conflicts.contains(path)),
    );
}

fn apply_scanned_entry(toc_entries: &mut BTreeMap<LockboxPath, TocEntry>, mut entry: TocEntry) {
    if entry.deleted || entry.node_kind != NodeKind::File {
        toc_entries.insert(entry.path.clone(), entry);
        return;
    }
    if entry.len == 0 {
        entry.len = recovered_len_from_chunks(&entry).unwrap_or(0);
    }
    toc_entries
        .entry(entry.path.clone())
        .and_modify(|existing| {
            if existing.deleted || existing.node_kind != NodeKind::File {
                *existing = entry.clone();
                return;
            }
            existing.len = existing.len.max(entry.len);
            existing.record_offset = existing.record_offset.min(entry.record_offset);
            existing.record_len = existing.record_len.max(entry.record_len);
            for chunk in &entry.chunks {
                merge_recovered_chunk(existing, chunk.clone());
            }
        })
        .or_insert(entry);
}

fn merge_recovered_chunk(entry: &mut TocEntry, chunk: crate::file_chunk::FileChunk) {
    if let Some(existing) = entry.chunks.iter_mut().find(|existing| {
        existing.compression_frame_id == chunk.compression_frame_id
            && existing.file_offset == chunk.file_offset
            && existing.compression_frame_offset == chunk.compression_frame_offset
            && existing.len == chunk.len
    }) {
        for segment in chunk.segments {
            if !existing.segments.iter().any(|existing_segment| {
                existing_segment.page_offset == segment.page_offset
                    && existing_segment.object_id == segment.object_id
                    && existing_segment.segment_offset == segment.segment_offset
            }) {
                existing.segments.push(segment);
            }
        }
    } else {
        entry.chunks.push(chunk);
    }
}

fn recovered_len_from_chunks(entry: &TocEntry) -> Result<u64> {
    entry.chunks.iter().try_fold(0u64, |max_end, chunk| {
        let end = chunk
            .file_offset
            .checked_add(chunk.len)
            .ok_or(Error::CorruptRecord)?;
        Ok(max_end.max(end))
    })
}

fn lockbox_id_from_bytes_unchecked(bytes: &[u8]) -> LockboxId {
    if let Ok(header) = read_header(bytes) {
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
