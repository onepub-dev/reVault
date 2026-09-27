use super::*;
use crate::file_format::allocation_map::{self as allocation, OwnedRecord, Snapshot, Transaction};
use crate::file_format::authenticated_index::{Entry, Index};
use crate::file_format::publication_anchor::{self as publication, Authority};
use crate::storage::StorageBackend;
use crate::{EncryptionMode, LockboxFormatOptions, OwnerSigningKeyPair, SigningMode, SizePadding};
const KEY: &[u8; 32] = &[91; 32];
fn archive() -> LockboxId {
    LockboxId::from_bytes([93; 16])
}
fn mode(encrypted: bool, signed: bool, compressed: bool, padded: bool) -> FormatMode {
    FormatMode::new(LockboxFormatOptions {
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
        compression: if compressed {
            Compression::default()
        } else {
            Compression::None
        },
        size_padding: if padded {
            SizePadding::Default
        } else {
            SizePadding::None
        },
    })
}
fn key(mode: FormatMode) -> Option<&'static [u8]> {
    (!mode.plaintext()).then_some(KEY.as_slice())
}

#[test]
fn persisted_extents_round_trip_through_selected_ownership_all_modes_and_units() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                for padded in [false, true] {
                    for unit in [65536, MAX_LOGICAL] {
                        let mode = mode(encrypted, signed, compressed, padded);
                        let authority = if signed {
                            Authority::Owner(&public)
                        } else if encrypted {
                            Authority::Symmetric(KEY)
                        } else {
                            Authority::Checksum
                        };
                        let signer = signed.then_some(&owner);
                        let codec = Codec::new(archive(), mode, key(mode)).unwrap();
                        let logical_unit = codec.logical_unit(unit).unwrap();
                        let content: Vec<_> = (0..logical_unit * 2 + 37)
                            .map(|i| ((i / 71 + i / 251) % 19) as u8)
                            .collect();
                        let mut storage = StorageBackend::memory(Vec::new());
                        allocation::create_empty(
                            &mut storage,
                            archive(),
                            mode,
                            &authority,
                            signer,
                            key(mode),
                        )
                        .unwrap();
                        let mut tx =
                            Transaction::begin(storage, archive(), mode, &authority, key(mode))
                                .unwrap();
                        let mut entries = Vec::new();
                        for (ordinal, plain) in content.chunks(logical_unit).enumerate() {
                            let (descriptor, bytes) = codec
                                .encode(
                                    [7; 16],
                                    ordinal as u64,
                                    (ordinal * logical_unit) as u64,
                                    plain,
                                )
                                .unwrap();
                            if padded {
                                assert_eq!(bytes.len() % PAD_UNIT, 0);
                            }
                            if !compressed {
                                assert!(bytes.len() <= 65536);
                            }
                            if compressed && plain.len() > 256 {
                                assert_eq!(descriptor.codec, COMPRESSION_ZSTD);
                            }
                            let extent = tx.append_encoded_extent(&bytes).unwrap();
                            let envelope =
                                OwnedRecord::encode(&descriptor.encode(), &[extent]).unwrap();
                            entries.push(
                                Entry::new(2, &(ordinal as u64).to_be_bytes(), &envelope).unwrap(),
                            );
                        }
                        tx.replace_all_sorted(entries.into_iter().map(Ok)).unwrap();
                        let (storage, committed) = tx.commit(&authority, signer).unwrap();
                        // Reopen separate bytes and authenticate membership before invoking codec.
                        let reopened = StorageBackend::memory(storage.read_all().unwrap());
                        let selected = publication::select(&reopened, archive(), mode, &authority)
                            .unwrap()
                            .anchor;
                        assert_eq!(selected, committed);
                        let index = Index::new(archive(), mode, key(mode)).unwrap();
                        Snapshot::inspect(&reopened, &selected, &index)
                            .unwrap()
                            .verify_reclaimed(&reopened)
                            .unwrap();
                        let mut decoder = Codec::new(archive(), mode, key(mode)).unwrap();
                        let mut decoded = Zeroizing::new(Vec::new());
                        index
                            .visit(&reopened, selected.index, selected.sealed_len, |entry| {
                                let record = OwnedRecord::decode(&entry.value)?;
                                let descriptor = Descriptor::decode(&record.metadata)?;
                                assert_eq!(descriptor.object, [7; 16]);
                                assert_eq!(
                                    descriptor.ordinal,
                                    u64::from_be_bytes(entry.key.as_slice().try_into().unwrap())
                                );
                                assert_eq!(descriptor.offset as usize, decoded.len());
                                decoded.extend_from_slice(&decoder.load(
                                    &reopened,
                                    record.extents[0],
                                    selected.sealed_len,
                                    &descriptor,
                                )?);
                                Ok(())
                            })
                            .unwrap();
                        assert_eq!(decoded.len(), content.len());
                        assert!(*decoded == content, "encrypted={encrypted},signed={signed},compressed={compressed},padded={padded},unit={unit},first mismatch={:?}", decoded.iter().zip(&content).position(|(a,b)|a!=b));
                    }
                }
            }
        }
    }
}

fn physical(mode: FormatMode, plain: &[u8]) -> (StorageBackend, Extent, Descriptor) {
    let codec = Codec::new(archive(), mode, key(mode)).unwrap();
    let (descriptor, bytes) = codec.encode([7; 16], 3, 256, plain).unwrap();
    let mut storage = StorageBackend::memory(vec![
        0;
        super::super::preparation_journal::DATA_START
            as usize
    ]);
    let start = storage.append(&bytes).unwrap();
    (
        storage,
        Extent {
            start,
            len: bytes.len() as u64,
            digest: strong_checksum(&bytes),
        },
        descriptor,
    )
}
#[test]
fn stored_commitment_and_encryption_context_prevent_substitution() {
    let mode = mode(true, true, true, true);
    let (storage, extent, descriptor) = physical(mode, &vec![0x41; 65536]);
    let sealed = storage.len().unwrap();
    let mut wrong_key = Codec::new(archive(), mode, Some(&[12; 32])).unwrap();
    assert!(wrong_key
        .load(&storage, extent, sealed, &descriptor)
        .is_err());
    let mut other_archive = Codec::new(LockboxId::from_bytes([33; 16]), mode, key(mode)).unwrap();
    assert!(other_archive
        .load(&storage, extent, sealed, &descriptor)
        .is_err());
    let mut codec = Codec::new(archive(), mode, key(mode)).unwrap();
    let mut substitutions = Vec::new();
    let mut wrong = descriptor.clone();
    wrong.object[0] ^= 1;
    substitutions.push(wrong);
    let mut wrong = descriptor.clone();
    wrong.ordinal += 1;
    substitutions.push(wrong);
    let mut wrong = descriptor.clone();
    wrong.offset += 1;
    substitutions.push(wrong);
    let mut wrong = descriptor.clone();
    wrong.logical_len -= 1;
    substitutions.push(wrong);
    for wrong in substitutions {
        assert!(codec.load(&storage, extent, sealed, &wrong).is_err());
    }
    for offset in [0, extent.len / 2, extent.len - 1] {
        let mut damaged = storage.clone();
        let byte = damaged.read_at(extent.start + offset, 1).unwrap()[0];
        damaged
            .write_at(extent.start + offset, &[byte ^ 0x80])
            .unwrap();
        assert!(codec.load(&damaged, extent, sealed, &descriptor).is_err());
    }
    let (mut neighbour, another, _) = physical(mode, &vec![0x41; 65536]);
    let replacement = neighbour
        .read_at(another.start, another.len as usize)
        .unwrap();
    // Random nonce changes stored commitment even for identical plaintext.
    assert_ne!(strong_checksum(&replacement), extent.digest);
    neighbour.write_at(another.start, &replacement).unwrap();
    assert!(codec.load(&neighbour, extent, sealed, &descriptor).is_err());
}

#[test]
fn malformed_descriptors_padding_and_oversized_codec_windows_are_rejected() {
    let mode = mode(false, false, true, true);
    let (mut storage, extent, descriptor) = physical(mode, &vec![0x31; 65536]);
    let mut codec = Codec::new(archive(), mode, None).unwrap();
    for length in [0, 1, 63, 65, 128] {
        assert!(Descriptor::decode(&vec![0; length]).is_err());
    }
    for offset in [0, 53, 63] {
        let mut bad = descriptor.encode();
        bad[offset] ^= 1;
        assert!(Descriptor::decode(&bad).is_err());
    }
    for wrong in [
        Descriptor {
            logical_len: MAX_LOGICAL as u32 + 1,
            ..descriptor.clone()
        },
        Descriptor {
            offset: u64::MAX,
            ..descriptor.clone()
        },
        Descriptor {
            encoded_len: u32::MAX,
            ..descriptor.clone()
        },
        Descriptor {
            allocation_len: u32::MAX,
            ..descriptor.clone()
        },
    ] {
        // Empty storage would return Truncated if reached; reject the descriptor first.
        assert!(matches!(
            codec.load(
                &StorageBackend::memory(Vec::new()),
                extent,
                u64::MAX,
                &wrong
            ),
            Err(Error::CorruptRecord)
        ));
    }
    storage
        .write_at(extent.start + extent.len - 1, &[1])
        .unwrap();
    let mutated = Extent {
        digest: strong_checksum(&storage.read_at(extent.start, extent.len as usize).unwrap()),
        ..extent
    };
    assert!(codec
        .load(&storage, mutated, storage.len().unwrap(), &descriptor)
        .is_err());
    // Valid Zstd magic, no content-size field, an enormous advertised window,
    // then an empty final raw block. Recompute the outer digest deliberately to
    // test the decoder boundary, not merely the stored-byte checksum rejection.
    let hostile = [0x28, 0xb5, 0x2f, 0xfd, 0, 0xff, 1, 0, 0];
    let mut bytes = vec![0; PAD_UNIT];
    bytes[..hostile.len()].copy_from_slice(&hostile);
    storage.write_at(extent.start, &bytes).unwrap();
    let hostile_descriptor = Descriptor {
        encoded_len: hostile.len() as u32,
        logical_len: MAX_LOGICAL as u32,
        ..descriptor
    };
    let hostile_extent = Extent {
        digest: strong_checksum(&bytes),
        ..extent
    };
    assert!(codec
        .load(
            &storage,
            hostile_extent,
            storage.len().unwrap(),
            &hostile_descriptor
        )
        .is_err());
    assert!(codec.scratch.len() < 8 * 1024 * 1024);
    assert!(codec.logical_unit(32768).is_err());
    assert!(codec.encode([0; 16], 0, 0, b"invalid identity").is_err());
    assert!(codec.encode([7; 16], 0, 0, &[]).is_err());
    assert!(codec
        .encode([7; 16], 0, 0, &vec![0; MAX_LOGICAL + 1])
        .is_err());
}

#[test]
fn repeated_decoder_matches_fresh_decoder_for_independent_extent_streams() {
    let mode = mode(false, false, true, false);
    let codec = Codec::new(archive(), mode, None).unwrap();
    let mut reused = Codec::new(archive(), mode, None).unwrap();
    for ordinal in 0..3u64 {
        let plain: Vec<_> = (ordinal * 65536..(ordinal + 1) * 65536)
            .map(|i| ((i / 71 + i / 251) % 19) as u8)
            .collect();
        let (descriptor, bytes) = codec
            .encode([7; 16], ordinal, ordinal * 65536, &plain)
            .unwrap();
        let encoded = &bytes[..descriptor.encoded_len as usize];
        let ordinary = crate::compression::decode_compression_frame(
            descriptor.codec,
            encoded,
            plain.len() as u64,
        )
        .unwrap();
        let mut storage = StorageBackend::memory(vec![
            0;
            super::super::preparation_journal::DATA_START
                as usize
        ]);
        let extent = Extent {
            start: storage.append(&bytes).unwrap(),
            len: bytes.len() as u64,
            digest: strong_checksum(&bytes),
        };
        let actual = reused
            .load(&storage, extent, storage.len().unwrap(), &descriptor)
            .unwrap();
        if let Some(directory) = std::env::var_os("REVAULT_EXTENT_DIAGNOSTIC") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::write(directory.join(format!("extent-{ordinal}.expected")), &plain).unwrap();
            std::fs::write(directory.join(format!("extent-{ordinal}.zst")), encoded).unwrap();
            std::fs::write(
                directory.join(format!("extent-{ordinal}.ordinary")),
                &ordinary,
            )
            .unwrap();
            std::fs::write(
                directory.join(format!("extent-{ordinal}.bounded")),
                &*actual,
            )
            .unwrap();
        }
        assert!(
            ordinary == plain,
            "ordinary decoder mismatch at extent {ordinal}"
        );
        assert!(
            *actual == plain,
            "bounded decoder mismatch at extent {ordinal}"
        );
    }
}

#[test]
fn raw_extent_budget_includes_encryption_and_rejects_oversized_packs() {
    for encrypted in [false, true] {
        let mode = mode(encrypted, false, false, true);
        let codec = Codec::new(archive(), mode, key(mode)).unwrap();
        let unit = codec.logical_unit(MAX_LOGICAL).unwrap();
        assert!(codec.encode([7; 16], 0, 0, &vec![1; unit + 1]).is_err());
        let (descriptor, stored) = codec.encode([7; 16], 0, 0, &vec![1; unit]).unwrap();
        assert!(stored.len() <= 65536);
        let mut oversized = descriptor;
        oversized.logical_len += 4096;
        oversized.encoded_len += 4096;
        oversized.allocation_len += 65536;
        let extent = Extent {
            start: super::super::preparation_journal::DATA_START,
            len: oversized.allocation_len as u64,
            digest: [0; 32],
        };
        assert!(codec.validate_extent(extent, u64::MAX, &oversized).is_err());
    }
}

#[test]
fn packed_codec_and_padding_contexts_are_distinct_and_bounded() {
    let mode = mode(true, true, true, false);
    let mut packed = Codec::packed(archive(), mode, key(mode)).unwrap();
    let mut standalone = Codec::new(archive(), mode, key(mode)).unwrap();
    let (descriptor, bytes) = packed
        .encode([7; 16], 0, 0, b"independent fragment")
        .unwrap();
    let mut storage = StorageBackend::memory(vec![
        0;
        super::super::preparation_journal::DATA_START
            as usize
    ]);
    let start = storage.append(&bytes).unwrap();
    let extent = Extent {
        start,
        len: bytes.len() as u64,
        digest: strong_checksum(&bytes),
    };
    assert!(standalone
        .load(&storage, extent, storage.len().unwrap(), &descriptor)
        .is_err());
    assert!(
        packed.verify_pack_padding(&bytes).is_err(),
        "file ciphertext must not be accepted as empty padding"
    );
    assert_eq!(
        &**packed
            .load(&storage, extent, storage.len().unwrap(), &descriptor)
            .unwrap(),
        b"independent fragment"
    );
    let padded_mode = super::tests::mode(true, true, true, true);
    let padded = Codec::packed(archive(), padded_mode, key(padded_mode)).unwrap();
    for used in [
        1,
        65508,
        65509,
        65535,
        65536,
        65537,
        MAX_LOGICAL - 1,
        MAX_LOGICAL + 28,
    ] {
        let total = padded.pack_padded_len(used).unwrap();
        assert_eq!(total % 65536, 0);
        assert!(total <= MAX_ALLOCATION);
        let padding = padded.pack_padding(total - used).unwrap();
        padded.verify_pack_padding(&padding).unwrap();
        if !padding.is_empty() {
            assert!(standalone.verify_pack_padding(&padding).is_err());
        }
    }
}
