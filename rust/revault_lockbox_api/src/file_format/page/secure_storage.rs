//! Secure-memory staging for the experimental segmented-value adapter.
//! Existing public writers and their wire representations are unchanged.
use super::*;

/// Retain the complete stored page in guarded memory. Plaintext mode keeps the
/// existing cleartext wire representation without ordinary plaintext scratch.
/// Protected mode reuses the established secure encoder; its returned Vec holds
/// ciphertext only. Callers must write/read back through scoped secure access.
pub(crate) fn encode_secure_storage(request: SecureSingleObjectPage<'_>) -> Result<SecureVec> {
    if !matches!(
        request.kind,
        PageObjectKind::VariableLeaf
            | PageObjectKind::VariableInternal
            | PageObjectKind::FormLeaf
            | PageObjectKind::FormInternal
    ) {
        return Err(Error::CorruptRecord);
    }
    if request.page_size != secure_page_size(request.payload.len(), request.format_mode)? {
        return Err(Error::CorruptRecord);
    }
    if !request.format_mode.plaintext() {
        return SecureVec::try_from_vec(encode_single_object_page_secure(request)?)
            .map_err(Into::into);
    }
    // Public plaintext private-tree pages use an uncompressed 16-byte body
    // header, followed by the 24-byte single-object framing and payload.
    let prefix = PAGE_HEADER_LEN + 32 + 16 + 24;
    let used = prefix
        .checked_add(request.payload.len())
        .ok_or(Error::CorruptRecord)?;
    if used > request.page_size || (request.format_mode.unpadded() && used != request.page_size) {
        return Err(Error::SecurityLimitExceeded(
            "secure page physical size".into(),
        ));
    }
    let stream_len = 24 + request.payload.len();
    let stored_len = 32 + 16 + stream_len;
    let flags = PAGE_FLAG_CLEAR_TEXT
        | if request.format_mode.unpadded() {
            PAGE_FLAG_UNPADDED
        } else {
            0
        };
    let mut page = SecureVec::new();
    page.resize_zeroed(prefix)?;
    page.with_mut_bytes(|bytes| {
        bytes[..8].copy_from_slice(PAGE_MAGIC);
        bytes[8..10].copy_from_slice(&PAGE_VERSION.to_le_bytes());
        bytes[10..12].copy_from_slice(&flags.to_le_bytes());
        bytes[12..16].copy_from_slice(&(PAGE_HEADER_LEN as u32).to_le_bytes());
        bytes[16..24].copy_from_slice(&request.page_id.to_le_bytes());
        bytes[24..32].copy_from_slice(&request.sequence.to_le_bytes());
        bytes[44..48].copy_from_slice(&(stored_len as u32).to_le_bytes());
        bytes[48..56].copy_from_slice(&(request.page_size as u64).to_le_bytes());
        let body = PAGE_HEADER_LEN + 32;
        bytes[body] = PAGE_BODY_VERSION;
        bytes[body + 1] = COMPRESSION_NONE;
        bytes[body + 4..body + 12].copy_from_slice(&(stream_len as u64).to_le_bytes());
        let object = body + 16;
        bytes[object..object + 4].copy_from_slice(&1u32.to_le_bytes());
        bytes[object + 4] = request.kind as u8;
        bytes[object + 5] = 1;
        bytes[object + 8..object + 16].copy_from_slice(&request.id.to_le_bytes());
        bytes[object + 16..object + 24]
            .copy_from_slice(&(request.payload.len() as u64).to_le_bytes());
    })?;
    page.try_extend_from_secure(request.payload)?;
    let body_digest = page.with_bytes(|bytes| strong_checksum(&bytes[PAGE_HEADER_LEN + 32..]))?;
    let header_digest = page.with_bytes(|bytes| strong_checksum(&bytes[..PAGE_CHECKSUM_START]))?;
    page.with_mut_bytes(|bytes| {
        bytes[PAGE_HEADER_LEN..PAGE_HEADER_LEN + 32].copy_from_slice(&body_digest);
        bytes[PAGE_CHECKSUM_START..PAGE_HEADER_LEN].copy_from_slice(&header_digest);
    })?;
    page.resize_zeroed(request.page_size)?;
    Ok(page)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn secure_staging_matches_plaintext_wire_and_secure_reader_all_modes() {
        use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
        for encrypted in [false, true] {
            for signed in [false, true] {
                for compression in [Compression::None, Compression::default()] {
                    for size_padding in [SizePadding::Default, SizePadding::None] {
                        let mode = crate::creation_options::FormatMode::new(LockboxFormatOptions {
                            encryption: if encrypted {
                                EncryptionMode::ChaCha20Poly1305
                            } else {
                                EncryptionMode::None
                            },
                            signing: if signed {
                                SigningMode::Owner
                            } else {
                                SigningMode::None
                            },
                            compression,
                            size_padding,
                        });
                        for length in [0, 1, 65536] {
                            let payload = SecureVec::try_from_slice(&vec![b'x'; length]).unwrap();
                            let archive = LockboxId::from_bytes([71; 16]);
                            let key = [17; 32];
                            let size = secure_page_size(length, mode).unwrap();
                            let mut page = encode_secure_storage(SecureSingleObjectPage {
                                format_mode: mode,
                                page_size: size,
                                lockbox_id: archive,
                                page_id: 51,
                                sequence: 7,
                                content_key: &key,
                                kind: PageObjectKind::VariableLeaf,
                                id: 29,
                                payload: &payload,
                            })
                            .unwrap();
                            assert_eq!(page.len(), size);
                            if !encrypted {
                                // Known synthetic reference only: compare the exact established
                                // ordinary writer bytes, including unpadded length and checksums.
                                let reference = encode_page_with_format(
                                    size,
                                    archive,
                                    51,
                                    7,
                                    &key,
                                    &[PageObject::new_secure(
                                        PageObjectKind::VariableLeaf,
                                        29,
                                        payload.try_clone().unwrap(),
                                    )],
                                    mode,
                                )
                                .unwrap();
                                page.with_bytes(|bytes| assert_eq!(bytes, reference))
                                    .unwrap();
                            }
                            let decoded = decode_single_object_page_secure_with_format(
                                &mut page, archive, &key, mode,
                            )
                            .unwrap();
                            assert_eq!((decoded.page_id, decoded.sequence), (51, 7));
                            assert_eq!(decoded.objects.len(), 1);
                            assert_eq!(decoded.objects[0].id, 29);
                            decoded.objects[0]
                                .secure_payload()
                                .unwrap()
                                .with_bytes(|bytes| assert_eq!(bytes, vec![b'x'; length]))
                                .unwrap();
                        }
                    }
                }
            }
        }
    }
}
