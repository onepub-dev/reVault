//! Explicit fragment recovery, separate from frozen whole-archive salvage.
use super::*;

impl RecoveryScanner {
    #[cfg(feature = "test-support")]
    pub fn stale_reference_fixture(
        password: &[u8],
        signing_record: &[u8],
        partial: bool,
        mirror_root: Option<&str>,
    ) -> Result<Vec<u8>> {
        use crate::archive_reader::SecretVec;
        let signer = OwnerSigningKeyPair::from_private_key_record(SecretVec::try_from_slice(
            signing_record,
        )?)?;
        let mut archive = Lockbox::create(password);
        archive.set_owner_signing_key(signer);
        archive.add_password(&crate::archive_reader::SecretString::try_from_slice(
            password,
        )?)?;
        let path = LockboxPath::new(format!(
            "{}/restored.bin",
            mirror_root.unwrap_or("").trim_end_matches('/')
        ))?;
        archive.create_parent_dirs_for(&path)?;
        archive.add_file_with_permissions(&path, &vec![42; 3 * 1024 * 1024], 0o640, false)?;
        let lock_path = LockboxPath::new(format!(
            "{}/.home.lbx.lock",
            mirror_root.unwrap_or("").trim_end_matches('/')
        ))?;
        if mirror_root.is_some() {
            archive.add_file_with_permissions(&lock_path, b"", 0o600, false)?;
        }
        archive.set_variable(&VariableName::new("KEPT")?, "preserved metadata")?;
        if let Some(root) = mirror_root {
            archive.create_mirror_project(
                crate::archive_reader::MirrorProject {
                    name: "home".to_owned(),
                    source: "/synthetic/source".to_owned(),
                    destination: LockboxPath::new(root)?,
                    includes: Vec::new(),
                    excludes: Vec::new(),
                    missing_file_policy: crate::archive_reader::MirrorMissingFilePolicy::Remove,
                    strict: false,
                    host_identity: None,
                },
                true,
            )?;
        }
        archive.commit()?;
        let entry = archive.toc_entries.get_mut(&path).unwrap();
        if partial {
            entry.chunks[1].compression_frame_digest[0] ^= 1;
        } else {
            entry.chunks[0].segments[0].page_offset = 7;
            entry.record_offset = 7;
            entry.record_object_id = 0;
        }
        archive.mark_toc_dirty(&path);
        if mirror_root.is_some() {
            let entry = archive.toc_entries.get_mut(&lock_path).unwrap();
            entry.record_offset = 7;
            entry.record_object_id = 0;
            archive.mark_toc_dirty(&lock_path);
        }
        archive.commit()?;
        archive.try_to_bytes()
    }

    /// Supplement legacy salvage while preserving its metadata and credentials.
    pub fn salvage_with_file_fragments(
        bytes: &[u8],
        key: &[u8],
        signer: &OwnerSigningKeyPair,
        emit: impl FnMut(u64, u64, &[u8]) -> Result<()>,
    ) -> Result<(Lockbox, serde_json::Value)> {
        Self::salvage_with_file_fragments_and_progress(bytes, key, signer, emit, |_, _| {})
    }

    pub fn salvage_with_file_fragments_and_progress(
        bytes: &[u8],
        key: &[u8],
        signer: &OwnerSigningKeyPair,
        emit: impl FnMut(u64, u64, &[u8]) -> Result<()>,
        progress: impl FnMut(u64, u64),
    ) -> Result<(Lockbox, serde_json::Value)> {
        let mut recovered = RecoverySession::new(bytes, key).salvage(signer)?;
        let plan = Self::plan_file_fragments_and_progress(
            bytes,
            key,
            Some(&mut recovered),
            emit,
            progress,
        )?;
        recovered.commit()?;
        Ok((recovered, plan))
    }

    pub fn plan_file_fragments(
        bytes: &[u8],
        key: &[u8],
        recovered: Option<&mut Lockbox>,
        emit: impl FnMut(u64, u64, &[u8]) -> Result<()>,
    ) -> Result<serde_json::Value> {
        Self::plan_file_fragments_and_progress(bytes, key, recovered, emit, |_, _| {})
    }

    pub fn plan_file_fragments_and_progress(
        bytes: &[u8],
        key: &[u8],
        mut recovered: Option<&mut Lockbox>,
        mut emit: impl FnMut(u64, u64, &[u8]) -> Result<()>,
        mut progress: impl FnMut(u64, u64),
    ) -> Result<serde_json::Value> {
        let session = RecoverySession::new(bytes, key);
        let unavailable = || serde_json::json!({"recovery_directory_protocol":1,"source_format":2,"membership":"unavailable","files":[],"reason":"Authenticated file membership unavailable; legacy archive salvage only"});
        let Some((root, latest)) = header_commit_root_for_recovery(&session.scanner, bytes) else {
            return Ok(unavailable());
        };
        let Ok(mut entries) =
            decode_toc_btree_from_offset(&session.scanner, root.toc_root_offset, 0)
        else {
            return Ok(unavailable());
        };
        let scan = session.scanner.scan_records();
        attach_scanned_file_segments(&mut entries, &collect_scanned_file_segments(&scan.records));
        drop(scan);
        let mut partial = Vec::new();
        let mut complete_files = 0u64;
        let total = entries
            .values()
            .filter(|entry| !entry.deleted && entry.node_kind == NodeKind::File)
            .count() as u64;
        for (index, entry) in entries
            .values()
            .filter(|entry| !entry.deleted && entry.node_kind == NodeKind::File)
            .enumerate()
        {
            progress(index as u64, total);
            let mut pieces = Vec::new();
            let selected = BTreeMap::from([(entry.path.clone(), entry.clone())]);
            let mut plan = recover_entries(&session, &selected, latest, &mut |_, offset, data| {
                pieces.push((offset, Zeroizing::new(data.to_vec())));
                Ok(())
            })?;
            let mut file = plan["files"].as_array_mut().unwrap().remove(0);
            if file["status"] == "complete" {
                complete_files += 1;
                if let Some(archive) = recovered.as_deref_mut() {
                    let mut data = Zeroizing::new(Vec::new());
                    for (_, piece) in pieces {
                        data.extend_from_slice(&piece);
                    }
                    let matches = archive
                        .get_file(&entry.path)
                        .map(Zeroizing::new)
                        .is_ok_and(|existing| *existing == *data);
                    mutate_recovered_path(archive, &entry.path, |archive| {
                        if !matches {
                            archive.create_parent_dirs_for(&entry.path)?;
                            let replace = archive.exists(&entry.path);
                            archive.add_file_with_permissions(
                                &entry.path,
                                &data,
                                entry.permissions,
                                replace,
                            )?;
                        }
                        archive.set_permissions(&entry.path, entry.permissions)
                    })?;
                }
            } else {
                if let Some(archive) = recovered.as_deref_mut() {
                    if archive.exists(&entry.path) {
                        mutate_recovered_path(archive, &entry.path, |archive| {
                            archive.delete(&entry.path)
                        })?;
                    }
                }
                let id = partial.len() as u64 + 1;
                file["id"] = serde_json::json!(id);
                file["permissions"] = serde_json::json!(entry.permissions);
                for (offset, data) in pieces {
                    emit(id, offset, &data)?;
                }
                partial.push(file);
            }
        }
        progress(total, total);
        Ok(
            serde_json::json!({"recovery_directory_protocol":1,"source_format":2,"membership":"verified_commit_toc","latest_commit":latest,"complete_files":complete_files,"files":partial}),
        )
    }

    pub fn recover_ranges(
        bytes: &[u8],
        key: &[u8],
        mut emit: impl FnMut(u64, u64, &[u8]) -> Result<()>,
    ) -> Result<serde_json::Value> {
        let session = RecoverySession::new(bytes, key);
        let membership_error = || {
            Error::InvalidInput("Cannot establish authenticated file membership for directory recovery. Retain the original; --output recovered.lbox can attempt archive salvage, which may omit damaged entries.".to_owned())
        };
        let (root, latest) = header_commit_root_for_recovery(&session.scanner, bytes)
            .ok_or_else(membership_error)?;
        let mut entries = decode_toc_btree_from_offset(&session.scanner, root.toc_root_offset, 0)
            .map_err(|_| membership_error())?;
        // Alternatives are candidates only. A complete reconstructed frame must
        // still match the digest selected by the authenticated current TOC.
        let scan = session.scanner.scan_records();
        attach_scanned_file_segments(&mut entries, &collect_scanned_file_segments(&scan.records));
        drop(scan);
        recover_entries(&session, &entries, latest, &mut emit)
    }
}

fn mutate_recovered_path(
    archive: &mut Lockbox,
    path: &LockboxPath,
    operation: impl FnOnce(&mut Lockbox) -> Result<()>,
) -> Result<()> {
    let project = archive.list_mirror_projects()?.into_iter().find(|project| {
        path == &project.destination || path.is_descendant_of(&project.destination)
    });
    let result = match project {
        Some(project) => {
            archive.with_mirror_project_mutation(&project.name, |archive, _| operation(archive))
        }
        None => operation(archive),
    };
    result.map_err(|error| {
        Error::InvalidOperation(format!("recovery mutation failed for {path}: {error}"))
    })
}

fn recover_entries(
    session: &RecoverySession<'_>,
    entries: &BTreeMap<LockboxPath, TocEntry>,
    latest: bool,
    emit: &mut impl FnMut(u64, u64, &[u8]) -> Result<()>,
) -> Result<serde_json::Value> {
    let mut files = Vec::new();
    for entry in entries
        .values()
        .filter(|entry| !entry.deleted && entry.node_kind == NodeKind::File)
    {
        let file_id = files.len() as u64 + 1;
        let mut chunks = entry.chunks.clone();
        chunks.sort_by_key(|chunk| chunk.file_offset);
        let size = entry.len;
        let mut cursor = 0u64;
        let valid_index = chunks.iter().all(|chunk| {
            let valid = (chunk.len > 0 || (size == 0 && chunk.file_offset == 0))
                && chunk.file_offset >= cursor
                && chunk
                    .file_offset
                    .checked_add(chunk.len)
                    .is_some_and(|end| end <= size);
            cursor = chunk.file_offset.saturating_add(chunk.len);
            valid
        });
        let mut ranges = Vec::new();
        let mut missing = Vec::new();
        if !valid_index || size > DEFAULT_MAX_FILE_BYTES {
            missing.push(serde_json::json!({"offset":0,"length":size,"reason":"invalid overlapping or out-of-range file index, or recovery size limit"}));
        } else {
            cursor = 0;
            for chunk in &chunks {
                if chunk.file_offset > cursor {
                    missing.push(serde_json::json!({"offset":cursor,"length":chunk.file_offset-cursor,"reason":"no indexed data covers this range"}));
                }
                match recover_chunk(&session.scanner, entry, chunk) {
                        Ok(data) => {
                            for (index, piece) in data.chunks(4 * 1024 * 1024).enumerate() {
                                let offset = chunk.file_offset + (index * 4 * 1024 * 1024) as u64;
                                emit(file_id, offset, piece)?;
                                ranges.push(serde_json::json!({"offset":offset,"length":piece.len()}));
                            }
                        }
                        Err(_) => missing.push(serde_json::json!({"offset":chunk.file_offset,"length":chunk.len,"reason":"no complete authenticated compression frame could be reconstructed"})),
                    }
                cursor = chunk.file_offset + chunk.len;
            }
            if cursor < size {
                missing.push(serde_json::json!({"offset":cursor,"length":size-cursor,"reason":"no indexed data covers this range"}));
            }
        }
        let recovered_bytes: u64 = ranges
            .iter()
            .map(|range| range["length"].as_u64().unwrap())
            .sum();
        let status = if missing.is_empty() {
            "complete"
        } else if recovered_bytes == 0 {
            "unrecoverable"
        } else {
            "partial"
        };
        files.push(serde_json::json!({"id":file_id,"path":entry.path.to_string(),"size":size,"status":status,"recovered_bytes":recovered_bytes,"ranges":ranges,"missing_ranges":missing}));
    }
    Ok(
        serde_json::json!({"recovery_directory_protocol":1,"source_format":2,"membership":"verified_commit_toc","latest_commit":latest,"files":files}),
    )
}

fn recover_chunk(
    scanner: &PageScanner<'_>,
    entry: &TocEntry,
    chunk: &crate::archive_reader::file_chunk::FileChunk,
) -> Result<Zeroizing<Vec<u8>>> {
    validate_compression_frame_lengths(chunk.compression_frame_len, chunk.compressed_len)?;
    if chunk.compressed_len
        > crate::archive_reader::constants::DEFAULT_MAX_PAGE_LOGICAL_BYTES as u64
    {
        return Err(Error::CorruptRecord);
    }
    let mut stored = Zeroizing::new(vec![
        0;
        usize::try_from(chunk.compressed_len)
            .map_err(|_| Error::CorruptRecord)?
    ]);
    let mut covered: Vec<(usize, usize)> = Vec::new();
    for segment in &chunk.segments {
        let Ok(record) = scanner.record_object_at(segment.page_offset, segment.object_id) else {
            continue;
        };
        let Ok(decoded) = decode_compression_frame_segment_payload_view(&record.payload) else {
            continue;
        };
        if decoded.compression_frame_id != chunk.compression_frame_id
            || decoded.compression_frame_len != chunk.compression_frame_len
            || decoded.compressed_len != chunk.compressed_len
            || decoded.compression != chunk.compression
            || decoded.compression_frame_digest != chunk.compression_frame_digest
            || decoded.segment_offset != segment.segment_offset
            || decoded.data.len() as u64 != segment.segment_len
            || decoded.manifest.as_ref().is_some_and(|manifest| {
                manifest
                    .slice_for(
                        &chunk.stored_path,
                        chunk.file_offset,
                        chunk.compression_frame_offset,
                        chunk.len,
                    )
                    .filter(|slice| slice.total_len == 0 || slice.total_len == entry.len)
                    .is_none()
            })
        {
            continue;
        }
        let start = usize::try_from(segment.segment_offset).map_err(|_| Error::CorruptRecord)?;
        let end = start
            .checked_add(decoded.data.len())
            .ok_or(Error::CorruptRecord)?;
        if end > stored.len() {
            continue;
        }
        for &(a, b) in &covered {
            let low = a.max(start);
            let high = b.min(end);
            if low < high && stored[low..high] != decoded.data[low - start..high - start] {
                return Err(Error::CorruptRecord);
            }
        }
        stored[start..end].copy_from_slice(decoded.data);
        covered.push((start, end));
    }
    covered.sort_unstable();
    let mut end = 0;
    for (start, next) in covered {
        if start > end {
            return Err(Error::CorruptRecord);
        }
        end = end.max(next);
    }
    if end != stored.len() || strong_checksum(&stored) != chunk.compression_frame_digest {
        return Err(Error::CorruptRecord);
    }
    let decoded = Zeroizing::new(
        crate::archive_reader::compression::decode_compression_frame(
            chunk.compression,
            &stored,
            chunk.compression_frame_len,
        )?,
    );
    let start =
        usize::try_from(chunk.compression_frame_offset).map_err(|_| Error::CorruptRecord)?;
    let end = start
        .checked_add(usize::try_from(chunk.len).map_err(|_| Error::CorruptRecord)?)
        .ok_or(Error::CorruptRecord)?;
    Ok(Zeroizing::new(
        decoded
            .get(start..end)
            .ok_or(Error::CorruptRecord)?
            .to_vec(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_recovery_uses_verified_alternatives_and_rejects_missing_or_untrusted_frames() {
        let key = b"range fixture key";
        let mut archive = Lockbox::create(key);
        let path = LockboxPath::new("/sample").unwrap();
        archive.add_file(&path, b"verified payload", false).unwrap();
        archive
            .add_file(&LockboxPath::new("/empty").unwrap(), b"", false)
            .unwrap();
        archive.commit().unwrap();
        let bytes = archive.to_bytes();
        let session = RecoverySession::new(&bytes, key);
        let (root, _) = header_commit_root_for_recovery(&session.scanner, &bytes).unwrap();
        let entries =
            decode_toc_btree_from_offset(&session.scanner, root.toc_root_offset, 0).unwrap();
        let mut entry = entries[&path].clone();
        let mut stale = entry.chunks[0].segments[0].clone();
        stale.page_offset = bytes.len() as u64 + 4096;
        entry.chunks[0].segments.insert(0, stale);
        assert!(read_page_file_bytes(&session.scanner, &entry).is_err());
        assert_eq!(
            &*recover_chunk(&session.scanner, &entry, &entry.chunks[0]).unwrap(),
            b"verified payload"
        );
        let selected = BTreeMap::from([(path.clone(), entry.clone())]);
        let mut rescued = Vec::new();
        let rescued_plan = recover_entries(&session, &selected, true, &mut |_, _, data| {
            rescued.extend_from_slice(data);
            Ok(())
        })
        .unwrap();
        assert_eq!(rescued, b"verified payload");
        assert_eq!(rescued_plan["files"][0]["status"], "complete");
        assert_eq!(
            rescued_plan["files"][0]["missing_ranges"],
            serde_json::json!([])
        );
        let mut missing = entry.chunks[0].clone();
        missing.segments.clear();
        assert!(recover_chunk(&session.scanner, &entry, &missing).is_err());
        let mut untrusted = entry.chunks[0].clone();
        untrusted.compression_frame_digest[0] ^= 1;
        assert!(recover_chunk(&session.scanner, &entry, &untrusted).is_err());
        let mut recovered = Vec::new();
        let plan = RecoveryScanner::recover_ranges(&bytes, key, |_, _, data| {
            recovered.extend_from_slice(data);
            Ok(())
        })
        .unwrap();
        assert_eq!(recovered, b"verified payload");
        assert!(plan["files"]
            .as_array()
            .unwrap()
            .iter()
            .all(|file| file["status"] == "complete"));
        assert!(plan["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|file| file["path"] == "/empty" && file["size"] == 0));
    }

    #[test]
    fn range_recovery_keeps_verified_chunks_around_gaps_and_refuses_overlaps() {
        let key = b"range gap fixture";
        let mut archive = Lockbox::create(key);
        let path = LockboxPath::new("/three-frames").unwrap();
        let payload = vec![37; 6 * 1024 * 1024];
        archive.add_file(&path, &payload, false).unwrap();
        archive.commit().unwrap();
        let bytes = archive.to_bytes();
        let session = RecoverySession::new(&bytes, key);
        let (root, _) = header_commit_root_for_recovery(&session.scanner, &bytes).unwrap();
        let entries =
            decode_toc_btree_from_offset(&session.scanner, root.toc_root_offset, 0).unwrap();
        let original = entries[&path].clone();
        assert_eq!(original.chunks.len(), 3);
        for gap in 0..3 {
            let mut entry = original.clone();
            // No CLI can create authenticated inconsistent chunk metadata.
            // Mutate the selected in-memory fixture solely to test rejection.
            entry.chunks[gap].compression_frame_digest[0] ^= 1;
            let mut emitted = Vec::new();
            let plan = recover_entries(
                &session,
                &BTreeMap::from([(path.clone(), entry)]),
                true,
                &mut |_, offset, data| {
                    emitted.push((offset, data.to_vec()));
                    Ok(())
                },
            )
            .unwrap();
            assert_eq!(plan["files"][0]["status"], "partial");
            assert_eq!(
                plan["files"][0]["missing_ranges"][0]["offset"],
                original.chunks[gap].file_offset
            );
            assert_eq!(
                plan["files"][0]["missing_ranges"][0]["length"],
                original.chunks[gap].len
            );
            for (offset, data) in emitted {
                assert_ne!(offset, original.chunks[gap].file_offset);
                assert_eq!(data, payload[offset as usize..offset as usize + data.len()]);
            }
        }
        let mut overlapping = original.clone();
        overlapping.chunks[1].file_offset = 0;
        let plan = recover_entries(
            &session,
            &BTreeMap::from([(path, overlapping)]),
            true,
            &mut |_, _, _| panic!("overlapping index must not emit bytes"),
        )
        .unwrap();
        assert_eq!(plan["files"][0]["status"], "unrecoverable");
    }
}
