//! Explicit fragment recovery from authenticated format-4 membership.
use super::*;

impl RecoveryScanner {
    /// Salvage archive metadata and restore complete verified file contents.
    /// Only incomplete file fragments are sent to the sidecar callback.
    pub fn salvage_with_file_fragments(
        bytes: &[u8],
        key: &[u8],
        signer: &OwnerSigningKeyPair,
        emit: impl FnMut(u64, u64, &[u8]) -> Result<()>,
    ) -> Result<(Lockbox, serde_json::Value)> {
        Self::salvage_with_file_fragments_and_progress(bytes, key, signer, emit, |_, _| {})
    }

    /// Restore files with ordinal progress, without exposing names or contents.
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

    /// Inspect partial-file coverage, optionally correcting a salvaged archive.
    pub fn plan_file_fragments(
        bytes: &[u8],
        key: &[u8],
        recovered: Option<&mut Lockbox>,
        emit: impl FnMut(u64, u64, &[u8]) -> Result<()>,
    ) -> Result<serde_json::Value> {
        Self::plan_file_fragments_and_progress(bytes, key, recovered, emit, |_, _| {})
    }

    /// Inspect file coverage with ordinal progress and optional restoration.
    pub fn plan_file_fragments_and_progress(
        bytes: &[u8],
        key: &[u8],
        mut recovered: Option<&mut Lockbox>,
        mut emit: impl FnMut(u64, u64, &[u8]) -> Result<()>,
        mut progress: impl FnMut(u64, u64),
    ) -> Result<serde_json::Value> {
        let session = RecoverySession::new(bytes, key);
        let unavailable = || serde_json::json!({"recovery_directory_protocol":1,"source_format":4,"membership":"unavailable","files":[],"reason":"Authenticated file membership unavailable; legacy archive salvage only"});
        let Some((root, latest)) = header_commit_root_for_recovery(&session.scanner, bytes) else {
            return Ok(unavailable());
        };
        let Ok(entries) = decode_toc_btree_from_offset(
            &session.scanner,
            root.toc_root_offset,
            root.toc_root_commitment,
            0,
        ) else {
            return Ok(unavailable());
        };
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
            // Bound plaintext buffering to one file, never the whole archive.
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
            serde_json::json!({"recovery_directory_protocol":1,"source_format":4,"membership":"verified_commit_toc","latest_commit":latest,"complete_files":complete_files,"files":partial}),
        )
    }

    /// Stream independently authenticated file ranges and return a coverage manifest.
    /// Missing data is reported as gaps, never substituted with invented bytes.
    /// Only regular files selected by an authenticated commit TOC are considered.
    pub fn recover_file_ranges(
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
        let entries = decode_toc_btree_from_offset(
            &session.scanner,
            root.toc_root_offset,
            root.toc_root_commitment,
            0,
        )
        .map_err(|_| membership_error())?;
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
    mut emit: &mut impl FnMut(u64, u64, &[u8]) -> Result<()>,
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
                    emit_sparse_range(file_id, cursor, chunk.file_offset, &mut emit, &mut ranges)?;
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
                emit_sparse_range(file_id, cursor, size, &mut emit, &mut ranges)?;
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
        serde_json::json!({"recovery_directory_protocol":1,"source_format":4,"membership":"verified_commit_toc","latest_commit":latest,"files":files}),
    )
}

fn emit_sparse_range(
    id: u64,
    mut start: u64,
    end: u64,
    emit: &mut impl FnMut(u64, u64, &[u8]) -> Result<()>,
    ranges: &mut Vec<serde_json::Value>,
) -> Result<()> {
    // Format 4 defines authenticated TOC holes as logical zero bytes. These
    // are known sparse contents, distinct from unreadable indexed chunks.
    let zeros = vec![0; (end - start).min(4 * 1024 * 1024) as usize];
    while start < end {
        let len = (end - start).min(zeros.len() as u64) as usize;
        emit(id, start, &zeros[..len])?;
        ranges.push(
            serde_json::json!({"offset":start,"length":len,"source":"authenticated_sparse_zero"}),
        );
        start += len as u64;
    }
    Ok(())
}

fn recover_chunk(
    scanner: &PageScanner<'_>,
    entry: &TocEntry,
    chunk: &crate::file_chunk::FileChunk,
) -> Result<Zeroizing<Vec<u8>>> {
    #[cfg(any(test, feature = "native-block-layout"))]
    if chunk.block_frame.is_some() {
        return scanner
            .read_native_chunk(entry.len, chunk)
            .map(Zeroizing::new);
    }
    validate_compression_frame_lengths(chunk.compression_frame_len, chunk.compressed_len)?;
    if chunk.compressed_len > crate::constants::DEFAULT_MAX_PAGE_LOGICAL_BYTES as u64 {
        return Err(Error::CorruptRecord);
    }
    let mut stored = Zeroizing::new(vec![
        0;
        usize::try_from(chunk.compressed_len)
            .map_err(|_| Error::CorruptRecord)?
    ]);
    let mut covered: Vec<(usize, usize)> = Vec::new();
    for segment in &chunk.segments {
        let Ok(payload) = scanner.file_segment_payload_at(
            segment.page_offset,
            segment.object_id,
            segment.page_len,
        ) else {
            continue;
        };
        let Ok(decoded) = decode_compression_frame_segment_payload_view(&payload) else {
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
    let decoded = Zeroizing::new(crate::compression::decode_compression_frame(
        chunk.compression,
        &stored,
        chunk.compression_frame_len,
    )?);
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
    fn recovery_scope_can_remove_stale_partial_mirror_entries_without_leaking_permissions() {
        for root in ["/", "/managed"] {
            let mut archive = Lockbox::create(b"mirror scope fixture");
            let path = LockboxPath::new(format!("{}/stale", root.trim_end_matches('/'))).unwrap();
            archive.create_parent_dirs_for(&path).unwrap();
            archive.add_file(&path, b"stale bytes", false).unwrap();
            archive
                .create_mirror_project(
                    crate::MirrorProject {
                        name: "home".to_owned(),
                        source: "/synthetic/source".to_owned(),
                        destination: LockboxPath::new(root).unwrap(),
                        includes: Vec::new(),
                        excludes: Vec::new(),
                        missing_file_policy: crate::MirrorMissingFilePolicy::Remove,
                        strict: false,
                        host_identity: None,
                    },
                    true,
                )
                .unwrap();
            archive.commit().unwrap();
            assert!(archive.delete(&path).is_err());
            mutate_recovered_path(&mut archive, &path, |archive| archive.delete(&path)).unwrap();
            assert!(!archive.exists(&path));
            assert!(archive.add_file(&path, b"ordinary write", false).is_err());
            assert!(archive.mirror_project("home").unwrap().is_some());
        }
    }

    #[test]
    fn current_salvage_keeps_complete_files_and_emits_only_incomplete_ranges() {
        let key = b"current partial fixture";
        let signer = OwnerSigningKeyPair::generate().unwrap();
        let mut archive = Lockbox::create(key);
        archive.set_owner_signing_key(signer.try_clone().unwrap());
        let partial = LockboxPath::new("/partial").unwrap();
        let complete = LockboxPath::new("/complete").unwrap();
        archive
            .add_file(&partial, &vec![42; 3 * 1024 * 1024], false)
            .unwrap();
        archive
            .add_file_with_permissions(&complete, b"complete", 0o640, false)
            .unwrap();
        archive.commit().unwrap();
        archive.toc_entries.get_mut(&partial).unwrap().chunks[1].compression_frame_digest[0] ^= 1;
        archive.mark_toc_dirty(&partial);
        archive.commit().unwrap();
        let bytes = archive.to_bytes();
        let mut fragments = Vec::new();
        let (recovered, plan) = RecoveryScanner::salvage_with_file_fragments(
            &bytes,
            key,
            &signer,
            |id, offset, data| {
                fragments.push((id, offset, data.to_vec()));
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(recovered.get_file(&complete).unwrap(), b"complete");
        assert!(!recovered.exists(&partial));
        assert_eq!(plan["files"].as_array().unwrap().len(), 1);
        assert_eq!(plan["files"][0]["status"], "partial");
        assert_eq!(fragments, vec![(1, 0, vec![42; 2 * 1024 * 1024])]);
    }

    #[test]
    fn current_range_recovery_preserves_content_empty_files_and_authenticated_sparse_bytes() {
        let key = b"current range fixture";
        let mut archive = Lockbox::create(key);
        let path = LockboxPath::new("/sample").unwrap();
        archive.add_file(&path, b"verified payload", false).unwrap();
        archive
            .add_file(&LockboxPath::new("/empty").unwrap(), b"", false)
            .unwrap();
        archive.commit().unwrap();
        let bytes = archive.to_bytes();
        let mut recovered = Vec::new();
        let plan = RecoveryScanner::recover_file_ranges(&bytes, key, |_, _, data| {
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
        let mut ranges = Vec::new();
        let mut sparse = Vec::new();
        emit_sparse_range(
            1,
            5,
            13,
            &mut |id, offset, data| {
                assert_eq!((id, offset), (1, 5));
                sparse.extend_from_slice(data);
                Ok(())
            },
            &mut ranges,
        )
        .unwrap();
        assert_eq!(sparse, vec![0; 8]);
        assert_eq!(ranges[0]["source"], "authenticated_sparse_zero");
    }
}
