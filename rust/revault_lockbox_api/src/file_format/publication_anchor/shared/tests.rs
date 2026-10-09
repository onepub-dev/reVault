//! Internal component fixtures: this shared-control profile has no CLI writer.
use super::*;
use crate::storage::StorageBackend;
use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
pub(super) fn mode(encrypted: bool, signed: bool) -> FormatMode {
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
        compression: Compression::None,
        size_padding: SizePadding::Default,
    })
}
pub(super) fn fixture(mode: FormatMode) -> (StorageBackend, Anchor) {
    let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
    // Synthetic opaque stored bytes only: confidentiality/typed-catalogue encoding
    // is not established by a publication placement test.
    let private = vec![37; PRIVATE_BYTES];
    let keys = vec![41; 4096];
    for bank in [0, FAILURE_REGION] {
        storage.write_at(bank + PRIVATE_START, &private).unwrap();
        storage.write_at(bank + KEYS_START, &keys).unwrap();
    }
    let index = RootRef {
        primary: PRIVATE_START,
        mirror: FAILURE_REGION + PRIVATE_START,
        len: PRIVATE_BYTES as u64,
        digest: strong_checksum(&private),
    };
    let keys = RootRef {
        primary: KEYS_START,
        mirror: FAILURE_REGION + KEYS_START,
        len: 4096,
        digest: strong_checksum(&keys),
    };
    let anchor = Anchor {
        archive: LockboxId::from_bytes([51; 16]),
        generation: 1,
        mode,
        sealed_len: REGION_LEN as u64,
        object_root: index.digest,
        previous: [0; 32],
        index,
        allocation: RootRef::default(),
        keys,
    };
    (storage, anchor)
}
#[test]
fn shared_control_profile_is_authenticated_and_cannot_cross_decode_as_separated() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            let (mut storage, anchor) = fixture(mode(encrypted, signed));
            let authority = if signed {
                Authority::Owner(&public)
            } else if encrypted {
                Authority::Symmetric(&[53; 32])
            } else {
                Authority::Checksum
            };
            let encoded = encode_in(
                &anchor,
                &authority,
                signed.then_some(&owner),
                Layout::Shared,
            )
            .unwrap();
            for bank in [0, FAILURE_REGION] {
                storage.write_at(bank, &encoded).unwrap();
            }
            assert_eq!(
                decode_in(
                    &encoded,
                    anchor.archive,
                    anchor.mode,
                    &authority,
                    Layout::Shared
                )
                .unwrap(),
                anchor
            );
            assert!(decode(&encoded, anchor.archive, anchor.mode, &authority).is_err());
            assert!(anchor.commitment().is_err()); // Old geometry never accepts inline roots.
            assert!(commitment(&anchor).is_ok());
            assert_eq!(
                read_root(&storage, &anchor, RootRole::Private).unwrap(),
                vec![37; PRIVATE_BYTES]
            );
            // One physical bank loss removes publication and its colocated metadata.
            // The other bank independently retains the complete authenticated root.
            for damaged_bank in [0, FAILURE_REGION] {
                let mut damaged = storage.clone();
                damaged
                    .write_at(damaged_bank, &vec![0; FAILURE_REGION as usize])
                    .unwrap();
                let surviving = FAILURE_REGION - damaged_bank;
                let bytes = damaged.read_at(surviving, SLOT_LEN).unwrap();
                let selected = decode_in(
                    &bytes,
                    anchor.archive,
                    anchor.mode,
                    &authority,
                    Layout::Shared,
                )
                .unwrap();
                assert_eq!(
                    read_root(&damaged, &selected, RootRole::Private).unwrap(),
                    vec![37; PRIVATE_BYTES]
                );
                assert_eq!(
                    read_root(&damaged, &selected, RootRole::PublicKeys).unwrap(),
                    vec![41; 4096]
                );
            }
        }
    }
}
#[test]
fn shared_control_roles_exclude_anchors_journals_and_other_private_subranges() {
    let (_, anchor) = fixture(mode(true, true));
    for bad_offset in [
        0,
        8192,
        KEYS_START,
        PRIVATE_START + 1,
        65535,
        FAILURE_REGION,
        FAILURE_REGION + 8192,
    ] {
        let mut bad = anchor.clone();
        bad.index.primary = bad_offset;
        assert!(
            bad.validate_in(Layout::Shared).is_err(),
            "private root at {bad_offset}"
        );
    }
    for bad_offset in [0, 8192, PRIVATE_START, KEYS_START + 1] {
        let mut bad = anchor.clone();
        bad.keys.primary = bad_offset;
        assert!(
            bad.validate_in(Layout::Shared).is_err(),
            "public root at {bad_offset}"
        );
    }
    let mut bad = anchor.clone();
    bad.index.len += 1;
    assert!(bad.validate_in(Layout::Shared).is_err());
    let mut bad = anchor.clone();
    bad.index.mirror = bad.index.primary;
    assert!(bad.validate_in(Layout::Shared).is_err());
    let mut bad = anchor.clone();
    bad.allocation = bad.index;
    assert!(bad.validate_in(Layout::Shared).is_err());
    let mut external = anchor;
    external.index.primary = REGION_LEN as u64;
    external.index.mirror = REGION_LEN as u64 + FAILURE_REGION;
    external.sealed_len = 4 * FAILURE_REGION;
    assert!(external.validate_in(Layout::Shared).is_ok());
    assert!(
        validate_fresh_shape(&external).is_err(),
        "fresh payload-only catalogue cannot own external metadata"
    );
    external.index.primary += 1;
    assert!(external.validate_in(Layout::Shared).is_err());
}

#[test]
fn changing_the_shared_profile_bytes_does_not_preserve_publication_authentication() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for signed in [false, true] {
        let (_, mut anchor) = fixture(mode(true, signed));
        anchor.index.primary = REGION_LEN as u64;
        anchor.index.mirror = REGION_LEN as u64 + FAILURE_REGION;
        anchor.keys.primary = 4 * FAILURE_REGION;
        anchor.keys.mirror = 5 * FAILURE_REGION;
        anchor.sealed_len = 6 * FAILURE_REGION;
        // Both profiles accept these external addresses, so rejection must not
        // depend on the geometry difference.
        anchor.validate().unwrap();
        anchor.validate_in(Layout::Shared).unwrap();
        let authority = if signed {
            Authority::Owner(&public)
        } else {
            Authority::Symmetric(&[53; 32])
        };
        let mut encoded = encode_in(
            &anchor,
            &authority,
            signed.then_some(&owner),
            Layout::Shared,
        )
        .unwrap();
        assert_ne!(commitment(&anchor).unwrap(), anchor.commitment().unwrap());
        encoded[..8].copy_from_slice(super::super::MAGIC);
        encoded[8..10].copy_from_slice(&2u16.to_le_bytes());
        let checksum = strong_checksum(&encoded[..CHECKSUM_START]);
        encoded[CHECKSUM_START..].copy_from_slice(&checksum);
        assert!(parse_untrusted(&encoded, anchor.archive, anchor.mode).is_ok());
        assert!(decode(&encoded, anchor.archive, anchor.mode, &authority).is_err());
    }
}

#[test]
fn shared_control_image_bootstraps_and_decrypts_after_either_region_is_lost() {
    use super::super::bootstrap::{self, Credential};
    use crate::{key_slot::KeySlot, SecretString};
    let password = SecretString::try_from_slice(b"synthetic shared-control password").unwrap();
    let key = [75; 32];
    let slots =
        [
            KeySlot::password_bytes(1, b"synthetic shared-control password", vec![74; 16], &key)
                .unwrap(),
        ];
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for signed in [false, true] {
        let (mut storage, mut anchor) = fixture(mode(true, signed));
        let codec = catalogue::Codec::new(anchor.archive, anchor.mode, Some(&key)).unwrap();
        let plain = b"synthetic private catalogue bytes";
        let private = codec.encode(plain).unwrap();
        let keys = bootstrap::directory(anchor.archive, 1, &slots).unwrap();
        for bank in [0, FAILURE_REGION] {
            storage.write_at(bank + PRIVATE_START, &private).unwrap();
            storage.write_at(bank + KEYS_START, &keys).unwrap();
        }
        anchor.index.digest = strong_checksum(&private);
        anchor.object_root = anchor.index.digest;
        anchor.keys.digest = strong_checksum(&keys);
        let authority = if signed {
            Authority::Owner(&public)
        } else {
            Authority::Symmetric(&key)
        };
        let encoded = encode_in(
            &anchor,
            &authority,
            signed.then_some(&owner),
            Layout::Shared,
        )
        .unwrap();
        for bank in [0, FAILURE_REGION] {
            storage.write_at(bank, &encoded).unwrap();
        }
        storage.sync().unwrap();
        for lost in [None, Some(0), Some(FAILURE_REGION)] {
            let mut reopened = StorageBackend::memory(storage.read_all().unwrap());
            if let Some(offset) = lost {
                reopened
                    .write_at(offset, &vec![0; FAILURE_REGION as usize])
                    .unwrap();
            }
            let opened = bootstrap::open_in(
                &reopened,
                anchor.archive,
                anchor.mode,
                signed.then_some(&public),
                Credential::Password(&password),
                None,
                Layout::Shared,
            )
            .unwrap();
            let stored = crate::page_buffer::ZeroizingBytes::new(
                read_root(&reopened, &opened.anchor, RootRole::Private).unwrap(),
            );
            opened
                .key
                .with_bytes(|key| {
                    let decoder =
                        catalogue::Codec::new(anchor.archive, anchor.mode, Some(key)).unwrap();
                    assert_eq!(decoder.decode(&stored).unwrap().as_slice(), plain);
                })
                .unwrap();
            assert!(select(&reopened, anchor.archive, anchor.mode, &authority).is_err());
        }
    }
}

/// Synthetic counter-exhaustion fixture: authenticate the original first, then
/// rebuild both signed/checksummed anchors and their idle preparation commitment.
/// No public operation can reach u64::MAX within a bounded test.
#[allow(clippy::too_many_arguments)]
pub(crate) fn rewrite_fixture_generation(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    generation: u64,
) -> Result<Anchor> {
    let (mut anchor, _) = open_private(storage, archive, mode, authority, key)?;
    anchor.previous = commitment(&anchor)?;
    anchor.generation = generation;
    let encoded = encode_in(&anchor, authority, signer, Layout::Shared)?;
    let base = commitment(&anchor)?;
    let stub =
        crate::file_format::preparation_journal::compact::initial_stub(archive, mode, key, base)?;
    for bank in [0, FAILURE_REGION] {
        storage.write_at(bank + 8192, &stub)?;
        storage.write_at(bank, &encoded)?;
    }
    storage.sync()?;
    let (reopened, _) = open_private(storage, archive, mode, authority, key)?;
    if reopened != anchor {
        return Err(Error::CorruptRecord);
    }
    Ok(anchor)
}
/// Synthetic valid-but-different successor: there is no public shared-tree slot
/// mutation API. Preserve lineage while replacing and authenticating wrappers,
/// so resume admission must compare access state rather than lineage alone.
pub(crate) fn replace_fixture_slots(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    slots: &[crate::key_slot::KeySlot],
) -> Result<Anchor> {
    let (mut anchor, _) = open_private(storage, archive, mode, authority, key)?;
    if anchor.keys.absent() {
        return Err(Error::CorruptHeader);
    }
    let bytes = super::super::bootstrap::directory(archive, anchor.generation, slots)?;
    if bytes.len() as u64 != anchor.keys.len {
        return Err(Error::CorruptHeader);
    }
    anchor.keys.digest = strong_checksum(&bytes);
    let encoded = encode_in(&anchor, authority, signer, Layout::Shared)?;
    let base = commitment(&anchor)?;
    let stub =
        crate::file_format::preparation_journal::compact::initial_stub(archive, mode, key, base)?;
    for offset in [anchor.keys.primary, anchor.keys.mirror] {
        storage.write_at(offset, &bytes)?;
    }
    for bank in [0, FAILURE_REGION] {
        storage.write_at(bank + 8192, &stub)?;
        storage.write_at(bank, &encoded)?;
    }
    storage.sync()?;
    if open_private(storage, archive, mode, authority, key)?.0 != anchor {
        return Err(Error::CorruptRecord);
    }
    Ok(anchor)
}
