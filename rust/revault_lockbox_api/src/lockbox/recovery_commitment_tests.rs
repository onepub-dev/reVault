//! Real archive-byte/signature tests for the isolated commitment experiment.
use super::*;
use crate::file_format::recovery_commitment::{Context, Object, Root, Tree};
use crate::signing::{commit_signatures_match_keypair, verify_commit_signatures};
use crate::{Compression, Encryption, LockboxCreateOptions, LockboxProtection, SecretVec, Signing};
use sha2::{Digest, Sha256};
use std::io::Cursor;

fn stored_commitment(bytes: &[u8], entry: &TocEntry) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"revault-experiment-stored-extents-v1\0");
    for chunk in &entry.chunks {
        for segment in &chunk.segments {
            digest.update(segment.page_len.to_le_bytes());
            digest.update(&bytes[segment.page_offset as usize..][..segment.page_len as usize]);
        }
    }
    digest.finalize().into()
}

fn authenticate_selected(
    root: &Root,
    signatures: &[crate::commit_auth::CommitSignature],
    owner: &OwnerSigningKeyPair,
    selected_message_digest: [u8; 32],
) -> bool {
    // The selected digest stands for a separately authenticated publication
    // anchor. Scanning for the largest signed generation cannot supply it.
    crate::crypto::strong_checksum(&root.message()) == selected_message_digest
        && commit_signatures_match_keypair(signatures, owner)
        && verify_commit_signatures(&root.message(), signatures).is_ok()
}

#[test]
fn independent_owner_proof_recovers_real_neighbor_without_opening_damaged_snapshot() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let attacker = OwnerSigningKeyPair::generate().unwrap();
    for encrypted in [false, true] {
        for compression in [Compression::None, Compression::default()] {
            let mut archive = Lockbox::create_in_memory_with_options(LockboxCreateOptions {
                compression,
                ..LockboxCreateOptions::new(
                    if encrypted {
                        Encryption::Encrypted(LockboxProtection::ContentKey(
                            SecretVec::try_from_slice(&[47; 32]).unwrap(),
                        ))
                    } else {
                        Encryption::None
                    },
                    Signing::Owner(&owner),
                )
            })
            .unwrap();
            archive.set_worker_policy(crate::WorkerPolicy::Single);
            let keep = LockboxPath::new("/keep").unwrap();
            let lose = LockboxPath::new("/lose").unwrap();
            let data = vec![53u8; 128 * 1024];
            archive
                .add_file_from_reader(&keep, Cursor::new(&data), false)
                .unwrap();
            archive
                .add_file_from_reader(&lose, Cursor::new(vec![79u8; data.len()]), false)
                .unwrap();
            archive.commit().unwrap();
            let bytes = archive.to_bytes();
            let objects: Vec<_> = archive
                .toc_entries
                .values()
                .map(|entry| Object {
                    namespace: 1,
                    key: entry.path.as_str().as_bytes().to_vec(),
                    metadata: crate::toc_codec::TocEncoder::new([entry]).encode(),
                    logical_len: entry.len,
                    content: stored_commitment(&bytes, entry),
                })
                .collect();
            let context = Context {
                archive: *archive.lockbox_id.as_bytes(),
                sequence: archive.sequence,
                format: u64::from(archive.format_mode.0),
            };
            let tree = Tree::build(context.clone(), &objects).unwrap();
            let root = tree.root().clone();
            let proof = tree.proof(0).unwrap();
            let signatures = owner.sign(&root.message());
            let selected = crate::crypto::strong_checksum(&root.message());
            // Persist the selected commitment through the candidate publication
            // protocol. This separate fixture store exercises the protocol
            // without claiming the current archive writer emits its encoding.
            use crate::file_format::publication_anchor::{
                self as publication, Anchor, Authority, RootRef, REGION_LEN,
            };
            use crate::storage::{Storage, StorageBackend};
            let mut publication_store = StorageBackend::memory(vec![0; REGION_LEN]);
            let root_bytes = root.message();
            let primary = publication_store.append(&root_bytes).unwrap();
            let mirror = publication_store.append(&root_bytes).unwrap();
            let published_root = Anchor {
                archive: archive.lockbox_id,
                generation: 1,
                mode: archive.format_mode,
                sealed_len: publication_store.len().unwrap(),
                object_root: root.digest,
                previous: [0; 32],
                index: RootRef {
                    primary,
                    mirror,
                    len: root_bytes.len() as u64,
                    digest: crate::crypto::strong_checksum(&root_bytes),
                },
                allocation: RootRef::default(),
                keys: RootRef::default(),
            };
            let public_owner = owner.public_key();
            let authority = Authority::Owner(&public_owner);
            publication::publish(
                &mut publication_store,
                &published_root,
                &authority,
                Some(&owner),
                None,
            )
            .unwrap();
            let reopened_publication =
                StorageBackend::memory(publication_store.read_all().unwrap());
            let selected_publication = publication::select(
                &reopened_publication,
                archive.lockbox_id,
                archive.format_mode,
                &authority,
            )
            .unwrap();
            assert_eq!(selected_publication.anchor.object_root, root.digest);
            assert_eq!(
                selected_publication
                    .anchor
                    .index
                    .read_verified(&reopened_publication)
                    .unwrap(),
                root_bytes
            );

            assert!(authenticate_selected(&root, &signatures, &owner, selected));
            // An attacker can generate a valid hybrid signature with another key.
            let forged_signatures = attacker.sign(&root.message());
            assert!(verify_commit_signatures(&root.message(), &forged_signatures).is_ok());
            assert!(!authenticate_selected(
                &root,
                &forged_signatures,
                &owner,
                selected
            ));
            // Damage a different physical file allocation after publication.
            let mut damaged = bytes;
            let lost = &archive.toc_entries[&lose].chunks[0].segments[0];
            damaged[lost.page_offset as usize..][..lost.page_len as usize].fill(0);
            let key = archive.key.with_bytes(|key| key.to_vec()).unwrap();
            let scanner = PageScanner::new(&damaged, archive.lockbox_id, &key);
            assert!(read_page_file_bytes(&scanner, &archive.toc_entries[&lose]).is_err());
            let survivor = &archive.toc_entries[&keep];
            let recovered = read_page_file_bytes(&scanner, survivor).unwrap();
            assert_eq!(recovered, data);
            let mut recovered_object = objects[0].clone();
            recovered_object.content = stored_commitment(&damaged, survivor);
            root.verify(&recovered_object, &proof).unwrap();
            // Recomputing local descriptors/checksums cannot change the signed root.
            recovered_object.content[0] ^= 1;
            assert!(root.verify(&recovered_object, &proof).is_err());
            let altered = Tree::build(context, &[recovered_object, objects[1].clone()]).unwrap();
            assert!(!authenticate_selected(
                altered.root(),
                &signatures,
                &owner,
                selected
            ));
            // Even a valid owner signature on another prepared root is not evidence
            // that it was the root selected by the publication protocol.
            let prepared_signatures = owner.sign(&altered.root().message());
            assert!(
                verify_commit_signatures(&altered.root().message(), &prepared_signatures).is_ok()
            );
            assert!(!authenticate_selected(
                altered.root(),
                &prepared_signatures,
                &owner,
                selected
            ));
        }
    }
}
