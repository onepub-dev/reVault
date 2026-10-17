//! Additional diagnostics around the frozen format-2 recovery reader.
//! This does not change what the reader accepts or what salvage retains.
use super::*;

impl RecoveryScanner {
    pub fn report_with_file_details(
        bytes: &[u8],
        key: &[u8],
    ) -> (RecoveryReport, Vec<(String, String)>) {
        let mut details = Vec::new();
        let report = RecoverySession::new(bytes, key).report_with_details(Some(&mut details));
        (report, details)
    }
}

pub(super) fn failure_reason(
    bytes: &[u8],
    scanner: &PageScanner<'_>,
    entry: &TocEntry,
    error: Error,
) -> &'static str {
    let reason = match error {
        Error::InvalidKey => "payload authentication failed",
        Error::Truncated => "file data is truncated",
        Error::SecurityLimitExceeded(_) => "file exceeds recovery safety limits",
        _ => "file metadata or content failed integrity validation",
    };
    if entry.node_kind == NodeKind::File {
        file_failure_stage(bytes, scanner, entry).unwrap_or(reason)
    } else {
        reason
    }
}

/// Explain a rejected file without changing the frozen reader's decision.
/// Only invariant names leave this function, never payloads, digests or keys.
fn file_failure_stage(
    bytes: &[u8],
    scanner: &PageScanner<'_>,
    entry: &TocEntry,
) -> Option<&'static str> {
    let mut chunks = entry.chunks.clone();
    chunks.sort_by_key(|chunk| chunk.file_offset);
    let expected_len = if entry.len == 0 {
        chunks
            .iter()
            .filter_map(|chunk| chunk.file_offset.checked_add(chunk.len))
            .max()
            .unwrap_or(0)
    } else {
        entry.len
    };
    if expected_len > DEFAULT_MAX_FILE_BYTES {
        return Some("file exceeds recovery safety limit");
    }
    let mut position = 0u64;
    for chunk in &chunks {
        if chunk.file_offset != position {
            return Some("file chunk offsets are not contiguous");
        }
        if validate_compression_frame_lengths(chunk.compression_frame_len, chunk.compressed_len)
            .is_err()
        {
            return Some("compression frame lengths violate safety limits");
        }
        if chunk.compressed_len
            > crate::archive_reader::constants::DEFAULT_MAX_PAGE_LOGICAL_BYTES as u64
        {
            return Some("compressed frame exceeds recovery safety limit");
        }
        let mut stored = Zeroizing::new(vec![0u8; usize::try_from(chunk.compressed_len).ok()?]);
        for segment in &chunk.segments {
            let record = match scanner.record_object_at(segment.page_offset, segment.object_id) {
                Ok(record) => record,
                Err(Error::InvalidKey) => {
                    return Some("file segment payload authentication failed")
                }
                Err(Error::Truncated) => {
                    return Some(page_truncation_reason(bytes, segment.page_offset))
                }
                Err(_) => return Some("file segment page or object could not be read"),
            };
            let decoded = match decode_compression_frame_segment_payload_view(&record.payload) {
                Ok(decoded) => decoded,
                Err(_) => return Some("file segment payload could not be decoded"),
            };
            if decoded.manifest.as_ref().is_some_and(|manifest| {
                manifest
                    .slice_for(
                        &chunk.stored_path,
                        chunk.file_offset,
                        chunk.compression_frame_offset,
                        chunk.len,
                    )
                    .filter(|slice| slice.total_len == 0 || slice.total_len == expected_len)
                    .is_none()
            }) {
                return Some("compression frame manifest does not contain the expected file slice");
            }
            if decoded.compression_frame_id != chunk.compression_frame_id
                || decoded.compression_frame_len != chunk.compression_frame_len
                || decoded.compressed_len != chunk.compressed_len
                || decoded.compression != chunk.compression
                || decoded.compression_frame_digest != chunk.compression_frame_digest
            {
                return Some("file segment metadata disagrees with the file index");
            }
            if decoded.segment_offset != segment.segment_offset
                || decoded.data.len() as u64 != segment.segment_len
            {
                return Some("file segment offset or length disagrees with the file index");
            }
            let start = usize::try_from(segment.segment_offset).ok()?;
            let Some(end) = start.checked_add(decoded.data.len()) else {
                return Some("file segment range overflows");
            };
            if end > stored.len() {
                return Some("file segment extends beyond its compression frame");
            }
            stored[start..end].copy_from_slice(decoded.data);
        }
        if strong_checksum(stored.as_slice()) != chunk.compression_frame_digest {
            return Some("compression frame digest does not match reconstructed segments");
        }
        let decoded = match crate::archive_reader::compression::decode_compression_frame(
            chunk.compression,
            stored.as_slice(),
            chunk.compression_frame_len,
        ) {
            Ok(bytes) => Zeroizing::new(bytes),
            Err(_) => return Some("compression frame decompression failed"),
        };
        let Some(end) = chunk.compression_frame_offset.checked_add(chunk.len) else {
            return Some("file slice range overflows");
        };
        if end > decoded.len() as u64 {
            return Some("file slice extends beyond the decompressed frame");
        }
        position = position.checked_add(chunk.len)?;
        if position > expected_len {
            return Some("recovered data exceeds the declared file length");
        }
    }
    (position != expected_len).then_some("recovered data is shorter than the declared file length")
}

fn page_truncation_reason(bytes: &[u8], offset: u64) -> &'static str {
    use crate::archive_reader::page::{PAGE_HEADER_LEN, PAGE_MAGIC};
    let Ok(start) = usize::try_from(offset) else {
        return "referenced file page offset cannot be represented on this platform";
    };
    if start >= bytes.len() {
        return "referenced file page offset is beyond the archive";
    }
    let Some(header) = start
        .checked_add(PAGE_HEADER_LEN)
        .and_then(|end| bytes.get(start..end))
    else {
        return "referenced file page header extends beyond the archive";
    };
    if &header[..8] != PAGE_MAGIC {
        return "referenced file page marker is missing at an in-range offset";
    }
    let header_len = u32::from_le_bytes(header[12..16].try_into().unwrap()) as usize;
    let body_len = u32::from_le_bytes(header[44..48].try_into().unwrap()) as usize;
    if header_len
        .checked_add(body_len)
        .and_then(|len| start.checked_add(len))
        .is_none_or(|end| end > bytes.len())
    {
        return "declared file page body extends beyond the archive";
    }
    "file page decoding reported truncation despite in-range storage"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_stages_distinguish_logical_index_failures() {
        let key = b"diagnostic fixture key";
        let mut archive = Lockbox::create(key);
        let path = LockboxPath::new("/fixture.txt").unwrap();
        archive.add_file(&path, b"fixture content", false).unwrap();
        archive.commit().unwrap();
        let bytes = archive.to_bytes();
        let session = RecoverySession::new(&bytes, key);
        let (root, _) = header_commit_root_for_recovery(&session.scanner, &bytes).unwrap();
        let entries =
            decode_toc_btree_from_offset(&session.scanner, root.toc_root_offset, 0).unwrap();
        let entry = entries.get(&path).unwrap();
        assert_eq!(file_failure_stage(&bytes, &session.scanner, entry), None);
        let mut shifted = entry.clone();
        shifted.chunks[0].file_offset += 1;
        assert_eq!(
            file_failure_stage(&bytes, &session.scanner, &shifted),
            Some("file chunk offsets are not contiguous")
        );
        let mut wrong_frame = entry.clone();
        wrong_frame.chunks[0].compression_frame_digest[0] ^= 1;
        assert_eq!(
            file_failure_stage(&bytes, &session.scanner, &wrong_frame),
            Some("file segment metadata disagrees with the file index")
        );
        assert!(read_page_file_bytes(&session.scanner, &shifted).is_err());
        assert!(read_page_file_bytes(&session.scanner, &wrong_frame).is_err());
        assert_eq!(
            read_page_file_bytes(&session.scanner, entry).unwrap(),
            b"fixture content"
        );
    }

    #[test]
    fn truncation_diagnostics_distinguish_missing_storage_from_missing_markers() {
        use crate::archive_reader::page::{PAGE_HEADER_LEN, PAGE_MAGIC};
        let mut bytes = vec![0; PAGE_HEADER_LEN + 16];
        assert_eq!(
            page_truncation_reason(&bytes, bytes.len() as u64),
            "referenced file page offset is beyond the archive"
        );
        assert_eq!(
            page_truncation_reason(&bytes, 17),
            "referenced file page header extends beyond the archive"
        );
        assert_eq!(
            page_truncation_reason(&bytes, 0),
            "referenced file page marker is missing at an in-range offset"
        );
        bytes[..8].copy_from_slice(PAGE_MAGIC);
        bytes[12..16].copy_from_slice(&(PAGE_HEADER_LEN as u32).to_le_bytes());
        bytes[44..48].copy_from_slice(&17_u32.to_le_bytes());
        assert_eq!(
            page_truncation_reason(&bytes, 0),
            "declared file page body extends beyond the archive"
        );
        bytes[44..48].copy_from_slice(&16_u32.to_le_bytes());
        assert_eq!(
            page_truncation_reason(&bytes, 0),
            "file page decoding reported truncation despite in-range storage"
        );
    }
}
