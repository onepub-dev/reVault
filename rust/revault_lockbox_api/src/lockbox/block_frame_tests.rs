//! Private layout-integration tests: the public writer is not activated for
//! block pages yet. Inject encoded pages and decoded TOC leaves, then exercise
//! public read APIs. These are not persisted-lifecycle or CLI E2E tests.
use super::*;
use crate::compression_frame_manifest::CompressionFrameSlice;
use crate::file_chunk::{BlockFrameReference, CompressionFrameSegment, FileChunk};
use crate::file_format::indexed_frame::block_page::{self, PageIdentity};
use crate::{
    Compression, Encryption, LockboxCreateOptions, LockboxProtection, Signing, SizePadding,
};
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;

fn install(archive: &mut Lockbox, input: &[u8]) -> Vec<LockboxPath> {
    let identity = PageIdentity {
        archive: archive.lockbox_id,
        mode: archive.format_mode,
        page_id: archive.sequence + 100,
        sequence: archive.sequence + 1,
    };
    let split = input.len() / 2;
    let slices = [0..split, split..input.len()]
        .into_iter()
        .enumerate()
        .map(|(i, range)| CompressionFrameSlice {
            path: LockboxPath::new(format!("/file-{i}")).unwrap(),
            permissions: 0o640,
            total_len: range.len() as u64,
            file_offset: 0,
            compression_frame_offset: range.start as u64,
            len: range.len() as u64,
        })
        .collect::<Vec<_>>();
    let (descriptor, page) = archive
        .key
        .with_bytes(|key| {
            block_page::encode(identity, identity.page_id + 1, input, slices.clone(), key)
        })
        .unwrap()
        .unwrap();
    let offset = archive.storage.append(&page).unwrap();
    let reference = Arc::new(BlockFrameReference {
        descriptor,
        sequence: identity.sequence,
    });
    let entries = slices
        .into_iter()
        .map(|slice| {
            let descriptor = &reference.descriptor;
            TocEntry {
                path: slice.path.clone(),
                len: slice.len,
                record_offset: offset,
                record_len: page.len() as u64,
                record_object_id: identity.page_id,
                deleted: false,
                node_kind: crate::node_kind::NodeKind::File,
                permissions: slice.permissions,
                chunks: vec![FileChunk {
                    block_frame: Some(reference.clone()),
                    stored_path: slice.path,
                    file_offset: 0,
                    len: slice.len,
                    compression_frame_offset: slice.compression_frame_offset,
                    compression_frame_len: descriptor.logical_len,
                    compressed_len: descriptor.stored_len,
                    compression: descriptor.compression,
                    compression_frame_id: descriptor.frame_id,
                    compression_frame_digest: descriptor.index_commitment,
                    segments: vec![CompressionFrameSegment {
                        page_offset: offset,
                        page_len: page.len() as u64,
                        object_id: identity.page_id,
                        segment_offset: 0,
                        segment_len: descriptor.stored_len,
                    }],
                }],
            }
        })
        .collect::<Vec<_>>();
    let wire = crate::toc_btree::encode_toc_leaf(&entries).unwrap();
    let crate::toc_btree::TocNode::Leaf(decoded) =
        crate::toc_btree::decode_toc_node(&wire).unwrap()
    else {
        panic!("expected leaf");
    };
    let paths = decoded.iter().map(|entry| entry.path.clone()).collect();
    archive
        .toc_entries
        .extend(decoded.into_iter().map(|entry| (entry.path.clone(), entry)));
    paths
}

#[test]
fn native_block_public_read_paths_and_signed_digest_validate_all_modes() {
    let signer = OwnerSigningKeyPair::generate().unwrap();
    let input = (0..65539).map(|i| (i % 251) as u8).collect::<Vec<_>>();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compression in [Compression::None, Compression::default()] {
                for size_padding in [SizePadding::Default, SizePadding::None] {
                    let mut archive =
                        Lockbox::create_in_memory_with_options(LockboxCreateOptions {
                            compression,
                            size_padding,
                            ..LockboxCreateOptions::new(
                                if encrypted {
                                    Encryption::Encrypted(LockboxProtection::ContentKey(
                                        SecretVec::try_from_slice(&[67; 32]).unwrap(),
                                    ))
                                } else {
                                    Encryption::None
                                },
                                if signed {
                                    Signing::Owner(&signer)
                                } else {
                                    Signing::None
                                },
                            )
                        })
                        .unwrap();
                    archive.commit().unwrap();
                    let paths = install(&mut archive, &input);
                    for (i, path) in paths.iter().enumerate() {
                        let expected = if i == 0 {
                            &input[..input.len() / 2]
                        } else {
                            &input[input.len() / 2..]
                        };
                        assert_eq!(archive.get_file(path).unwrap(), expected);
                        assert_eq!(
                            archive.read_file_range(path, 16380, 13).unwrap(),
                            expected[16380..16393]
                        );
                        let mut reader = archive.open_file(path).unwrap();
                        reader.seek(SeekFrom::Start(16380)).unwrap();
                        let mut sample = [0; 13];
                        reader.read_exact(&mut sample).unwrap();
                        assert_eq!(sample, expected[16380..16393]);
                        reader.rewind().unwrap();
                        let mut content = Vec::new();
                        reader.read_to_end(&mut content).unwrap();
                        assert_eq!(content, expected);
                        let mut extracted = Vec::new();
                        archive
                            .extract_file_to_writer(path, &mut extracted)
                            .unwrap();
                        assert_eq!(extracted, expected);
                    }
                    for order in [
                        files::ContentStreamOrder::Logical,
                        files::ContentStreamOrder::Physical,
                    ] {
                        let mut content = BTreeMap::<LockboxPath, Vec<u8>>::new();
                        archive
                            .stream_content(
                                files::ContentStreamOptions { order },
                                |chunk, reader| {
                                    let bytes = content.entry(chunk.path).or_default();
                                    assert_eq!(bytes.len() as u64, chunk.file_offset);
                                    reader.read_to_end(bytes).unwrap();
                                    Ok(())
                                },
                            )
                            .unwrap();
                        assert_eq!(content[&paths[0]], input[..input.len() / 2]);
                        assert_eq!(content[&paths[1]], input[input.len() / 2..]);
                    }
                    // Canonical signing must eagerly read data, not just hash the index.
                    assert!(archive.signed_content_digest().is_ok());
                    let original = archive.toc_entries[&paths[0]].clone();
                    archive.toc_entries.get_mut(&paths[0]).unwrap().chunks[0].stored_path =
                        LockboxPath::new("/wrong").unwrap();
                    assert!(archive.get_file(&paths[0]).is_err());
                    archive
                        .toc_entries
                        .insert(paths[0].clone(), original.clone());
                    archive.toc_entries.get_mut(&paths[0]).unwrap().len += 1;
                    assert!(archive.read_file_range(&paths[0], 0, 1).is_err());
                    archive
                        .toc_entries
                        .insert(paths[0].clone(), original.clone());
                    for mutation in 0..13 {
                        let mut changed = original.clone();
                        let chunk = &mut changed.chunks[0];
                        match mutation {
                            0 => {
                                Arc::make_mut(chunk.block_frame.as_mut().unwrap())
                                    .descriptor
                                    .archive = LockboxId::from_bytes([19; 16])
                            }
                            1 => {
                                Arc::make_mut(chunk.block_frame.as_mut().unwrap())
                                    .descriptor
                                    .mode
                                    .0 ^= 0x100
                            }
                            2 => Arc::make_mut(chunk.block_frame.as_mut().unwrap()).sequence += 1,
                            3 => chunk.compression_frame_id += 1,
                            4 => chunk.compression_frame_digest[0] ^= 1,
                            5 => chunk.compressed_len += 1,
                            6 => chunk.compression_frame_len += 1,
                            7 => chunk.segments[0].object_id += 1,
                            8 => chunk.segments[0].page_len -= 1,
                            9 => chunk.segments[0].page_offset += 1,
                            10 => chunk.segments[0].segment_offset += 1,
                            11 => chunk.segments[0].segment_len += 1,
                            12 => chunk.segments.push(chunk.segments[0].clone()),
                            _ => unreachable!(),
                        }
                        archive.toc_entries.insert(paths[0].clone(), changed);
                        assert!(
                            archive.read_file_range(&paths[0], 0, 1).is_err(),
                            "accepted reference mutation {mutation}"
                        );
                    }
                    archive
                        .toc_entries
                        .insert(paths[0].clone(), original.clone());
                    let segment = &original.chunks[0].segments[0];
                    let page = &archive.to_bytes()[segment.page_offset as usize..]
                        [..segment.page_len as usize];
                    let used = crate::page::PAGE_HEADER_LEN
                        + u32::from_le_bytes(page[44..48].try_into().unwrap()) as usize;
                    let damaged = page[used - 1] ^ 1;
                    archive
                        .storage
                        .write_at(segment.page_offset + used as u64 - 1, &[damaged])
                        .unwrap();
                    if compression == Compression::None {
                        // Raw range authentication touches only selected blocks;
                        // signed snapshot verification below must still visit all.
                        assert_eq!(
                            archive.read_file_range(&paths[0], 0, 1).unwrap(),
                            input[..1]
                        );
                    }
                    assert!(archive.get_file(&paths[1]).is_err());
                    assert!(archive.signed_content_digest().is_err());
                }
            }
        }
    }
}
